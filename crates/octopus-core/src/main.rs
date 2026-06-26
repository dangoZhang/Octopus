use octopus_core::{
    default_permissions, default_tentacle_profiles, inspect_tentacle_manifests,
    load_tentacle_manifests, plan_tentacle_evolution_apply, propose_tentacle_evolution_with_client,
    propose_tentacle_evolution_with_state, scaffold_tentacle, write_tentacle_apply_artifacts,
    write_tentacle_evolution_artifacts, AdaptReport, CapabilityGrant, ChatClient, ChatMessage,
    ChatRole, EnvironmentReport, EvolutionApplyArtifact, EvolutionApplyPlan, EvolutionArtifact,
    EvolutionOutcome, Feed, GoalChat, Harness, HarnessState, HeartbeatReport, Need, NeedKind,
    OpenAiCompatibleChatClient, OpenAiCompatibleConfig, SelfIterationPlan, Status, StatusReport,
    TentacleEvolutionProposal, TentacleManifestReport, TentacleScaffold,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(serde::Serialize)]
struct DemoReport {
    workspace: String,
    adapted_profiles: Vec<String>,
    adapted_tentacles: Vec<String>,
    installed_tentacles: Vec<String>,
    goal_summary: String,
    observe_summary: String,
    probe_summary: String,
    hearts: Vec<String>,
    self_iteration_mode: String,
    pet: String,
    next: Vec<String>,
}

#[derive(serde::Serialize)]
struct DoctorReport {
    state_path: String,
    state_exists: bool,
    status: StatusReport,
    environment: EnvironmentReport,
    manifest_count: usize,
    broken_manifests: Vec<String>,
    llm: DoctorLlmReport,
    pet: DoctorPetReport,
    self_iteration_mode: String,
    warnings: Vec<String>,
    next: Vec<String>,
}

#[derive(serde::Serialize)]
struct DoctorLlmReport {
    configured: bool,
    config_prefix: String,
    chat_prefix: String,
    manifest_prefix: String,
    evolve_prefix: String,
    model: Option<String>,
    base_url: Option<String>,
    api_key_present: bool,
    curl_command: String,
    curl_available: bool,
    chat_goal_refinement_enabled: bool,
    manifest_planning_enabled: bool,
    evolution_planning_enabled: bool,
    message: String,
}

#[derive(serde::Serialize)]
struct DoctorPetReport {
    path: String,
    exists: bool,
}

#[derive(Debug, serde::Serialize)]
struct PetReport {
    state: String,
    title: String,
    summary: String,
    color: String,
    fallback: String,
    path: String,
    target: String,
    exists: bool,
}

#[derive(serde::Serialize)]
struct InitReport {
    state_path: String,
    state_existed: bool,
    adapt: AdaptReport,
    installed_profiles: Vec<String>,
    installed_tentacles: Vec<String>,
    files: Vec<InitFileReport>,
    next: Vec<String>,
}

#[derive(serde::Serialize)]
struct InitFileReport {
    path: String,
    created: bool,
}

#[derive(serde::Serialize)]
struct SkillReport {
    id: String,
    name: String,
    description: String,
    source_kind: String,
    source: String,
    brain_kind: String,
    needs: Vec<String>,
    tools: Vec<String>,
    runtimes: Vec<String>,
    evolution_surfaces: Vec<String>,
    installed: bool,
    llm_ready: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Language {
    En,
    Zh,
}

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err(usage());
    }

    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!("octopus {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let mut state = PathBuf::from(".octopus/state.json");
    let mut json = false;
    let mut language = Language::En;
    let mut rest = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--state" => {
                index += 1;
                let Some(path) = args.get(index) else {
                    return Err("--state requires a path".to_string());
                };
                state = PathBuf::from(path);
            }
            "--lang" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("--lang requires en or zh".to_string());
                };
                language = parse_language(value)?;
            }
            "--json" => json = true,
            value => rest.push(value.to_string()),
        }
        index += 1;
    }

    match rest.first().map(String::as_str) {
        Some("need") => {
            let kind = rest
                .get(1)
                .ok_or_else(|| "need requires a kind".to_string())
                .and_then(|value| parse_kind(value))?;
            let query = rest
                .get(2..)
                .filter(|values| !values.is_empty())
                .map(|values| values.join(" "))
                .ok_or_else(|| "need requires a query".to_string())?;
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let mut harness = harness_for_need(loaded, &kind)?;
            let feedback = harness.feed(&[Need::new(kind, query)]);
            harness
                .state
                .save(&state)
                .map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&feedback).map_err(|error| error.to_string())?
                );
            } else {
                println!("{}", localize_summary(&feedback.summary, language));
            }
            Ok(())
        }
        Some("routes") => {
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&loaded.routes.scores)
                    .map_err(|error| error.to_string())?
            );
            Ok(())
        }
        Some("chat") => {
            let message = rest
                .get(1..)
                .filter(|values| !values.is_empty())
                .map(|values| values.join(" "))
                .ok_or_else(|| "chat requires a message".to_string())?;
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let mut harness = Harness::with_state(loaded);
            let chat = if chat_llm_enabled() {
                let mut client = chat_llm_client()?;
                harness.chat_with_client(message, &mut client)?
            } else {
                harness.chat(message)
            };
            harness
                .state
                .save(&state)
                .map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&chat).map_err(|error| error.to_string())?
                );
            } else {
                print_chat(&chat, language);
            }
            Ok(())
        }
        Some("llm") => {
            let message = rest
                .get(1..)
                .filter(|values| !values.is_empty())
                .map(|values| values.join(" "))
                .ok_or_else(|| "llm requires a message".to_string())?;
            let mut client = OpenAiCompatibleChatClient::from_env()?;
            let response = client.chat(&[
                ChatMessage::new(
                    ChatRole::System,
                    "You are an Octopus provider check. Return concise, useful output.",
                ),
                ChatMessage::new(ChatRole::User, message),
            ])?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&response).map_err(|error| error.to_string())?
                );
            } else {
                println!("{}", response.content);
            }
            Ok(())
        }
        Some("demo") => {
            let repository = rest
                .get(1)
                .map(String::as_str)
                .unwrap_or("dangoZhang/Octopus");
            let report = run_demo(repository)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_demo_report(&report, language);
            }
            Ok(())
        }
        Some("init") => {
            let root = rest
                .get(1)
                .map(PathBuf::from)
                .unwrap_or_else(default_tentacles_root);
            let report = init_workspace(state.clone(), root)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_init_report(&report, language);
            }
            Ok(())
        }
        Some("goal") => {
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&loaded.goal).map_err(|error| error.to_string())?
                );
            } else {
                print_goal(&loaded, language);
            }
            Ok(())
        }
        Some("status") => {
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let report = loaded.status_report_with_state(Some(&state));
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_status_report(&report, language);
            }
            Ok(())
        }
        Some("doctor") => {
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let report = doctor_report(&loaded, state.clone())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_doctor_report(&report, language);
            }
            Ok(())
        }
        Some("pet") => {
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let pet_state = rest.get(1).map(String::as_str).unwrap_or("auto");
            let report = pet_report_for_state(&loaded, &state, pet_state)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_pet_report(&report, language);
            }
            Ok(())
        }
        Some("beat") => {
            let memory_keep = rest
                .get(1)
                .map(|value| {
                    value
                        .parse::<usize>()
                        .map_err(|_| format!("invalid memory keep: {value}"))
                })
                .transpose()?
                .unwrap_or(200);
            let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let report = loaded.beat(memory_keep);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_heartbeat_report(&report, language);
            }
            Ok(())
        }
        Some("oauth") => {
            let provider = rest
                .get(1)
                .ok_or_else(|| "oauth requires a provider".to_string())?;
            if provider == "revoke" {
                let grant = rest
                    .get(2)
                    .ok_or_else(|| "oauth revoke requires a grant id".to_string())?;
                let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
                if !loaded.revoke_grant(grant) {
                    return Err(format!("unknown grant: {grant}"));
                }
                loaded.save(&state).map_err(|error| error.to_string())?;
                match language {
                    Language::En => println!("revoked {grant}"),
                    Language::Zh => println!("已撤销 {grant}"),
                }
                return Ok(());
            }
            let scope = rest
                .get(2)
                .ok_or_else(|| "oauth requires a scope".to_string())?;
            let permissions = rest
                .get(3..)
                .filter(|values| !values.is_empty())
                .map_or_else(
                    || default_permissions(provider),
                    |values| values.iter().map(String::from).collect(),
                );
            let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let grant = loaded.grant_oauth(provider, scope, permissions);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&grant).map_err(|error| error.to_string())?
                );
            } else {
                print_grant(&grant, language);
            }
            Ok(())
        }
        Some("self-iterate") => {
            let repository = rest
                .get(1)
                .ok_or_else(|| "self-iterate requires a repository".to_string())?;
            let objective = rest
                .get(2..)
                .filter(|values| !values.is_empty())
                .map(|values| values.join(" "));
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let plan = loaded.self_iteration_plan(repository.as_str(), objective.as_deref());
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&plan).map_err(|error| error.to_string())?
                );
            } else {
                print_self_iteration_plan(&plan, language);
            }
            Ok(())
        }
        Some("evolve") => {
            if rest.get(1).map(String::as_str) == Some("apply") {
                let tentacle_id = rest
                    .get(2)
                    .ok_or_else(|| "evolve apply requires a tentacle id".to_string())?;
                let candidate_id = rest
                    .get(3)
                    .ok_or_else(|| "evolve apply requires a candidate id".to_string())?;
                let objective = rest
                    .get(4..)
                    .filter(|values| !values.is_empty())
                    .map(|values| values.join(" "))
                    .unwrap_or_else(|| "apply evolution candidate".to_string());
                let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
                let proposal = propose_evolution_for_cli(tentacle_id, &objective, &loaded)?;
                let cwd = env::current_dir().map_err(|error| error.to_string())?;
                let artifact = write_tentacle_evolution_artifacts(&cwd, &proposal)?;
                let plan = plan_tentacle_evolution_apply(&proposal, &loaded, candidate_id)?;
                let apply_artifact = write_tentacle_apply_artifacts(cwd, &plan)?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "apply": plan,
                            "apply_artifact": apply_artifact,
                            "evolution_artifact": artifact,
                        }))
                        .map_err(|error| error.to_string())?
                    );
                } else {
                    print_evolution_apply_plan(&plan, &apply_artifact, language);
                }
                return Ok(());
            }
            if rest.get(1).map(String::as_str) == Some("score") {
                let tentacle_id = rest
                    .get(2)
                    .ok_or_else(|| "evolve score requires a tentacle id".to_string())?;
                let candidate_id = rest
                    .get(3)
                    .ok_or_else(|| "evolve score requires a candidate id".to_string())?;
                let status = rest
                    .get(4)
                    .ok_or_else(|| "evolve score requires a status".to_string())
                    .and_then(|value| parse_status(value))?;
                let summary = rest
                    .get(5..)
                    .filter(|values| !values.is_empty())
                    .map(|values| values.join(" "))
                    .unwrap_or_else(|| "recorded evolution outcome".to_string());
                let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
                let outcome = loaded.record_evolution_outcome(
                    tentacle_id.as_str(),
                    candidate_id.as_str(),
                    status,
                    summary,
                );
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&outcome).map_err(|error| error.to_string())?
                    );
                } else {
                    print_evolution_outcome(&outcome, language);
                }
                return Ok(());
            }
            let tentacle_id = rest
                .get(1)
                .ok_or_else(|| "evolve requires a tentacle id".to_string())?;
            let objective = rest
                .get(2..)
                .filter(|values| !values.is_empty())
                .map(|values| values.join(" "))
                .unwrap_or_else(|| "improve feed quality".to_string());
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let proposal = propose_evolution_for_cli(tentacle_id, &objective, &loaded)?;
            let cwd = env::current_dir().map_err(|error| error.to_string())?;
            let artifact = write_tentacle_evolution_artifacts(cwd, &proposal)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "proposal": proposal,
                        "artifact": artifact,
                    }))
                    .map_err(|error| error.to_string())?
                );
            } else {
                print_evolution_proposal(&proposal, &artifact, language);
            }
            Ok(())
        }
        Some("scaffold") => {
            let tentacle_id = rest
                .get(1)
                .ok_or_else(|| "scaffold requires a tentacle id".to_string())?;
            let runtime = rest.get(2).map(String::as_str);
            let cwd = env::current_dir().map_err(|error| error.to_string())?;
            let scaffold = scaffold_tentacle(cwd, tentacle_id, runtime)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&scaffold).map_err(|error| error.to_string())?
                );
            } else {
                print_tentacle_scaffold(&scaffold, language);
            }
            Ok(())
        }
        Some("probe") => {
            let tentacle_id = rest
                .get(1)
                .ok_or_else(|| "probe requires a tentacle id".to_string())?;
            let kind = rest
                .get(2)
                .ok_or_else(|| "probe requires a need kind".to_string())
                .and_then(|value| parse_kind(value))?;
            let query = rest
                .get(3..)
                .filter(|values| !values.is_empty())
                .map(|values| values.join(" "))
                .ok_or_else(|| "probe requires a query".to_string())?;
            let feed = probe_tentacle(default_tentacles_root(), tentacle_id, kind, query)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&feed).map_err(|error| error.to_string())?
                );
            } else {
                print_probe_feed(&feed, language);
            }
            Ok(())
        }
        Some("catalog") => {
            let profiles = default_tentacle_profiles();
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&profiles).map_err(|error| error.to_string())?
                );
            } else {
                print_catalog(&profiles, language);
            }
            Ok(())
        }
        Some("skills") => {
            let root = rest
                .get(1)
                .map(PathBuf::from)
                .unwrap_or_else(default_tentacles_root);
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let reports = skill_reports(&loaded, root)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&reports).map_err(|error| error.to_string())?
                );
            } else {
                print_skill_reports(&reports, language);
            }
            Ok(())
        }
        Some("manifests") => {
            let root = rest
                .get(1)
                .map(PathBuf::from)
                .unwrap_or_else(default_tentacles_root);
            let reports = inspect_tentacle_manifests(&root).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&reports).map_err(|error| error.to_string())?
                );
            } else {
                print_manifest_reports(&reports, language);
            }
            Ok(())
        }
        Some("env") => {
            let cwd = env::current_dir().map_err(|error| error.to_string())?;
            let report = EnvironmentReport::detect(cwd);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_environment(&report, language);
            }
            Ok(())
        }
        Some("adapt") => {
            let root = rest
                .get(1)
                .map(PathBuf::from)
                .unwrap_or_else(default_tentacles_root);
            let cwd = env::current_dir().map_err(|error| error.to_string())?;
            let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let report = loaded.adapt_environment(cwd, root);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_adapt_report(&report, language);
            }
            Ok(())
        }
        Some("install") => {
            let profile = rest
                .get(1)
                .ok_or_else(|| "install requires a profile id".to_string())?;
            let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let profile_installed = loaded.install_profile(profile).is_ok();
            let manifest_installed = loaded
                .install_manifest(default_tentacles_root(), profile)
                .ok();
            if !profile_installed && manifest_installed.is_none() {
                return Err(format!("unknown profile or tentacle manifest: {profile}"));
            }
            loaded.save(&state).map_err(|error| error.to_string())?;
            match (language, manifest_installed) {
                (Language::En, Some(tentacle)) => {
                    println!(
                        "installed {} (runtimes: {})",
                        profile,
                        tentacle.runtime_kinds.join(",")
                    )
                }
                (Language::Zh, Some(tentacle)) => {
                    println!(
                        "已安装 {} (runtimes: {})",
                        profile,
                        tentacle.runtime_kinds.join(",")
                    )
                }
                (Language::En, None) => println!("installed {profile}"),
                (Language::Zh, None) => println!("已安装 {profile}"),
            }
            Ok(())
        }
        Some("installed") => {
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "profiles": loaded.installed_profiles,
                        "tentacles": loaded.installed_tentacles,
                    }))
                    .map_err(|error| error.to_string())?
                );
            } else if loaded.installed_profiles.is_empty() && loaded.installed_tentacles.is_empty()
            {
                match language {
                    Language::En => println!("no tentacles installed"),
                    Language::Zh => println!("尚未安装触手"),
                }
            } else {
                print_installed(&loaded, language);
            }
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn parse_language(value: &str) -> Result<Language, String> {
    match value {
        "en" => Ok(Language::En),
        "zh" | "zh-CN" | "zh_Hans" => Ok(Language::Zh),
        _ => Err(format!("unknown language: {value}")),
    }
}

