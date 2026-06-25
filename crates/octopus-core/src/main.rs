use octopus_core::{
    default_permissions, default_tentacle_profiles, inspect_tentacle_manifests,
    propose_tentacle_evolution, scaffold_tentacle, write_tentacle_evolution_artifacts, AdaptReport,
    CapabilityGrant, ChatClient, ChatMessage, ChatRole, EnvironmentReport, EvolutionArtifact,
    GoalChat, Harness, HarnessState, HeartbeatReport, Need, NeedKind, OpenAiCompatibleChatClient,
    OpenAiCompatibleConfig, SelfIterationPlan, StatusReport, TentacleEvolutionProposal,
    TentacleManifestReport, TentacleScaffold,
};
use std::env;
use std::path::PathBuf;

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
            let chat = harness.chat(message);
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
        Some("status") | Some("doctor") => {
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
            let tentacle_id = rest
                .get(1)
                .ok_or_else(|| "evolve requires a tentacle id".to_string())?;
            let objective = rest
                .get(2..)
                .filter(|values| !values.is_empty())
                .map(|values| values.join(" "))
                .unwrap_or_else(|| "improve feed quality".to_string());
            let proposal =
                propose_tentacle_evolution(default_tentacles_root(), tentacle_id, &objective)?;
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
        Language::En => println!("Installable tentacles:"),
        Language::Zh => println!("可安装触手:"),
    }
    for profile in profiles {
        println!("- {}: {}", profile.id, profile.description);
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
            "- {}: brain={}, runtime={}, tools={}, status={}",
            report.id,
            report.brain_kind,
            report.runtime_kinds.join(","),
            report.tool_count,
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
        }
        Language::Zh => {
            println!("目标: {}", chat.goal.objective);
            println!(
                "第 {} 轮: {}",
                chat.turn.index,
                localize_summary(&chat.turn.summary, language)
            );
            println!("调整: {}", chat.goal.constraints.len());
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
    match language {
        Language::En => {
            println!("tentacle: {}", proposal.tentacle_id);
            println!("objective: {}", proposal.objective);
            println!("proposal: {}", artifact.proposal_path);
            println!("json: {}", artifact.json_path);
            println!("editable: {}", join_or_none(&proposal.editable));
            println!("checks: {}", join_or_none(&proposal.checks));
        }
        Language::Zh => {
            println!("触手: {}", proposal.tentacle_id);
            println!("目标: {}", proposal.objective);
            println!("草案: {}", artifact.proposal_path);
            println!("JSON: {}", artifact.json_path);
            println!("可编辑: {}", join_or_none(&proposal.editable));
            println!("检查: {}", join_or_none(&proposal.checks));
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
            OpenAiCompatibleConfig::from_env()?,
        ));
    }
    Ok(Harness::with_state(state))
}

fn manifest_llm_enabled() -> bool {
    env::var("OCTOPUS_LLM_MANIFEST")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
}

fn usage() -> String {
    "usage: octopus-core [--state path] [--lang en|zh] [--json] need <kind> <query> | chat <message> | llm <message> | goal | status | doctor | beat [memory_keep] | oauth <provider> <scope> [permissions...] | oauth revoke <grant> | self-iterate <repo> | evolve <tentacle> <objective> | scaffold <tentacle> [python|node|shell|http] | routes | catalog | manifests [root] | env | adapt [root] | install <profile> | installed".to_string()
}

#[cfg(test)]
mod tests {
    use super::{localize_summary, run, Language};
    use std::fs;
    use std::path::Path;

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
    fn cli_persists_memory_between_runs() {
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
        run(vec!["catalog".to_string()]).unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles")
            .to_string_lossy()
            .to_string();
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
    fn cli_status_and_doctor_commands_run() {
        let path =
            std::env::temp_dir().join(format!("octopus-status-state-{}.json", std::process::id()));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

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

    #[cfg(unix)]
    #[test]
    fn cli_llm_uses_openai_compatible_env() {
        use std::os::unix::fs::PermissionsExt;

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
    fn cli_need_can_use_manifest_llm_planning() {
        use std::os::unix::fs::PermissionsExt;

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
        let old_model = std::env::var("OCTOPUS_LLM_MODEL").ok();
        let old_base_url = std::env::var("OCTOPUS_LLM_BASE_URL").ok();
        let old_curl = std::env::var("OCTOPUS_LLM_CURL").ok();
        std::env::set_var("OCTOPUS_LLM_MANIFEST", "1");
        std::env::set_var("OCTOPUS_LLM_MODEL", "test-model");
        std::env::set_var("OCTOPUS_LLM_BASE_URL", "https://llm.example/v1");
        std::env::set_var("OCTOPUS_LLM_CURL", curl.to_string_lossy().to_string());

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
        restore_env("OCTOPUS_LLM_MODEL", old_model);
        restore_env("OCTOPUS_LLM_BASE_URL", old_base_url);
        restore_env("OCTOPUS_LLM_CURL", old_curl);
        let content = fs::read_to_string(&state_path).unwrap();
        assert!(content.contains("observe:swe-agent"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cli_evolve_writes_tentacle_evolution_draft() {
        let _cwd = CwdGuard::new();
        let dir = std::env::temp_dir().join(format!("octopus-cli-evolve-{}", std::process::id()));
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
        assert!(proposal.exists());
        assert!(json.exists());
        assert!(fs::read_to_string(proposal)
            .unwrap()
            .contains("Tentacle Evolution: swe-agent"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cli_scaffold_installs_local_tentacle() {
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

        let content = fs::read_to_string(state_path).unwrap();
        assert!(content.contains("local_feed"));
        assert!(dir.join("tentacles/local_feed/manifest.json").exists());
        assert!(dir.join("tentacles/local_feed/tools/feed.py").exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cli_installs_profile_into_state() {
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
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_oauth_unlocks_self_iteration_plan() {
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
