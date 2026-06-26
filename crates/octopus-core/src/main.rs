use octopus_core::{
    default_permissions, default_tentacle_profiles, inspect_tentacle_manifests,
    load_tentacle_manifests, plan_tentacle_evolution_apply, propose_tentacle_evolution_with_client,
    propose_tentacle_evolution_with_state, recommend_tentacle_evolution_apply, scaffold_tentacle,
    think_tentacle, write_harness_beat_evolution_artifacts, write_tentacle_apply_artifacts,
    write_tentacle_evolution_artifacts, AdaptReport, CapabilityGrant, ChatClient, ChatMessage,
    ChatRole, CheckHistoryInput, CheckHistoryRecord, EnvironmentReport, EvolutionApplyArtifact,
    EvolutionApplyPlan, EvolutionArtifact, EvolutionOutcome, EvolutionRecommendation, Feed,
    FeedTraceRecord, GoalChat, Harness, HarnessBeatEvolution, HarnessState, HeartBeat,
    HeartbeatReport, InstalledTentacle, LoadedTentacleManifest, Need, NeedKind,
    OpenAiCompatibleChatClient, OpenAiCompatibleConfig, SelfIterationPlan, Status, StatusReport,
    TentacleEvolutionProposal, TentacleManifestReport, TentacleProfile, TentacleScaffold,
    TentacleThinkingPlan, TentacleToolAction, TentacleToolCandidate, ToolPermission,
};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::panic;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

#[derive(serde::Deserialize)]
struct BridgeRunRequest {
    args: Vec<String>,
}