fn parse_kind(value: &str) -> Result<NeedKind, String> {
    match value {
        "verify" => Ok(NeedKind::Verify),
        "reproduce" => Ok(NeedKind::Reproduce),
        "compare" => Ok(NeedKind::Compare),
        "remember" => Ok(NeedKind::Remember),
        "forget" => Ok(NeedKind::Forget),
        "execute" => Ok(NeedKind::Execute),
        "recall" => Ok(NeedKind::Recall),
        "observe" => Ok(NeedKind::Observe),
        _ => Err(format!("unknown need kind: {value}")),
    }
}

fn localize_summary(summary: &str, language: Language) -> String {
    if language == Language::En {
        return summary.to_string();
    }
    if let Some(id) = summary.strip_prefix("remembered ") {
        return format!("已记住 {id}");
    }
    if let Some(count) = summary
        .strip_prefix("forgot ")
        .and_then(|value| value.strip_suffix(" memories"))
    {
        return format!("已遗忘 {count} 条记忆");
    }
    match summary {
        "nothing recalled" => "没有召回内容".to_string(),
        "no tentacle supports this need" => "没有触手支持这个需求".to_string(),
        _ => summary.to_string(),
    }
}

fn print_catalog(profiles: &[octopus_core::TentacleProfile], language: Language) {
    match language {
        Language::En => println!("Installable profiles:"),
        Language::Zh => println!("可安装配置:"),
    }
    for profile in profiles {
        println!("- {}: {}", profile.id, profile.description);
    }
}

fn print_skill_reports(reports: &[SkillReport], language: Language) {
    match language {
        Language::En => println!("Skills:"),
        Language::Zh => println!("能力:"),
    }
    if reports.is_empty() {
        match language {
            Language::En => println!("- none"),
            Language::Zh => println!("- 无"),
        }
        return;
    }
    for report in reports {
        let installed = match (language, report.installed) {
            (Language::En, true) => "installed",
            (Language::En, false) => "available",
            (Language::Zh, true) => "已安装",
            (Language::Zh, false) => "可用",
        };
        println!(
            "- {} [{}:{}; {}] needs={} tools={} runtimes={} surfaces={} :: {}",
            report.id,
            report.source_kind,
            report.source,
            installed,
            join_or_none(&report.needs),
            join_or_none(&report.tools),
            join_or_none(&report.runtimes),
            join_or_none(&report.evolution_surfaces),
            report.description
        );
    }
}

fn print_manifest_reports(reports: &[TentacleManifestReport], language: Language) {
    match language {
        Language::En => println!("Tentacle manifests:"),
        Language::Zh => println!("触手 manifest:"),
    }
    for report in reports {
        let status = if report.missing_entrypoints.is_empty() {
            "ok".to_string()
        } else {
            format!("missing {}", report.missing_entrypoints.join(","))
        };
        println!(
            "- {}: brain={}, runtime={}, tools={}, surfaces={}, status={}",
            report.id,
            report.brain_kind,
            report.runtime_kinds.join(","),
            report.tool_count,
            join_or_none(&report.evolution_surfaces),
            status
        );
    }
}

fn print_installed(state: &HarnessState, language: Language) {
    if !state.installed_profiles.is_empty() {
        match language {
            Language::En => println!("profiles: {}", state.installed_profiles.join(", ")),
            Language::Zh => println!("配置: {}", state.installed_profiles.join(", ")),
        }
    }
    if !state.installed_tentacles.is_empty() {
        let tentacles = state
            .installed_tentacles
            .iter()
            .map(|tentacle| format!("{}({})", tentacle.id, tentacle.runtime_kinds.join(",")))
            .collect::<Vec<_>>()
            .join(", ");
        match language {
            Language::En => println!("tentacles: {tentacles}"),
            Language::Zh => println!("触手: {tentacles}"),
        }
    }
}

fn print_chat(chat: &GoalChat, language: Language) {
    match language {
        Language::En => {
            println!("goal: {}", chat.goal.objective);
            println!("turn {}: {}", chat.turn.index, chat.turn.summary);
            println!("refinements: {}", chat.goal.constraints.len());
            if let Some(needs) = chat.goal.signals.get("suggested_needs") {
                println!("suggested needs: {needs}");
            }
        }
        Language::Zh => {
            println!("目标: {}", chat.goal.objective);
            println!(
                "第 {} 轮: {}",
                chat.turn.index,
                localize_summary(&chat.turn.summary, language)
            );
            println!("调整: {}", chat.goal.constraints.len());
            if let Some(needs) = chat.goal.signals.get("suggested_needs") {
                println!("建议需求: {needs}");
            }
        }
    }
}

fn print_heartbeat_report(report: &HeartbeatReport, language: Language) {
    for beat in &report.beats {
        match language {
            Language::En => println!("{}: {}", beat.name, beat.summary),
            Language::Zh => println!("{}: {}", localize_beat(&beat.name), beat.summary),
        }
    }
}

fn print_pet_report(report: &PetReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus pet");
            println!("pixel: {}", report.fallback);
            println!("state: {}", report.state);
            println!("summary: {}", report.summary);
            println!("url: {}", report.target);
            println!("exists: {}", report.exists);
        }
        Language::Zh => {
            println!("章鱼桌宠");
            println!("像素: {}", report.fallback);
            println!("状态: {}", report.state);
            println!("摘要: {}", report.summary);
            println!("入口: {}", report.target);
            println!("存在: {}", report.exists);
        }
    }
}

fn run_demo(repository: &str) -> Result<DemoReport, String> {
    let workspace = env::temp_dir().join(format!("octopus-demo-{}", unique_suffix()));
    let _ = fs::remove_dir_all(&workspace);
    fs::create_dir_all(&workspace).map_err(|error| error.to_string())?;

    let mut state = HarnessState::default();
    let cwd = env::current_dir().map_err(|error| error.to_string())?;
    let adapt = state.adapt_environment(cwd, default_tentacles_root());
    let _ = state.install_manifest(default_tentacles_root(), "swe-agent");
    let scaffold = scaffold_tentacle(&workspace, "demo-feed", Some("python"))?;
    state.install_manifest(workspace.join("tentacles"), "demo-feed")?;

    let mut harness = Harness::with_state(state);
    let chat = harness.chat("build a clean-brain agent");
    let feedback = harness.feed(&[Need::new(NeedKind::Observe, "README.md")]);
    let beat = harness.state.beat(200);
    let plan = harness
        .state
        .self_iteration_plan(repository, Some("improve usability"));
    let probe = probe_tentacle(
        workspace.join("tentacles"),
        "demo-feed",
        NeedKind::Observe,
        "README.md".to_string(),
    )?;

    let installed_tentacles = harness
        .state
        .installed_tentacles
        .iter()
        .map(|tentacle| tentacle.id.clone())
        .collect::<Vec<_>>();
    let pet = format!(
        "{}?state=harness",
        repo_root().join("docs/pet.html").to_string_lossy()
    );

    Ok(DemoReport {
        workspace: workspace.to_string_lossy().to_string(),
        adapted_profiles: adapt.installed_profiles,
        adapted_tentacles: adapt.installed_tentacles,
        installed_tentacles,
        goal_summary: chat.turn.summary,
        observe_summary: feedback.summary,
        probe_summary: probe.summary,
        hearts: beat
            .beats
            .iter()
            .map(|beat| format!("{}={}", beat.name, beat.summary))
            .collect(),
        self_iteration_mode: plan.mode,
        pet,
        next: vec![
            format!("review {}", scaffold.manifest_path),
            "install a tentacle".to_string(),
            "run need observe .".to_string(),
            "open the pet state page".to_string(),
        ],
    })
}

fn print_demo_report(report: &DemoReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus demo");
            println!("workspace: {}", report.workspace);
            println!("adapted: {}", join_or_none(&report.adapted_tentacles));
            println!("installed: {}", join_or_none(&report.installed_tentacles));
            println!("goal: {}", report.goal_summary);
            println!("observe: {}", report.observe_summary);
            println!("probe: {}", report.probe_summary);
            println!("hearts: {}", report.hearts.join(", "));
            println!("self-iteration: {}", report.self_iteration_mode);
            println!("pet: {}", report.pet);
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼 demo");
            println!("workspace: {}", report.workspace);
            println!("已适配: {}", join_or_none(&report.adapted_tentacles));
            println!("已安装: {}", join_or_none(&report.installed_tentacles));
            println!("目标: {}", localize_summary(&report.goal_summary, language));
            println!(
                "观察: {}",
                localize_summary(&report.observe_summary, language)
            );
            println!(
                "probe: {}",
                localize_summary(&report.probe_summary, language)
            );
            println!("心脏: {}", report.hearts.join(", "));
            println!("自迭代: {}", report.self_iteration_mode);
            println!("桌宠: {}", report.pet);
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

fn init_workspace(state_path: PathBuf, tentacles_root: PathBuf) -> Result<InitReport, String> {
    let state_existed = state_path.exists();
    let cwd = env::current_dir().map_err(|error| error.to_string())?;
    let mut state = HarnessState::load(&state_path).map_err(|error| error.to_string())?;
    let adapt = state.adapt_environment(&cwd, tentacles_root);
    state.save(&state_path).map_err(|error| error.to_string())?;
    let files = init_files(&state_path)?;
    let state_arg = shell_arg(&state_path.to_string_lossy());
    let mut next = vec![
        format!("octopus --state {state_arg} chat \"describe your goal\""),
        format!("octopus --state {state_arg} skills"),
        format!("octopus --state {state_arg} pet"),
        format!("octopus --state {state_arg} need observe ."),
        format!("octopus --state {state_arg} doctor"),
    ];
    if let Some(env_file) = files
        .iter()
        .find(|file| file.path.ends_with("llm.env.example"))
    {
        let example = PathBuf::from(&env_file.path);
        let target = example.with_file_name("llm.env");
        next.push(format!(
            "copy {} to {}, fill it, then source {}",
            shell_arg(&env_file.path),
            shell_arg(&target.to_string_lossy()),
            shell_arg(&target.to_string_lossy())
        ));
    }
    Ok(InitReport {
        state_path: state_path.to_string_lossy().to_string(),
        state_existed,
        adapt,
        installed_profiles: state.installed_profiles,
        installed_tentacles: state
            .installed_tentacles
            .into_iter()
            .map(|tentacle| tentacle.id)
            .collect(),
        files,
        next,
    })
}

fn init_files(state_path: &Path) -> Result<Vec<InitFileReport>, String> {
    let directory = state_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let gitignore = write_init_file(
        &directory.join(".gitignore"),
        "state.json\nllm.env\nself-iteration/\nevolution/\n",
    )?;
    let llm_env = write_init_file(&directory.join("llm.env.example"), LLM_ENV_EXAMPLE)?;
    Ok(vec![gitignore, llm_env])
}

const LLM_ENV_EXAMPLE: &str = r#"# Copy to llm.env, fill the key, then source that file.
export OCTOPUS_LLM_MODEL=gpt-4.1-mini
export OCTOPUS_LLM_BASE_URL=https://api.openai.com/v1
export OCTOPUS_LLM_API_KEY=
export OCTOPUS_CHAT_LLM=1
export OCTOPUS_LLM_MANIFEST=1
# Optional: point chat or tentacle planning at another OpenAI-compatible env prefix.
# export OCTOPUS_CHAT_LLM_PREFIX=OCTOPUS_LLM
# export OCTOPUS_MANIFEST_LLM_PREFIX=OCTOPUS_LLM
"#;

fn write_init_file(path: &Path, content: &str) -> Result<InitFileReport, String> {
    let created = !path.exists();
    if created {
        fs::write(path, content).map_err(|error| error.to_string())?;
    }
    Ok(InitFileReport {
        path: path.to_string_lossy().to_string(),
        created,
    })
}

fn print_init_report(report: &InitReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus init");
            println!(
                "state: {} ({})",
                report.state_path,
                if report.state_existed {
                    "existing"
                } else {
                    "created"
                }
            );
            println!(
                "adapted profiles: {}",
                join_or_none(&report.adapt.installed_profiles)
            );
            println!(
                "adapted tentacles: {}",
                join_or_none(&report.adapt.installed_tentacles)
            );
            println!(
                "installed profiles: {}",
                join_or_none(&report.installed_profiles)
            );
            println!(
                "installed tentacles: {}",
                join_or_none(&report.installed_tentacles)
            );
            println!("files: {}", init_file_summary(&report.files));
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼初始化");
            println!(
                "状态: {} ({})",
                report.state_path,
                if report.state_existed {
                    "已存在"
                } else {
                    "已创建"
                }
            );
            println!(
                "已适配配置: {}",
                join_or_none(&report.adapt.installed_profiles)
            );
            println!(
                "已适配触手: {}",
                join_or_none(&report.adapt.installed_tentacles)
            );
            println!("已安装配置: {}", join_or_none(&report.installed_profiles));
            println!("已安装触手: {}", join_or_none(&report.installed_tentacles));
            println!("文件: {}", init_file_summary(&report.files));
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

fn init_file_summary(files: &[InitFileReport]) -> String {
    let items = files
        .iter()
        .map(|file| {
            format!(
                "{}({})",
                file.path,
                if file.created { "created" } else { "existing" }
            )
        })
        .collect::<Vec<_>>();
    join_or_none(&items)
}

fn pet_report_for_state(
    state: &HarnessState,
    state_path: &Path,
    requested: &str,
) -> Result<PetReport, String> {
    let pet_state = if requested == "auto" {
        let status = state.status_report_with_state(Some(state_path));
        auto_pet_state(&status)
    } else {
        requested
    };
    pet_report(pet_state)
}

fn auto_pet_state(report: &StatusReport) -> &'static str {
    if report
        .goal
        .as_ref()
        .is_some_and(|goal| goal.status == octopus_core::GoalStatus::Blocked)
        || report.tentacles.is_empty()
    {
        return "blocked";
    }
    if report
        .goal
        .as_ref()
        .is_some_and(|goal| goal.status == octopus_core::GoalStatus::Satisfied)
    {
        return "success";
    }
    if report.route_count > 0 {
        return "harness";
    }
    if report.memory_count > 0 {
        return "memory";
    }
    "heartbeat"
}

fn pet_report(state: &str) -> Result<PetReport, String> {
    let (state, title, summary, color, fallback) = pet_state_info(state)?;
    let path = repo_root().join("docs/pet.html");
    let path_text = path.to_string_lossy().to_string();
    let target = format!("{}?state={state}", file_url(&path));
    Ok(PetReport {
        state: state.to_string(),
        title: title.to_string(),
        summary: summary.to_string(),
        color: color.to_string(),
        fallback: fallback.to_string(),
        target,
        exists: path.exists(),
        path: path_text,
    })
}

fn file_url(path: &Path) -> String {
    format!("file://{}", percent_encode_path(&path.to_string_lossy()))
}

fn percent_encode_path(path: &str) -> String {
    let mut encoded = String::new();
    for byte in path.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'.' | b'-' | b'_' | b'~' => {
                encoded.push(*byte as char)
            }
            value => encoded.push_str(&format!("%{value:02X}")),
        }
    }
    encoded
}

fn pet_state_info(
    state: &str,
) -> Result<
    (
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    ),
    String,