#[derive(serde::Serialize)]
struct BridgeRunResponse {
    ok: bool,
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

#[derive(serde::Serialize)]
struct SelfIterationPrReport {
    repository: String,
    mode: String,
    grant_id: String,
    branch: String,
    title: String,
    body_path: String,
    publish_path: Option<String>,
    output: String,
    next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct PetReport {
    state: String,
    title: String,
    summary: String,
    color: String,
    fallback: String,
    event_source: Option<String>,
    event_summary: Option<String>,
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
struct InstallReport {
    id: String,
    name: String,
    source_kind: String,
    state_path: String,
    description: String,
    brain_kind: String,
    installed: bool,
    needs: Vec<String>,
    tools: Vec<String>,
    runtimes: Vec<String>,
    evolution_surfaces: Vec<String>,
    grants: Vec<InstallGrantReport>,
    checks: Vec<String>,
    check_history: Vec<CheckHistoryRecord>,
    next: Vec<String>,
}

#[derive(serde::Serialize)]
struct InstallGrantReport {
    provider: String,
    scope: String,
    permissions: Vec<String>,
    reason: Option<String>,
    active: bool,
    command: String,
}

type InstallGrantGroups = BTreeMap<(String, String), (BTreeSet<String>, BTreeSet<String>)>;

#[derive(Debug, serde::Serialize)]
struct CheckReport {
    id: String,
    name: String,
    source_kind: String,
    cwd: String,
    passed: bool,
    results: Vec<CheckResultReport>,
    history: Vec<CheckHistoryRecord>,
}

#[derive(Debug, serde::Serialize)]
struct CheckResultReport {
    index: usize,
    command: String,
    cwd: String,
    status: Status,
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

#[derive(Clone, Debug)]
struct SelectedCheck {
    index: usize,
    command: String,
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

#[derive(Clone, Debug, serde::Serialize)]
struct ProviderProfile {
    id: String,
    name: String,
    description: String,
    base_url: String,
    model: String,
    key_env: String,
    notes: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct ProviderEnvReport {
    profile: ProviderProfile,
    prefix: String,
    exports: Vec<String>,
    shell: String,
}

#[derive(Debug, serde::Serialize)]
struct ProviderCheckReport {
    ok: bool,
    prefix: String,
    model: String,
    base_url: String,
    api_key_present: bool,
    content: String,
    metadata: std::collections::BTreeMap<String, String>,
    next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct ProviderStatusReport {
    layers: Vec<ProviderLayerStatus>,
    next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct ProviderLayerStatus {
    layer: String,
    purpose: String,
    enabled: bool,
    prefix: String,
    configured: bool,
    model: Option<String>,
    base_url: Option<String>,
    api_key_present: bool,
    curl_command: String,
    curl_available: bool,
    check_command: String,
    message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Language {
    En,
    Zh,
}

fn main() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        if !is_broken_pipe_panic(info.payload()) {
            default_hook(info);
        }
    }));
    match panic::catch_unwind(|| run(env::args().skip(1).collect())) {
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
        Err(payload) if is_broken_pipe_panic(payload.as_ref()) => {}
        Err(payload) => panic::resume_unwind(payload),
    }
}

fn is_broken_pipe_panic(payload: &(dyn std::any::Any + Send)) -> bool {
    panic_payload_text(payload).is_some_and(|text| {
        text.contains("failed printing to stdout") && text.contains("Broken pipe")
    })
}

fn panic_payload_text(payload: &(dyn std::any::Any + Send)) -> Option<&str> {
    if let Some(text) = payload.downcast_ref::<&str>() {
        return Some(*text);
    }
    payload.downcast_ref::<String>().map(String::as_str)
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
                print_feedback(&feedback, language);
            }
            Ok(())
        }
        Some("think") => {
            let tentacle_id = rest
                .get(1)
                .ok_or_else(|| "think requires a tentacle id".to_string())?;
            let kind = rest
                .get(2)
                .ok_or_else(|| "think requires a need kind".to_string())
                .and_then(|value| parse_kind(value))?;
            let query = rest
                .get(3..)
                .filter(|values| !values.is_empty())
                .map(|values| values.join(" "))
                .ok_or_else(|| "think requires a query".to_string())?;
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let llm_config = if manifest_llm_enabled() {
                Some(manifest_llm_config()?)
            } else {
                None
            };
            let plan = think_tentacle(
                default_tentacles_root(),
                tentacle_id,
                &Need::new(kind, query),
                llm_config,
                &loaded.grants,
            )?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&plan).map_err(|error| error.to_string())?
                );
            } else {
                print_thinking_plan(&plan, language);
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
        Some("traces") => {
            let limit = rest
                .get(1)
                .map(|value| {
                    value
                        .parse::<usize>()
                        .map_err(|_| format!("invalid trace limit: {value}"))
                })
                .transpose()?
                .unwrap_or(10);
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let traces = loaded.recent_feed_traces(limit);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&traces).map_err(|error| error.to_string())?
                );
            } else {
                print_feed_traces(&traces, language);
            }
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
        Some("providers") => {
            let profiles = provider_profiles();
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&profiles).map_err(|error| error.to_string())?
                );
            } else {
                print_provider_profiles(&profiles, language);
            }
            Ok(())
        }
        Some("provider") => {
            if rest.get(1).map(String::as_str) == Some("status") {
                let report = provider_status_report();
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                    );
                } else {
                    print_provider_status(&report, language);
                }
                return Ok(());
            }
            if rest.get(1).map(String::as_str) == Some("check") {
                let prefix = rest.get(2).map(String::as_str).unwrap_or("OCTOPUS_LLM");
                let message = rest
                    .get(3..)
                    .filter(|values| !values.is_empty())
                    .map(|values| values.join(" "))
                    .unwrap_or_else(|| "Reply with a short Octopus provider check.".to_string());
                let report = provider_check_report(prefix, &message)?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                    );
                } else {
                    print_provider_check(&report, language);
                }
                return Ok(());
            }
            let profile_id = rest
                .get(1)
                .ok_or_else(|| "provider requires a profile id".to_string())?;
            let prefix = rest.get(2).map(String::as_str).unwrap_or("OCTOPUS_LLM");
            let report = provider_env_report(profile_id, prefix)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_provider_env(&report, language);
            }
            Ok(())
        }
        Some("bridge") => {
            let addr = rest.get(1).map(String::as_str).unwrap_or("127.0.0.1:8765");
            run_bridge(addr)
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
            let mut report = loaded.beat(memory_keep);
            let cwd = env::current_dir().map_err(|error| error.to_string())?;
            let evolution =
                write_harness_beat_evolution_artifacts(default_tentacles_root(), &cwd, &loaded)?;
            if let Some(evolution) = &evolution {
                attach_harness_beat_evolution(&mut report, evolution);
                loaded.record_pet_event(
                    "harness",
                    "harness beat",
                    format!(
                        "recommended {} for {}",
                        evolution.candidate_id, evolution.tentacle_id
                    ),
                    Status::Partial,
                );
            }
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
            if rest.get(1).map(String::as_str) == Some("pr") {
                let repository = rest
                    .get(2)
                    .ok_or_else(|| "self-iterate pr requires a repository".to_string())?;
                let objective = rest
                    .get(3..)
                    .filter(|values| !values.is_empty())
                    .map(|values| values.join(" "));
                let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
                let plan = loaded.self_iteration_plan(repository.as_str(), objective.as_deref());
                let report = publish_self_iteration_pr(&plan)?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                    );
                } else {
                    print_self_iteration_pr_report(&report, language);
                }
                return Ok(());
            }
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
            if rest.get(1).map(String::as_str) == Some("recommend") {
                let tentacle_id = rest
                    .get(2)
                    .ok_or_else(|| "evolve recommend requires a tentacle id".to_string())?;
                let objective = rest
                    .get(3..)
                    .filter(|values| !values.is_empty())
                    .map(|values| values.join(" "))
                    .unwrap_or_else(|| "recommend next evolution candidate".to_string());
                let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
                let proposal = propose_evolution_for_cli(tentacle_id, &objective, &loaded)?;
                let cwd = env::current_dir().map_err(|error| error.to_string())?;
                let artifact = write_tentacle_evolution_artifacts(&cwd, &proposal)?;
                let recommendation = recommend_tentacle_evolution_apply(&proposal, &loaded)?;
                let apply_artifact = write_tentacle_apply_artifacts(cwd, &recommendation.apply)?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "recommendation": recommendation,
                            "apply_artifact": apply_artifact,
                            "evolution_artifact": artifact,
                        }))
                        .map_err(|error| error.to_string())?
                    );
                } else {
                    print_evolution_recommendation(&recommendation, &apply_artifact, language);
                }
                return Ok(());
            }
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
            let root = default_tentacles_root();
            let profile_metadata = default_tentacle_profiles()
                .into_iter()
                .find(|candidate| candidate.id == profile.as_str());
            let manifest_metadata = load_tentacle_manifests(&root)
                .map_err(|error| error.to_string())?
                .into_iter()
                .find(|loaded| loaded.manifest.id == profile.as_str());
            let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let profile_installed = loaded.install_profile(profile).is_ok();
            let manifest_installed = loaded.install_manifest(&root, profile).ok();
            if !profile_installed && manifest_installed.is_none() {
                return Err(format!("unknown profile or tentacle manifest: {profile}"));
            }
            loaded.save(&state).map_err(|error| error.to_string())?;
            let report = install_report(
                profile,
                &state,
                profile_installed,
                manifest_installed.as_ref(),
                profile_metadata.as_ref(),
                manifest_metadata.as_ref(),
                &loaded,
            );
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_install_report(&report, language);
            }
            Ok(())
        }
        Some("check") => {
            let tentacle_id = rest
                .get(1)
                .ok_or_else(|| "check requires a tentacle id".to_string())?;
            let selected = rest
                .get(2)
                .map(|value| parse_check_index(value))
                .transpose()?;
            let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let mut report = check_report(tentacle_id, selected)?;
            record_check_report(&mut loaded, &mut report);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_check_report(&report, language);
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

fn print_feedback(feedback: &octopus_core::Feedback, language: Language) {
    println!("{}", localize_summary(&feedback.summary, language));
    for feed in &feedback.feeds {
        if let Some(trace) = feed_trace(feed) {
            match language {
                Language::En => println!("feed_trace: {trace}"),
                Language::Zh => println!("Feed轨迹: {trace}"),
            }
        }
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

fn print_install_report(report: &InstallReport, language: Language) {
    match language {
        Language::En => {
            println!("installed {}", report.id);
            println!("source: {}", report.source_kind);
            println!("state: {}", report.state_path);
            println!("brain: {}", report.brain_kind);
            println!("needs: {}", join_or_none(&report.needs));
            println!("tools: {}", join_or_none(&report.tools));
            println!("runtimes: {}", join_or_none(&report.runtimes));
            println!("surfaces: {}", join_or_none(&report.evolution_surfaces));
            print_install_grants(report, language);
            print_install_checks(report, language);
            print_install_check_history(report, language);
            print_install_next(report, language);
        }
        Language::Zh => {
            println!("已安装 {}", report.id);
            println!("来源: {}", report.source_kind);
            println!("状态文件: {}", report.state_path);
            println!("触手脑: {}", report.brain_kind);
            println!("需求: {}", join_or_none(&report.needs));
            println!("工具: {}", join_or_none(&report.tools));
            println!("运行时: {}", join_or_none(&report.runtimes));
            println!("进化面: {}", join_or_none(&report.evolution_surfaces));
            print_install_grants(report, language);
            print_install_checks(report, language);
            print_install_check_history(report, language);
            print_install_next(report, language);
        }
    }
}

fn print_install_grants(report: &InstallReport, language: Language) {
    if report.grants.is_empty() {
        match language {
            Language::En => println!("grants: none required"),
            Language::Zh => println!("授权: 无需授权"),
        }
        return;
    }
    match language {
        Language::En => println!("grants:"),
        Language::Zh => println!("授权:"),
    }
    for grant in &report.grants {
        let status = match (language, grant.active) {
            (Language::En, true) => "active",
            (Language::En, false) => "missing",
            (Language::Zh, true) => "已生效",
            (Language::Zh, false) => "待授权",
        };
        if let Some(reason) = &grant.reason {
            println!("- {status}: {} ({reason})", grant.command);
        } else {
            println!("- {status}: {}", grant.command);
        }
    }
}

fn print_install_checks(report: &InstallReport, language: Language) {
    if report.checks.is_empty() {
        return;
    }
    match language {
        Language::En => println!("checks:"),
        Language::Zh => println!("检查:"),
    }
    for check in &report.checks {
        println!("- {check}");
    }
}

fn print_install_check_history(report: &InstallReport, language: Language) {
    if report.check_history.is_empty() {
        return;
    }
    match language {
        Language::En => println!("check history:"),
        Language::Zh => println!("检查历史:"),
    }
    for record in &report.check_history {
        println!("- {}", check_history_line(record));
    }
}

fn print_install_next(report: &InstallReport, language: Language) {
    if report.next.is_empty() {
        return;
    }
    match language {
        Language::En => println!("next:"),
        Language::Zh => println!("下一步:"),
    }
    for command in &report.next {
        println!("- {command}");
    }
}

fn print_check_report(report: &CheckReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus check {}", report.id);
            println!("source: {}", report.source_kind);
            println!("cwd: {}", report.cwd);
            println!("passed: {}", report.passed);
        }
        Language::Zh => {
            println!("章鱼检查 {}", report.id);
            println!("来源: {}", report.source_kind);
            println!("目录: {}", report.cwd);
            println!("通过: {}", report.passed);
        }
    }
    for result in &report.results {
        let label = match result.status {
            Status::Satisfied => "ok",
            Status::Failed => "failed",
            Status::Partial => "partial",
            Status::Unsupported => "unsupported",
        };
        println!("- {label}: {}", result.command);
        if !result.stdout.trim().is_empty() {
            println!("  stdout: {}", compact_output(&result.stdout));
        }
        if !result.stderr.trim().is_empty() {
            println!("  stderr: {}", compact_output(&result.stderr));
        }
    }
    if !report.history.is_empty() {
        match language {
            Language::En => println!("recent history:"),
            Language::Zh => println!("最近历史:"),
        }
        for record in &report.history {
            println!("- {}", check_history_line(record));
        }
    }
}

fn check_history_line(record: &CheckHistoryRecord) -> String {
    let label = match record.status {
        Status::Satisfied => "ok",
        Status::Failed => "failed",
        Status::Partial => "partial",
        Status::Unsupported => "unsupported",
    };
    let check = record
        .command_index
        .map(|index| format!("check {index}"))
        .unwrap_or_else(|| "check".to_string());
    format!(
        "#{} {label} {check}: {}",
        record.index,
        compact_output(&record.command)
    )
}

fn compact_output(value: &str) -> String {
    let value = value.trim().replace('\n', " ");
    let compact = value.chars().take(180).collect::<String>();
    if compact.len() == value.len() {
        compact
    } else {
        format!("{compact}...")
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

fn attach_harness_beat_evolution(report: &mut HeartbeatReport, evolution: &HarnessBeatEvolution) {
    let Some(beat) = report.beats.iter_mut().find(|beat| beat.name == "harness") else {
        return;
    };
    beat.changed = true;
    beat.summary = format!(
        "{}; recommended {} for {}",
        beat.summary, evolution.candidate_id, evolution.tentacle_id
    );
    beat.data.insert(
        "evolution_source_check".to_string(),
        evolution.source_check_index.to_string(),
    );
    beat.data.insert(
        "evolution_tentacle".to_string(),
        evolution.tentacle_id.clone(),
    );
    beat.data.insert(
        "evolution_candidate".to_string(),
        evolution.candidate_id.clone(),
    );
    beat.data
        .insert("evolution_status".to_string(), evolution.status.clone());
    beat.data
        .insert("evolution_reason".to_string(), evolution.reason.clone());
    beat.data.insert(
        "evolution_plan".to_string(),
        evolution.apply_plan_path.clone(),
    );
    beat.data.insert(
        "evolution_plan_preview".to_string(),
        evolution.apply_plan_preview.clone(),
    );
    beat.data
        .insert("evolution_next".to_string(), evolution.next_action.clone());
    if let Some(patch) = &evolution.patch_path {
        beat.data
            .insert("evolution_patch".to_string(), patch.clone());
    }
}

fn print_heartbeat_report(report: &HeartbeatReport, language: Language) {
    for beat in &report.beats {
        match language {
            Language::En => println!("{}: {}", beat.name, beat.summary),
            Language::Zh => println!("{}: {}", localize_beat(&beat.name), beat.summary),
        }
        if beat.name == "harness" {
            print_harness_beat_evolution(beat, language);
        }
    }
}

fn print_harness_beat_evolution(beat: &HeartBeat, language: Language) {
    let Some(candidate) = beat.data.get("evolution_candidate") else {
        return;
    };
    let tentacle = beat
        .data
        .get("evolution_tentacle")
        .map(String::as_str)
        .unwrap_or("unknown");
    let status = beat
        .data
        .get("evolution_status")
        .map(String::as_str)
        .unwrap_or("unknown");
    let next = beat
        .data
        .get("evolution_next")
        .map(String::as_str)
        .unwrap_or("review evolution plan");
    let plan = beat
        .data
        .get("evolution_plan")
        .map(String::as_str)
        .unwrap_or("none");
    match language {
        Language::En => {
            println!("  recommendation: {tentacle} {candidate} ({status})");
            println!("  plan: {plan}");
            println!("  next: {next}");
        }
        Language::Zh => {
            println!("  推荐: {tentacle} {candidate} ({status})");
            println!("  计划: {plan}");
            println!("  下一步: {next}");
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
            if let Some(source) = &report.event_source {
                println!("event: {source}");
            }
            if let Some(summary) = &report.event_summary {
                println!("event_summary: {summary}");
            }
            println!("url: {}", report.target);
            println!("exists: {}", report.exists);
        }
        Language::Zh => {
            println!("章鱼桌宠");
            println!("像素: {}", report.fallback);
            println!("状态: {}", report.state);
            println!("摘要: {}", report.summary);
            if let Some(source) = &report.event_source {
                println!("事件: {source}");
            }
            if let Some(summary) = &report.event_summary {
                println!("事件摘要: {summary}");
            }
            println!("入口: {}", report.target);
            println!("存在: {}", report.exists);
        }
    }
}

fn provider_profiles() -> Vec<ProviderProfile> {
    vec![
        ProviderProfile {
            id: "openai".to_string(),
            name: "OpenAI-compatible cloud".to_string(),
            description: "Default hosted chat-completions provider.".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4.1-mini".to_string(),
            key_env: "OPENAI_API_KEY".to_string(),
            notes: vec![
                "Set OPENAI_API_KEY before sourcing the generated env.".to_string(),
                "Works for chat, manifest tool planning, and harness evolution.".to_string(),
            ],
        },
        ProviderProfile {
            id: "local".to_string(),
            name: "Local OpenAI-compatible server".to_string(),
            description: "For Ollama, LM Studio, llama.cpp server, or another local adapter."
                .to_string(),
            base_url: "http://localhost:11434/v1".to_string(),
            model: "llama3.1".to_string(),
            key_env: "".to_string(),
            notes: vec![
                "Change the model to whatever your local server exposes.".to_string(),
                "API key is intentionally empty for most local adapters.".to_string(),
            ],
        },
        ProviderProfile {
            id: "openrouter".to_string(),
            name: "OpenRouter-compatible cloud".to_string(),
            description: "Hosted router using the OpenAI-compatible chat API.".to_string(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model: "openai/gpt-4.1-mini".to_string(),
            key_env: "OPENROUTER_API_KEY".to_string(),
            notes: vec![
                "Set OPENROUTER_API_KEY before sourcing the generated env.".to_string(),
                "Change the model id to any enabled router model.".to_string(),
            ],
        },
        ProviderProfile {
            id: "custom".to_string(),
            name: "Custom OpenAI-compatible endpoint".to_string(),
            description: "Template for any provider that speaks chat completions.".to_string(),
            base_url: "https://example.com/v1".to_string(),
            model: "your-model".to_string(),
            key_env: "OCTOPUS_PROVIDER_API_KEY".to_string(),
            notes: vec![
                "Edit base URL, model, and key env to match your provider.".to_string(),
                "Use a different prefix for chat, manifest, or evolution if needed.".to_string(),
            ],
        },
    ]
}

fn provider_profile(id: &str) -> Result<ProviderProfile, String> {
    provider_profiles()
        .into_iter()
        .find(|profile| profile.id == id)
        .ok_or_else(|| {
            format!(
                "unknown provider profile: {id}; expected {}",
                provider_profiles()
                    .iter()
                    .map(|profile| profile.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
}

fn provider_env_report(profile_id: &str, prefix: &str) -> Result<ProviderEnvReport, String> {
    if !valid_env_prefix(prefix) {
        return Err(format!(
            "invalid provider env prefix: {prefix}; use letters, digits, and underscore"
        ));
    }
    let profile = provider_profile(profile_id)?;
    let mut exports = vec![
        format!("export {prefix}_MODEL={}", shell_value(&profile.model)),
        format!(
            "export {prefix}_BASE_URL={}",
            shell_value(&profile.base_url)
        ),
    ];
    if profile.key_env.is_empty() {
        exports.push(format!("export {prefix}_API_KEY="));
    } else {
        exports.push(format!(
            "export {prefix}_API_KEY=\"${{{}:-}}\"",
            profile.key_env
        ));
    }
    exports.extend([
        "export OCTOPUS_CHAT_LLM=1".to_string(),
        "export OCTOPUS_LLM_MANIFEST=1".to_string(),
        "export OCTOPUS_LLM_EVOLVE=1".to_string(),
        format!("export OCTOPUS_CHAT_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_MANIFEST_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_EVOLVE_LLM_PREFIX={prefix}"),
    ]);
    let shell = exports.join("\n");
    Ok(ProviderEnvReport {
        profile,
        prefix: prefix.to_string(),
        exports,
        shell,
    })
}

fn provider_check_report(prefix: &str, message: &str) -> Result<ProviderCheckReport, String> {
    if !valid_env_prefix(prefix) {
        return Err(format!(
            "invalid provider env prefix: {prefix}; use letters, digits, and underscore"
        ));
    }
    let config = OpenAiCompatibleConfig::from_env_prefix(prefix)
        .map_err(|error| provider_check_error(prefix, &error))?;
    let mut client = OpenAiCompatibleChatClient::new(config.clone());
    let response = client
        .chat(&[
            ChatMessage::new(
                ChatRole::System,
                "You are an Octopus provider check. Reply briefly and confirm the endpoint works.",
            ),
            ChatMessage::new(ChatRole::User, message),
        ])
        .map_err(|error| provider_check_error(prefix, &error))?;
    Ok(ProviderCheckReport {
        ok: true,
        prefix: prefix.to_string(),
        model: config.model,
        base_url: config.base_url,
        api_key_present: config.api_key.is_some(),
        content: response.content,
        metadata: response.metadata,
        next: vec![
            "octopus chat \"refine the goal\"".to_string(),
            "octopus need observe README.md".to_string(),
            "octopus evolve swe-agent \"improve tool-side feedback\"".to_string(),
        ],
    })
}

fn provider_check_error(prefix: &str, error: &str) -> String {
    format!(
        "provider check failed for {prefix}: {error}\nnext: octopus providers; octopus provider openai > .octopus/llm.env; source .octopus/llm.env; octopus provider check {prefix}"
    )
}

fn provider_status_report() -> ProviderStatusReport {
    let layers = vec![
        provider_layer_status(
            "chat_goal",
            "chat refines Goal before Needs",
            chat_llm_enabled(),
            chat_llm_prefix(),
            "export OCTOPUS_CHAT_LLM=1",
        ),
        provider_layer_status(
            "tentacle_planning",
            "tentacle brain chooses tool metadata before Feed",
            manifest_llm_enabled(),
            manifest_llm_prefix(),
            "export OCTOPUS_LLM_MANIFEST=1",
        ),
        provider_layer_status(
            "harness_evolution",
            "LLM proposes prompt/meta/code/policy evolution candidates",
            evolve_llm_enabled(),
            evolve_llm_prefix(),
            "export OCTOPUS_LLM_EVOLVE=1",
        ),
    ];
    let mut next = vec!["octopus providers".to_string()];
    if layers.iter().any(|layer| !layer.configured) {
        next.push("octopus provider openai > .octopus/llm.env".to_string());
        next.push("source .octopus/llm.env".to_string());
    }
    for layer in &layers {
        if layer.configured && layer.curl_available {
            next.push(layer.check_command.clone());
        } else if !layer.enabled {
            next.push(layer.message.clone());
        }
    }
    next.sort();
    next.dedup();
    ProviderStatusReport { layers, next }
}

fn provider_layer_status(
    layer: &str,
    purpose: &str,
    enabled: bool,
    prefix: String,
    enable_hint: &str,
) -> ProviderLayerStatus {
    match OpenAiCompatibleConfig::from_env_prefix(&prefix) {
        Ok(config) => ProviderLayerStatus {
            layer: layer.to_string(),
            purpose: purpose.to_string(),
            enabled,
            prefix: prefix.clone(),
            configured: true,
            model: Some(config.model),
            base_url: Some(config.base_url),
            api_key_present: config.api_key.is_some(),
            curl_available: command_ready(&config.curl_command),
            curl_command: config.curl_command,
            check_command: format!("octopus provider check {prefix}"),
            message: if enabled {
                "configured".to_string()
            } else {
                format!("configured but disabled; run `{enable_hint}`")
            },
        },
        Err(error) => {
            let curl_command =
                env::var(format!("{prefix}_CURL")).unwrap_or_else(|_| "curl".to_string());
            ProviderLayerStatus {
                layer: layer.to_string(),
                purpose: purpose.to_string(),
                enabled,
                prefix: prefix.clone(),
                configured: false,
                model: None,
                base_url: None,
                api_key_present: false,
                curl_available: command_ready(&curl_command),
                curl_command,
                check_command: format!("octopus provider check {prefix}"),
                message: error,
            }
        }
    }
}

fn valid_env_prefix(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_uppercase() || first == '_')
        && chars.all(|item| item.is_ascii_uppercase() || item.is_ascii_digit() || item == '_')
}

fn shell_value(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn print_provider_profiles(profiles: &[ProviderProfile], language: Language) {
    match language {
        Language::En => println!("Provider profiles:"),
        Language::Zh => println!("Provider 配置:"),
    }
    for profile in profiles {
        match language {
            Language::En => println!(
                "- {}: {} ({}, {})",
                profile.id, profile.description, profile.model, profile.base_url
            ),
            Language::Zh => println!(
                "- {}: {} ({}, {})",
                profile.id, profile.description, profile.model, profile.base_url
            ),
        }
    }
}

fn print_provider_env(report: &ProviderEnvReport, language: Language) {
    match language {
        Language::En => {
            println!("# Octopus provider: {}", report.profile.id);
            println!("# source this output before running octopus llm/chat/need/evolve");
        }
        Language::Zh => {
            println!("# Octopus provider: {}", report.profile.id);
            println!("# 先 source 这些变量，再运行 octopus llm/chat/need/evolve");
        }
    }
    println!("{}", report.shell);
}

fn print_provider_check(report: &ProviderCheckReport, language: Language) {
    match language {
        Language::En => {
            println!("Provider check: ok");
            println!("prefix: {}", report.prefix);
            println!("model: {}", report.model);
            println!("base_url: {}", report.base_url);
            println!(
                "api_key: {}",
                if report.api_key_present {
                    "present"
                } else {
                    "empty"
                }
            );
            println!("reply: {}", report.content);
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("Provider 检查: ok");
            println!("前缀: {}", report.prefix);
            println!("模型: {}", report.model);
            println!("地址: {}", report.base_url);
            println!(
                "密钥: {}",
                if report.api_key_present {
                    "已设置"
                } else {
                    "空"
                }
            );
            println!("回复: {}", report.content);
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

fn print_provider_status(report: &ProviderStatusReport, language: Language) {
    match language {
        Language::En => println!("Provider status"),
        Language::Zh => println!("Provider 状态"),
    }
    for layer in &report.layers {
        let model = layer.model.as_deref().unwrap_or("none");
        let base_url = layer.base_url.as_deref().unwrap_or("none");
        match language {
            Language::En => println!(
                "- {}: enabled={} configured={} prefix={} model={} base_url={} api_key={} curl={}({}) check=\"{}\" message={}",
                layer.layer,
                layer.enabled,
                layer.configured,
                layer.prefix,
                model,
                base_url,
                if layer.api_key_present { "present" } else { "empty" },
                layer.curl_command,
                if layer.curl_available { "ok" } else { "missing" },
                layer.check_command,
                layer.message,
            ),
            Language::Zh => println!(
                "- {}: 启用={} 已配置={} 前缀={} 模型={} 地址={} 密钥={} curl={}({}) 检查=\"{}\" 说明={}",
                layer.layer,
                layer.enabled,
                layer.configured,
                layer.prefix,
                model,
                base_url,
                if layer.api_key_present { "已设置" } else { "空" },
                layer.curl_command,
                if layer.curl_available { "可用" } else { "缺失" },
                layer.check_command,
                layer.message,
            ),
        }
    }
    match language {
        Language::En => println!("next: {}", join_or_none(&report.next)),
        Language::Zh => println!("下一步: {}", join_or_none(&report.next)),
    }
}

fn run_bridge(addr: &str) -> Result<(), String> {
    let listener =
        TcpListener::bind(addr).map_err(|error| format!("bridge bind failed: {error}"))?;
    println!("Octopus bridge: http://{addr}");
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = handle_bridge_connection(&mut stream) {
                    let body = serde_json::json!({ "error": error }).to_string();
                    let _ =
                        write_http_response(&mut stream, 500, "application/json", body.as_bytes());
                }
            }
            Err(error) => eprintln!("bridge connection failed: {error}"),
        }
    }
    Ok(())
}

fn handle_bridge_connection(stream: &mut TcpStream) -> Result<(), String> {
    let request = read_http_request(stream)?;
    match (request.method.as_str(), request.path.as_str()) {
        ("OPTIONS", "/api/run" | "/api/stream") => {
            write_http_response(stream, 204, "text/plain", b"")
        }
        ("POST", "/api/run") => {
            let request = serde_json::from_slice::<BridgeRunRequest>(&request.body)
                .map_err(|error| format!("invalid bridge JSON: {error}"))?;
            let response = run_bridge_command(&request.args)?;
            let body = serde_json::to_vec_pretty(&response).map_err(|error| error.to_string())?;
            write_http_response(stream, 200, "application/json", &body)
        }
        ("POST", "/api/stream") => {
            let request = serde_json::from_slice::<BridgeRunRequest>(&request.body)
                .map_err(|error| format!("invalid bridge JSON: {error}"))?;
            stream_bridge_command(stream, &request.args)
        }
        ("GET", path) => {
            if let Ok((content_type, body)) = bridge_static(path) {
                write_http_response(stream, 200, content_type, &body)
            } else {
                write_http_response(stream, 404, "text/plain", b"not found")
            }
        }
        _ => write_http_response(stream, 404, "text/plain", b"not found"),
    }
}

struct HttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    let mut data = Vec::new();
    let mut buffer = [0_u8; 4096];
    let header_end = loop {
        let bytes = stream
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if bytes == 0 {
            return Err("empty bridge request".to_string());
        }
        data.extend_from_slice(&buffer[..bytes]);
        if let Some(index) = find_header_end(&data) {
            break index;
        }
        if data.len() > 64 * 1024 {
            return Err("bridge request headers too large".to_string());
        }
    };
    let header = String::from_utf8_lossy(&data[..header_end]).to_string();
    let content_length = http_content_length(&header)?;
    while data.len() < header_end + 4 + content_length {
        let bytes = stream
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if bytes == 0 {
            break;
        }
        data.extend_from_slice(&buffer[..bytes]);
    }
    let request_line = header
        .lines()
        .next()
        .ok_or_else(|| "missing request line".to_string())?;
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| "missing method".to_string())?
        .to_string();
    let path = parts
        .next()
        .ok_or_else(|| "missing path".to_string())?
        .split('?')
        .next()
        .unwrap_or("/")
        .to_string();
    let body_start = header_end + 4;
    let body_end = (body_start + content_length).min(data.len());
    Ok(HttpRequest {
        method,
        path,
        body: data[body_start..body_end].to_vec(),
    })
}

fn find_header_end(data: &[u8]) -> Option<usize> {
    data.windows(4).position(|window| window == b"\r\n\r\n")
}

fn http_content_length(header: &str) -> Result<usize, String> {
    header
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>())
        })
        .transpose()
        .map_err(|_| "invalid content-length".to_string())
        .map(|value| value.unwrap_or(0))
}

fn write_http_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        204 => "No Content",
        404 => "Not Found",
        _ => "Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\ncontent-type: {content_type}; charset=utf-8\r\ncontent-length: {}\r\naccess-control-allow-origin: *\r\naccess-control-allow-headers: content-type\r\naccess-control-allow-methods: GET, POST, OPTIONS\r\nconnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(header.as_bytes())
        .and_then(|_| stream.write_all(body))
        .map_err(|error| error.to_string())
}

fn write_http_stream_header(stream: &mut TcpStream) -> Result<(), String> {
    stream
        .write_all(
            b"HTTP/1.1 200 OK\r\ncontent-type: text/event-stream; charset=utf-8\r\ncache-control: no-cache\r\naccess-control-allow-origin: *\r\naccess-control-allow-headers: content-type\r\naccess-control-allow-methods: GET, POST, OPTIONS\r\nconnection: close\r\n\r\n",
        )
        .map_err(|error| error.to_string())
}

fn write_sse_event(
    stream: &mut TcpStream,
    event: &str,
    value: &serde_json::Value,
) -> Result<(), String> {
    let data = serde_json::to_string(value).map_err(|error| error.to_string())?;
    stream
        .write_all(format!("event: {event}\ndata: {data}\n\n").as_bytes())
        .and_then(|_| stream.flush())
        .map_err(|error| error.to_string())
}

fn bridge_static(path: &str) -> Result<(&'static str, Vec<u8>), String> {
    let file = match path {
        "/" | "/app.html" => "app.html",
        "/index.html" => "index.html",
        "/pet.html" => "pet.html",
        "/quickstart.html" => "quickstart.html",
        "/about.html" => "about.html",
        "/references.html" => "references.html",
        "/self-iteration.html" => "self-iteration.html",
        _ => return Err("bridge page not found".to_string()),
    };
    let path = repo_root().join("docs").join(file);
    fs::read(path)
        .map(|body| ("text/html", body))
        .map_err(|error| format!("bridge static file unavailable: {error}"))
}

fn run_bridge_command(args: &[String]) -> Result<BridgeRunResponse, String> {
    if !bridge_command_allowed(args) {
        return Err("bridge command not allowed".to_string());
    }
    let output = Command::new(env::current_exe().map_err(|error| error.to_string())?)
        .args(args)
        .output()
        .map_err(|error| format!("bridge command failed to start: {error}"))?;
    Ok(BridgeRunResponse {
        ok: output.status.success(),
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

fn stream_bridge_command(stream: &mut TcpStream, args: &[String]) -> Result<(), String> {
    if !bridge_command_allowed(args) {
        return Err("bridge command not allowed".to_string());
    }
    write_http_stream_header(stream)?;
    write_sse_event(
        stream,
        "start",
        &serde_json::json!({ "args": args, "summary": "started" }),
    )?;
    let mut child = Command::new(env::current_exe().map_err(|error| error.to_string())?)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("bridge command failed to start: {error}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "bridge command stdout unavailable".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "bridge command stderr unavailable".to_string())?;
    let (sender, receiver) = mpsc::channel::<(String, String)>();
    spawn_stream_reader("stdout", stdout, sender.clone());
    spawn_stream_reader("stderr", stderr, sender);
    loop {
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok((kind, text)) => {
                write_sse_event(stream, &kind, &serde_json::json!({ "text": text }))?;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if child
                    .try_wait()
                    .map_err(|error| error.to_string())?
                    .is_some()
                {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if child
                    .try_wait()
                    .map_err(|error| error.to_string())?
                    .is_some()
                {
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
    let status = child.wait().map_err(|error| error.to_string())?;
    while let Ok((kind, text)) = receiver.try_recv() {
        write_sse_event(stream, &kind, &serde_json::json!({ "text": text }))?;
    }
    write_sse_event(
        stream,
        "done",
        &serde_json::json!({
            "ok": status.success(),
            "code": status.code(),
        }),
    )
}

fn spawn_stream_reader<R>(kind: &str, reader: R, sender: mpsc::Sender<(String, String)>)
where
    R: Read + Send + 'static,
{
    let kind = kind.to_string();
    thread::spawn(move || {
        for text in BufReader::new(reader).lines().map_while(Result::ok) {
            let _ = sender.send((kind.clone(), text));
        }
    });
}

fn bridge_command_allowed(args: &[String]) -> bool {
    let Some(command) = bridge_command_name(args) else {
        return false;
    };
    if command == "oauth" {
        return bridge_oauth_allowed(args);
    }
    if command == "check" {
        return bridge_check_allowed(args);
    }
    if command == "self-iterate" && args.iter().any(|arg| arg == "pr") {
        return false;
    }
    matches!(
        command,
        "init"
            | "chat"
            | "think"
            | "need"
            | "pet"
            | "doctor"
            | "beat"
            | "status"
            | "goal"
            | "installed"
            | "install"
            | "skills"
            | "catalog"
            | "manifests"
            | "providers"
            | "provider"
            | "routes"
            | "traces"
            | "env"
            | "adapt"
            | "self-iterate"
    )
}

fn bridge_check_allowed(args: &[String]) -> bool {
    let Some(index) = bridge_command_index(args) else {
        return false;
    };
    let Some(tentacle) = args.get(index + 1) else {
        return false;
    };
    (args.len() == index + 2
        || (args.len() == index + 3
            && args
                .get(index + 2)
                .is_some_and(|value| parse_check_index(value).is_ok())))
        && matches!(
            tentacle.as_str(),
            "swe-agent"
                | "computer-use-agent"
                | "bash-only"
                | "repo-maintainer"
                | "json-feed"
                | "visual"
        )
}

fn bridge_oauth_allowed(args: &[String]) -> bool {
    let Some(index) = bridge_command_index(args) else {
        return false;
    };
    args.get(index).is_some_and(|command| command == "oauth")
        && args
            .get(index + 1)
            .is_some_and(|provider| provider == "octopus")
        && args
            .get(index + 2)
            .is_some_and(|scope| scope.starts_with("tool:"))
        && args.get(index + 3).is_some()
        && args
            .iter()
            .skip(index + 3)
            .all(|permission| permission.starts_with("tool:"))
}

fn bridge_command_name(args: &[String]) -> Option<&str> {
    bridge_command_index(args).and_then(|index| args.get(index).map(String::as_str))
}

fn bridge_command_index(args: &[String]) -> Option<usize> {
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--state" | "--lang" => index += 2,
            "--json" => index += 1,
            value if value.starts_with('-') => return None,
            _ => return Some(index),
        }
    }
    None
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
# You can also generate this file with: octopus provider openai > llm.env
export OCTOPUS_LLM_MODEL=gpt-4.1-mini
export OCTOPUS_LLM_BASE_URL=https://api.openai.com/v1
export OCTOPUS_LLM_API_KEY=
export OCTOPUS_CHAT_LLM=1
export OCTOPUS_LLM_MANIFEST=1
export OCTOPUS_LLM_EVOLVE=1
# Optional: point chat, tentacle planning, or evolution at another OpenAI-compatible env prefix.
# export OCTOPUS_CHAT_LLM_PREFIX=OCTOPUS_LLM
# export OCTOPUS_MANIFEST_LLM_PREFIX=OCTOPUS_LLM
# export OCTOPUS_EVOLVE_LLM_PREFIX=OCTOPUS_LLM
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
    let status = if requested == "auto" {
        Some(state.status_report_with_state(Some(state_path)))
    } else {
        None
    };
    let pet_state = if let Some(status) = &status {
        auto_pet_state(status)
    } else {
        requested.to_string()
    };
    let mut report = pet_report(&pet_state)?;
    if let Some(event) = status.and_then(|status| status.last_pet_event) {
        if event.state == report.state {
            report.event_source = Some(event.source);
            report.event_summary = Some(event.summary);
        }
    }
    Ok(report)
}

fn auto_pet_state(report: &StatusReport) -> String {
    if report
        .goal
        .as_ref()
        .is_some_and(|goal| goal.status == octopus_core::GoalStatus::Blocked)
        || report.tentacles.is_empty()
    {
        return "blocked".to_string();
    }
    if report
        .goal
        .as_ref()
        .is_some_and(|goal| goal.status == octopus_core::GoalStatus::Satisfied)
    {
        return "success".to_string();
    }
    if let Some(event) = &report.last_pet_event {
        if pet_state_info(&event.state).is_ok() {
            return event.state.clone();
        }
    }
    if report.route_count > 0 {
        return "harness".to_string();
    }
    if report.memory_count > 0 {
        return "memory".to_string();
    }
    "heartbeat".to_string()
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
        event_source: None,
        event_summary: None,
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
            println!("feed_traces: {}", report.feed_trace_count);
            if let Some(trace) = &report.latest_feed_trace {
                println!("latest_trace: {}", trace_line(trace));
            }
            println!("check_history: {}", report.check_history_count);
            if let Some(record) = &report.latest_check {
                println!("latest_check: {}", check_history_line(record));
            }
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
            println!("Feed轨迹数: {}", report.feed_trace_count);
            if let Some(trace) = &report.latest_feed_trace {
                println!("最近轨迹: {}", trace_line(trace));
            }
            println!("检查历史数: {}", report.check_history_count);
            if let Some(record) = &report.latest_check {
                println!("最近检查: {}", check_history_line(record));
            }
            println!("警告: {}", join_or_none(&report.warnings));
            println!("下一步: {}", report.next_action);
        }
    }
}

fn print_feed_traces(traces: &[FeedTraceRecord], language: Language) {
    match language {
        Language::En => println!("Feed traces"),
        Language::Zh => println!("Feed轨迹"),
    }
    if traces.is_empty() {
        match language {
            Language::En => println!("none"),
            Language::Zh => println!("暂无"),
        }
        return;
    }
    for trace in traces {
        println!("{}", trace_line(trace));
    }
}

fn trace_line(trace: &FeedTraceRecord) -> String {
    let tentacle = trace.tentacle.as_deref().unwrap_or("none");
    let tool = trace.tool.as_deref().unwrap_or("none");
    let plan = trace.plan_source.as_deref().unwrap_or("none");
    format!(
        "#{} {}:{} -> {:?} via {}/{} plan={} evidence={} :: {}",
        trace.index,
        need_label(&trace.need_kind),
        trace.need_query,
        trace.status,
        tentacle,
        tool,
        plan,
        trace.evidence_count,
        trace.summary
    )
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
    next.push("octopus provider status".to_string());
    if !llm.configured {
        next.push("octopus providers".to_string());
        next.push("octopus provider openai > .octopus/llm.env".to_string());
    } else if llm.curl_available {
        next.push(format!("octopus provider check {}", llm.config_prefix));
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
            message: "provider env configured; run provider check for live validation".to_string(),
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

fn publish_self_iteration_pr(plan: &SelfIterationPlan) -> Result<SelfIterationPrReport, String> {
    if !plan.authorized {
        return Err(format!(
            "self-iterate pr requires github grant for {}; run: octopus oauth github {}",
            plan.repository, plan.repository
        ));
    }
    let grant_id = plan
        .grant_id
        .clone()
        .ok_or_else(|| "self-iterate pr missing grant id".to_string())?;
    let draft = plan
        .draft
        .as_ref()
        .ok_or_else(|| "self-iterate pr missing draft".to_string())?;
    let root = env::current_dir().map_err(|error| error.to_string())?;
    let out_dir = root.join(".octopus/self-iteration");
    fs::create_dir_all(&out_dir).map_err(|error| error.to_string())?;
    let body_path = out_dir.join("PR_BODY.md");
    fs::write(&body_path, &draft.body).map_err(|error| error.to_string())?;
    let script = repo_root().join("tentacles/repo-maintainer/tools/publish_pr.sh");
    if !script.exists() {
        return Err(format!("publish_pr tool missing: {}", script.display()));
    }

    let output = Command::new(&script)
        .arg(&root)
        .arg(&plan.repository)
        .arg(&draft.branch)
        .arg(&draft.title)
        .arg(&body_path)
        .output()
        .map_err(|error| format!("publish_pr failed to start: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        let message = if stderr.trim().is_empty() {
            stdout
        } else {
            stderr
        };
        return Err(trim_cli_output(&message));
    }
    Ok(SelfIterationPrReport {
        repository: plan.repository.clone(),
        mode: if env_flag("OCTOPUS_PR_DRY_RUN") {
            "dry-run".to_string()
        } else {
            "published".to_string()
        },
        grant_id,
        branch: draft.branch.clone(),
        title: draft.title.clone(),
        body_path: body_path.to_string_lossy().to_string(),
        publish_path: extract_key(&stdout, "publish").or_else(|| extract_key(&stdout, "pr_url")),
        output: trim_cli_output(&stdout),
        next: vec![
            "watch GitHub checks".to_string(),
            "score the result with evolve score after review".to_string(),
        ],
    })
}

fn print_self_iteration_pr_report(report: &SelfIterationPrReport, language: Language) {
    match language {
        Language::En => {
            println!("self-iteration pr: {}", report.mode);
            println!("repo: {}", report.repository);
            println!("grant: {}", report.grant_id);
            println!("branch: {}", report.branch);
            println!("title: {}", report.title);
            if let Some(path) = &report.publish_path {
                println!("publish: {path}");
            }
            println!("body: {}", report.body_path);
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("自迭代 PR: {}", report.mode);
            println!("仓库: {}", report.repository);
            println!("授权: {}", report.grant_id);
            println!("分支: {}", report.branch);
            println!("标题: {}", report.title);
            if let Some(path) = &report.publish_path {
                println!("发布: {path}");
            }
            println!("正文: {}", report.body_path);
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

fn extract_key(output: &str, key: &str) -> Option<String> {
    output.lines().find_map(|line| {
        line.strip_prefix(&format!("{key}="))
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn env_flag(key: &str) -> bool {
    env::var(key)
        .map(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

fn trim_cli_output(output: &str) -> String {
    let output = output.trim();
    if output.len() <= 500 {
        output.to_string()
    } else {
        format!("{}...", output.chars().take(500).collect::<String>())
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
            println!("feed traces: {}", proposal.recent_feed_traces.len());
            println!("check history: {}", proposal.recent_check_history.len());
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
            println!("Feed轨迹: {}", proposal.recent_feed_traces.len());
            println!("检查历史: {}", proposal.recent_check_history.len());
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

fn print_evolution_recommendation(
    recommendation: &EvolutionRecommendation,
    artifact: &EvolutionApplyArtifact,
    language: Language,
) {
    match language {
        Language::En => {
            println!("tentacle: {}", recommendation.tentacle_id);
            println!("recommended: {}", recommendation.candidate_id);
            println!("surface: {}", recommendation.surface_id);
            println!("title: {}", recommendation.candidate_title);
            println!("score: {:.2}", recommendation.recommendation_score);
            println!("outcomes: {}", recommendation.outcome_count);
            println!("check history: {}", recommendation.check_history_count);
            println!("reason: {}", recommendation.reason);
            println!("status: {}", recommendation.apply.status);
            println!("plan: {}", artifact.plan_path);
            if let Some(path) = &artifact.patch_path {
                println!("patch: {path}");
            }
        }
        Language::Zh => {
            println!("触手: {}", recommendation.tentacle_id);
            println!("推荐: {}", recommendation.candidate_id);
            println!("可进化面: {}", recommendation.surface_id);
            println!("标题: {}", recommendation.candidate_title);
            println!("分数: {:.2}", recommendation.recommendation_score);
            println!("历史结果: {}", recommendation.outcome_count);
            println!("检查历史: {}", recommendation.check_history_count);
            println!("原因: {}", recommendation.reason);
            println!("状态: {}", recommendation.apply.status);
            println!("计划: {}", artifact.plan_path);
            if let Some(path) = &artifact.patch_path {
                println!("补丁: {path}");
            }
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
            if let Some(trace) = feed_trace(feed) {
                println!("feed_trace: {trace}");
            }
            println!("evidence: {}", feed.evidence.len());
            println!("metadata: {}", probe_metadata(feed));
        }
        Language::Zh => {
            println!("状态: {:?}", feed.status);
            println!("摘要: {}", localize_summary(&feed.summary, language));
            if let Some(trace) = feed_trace(feed) {
                println!("Feed轨迹: {trace}");
            }
            println!("证据: {}", feed.evidence.len());
            println!("元数据: {}", probe_metadata(feed));
        }
    }
}

fn print_thinking_plan(plan: &TentacleThinkingPlan, language: Language) {
    match language {
        Language::En => {
            println!("Octopus think");
            println!("tentacle: {}", plan.tentacle_id);
            println!("brain: {}", plan.brain_kind);
            println!(
                "need: {} {}",
                need_kind_label(&plan.need.kind),
                plan.need.query
            );
            println!("selected_tool: {}", plan.selected_tool.id);
            println!("plan_source: {}", plan.plan_source);
            println!("reason: {}", plan.reason);
            println!("actions: {}", thinking_actions(&plan.actions));
            println!(
                "authorization: {}",
                thinking_authorization(&plan.selected_tool)
            );
            println!("candidates: {}", thinking_candidates(&plan.candidates));
            println!("context: {}", plan.context_policy);
        }
        Language::Zh => {
            println!("章鱼触手思考");
            println!("触手: {}", plan.tentacle_id);
            println!("大脑: {}", plan.brain_kind);
            println!(
                "需求: {} {}",
                need_kind_label(&plan.need.kind),
                plan.need.query
            );
            println!("选择工具: {}", plan.selected_tool.id);
            println!("规划来源: {}", plan.plan_source);
            println!("原因: {}", plan.reason);
            println!("动作: {}", thinking_actions(&plan.actions));
            println!("授权: {}", thinking_authorization(&plan.selected_tool));
            println!("候选工具: {}", thinking_candidates(&plan.candidates));
            println!("上下文: {}", plan.context_policy);
        }
    }
}

fn thinking_candidates(candidates: &[TentacleToolCandidate]) -> String {
    if candidates.is_empty() {
        return "none".to_string();
    }
    candidates
        .iter()
        .map(|candidate| candidate.id.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn thinking_actions(actions: &[TentacleToolAction]) -> String {
    if actions.is_empty() {
        return "none".to_string();
    }
    actions
        .iter()
        .enumerate()
        .map(|(index, action)| {
            if action.reason.trim().is_empty() {
                format!("{}:{}", index + 1, action.tool.id)
            } else {
                format!("{}:{}({})", index + 1, action.tool.id, action.reason)
            }
        })
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn thinking_authorization(candidate: &TentacleToolCandidate) -> String {
    if let Some(grant) = &candidate.active_grant {
        return format!("active {grant}");
    }
    if candidate.authorization_required {
        return candidate
            .permission
            .as_ref()
            .map(|permission| format!("required {permission}"))
            .unwrap_or_else(|| "required".to_string());
    }
    "not_required".to_string()
}

fn need_kind_label(kind: &NeedKind) -> String {
    format!("{kind:?}").to_ascii_lowercase()
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

fn feed_trace(feed: &Feed) -> Option<String> {
    let tentacle = feed.metadata.get("tentacle_brain")?;
    let tool = feed.metadata.get("tool")?;
    let source = feed
        .metadata
        .get("plan_source")
        .map(String::as_str)
        .unwrap_or("unknown");
    Some(format!("{tentacle}/{tool} via {source}"))
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

fn install_report(
    id: &str,
    state_path: &Path,
    profile_installed: bool,
    manifest_installed: Option<&InstalledTentacle>,
    profile: Option<&TentacleProfile>,
    manifest: Option<&LoadedTentacleManifest>,
    state: &HarnessState,
) -> InstallReport {
    let source_kind = match (profile_installed, manifest_installed.is_some()) {
        (true, true) => "profile+manifest",
        (true, false) => "profile",
        (false, true) => "manifest",
        (false, false) => "unknown",
    }
    .to_string();

    let name = manifest_installed
        .map(|tentacle| tentacle.name.clone())
        .or_else(|| profile.map(|profile| profile.name.clone()))
        .unwrap_or_else(|| title_from_id(id));
    let description = manifest
        .map(|loaded| loaded.manifest.description.clone())
        .or_else(|| profile.map(|profile| profile.description.clone()))
        .unwrap_or_default();
    let brain_kind = manifest_installed
        .map(|tentacle| tentacle.brain_kind.clone())
        .or_else(|| profile.map(|profile| profile.brain.kind.clone()))
        .unwrap_or_else(|| "unknown".to_string());
    let needs = manifest_installed
        .map(|tentacle| tentacle.needs.clone())
        .or_else(|| profile.map(profile_needs))
        .unwrap_or_default();
    let tools = manifest_installed
        .map(|tentacle| {
            tentacle
                .tool_meta
                .iter()
                .map(|tool| format!("{}:{}", tool.id, tool.kind))
                .collect::<Vec<_>>()
        })
        .or_else(|| profile.map(profile_tools))
        .unwrap_or_default();
    let runtimes = manifest_installed
        .map(|tentacle| tentacle.runtime_kinds.clone())
        .or_else(|| profile.map(profile_runtimes))
        .unwrap_or_default();
    let evolution_surfaces = manifest_installed
        .map(|tentacle| tentacle.evolution_surfaces.clone())
        .or_else(|| profile.map(profile_surfaces))
        .unwrap_or_default();
    let grants = install_grants(state_path, state, manifest_installed, profile);
    let checks = install_checks(manifest, profile);
    let check_history = state.recent_check_history_for_tentacle(id, 8);
    let next = install_next_commands(
        id,
        state_path,
        manifest_installed.is_some(),
        &needs,
        &grants,
    );

    InstallReport {
        id: id.to_string(),
        name,
        source_kind,
        state_path: state_path.to_string_lossy().to_string(),
        description,
        brain_kind,
        installed: profile_installed || manifest_installed.is_some(),
        needs,
        tools,
        runtimes,
        evolution_surfaces,
        grants,
        checks,
        check_history,
        next,
    }
}

fn profile_needs(profile: &TentacleProfile) -> Vec<String> {
    let mut needs = profile
        .skills
        .iter()
        .flat_map(|skill| skill.needs.iter().map(need_label))
        .collect::<Vec<_>>();
    needs.sort();
    needs.dedup();
    needs
}

fn profile_tools(profile: &TentacleProfile) -> Vec<String> {
    profile
        .tools
        .iter()
        .map(|tool| format!("{}:{}", tool.id, tool.implementation.kind))
        .collect()
}

fn profile_runtimes(profile: &TentacleProfile) -> Vec<String> {
    let mut runtimes = profile
        .tools
        .iter()
        .map(|tool| tool.implementation.kind.clone())
        .collect::<Vec<_>>();
    runtimes.sort();
    runtimes.dedup();
    runtimes
}

fn profile_surfaces(profile: &TentacleProfile) -> Vec<String> {
    profile
        .evolution
        .surfaces
        .iter()
        .map(|surface| surface.id.clone())
        .collect()
}

fn install_grants(
    state_path: &Path,
    state: &HarnessState,
    manifest: Option<&InstalledTentacle>,
    profile: Option<&TentacleProfile>,
) -> Vec<InstallGrantReport> {
    let mut grouped = InstallGrantGroups::new();
    if let Some(tentacle) = manifest {
        for permission in tentacle
            .tool_meta
            .iter()
            .filter_map(|tool| tool.permission.as_ref())
        {
            collect_install_grant(&mut grouped, permission);
        }
    } else if let Some(profile) = profile {
        for permission in profile
            .tools
            .iter()
            .filter_map(|tool| tool.permission.as_ref())
        {
            collect_install_grant(&mut grouped, permission);
        }
    }
    grouped
        .into_iter()
        .map(|((provider, scope), (permissions, reasons))| {
            let permission = ToolPermission {
                provider,
                scope,
                permissions: permissions.into_iter().collect(),
                reason: (!reasons.is_empty())
                    .then(|| reasons.into_iter().collect::<Vec<_>>().join("; ")),
            };
            install_grant_report(state_path, state, permission)
        })
        .collect()
}

fn collect_install_grant(grouped: &mut InstallGrantGroups, permission: &ToolPermission) {
    let entry = grouped
        .entry((permission.provider.clone(), permission.scope.clone()))
        .or_default();
    for required in &permission.permissions {
        entry.0.insert(required.clone());
    }
    if let Some(reason) = &permission.reason {
        entry.1.insert(reason.clone());
    }
}

fn install_grant_report(
    state_path: &Path,
    state: &HarnessState,
    permission: ToolPermission,
) -> InstallGrantReport {
    let command = oauth_command(state_path, &permission);
    let active = state
        .active_grant(&permission.provider, &permission.scope)
        .is_some_and(|grant| {
            permission
                .permissions
                .iter()
                .all(|required| grant.permissions.iter().any(|owned| owned == required))
        });
    InstallGrantReport {
        provider: permission.provider.clone(),
        scope: permission.scope.clone(),
        permissions: permission.permissions.clone(),
        reason: permission.reason.clone(),
        active,
        command,
    }
}

fn oauth_command(state_path: &Path, permission: &ToolPermission) -> String {
    let mut parts = vec![
        "octopus".to_string(),
        "--state".to_string(),
        shell_arg(&state_path.to_string_lossy()),
        "oauth".to_string(),
        shell_arg(&permission.provider),
        shell_arg(&permission.scope),
    ];
    parts.extend(
        permission
            .permissions
            .iter()
            .map(|permission| shell_arg(permission)),
    );
    parts.join(" ")
}

fn install_checks(
    manifest: Option<&LoadedTentacleManifest>,
    profile: Option<&TentacleProfile>,
) -> Vec<String> {
    if let Some(manifest) = manifest {
        let directory = Path::new(&manifest.path)
            .parent()
            .map(|path| shell_arg(&path.to_string_lossy()))
            .unwrap_or_else(|| ".".to_string());
        return manifest
            .manifest
            .evolution
            .checks
            .iter()
            .map(|check| format!("(cd {directory} && {check})"))
            .collect();
    }
    profile
        .map(|profile| profile.evolution.checks.clone())
        .unwrap_or_default()
}

fn install_next_commands(
    id: &str,
    state_path: &Path,
    has_manifest: bool,
    needs: &[String],
    grants: &[InstallGrantReport],
) -> Vec<String> {
    let prefix = format!(
        "octopus --state {}",
        shell_arg(&state_path.to_string_lossy())
    );
    let mut commands = grants
        .iter()
        .filter(|grant| !grant.active)
        .map(|grant| grant.command.clone())
        .collect::<Vec<_>>();
    let sample_need = sample_need(needs);
    let sample_query = if sample_need == "execute" {
        shell_arg("echo octopus")
    } else {
        ".".to_string()
    };
    if has_manifest {
        commands.push(format!(
            "{prefix} think {} {sample_need} {sample_query}",
            shell_arg(id)
        ));
    }
    commands.push(format!("{prefix} need {sample_need} {sample_query}"));
    commands.push(format!("{prefix} doctor"));
    commands
}

fn check_report(tentacle_id: &str, selected: Option<usize>) -> Result<CheckReport, String> {
    let root = default_tentacles_root();
    if let Some(loaded) = load_tentacle_manifests(&root)
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|loaded| loaded.manifest.id == tentacle_id)
    {
        let cwd = Path::new(&loaded.path)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or(root);
        let checks = selected_checks(&loaded.manifest.evolution.checks, selected)?;
        let results = run_check_commands(&cwd, &checks);
        let passed = results
            .iter()
            .all(|result| result.status == Status::Satisfied);
        return Ok(CheckReport {
            id: loaded.manifest.id,
            name: loaded.manifest.name,
            source_kind: "manifest".to_string(),
            cwd: cwd.to_string_lossy().to_string(),
            passed,
            results,
            history: Vec::new(),
        });
    }

    let Some(profile) = default_tentacle_profiles()
        .into_iter()
        .find(|profile| profile.id == tentacle_id)
    else {
        return Err(format!(
            "unknown profile or tentacle manifest: {tentacle_id}"
        ));
    };
    let cwd = env::current_dir().map_err(|error| error.to_string())?;
    let checks = selected_checks(&profile.evolution.checks, selected)?;
    let results = run_check_commands(&cwd, &checks);
    let passed = results
        .iter()
        .all(|result| result.status == Status::Satisfied);
    Ok(CheckReport {
        id: profile.id,
        name: profile.name,
        source_kind: "profile".to_string(),
        cwd: cwd.to_string_lossy().to_string(),
        passed,
        results,
        history: Vec::new(),
    })
}

fn record_check_report(state: &mut HarnessState, report: &mut CheckReport) {
    for result in &report.results {
        state.record_check_history(CheckHistoryInput {
            tentacle_id: report.id.clone(),
            source_kind: report.source_kind.clone(),
            command_index: Some(result.index),
            command: result.command.clone(),
            cwd: result.cwd.clone(),
            status: result.status.clone(),
            code: result.code,
            stdout: result.stdout.clone(),
            stderr: result.stderr.clone(),
        });
    }
    let summary = if report.passed {
        format!("check {} passed", report.id)
    } else {
        format!("check {} failed", report.id)
    };
    state.record_pet_event(
        if report.passed { "success" } else { "blocked" },
        format!("check:{}", report.id),
        summary,
        if report.passed {
            Status::Satisfied
        } else {
            Status::Failed
        },
    );
    report.history = state.recent_check_history_for_tentacle(&report.id, 8);
}

fn parse_check_index(value: &str) -> Result<usize, String> {
    let index = value
        .parse::<usize>()
        .map_err(|_| format!("invalid check index: {value}"))?;
    if index == 0 {
        return Err("check index starts at 1".to_string());
    }
    Ok(index)
}

fn selected_checks(
    checks: &[String],
    selected: Option<usize>,
) -> Result<Vec<SelectedCheck>, String> {
    let Some(index) = selected else {
        return Ok(checks
            .iter()
            .enumerate()
            .map(|(offset, command)| SelectedCheck {
                index: offset + 1,
                command: command.clone(),
            })
            .collect());
    };
    let Some(check) = checks.get(index - 1) else {
        return Err(format!(
            "check index {index} out of range; available checks: {}",
            checks.len()
        ));
    };
    Ok(vec![SelectedCheck {
        index,
        command: check.clone(),
    }])
}

fn run_check_commands(cwd: &Path, checks: &[SelectedCheck]) -> Vec<CheckResultReport> {
    checks
        .iter()
        .map(|check| run_check_command(cwd, check))
        .collect()
}

fn run_check_command(cwd: &Path, check: &SelectedCheck) -> CheckResultReport {
    let output = shell_check_command(&check.command)
        .current_dir(cwd)
        .output();
    match output {
        Ok(output) => CheckResultReport {
            index: check.index,
            command: check.command.clone(),
            cwd: cwd.to_string_lossy().to_string(),
            status: if output.status.success() {
                Status::Satisfied
            } else {
                Status::Failed
            },
            code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        },
        Err(error) => CheckResultReport {
            index: check.index,
            command: check.command.clone(),
            cwd: cwd.to_string_lossy().to_string(),
            status: Status::Failed,
            code: None,
            stdout: String::new(),
            stderr: error.to_string(),
        },
    }
}

fn shell_check_command(check: &str) -> Command {
    #[cfg(windows)]
    {
        let mut command = Command::new("cmd");
        command.args(["/C", check]);
        command
    }
    #[cfg(not(windows))]
    {
        let mut command = Command::new("sh");
        command.args(["-lc", check]);
        command
    }
}

fn sample_need(needs: &[String]) -> String {
    if needs.iter().any(|need| need == "observe") {
        "observe".to_string()
    } else if needs.iter().any(|need| need == "verify") {
        "verify".to_string()
    } else if needs.iter().any(|need| need == "execute") {
        "execute".to_string()
    } else {
        needs
            .first()
            .cloned()
            .unwrap_or_else(|| "observe".to_string())
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
    "usage: octopus [--version] [--state path] [--lang en|zh] [--json] init [tentacles-root] | need <kind> <query> | think <tentacle> <kind> <query> | chat <message> | llm <message> | providers | provider <profile> [prefix] | provider status | provider check [prefix] [message] | bridge [addr] | demo [repo] | goal | status | doctor | pet [state] | beat [memory_keep] | oauth <provider> <scope> [permissions...] | oauth revoke <grant> | self-iterate <repo> | self-iterate pr <repo> [objective] | evolve <tentacle> <objective> | evolve recommend <tentacle> [objective] | evolve apply <tentacle> <candidate> [objective] | evolve score <tentacle> <candidate> <status> [summary] | scaffold <tentacle> [runtime] | probe <tentacle> <kind> <query> | traces [limit] | routes | catalog | skills [root] | manifests [root] | env | adapt [root] | install <profile> | check <tentacle> [index] | installed".to_string()
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
        bridge_command_allowed, bridge_command_name, check_report, http_content_length,
        install_report, is_broken_pipe_panic, localize_summary, percent_encode_path, pet_report,
        pet_report_for_state, provider_status_report, run, skill_reports, usage, Language,
    };
    use octopus_core::{
        default_tentacle_profiles, load_tentacle_manifests, CheckHistoryInput, Feed, Goal,
        GoalStatus, HarnessState, Need, NeedKind, Status,
    };
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
    fn cli_treats_stdout_broken_pipe_as_clean_exit() {
        let payload: &(dyn std::any::Any + Send) =
            &"failed printing to stdout: Broken pipe (os error 32)";
        assert!(is_broken_pipe_panic(payload));
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
    fn cli_beat_writes_harness_evolution_hint_from_failed_check() {
        let _env = env_guard();
        let _cwd = CwdGuard::new();
        let workspace =
            std::env::temp_dir().join(format!("octopus-beat-evolution-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        fs::create_dir_all(&workspace).unwrap();
        std::env::set_current_dir(&workspace).unwrap();
        let state_path = workspace.join("state.json");
        let state_arg = state_path.to_string_lossy().to_string();
        let mut state = HarnessState::default();
        state.record_check_history(CheckHistoryInput {
            tentacle_id: "swe-agent".to_string(),
            source_kind: "manifest".to_string(),
            command_index: Some(1),
            command: "tools/read.sh README.md 1 2".to_string(),
            cwd: "tentacles/swe-agent".to_string(),
            status: Status::Failed,
            code: Some(1),
            stdout: String::new(),
            stderr: "range output failed".to_string(),
        });
        state.save(&state_path).unwrap();

        run(vec![
            "--state".to_string(),
            state_arg,
            "beat".to_string(),
            "200".to_string(),
        ])
        .unwrap();

        let restored = HarnessState::load(&state_path).unwrap();
        assert_eq!(
            restored.last_pet_event.as_ref().unwrap().source,
            "harness beat"
        );
        assert_eq!(restored.last_pet_event.as_ref().unwrap().state, "harness");
        let plan = workspace
            .join(".octopus")
            .join("evolution")
            .join("swe-agent")
            .join("apply")
            .join("03-runtime-code.md");
        assert!(plan.exists());
        let plan_content = fs::read_to_string(plan).unwrap();
        assert!(plan_content.contains("feedback focus:"));
        assert!(plan_content.contains("range output failed"));
        let _ = fs::remove_dir_all(workspace);
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
        run(vec!["providers".to_string()]).unwrap();
        run(vec!["--json".to_string(), "providers".to_string()]).unwrap();
        run(vec!["provider".to_string(), "status".to_string()]).unwrap();
        run(vec![
            "--json".to_string(),
            "provider".to_string(),
            "status".to_string(),
        ])
        .unwrap();
        run(vec!["provider".to_string(), "openai".to_string()]).unwrap();
        run(vec![
            "--json".to_string(),
            "provider".to_string(),
            "local".to_string(),
            "OCTOPUS_LOCAL".to_string(),
        ])
        .unwrap();
        assert!(run(vec![
            "provider".to_string(),
            "openai".to_string(),
            "octopus-local".to_string(),
        ])
        .is_err());
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
            "think".to_string(),
            "computer-use-agent".to_string(),
            "observe".to_string(),
            "current".to_string(),
            "browser".to_string(),
            "tab".to_string(),
        ])
        .unwrap();
        run(vec![
            "--json".to_string(),
            "think".to_string(),
            "swe-agent".to_string(),
            "observe".to_string(),
            "README.md".to_string(),
        ])
        .unwrap();
        run(vec![
            "--lang".to_string(),
            "zh".to_string(),
            "env".to_string(),
        ])
        .unwrap();
        assert!(usage().contains("bridge [addr]"));
        assert!(usage().contains("think <tentacle> <kind> <query>"));
        assert!(usage().contains("provider status"));
        assert!(usage().contains("traces [limit]"));
    }

    #[test]
    fn provider_status_reports_all_llm_layers() {
        let _env = env_guard();
        let old_chat = std::env::var("OCTOPUS_CHAT_LLM").ok();
        let old_manifest = std::env::var("OCTOPUS_LLM_MANIFEST").ok();
        let old_evolve = std::env::var("OCTOPUS_LLM_EVOLVE").ok();
        let old_chat_prefix = std::env::var("OCTOPUS_CHAT_LLM_PREFIX").ok();
        let old_manifest_prefix = std::env::var("OCTOPUS_MANIFEST_LLM_PREFIX").ok();
        let old_evolve_prefix = std::env::var("OCTOPUS_EVOLVE_LLM_PREFIX").ok();
        let old_model = std::env::var("OCTOPUS_STATUS_TEST_MODEL").ok();
        let old_base_url = std::env::var("OCTOPUS_STATUS_TEST_BASE_URL").ok();
        let old_api_key = std::env::var("OCTOPUS_STATUS_TEST_API_KEY").ok();
        std::env::set_var("OCTOPUS_CHAT_LLM", "1");
        std::env::set_var("OCTOPUS_LLM_MANIFEST", "1");
        std::env::set_var("OCTOPUS_LLM_EVOLVE", "1");
        std::env::set_var("OCTOPUS_CHAT_LLM_PREFIX", "OCTOPUS_STATUS_TEST");
        std::env::set_var("OCTOPUS_MANIFEST_LLM_PREFIX", "OCTOPUS_STATUS_TEST");
        std::env::set_var("OCTOPUS_EVOLVE_LLM_PREFIX", "OCTOPUS_STATUS_TEST");
        std::env::set_var("OCTOPUS_STATUS_TEST_MODEL", "test-model");
        std::env::set_var("OCTOPUS_STATUS_TEST_BASE_URL", "https://llm.example/v1");
        std::env::remove_var("OCTOPUS_STATUS_TEST_API_KEY");

        let report = provider_status_report();

        assert_eq!(report.layers.len(), 3);
        assert!(report
            .layers
            .iter()
            .all(|layer| layer.enabled && layer.configured));
        assert!(report.layers.iter().any(|layer| {
            layer.layer == "tentacle_planning"
                && layer.prefix == "OCTOPUS_STATUS_TEST"
                && layer.model.as_deref() == Some("test-model")
        }));
        assert!(report
            .next
            .contains(&"octopus provider check OCTOPUS_STATUS_TEST".to_string()));

        restore_env("OCTOPUS_CHAT_LLM", old_chat);
        restore_env("OCTOPUS_LLM_MANIFEST", old_manifest);
        restore_env("OCTOPUS_LLM_EVOLVE", old_evolve);
        restore_env("OCTOPUS_CHAT_LLM_PREFIX", old_chat_prefix);
        restore_env("OCTOPUS_MANIFEST_LLM_PREFIX", old_manifest_prefix);
        restore_env("OCTOPUS_EVOLVE_LLM_PREFIX", old_evolve_prefix);
        restore_env("OCTOPUS_STATUS_TEST_MODEL", old_model);
        restore_env("OCTOPUS_STATUS_TEST_BASE_URL", old_base_url);
        restore_env("OCTOPUS_STATUS_TEST_API_KEY", old_api_key);
    }

    #[test]
    fn bridge_allows_only_octopus_commands() {
        assert_eq!(
            bridge_command_name(&[
                "--state".to_string(),
                "state.json".to_string(),
                "--json".to_string(),
                "doctor".to_string()
            ]),
            Some("doctor")
        );
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "think".to_string(),
            "swe-agent".to_string(),
            "observe".to_string(),
            "README.md".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "traces".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "need".to_string(),
            "observe".to_string(),
            ".".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--json".to_string(),
            "check".to_string(),
            "computer-use-agent".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--json".to_string(),
            "check".to_string(),
            "computer-use-agent".to_string(),
            "1".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "check".to_string(),
            "computer-use-agent".to_string(),
            "1".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--json".to_string(),
            "check".to_string(),
            "custom-feed".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--json".to_string(),
            "check".to_string(),
            "computer-use-agent".to_string(),
            "0".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--json".to_string(),
            "check".to_string(),
            "computer-use-agent".to_string(),
            "one".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "oauth".to_string(),
            "octopus".to_string(),
            "tool:computer-use-agent".to_string(),
            "tool:observe".to_string(),
            "tool:ui".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "sh".to_string(),
            "-c".to_string(),
            "echo unsafe".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "oauth".to_string(),
            "github".to_string(),
            "dangoZhang/Octopus".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "oauth".to_string(),
            "octopus".to_string(),
            "tool:computer-use-agent".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "oauth".to_string(),
            "octopus".to_string(),
            "tool:computer-use-agent".to_string(),
            "harness:write".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "oauth".to_string(),
            "revoke".to_string(),
            "octopus:tool:computer-use-agent".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "self-iterate".to_string(),
            "pr".to_string(),
            "dangoZhang/Octopus".to_string()
        ]));
        assert_eq!(
            http_content_length("POST /api/run HTTP/1.1\r\nContent-Length: 42\r\n").unwrap(),
            42
        );
    }

    #[test]
    fn check_report_runs_manifest_checks() {
        let report = check_report("json-feed", None).unwrap();

        assert_eq!(report.id, "json-feed");
        assert_eq!(report.source_kind, "manifest");
        assert!(report.passed);
        assert!(report
            .results
            .iter()
            .any(|result| result.command.contains("py_compile")));

        let selected = check_report("computer-use-agent", Some(1)).unwrap();
        assert_eq!(selected.results.len(), 1);
        assert!(selected.results[0].command.contains("window_status"));
        assert!(check_report("json-feed", Some(99))
            .unwrap_err()
            .contains("out of range"));
    }

    #[test]
    fn cli_check_persists_history_in_state() {
        let _env = env_guard();
        let path =
            std::env::temp_dir().join(format!("octopus-check-state-{}.json", std::process::id()));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "check".to_string(),
            "json-feed".to_string(),
            "1".to_string(),
        ])
        .unwrap();

        let restored = HarnessState::load(&path).unwrap();
        assert_eq!(restored.check_history.len(), 1);
        assert_eq!(restored.check_history[0].tentacle_id, "json-feed");
        assert_eq!(restored.check_history[0].command_index, Some(1));
        assert_eq!(restored.last_pet_event.as_ref().unwrap().state, "success");
        let _ = fs::remove_file(path);
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
    fn pet_auto_state_prefers_recent_action_event() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let state_path = Path::new("state.json");
        let mut state = HarnessState::default();
        state.install_manifest(root, "swe-agent").unwrap();
        state.record_pet_event(
            "success",
            "swe-agent",
            "observed repo from feed",
            Status::Satisfied,
        );

        let report = pet_report_for_state(&state, state_path, "auto").unwrap();

        assert_eq!(report.state, "success");
        assert_eq!(report.fallback, "🟩");
        assert_eq!(report.event_source.as_deref(), Some("swe-agent"));
        assert_eq!(
            report.event_summary.as_deref(),
            Some("observed repo from feed")
        );
        assert!(report.target.contains("docs/pet.html?state=success"));

        state.record_pet_event("not-a-pet-state", "manual", "bad state", Status::Failed);
        let report = pet_report_for_state(&state, state_path, "auto").unwrap();

        assert_eq!(report.state, "heartbeat");
        assert!(report.event_source.is_none());
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
        run(vec![
            "provider".to_string(),
            "check".to_string(),
            "OCTOPUS_LLM".to_string(),
            "hello".to_string(),
        ])
        .unwrap();
        run(vec![
            "--json".to_string(),
            "provider".to_string(),
            "check".to_string(),
            "OCTOPUS_LLM".to_string(),
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
{"choices":[{"message":{"content":"{\"summary\":\"llm evolve selected runtime\",\"candidates\":[{\"surface_id\":\"runtime_code\",\"title\":\"LLM runtime patch\",\"target\":\"tools/read.sh\",\"rationale\":\"tool-side action feed\",\"change_plan\":[\"preserve Goal Mem Need Feed boundary\",\"edit runtime adapter\"],\"checks\":[\"tentacles/swe-agent/tools/read.sh README.md 1 2\"],\"suggested_patch\":\"diff --git a/tentacles/swe-agent/tools/read.sh b/tentacles/swe-agent/tools/read.sh\\n--- a/tentacles/swe-agent/tools/read.sh\\n+++ b/tentacles/swe-agent/tools/read.sh\\n@@ -1,2 +1,3 @@\\n #!/bin/sh\\n+# provider-assisted draft\\n set -eu\\n\"}]}"}}]}
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
        let draft = fs::read_to_string(
            dir.join(".octopus")
                .join("evolution")
                .join("swe-agent")
                .join("patches")
                .join("03-runtime-code.patch.md"),
        )
        .unwrap();
        assert!(proposal.contains("generator: `llm`"));
        assert!(candidates.contains("LLM runtime patch"));
        assert!(candidates.contains("provider patch: attached"));
        assert!(draft.contains("provider-assisted draft"));
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
            state.clone(),
            "evolve".to_string(),
            "recommend".to_string(),
            "swe-agent".to_string(),
            "next".to_string(),
        ])
        .unwrap();
        let recommended_plan = fs::read_to_string(
            dir.join(".octopus")
                .join("evolution")
                .join("swe-agent")
                .join("apply")
                .join("03-runtime-code.md"),
        )
        .unwrap();
        assert!(recommended_plan.contains("authorized: true"));
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
    fn install_report_exposes_manifest_grants_checks_and_next_steps() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let state_path = Path::new(".octopus/state with space.json");
        let mut state = HarnessState::default();
        state
            .install_profile("computer-use-agent")
            .expect("profile should install");
        let installed = state
            .install_manifest(&root, "computer-use-agent")
            .expect("manifest should install");
        let profile = default_tentacle_profiles()
            .into_iter()
            .find(|profile| profile.id == "computer-use-agent");
        let manifest = load_tentacle_manifests(&root)
            .unwrap()
            .into_iter()
            .find(|loaded| loaded.manifest.id == "computer-use-agent");

        let report = install_report(
            "computer-use-agent",
            state_path,
            true,
            Some(&installed),
            profile.as_ref(),
            manifest.as_ref(),
            &state,
        );

        assert_eq!(report.source_kind, "profile+manifest");
        assert!(report
            .tools
            .iter()
            .any(|tool| tool == "clipboard_read:shell"));
        assert!(report.runtimes.iter().any(|runtime| runtime == "shell"));
        assert!(report
            .checks
            .iter()
            .any(|check| check.contains("OCTOPUS_CLIPBOARD_DRY_RUN=1")));
        assert!(report.grants.iter().any(|grant| {
            !grant.active
                && grant.scope == "tool:computer-use-agent"
                && grant
                    .permissions
                    .iter()
                    .any(|permission| permission == "tool:observe")
                && grant
                    .permissions
                    .iter()
                    .any(|permission| permission == "tool:ui")
                && grant.command.contains("'.octopus/state with space.json'")
                && grant.command.contains("'tool:observe'")
                && grant.command.contains("'tool:ui'")
        }));
        assert!(report
            .next
            .iter()
            .any(|command| { command.contains("think computer-use-agent observe .") }));
        assert!(report
            .next
            .iter()
            .any(|command| { command.contains("need observe .") }));
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
        run(vec![
            "--state".to_string(),
            state.clone(),
            "traces".to_string(),
        ])
        .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("observe:swe-agent"));
        assert!(content.contains("feed_traces"));
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
        let error = run(vec![
            "--state".to_string(),
            state.clone(),
            "self-iterate".to_string(),
            "pr".to_string(),
            "dangoZhang/Octopus".to_string(),
        ])
        .unwrap_err();
        assert!(error.contains("requires github grant"));
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