> {
    match state {
        "heartbeat" | "alive" => Ok((
            "heartbeat",
            "Heartbeat",
            "Kernel and chat loop are alive.",
            "#0f766e",
            "🟩",
        )),
        "memory" => Ok((
            "memory",
            "Memory beat",
            "Context was recalled, compacted, or forgotten.",
            "#6d5bd0",
            "🟪",
        )),
        "harness" | "route" => Ok((
            "harness",
            "Harness beat",
            "Routes or tools are adapting from feedback.",
            "#cf4d32",
            "🟥",
        )),
        "blocked" => Ok((
            "blocked",
            "Blocked",
            "The harness needs a grant or external change.",
            "#a16207",
            "🟨",
        )),
        "success" | "satisfied" => Ok((
            "success",
            "Success",
            "Feedback returned useful evidence.",
            "#16833a",
            "🟩",
        )),
        value => Err(format!(
            "unknown pet state: {value}; expected heartbeat, memory, harness, blocked, or success"
        )),
    }
}

fn skill_reports(state: &HarnessState, root: PathBuf) -> Result<Vec<SkillReport>, String> {
    let mut reports = Vec::new();
    let manifests = load_tentacle_manifests(&root).map_err(|error| error.to_string())?;
    let manifest_ids = manifests
        .iter()
        .map(|loaded| loaded.manifest.id.as_str())
        .collect::<Vec<_>>();
    for profile in default_tentacle_profiles() {
        if manifest_ids.contains(&profile.id.as_str()) {
            continue;
        }
        let mut runtimes = profile
            .tools
            .iter()
            .map(|tool| tool.implementation.kind.clone())
            .collect::<Vec<_>>();
        runtimes.sort();
        runtimes.dedup();
        let installed = state
            .installed_profiles
            .iter()
            .any(|installed| installed == &profile.id)
            || state
                .installed_tentacles
                .iter()
                .any(|installed| installed.id == profile.id);
        let evolution_surfaces = profile
            .evolution
            .surfaces
            .iter()
            .map(|surface| surface.id.clone())
            .collect::<Vec<_>>();
        for skill in profile.skills {
            reports.push(SkillReport {
                id: skill.id,
                name: skill.name,
                description: skill.description,
                source_kind: "profile".to_string(),
                source: profile.id.clone(),
                brain_kind: profile.brain.kind.clone(),
                needs: skill.needs.iter().map(need_label).collect(),
                tools: skill.tools,
                runtimes: runtimes.clone(),
                evolution_surfaces: evolution_surfaces.clone(),
                installed,
                llm_ready: profile.llm_ready,
            });
        }
    }

    for loaded in manifests {
        let mut runtimes = loaded
            .manifest
            .tools
            .iter()
            .map(|tool| tool.implementation.kind.clone())
            .collect::<Vec<_>>();
        runtimes.sort();
        runtimes.dedup();
        let tools = loaded
            .manifest
            .tools
            .iter()
            .map(|tool| tool.id.clone())
            .collect::<Vec<_>>();
        let installed = state
            .installed_tentacles
            .iter()
            .any(|installed| installed.id == loaded.manifest.id);
        let evolution_surfaces = loaded
            .manifest
            .evolution
            .surfaces
            .iter()
            .map(|surface| surface.id.clone())
            .collect::<Vec<_>>();
        for skill in loaded.manifest.skills {
            reports.push(SkillReport {
                id: skill.id.clone(),
                name: title_from_id(&skill.id),
                description: skill.description,
                source_kind: "manifest".to_string(),
                source: loaded.manifest.id.clone(),
                brain_kind: loaded.manifest.brain.kind.clone(),
                needs: skill.needs.iter().map(need_label).collect(),
                tools: tools.clone(),
                runtimes: runtimes.clone(),
                evolution_surfaces: evolution_surfaces.clone(),
                installed,
                llm_ready: loaded.manifest.brain.kind == "llm",
            });
        }
    }
    reports.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.source_kind.cmp(&right.source_kind))
            .then_with(|| left.source.cmp(&right.source))
    });
    Ok(reports)
}

fn need_label(kind: &NeedKind) -> String {
    match kind {
        NeedKind::Verify => "verify",
        NeedKind::Reproduce => "reproduce",
        NeedKind::Compare => "compare",
        NeedKind::Remember => "remember",
        NeedKind::Forget => "forget",
        NeedKind::Execute => "execute",
        NeedKind::Recall => "recall",
        NeedKind::Observe => "observe",
    }
    .to_string()
}

fn title_from_id(value: &str) -> String {
    value
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!(
                    "{}{}",
                    first.to_uppercase().collect::<String>(),
                    chars.as_str()
                ),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_arg(value: &str) -> String {
    if !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || "/._-=".contains(ch))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn localize_beat(name: &str) -> &str {
    match name {
        "heartbeat" => "心跳",
        "memory" => "记忆",
        "harness" => "harness",
        _ => name,
    }
}

fn print_status_report(report: &StatusReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus status");
            println!(
                "hearts: {}",
                report
                    .hearts
                    .iter()
                    .map(|beat| format!("{}={}", beat.name, beat.summary))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!("profiles: {}", join_or_none(&report.installed_profiles));
            println!("tentacles: {}", status_tentacles(report));
            println!("goal: {}", status_goal(report));
            println!("grants: {}", join_or_none(&report.active_grants));
            println!("warnings: {}", join_or_none(&report.warnings));
            println!("next: {}", report.next_action);
        }
        Language::Zh => {
            println!("章鱼状态");
            println!(
                "心脏: {}",
                report
                    .hearts
                    .iter()
                    .map(|beat| format!("{}={}", localize_beat(&beat.name), beat.summary))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!("配置: {}", join_or_none(&report.installed_profiles));
            println!("触手: {}", status_tentacles(report));
            println!("目标: {}", status_goal(report));
            println!("授权: {}", join_or_none(&report.active_grants));
            println!("警告: {}", join_or_none(&report.warnings));
            println!("下一步: {}", report.next_action);
        }
    }
}

fn doctor_report(state: &HarnessState, state_path: PathBuf) -> Result<DoctorReport, String> {
    let cwd = env::current_dir().map_err(|error| error.to_string())?;
    let environment = EnvironmentReport::detect(&cwd);
    let manifests =
        inspect_tentacle_manifests(default_tentacles_root()).map_err(|error| error.to_string())?;
    let broken_manifests = manifests
        .iter()
        .filter(|manifest| !manifest.missing_entrypoints.is_empty())
        .map(|manifest| {
            format!(
                "{} missing {}",
                manifest.id,
                manifest.missing_entrypoints.join(",")
            )
        })
        .collect::<Vec<_>>();
    let status = state.status_report_with_state(Some(&state_path));
    let llm = doctor_llm_report();
    let pet_path = repo_root().join("docs/pet.html");
    let pet = DoctorPetReport {
        path: pet_path.to_string_lossy().to_string(),
        exists: pet_path.exists(),
    };
    let self_iteration = state.self_iteration_plan("dangoZhang/Octopus", Some("doctor"));
    let mut warnings = status.warnings.clone();
    warnings.extend(broken_manifests.iter().cloned());
    if !llm.configured {
        warnings.push("llm provider not configured".to_string());
    } else if !llm.curl_available {
        warnings.push(format!(
            "llm curl command unavailable: {}",
            llm.curl_command
        ));
    }
    if !pet.exists {
        warnings.push("pet page missing".to_string());
    }
    warnings.sort();
    warnings.dedup();

    let mut next = vec![status.next_action.clone()];
    if !llm.configured {
        next.push("set OCTOPUS_LLM_MODEL and OCTOPUS_LLM_API_KEY".to_string());
    }
    next.push("octopus demo dangoZhang/Octopus".to_string());
    next.push(format!("open {}", pet.path));
    next.sort();
    next.dedup();

    Ok(DoctorReport {
        state_path: state_path.to_string_lossy().to_string(),
        state_exists: state_path.exists(),
        status,
        environment,
        manifest_count: manifests.len(),
        broken_manifests,
        llm,
        pet,
        self_iteration_mode: self_iteration.mode,
        warnings,
        next,
    })
}

fn doctor_llm_report() -> DoctorLlmReport {
    let config_prefix = doctor_llm_prefix();
    let chat_prefix = chat_llm_prefix();
    let manifest_prefix = manifest_llm_prefix();
    let evolve_prefix = evolve_llm_prefix();
    match OpenAiCompatibleConfig::from_env_prefix(&config_prefix) {
        Ok(config) => DoctorLlmReport {
            configured: true,
            config_prefix,
            chat_prefix,
            manifest_prefix,
            evolve_prefix,
            model: Some(config.model),
            base_url: Some(config.base_url),
            api_key_present: config.api_key.is_some(),
            curl_available: command_ready(&config.curl_command),
            curl_command: config.curl_command,
            chat_goal_refinement_enabled: chat_llm_enabled(),
            manifest_planning_enabled: manifest_llm_enabled(),
            evolution_planning_enabled: evolve_llm_enabled(),
            message: "provider env configured; run llm for live check".to_string(),
        },
        Err(error) => {
            let curl_command =
                env::var(format!("{config_prefix}_CURL")).unwrap_or_else(|_| "curl".to_string());
            DoctorLlmReport {
                configured: false,
                config_prefix,
                chat_prefix,
                manifest_prefix,
                evolve_prefix,
                model: None,
                base_url: None,
                api_key_present: false,
                curl_available: command_ready(&curl_command),
                curl_command,
                chat_goal_refinement_enabled: chat_llm_enabled(),
                manifest_planning_enabled: manifest_llm_enabled(),
                evolution_planning_enabled: evolve_llm_enabled(),
                message: error,
            }
        }
    }
}

fn print_doctor_report(report: &DoctorReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus doctor");
            println!(
                "state: {} ({})",
                report.state_path,
                if report.state_exists { "exists" } else { "new" }
            );
            println!(
                "environment: manifests={}, commands={}, recommended={}",
                join_or_none(&report.environment.manifests),
                join_or_none(&report.environment.commands),
                join_or_none(&report.environment.recommended_profiles)
            );
            println!(
                "tentacles: installed={}, manifests={}, broken={}",
                status_tentacles(&report.status),
                report.manifest_count,
                join_or_none(&report.broken_manifests)
            );
            println!("llm: {}", doctor_llm_line(&report.llm));
            println!(
                "pet: {} ({})",
                report.pet.path,
                if report.pet.exists {
                    "exists"
                } else {
                    "missing"
                }
            );
            println!("self-iteration: {}", report.self_iteration_mode);
            println!("warnings: {}", join_or_none(&report.warnings));
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼体检");
            println!(
                "状态: {} ({})",
                report.state_path,
                if report.state_exists {
                    "存在"
                } else {
                    "新建"
                }
            );
            println!(
                "环境: 项目={}, 命令={}, 推荐={}",
                join_or_none(&report.environment.manifests),
                join_or_none(&report.environment.commands),
                join_or_none(&report.environment.recommended_profiles)
            );
            println!(
                "触手: 已安装={}, manifests={}, 异常={}",
                status_tentacles(&report.status),
                report.manifest_count,
                join_or_none(&report.broken_manifests)
            );
            println!("LLM: {}", doctor_llm_line(&report.llm));
            println!(
                "桌宠: {} ({})",
                report.pet.path,
                if report.pet.exists {
                    "存在"
                } else {
                    "缺失"
                }
            );
            println!("自迭代: {}", report.self_iteration_mode);
            println!("警告: {}", join_or_none(&report.warnings));
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

fn doctor_llm_line(report: &DoctorLlmReport) -> String {
    if report.configured {
        format!(
            "configured prefix={} model={} base_url={} api_key={} curl={}({}) chat_refinement={}({}) manifest_planning={}({}) evolution_planning={}({})",
            report.config_prefix,
            report.model.as_deref().unwrap_or("none"),
            report.base_url.as_deref().unwrap_or("none"),
            if report.api_key_present {
                "present"
            } else {
                "empty"
            },
            report.curl_command,
            if report.curl_available {
                "ok"
            } else {
                "missing"
            },
            report.chat_goal_refinement_enabled,
            report.chat_prefix,
            report.manifest_planning_enabled,
            report.manifest_prefix,
            report.evolution_planning_enabled,
            report.evolve_prefix
        )
    } else {
        format!(
            "not configured prefix={} ({}) curl={}({}) chat_refinement={}({}) manifest_planning={}({}) evolution_planning={}({})",
            report.config_prefix,
            report.message,
            report.curl_command,
            if report.curl_available {
                "ok"
            } else {
                "missing"
            },
            report.chat_goal_refinement_enabled,
            report.chat_prefix,
            report.manifest_planning_enabled,
            report.manifest_prefix,
            report.evolution_planning_enabled,
            report.evolve_prefix
        )
    }
}

fn status_tentacles(report: &StatusReport) -> String {
    let items = report
        .tentacles
        .iter()
        .map(|tentacle| {
            format!(
                "{}({}; {} tools)",
                tentacle.id,
                tentacle.runtime_kinds.join(","),
                tentacle.tool_count
            )
        })
        .collect::<Vec<_>>();
    join_or_none(&items)
}

fn status_goal(report: &StatusReport) -> String {
    report.goal.as_ref().map_or_else(
        || "none".to_string(),
        |goal| format!("{} ({} refinements)", goal.objective, goal.refinements),
    )
}

fn print_goal(state: &HarnessState, language: Language) {
    let Some(goal) = &state.goal else {
        match language {
            Language::En => println!("no active goal"),
            Language::Zh => println!("没有活跃目标"),
        }
        return;
    };
    match language {
        Language::En => {
            println!("goal: {}", goal.objective);
            if !goal.constraints.is_empty() {
                println!("refinements:");
                for item in &goal.constraints {
                    println!("- {item}");
                }
            }
        }
        Language::Zh => {
            println!("目标: {}", goal.objective);
            if !goal.constraints.is_empty() {
                println!("调整:");
                for item in &goal.constraints {
                    println!("- {item}");
                }
            }
        }
    }
}

fn print_grant(grant: &CapabilityGrant, language: Language) {
    match language {
        Language::En => {
            println!("oauth grant active: {}", grant.id);
            println!("permissions: {}", grant.permissions.join(", "));
        }
        Language::Zh => {
            println!("OAuth 授权已启用: {}", grant.id);
            println!("权限: {}", grant.permissions.join(", "));
        }
    }
}

fn print_self_iteration_plan(plan: &SelfIterationPlan, language: Language) {
    match language {
        Language::En => {
            println!("repo: {}", plan.repository);
            println!("mode: {}", plan.mode);
            println!("authorized: {}", plan.authorized);
            println!("next: {}", plan.steps.join(" -> "));
            if let Some(draft) = &plan.draft {
                println!("draft branch: {}", draft.branch);
                println!("draft title: {}", draft.title);
            }
        }
        Language::Zh => {
            println!("仓库: {}", plan.repository);
            println!("模式: {}", plan.mode);
            println!("已授权: {}", plan.authorized);
            println!("下一步: {}", plan.steps.join(" -> "));
            if let Some(draft) = &plan.draft {
                println!("草稿分支: {}", draft.branch);
                println!("草稿标题: {}", draft.title);
            }
        }
    }
}

fn print_evolution_proposal(
    proposal: &TentacleEvolutionProposal,
    artifact: &EvolutionArtifact,
    language: Language,
) {
    let surfaces = proposal
        .surfaces
        .iter()
        .map(|surface| surface.id.clone())
        .collect::<Vec<_>>();
    match language {
        Language::En => {
            println!("tentacle: {}", proposal.tentacle_id);
            println!("objective: {}", proposal.objective);
            println!("generator: {}", proposal.generator);
            if !proposal.planner_summary.trim().is_empty() {
                println!("planner: {}", proposal.planner_summary);
            }
            println!("proposal: {}", artifact.proposal_path);
            println!("candidates: {}", artifact.candidates_path);
            println!("patch drafts: {}", artifact.drafts_path);
            println!("json: {}", artifact.json_path);
            println!("candidate count: {}", proposal.patch_candidates.len());
            println!("editable: {}", join_or_none(&proposal.editable));
            println!("surfaces: {}", join_or_none(&surfaces));
            println!("checks: {}", join_or_none(&proposal.checks));
        }
        Language::Zh => {
            println!("触手: {}", proposal.tentacle_id);
            println!("目标: {}", proposal.objective);
            println!("生成器: {}", proposal.generator);
            if !proposal.planner_summary.trim().is_empty() {
                println!("规划: {}", proposal.planner_summary);
            }
            println!("草案: {}", artifact.proposal_path);
            println!("候选: {}", artifact.candidates_path);
            println!("补丁草案: {}", artifact.drafts_path);
            println!("JSON: {}", artifact.json_path);
            println!("候选数量: {}", proposal.patch_candidates.len());
            println!("可编辑: {}", join_or_none(&proposal.editable));
            println!("可进化面: {}", join_or_none(&surfaces));
            println!("检查: {}", join_or_none(&proposal.checks));
        }
    }
}

fn print_evolution_apply_plan(
    plan: &EvolutionApplyPlan,
    artifact: &EvolutionApplyArtifact,
    language: Language,
) {
    match language {
        Language::En => {
            println!("tentacle: {}", plan.tentacle_id);
            println!("candidate: {}", plan.candidate_id);
            println!("status: {}", plan.status);
            println!("authorized: {}", plan.authorized);
            println!("required grant: {}", plan.required_grant);
            println!("plan: {}", artifact.plan_path);
            if let Some(path) = &artifact.patch_path {
                println!("patch: {path}");
            }
            println!("json: {}", artifact.json_path);
            println!("next: {}", join_or_none(&plan.next_steps));
        }
        Language::Zh => {
            println!("触手: {}", plan.tentacle_id);
            println!("候选: {}", plan.candidate_id);
            println!("状态: {}", plan.status);
            println!("已授权: {}", plan.authorized);
            println!("所需授权: {}", plan.required_grant);
            println!("计划: {}", artifact.plan_path);
            if let Some(path) = &artifact.patch_path {
                println!("补丁: {path}");
            }
            println!("JSON: {}", artifact.json_path);
            println!("下一步: {}", join_or_none(&plan.next_steps));
        }
    }
}

fn print_evolution_outcome(outcome: &EvolutionOutcome, language: Language) {
    match language {
        Language::En => {
            println!("tentacle: {}", outcome.tentacle_id);
            println!("candidate: {}", outcome.candidate_id);
            println!("status: {:?}", outcome.status);
            println!("score: {:.2}", outcome.score);
            println!("summary: {}", outcome.summary);
        }
        Language::Zh => {
            println!("触手: {}", outcome.tentacle_id);
            println!("候选: {}", outcome.candidate_id);
            println!("状态: {:?}", outcome.status);
            println!("分数: {:.2}", outcome.score);
            println!("摘要: {}", outcome.summary);
        }
    }
}

fn print_tentacle_scaffold(scaffold: &TentacleScaffold, language: Language) {
    match language {
        Language::En => {
            println!("tentacle: {}", scaffold.tentacle_id);
            println!("runtime: {}", scaffold.runtime);
            println!("manifest: {}", scaffold.manifest_path);
            if let Some(path) = &scaffold.tool_path {
                println!("tool: {path}");
            }
            println!("next: {}", join_or_none(&scaffold.next_steps));
        }
        Language::Zh => {
            println!("触手: {}", scaffold.tentacle_id);
            println!("runtime: {}", scaffold.runtime);
            println!("manifest: {}", scaffold.manifest_path);
            if let Some(path) = &scaffold.tool_path {
                println!("工具: {path}");
            }
            println!("下一步: {}", join_or_none(&scaffold.next_steps));
        }
    }
}

fn print_probe_feed(feed: &Feed, language: Language) {
    match language {
        Language::En => {
            println!("status: {:?}", feed.status);
            println!("summary: {}", feed.summary);
            println!("evidence: {}", feed.evidence.len());
            println!("metadata: {}", probe_metadata(feed));
        }
        Language::Zh => {
            println!("状态: {:?}", feed.status);
            println!("摘要: {}", localize_summary(&feed.summary, language));
            println!("证据: {}", feed.evidence.len());
            println!("元数据: {}", probe_metadata(feed));
        }
    }
}

fn probe_metadata(feed: &Feed) -> String {
    let keys = [
        "tentacle_brain",
        "tool",
        "runtime",
        "contract",
        "plan_source",
        "returncode",
    ];
    let values = keys
        .iter()
        .filter_map(|key| {
            feed.metadata
                .get(*key)
                .map(|value| format!("{key}={value}"))
        })
        .collect::<Vec<_>>();
    join_or_none(&values)
}

fn print_environment(report: &EnvironmentReport, language: Language) {
    match language {
        Language::En => {
            println!("Environment:");
            println!("manifests: {}", report.manifests.join(", "));
            println!("commands: {}", report.commands.join(", "));
            println!("recommended: {}", report.recommended_profiles.join(", "));
        }
        Language::Zh => {
            println!("环境:");
            println!("项目特征: {}", report.manifests.join(", "));
            println!("可用命令: {}", report.commands.join(", "));
            println!("推荐触手: {}", report.recommended_profiles.join(", "));
        }
    }
}

fn print_adapt_report(report: &AdaptReport, language: Language) {
    match language {
        Language::En => {
            println!(
                "adapted profiles: {}",
                join_or_none(&report.installed_profiles)
            );
            println!(
                "adapted tentacles: {}",
                join_or_none(&report.installed_tentacles)
            );
            println!(
                "recommended: {}",
                report.environment.recommended_profiles.join(", ")
            );
        }
        Language::Zh => {
            println!("已适配配置: {}", join_or_none(&report.installed_profiles));
            println!("已适配触手: {}", join_or_none(&report.installed_tentacles));
            println!(
                "推荐触手: {}",
                report.environment.recommended_profiles.join(", ")
            );
        }
    }
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn default_tentacles_root() -> PathBuf {
    let cwd_root = env::current_dir()
        .ok()
        .map(|cwd| cwd.join("tentacles"))
        .filter(|path| path.exists());
    cwd_root.unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles")
    })
}

fn repo_root() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::canonicalize(&root).unwrap_or(root)
}

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{}-{nanos}", std::process::id())
}

fn command_ready(command: &str) -> bool {
    Command::new(command).arg("--version").output().is_ok()
}

fn probe_tentacle(
    root: PathBuf,
    tentacle_id: &str,
    kind: NeedKind,
    query: String,
) -> Result<Feed, String> {
    let mut state = HarnessState::default();
    state.install_manifest(root, tentacle_id)?;
    let mut harness = Harness::with_state(state);
    Ok(harness.feed_one(&Need::new(kind, query)))
}

fn harness_for_need(state: HarnessState, kind: &NeedKind) -> Result<Harness, String> {
    if matches!(
        kind,
        NeedKind::Remember | NeedKind::Recall | NeedKind::Forget
    ) {
        return Ok(Harness::with_state(state));
    }
    if manifest_llm_enabled() {
        return Ok(Harness::with_state_and_manifest_llm(
            state,
            manifest_llm_config()?,
        ));
    }
    Ok(Harness::with_state(state))
}

fn propose_evolution_for_cli(
    tentacle_id: &str,
    objective: &str,
    state: &HarnessState,
) -> Result<TentacleEvolutionProposal, String> {
    if evolve_llm_enabled() {
        let mut client = OpenAiCompatibleChatClient::new(OpenAiCompatibleConfig::from_env_prefix(
            &evolve_llm_prefix(),
        )?);
        return propose_tentacle_evolution_with_client(
            default_tentacles_root(),
            tentacle_id,
            objective,
            state,
            &mut client,
        );
    }
    propose_tentacle_evolution_with_state(default_tentacles_root(), tentacle_id, objective, state)
}

fn chat_llm_client() -> Result<OpenAiCompatibleChatClient, String> {
    Ok(OpenAiCompatibleChatClient::new(
        OpenAiCompatibleConfig::from_env_prefix(&chat_llm_prefix())?,
    ))
}

fn manifest_llm_config() -> Result<OpenAiCompatibleConfig, String> {
    OpenAiCompatibleConfig::from_env_prefix(&manifest_llm_prefix())
}

fn doctor_llm_prefix() -> String {
    if chat_llm_enabled() {
        chat_llm_prefix()
    } else if manifest_llm_enabled() {
        manifest_llm_prefix()
    } else if evolve_llm_enabled() {
        evolve_llm_prefix()
    } else {
        "OCTOPUS_LLM".to_string()
    }
}

fn chat_llm_prefix() -> String {
    env::var("OCTOPUS_CHAT_LLM_PREFIX").unwrap_or_else(|_| "OCTOPUS_LLM".to_string())
}

fn manifest_llm_prefix() -> String {
    env::var("OCTOPUS_MANIFEST_LLM_PREFIX").unwrap_or_else(|_| "OCTOPUS_LLM".to_string())
}

fn evolve_llm_prefix() -> String {
    env::var("OCTOPUS_EVOLVE_LLM_PREFIX").unwrap_or_else(|_| "OCTOPUS_LLM".to_string())
}

fn manifest_llm_enabled() -> bool {
    env::var("OCTOPUS_LLM_MANIFEST")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
}

fn chat_llm_enabled() -> bool {
    env::var("OCTOPUS_CHAT_LLM")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
}

fn evolve_llm_enabled() -> bool {
    env::var("OCTOPUS_LLM_EVOLVE")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
}

fn usage() -> String {
    "usage: octopus [--version] [--state path] [--lang en|zh] [--json] init [tentacles-root] | need <kind> <query> | chat <message> | llm <message> | demo [repo] | goal | status | doctor | pet [state] | beat [memory_keep] | oauth <provider> <scope> [permissions...] | oauth revoke <grant> | self-iterate <repo> | evolve <tentacle> <objective> | evolve apply <tentacle> <candidate> [objective] | evolve score <tentacle> <candidate> <status> [summary] | scaffold <tentacle> [runtime] | probe <tentacle> <kind> <query> | routes | catalog | skills [root] | manifests [root] | env | adapt [root] | install <profile> | installed".to_string()
}

fn parse_status(value: &str) -> Result<Status, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "satisfied" | "success" | "ok" => Ok(Status::Satisfied),
        "partial" => Ok(Status::Partial),
        "failed" | "fail" | "error" => Ok(Status::Failed),
        "unsupported" | "skip" => Ok(Status::Unsupported),
        _ => Err(format!(
            "unknown status: {value}; expected satisfied, partial, failed, or unsupported"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        localize_summary, percent_encode_path, pet_report, pet_report_for_state, run,
        skill_reports, usage, Language,
    };
    use octopus_core::{Feed, Goal, GoalStatus, HarnessState, Need, NeedKind};
    use std::fs;
    use std::path::Path;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_guard() -> MutexGuard<'static, ()> {
        ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    fn restore_env(key: &str, value: Option<String>) {
        if let Some(value) = value {
            std::env::set_var(key, value);
        } else {
            std::env::remove_var(key);
        }
    }

    struct CwdGuard {
        original: std::path::PathBuf,
    }

    impl CwdGuard {
        fn new() -> Self {
            Self {
                original: std::env::current_dir().unwrap(),
            }
        }
    }

    impl Drop for CwdGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original);
        }
    }

    #[test]
    fn cli_version_command_runs() {
        run(vec!["--version".to_string()]).unwrap();
        assert!(usage().contains("--version"));
    }

    #[test]
    fn cli_persists_memory_between_runs() {
        let _env = env_guard();
        let path =
            std::env::temp_dir().join(format!("octopus-cli-state-{}.json", std::process::id()));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "need".to_string(),
            "remember".to_string(),
            "clean".to_string(),
            "brain".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state,
            "need".to_string(),
            "recall".to_string(),
            "clean".to_string(),
        ])
        .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("clean brain"));
        assert!(content.contains("recall:memory"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_beat_compacts_memory_and_saves_state() {
        let _env = env_guard();
        let path =
            std::env::temp_dir().join(format!("octopus-beat-state-{}.json", std::process::id()));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "need".to_string(),
            "remember".to_string(),
            "first".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "need".to_string(),
            "remember".to_string(),
            "second".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "beat".to_string(),
            "1".to_string(),
        ])
        .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(!content.contains("first"));
        assert!(content.contains("second"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_rejects_unknown_need_kind() {
        let _env = env_guard();
        let error = run(vec![
            "need".to_string(),
            "unknown".to_string(),
            "query".to_string(),
        ])
        .unwrap_err();

        assert!(error.contains("unknown need kind"));
    }

    #[test]
    fn cli_localizes_builtin_feedback() {
        assert_eq!(localize_summary("remembered m1", Language::Zh), "已记住 m1");
        assert_eq!(
            localize_summary("forgot 2 memories", Language::Zh),
            "已遗忘 2 条记忆"
        );
        assert_eq!(
            localize_summary("nothing recalled", Language::Zh),
            "没有召回内容"
        );
    }

    #[test]
    fn cli_catalog_and_env_commands_run() {
        let _env = env_guard();
        run(vec!["catalog".to_string()]).unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles")
            .to_string_lossy()
            .to_string();
        run(vec!["skills".to_string(), root.clone()]).unwrap();
        run(vec![
            "--json".to_string(),
            "skills".to_string(),
            root.clone(),
        ])
        .unwrap();
        run(vec!["manifests".to_string(), root]).unwrap();
        run(vec![
            "--lang".to_string(),
            "zh".to_string(),
            "env".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn cli_adapts_environment_into_state() {
        let _env = env_guard();
        let path =
            std::env::temp_dir().join(format!("octopus-adapt-state-{}.json", std::process::id()));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "adapt".to_string(),
        ])
        .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("installed_profiles"));
        assert!(content.contains("installed_tentacles"));
        assert!(content.contains("visual"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_init_creates_state_files_and_adapts_workspace() {
        let _env = env_guard();
        let dir = std::env::temp_dir().join(format!("octopus-cli-init-{}", std::process::id()));
        let state_path = dir.join("state.json");
        let state = state_path.to_string_lossy().to_string();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles")
            .to_string_lossy()
            .to_string();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        run(vec![
            "--state".to_string(),
            state.clone(),
            "init".to_string(),
            root.clone(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state,
            "--json".to_string(),
            "init".to_string(),
            root,
        ])
        .unwrap();

        let state = fs::read_to_string(state_path).unwrap();
        assert!(state.contains("installed_profiles"));
        assert!(state.contains("visual"));
        assert!(dir.join(".gitignore").exists());
        let env_example = fs::read_to_string(dir.join("llm.env.example")).unwrap();
        assert!(env_example.contains("OCTOPUS_LLM_MANIFEST"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn skills_report_profiles_manifests_and_install_status() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();
        let before = skill_reports(&state, root.clone()).unwrap();
        assert!(before.iter().any(|skill| skill.id == "swe-workflow"));
        assert!(before
            .iter()
            .any(|skill| skill.id == "computer-use" && skill.source_kind == "manifest"));
        assert!(before.iter().any(|skill| {
            skill.id == "json-feed"
                && skill.source_kind == "manifest"
                && skill.runtimes.contains(&"python".to_string())
        }));

        state.install_manifest(root.clone(), "swe-agent").unwrap();
        state.install_manifest(root.clone(), "json-feed").unwrap();
        let after = skill_reports(&state, root).unwrap();
        assert!(after.iter().any(|skill| {
            skill.id == "swe-workflow" && skill.source == "swe-agent" && skill.installed
        }));
        assert!(after
            .iter()
            .any(|skill| skill.id == "json-feed" && skill.installed));
    }

    #[test]
    fn pet_report_maps_state_to_local_target() {
        let report = pet_report("route").unwrap();

        assert_eq!(report.state, "harness");
        assert_eq!(report.fallback, "🟥");
        assert!(report.target.starts_with("file://"));
        assert!(report.target.contains("docs/pet.html?state=harness"));
        assert!(report.exists);
        assert_eq!(
            percent_encode_path("/tmp/Octopus Pet.html"),
            "/tmp/Octopus%20Pet.html"
        );
        assert!(pet_report("unknown")
            .unwrap_err()
            .contains("unknown pet state"));
    }

    #[test]
    fn pet_auto_state_tracks_harness_state() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let state_path = Path::new("state.json");
        let mut state = HarnessState::default();

        assert_eq!(
            pet_report_for_state(&state, state_path, "auto")
                .unwrap()
                .state,
            "blocked"
        );

        state.install_manifest(root, "swe-agent").unwrap();
        assert_eq!(
            pet_report_for_state(&state, state_path, "auto")
                .unwrap()
                .state,
            "heartbeat"
        );

        state.memory.remember("use memory beat");
        assert_eq!(
            pet_report_for_state(&state, state_path, "auto")
                .unwrap()
                .state,
            "memory"
        );

        let mut feed = Feed::satisfied(
            &Need::new(NeedKind::Observe, "repo state"),
            "observed repo",
            "swe-agent",
        );
        feed.metadata
            .insert("tentacle".to_string(), "swe-agent".to_string());
        state.routes.learn(&feed);
        assert_eq!(
            pet_report_for_state(&state, state_path, "auto")
                .unwrap()
                .state,
            "harness"
        );

        let mut goal = Goal::new("ship octopus");
        goal.status = GoalStatus::Satisfied;
        state.goal = Some(goal);
        assert_eq!(
            pet_report_for_state(&state, state_path, "auto")
                .unwrap()
                .state,
            "success"
        );

        state.goal.as_mut().unwrap().status = GoalStatus::Blocked;
        assert_eq!(
            pet_report_for_state(&state, state_path, "auto")
                .unwrap()
                .state,
            "blocked"
        );
    }

    #[test]
    fn cli_status_and_doctor_commands_run() {
        let _env = env_guard();
        let path =
            std::env::temp_dir().join(format!("octopus-status-state-{}.json", std::process::id()));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "pet".to_string(),
            "memory".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "pet".to_string(),
            "success".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "status".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--lang".to_string(),
            "zh".to_string(),
            "doctor".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "doctor".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "install".to_string(),
            "swe-agent".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "status".to_string(),
        ])
        .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("swe-agent"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_demo_runs_end_to_end() {
        let _env = env_guard();
        run(vec!["demo".to_string(), "dangoZhang/Octopus".to_string()]).unwrap();
        run(vec![
            "--json".to_string(),
            "demo".to_string(),
            "dangoZhang/Octopus".to_string(),
        ])
        .unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn cli_llm_uses_openai_compatible_env() {
        use std::os::unix::fs::PermissionsExt;

        let _env = env_guard();
        let dir = std::env::temp_dir().join(format!("octopus-cli-llm-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let curl = dir.join("fake-curl.sh");
        fs::write(
            &curl,
            "#!/bin/sh\nprintf '%s' '{\"choices\":[{\"message\":{\"content\":\"llm ok\"}}]}'\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&curl).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&curl, permissions).unwrap();

        let old_model = std::env::var("OCTOPUS_LLM_MODEL").ok();
        let old_base_url = std::env::var("OCTOPUS_LLM_BASE_URL").ok();
        let old_api_key = std::env::var("OCTOPUS_LLM_API_KEY").ok();
        let old_timeout = std::env::var("OCTOPUS_LLM_TIMEOUT").ok();
        let old_curl = std::env::var("OCTOPUS_LLM_CURL").ok();
        std::env::set_var("OCTOPUS_LLM_MODEL", "test-model");
        std::env::set_var("OCTOPUS_LLM_BASE_URL", "https://llm.example/v1");
        std::env::remove_var("OCTOPUS_LLM_API_KEY");
        std::env::set_var("OCTOPUS_LLM_TIMEOUT", "1");
        std::env::set_var("OCTOPUS_LLM_CURL", curl.to_string_lossy().to_string());

        run(vec![
            "--json".to_string(),
            "llm".to_string(),
            "hello".to_string(),
        ])
        .unwrap();

        restore_env("OCTOPUS_LLM_MODEL", old_model);
        restore_env("OCTOPUS_LLM_BASE_URL", old_base_url);
        restore_env("OCTOPUS_LLM_API_KEY", old_api_key);
        restore_env("OCTOPUS_LLM_TIMEOUT", old_timeout);
        restore_env("OCTOPUS_LLM_CURL", old_curl);
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn cli_chat_can_use_llm_goal_refiner() {
        use std::os::unix::fs::PermissionsExt;

        let _env = env_guard();
        let dir = std::env::temp_dir().join(format!("octopus-cli-chat-llm-{}", std::process::id()));
        let state_path = dir.join("state.json");
        let state = state_path.to_string_lossy().to_string();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let curl = dir.join("fake-curl.sh");
        fs::write(
            &curl,
            r#"#!/bin/sh
printf '%s' '{"choices":[{"message":{"content":"{\"objective\":\"build Octopus\",\"constraints\":[\"keep tools outside brain\"],\"summary\":\"chat llm refined\",\"needs\":[{\"kind\":\"observe\",\"query\":\"inspect docs\"}]}"}}]}'
"#,
        )
        .unwrap();
        let mut permissions = fs::metadata(&curl).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&curl, permissions).unwrap();

        let old_chat = std::env::var("OCTOPUS_CHAT_LLM").ok();
        let old_prefix = std::env::var("OCTOPUS_CHAT_LLM_PREFIX").ok();
        let old_model = std::env::var("OCTOPUS_CHAT_TEST_MODEL").ok();
        let old_base_url = std::env::var("OCTOPUS_CHAT_TEST_BASE_URL").ok();
        let old_api_key = std::env::var("OCTOPUS_CHAT_TEST_API_KEY").ok();
        let old_curl = std::env::var("OCTOPUS_CHAT_TEST_CURL").ok();
        std::env::set_var("OCTOPUS_CHAT_LLM", "1");
        std::env::set_var("OCTOPUS_CHAT_LLM_PREFIX", "OCTOPUS_CHAT_TEST");
        std::env::set_var("OCTOPUS_CHAT_TEST_MODEL", "test-model");
        std::env::set_var("OCTOPUS_CHAT_TEST_BASE_URL", "https://llm.example/v1");
        std::env::remove_var("OCTOPUS_CHAT_TEST_API_KEY");
        std::env::set_var("OCTOPUS_CHAT_TEST_CURL", curl.to_string_lossy().to_string());

        run(vec![
            "--state".to_string(),
            state.clone(),
            "chat".to_string(),
            "make".to_string(),
            "tools".to_string(),
            "think".to_string(),
        ])
        .unwrap();

        restore_env("OCTOPUS_CHAT_LLM", old_chat);
        restore_env("OCTOPUS_CHAT_LLM_PREFIX", old_prefix);
        restore_env("OCTOPUS_CHAT_TEST_MODEL", old_model);
        restore_env("OCTOPUS_CHAT_TEST_BASE_URL", old_base_url);
        restore_env("OCTOPUS_CHAT_TEST_API_KEY", old_api_key);
        restore_env("OCTOPUS_CHAT_TEST_CURL", old_curl);
        let content = fs::read_to_string(&state_path).unwrap();
        assert!(content.contains("chat llm refined"));
        assert!(content.contains("observe: inspect docs"));
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn cli_need_can_use_manifest_llm_planning() {
        use std::os::unix::fs::PermissionsExt;

        let _env = env_guard();
        let dir =
            std::env::temp_dir().join(format!("octopus-cli-manifest-llm-{}", std::process::id()));
        let state_path = dir.join("state.json");
        let state = state_path.to_string_lossy().to_string();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let curl = dir.join("fake-curl.sh");
        fs::write(
            &curl,
            "#!/bin/sh\nprintf '%s' '{\"choices\":[{\"message\":{\"content\":\"{\\\"calls\\\":[{\\\"tool\\\":\\\"read\\\",\\\"reason\\\":\\\"inspect requested file\\\"}],\\\"summary\\\":\\\"planned by llm\\\"}\"}}]}'\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&curl).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&curl, permissions).unwrap();

        run(vec![
            "--state".to_string(),
            state.clone(),
            "install".to_string(),
            "swe-agent".to_string(),
        ])
        .unwrap();
        let old_manifest = std::env::var("OCTOPUS_LLM_MANIFEST").ok();
        let old_prefix = std::env::var("OCTOPUS_MANIFEST_LLM_PREFIX").ok();
        let old_model = std::env::var("OCTOPUS_MANIFEST_TEST_MODEL").ok();
        let old_base_url = std::env::var("OCTOPUS_MANIFEST_TEST_BASE_URL").ok();
        let old_curl = std::env::var("OCTOPUS_MANIFEST_TEST_CURL").ok();
        std::env::set_var("OCTOPUS_LLM_MANIFEST", "1");
        std::env::set_var("OCTOPUS_MANIFEST_LLM_PREFIX", "OCTOPUS_MANIFEST_TEST");
        std::env::set_var("OCTOPUS_MANIFEST_TEST_MODEL", "test-model");
        std::env::set_var("OCTOPUS_MANIFEST_TEST_BASE_URL", "https://llm.example/v1");
        std::env::set_var(
            "OCTOPUS_MANIFEST_TEST_CURL",
            curl.to_string_lossy().to_string(),
        );

        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "need".to_string(),
            "observe".to_string(),
            "Cargo.toml".to_string(),
            "1".to_string(),
            "1".to_string(),
        ])
        .unwrap();

        restore_env("OCTOPUS_LLM_MANIFEST", old_manifest);
        restore_env("OCTOPUS_MANIFEST_LLM_PREFIX", old_prefix);
        restore_env("OCTOPUS_MANIFEST_TEST_MODEL", old_model);
        restore_env("OCTOPUS_MANIFEST_TEST_BASE_URL", old_base_url);
        restore_env("OCTOPUS_MANIFEST_TEST_CURL", old_curl);
        let content = fs::read_to_string(&state_path).unwrap();
        assert!(content.contains("observe:swe-agent"));
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn cli_evolve_can_use_llm_planning() {
        use std::os::unix::fs::PermissionsExt;

        let _env = env_guard();
        let _cwd = CwdGuard::new();
        let dir =
            std::env::temp_dir().join(format!("octopus-cli-evolve-llm-{}", std::process::id()));
        let state_path = dir.join("state.json");
        let state = state_path.to_string_lossy().to_string();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let curl = dir.join("fake-curl.sh");
        fs::write(
            &curl,
            r#"#!/bin/sh
cat <<'JSON'
{"choices":[{"message":{"content":"{\"summary\":\"llm evolve selected runtime\",\"candidates\":[{\"surface_id\":\"runtime_code\",\"title\":\"LLM runtime patch\",\"target\":\"tools/read.sh\",\"rationale\":\"tool-side action feed\",\"change_plan\":[\"preserve Goal Mem Need Feed boundary\",\"edit runtime adapter\"],\"checks\":[\"tentacles/swe-agent/tools/read.sh README.md 1 2\"]}]}"}}]}
JSON
"#,
        )
        .unwrap();
        let mut permissions = fs::metadata(&curl).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&curl, permissions).unwrap();

        let old_evolve = std::env::var("OCTOPUS_LLM_EVOLVE").ok();
        let old_prefix = std::env::var("OCTOPUS_EVOLVE_LLM_PREFIX").ok();
        let old_model = std::env::var("OCTOPUS_EVOLVE_TEST_MODEL").ok();
        let old_base_url = std::env::var("OCTOPUS_EVOLVE_TEST_BASE_URL").ok();
        let old_curl = std::env::var("OCTOPUS_EVOLVE_TEST_CURL").ok();
        std::env::set_var("OCTOPUS_LLM_EVOLVE", "1");
        std::env::set_var("OCTOPUS_EVOLVE_LLM_PREFIX", "OCTOPUS_EVOLVE_TEST");
        std::env::set_var("OCTOPUS_EVOLVE_TEST_MODEL", "test-model");
        std::env::set_var("OCTOPUS_EVOLVE_TEST_BASE_URL", "https://llm.example/v1");
        std::env::set_var(
            "OCTOPUS_EVOLVE_TEST_CURL",
            curl.to_string_lossy().to_string(),
        );

        run(vec![
            "--state".to_string(),
            state,
            "evolve".to_string(),
            "swe-agent".to_string(),
            "improve".to_string(),
            "feed".to_string(),
        ])
        .unwrap();

        restore_env("OCTOPUS_LLM_EVOLVE", old_evolve);
        restore_env("OCTOPUS_EVOLVE_LLM_PREFIX", old_prefix);
        restore_env("OCTOPUS_EVOLVE_TEST_MODEL", old_model);
        restore_env("OCTOPUS_EVOLVE_TEST_BASE_URL", old_base_url);
        restore_env("OCTOPUS_EVOLVE_TEST_CURL", old_curl);
        let proposal = fs::read_to_string(
            dir.join(".octopus")
                .join("evolution")
                .join("swe-agent")
                .join("PROPOSAL.md"),
        )
        .unwrap();
        let candidates = fs::read_to_string(
            dir.join(".octopus")
                .join("evolution")
                .join("swe-agent")
                .join("PATCH_CANDIDATES.md"),
        )
        .unwrap();
        assert!(proposal.contains("generator: `llm`"));
        assert!(candidates.contains("LLM runtime patch"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cli_evolve_writes_tentacle_evolution_draft() {
        let _env = env_guard();
        let _cwd = CwdGuard::new();
        let dir = std::env::temp_dir().join(format!("octopus-cli-evolve-{}", std::process::id()));
        let state_path = dir.join("state.json");
        let state = state_path.to_string_lossy().to_string();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        std::env::set_current_dir(&dir).unwrap();

        run(vec![
            "--json".to_string(),
            "evolve".to_string(),
            "swe-agent".to_string(),
            "improve".to_string(),
            "repo".to_string(),
            "observation".to_string(),
        ])
        .unwrap();

        let proposal = dir
            .join(".octopus")
            .join("evolution")
            .join("swe-agent")
            .join("PROPOSAL.md");
        let json = dir
            .join(".octopus")
            .join("evolution")
            .join("swe-agent")
            .join("proposal.json");
        let candidates = dir
            .join(".octopus")
            .join("evolution")
            .join("swe-agent")
            .join("PATCH_CANDIDATES.md");
        let drafts = dir
            .join(".octopus")
            .join("evolution")
            .join("swe-agent")
            .join("PATCH_DRAFTS.md");
        let runtime_draft = dir
            .join(".octopus")
            .join("evolution")
            .join("swe-agent")
            .join("patches")
            .join("03-runtime-code.patch.md");
        assert!(proposal.exists());
        assert!(candidates.exists());
        assert!(drafts.exists());
        assert!(runtime_draft.exists());
        assert!(json.exists());
        let markdown = fs::read_to_string(proposal).unwrap();
        assert!(markdown.contains("Tentacle Evolution: swe-agent"));
        assert!(markdown.contains("Evolution Surfaces"));
        let candidates_markdown = fs::read_to_string(candidates).unwrap();
        assert!(candidates_markdown.contains("Patch Candidates: swe-agent"));
        assert!(candidates_markdown.contains("runtime_code"));
        assert!(fs::read_to_string(drafts)
            .unwrap()
            .contains("Patch Drafts: swe-agent"));
        assert!(fs::read_to_string(runtime_draft)
            .unwrap()
            .contains("authorization_required: true"));

        run(vec![
            "--state".to_string(),
            state.clone(),
            "evolve".to_string(),
            "apply".to_string(),
            "swe-agent".to_string(),
            "runtime_code".to_string(),
        ])
        .unwrap();
        let apply_plan = dir
            .join(".octopus")
            .join("evolution")
            .join("swe-agent")
            .join("apply")
            .join("03-runtime-code.md");
        let apply_patch = dir
            .join(".octopus")
            .join("evolution")
            .join("swe-agent")
            .join("apply")
            .join("03-runtime-code.patch");
        assert!(fs::read_to_string(&apply_plan)
            .unwrap()
            .contains("needs_authorization"));
        assert!(!apply_patch.exists());

        run(vec![
            "--state".to_string(),
            state.clone(),
            "oauth".to_string(),
            "octopus".to_string(),
            "evolve:swe-agent".to_string(),
            "harness:write".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "evolve".to_string(),
            "apply".to_string(),
            "swe-agent".to_string(),
            "03-runtime-code".to_string(),
        ])
        .unwrap();
        assert!(fs::read_to_string(apply_plan)
            .unwrap()
            .contains("authorized: true"));
        let patch = fs::read_to_string(apply_patch).unwrap();
        assert!(patch.contains("diff --git"));
        assert!(patch.contains("Octopus evolution candidate 03-runtime-code"));

        run(vec![
            "--state".to_string(),
            state.clone(),
            "evolve".to_string(),
            "score".to_string(),
            "swe-agent".to_string(),
            "03-runtime-code".to_string(),
            "satisfied".to_string(),
            "patch".to_string(),
            "improved".to_string(),
            "feed".to_string(),
        ])
        .unwrap();
        let state_content = fs::read_to_string(&state_path).unwrap();
        assert!(state_content.contains("evolution_outcomes"));
        assert!(state_content.contains("patch improved feed"));
        run(vec![
            "--state".to_string(),
            state,
            "evolve".to_string(),
            "swe-agent".to_string(),
            "next".to_string(),
        ])
        .unwrap();
        let updated_markdown = fs::read_to_string(
            dir.join(".octopus")
                .join("evolution")
                .join("swe-agent")
                .join("PROPOSAL.md"),
        )
        .unwrap();
        assert!(updated_markdown.contains("Previous Outcomes"));
        assert!(updated_markdown.contains("patch improved feed"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cli_scaffold_installs_local_tentacle() {
        let _env = env_guard();
        let _cwd = CwdGuard::new();
        let dir = std::env::temp_dir().join(format!("octopus-cli-scaffold-{}", std::process::id()));
        let state_path = dir.join("state.json");
        let state = state_path.to_string_lossy().to_string();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        std::env::set_current_dir(&dir).unwrap();

        run(vec![
            "scaffold".to_string(),
            "local_feed".to_string(),
            "python".to_string(),
        ])
        .unwrap();
        run(vec!["manifests".to_string()]).unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "install".to_string(),
            "local_feed".to_string(),
        ])
        .unwrap();
        run(vec![
            "--json".to_string(),
            "probe".to_string(),
            "local_feed".to_string(),
            "observe".to_string(),
            "README.md".to_string(),
        ])
        .unwrap();

        let content = fs::read_to_string(state_path).unwrap();
        assert!(content.contains("local_feed"));
        assert!(dir.join("tentacles/local_feed/manifest.json").exists());
        assert!(dir.join("tentacles/local_feed/tools/feed.py").exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cli_installs_profile_into_state() {
        let _env = env_guard();
        let path =
            std::env::temp_dir().join(format!("octopus-install-state-{}.json", std::process::id()));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "install".to_string(),
            "research".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "install".to_string(),
            "swe-agent".to_string(),
        ])
        .unwrap();
        run(vec!["--state".to_string(), state, "installed".to_string()]).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("research"));
        assert!(content.contains("installed_tentacles"));
        assert!(content.contains("tools/read.sh"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_installed_manifest_feeds_need() {
        let _env = env_guard();
        let path =
            std::env::temp_dir().join(format!("octopus-feed-state-{}.json", std::process::id()));
        let state = path.to_string_lossy().to_string();
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .to_string_lossy()
            .to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "install".to_string(),
            "swe-agent".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "need".to_string(),
            "observe".to_string(),
            repo,
        ])
        .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("observe:swe-agent"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_chat_refines_goal() {
        let _env = env_guard();
        let old_chat = std::env::var("OCTOPUS_CHAT_LLM").ok();
        std::env::remove_var("OCTOPUS_CHAT_LLM");
        let path =
            std::env::temp_dir().join(format!("octopus-chat-state-{}.json", std::process::id()));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "chat".to_string(),
            "build".to_string(),
            "octopus".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "chat".to_string(),
            "make".to_string(),
            "tools".to_string(),
            "think".to_string(),
        ])
        .unwrap();
        run(vec!["--state".to_string(), state, "goal".to_string()]).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"objective\": \"build octopus\""));
        assert!(content.contains("make tools think"));
        assert!(content.contains("goal_turns"));
        restore_env("OCTOPUS_CHAT_LLM", old_chat);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_oauth_unlocks_self_iteration_plan() {
        let _env = env_guard();
        let path =
            std::env::temp_dir().join(format!("octopus-oauth-state-{}.json", std::process::id()));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "self-iterate".to_string(),
            "dangoZhang/Octopus".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "oauth".to_string(),
            "github".to_string(),
            "dangoZhang/Octopus".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "self-iterate".to_string(),
            "dangoZhang/Octopus".to_string(),
        ])
        .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("github:dangoZhang/Octopus"));
        assert!(content.contains("pull_request:write"));
        let _ = fs::remove_file(path);
    }
}
