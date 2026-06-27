use octopus_core::{
    default_permissions, default_tentacle_profiles, feed_tentacle, inspect_tentacle_manifests,
    load_tentacle_manifests, plan_tentacle_evolution_apply, propose_tentacle_evolution_with_client,
    propose_tentacle_evolution_with_state, recommend_tentacle_evolution_apply, scaffold_tentacle,
    think_tentacle, write_harness_beat_evolution_artifacts, write_tentacle_apply_artifacts,
    write_tentacle_evolution_artifacts, AdaptReport, BrainExploreDraft, BrainExploreReport,
    BrainGoalReport, BrainGoalSaveReport, BrainPromptReport, CapabilityGrant, ChatClient,
    ChatMessage, ChatRole, CheckHistoryInput, CheckHistoryRecord, ContextReport, EnvironmentReport,
    EvolutionApplyArtifact, EvolutionApplyPlan, EvolutionArtifact, EvolutionOutcome,
    EvolutionRecommendation, Feed, FeedFeedbackOutcome, FeedTraceRecord, Goal, GoalChat,
    GoalNeedSuggestion, GoalRefinement, Harness, HarnessBeatEvolution, HarnessState, HeartBeat,
    HeartbeatReport, InstalledTentacle, LoadedTentacleManifest, Need, NeedKind, NeedQueueItem,
    NeedQueueReport, NeedQueueSaveReport, NeedQueueStatus, NeedQueueTakeReport,
    OpenAiCompatibleChatClient, OpenAiCompatibleConfig, RepairOutcome, RouteReport,
    SelfIterationPlan, Status, StatusReport, TentacleEvolutionProposal, TentacleManifestReport,
    TentacleProfile, TentacleScaffold, TentacleThinkingPlan, TentacleToolAction,
    TentacleToolCandidate, ToolPermission, CLEAN_BRAIN_CONTEXT_POLICY, TENTACLE_CONTEXT_POLICY,
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
struct BootstrapReport {
    state_path: String,
    state_existed: bool,
    files: Vec<InitFileReport>,
    adapt: AdaptReport,
    seed_tentacles: Vec<String>,
    installed_tentacles: Vec<String>,
    skipped_tentacles: Vec<String>,
    report: ProductReport,
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

#[derive(Debug, serde::Serialize)]
struct StarterReport {
    objective: String,
    policy: String,
    state_path: String,
    installed: Vec<String>,
    groups: Vec<StarterGroup>,
    recommendations: Vec<StarterRecommendation>,
    next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct StarterGroup {
    id: String,
    label: String,
    description: String,
    count: usize,
}

#[derive(Debug, serde::Serialize)]
struct StarterRecommendation {
    id: String,
    name: String,
    source_kind: String,
    group: String,
    group_label: String,
    group_reason: String,
    reason: String,
    signals: Vec<String>,
    installed: bool,
    llm_ready: bool,
    needs: Vec<String>,
    tools: Vec<String>,
    runtimes: Vec<String>,
    evolution_surfaces: Vec<String>,
    first_need_kind: String,
    first_need_query: String,
    install_command: String,
    first_need_command: String,
    check_command: String,
}

#[derive(Debug)]
struct StarterCandidate {
    id: String,
    name: String,
    source_kind: String,
    descriptions: BTreeSet<String>,
    installed: bool,
    llm_ready: bool,
    needs: BTreeSet<String>,
    tools: BTreeSet<String>,
    runtimes: BTreeSet<String>,
    evolution_surfaces: BTreeSet<String>,
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
struct ProviderSavedEnvReport {
    path: String,
    env: ProviderEnvReport,
    next: Vec<String>,
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

#[derive(Debug, serde::Serialize)]
struct ProductReport {
    version: String,
    state_path: String,
    state_exists: bool,
    context: ProductContextPolicy,
    capabilities: Vec<ProductCapability>,
    gaps: Vec<ProductGap>,
    next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct RepairReport {
    state_path: String,
    tentacle: String,
    feed: Feed,
    repair_plan: Option<RepairPlanReport>,
    queued: Vec<NeedQueueItem>,
    queue: NeedQueueReport,
    next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct RepairPlanReport {
    path: String,
    review: String,
    status: String,
    schema: String,
    session: String,
    target_tentacle: String,
    target_tool: String,
    candidate: String,
    review_boundary: String,
    check_command: String,
    grant_command: String,
    apply_command: String,
    score_command: String,
    suggested_commands: String,
    next_need_kind: String,
    next_need_query: String,
}

#[derive(Debug, serde::Serialize)]
struct RepairScoreReport {
    outcome: RepairOutcome,
    journal: Option<RepairOutcomeJournalReport>,
    recent: Vec<RepairOutcome>,
    next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct RepairOutcomeJournalReport {
    session_path: String,
    outcome_path: String,
    outcomes_file: String,
}

#[derive(Debug, serde::Serialize)]
struct NeedQueueScriptReport {
    policy: String,
    script_path: String,
    command_count: usize,
    commands: Vec<String>,
    next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct NeedQueueReviewSessionReport {
    policy: String,
    prompt: String,
    pending_count: usize,
    session_dir: String,
    prompt_path: String,
    messages_path: String,
    review_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    draft_path: Option<String>,
    command_path: String,
    next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct PreflightReport {
    version: String,
    target: String,
    state_path: String,
    live: bool,
    release_ready: bool,
    current_head: Option<String>,
    checks: Vec<PreflightCheck>,
    live_provider: Option<ProviderCheckReport>,
    next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct PreflightCheck {
    id: String,
    status: String,
    required: bool,
    evidence: String,
    next: String,
}

#[derive(Debug)]
struct RealMachineRecordStatus {
    passed: bool,
    evidence: String,
}

#[derive(Debug, serde::Serialize)]
struct PreflightScriptReport {
    path: String,
    state_path: String,
    commands: Vec<String>,
    next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct PreflightRecordReport {
    path: String,
    state_path: String,
    version: String,
    current_head: Option<String>,
    commands: Vec<String>,
    next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct ProductContextPolicy {
    brain: String,
    tentacle: String,
    feedback_loop: String,
}

#[derive(Debug, serde::Serialize)]
struct ProductCapability {
    id: String,
    status: String,
    evidence: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct ProductGap {
    id: String,
    impact: String,
    next: String,
}

#[derive(Debug, serde::Serialize)]
struct BrainSessionReport {
    policy: String,
    mode: String,
    prompt: String,
    session_dir: String,
    prompt_path: String,
    messages_path: String,
    reply_path: String,
    draft_path: Option<String>,
    command_path: String,
    apply_command: String,
    next: Vec<String>,
}

const DEFAULT_PROVIDER_ENV_PATH: &str = ".octopus/llm.env";

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
    env::set_var("OCTOPUS_STATE_PATH", &state);

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
        Some("repair") => {
            if rest.get(1).map(String::as_str) == Some("score") {
                let trace_index = rest
                    .get(2)
                    .ok_or_else(|| "repair score requires a feed trace index".to_string())
                    .and_then(|value| parse_trace_index(value))?;
                let status = rest
                    .get(3)
                    .ok_or_else(|| "repair score requires a status".to_string())
                    .and_then(|value| parse_status(value))?;
                let summary = rest
                    .get(4..)
                    .filter(|values| !values.is_empty())
                    .map(|values| values.join(" "))
                    .unwrap_or_else(|| "recorded repair outcome".to_string());
                let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
                let trace = loaded
                    .feed_traces
                    .iter()
                    .find(|trace| trace.index == trace_index)
                    .cloned();
                let outcome = loaded.record_repair_outcome(Some(trace_index), status, summary)?;
                let cwd = env::current_dir().map_err(|error| error.to_string())?;
                let journal = write_repair_score_journal(&cwd, trace.as_ref(), &outcome)?;
                let report = RepairScoreReport {
                    outcome,
                    journal,
                    recent: loaded.recent_repair_outcomes(5),
                    next: vec![
                        "octopus repair .".to_string(),
                        "octopus report".to_string(),
                        "octopus beat 200".to_string(),
                    ],
                };
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                    );
                } else {
                    print_repair_score_report(&report, language);
                }
                return Ok(());
            }
            let query = rest
                .get(1..)
                .filter(|values| !values.is_empty())
                .map(|values| values.join(" "))
                .unwrap_or_else(|| ".".to_string());
            let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let report = repair_report(&state, &mut loaded, query)?;
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_repair_report(&report, language);
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
            if let Some(kind_value) = rest.get(1) {
                let kind = parse_kind(kind_value)?;
                let query = if rest.len() > 2 {
                    rest[2..].join(" ")
                } else {
                    ".".to_string()
                };
                let harness = harness_for_need(loaded, &kind)?;
                let report = harness.route_report(&Need::new(kind, query));
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                    );
                } else {
                    print_route_report(&report, language);
                }
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&loaded.routes.scores)
                        .map_err(|error| error.to_string())?
                );
            }
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
        Some("feedback") => {
            let trace_index = rest
                .get(1)
                .ok_or_else(|| "feedback requires a feed trace index".to_string())
                .and_then(|value| parse_trace_index(value))?;
            let status = rest
                .get(2)
                .ok_or_else(|| "feedback requires a status".to_string())
                .and_then(|value| parse_status(value))?;
            let summary = rest
                .get(3..)
                .filter(|values| !values.is_empty())
                .map(|values| values.join(" "))
                .unwrap_or_else(|| "recorded feed feedback".to_string());
            let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let outcome = loaded.record_feed_feedback(trace_index, status, summary)?;
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&outcome).map_err(|error| error.to_string())?
                );
            } else {
                print_feed_feedback_outcome(&outcome, language);
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
        Some("brain") => {
            let mut live = false;
            let mut save = false;
            let mut refine_goal = false;
            let mut session = false;
            let mut apply_path = None;
            let mut apply_json = None;
            let mut prompt = Vec::new();
            let mut brain_index = 1;
            while brain_index < rest.len() {
                match rest[brain_index].as_str() {
                    "--live" => live = true,
                    "--save" => save = true,
                    "--goal" => refine_goal = true,
                    "--session" => session = true,
                    "--apply" => {
                        brain_index += 1;
                        let Some(path) = rest.get(brain_index) else {
                            return Err("brain --apply requires a path or -".to_string());
                        };
                        apply_path = Some(path.clone());
                    }
                    "--apply-json" => {
                        brain_index += 1;
                        let Some(payload) = rest.get(brain_index) else {
                            return Err("brain --apply-json requires JSON".to_string());
                        };
                        apply_json = Some(payload.clone());
                    }
                    value => prompt.push(value.to_string()),
                }
                brain_index += 1;
            }
            let prompt = (!prompt.is_empty()).then(|| prompt.join(" "));
            let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let prompt = prompt
                .or_else(|| loaded.goal.as_ref().map(|goal| goal.objective.clone()))
                .unwrap_or_else(|| "what should the brain ask next?".to_string());
            if session {
                let report = write_brain_session(&loaded, &state, &prompt, refine_goal, live)?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                    );
                } else {
                    print_brain_session(&report, language);
                }
            } else if let Some(payload) = apply_json
                .map(Ok)
                .or_else(|| apply_path.map(|path| read_brain_apply_payload(&path)))
                .transpose()?
            {
                if refine_goal {
                    let refinement =
                        parse_brain_reply::<GoalRefinement>(&payload, "clean-brain goal")?;
                    let report =
                        loaded.clean_brain_goal_from_refinement(prompt.clone(), 6, refinement);
                    if save {
                        let saved = loaded.queue_goal_report(&report);
                        loaded.save(&state).map_err(|error| error.to_string())?;
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&saved)
                                    .map_err(|error| error.to_string())?
                            );
                        } else {
                            print_brain_goal_save(&saved, language);
                        }
                    } else {
                        loaded.save(&state).map_err(|error| error.to_string())?;
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&report)
                                    .map_err(|error| error.to_string())?
                            );
                        } else {
                            print_brain_goal(&report, language);
                        }
                    }
                } else {
                    let draft =
                        parse_brain_reply::<BrainExploreDraft>(&payload, "clean-brain explore")?;
                    let report = loaded.clean_brain_explore_from_draft(prompt.clone(), 6, draft);
                    if save {
                        let saved = loaded.queue_exploration_report(&report);
                        loaded.save(&state).map_err(|error| error.to_string())?;
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&saved)
                                    .map_err(|error| error.to_string())?
                            );
                        } else {
                            print_need_queue_save(&saved, language);
                        }
                    } else if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .map_err(|error| error.to_string())?
                        );
                    } else {
                        print_brain_explore(&report, language);
                    }
                }
            } else if refine_goal {
                let report = if live || clean_brain_llm_enabled() {
                    let mut client = clean_brain_llm_client()?;
                    loaded.clean_brain_goal_with_client(prompt.clone(), 6, &mut client)?
                } else {
                    loaded.clean_brain_goal(prompt.clone(), 6)
                };
                if save {
                    let saved = loaded.queue_goal_report(&report);
                    loaded.save(&state).map_err(|error| error.to_string())?;
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&saved)
                                .map_err(|error| error.to_string())?
                        );
                    } else {
                        print_brain_goal_save(&saved, language);
                    }
                } else {
                    loaded.save(&state).map_err(|error| error.to_string())?;
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .map_err(|error| error.to_string())?
                        );
                    } else {
                        print_brain_goal(&report, language);
                    }
                }
            } else if live || save {
                let mut client = clean_brain_llm_client()?;
                let report = loaded.clean_brain_explore_with_client(prompt, 6, &mut client)?;
                if save {
                    let saved = loaded.queue_exploration_report(&report);
                    loaded.save(&state).map_err(|error| error.to_string())?;
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&saved)
                                .map_err(|error| error.to_string())?
                        );
                    } else {
                        print_need_queue_save(&saved, language);
                    }
                } else if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                    );
                } else {
                    print_brain_explore(&report, language);
                }
            } else {
                let report = loaded.clean_brain_prompt(prompt, 6);
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                    );
                } else {
                    print_brain_prompt(&report, language);
                }
            }
            Ok(())
        }
        Some("explore") => {
            let save = rest.get(1).map(String::as_str) == Some("--save");
            let prompt_start = if save { 2 } else { 1 };
            let prompt = rest
                .get(prompt_start..)
                .filter(|values| !values.is_empty())
                .map(|values| values.join(" "));
            let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let prompt = prompt
                .or_else(|| loaded.goal.as_ref().map(|goal| goal.objective.clone()))
                .unwrap_or_else(|| "next useful Need".to_string());
            let report = if clean_brain_llm_enabled() {
                let mut client = clean_brain_llm_client()?;
                loaded.clean_brain_explore_with_client(prompt.clone(), 6, &mut client)?
            } else {
                loaded.clean_brain_explore(prompt.clone(), 6)
            };
            if save {
                let saved = loaded.queue_exploration_report(&report);
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                    );
                } else {
                    print_need_queue_save(&saved, language);
                }
                return Ok(());
            }
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_explore(&report, language);
            }
            Ok(())
        }
        Some("needs") => {
            let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            match rest.get(1).map(String::as_str) {
                Some("take") => {
                    let index = rest
                        .get(2)
                        .ok_or_else(|| "needs take requires a queue index".to_string())
                        .and_then(|value| parse_queue_index(value))?;
                    let taken = loaded.take_queued_need(index)?;
                    loaded.save(&state).map_err(|error| error.to_string())?;
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&taken)
                                .map_err(|error| error.to_string())?
                        );
                    } else {
                        print_need_queue_take(&taken, language);
                    }
                    Ok(())
                }
                Some("drop") => {
                    let index = rest
                        .get(2)
                        .ok_or_else(|| "needs drop requires a queue index".to_string())
                        .and_then(|value| parse_queue_index(value))?;
                    let item = loaded.drop_queued_need(index)?;
                    loaded.save(&state).map_err(|error| error.to_string())?;
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&item)
                                .map_err(|error| error.to_string())?
                        );
                    } else {
                        print_need_queue_item(&item, language);
                    }
                    Ok(())
                }
                Some("script") => {
                    let path = rest.get(2).map(String::as_str);
                    let report = write_need_queue_script(&loaded, &state, path)?;
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .map_err(|error| error.to_string())?
                        );
                    } else {
                        print_need_queue_script(&report, language);
                    }
                    Ok(())
                }
                Some("session") => {
                    let mut live = false;
                    let mut prompt = Vec::new();
                    for value in rest.iter().skip(2) {
                        if value == "--live" {
                            live = true;
                        } else {
                            prompt.push(value.clone());
                        }
                    }
                    let prompt = if prompt.is_empty() {
                        "review pending clean-brain Needs".to_string()
                    } else {
                        prompt.join(" ")
                    };
                    let report = write_need_queue_review_session(&loaded, &state, &prompt, live)?;
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .map_err(|error| error.to_string())?
                        );
                    } else {
                        print_need_queue_review_session(&report, language);
                    }
                    Ok(())
                }
                Some(other) => Err(format!("unknown needs command: {other}")),
                None => {
                    let report = loaded.need_queue_report(12);
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .map_err(|error| error.to_string())?
                        );
                    } else {
                        print_need_queue(&report, language);
                    }
                    Ok(())
                }
            }
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
            if rest.get(1).map(String::as_str) == Some("save") {
                let profile_id = rest
                    .get(2)
                    .ok_or_else(|| "provider save requires a profile id".to_string())?;
                let prefix = rest.get(3).map(String::as_str).unwrap_or("OCTOPUS_LLM");
                let path = rest
                    .get(4)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from(DEFAULT_PROVIDER_ENV_PATH));
                let report = save_provider_env_report(profile_id, prefix, &path)?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                    );
                } else {
                    print_provider_saved_env(&report, language);
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
        Some("bootstrap") => {
            let root = rest
                .get(1)
                .map(PathBuf::from)
                .unwrap_or_else(default_tentacles_root);
            let report = bootstrap_workspace(state.clone(), root)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_bootstrap_report(&report, language);
            }
            Ok(())
        }
        Some("goal") => {
            let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            if rest.get(1).map(String::as_str) == Some("set") {
                let objective = rest
                    .get(2..)
                    .filter(|values| !values.is_empty())
                    .map(|values| values.join(" "))
                    .ok_or_else(|| "goal set requires an objective".to_string())?;
                loaded.goal = Some(Goal::new(objective));
                loaded.save(&state).map_err(|error| error.to_string())?;
            }
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
        Some("context") => {
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let next_need = parse_context_need(&rest)?;
            let report = loaded.context_report(next_need, 6);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_context_report(&report, language);
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
        Some("report") => {
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let report = product_report(&loaded, &state)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_product_report(&report, language);
            }
            Ok(())
        }
        Some("preflight") => {
            if rest.get(1).map(String::as_str) == Some("script") {
                let path = rest
                    .get(2)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from(".octopus/preflight.sh"));
                let report = write_preflight_script(&state, &path)?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                    );
                } else {
                    print_preflight_script_report(&report, language);
                }
                return Ok(());
            }
            if rest.get(1).map(String::as_str) == Some("record") {
                let path = rest
                    .get(2)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from(".octopus/real-machine-record.md"));
                let report = write_preflight_record(&state, &path)?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                    );
                } else {
                    print_preflight_record_report(&report, language);
                }
                return Ok(());
            }
            let live = rest.iter().skip(1).any(|arg| arg == "--live");
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let report = preflight_report(&loaded, &state, live)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_preflight_report(&report, language);
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
        Some("starter") => {
            let objective = rest
                .get(1..)
                .filter(|values| !values.is_empty())
                .map(|values| values.join(" "));
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let report = starter_report(&loaded, &state, default_tentacles_root(), objective)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_starter_report(&report, language);
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

fn parse_context_need(rest: &[String]) -> Result<Option<Need>, String> {
    match rest.len() {
        1 => Ok(None),
        2 => Err("context needs both kind and query, or no arguments".to_string()),
        _ => {
            let kind = parse_kind(&rest[1])?;
            let query = rest[2..].join(" ");
            Ok(Some(Need::new(kind, query)))
        }
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
        if let Some(index) = feed.metadata.get("feed_trace_index") {
            match language {
                Language::En => println!("trace_index: {index}"),
                Language::Zh => println!("Feed轨迹编号: {index}"),
            }
        }
        if let Some(trace) = feed_trace(feed) {
            match language {
                Language::En => println!("feed_trace: {trace}"),
                Language::Zh => println!("Feed轨迹: {trace}"),
            }
        }
    }
}

fn print_repair_report(report: &RepairReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus repair");
            println!("tentacle: {}", report.tentacle);
            println!("state: {}", report.state_path);
            println!("feed: {:?} {}", report.feed.status, report.feed.summary);
            if let Some(index) = report.feed.metadata.get("feed_trace_index") {
                println!("trace_index: {index}");
            }
            if let Some(plan) = &report.repair_plan {
                println!("repair_plan: {}", plan.path);
                print_optional_line("review", &plan.review);
                println!("plan_status: {}", plan.status);
                println!("target: {}/{}", plan.target_tentacle, plan.target_tool);
                print_optional_line("check", &plan.check_command);
                print_optional_line("grant", &plan.grant_command);
                print_optional_line("apply", &plan.apply_command);
                print_optional_line("score", &plan.score_command);
            }
            println!("queued: {}", report.queued.len());
            for item in &report.queued {
                println!(
                    "- #{} {}:{}",
                    item.index,
                    need_label(&item.need.kind),
                    item.need.query
                );
            }
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼修复");
            println!("触手: {}", report.tentacle);
            println!("状态文件: {}", report.state_path);
            println!("Feed: {:?} {}", report.feed.status, report.feed.summary);
            if let Some(index) = report.feed.metadata.get("feed_trace_index") {
                println!("Feed轨迹编号: {index}");
            }
            if let Some(plan) = &report.repair_plan {
                println!("修复计划: {}", plan.path);
                print_optional_line("审阅", &plan.review);
                println!("计划状态: {}", plan.status);
                println!("目标: {}/{}", plan.target_tentacle, plan.target_tool);
                print_optional_line("检查", &plan.check_command);
                print_optional_line("授权", &plan.grant_command);
                print_optional_line("应用", &plan.apply_command);
                print_optional_line("评分", &plan.score_command);
            }
            println!("排队Need: {}", report.queued.len());
            for item in &report.queued {
                println!(
                    "- #{} {}:{}",
                    item.index,
                    need_label(&item.need.kind),
                    item.need.query
                );
            }
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

fn print_optional_line(label: &str, value: &str) {
    if !value.trim().is_empty() {
        println!("{label}: {value}");
    }
}

fn print_repair_score_report(report: &RepairScoreReport, language: Language) {
    match language {
        Language::En => {
            println!("recorded repair outcome #{}", report.outcome.index);
            if let Some(trace_index) = report.outcome.trace_index {
                println!("feed_trace: {trace_index}");
            }
            println!("status: {:?}", report.outcome.status);
            println!("score: {:.2}", report.outcome.score);
            println!("summary: {}", report.outcome.summary);
            if let Some(journal) = &report.journal {
                println!("outcome: {}", journal.outcome_path);
                println!("journal: {}", journal.outcomes_file);
            }
            println!("recent_repair_outcomes: {}", report.recent.len());
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("已记录 repair 结果 #{}", report.outcome.index);
            if let Some(trace_index) = report.outcome.trace_index {
                println!("Feed trace: {trace_index}");
            }
            println!("状态: {:?}", report.outcome.status);
            println!("分数: {:.2}", report.outcome.score);
            println!("摘要: {}", report.outcome.summary);
            if let Some(journal) = &report.journal {
                println!("结果文件: {}", journal.outcome_path);
                println!("结果日志: {}", journal.outcomes_file);
            }
            println!("近期 repair 结果: {}", report.recent.len());
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

fn write_repair_score_journal(
    cwd: &Path,
    trace: Option<&FeedTraceRecord>,
    outcome: &RepairOutcome,
) -> Result<Option<RepairOutcomeJournalReport>, String> {
    let Some(trace) = trace else {
        return Ok(None);
    };
    let Some(session) = trace.metadata.get("session") else {
        return Ok(None);
    };
    let workspace = trace
        .metadata
        .get("workspace")
        .map(PathBuf::from)
        .unwrap_or_else(|| cwd.to_path_buf());
    let session_path = metadata_path(&workspace, session);
    let Some(session_dir) = session_path.parent() else {
        return Ok(None);
    };
    if !session_path.exists() {
        return Ok(None);
    }
    let repair_root = workspace.join(".octopus").join("harness-repair");
    fs::create_dir_all(&repair_root).map_err(|error| error.to_string())?;
    let outcomes_file = repair_root.join("outcomes.jsonl");
    let outcome_path = session_dir.join("OUTCOME.md");
    let status = cli_status_key(&outcome.status);
    let target = trace
        .metadata
        .get("target_tentacle")
        .cloned()
        .unwrap_or_else(|| outcome.tentacle_id.clone());
    let candidate = trace
        .metadata
        .get("candidate")
        .cloned()
        .unwrap_or_else(|| "none".to_string());
    let draft_status = trace
        .metadata
        .get("llm_draft_status")
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());
    let timestamp_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_secs();
    let record = serde_json::json!({
        "schema_version": "octopus-harness-repair-outcome-v1",
        "timestamp_secs": timestamp_secs,
        "workspace": workspace.to_string_lossy(),
        "session": display_path(&workspace, &session_path),
        "trace_index": outcome.trace_index,
        "target_tentacle": target,
        "candidate": candidate,
        "source": trace.metadata.get("source").cloned().unwrap_or_else(|| "repair_score".to_string()),
        "next_need": trace.metadata.get("next_need").cloned().unwrap_or_default(),
        "draft_status": draft_status,
        "outcome_status": status,
        "summary": truncate_chars(&outcome.summary, 500),
    });
    let mut handle = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&outcomes_file)
        .map_err(|error| error.to_string())?;
    writeln!(handle, "{record}").map_err(|error| error.to_string())?;
    fs::write(
        &outcome_path,
        format!(
            "# Harness Repair Outcome\n\nstatus: `{}`\nsession: `{}`\ntarget: `{}`\ncandidate: `{}`\ndraft_status: `{}`\n\n{}\n\njournal: `{}`\n",
            status,
            display_path(&workspace, &session_path),
            record["target_tentacle"].as_str().unwrap_or("unknown"),
            record["candidate"].as_str().unwrap_or("none"),
            record["draft_status"].as_str().unwrap_or("unknown"),
            outcome.summary,
            display_path(&workspace, &outcomes_file)
        ),
    )
    .map_err(|error| error.to_string())?;
    Ok(Some(RepairOutcomeJournalReport {
        session_path: display_path(&workspace, &session_path),
        outcome_path: display_path(&workspace, &outcome_path),
        outcomes_file: display_path(&workspace, &outcomes_file),
    }))
}

fn metadata_path(workspace: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        workspace.join(path)
    }
}

fn display_path(workspace: &Path, path: &Path) -> String {
    path.strip_prefix(workspace)
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

fn cli_status_key(status: &Status) -> &'static str {
    match status {
        Status::Satisfied => "satisfied",
        Status::Partial => "partial",
        Status::Failed => "failed",
        Status::Unsupported => "unsupported",
    }
}

fn truncate_chars(value: &str, max: usize) -> String {
    let mut output = value.chars().take(max).collect::<String>();
    if value.chars().count() > max {
        output.push_str("...");
    }
    output
}

fn print_feed_feedback_outcome(outcome: &FeedFeedbackOutcome, language: Language) {
    match language {
        Language::En => {
            println!("recorded feed feedback #{}", outcome.trace.index);
            println!("status: {:?}", outcome.status);
            if let Some(score) = outcome.route_score {
                println!("route_score: {score:.2}");
            }
            println!("pet: {}", outcome.pet_event.state);
        }
        Language::Zh => {
            println!("已记录 Feed 反馈 #{}", outcome.trace.index);
            println!("状态: {:?}", outcome.status);
            if let Some(score) = outcome.route_score {
                println!("路由分数: {score:.2}");
            }
            println!("章鱼状态: {}", outcome.pet_event.state);
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

fn print_starter_report(report: &StarterReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus starter");
            println!("objective: {}", report.objective);
            println!("policy: {}", report.policy);
            println!("state: {}", report.state_path);
            println!("installed: {}", join_or_none(&report.installed));
            println!("groups:");
            for group in &report.groups {
                println!("- {} ({}) {}", group.label, group.count, group.description);
            }
            println!("recommendations:");
            for item in &report.recommendations {
                let status = if item.installed {
                    "installed"
                } else {
                    "available"
                };
                println!(
                    "- {} [{}; {}; group={}; llm={}] {}",
                    item.id,
                    item.source_kind,
                    status,
                    item.group_label,
                    item.llm_ready,
                    item.reason
                );
                println!("  group_reason: {}", item.group_reason);
                println!("  signals: {}", join_or_none(&item.signals));
                println!("  install: {}", item.install_command);
                println!("  first_need: {}", item.first_need_command);
                println!("  check: {}", item.check_command);
            }
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼启动推荐");
            println!("目标: {}", report.objective);
            println!("策略: {}", report.policy);
            println!("状态文件: {}", report.state_path);
            println!("已安装: {}", join_or_none(&report.installed));
            println!("分组:");
            for group in &report.groups {
                println!("- {} ({}) {}", group.label, group.count, group.description);
            }
            println!("推荐:");
            for item in &report.recommendations {
                let status = if item.installed {
                    "已安装"
                } else {
                    "可安装"
                };
                println!(
                    "- {} [{}; {}; 分组={}; LLM={}] {}",
                    item.id,
                    item.source_kind,
                    status,
                    item.group_label,
                    item.llm_ready,
                    item.reason
                );
                println!("  分组原因: {}", item.group_reason);
                println!("  信号: {}", join_or_none(&item.signals));
                println!("  安装: {}", item.install_command);
                println!("  第一条Need: {}", item.first_need_command);
                println!("  检查: {}", item.check_command);
            }
            println!("下一步: {}", join_or_none(&report.next));
        }
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
        "evolution_source_kind".to_string(),
        evolution.source_kind.clone(),
    );
    beat.data.insert(
        "evolution_source_index".to_string(),
        evolution.source_index.to_string(),
    );
    beat.data.insert(
        "evolution_source_summary".to_string(),
        evolution.source_summary.clone(),
    );
    beat.data.insert(
        "evolution_tentacle".to_string(),
        evolution.tentacle_id.clone(),
    );
    beat.data.insert(
        "evolution_candidate".to_string(),
        evolution.candidate_id.clone(),
    );
    beat.data.insert(
        "evolution_score_target".to_string(),
        format!("{} {}", evolution.tentacle_id, evolution.candidate_id),
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
    let source_kind = beat
        .data
        .get("evolution_source_kind")
        .map(String::as_str)
        .unwrap_or("check_history");
    let source_index = beat
        .data
        .get("evolution_source_index")
        .map(String::as_str)
        .unwrap_or("unknown");
    match language {
        Language::En => {
            println!("  recommendation: {tentacle} {candidate} ({status})");
            println!("  source: {source_kind} #{source_index}");
            println!("  plan: {plan}");
            println!("  next: {next}");
        }
        Language::Zh => {
            println!("  推荐: {tentacle} {candidate} ({status})");
            println!("  来源: {source_kind} #{source_index}");
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
            id: "deepseek".to_string(),
            name: "DeepSeek OpenAI-compatible cloud".to_string(),
            description: "DeepSeek chat models through their OpenAI-compatible endpoint."
                .to_string(),
            base_url: "https://api.deepseek.com".to_string(),
            model: "deepseek-v4-flash".to_string(),
            key_env: "DEEPSEEK_API_KEY".to_string(),
            notes: vec![
                "Set DEEPSEEK_API_KEY before sourcing the generated env.".to_string(),
                "Good fit for clean-brain exploration and economical tool-side planning."
                    .to_string(),
            ],
        },
        ProviderProfile {
            id: "groq".to_string(),
            name: "Groq OpenAI-compatible cloud".to_string(),
            description: "Fast hosted inference through Groq's OpenAI-compatible endpoint."
                .to_string(),
            base_url: "https://api.groq.com/openai/v1".to_string(),
            model: "openai/gpt-oss-20b".to_string(),
            key_env: "GROQ_API_KEY".to_string(),
            notes: vec![
                "Set GROQ_API_KEY before sourcing the generated env.".to_string(),
                "Useful for low-latency Need exploration and provider checks.".to_string(),
            ],
        },
        ProviderProfile {
            id: "gemini".to_string(),
            name: "Gemini OpenAI-compatible cloud".to_string(),
            description: "Gemini models through Google's OpenAI-compatible endpoint.".to_string(),
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
            model: "gemini-3.5-flash".to_string(),
            key_env: "GEMINI_API_KEY".to_string(),
            notes: vec![
                "Set GEMINI_API_KEY before sourcing the generated env.".to_string(),
                "Use another Gemini model id if your account exposes it.".to_string(),
            ],
        },
        ProviderProfile {
            id: "dashscope".to_string(),
            name: "DashScope OpenAI-compatible cloud".to_string(),
            description: "Qwen models through Alibaba Cloud DashScope compatibility mode."
                .to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            model: "qwen-plus".to_string(),
            key_env: "DASHSCOPE_API_KEY".to_string(),
            notes: vec![
                "Set DASHSCOPE_API_KEY before sourcing the generated env.".to_string(),
                "Change the model to a Qwen id enabled for your account.".to_string(),
                "Change the base URL if your region or workspace requires a regional endpoint."
                    .to_string(),
            ],
        },
        ProviderProfile {
            id: "moonshot".to_string(),
            name: "Moonshot OpenAI-compatible cloud".to_string(),
            description: "Kimi/Moonshot models through an OpenAI-compatible endpoint.".to_string(),
            base_url: "https://api.moonshot.ai/v1".to_string(),
            model: "kimi-k2.6".to_string(),
            key_env: "MOONSHOT_API_KEY".to_string(),
            notes: vec![
                "Set MOONSHOT_API_KEY before sourcing the generated env.".to_string(),
                "Use a larger context model id when clean-brain memory grows.".to_string(),
            ],
        },
        ProviderProfile {
            id: "lmstudio".to_string(),
            name: "LM Studio local server".to_string(),
            description: "LM Studio's local OpenAI-compatible server.".to_string(),
            base_url: "http://localhost:1234/v1".to_string(),
            model: "local-model".to_string(),
            key_env: "".to_string(),
            notes: vec![
                "Start the LM Studio local server before provider check.".to_string(),
                "Replace local-model with the served model id.".to_string(),
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
        "export OCTOPUS_BRAIN_LLM=1".to_string(),
        "export OCTOPUS_LLM_MANIFEST=1".to_string(),
        "export OCTOPUS_LLM_EVOLVE=1".to_string(),
        format!("export OCTOPUS_CHAT_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_BRAIN_LLM_PREFIX={prefix}"),
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

fn save_provider_env_report(
    profile_id: &str,
    prefix: &str,
    path: &Path,
) -> Result<ProviderSavedEnvReport, String> {
    let env = provider_env_report(profile_id, prefix)?;
    if let Some(parent) = path.parent().filter(|value| !value.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, format!("{}\n", env.shell)).map_err(|error| error.to_string())?;
    let path = path.to_string_lossy().to_string();
    let prefix = env.prefix.clone();
    Ok(ProviderSavedEnvReport {
        path: path.clone(),
        env,
        next: vec![
            format!("source {path}"),
            "octopus provider status".to_string(),
            format!("octopus provider check {prefix}"),
        ],
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
        "provider check failed for {prefix}: {error}\nnext: octopus providers; octopus provider save openai; source .octopus/llm.env; octopus provider check {prefix}"
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
            "clean_brain",
            "clean brain explores Goal/Mem/Need/Feed into new Needs",
            brain_llm_enabled(),
            brain_llm_prefix(),
            "export OCTOPUS_BRAIN_LLM=1",
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
        next.push("octopus provider save openai".to_string());
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

fn print_provider_saved_env(report: &ProviderSavedEnvReport, language: Language) {
    match language {
        Language::En => println!("Saved provider env: {}", report.path),
        Language::Zh => println!("已保存 provider env: {}", report.path),
    }
    for item in &report.next {
        println!("next: {item}");
    }
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
    let mut command = Command::new(env::current_exe().map_err(|error| error.to_string())?);
    command.args(args);
    apply_bridge_env_overlay(&mut command);
    let output = command
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
    let mut command = Command::new(env::current_exe().map_err(|error| error.to_string())?);
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    apply_bridge_env_overlay(&mut command);
    let mut child = command
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

fn apply_bridge_env_overlay(command: &mut Command) {
    for (key, value) in bridge_env_overlay() {
        command.env(key, value);
    }
}

fn bridge_env_overlay() -> Vec<(String, String)> {
    fs::read_to_string(DEFAULT_PROVIDER_ENV_PATH)
        .map(|content| parse_bridge_env_overlay(&content))
        .unwrap_or_default()
}

fn parse_bridge_env_overlay(content: &str) -> Vec<(String, String)> {
    content.lines().filter_map(parse_bridge_env_line).collect()
}

fn parse_bridge_env_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let assignment = line.strip_prefix("export ").unwrap_or(line);
    let (key, value) = assignment.split_once('=')?;
    let key = key.trim();
    if !key.starts_with("OCTOPUS_") || !valid_env_prefix(key) {
        return None;
    }
    Some((key.to_string(), bridge_env_value(value.trim())))
}

fn bridge_env_value(value: &str) -> String {
    if let Some(inner) = value
        .strip_prefix('"')
        .and_then(|item| item.strip_suffix('"'))
    {
        if let Some(name) = inner
            .strip_prefix("${")
            .and_then(|item| item.strip_suffix(":-}"))
            .filter(|name| valid_env_prefix(name))
        {
            return env::var(name).unwrap_or_default();
        }
        return inner.replace("\\\"", "\"").replace("\\\\", "\\");
    }
    if let Some(inner) = value
        .strip_prefix('\'')
        .and_then(|item| item.strip_suffix('\''))
    {
        return inner.replace("'\\''", "'");
    }
    value.to_string()
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
    if command == "evolve" {
        return bridge_evolve_allowed(args);
    }
    if command == "feedback" {
        return bridge_feedback_allowed(args);
    }
    if command == "repair" {
        return bridge_repair_allowed(args);
    }
    if command == "provider" {
        return bridge_provider_allowed(args);
    }
    if command == "preflight" {
        return bridge_preflight_allowed(args);
    }
    if command == "self-iterate" && args.iter().any(|arg| arg == "pr") {
        return false;
    }
    matches!(
        command,
        "init"
            | "chat"
            | "brain"
            | "explore"
            | "needs"
            | "think"
            | "need"
            | "bootstrap"
            | "pet"
            | "doctor"
            | "report"
            | "beat"
            | "status"
            | "context"
            | "goal"
            | "installed"
            | "install"
            | "skills"
            | "catalog"
            | "starter"
            | "manifests"
            | "providers"
            | "routes"
            | "traces"
            | "feedback"
            | "env"
            | "adapt"
            | "self-iterate"
    )
}

fn bridge_repair_allowed(args: &[String]) -> bool {
    let Some(index) = bridge_command_index(args) else {
        return false;
    };
    let rest = &args[index + 1..];
    if rest.first().map(String::as_str) != Some("score") {
        return true;
    }
    let Some(trace_index) = rest.get(1) else {
        return false;
    };
    let Some(status) = rest.get(2) else {
        return false;
    };
    parse_trace_index(trace_index).is_ok()
        && parse_status(status).is_ok_and(|status| {
            matches!(status, Status::Satisfied | Status::Partial | Status::Failed)
        })
}

fn bridge_preflight_allowed(args: &[String]) -> bool {
    let Some(index) = bridge_command_index(args) else {
        return false;
    };
    let rest = &args[index + 1..];
    rest.is_empty()
        || (rest.len() == 1 && matches!(rest[0].as_str(), "--live" | "script" | "record"))
}

fn bridge_feedback_allowed(args: &[String]) -> bool {
    let Some(index) = bridge_command_index(args) else {
        return false;
    };
    let Some(trace_index) = args.get(index + 1) else {
        return false;
    };
    let Some(status) = args.get(index + 2) else {
        return false;
    };
    parse_trace_index(trace_index).is_ok()
        && parse_status(status).is_ok_and(|status| {
            matches!(status, Status::Satisfied | Status::Partial | Status::Failed)
        })
}

fn bridge_evolve_allowed(args: &[String]) -> bool {
    let Some(index) = bridge_command_index(args) else {
        return false;
    };
    match args.get(index + 1).map(String::as_str) {
        Some("score") => {
            let Some(tentacle) = args.get(index + 2) else {
                return false;
            };
            let Some(candidate) = args.get(index + 3) else {
                return false;
            };
            let Some(status) = args.get(index + 4) else {
                return false;
            };
            bridge_seed_tentacle(tentacle)
                && bridge_safe_candidate(candidate)
                && parse_status(status).is_ok_and(|status| {
                    matches!(status, Status::Satisfied | Status::Partial | Status::Failed)
                })
        }
        Some("apply") => {
            let Some(tentacle) = args.get(index + 2) else {
                return false;
            };
            let Some(candidate) = args.get(index + 3) else {
                return false;
            };
            bridge_seed_tentacle(tentacle) && bridge_safe_candidate(candidate)
        }
        _ => false,
    }
}

fn bridge_provider_allowed(args: &[String]) -> bool {
    let Some(index) = bridge_command_index(args) else {
        return false;
    };
    match args.get(index + 1).map(String::as_str) {
        Some("status") => args.len() == index + 2,
        Some("check") => args
            .get(index + 2)
            .is_none_or(|prefix| valid_env_prefix(prefix)),
        Some("save") => {
            let Some(profile) = args.get(index + 2) else {
                return false;
            };
            provider_profile(profile).is_ok()
                && args
                    .get(index + 3)
                    .is_none_or(|prefix| valid_env_prefix(prefix))
                && args.len() <= index + 4
        }
        Some(profile) => {
            provider_profile(profile).is_ok()
                && args
                    .get(index + 2)
                    .is_none_or(|prefix| valid_env_prefix(prefix))
                && args.len() <= index + 3
        }
        None => false,
    }
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
        && bridge_seed_tentacle(tentacle)
}

fn bridge_seed_tentacle(tentacle: &str) -> bool {
    matches!(
        tentacle,
        "swe-agent"
            | "computer-use-agent"
            | "bash-only"
            | "repo-maintainer"
            | "harness-repair-agent"
            | "json-feed"
            | "visual"
    )
}

fn bridge_safe_candidate(candidate: &str) -> bool {
    !candidate.is_empty()
        && candidate.len() <= 96
        && candidate
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':'))
}

fn bridge_oauth_allowed(args: &[String]) -> bool {
    let Some(index) = bridge_command_index(args) else {
        return false;
    };
    if args.get(index).is_none_or(|command| command != "oauth")
        || args
            .get(index + 1)
            .is_none_or(|provider| provider != "octopus")
    {
        return false;
    }
    let Some(scope) = args.get(index + 2) else {
        return false;
    };
    if scope.starts_with("tool:") {
        return args.get(index + 3).is_some()
            && args
                .iter()
                .skip(index + 3)
                .all(|permission| permission.starts_with("tool:"));
    }
    scope
        .strip_prefix("evolve:")
        .is_some_and(bridge_seed_tentacle)
        && args.len() == index + 4
        && args
            .get(index + 3)
            .is_some_and(|permission| permission == "harness:write")
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

fn bootstrap_workspace(
    state_path: PathBuf,
    tentacles_root: PathBuf,
) -> Result<BootstrapReport, String> {
    let state_existed = state_path.exists();
    let cwd = env::current_dir().map_err(|error| error.to_string())?;
    let mut state = HarnessState::load(&state_path).map_err(|error| error.to_string())?;
    let files = init_files(&state_path)?;
    let adapt = state.adapt_environment(&cwd, &tentacles_root);
    let seed_tentacles = bootstrap_seed_tentacles()
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut installed_tentacles = Vec::new();
    let mut skipped_tentacles = Vec::new();
    for tentacle_id in &seed_tentacles {
        let was_installed = state
            .installed_tentacles
            .iter()
            .any(|tentacle| tentacle.id == *tentacle_id);
        let _ = state.install_profile(tentacle_id);
        match state.install_manifest(&tentacles_root, tentacle_id) {
            Ok(tentacle) if !was_installed => installed_tentacles.push(tentacle.id),
            Ok(_) => {}
            Err(error) => skipped_tentacles.push(format!("{tentacle_id}: {error}")),
        }
    }
    state.beat(200);
    state.save(&state_path).map_err(|error| error.to_string())?;
    let report = product_report(&state, &state_path)?;
    let state_arg = shell_arg(&state_path.to_string_lossy());
    let mut next = vec![
        format!("octopus --state {state_arg} report"),
        format!("octopus --state {state_arg} context observe ."),
        format!("octopus --state {state_arg} think swe-agent observe README.md"),
        format!("octopus --state {state_arg} think harness-repair-agent execute ."),
        format!("octopus --state {state_arg} need execute \"repair harness session\""),
        format!("octopus --state {state_arg} need observe README.md"),
        format!("octopus --state {state_arg} check swe-agent"),
        format!("octopus --state {state_arg} check harness-repair-agent"),
        format!("octopus --state {state_arg} pet"),
    ];
    next.extend(report.next.iter().cloned());
    next.sort();
    next.dedup();
    Ok(BootstrapReport {
        state_path: state_path.to_string_lossy().to_string(),
        state_existed,
        files,
        adapt,
        seed_tentacles,
        installed_tentacles,
        skipped_tentacles,
        report,
        next,
    })
}

fn bootstrap_seed_tentacles() -> Vec<&'static str> {
    vec![
        "swe-agent",
        "json-feed",
        "computer-use-agent",
        "repo-maintainer",
        "harness-repair-agent",
        "bash-only",
        "visual",
    ]
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
# You can also generate this file with: octopus provider save openai OCTOPUS_LLM llm.env
export OCTOPUS_LLM_MODEL=gpt-4.1-mini
export OCTOPUS_LLM_BASE_URL=https://api.openai.com/v1
export OCTOPUS_LLM_API_KEY=
export OCTOPUS_CHAT_LLM=1
export OCTOPUS_BRAIN_LLM=1
export OCTOPUS_LLM_MANIFEST=1
export OCTOPUS_LLM_EVOLVE=1
# Optional: point chat, clean brain, tentacle planning, or evolution at another OpenAI-compatible env prefix.
# export OCTOPUS_CHAT_LLM_PREFIX=OCTOPUS_LLM
# export OCTOPUS_BRAIN_LLM_PREFIX=OCTOPUS_LLM
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

fn starter_report(
    state: &HarnessState,
    state_path: &Path,
    root: PathBuf,
    objective: Option<String>,
) -> Result<StarterReport, String> {
    let objective = objective
        .filter(|value| !value.trim().is_empty())
        .or_else(|| state.goal.as_ref().map(|goal| goal.objective.clone()))
        .unwrap_or_else(|| "start Octopus on this project".to_string());
    let keywords = starter_keywords(&objective);
    let mut candidates = BTreeMap::<String, StarterCandidate>::new();
    for skill in skill_reports(state, root)? {
        let entry = candidates
            .entry(skill.source.clone())
            .or_insert_with(|| StarterCandidate {
                id: skill.source.clone(),
                name: title_from_id(&skill.source),
                source_kind: skill.source_kind.clone(),
                descriptions: BTreeSet::new(),
                installed: skill.installed,
                llm_ready: skill.llm_ready,
                needs: BTreeSet::new(),
                tools: BTreeSet::new(),
                runtimes: BTreeSet::new(),
                evolution_surfaces: BTreeSet::new(),
            });
        entry.descriptions.insert(skill.description);
        entry.installed |= skill.installed;
        entry.llm_ready |= skill.llm_ready;
        entry.needs.extend(skill.needs);
        entry.tools.extend(skill.tools);
        entry.runtimes.extend(skill.runtimes);
        entry.evolution_surfaces.extend(skill.evolution_surfaces);
    }

    let mut scored = candidates
        .into_values()
        .map(|candidate| {
            let matched = starter_matches(&candidate, &keywords);
            let score = (matched.len() as i32 * 10) + starter_priority_score(&candidate.id);
            (score, matched, candidate)
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| starter_priority(&left.2.id).cmp(&starter_priority(&right.2.id)))
            .then_with(|| left.2.id.cmp(&right.2.id))
    });

    let state_path_text = state_path.to_string_lossy().to_string();
    let mut recommendations = Vec::new();
    for (_, matched, candidate) in scored.into_iter().take(6) {
        let needs = candidate.needs.iter().cloned().collect::<Vec<_>>();
        let tools = candidate.tools.iter().cloned().collect::<Vec<_>>();
        let runtimes = candidate.runtimes.iter().cloned().collect::<Vec<_>>();
        let evolution_surfaces = candidate
            .evolution_surfaces
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let first_need = starter_need_kind(&objective, &needs);
        let group = starter_group(&candidate.id, &needs, &tools, &evolution_surfaces);
        let reason = if matched.is_empty() {
            "starter coverage from manifest metadata".to_string()
        } else {
            format!("matches {}", matched.join(", "))
        };
        let group_reason = group.2.to_string();
        let signals = starter_signals(&candidate, &matched, group, &first_need);
        recommendations.push(StarterRecommendation {
            id: candidate.id.clone(),
            name: candidate.name,
            source_kind: candidate.source_kind,
            group: group.0.to_string(),
            group_label: group.1.to_string(),
            group_reason,
            reason,
            signals,
            installed: candidate.installed,
            llm_ready: candidate.llm_ready,
            needs,
            tools,
            runtimes,
            evolution_surfaces,
            first_need_kind: first_need.clone(),
            first_need_query: objective.clone(),
            install_command: octopus_state_command(
                state_path,
                &["install".to_string(), candidate.id.clone()],
            ),
            first_need_command: octopus_state_command(
                state_path,
                &["need".to_string(), first_need, shell_value(&objective)],
            ),
            check_command: octopus_state_command(
                state_path,
                &["check".to_string(), candidate.id.clone()],
            ),
        });
    }
    let groups = starter_groups(&recommendations);

    let mut installed = state.installed_profiles.clone();
    installed.extend(
        state
            .installed_tentacles
            .iter()
            .map(|tentacle| tentacle.id.clone()),
    );
    installed.sort();
    installed.dedup();

    let mut next = Vec::new();
    if let Some(first) = recommendations.first() {
        if !first.installed {
            next.push(first.install_command.clone());
        }
        next.push(first.first_need_command.clone());
        next.push(first.check_command.clone());
    }
    next.push("octopus bridge".to_string());
    next.sort();
    next.dedup();

    Ok(StarterReport {
        objective,
        policy: "choose starter tentacles from profile and manifest metadata; no tool execution"
            .to_string(),
        state_path: state_path_text,
        installed,
        groups,
        recommendations,
        next,
    })
}

fn starter_group<'a>(
    id: &str,
    needs: &[String],
    tools: &[String],
    evolution_surfaces: &[String],
) -> (&'a str, &'a str, &'a str) {
    let has_need = |value: &str| needs.iter().any(|need| need == value);
    let has_tool = |value: &str| tools.iter().any(|tool| tool.contains(value));
    let has_surface = |value: &str| {
        evolution_surfaces
            .iter()
            .any(|surface| surface.contains(value))
    };
    match id {
        "swe-agent" => ("repo", "Repo", "inspect, edit, and verify code work"),
        "computer-use-agent" => (
            "desktop",
            "Desktop",
            "browser, screen, clipboard, and MCP work",
        ),
        "repo-maintainer" => (
            "self_iteration",
            "Self-Iteration",
            "GitHub, PR, release, and repository maintenance",
        ),
        "harness-repair-agent" => (
            "repair",
            "Repair",
            "diagnose and repair harness feedback loops",
        ),
        "bash-only" => ("script", "Script", "write-and-run local execution harness"),
        "json-feed" => (
            "runtime",
            "Runtime",
            "structured Feed adapter and non-shell runtime seed",
        ),
        "research" => (
            "research",
            "Research",
            "source, compare, and evidence gathering",
        ),
        "memory" => (
            "memory",
            "Memory",
            "remember, recall, forget, and compact context",
        ),
        "visual" => (
            "visual",
            "Visual",
            "pixel state and color interaction layer",
        ),
        _ if has_tool("github") || has_need("publish") => (
            "self_iteration",
            "Self-Iteration",
            "GitHub, PR, release, and repository maintenance",
        ),
        _ if has_tool("browser") || has_tool("screen") || has_tool("clipboard") => (
            "desktop",
            "Desktop",
            "browser, screen, clipboard, and MCP work",
        ),
        _ if has_surface("runtime_code") || has_need("repair") => (
            "repair",
            "Repair",
            "diagnose and repair harness feedback loops",
        ),
        _ if has_need("compare") || has_need("verify") => (
            "research",
            "Research",
            "source, compare, and evidence gathering",
        ),
        _ => ("repo", "Repo", "inspect, edit, and verify code work"),
    }
}

fn starter_group_description(id: &str) -> &'static str {
    match id {
        "repo" => "inspect, edit, and verify code work",
        "desktop" => "browser, screen, clipboard, and MCP work",
        "self_iteration" => "GitHub, PR, release, and repository maintenance",
        "repair" => "diagnose and repair harness feedback loops",
        "script" => "write-and-run local execution harness",
        "runtime" => "structured Feed adapter and non-shell runtime seed",
        "research" => "source, compare, and evidence gathering",
        "memory" => "remember, recall, forget, and compact context",
        "visual" => "pixel state and color interaction layer",
        _ => "starter work",
    }
}

fn starter_signals(
    candidate: &StarterCandidate,
    matched: &[String],
    group: (&str, &str, &str),
    first_need: &str,
) -> Vec<String> {
    let mut signals = Vec::new();
    signals.push(format!("group: {} - {}", group.1, group.2));
    if !matched.is_empty() {
        signals.push(format!("objective: {}", matched.join(", ")));
    }
    signals.push(format!("first Need: {first_need}"));
    let needs = starter_set_preview(&candidate.needs, 5);
    if !needs.is_empty() {
        signals.push(format!("needs: {}", needs.join(", ")));
    }
    let tools = starter_set_preview(&candidate.tools, 5);
    if !tools.is_empty() {
        signals.push(format!("tools: {}", tools.join(", ")));
    }
    let runtimes = starter_set_preview(&candidate.runtimes, 4);
    if !runtimes.is_empty() {
        signals.push(format!("runtimes: {}", runtimes.join(", ")));
    }
    let surfaces = starter_set_preview(&candidate.evolution_surfaces, 4);
    if !surfaces.is_empty() {
        signals.push(format!("evolves: {}", surfaces.join(", ")));
    }
    signals.push(if candidate.llm_ready {
        "tool-side brain: llm".to_string()
    } else {
        "tool-side brain: manifest".to_string()
    });
    signals.push(if candidate.installed {
        "state: installed".to_string()
    } else {
        "state: available".to_string()
    });
    let mut seen = BTreeSet::new();
    signals
        .into_iter()
        .filter(|signal| seen.insert(signal.clone()))
        .collect()
}

fn starter_set_preview(values: &BTreeSet<String>, limit: usize) -> Vec<String> {
    values.iter().take(limit).cloned().collect()
}

fn starter_groups(recommendations: &[StarterRecommendation]) -> Vec<StarterGroup> {
    let mut counts = BTreeMap::<(String, String), usize>::new();
    for item in recommendations {
        *counts
            .entry((item.group.clone(), item.group_label.clone()))
            .or_insert(0) += 1;
    }
    let mut groups = counts
        .into_iter()
        .map(|((id, label), count)| StarterGroup {
            description: starter_group_description(&id).to_string(),
            id,
            label,
            count,
        })
        .collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        starter_group_priority(&left.id)
            .cmp(&starter_group_priority(&right.id))
            .then_with(|| left.label.cmp(&right.label))
    });
    groups
}

fn starter_group_priority(id: &str) -> i32 {
    match id {
        "repo" => 0,
        "desktop" => 1,
        "self_iteration" => 2,
        "repair" => 3,
        "research" => 4,
        "script" => 5,
        "runtime" => 6,
        "memory" => 7,
        "visual" => 8,
        _ => 20,
    }
}

fn starter_keywords(objective: &str) -> BTreeSet<String> {
    let mut keywords = words_for_match(objective);
    if contains_any(
        &keywords,
        &[
            "bug", "code", "edit", "file", "fix", "patch", "repo", "test",
        ],
    ) {
        keywords.extend(words_for_match(
            "code repo edit inspect patch test swe repository implementation",
        ));
    }
    if contains_any(
        &keywords,
        &[
            "browser", "computer", "desktop", "mcp", "screen", "ui", "window",
        ],
    ) {
        keywords.extend(words_for_match(
            "computer desktop browser window screenshot mcp ui clipboard",
        ));
    }
    if contains_any(
        &keywords,
        &["github", "pr", "publish", "release", "workflow", "ci"],
    ) {
        keywords.extend(words_for_match(
            "github pull request workflow release maintainer publish",
        ));
    }
    if contains_any(
        &keywords,
        &["beat", "evolve", "harness", "heartbeat", "repair"],
    ) {
        keywords.extend(words_for_match(
            "harness repair heartbeat evolution adapter diagnostics",
        ));
    }
    if contains_any(
        &keywords,
        &["compare", "evidence", "research", "source", "verify"],
    ) {
        keywords.extend(words_for_match(
            "verify compare observe evidence source research",
        ));
    }
    if contains_any(&keywords, &["forget", "memory", "recall", "remember"]) {
        keywords.extend(words_for_match("memory remember recall forget compact"));
    }
    if contains_any(&keywords, &["bash", "script", "shell"]) {
        keywords.extend(words_for_match("bash shell script execute"));
    }
    if contains_any(
        &keywords,
        &["feed", "json", "python", "runtime", "structured"],
    ) {
        keywords.extend(words_for_match("json python structured feed runtime"));
    }
    keywords
}

fn starter_matches(candidate: &StarterCandidate, keywords: &BTreeSet<String>) -> Vec<String> {
    let haystack = words_for_match(&format!(
        "{} {} {} {} {} {}",
        candidate.id,
        candidate.name,
        candidate
            .descriptions
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(" "),
        candidate
            .needs
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(" "),
        candidate
            .tools
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(" "),
        candidate
            .runtimes
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    ));
    let mut matched = keywords
        .iter()
        .filter(|keyword| haystack.contains(*keyword))
        .take(6)
        .cloned()
        .collect::<Vec<_>>();
    matched.sort();
    matched
}

fn words_for_match(value: &str) -> BTreeSet<String> {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter_map(|word| {
            let normalized = word.to_ascii_lowercase();
            ((normalized.len() > 2 || matches!(normalized.as_str(), "ci" | "pr" | "ui"))
                && !starter_stopword(&normalized))
            .then_some(normalized)
        })
        .collect()
}

fn starter_stopword(word: &str) -> bool {
    matches!(
        word,
        "and"
            | "are"
            | "can"
            | "for"
            | "from"
            | "how"
            | "into"
            | "make"
            | "next"
            | "should"
            | "that"
            | "the"
            | "this"
            | "use"
            | "what"
            | "with"
    )
}

fn contains_any(words: &BTreeSet<String>, values: &[&str]) -> bool {
    values.iter().any(|value| words.contains(*value))
}

fn starter_priority(id: &str) -> i32 {
    match id {
        "swe-agent" => 0,
        "computer-use-agent" => 1,
        "harness-repair-agent" => 2,
        "json-feed" => 3,
        "repo-maintainer" => 4,
        "bash-only" => 5,
        "research" => 6,
        "memory" => 7,
        "visual" => 8,
        _ => 20,
    }
}

fn starter_priority_score(id: &str) -> i32 {
    20 - starter_priority(id)
}

fn starter_need_kind(objective: &str, needs: &[String]) -> String {
    let words = words_for_match(objective);
    let contains = |value: &str| needs.iter().any(|need| need == value);
    if contains_any(&words, &["run", "execute", "build", "fix", "publish"]) && contains("execute") {
        "execute".to_string()
    } else if contains_any(&words, &["check", "test", "verify"]) && contains("verify") {
        "verify".to_string()
    } else if contains_any(&words, &["compare", "choose"]) && contains("compare") {
        "compare".to_string()
    } else if contains_any(&words, &["remember", "memory"]) && contains("remember") {
        "remember".to_string()
    } else if contains("observe") {
        "observe".to_string()
    } else {
        needs
            .first()
            .cloned()
            .unwrap_or_else(|| "observe".to_string())
    }
}

fn octopus_state_command(state_path: &Path, args: &[String]) -> String {
    let mut parts = vec!["octopus".to_string()];
    if state_path != Path::new(".octopus/state.json") {
        parts.push("--state".to_string());
        parts.push(shell_value(&state_path.to_string_lossy()));
    }
    parts.extend(args.iter().cloned());
    parts.join(" ")
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
            println!("need_queue: {}", report.need_queue_count);
            if let Some(item) = &report.latest_need_queue_item {
                println!("latest_need: {}", need_queue_line(item));
            }
            println!("feed_traces: {}", report.feed_trace_count);
            if let Some(trace) = &report.latest_feed_trace {
                println!("latest_trace: {}", trace_line(trace));
            }
            println!("check_history: {}", report.check_history_count);
            if let Some(record) = &report.latest_check {
                println!("latest_check: {}", check_history_line(record));
            }
            println!("repair_outcomes: {}", report.repair_outcome_count);
            if let Some(outcome) = &report.latest_repair_outcome {
                println!(
                    "latest_repair: #{} {:?} {}",
                    outcome.index, outcome.status, outcome.summary
                );
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
            println!("Need队列: {}", report.need_queue_count);
            if let Some(item) = &report.latest_need_queue_item {
                println!("最近Need: {}", need_queue_line(item));
            }
            println!("Feed轨迹数: {}", report.feed_trace_count);
            if let Some(trace) = &report.latest_feed_trace {
                println!("最近轨迹: {}", trace_line(trace));
            }
            println!("检查历史数: {}", report.check_history_count);
            if let Some(record) = &report.latest_check {
                println!("最近检查: {}", check_history_line(record));
            }
            println!("repair结果数: {}", report.repair_outcome_count);
            if let Some(outcome) = &report.latest_repair_outcome {
                println!(
                    "最近repair: #{} {:?} {}",
                    outcome.index, outcome.status, outcome.summary
                );
            }
            println!("警告: {}", join_or_none(&report.warnings));
            println!("下一步: {}", report.next_action);
        }
    }
}

fn print_context_report(report: &ContextReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus context");
            println!("brain: {}", report.brain.policy);
            println!(
                "goal: {}",
                report
                    .brain
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("none")
            );
            println!("mem: {}", report.brain.mem.len());
            if let Some(need) = &report.brain.next_need {
                println!("next_need: {} {}", need_label(&need.kind), need.query);
            }
            print_context_turns(report, language);
            print_context_tentacles(report, language);
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼上下文");
            println!("主脑: {}", report.brain.policy);
            println!(
                "目标: {}",
                report
                    .brain
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("无")
            );
            println!("记忆: {}", report.brain.mem.len());
            if let Some(need) = &report.brain.next_need {
                println!("下个Need: {} {}", need_label(&need.kind), need.query);
            }
            print_context_turns(report, language);
            print_context_tentacles(report, language);
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

fn repair_report(
    state_path: &Path,
    state: &mut HarnessState,
    query: String,
) -> Result<RepairReport, String> {
    let tentacle_id = "harness-repair-agent";
    let root = default_tentacles_root();
    let _ = state.install_profile(tentacle_id);
    state.install_manifest(&root, tentacle_id)?;
    state.save(state_path).map_err(|error| error.to_string())?;
    let llm_config = if manifest_llm_enabled() {
        Some(manifest_llm_config()?)
    } else {
        None
    };
    let need_kind = if latest_repair_plan_for_query(&query).is_some() {
        NeedKind::Verify
    } else {
        NeedKind::Observe
    };
    let need = Need::new(need_kind, query.clone());
    let mut feed = feed_tentacle(&root, tentacle_id, &need, llm_config, &state.grants)?;
    feed.metadata
        .insert("tentacle".to_string(), tentacle_id.to_string());
    state.record_pet_event_from_feed(&feed);
    let trace = state.record_feed_trace_from_feed(&feed);
    feed.metadata
        .insert("feed_trace_index".to_string(), trace.index.to_string());
    let mut queued = Vec::new();
    if let Some(next_need) = repair_next_need_from_feed(&feed) {
        queued.push(state.queue_need_suggestion(
            next_need,
            tentacle_id,
            query,
            feed.summary.clone(),
        ));
    }
    let repair_plan = repair_plan_report_from_feed(&feed);
    let queue = state.need_queue_report(8);
    let mut next = queued
        .iter()
        .map(|item| format!("octopus needs take {}", item.index))
        .collect::<Vec<_>>();
    if let Some(plan) = &repair_plan {
        if !plan.review.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.review)));
        }
        for command in [
            &plan.check_command,
            &plan.grant_command,
            &plan.apply_command,
            &plan.score_command,
        ] {
            if !command.trim().is_empty() {
                next.push(command.to_string());
            }
        }
    }
    next.push("octopus needs".to_string());
    next.push("octopus beat 200".to_string());
    next.push("octopus traces".to_string());
    next.sort();
    next.dedup();
    Ok(RepairReport {
        state_path: state_path.to_string_lossy().to_string(),
        tentacle: tentacle_id.to_string(),
        feed,
        repair_plan,
        queued,
        queue,
        next,
    })
}

fn latest_repair_plan_for_query(query: &str) -> Option<PathBuf> {
    let workspace = repair_workspace_from_query(query).ok()?;
    let repair_root = workspace.join(".octopus/harness-repair");
    let mut latest: Option<(SystemTime, PathBuf)> = None;
    for entry in fs::read_dir(repair_root).ok()? {
        let path = entry.ok()?.path().join("REPAIR_PLAN.json");
        if !path.exists() {
            continue;
        }
        let modified = fs::metadata(&path)
            .and_then(|metadata| metadata.modified())
            .unwrap_or(UNIX_EPOCH);
        if latest.as_ref().map_or(true, |(time, _)| modified > *time) {
            latest = Some((modified, path));
        }
    }
    latest.map(|(_, path)| path)
}

fn repair_workspace_from_query(query: &str) -> Result<PathBuf, String> {
    let token = query
        .split_whitespace()
        .next()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(".");
    let mut path = PathBuf::from(token);
    if !path.is_absolute() {
        path = env::current_dir()
            .map_err(|error| error.to_string())?
            .join(path);
    }
    if path.is_file() {
        path = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
    }
    if !path.exists() || !path.is_dir() {
        return env::current_dir().map_err(|error| error.to_string());
    }
    path.canonicalize().map_err(|error| error.to_string())
}

fn repair_plan_report_from_feed(feed: &Feed) -> Option<RepairPlanReport> {
    let metadata = &feed.metadata;
    let path = metadata.get("repair_plan")?.to_string();
    let score_command = metadata_value(metadata, "score_command");
    let score_command = metadata
        .get("feed_trace_index")
        .map(|index| score_command.replace("<trace-index>", index))
        .unwrap_or(score_command);
    Some(RepairPlanReport {
        path,
        review: metadata_value(metadata, "review"),
        status: metadata_value(metadata, "repair_plan_status"),
        schema: metadata_value(metadata, "repair_plan_schema"),
        session: metadata_value(metadata, "repair_plan_session"),
        target_tentacle: metadata_value(metadata, "target_tentacle"),
        target_tool: metadata_value(metadata, "target_tool"),
        candidate: metadata_value(metadata, "candidate"),
        review_boundary: metadata_value(metadata, "review_boundary"),
        check_command: metadata_value(metadata, "check_command"),
        grant_command: metadata_value(metadata, "grant_command"),
        apply_command: metadata_value(metadata, "apply_command"),
        score_command,
        suggested_commands: metadata_value(metadata, "suggested_commands"),
        next_need_kind: metadata_value(metadata, "next_need_kind"),
        next_need_query: metadata_value(metadata, "next_need_query"),
    })
}

fn metadata_value(metadata: &BTreeMap<String, String>, key: &str) -> String {
    metadata.get(key).cloned().unwrap_or_default()
}

fn repair_next_need_from_feed(feed: &Feed) -> Option<GoalNeedSuggestion> {
    let query = feed
        .metadata
        .get("next_need_query")
        .or_else(|| feed.metadata.get("next_need"))?
        .trim();
    if query.is_empty() {
        return None;
    }
    let kind = feed
        .metadata
        .get("next_need_kind")
        .and_then(|value| parse_kind(value).ok())
        .unwrap_or(NeedKind::Verify);
    Some(GoalNeedSuggestion {
        kind,
        query: query.to_string(),
    })
}

fn product_report(state: &HarnessState, state_path: &Path) -> Result<ProductReport, String> {
    let status = state.status_report_with_state(Some(state_path));
    let cwd = env::current_dir().map_err(|error| error.to_string())?;
    let environment = EnvironmentReport::detect(&cwd);
    let manifests =
        inspect_tentacle_manifests(default_tentacles_root()).map_err(|error| error.to_string())?;
    let provider = provider_status_report();
    let pet_exists = repo_root().join("docs/pet.html").exists();
    let app_exists = repo_root().join("docs/app.html").exists();
    let docs_exists = repo_root().join("docs/index.html").exists();
    let broken_manifests = manifests
        .iter()
        .filter(|manifest| !manifest.missing_entrypoints.is_empty())
        .map(|manifest| manifest.id.clone())
        .collect::<Vec<_>>();
    let runtime_kinds = manifests
        .iter()
        .flat_map(|manifest| manifest.runtime_kinds.iter().cloned())
        .collect::<BTreeSet<_>>();
    let required_surfaces = [
        "brain_prompt",
        "tool_meta",
        "runtime_code",
        "evolution_policy",
    ];
    let harness_surfaces_ready = manifests.iter().any(|manifest| {
        required_surfaces.iter().all(|surface| {
            manifest
                .evolution_surfaces
                .iter()
                .any(|item| item == surface)
        })
    });
    let provider_configured = provider.layers.iter().any(|layer| layer.configured);
    let provider_ready = provider
        .layers
        .iter()
        .all(|layer| layer.enabled && layer.configured && layer.curl_available);
    let github_ready = state.active_grant("github", "dangoZhang/Octopus").is_some();

    let mut capabilities = vec![
        product_capability(
            "clean_brain",
            "ready",
            "brain context is Goal + Mem + Need + Feed",
            Some("octopus context observe ."),
        ),
        product_capability(
            "context_inspection",
            "ready",
            "CLI, JSON, and native app expose clean-brain and tentacle context boundaries",
            Some("octopus context observe ."),
        ),
        product_capability(
            "clean_brain_explore",
            "ready",
            "Goal/Mem/Need/Feed exploration returns Need suggestions without tool execution",
            Some("octopus explore"),
        ),
        product_capability(
            "clean_brain_audit",
            "ready",
            "clean-brain Goal and exploration reports flag implementation burden and expose only clean Needs for queueing",
            Some("octopus brain --live \"what should the brain ask next?\""),
        ),
        product_capability(
            "need_queue",
            "ready",
            format!(
                "{} pending clean-brain Need suggestions",
                status.need_queue_count
            ),
            Some("octopus explore --save"),
        ),
        product_capability(
            "need_queue_script",
            "ready",
            "writes pending clean-brain Needs into a reviewable Feed script without execution",
            Some("octopus needs script"),
        ),
        product_capability(
            "need_queue_review",
            "ready",
            "writes prompt, messages, review template, and optional live draft for pending Needs",
            Some("octopus needs session"),
        ),
        product_capability(
            "harness_repair_queue",
            if state
                .installed_tentacles
                .iter()
                .any(|tentacle| tentacle.id == "harness-repair-agent")
            {
                "ready"
            } else {
                "available"
            },
            "harness-repair Feed can queue the next clean-brain Need",
            Some("octopus repair ."),
        ),
        product_capability(
            "clean_brain_prompt",
            "ready",
            "exports pasteable Goal/Mem/Need/Feed messages for any chat model",
            Some("octopus brain"),
        ),
        product_capability(
            "external_brain_reply",
            "ready",
            "imports chat-model JSON into Goal or Need Queue without Feed execution",
            Some("octopus brain --apply reply.json --save"),
        ),
        product_capability(
            "brain_session",
            "ready",
            "writes prompt, messages, reply template, optional live draft, and apply command for external chat",
            Some("octopus brain --session"),
        ),
        product_capability(
            "tentacle_brains",
            if status.tentacles.is_empty() {
                "available"
            } else {
                "ready"
            },
            format!(
                "{} installed, {} manifests",
                status.tentacles.len(),
                manifests.len()
            ),
            Some("octopus install swe-agent"),
        ),
        product_capability(
            "runtime_neutral_harness",
            if harness_surfaces_ready {
                "ready"
            } else {
                "needs_manifest"
            },
            format!(
                "runtimes: {}; surfaces: {}",
                join_or_none(&runtime_kinds.iter().cloned().collect::<Vec<_>>()),
                required_surfaces.join(", ")
            ),
            Some("octopus manifests"),
        ),
        product_capability(
            "three_hearts",
            if status.hearts.len() >= 3 {
                "ready"
            } else {
                "missing"
            },
            status
                .hearts
                .iter()
                .map(|beat| format!("{}={}", beat.name, beat.summary))
                .collect::<Vec<_>>()
                .join(", "),
            Some("octopus beat 200"),
        ),
        product_capability(
            "color_pet",
            if pet_exists { "ready" } else { "missing" },
            repo_root()
                .join("docs/pet.html")
                .to_string_lossy()
                .to_string(),
            Some("octopus pet"),
        ),
        product_capability(
            "native_html_app",
            if app_exists { "ready" } else { "missing" },
            repo_root()
                .join("docs/app.html")
                .to_string_lossy()
                .to_string(),
            Some("octopus bridge"),
        ),
        product_capability(
            "starter_panel",
            if app_exists { "ready" } else { "missing" },
            "native HTML app renders and filters starter tentacle recommendations with evidence signals, install, check, and first-Need actions",
            Some("octopus starter \"build a clean-brain agent\""),
        ),
        product_capability(
            "local_bootstrap",
            "ready",
            "init, environment adaptation, seed tentacle install, heartbeat, and report in one command",
            Some("octopus bootstrap"),
        ),
        product_capability(
            "harness_apply_review",
            "ready",
            "harness beat recommendations can be granted and written as reviewable apply artifacts",
            Some("octopus evolve apply swe-agent 03-runtime-code"),
        ),
        product_capability(
            "harness_self_repair",
            if status
                .tentacles
                .iter()
                .any(|tentacle| tentacle.id == "harness-repair-agent")
            {
                "ready"
            } else {
                "available"
            },
            format!(
                "harness-repair-agent can turn signals into repair Feed; {} scored repair outcomes",
                status.repair_outcome_count
            ),
            Some("octopus install harness-repair-agent"),
        ),
        product_capability(
            "goal_setting",
            "ready",
            "human goals can be set directly without Feed execution",
            Some("octopus goal set \"build a clean-brain agent\""),
        ),
        product_capability(
            "route_report",
            "ready",
            format!(
                "{} route scores, {} Feed traces",
                status.route_count, status.feed_trace_count
            ),
            Some("octopus routes observe ."),
        ),
        product_capability(
            "provider_layers",
            if provider_ready {
                "ready"
            } else if provider_configured {
                "configured"
            } else {
                "setup_needed"
            },
            provider
                .layers
                .iter()
                .map(|layer| {
                    format!(
                        "{} enabled={} configured={}",
                        layer.layer, layer.enabled, layer.configured
                    )
                })
                .collect::<Vec<_>>()
                .join("; "),
            Some("octopus provider status"),
        ),
        product_capability(
            "self_iteration",
            if github_ready {
                "pr_ready"
            } else {
                "oauth_required"
            },
            state.self_iteration_plan("dangoZhang/Octopus", None).mode,
            Some("octopus self-iterate dangoZhang/Octopus"),
        ),
        product_capability(
            "docs_home",
            if docs_exists { "ready" } else { "missing" },
            "README story plus native docs homepage".to_string(),
            Some("open docs/index.html"),
        ),
    ];
    capabilities.sort_by(|left, right| left.id.cmp(&right.id));

    let mut gaps = Vec::new();
    let state_args = format!(" --state {}", shell_arg(&state_path.to_string_lossy()));
    if status.goal.is_none() {
        gaps.push(product_gap(
            "goal_loop_empty",
            "human goal has not been set in this state",
            format!("octopus{state_args} goal set \"describe your goal\""),
        ));
    }
    if status.tentacles.is_empty() {
        gaps.push(product_gap(
            "no_installed_tentacles",
            "Needs cannot reach tool-side brains yet",
            format!("octopus{state_args} bootstrap"),
        ));
    }
    if !broken_manifests.is_empty() {
        gaps.push(product_gap(
            "broken_manifests",
            format!("missing entrypoints: {}", broken_manifests.join(", ")),
            "octopus manifests",
        ));
    }
    if !provider_ready {
        let next = if provider_configured {
            "octopus provider check"
        } else {
            "octopus provider save openai"
        };
        gaps.push(product_gap(
            "provider_feedback",
            "LLM-backed chat, tentacle planning, or evolution is not fully live",
            next,
        ));
    }
    if status.feed_trace_count == 0 {
        gaps.push(product_gap(
            "feed_trace_empty",
            "harness evolution has no Feed data to learn from",
            "octopus need observe .",
        ));
    }
    if status.check_history_count == 0 {
        gaps.push(product_gap(
            "check_history_empty",
            "harness beat cannot target failing runtime checks yet",
            "octopus check swe-agent",
        ));
    }
    if state.evolution_outcomes.is_empty() {
        gaps.push(product_gap(
            "evolution_scores_empty",
            "recommendations have not received outcome feedback",
            "octopus evolve score swe-agent 03-runtime-code partial \"reviewed\"",
        ));
    }
    if status.repair_outcome_count == 0 {
        gaps.push(product_gap(
            "repair_outcomes_empty",
            "harness repair sessions have no scored outcome memory yet",
            "octopus repair score <trace-index> satisfied \"repair improved harness\"",
        ));
    }
    if environment.recommended_profiles.is_empty() {
        gaps.push(product_gap(
            "environment_signal_sparse",
            "adapter selection has little local project evidence",
            "octopus env",
        ));
    }
    gaps.push(product_gap(
        "real_machine_gate",
        "0.1.0 and later tags need recorded real-machine testing",
        "update docs/real-machine-test.md before a 0.1.0 tag",
    ));
    gaps.sort_by(|left, right| left.id.cmp(&right.id));

    let mut next = gaps.iter().map(|gap| gap.next.clone()).collect::<Vec<_>>();
    next.push(status.next_action);
    next.extend(provider.next);
    next.sort();
    next.dedup();

    Ok(ProductReport {
        version: env!("CARGO_PKG_VERSION").to_string(),
        state_path: state_path.to_string_lossy().to_string(),
        state_exists: state_path.exists(),
        context: ProductContextPolicy {
            brain: "Goal + Mem + Need + Feed".to_string(),
            tentacle: "Need + Tool + Action + Tool + Action -> Feed".to_string(),
            feedback_loop: "Need -> Feed -> Feedback".to_string(),
        },
        capabilities,
        gaps,
        next,
    })
}

fn product_capability(
    id: impl Into<String>,
    status: impl Into<String>,
    evidence: impl Into<String>,
    command: Option<&str>,
) -> ProductCapability {
    ProductCapability {
        id: id.into(),
        status: status.into(),
        evidence: evidence.into(),
        command: command.map(str::to_string),
    }
}

fn product_gap(
    id: impl Into<String>,
    impact: impl Into<String>,
    next: impl Into<String>,
) -> ProductGap {
    ProductGap {
        id: id.into(),
        impact: impact.into(),
        next: next.into(),
    }
}

fn preflight_report(
    state: &HarnessState,
    state_path: &Path,
    live: bool,
) -> Result<PreflightReport, String> {
    let product = product_report(state, state_path)?;
    let doctor = doctor_report(state, state_path.to_path_buf())?;
    let provider = provider_status_report();
    let status = &doctor.status;
    let current_head = git_short_head();
    let real_machine_doc = repo_root().join("docs/real-machine-test.md");
    let real_machine_log = fs::read_to_string(&real_machine_doc).unwrap_or_default();
    let real_machine_record =
        real_machine_record_status(current_head.as_deref(), &real_machine_log);
    let seed_tentacles = bootstrap_seed_tentacles();
    let missing_seed_tentacles = seed_tentacles
        .iter()
        .filter(|id| {
            !state
                .installed_tentacles
                .iter()
                .any(|tentacle| tentacle.id == **id)
        })
        .map(|id| (*id).to_string())
        .collect::<Vec<_>>();
    let provider_layers_ready = provider.layers.iter().all(|layer| {
        layer.enabled && layer.configured && layer.api_key_present && layer.curl_available
    });
    let github_grant = state.active_grant("github", "dangoZhang/Octopus").is_some();
    let gh_ready = command_ready("gh");
    let docs_ready = repo_root().join("README.md").exists()
        && repo_root().join("docs/index.html").exists()
        && repo_root().join("docs/app.html").exists()
        && repo_root().join("docs/pet.html").exists();
    let feedback_ready = status.feed_trace_count > 0
        && status.check_history_count > 0
        && !state.evolution_outcomes.is_empty();
    let mut live_provider = None;
    let mut live_provider_error = None;
    if live {
        match provider_check_report(
            &doctor.llm.config_prefix,
            "Reply with 'octopus provider live'.",
        ) {
            Ok(report) => live_provider = Some(report),
            Err(error) => live_provider_error = Some(error),
        }
    }

    let mut checks = vec![
        preflight_check(
            "state",
            state_path.exists(),
            true,
            format!(
                "{} ({})",
                state_path.to_string_lossy(),
                if state_path.exists() { "exists" } else { "missing" }
            ),
            "octopus bootstrap",
        ),
        preflight_check(
            "seed_tentacles",
            missing_seed_tentacles.is_empty(),
            true,
            if missing_seed_tentacles.is_empty() {
                format!("{} seed tentacles installed", seed_tentacles.len())
            } else {
                format!("missing {}", missing_seed_tentacles.join(", "))
            },
            "octopus bootstrap",
        ),
        preflight_check(
            "manifest_integrity",
            doctor.broken_manifests.is_empty() && doctor.manifest_count > 0,
            true,
            if doctor.broken_manifests.is_empty() {
                format!("{} manifests clean", doctor.manifest_count)
            } else {
                doctor.broken_manifests.join("; ")
            },
            "octopus manifests",
        ),
        preflight_check(
            "clean_brain_boundary",
            product.context.brain == CLEAN_BRAIN_CONTEXT_POLICY
                && product.context.tentacle == TENTACLE_CONTEXT_POLICY,
            true,
            format!(
                "brain={}, tentacle={}",
                product.context.brain, product.context.tentacle
            ),
            "octopus context observe .",
        ),
        preflight_check(
            "docs_and_pet",
            docs_ready,
            true,
            "README, docs homepage, app, and pixel pet files",
            "octopus bridge",
        ),
        preflight_check(
            "provider_layers",
            provider_layers_ready,
            true,
            provider
                .layers
                .iter()
                .map(|layer| {
                    format!(
                        "{} enabled={} configured={} key={} curl={}",
                        layer.layer,
                        layer.enabled,
                        layer.configured,
                        layer.api_key_present,
                        layer.curl_available
                    )
                })
                .collect::<Vec<_>>()
                .join("; "),
            "octopus provider save openai; source .octopus/llm.env; octopus provider status",
        ),
        preflight_status_check(
            "live_provider",
            if live {
                if live_provider.is_some() {
                    "pass"
                } else {
                    "fail"
                }
            } else {
                "skipped"
            },
            true,
            live_provider
                .as_ref()
                .map(|report| report.content.clone())
                .or(live_provider_error)
                .unwrap_or_else(|| "not run; pass --live to call the provider".to_string()),
            format!("octopus --state {} preflight --live", shell_arg(&state_path.to_string_lossy())),
        ),
        preflight_check(
            "feedback_data",
            feedback_ready,
            true,
            format!(
                "traces={}, checks={}, evolution_scores={}",
                status.feed_trace_count,
                status.check_history_count,
                state.evolution_outcomes.len()
            ),
            "octopus need observe .; octopus check swe-agent; octopus evolve score swe-agent 03-runtime-code partial \"reviewed\"",
        ),
        preflight_check(
            "github_pr_path",
            github_grant && gh_ready,
            true,
            format!("gh_cli={}, oauth_grant={}", gh_ready, github_grant),
            "gh auth login; octopus oauth github dangoZhang/Octopus repo workflow",
        ),
        preflight_check(
            "real_machine_record",
            real_machine_record.passed,
            true,
            real_machine_record.evidence,
            "octopus preflight record",
        ),
        preflight_check(
            "desktop_adapters",
            state
                .installed_tentacles
                .iter()
                .any(|tentacle| tentacle.id == "computer-use-agent"),
            false,
            "computer-use-agent installed for desktop/browser probes",
            "octopus install computer-use-agent",
        ),
        preflight_check(
            "harness_repair",
            state
                .installed_tentacles
                .iter()
                .any(|tentacle| tentacle.id == "harness-repair-agent"),
            false,
            "harness-repair-agent installed for tool-side diagnosis",
            "octopus install harness-repair-agent",
        ),
    ];
    checks.sort_by(|left, right| left.id.cmp(&right.id));
    let release_ready = checks
        .iter()
        .filter(|check| check.required)
        .all(|check| check.status == "pass");
    let mut next = checks
        .iter()
        .filter(|check| check.status != "pass")
        .map(|check| check.next.clone())
        .collect::<Vec<_>>();
    next.push("octopus preflight script".to_string());
    next.push("octopus preflight record".to_string());
    next.push("octopus report".to_string());
    next.sort();
    next.dedup();

    Ok(PreflightReport {
        version: env!("CARGO_PKG_VERSION").to_string(),
        target: "0.1.0".to_string(),
        state_path: state_path.to_string_lossy().to_string(),
        live,
        release_ready,
        current_head,
        checks,
        live_provider,
        next,
    })
}

fn preflight_check(
    id: impl Into<String>,
    passed: bool,
    required: bool,
    evidence: impl Into<String>,
    next: impl Into<String>,
) -> PreflightCheck {
    preflight_status_check(
        id,
        if passed {
            "pass"
        } else if required {
            "fail"
        } else {
            "warn"
        },
        required,
        evidence,
        next,
    )
}

fn preflight_status_check(
    id: impl Into<String>,
    status: impl Into<String>,
    required: bool,
    evidence: impl Into<String>,
    next: impl Into<String>,
) -> PreflightCheck {
    PreflightCheck {
        id: id.into(),
        status: status.into(),
        required,
        evidence: evidence.into(),
        next: next.into(),
    }
}

fn write_preflight_script(
    state_path: &Path,
    output_path: &Path,
) -> Result<PreflightScriptReport, String> {
    if let Some(parent) = output_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let commands = preflight_script_commands();
    let state_default = shell_arg(&state_path.to_string_lossy());
    let content = format!(
        r#"#!/bin/sh
set -eu

STATE=${{OCTOPUS_PREFLIGHT_STATE:-{state_default}}}
ROOT=${{OCTOPUS_PREFLIGHT_ROOT:-$(pwd)}}

cd "$ROOT"

{commands}

if [ "${{OCTOPUS_PREFLIGHT_LIVE:-0}}" = "1" ]; then
  octopus provider status
  octopus provider check "${{OCTOPUS_LLM_PREFIX:-OCTOPUS_LLM}}"
  octopus --state "$STATE" preflight --live
else
  octopus --state "$STATE" preflight
fi

if [ "${{OCTOPUS_PREFLIGHT_PR_DRY_RUN:-0}}" = "1" ]; then
  gh auth status
  octopus --state "$STATE" self-iterate dangoZhang/Octopus
  OCTOPUS_PR_DRY_RUN=1 octopus --state "$STATE" self-iterate pr dangoZhang/Octopus "preflight dry run"
fi
"#,
        commands = commands.join("\n")
    );
    fs::write(output_path, content).map_err(|error| error.to_string())?;
    Ok(PreflightScriptReport {
        path: output_path.to_string_lossy().to_string(),
        state_path: state_path.to_string_lossy().to_string(),
        commands,
        next: vec![
            format!("sh {}", shell_arg(&output_path.to_string_lossy())),
            format!(
                "OCTOPUS_PREFLIGHT_LIVE=1 sh {}",
                shell_arg(&output_path.to_string_lossy())
            ),
            format!(
                "OCTOPUS_PREFLIGHT_PR_DRY_RUN=1 sh {}",
                shell_arg(&output_path.to_string_lossy())
            ),
        ],
    })
}

fn write_preflight_record(
    state_path: &Path,
    output_path: &Path,
) -> Result<PreflightRecordReport, String> {
    if let Some(parent) = output_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let current_head = git_short_head();
    let version = env!("CARGO_PKG_VERSION").to_string();
    let commands = preflight_record_commands(state_path, current_head.as_deref());
    let head_for_record = current_head.as_deref().unwrap_or("unknown");
    let content = format!(
        r#"# Real-Machine Record

Append the completed result to `docs/real-machine-test.md` after the run.

## Machine

- Date:
- Tester:
- Machine:
- OS:
- Shell:
- Rust:
- Python:
- Git commit: `{head}`
- Package version: `{version}`
- State: `{state}`

## Commands

```bash
{commands}
```

## Results

- Install and doctor:
- Core loop:
- Bridge/app:
- Live provider:
- PR dry run:

## Decision

- Pass or fail:
- Follow-up:
"#,
        head = head_for_record,
        version = version,
        state = state_path.to_string_lossy(),
        commands = commands.join("\n")
    );
    fs::write(output_path, content).map_err(|error| error.to_string())?;
    Ok(PreflightRecordReport {
        path: output_path.to_string_lossy().to_string(),
        state_path: state_path.to_string_lossy().to_string(),
        version,
        current_head,
        commands,
        next: vec![
            format!("review {}", shell_arg(&output_path.to_string_lossy())),
            "run the commands on a clean real machine".to_string(),
            "append the completed result to docs/real-machine-test.md".to_string(),
            "octopus preflight".to_string(),
        ],
    })
}

fn preflight_script_commands() -> Vec<String> {
    [
        "octopus --version",
        "octopus --state \"$STATE\" bootstrap",
        "octopus --state \"$STATE\" doctor",
        "octopus --state \"$STATE\" install swe-agent",
        "octopus --state \"$STATE\" install computer-use-agent",
        "octopus --state \"$STATE\" install harness-repair-agent",
        "octopus --state \"$STATE\" install bash-only",
        "octopus --state \"$STATE\" goal set \"build a clean-brain agent\"",
        "octopus --state \"$STATE\" explore \"what should the brain ask next?\"",
        "octopus --state \"$STATE\" context observe .",
        "octopus --state \"$STATE\" think swe-agent observe README.md",
        "octopus --state \"$STATE\" need observe README.md",
        "octopus --state \"$STATE\" traces",
        "octopus --state \"$STATE\" repair .",
        "octopus --state \"$STATE\" needs",
        "octopus --state \"$STATE\" check swe-agent",
        "octopus --state \"$STATE\" feedback 1 satisfied \"preflight feed worked\"",
        "octopus --state \"$STATE\" routes observe README.md",
        "octopus --state \"$STATE\" beat 200",
        "octopus --state \"$STATE\" pet harness",
        "octopus --state \"$STATE\" report",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn preflight_record_commands(state_path: &Path, current_head: Option<&str>) -> Vec<String> {
    let rev = current_head.unwrap_or("HEAD");
    let state_default = shell_arg(&state_path.to_string_lossy());
    let mut commands = vec![
        "tmp=$(mktemp -d)".to_string(),
        format!(
            "cargo install --git https://github.com/dangoZhang/Octopus --rev {} octopus-core --locked --bin octopus --force --root \"$tmp/root\"",
            shell_arg(rev)
        ),
        "OCTOPUS=\"$tmp/root/bin/octopus\"".to_string(),
        format!("STATE=${{OCTOPUS_PREFLIGHT_STATE:-{state_default}}}"),
    ];
    commands.extend(preflight_script_commands().into_iter().map(|command| {
        command
            .replace("octopus --version", "\"$OCTOPUS\" --version")
            .replace(
                "octopus --state \"$STATE\"",
                "\"$OCTOPUS\" --state \"$STATE\"",
            )
    }));
    commands.extend([
        "\"$OCTOPUS\" --state \"$STATE\" preflight".to_string(),
        "\"$OCTOPUS\" provider status".to_string(),
        "\"$OCTOPUS\" provider check \"${OCTOPUS_LLM_PREFIX:-OCTOPUS_LLM}\"".to_string(),
        "\"$OCTOPUS\" --state \"$STATE\" preflight --live".to_string(),
        "\"$OCTOPUS\" bridge 127.0.0.1:18765 &".to_string(),
        "BRIDGE_PID=$!".to_string(),
        "trap 'kill \"$BRIDGE_PID\" 2>/dev/null || true' EXIT".to_string(),
        "sleep 1".to_string(),
        "curl -fsS http://127.0.0.1:18765/app.html >/dev/null".to_string(),
        "curl -fsS 'http://127.0.0.1:18765/pet.html?state=harness' >/dev/null".to_string(),
        "curl -fsS -X POST http://127.0.0.1:18765/api/run -H 'content-type: application/json' --data-binary \"{\\\"args\\\":[\\\"--state\\\",\\\"$STATE\\\",\\\"--json\\\",\\\"doctor\\\"]}\"".to_string(),
        "kill \"$BRIDGE_PID\"".to_string(),
        "\"$OCTOPUS\" --state \"$STATE\" self-iterate dangoZhang/Octopus".to_string(),
        "OCTOPUS_PR_DRY_RUN=1 \"$OCTOPUS\" --state \"$STATE\" self-iterate pr dangoZhang/Octopus \"preflight dry run\"".to_string(),
    ]);
    commands
}

fn git_short_head() -> Option<String> {
    git_short_rev("HEAD")
}

fn real_machine_record_status(current_head: Option<&str>, log: &str) -> RealMachineRecordStatus {
    let parent_head = git_short_rev("HEAD^");
    let changed_files = git_changed_files("HEAD^", "HEAD");
    real_machine_record_status_from_parts(
        current_head,
        parent_head.as_deref(),
        changed_files.as_deref(),
        log,
    )
}

fn real_machine_record_status_from_parts(
    current_head: Option<&str>,
    parent_head: Option<&str>,
    changed_files: Option<&[String]>,
    log: &str,
) -> RealMachineRecordStatus {
    let Some(head) = current_head else {
        return RealMachineRecordStatus {
            passed: false,
            evidence: "git head unavailable".to_string(),
        };
    };

    if log.contains(head) {
        return RealMachineRecordStatus {
            passed: true,
            evidence: format!("head {head} recorded=true"),
        };
    }

    let parent_recorded = parent_head.is_some_and(|parent| log.contains(parent));
    let docs_only = changed_files.is_some_and(|files| {
        !files.is_empty()
            && files
                .iter()
                .all(|path| real_machine_record_doc_path(path.as_str()))
    });

    if parent_recorded && docs_only {
        return RealMachineRecordStatus {
            passed: true,
            evidence: format!(
                "head {head} recorded=false; parent {} recorded=true; docs_only=true",
                parent_head.unwrap_or("unknown")
            ),
        };
    }

    RealMachineRecordStatus {
        passed: false,
        evidence: format!(
            "head {head} recorded=false; parent {} recorded={parent_recorded}; docs_only={docs_only}",
            parent_head.unwrap_or("unknown")
        ),
    }
}

fn real_machine_record_doc_path(path: &str) -> bool {
    matches!(
        path,
        "docs/real-machine-test.md" | "docs/product-gap.md" | "docs/version-plan.md"
    )
}

fn git_short_rev(rev: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short", rev])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_changed_files(base: &str, head: &str) -> Option<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--name-only", base, head])
        .output()
        .ok()?;
    output.status.success().then(|| {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect()
    })
}

fn print_product_report(report: &ProductReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus report");
            println!("version: {}", report.version);
            println!(
                "state: {} ({})",
                report.state_path,
                if report.state_exists { "exists" } else { "new" }
            );
            println!("brain: {}", report.context.brain);
            println!("tentacle: {}", report.context.tentacle);
            println!("loop: {}", report.context.feedback_loop);
            println!("capabilities:");
            for capability in &report.capabilities {
                let command = capability
                    .command
                    .as_ref()
                    .map(|command| format!(" next={command}"))
                    .unwrap_or_default();
                println!(
                    "- {} [{}]: {}{}",
                    capability.id, capability.status, capability.evidence, command
                );
            }
            println!("gaps:");
            for gap in &report.gaps {
                println!("- {}: {}; next={}", gap.id, gap.impact, gap.next);
            }
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼报告");
            println!("版本: {}", report.version);
            println!(
                "状态文件: {} ({})",
                report.state_path,
                if report.state_exists {
                    "存在"
                } else {
                    "新建"
                }
            );
            println!("主脑上下文: {}", report.context.brain);
            println!("触手上下文: {}", report.context.tentacle);
            println!("循环: {}", report.context.feedback_loop);
            println!("能力:");
            for capability in &report.capabilities {
                let command = capability
                    .command
                    .as_ref()
                    .map(|command| format!(" 下一步={command}"))
                    .unwrap_or_default();
                println!(
                    "- {} [{}]: {}{}",
                    capability.id, capability.status, capability.evidence, command
                );
            }
            println!("缺口:");
            for gap in &report.gaps {
                println!("- {}: {}; 下一步={}", gap.id, gap.impact, gap.next);
            }
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

fn print_preflight_report(report: &PreflightReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus preflight");
            println!("target: {}", report.target);
            println!("version: {}", report.version);
            println!("state: {}", report.state_path);
            println!("live: {}", report.live);
            println!("release_ready: {}", report.release_ready);
            println!(
                "head: {}",
                report.current_head.as_deref().unwrap_or("unknown")
            );
            println!("checks:");
            for check in &report.checks {
                println!(
                    "- {} [{} required={}]: {}; next={}",
                    check.id, check.status, check.required, check.evidence, check.next
                );
            }
            if let Some(provider) = &report.live_provider {
                println!("live_provider_reply: {}", provider.content);
            }
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼发布前检查");
            println!("目标版本: {}", report.target);
            println!("版本: {}", report.version);
            println!("状态文件: {}", report.state_path);
            println!("Live检查: {}", report.live);
            println!("可发布: {}", report.release_ready);
            println!(
                "当前提交: {}",
                report.current_head.as_deref().unwrap_or("未知")
            );
            println!("检查项:");
            for check in &report.checks {
                println!(
                    "- {} [{} 必需={}]: {}; 下一步={}",
                    check.id, check.status, check.required, check.evidence, check.next
                );
            }
            if let Some(provider) = &report.live_provider {
                println!("Live Provider 回复: {}", provider.content);
            }
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

fn print_preflight_script_report(report: &PreflightScriptReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus preflight script");
            println!("path: {}", report.path);
            println!("state: {}", report.state_path);
            println!("commands: {}", report.commands.len());
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼发布前检查脚本");
            println!("路径: {}", report.path);
            println!("状态文件: {}", report.state_path);
            println!("命令数: {}", report.commands.len());
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

fn print_preflight_record_report(report: &PreflightRecordReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus preflight record");
            println!("path: {}", report.path);
            println!("version: {}", report.version);
            println!(
                "head: {}",
                report.current_head.as_deref().unwrap_or("unknown")
            );
            println!("commands: {}", report.commands.len());
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼实机记录模板");
            println!("路径: {}", report.path);
            println!("版本: {}", report.version);
            println!(
                "当前提交: {}",
                report.current_head.as_deref().unwrap_or("未知")
            );
            println!("命令数: {}", report.commands.len());
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

fn print_brain_explore(report: &BrainExploreReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus explore");
            println!("source: {}", report.source);
            println!("brain: {}", report.policy);
            println!(
                "goal: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("none")
            );
            println!("summary: {}", report.summary);
            println!("audit: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("next: {next}");
            }
        }
        Language::Zh => {
            println!("章鱼探索");
            println!("来源: {}", report.source);
            println!("主脑: {}", report.policy);
            println!(
                "目标: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("无")
            );
            println!("摘要: {}", report.summary);
            println!("审计: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("Need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("下一步: {next}");
            }
        }
    }
}

fn print_brain_goal_save(report: &BrainGoalSaveReport, language: Language) {
    print_brain_goal(&report.goal, language);
    match language {
        Language::En => println!("queued: {}", report.queued.len()),
        Language::Zh => println!("已入队: {}", report.queued.len()),
    }
    print_need_queue(&report.queue, language);
}

fn print_brain_goal(report: &BrainGoalReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus brain goal");
            println!("source: {}", report.source);
            println!("brain: {}", report.policy);
            println!("goal: {}", report.goal.objective);
            println!("constraints: {}", report.goal.constraints.len());
            println!("summary: {}", report.summary);
            println!("audit: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("next: {next}");
            }
        }
        Language::Zh => {
            println!("章鱼主脑目标");
            println!("来源: {}", report.source);
            println!("主脑: {}", report.policy);
            println!("目标: {}", report.goal.objective);
            println!("约束: {}", report.goal.constraints.len());
            println!("摘要: {}", report.summary);
            println!("审计: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("Need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("下一步: {next}");
            }
        }
    }
}

fn clean_brain_print_needs<'a>(
    audit: &'a octopus_core::BrainNeedAudit,
    raw_needs: &'a [GoalNeedSuggestion],
) -> &'a [GoalNeedSuggestion] {
    if audit.issue_count == 0 {
        raw_needs
    } else {
        &audit.clean_needs
    }
}

fn print_polluted_need_count(audit: &octopus_core::BrainNeedAudit, language: Language) {
    if audit.issue_count == 0 {
        return;
    }
    match language {
        Language::En => println!("blocked_needs: {}", audit.issue_count),
        Language::Zh => println!("已阻止Need: {}", audit.issue_count),
    }
}

fn brain_audit_line(audit: &octopus_core::BrainNeedAudit) -> String {
    if audit.issue_count == 0 {
        return audit.summary.clone();
    }
    let signals = audit
        .issues
        .iter()
        .map(|issue| format!("#{} {}", issue.index, issue.signal))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{} ({signals})", audit.summary)
}

fn print_need_queue_save(report: &NeedQueueSaveReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus Need queue");
            println!("saved: {}", report.queued.len());
        }
        Language::Zh => {
            println!("章鱼 Need 队列");
            println!("已保存: {}", report.queued.len());
        }
    }
    print_need_queue(&report.queue, language);
}

fn print_need_queue(report: &NeedQueueReport, language: Language) {
    match language {
        Language::En => {
            println!("Need queue");
            println!("brain: {}", report.policy);
            if report.pending.is_empty() {
                println!("pending: none");
            } else {
                println!("pending:");
            }
        }
        Language::Zh => {
            println!("Need 队列");
            println!("主脑: {}", report.policy);
            if report.pending.is_empty() {
                println!("待处理: 无");
            } else {
                println!("待处理:");
            }
        }
    }
    for item in &report.pending {
        print_need_queue_item(item, language);
    }
    if !report.history.is_empty() {
        match language {
            Language::En => println!("history:"),
            Language::Zh => println!("历史:"),
        }
        for item in &report.history {
            print_need_queue_item(item, language);
        }
    }
    for next in &report.next {
        match language {
            Language::En => println!("next: {next}"),
            Language::Zh => println!("下一步: {next}"),
        }
    }
}

fn print_need_queue_take(report: &NeedQueueTakeReport, language: Language) {
    print_need_queue_item(&report.item, language);
    match language {
        Language::En => println!("command: {}", report.command),
        Language::Zh => println!("命令: {}", report.command),
    }
    for next in &report.next {
        match language {
            Language::En => println!("next: {next}"),
            Language::Zh => println!("下一步: {next}"),
        }
    }
}

fn print_need_queue_script(report: &NeedQueueScriptReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus Need script");
            println!("brain: {}", report.policy);
            println!("path: {}", report.script_path);
            println!("commands: {}", report.command_count);
        }
        Language::Zh => {
            println!("章鱼 Need 脚本");
            println!("主脑: {}", report.policy);
            println!("路径: {}", report.script_path);
            println!("命令数: {}", report.command_count);
        }
    }
    for next in &report.next {
        match language {
            Language::En => println!("next: {next}"),
            Language::Zh => println!("下一步: {next}"),
        }
    }
}

fn print_need_queue_review_session(report: &NeedQueueReviewSessionReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus Need review session");
            println!("brain: {}", report.policy);
            println!("pending: {}", report.pending_count);
            println!("session: {}", report.session_dir);
            println!("prompt: {}", report.prompt_path);
            println!("messages: {}", report.messages_path);
            println!("review: {}", report.review_path);
            if let Some(draft_path) = &report.draft_path {
                println!("draft: {draft_path}");
            }
            println!("commands: {}", report.command_path);
        }
        Language::Zh => {
            println!("章鱼 Need 审阅会话");
            println!("主脑: {}", report.policy);
            println!("待处理: {}", report.pending_count);
            println!("会话: {}", report.session_dir);
            println!("提示词: {}", report.prompt_path);
            println!("消息: {}", report.messages_path);
            println!("审阅: {}", report.review_path);
            if let Some(draft_path) = &report.draft_path {
                println!("草稿: {draft_path}");
            }
            println!("命令: {}", report.command_path);
        }
    }
    for next in &report.next {
        match language {
            Language::En => println!("next: {next}"),
            Language::Zh => println!("下一步: {next}"),
        }
    }
}

fn print_need_queue_item(item: &NeedQueueItem, language: Language) {
    let status = match &item.status {
        NeedQueueStatus::Pending => match language {
            Language::En => "pending",
            Language::Zh => "待处理",
        },
        NeedQueueStatus::Taken => match language {
            Language::En => "taken",
            Language::Zh => "已取用",
        },
        NeedQueueStatus::Dropped => match language {
            Language::En => "dropped",
            Language::Zh => "已丢弃",
        },
    };
    match language {
        Language::En => println!(
            "#{} [{}] {} {}",
            item.index,
            status,
            need_label(&item.need.kind),
            item.need.query
        ),
        Language::Zh => println!(
            "#{} [{}] {} {}",
            item.index,
            status,
            need_label(&item.need.kind),
            item.need.query
        ),
    }
}

fn print_brain_prompt(report: &BrainPromptReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus brain prompt");
            println!("brain: {}", report.policy);
            println!(
                "goal: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("none")
            );
            println!("messages: {}", report.messages.len());
            println!("{}", report.prompt_text);
            for next in &report.next {
                println!("next: {next}");
            }
        }
        Language::Zh => {
            println!("章鱼主脑提示词");
            println!("主脑: {}", report.policy);
            println!(
                "目标: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("无")
            );
            println!("消息: {}", report.messages.len());
            println!("{}", report.prompt_text);
            for next in &report.next {
                println!("下一步: {next}");
            }
        }
    }
}

fn print_brain_session(report: &BrainSessionReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus brain session");
            println!("mode: {}", report.mode);
            println!("brain: {}", report.policy);
            println!("session: {}", report.session_dir);
            println!("prompt: {}", report.prompt_path);
            println!("messages: {}", report.messages_path);
            println!("reply: {}", report.reply_path);
            if let Some(draft_path) = &report.draft_path {
                println!("draft: {draft_path}");
            }
            println!("commands: {}", report.command_path);
            println!("apply: {}", report.apply_command);
            for next in &report.next {
                println!("next: {next}");
            }
        }
        Language::Zh => {
            println!("章鱼主脑会话");
            println!("模式: {}", report.mode);
            println!("主脑: {}", report.policy);
            println!("会话: {}", report.session_dir);
            println!("提示词: {}", report.prompt_path);
            println!("消息: {}", report.messages_path);
            println!("回复: {}", report.reply_path);
            if let Some(draft_path) = &report.draft_path {
                println!("草稿: {draft_path}");
            }
            println!("命令: {}", report.command_path);
            println!("应用: {}", report.apply_command);
            for next in &report.next {
                println!("下一步: {next}");
            }
        }
    }
}

fn print_context_turns(report: &ContextReport, language: Language) {
    match language {
        Language::En => println!("recent Need/Feed:"),
        Language::Zh => println!("最近Need/Feed:"),
    }
    if report.brain.turns.is_empty() {
        match language {
            Language::En => println!("- none"),
            Language::Zh => println!("- 暂无"),
        }
        return;
    }
    for turn in &report.brain.turns {
        println!(
            "- {}:{} -> {:?} :: {}",
            need_label(&turn.need.kind),
            turn.need.query,
            turn.feed.status,
            turn.feed.summary
        );
    }
}

fn print_context_tentacles(report: &ContextReport, language: Language) {
    match language {
        Language::En => println!("tentacles:"),
        Language::Zh => println!("触手:"),
    }
    if report.tentacles.is_empty() {
        match language {
            Language::En => println!("- none installed"),
            Language::Zh => println!("- 暂未安装"),
        }
        return;
    }
    for tentacle in &report.tentacles {
        let tools = tentacle
            .tools
            .iter()
            .map(|tool| tool.id.clone())
            .collect::<Vec<_>>();
        println!(
            "- {}: {} | tools={}",
            tentacle.id,
            tentacle.policy,
            join_or_none(&tools)
        );
        if let Some(action) = tentacle.recent_actions.last() {
            println!(
                "  latest: #{} {} -> {:?} {}",
                action.index,
                action.tool.as_deref().unwrap_or("no tool"),
                action.status,
                action.summary
            );
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

fn print_route_report(report: &RouteReport, language: Language) {
    match language {
        Language::En => {
            println!(
                "Route report: {} {}",
                need_label(&report.need.kind),
                report.need.query
            );
            if let Some(selected) = &report.selected {
                println!(
                    "selected: {} score={:.2} reason={}",
                    selected.tentacle, selected.score, selected.reason
                );
            } else {
                println!("selected: none");
            }
            for option in &report.candidates {
                let marker = if option.selected { "*" } else { "-" };
                println!(
                    "{marker} {} score={:.2} supported={} traces={} :: {}",
                    option.tentacle,
                    option.score,
                    option.supported,
                    option.trace_count,
                    option.reason
                );
                if let Some(trace) = &option.last_trace {
                    println!("  last_trace: {}", trace_line(trace));
                }
            }
            for next in &report.next {
                println!("next: {next}");
            }
        }
        Language::Zh => {
            println!(
                "路由报告: {} {}",
                need_label(&report.need.kind),
                report.need.query
            );
            if let Some(selected) = &report.selected {
                println!(
                    "已选择: {} 分数={:.2} 原因={}",
                    selected.tentacle, selected.score, selected.reason
                );
            } else {
                println!("已选择: 无");
            }
            for option in &report.candidates {
                let marker = if option.selected { "*" } else { "-" };
                println!(
                    "{marker} {} 分数={:.2} 支持={} 轨迹={} :: {}",
                    option.tentacle,
                    option.score,
                    option.supported,
                    option.trace_count,
                    option.reason
                );
                if let Some(trace) = &option.last_trace {
                    println!("  最近轨迹: {}", trace_line(trace));
                }
            }
            for next in &report.next {
                println!("下一步: {next}");
            }
        }
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

fn need_queue_line(item: &NeedQueueItem) -> String {
    format!(
        "#{} {:?} {}:{}",
        item.index,
        item.status,
        need_label(&item.need.kind),
        item.need.query
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
        next.push("octopus provider save openai".to_string());
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
            println!("feed traces: {}", recommendation.feed_trace_count);
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
            println!("Feed轨迹: {}", recommendation.feed_trace_count);
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

fn print_bootstrap_report(report: &BootstrapReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus bootstrap");
            println!(
                "state: {} ({})",
                report.state_path,
                if report.state_existed {
                    "existing"
                } else {
                    "new"
                }
            );
            println!("seeds: {}", join_or_none(&report.seed_tentacles));
            println!("installed: {}", join_or_none(&report.installed_tentacles));
            println!("skipped: {}", join_or_none(&report.skipped_tentacles));
            println!(
                "adapted: {}",
                join_or_none(&report.adapt.installed_tentacles)
            );
            println!("capabilities: {}", report.report.capabilities.len());
            println!("gaps: {}", report.report.gaps.len());
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼启动");
            println!(
                "状态: {} ({})",
                report.state_path,
                if report.state_existed {
                    "已有"
                } else {
                    "新建"
                }
            );
            println!("种子触手: {}", join_or_none(&report.seed_tentacles));
            println!("已安装: {}", join_or_none(&report.installed_tentacles));
            println!("跳过: {}", join_or_none(&report.skipped_tentacles));
            println!(
                "已适配: {}",
                join_or_none(&report.adapt.installed_tentacles)
            );
            println!("能力数: {}", report.report.capabilities.len());
            println!("缺口数: {}", report.report.gaps.len());
            println!("下一步: {}", join_or_none(&report.next));
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

fn clean_brain_llm_client() -> Result<OpenAiCompatibleChatClient, String> {
    Ok(OpenAiCompatibleChatClient::new(
        OpenAiCompatibleConfig::from_env_prefix(&clean_brain_llm_prefix())?,
    ))
}

fn manifest_llm_config() -> Result<OpenAiCompatibleConfig, String> {
    OpenAiCompatibleConfig::from_env_prefix(&manifest_llm_prefix())
}

fn doctor_llm_prefix() -> String {
    if brain_llm_enabled() {
        brain_llm_prefix()
    } else if chat_llm_enabled() {
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

fn brain_llm_prefix() -> String {
    env::var("OCTOPUS_BRAIN_LLM_PREFIX").unwrap_or_else(|_| "OCTOPUS_LLM".to_string())
}

fn clean_brain_llm_prefix() -> String {
    if brain_llm_enabled() {
        brain_llm_prefix()
    } else if chat_llm_enabled() {
        chat_llm_prefix()
    } else {
        brain_llm_prefix()
    }
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

fn brain_llm_enabled() -> bool {
    env::var("OCTOPUS_BRAIN_LLM")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
}

fn clean_brain_llm_enabled() -> bool {
    brain_llm_enabled() || chat_llm_enabled()
}

fn evolve_llm_enabled() -> bool {
    env::var("OCTOPUS_LLM_EVOLVE")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
}

fn read_brain_apply_payload(path: &str) -> Result<String, String> {
    if path == "-" {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .map_err(|error| format!("failed to read stdin: {error}"))?;
        return Ok(text);
    }
    fs::read_to_string(path).map_err(|error| format!("failed to read {path}: {error}"))
}

fn write_brain_session(
    state: &HarnessState,
    state_path: &Path,
    prompt: &str,
    refine_goal: bool,
    live: bool,
) -> Result<BrainSessionReport, String> {
    let prompt_report = state.clean_brain_prompt(prompt.to_string(), 6);
    let (mode, messages, reply_template) = if refine_goal {
        (
            "goal".to_string(),
            brain_goal_session_messages(&prompt_report),
            serde_json::json!({
                "objective": prompt_report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or(prompt),
                "constraints": ["Need only"],
                "summary": "short goal update",
                "needs": [
                    {"kind": "observe", "query": "short cognitive request"}
                ]
            }),
        )
    } else {
        (
            "explore".to_string(),
            prompt_report.messages.clone(),
            serde_json::json!({
                "summary": "short exploration",
                "needs": [
                    {"kind": "verify", "query": "short cognitive request"}
                ]
            }),
        )
    };
    let session_dir = next_brain_session_dir(state_path)?;
    fs::create_dir_all(&session_dir).map_err(|error| error.to_string())?;

    let prompt_path = session_dir.join("PROMPT.md");
    let messages_path = session_dir.join("messages.json");
    let reply_path = session_dir.join("REPLY.json");
    let draft_path = session_dir.join("DRAFT.json");
    let command_path = session_dir.join("COMMANDS.sh");
    let state_arg = shell_arg(state_path.to_string_lossy().as_ref());
    let prompt_arg = shell_arg(prompt);
    let reply_arg = shell_arg(reply_path.to_string_lossy().as_ref());
    let apply_command = if refine_goal {
        format!("octopus --state {state_arg} brain --goal --apply {reply_arg} --save {prompt_arg}")
    } else {
        format!("octopus --state {state_arg} brain --apply {reply_arg} --save {prompt_arg}")
    };

    fs::write(
        &prompt_path,
        brain_session_prompt_markdown(&prompt_report, &messages, &mode, &apply_command),
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        &messages_path,
        serde_json::to_string_pretty(&messages).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        &reply_path,
        serde_json::to_string_pretty(&reply_template).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let draft_path = if live {
        let mut client = clean_brain_llm_client()?;
        let response = client.chat(&messages)?;
        let draft = clean_brain_session_draft_json(&response.content)?;
        fs::write(&draft_path, draft).map_err(|error| error.to_string())?;
        Some(draft_path)
    } else {
        None
    };
    fs::write(
        &command_path,
        format!(
            "#!/usr/bin/env sh\nset -eu\n# Paste the accepted model reply into REPLY.json first.\n{apply_command}\n"
        ),
    )
    .map_err(|error| error.to_string())?;
    let draft_path_string = draft_path
        .as_ref()
        .map(|path| path.to_string_lossy().to_string());
    let mut next = Vec::new();
    if draft_path_string.is_some() {
        next.push("review DRAFT.json, then copy accepted JSON into REPLY.json".to_string());
    } else {
        next.push("paste PROMPT.md or messages.json into a chat model".to_string());
        next.push("replace REPLY.json with the model JSON reply".to_string());
    }
    next.push(apply_command.clone());

    Ok(BrainSessionReport {
        policy: prompt_report.policy,
        mode,
        prompt: prompt.to_string(),
        session_dir: session_dir.to_string_lossy().to_string(),
        prompt_path: prompt_path.to_string_lossy().to_string(),
        messages_path: messages_path.to_string_lossy().to_string(),
        reply_path: reply_path.to_string_lossy().to_string(),
        draft_path: draft_path_string,
        command_path: command_path.to_string_lossy().to_string(),
        apply_command: apply_command.clone(),
        next,
    })
}

fn clean_brain_session_draft_json(content: &str) -> Result<String, String> {
    if let Some(object) = extract_json_object(content) {
        return Ok(format!("{}\n", object.trim()));
    }
    serde_json::to_string_pretty(&serde_json::json!({ "content": content }))
        .map(|json| format!("{json}\n"))
        .map_err(|error| error.to_string())
}

fn write_need_queue_script(
    state: &HarnessState,
    state_path: &Path,
    path: Option<&str>,
) -> Result<NeedQueueScriptReport, String> {
    let queue = state.need_queue_report(1000);
    let script_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| default_need_queue_script_path(state_path));
    if let Some(parent) = script_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let state_arg = shell_arg(state_path.to_string_lossy().as_ref());
    let mut commands = Vec::new();
    let mut lines = vec![
        "#!/usr/bin/env sh".to_string(),
        "set -eu".to_string(),
        String::new(),
        "# Review before running. The clean brain emitted Needs only; this script turns accepted Needs into Feed.".to_string(),
        String::new(),
    ];
    if queue.pending.is_empty() {
        lines.push("# No pending Needs.".to_string());
    }
    for item in &queue.pending {
        let command = format!(
            "octopus --state {state_arg} need {} {}",
            need_label(&item.need.kind),
            shell_arg(&item.need.query)
        );
        let take = format!("octopus --state {state_arg} needs take {}", item.index);
        lines.push(format!("# Need {} from {}", item.index, item.source));
        lines.push(format!("# Summary: {}", item.summary));
        lines.push(command.clone());
        lines.push(take.clone());
        lines.push(String::new());
        commands.push(command);
        commands.push(take);
    }
    fs::write(&script_path, format!("{}\n", lines.join("\n")))
        .map_err(|error| error.to_string())?;
    make_executable(&script_path)?;
    let script = script_path.to_string_lossy().to_string();
    let next = if queue.pending.is_empty() {
        vec!["octopus explore --save \"what should the brain ask next?\"".to_string()]
    } else {
        vec![
            format!("review {}", shell_arg(&script)),
            format!("sh {}", shell_arg(&script)),
            "octopus needs".to_string(),
        ]
    };
    Ok(NeedQueueScriptReport {
        policy: queue.policy,
        script_path: script,
        command_count: commands.len(),
        commands,
        next,
    })
}

fn default_need_queue_script_path(state_path: &Path) -> PathBuf {
    state_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new(".octopus"))
        .join("needs")
        .join("RUN.sh")
}

fn write_need_queue_review_session(
    state: &HarnessState,
    state_path: &Path,
    prompt: &str,
    live: bool,
) -> Result<NeedQueueReviewSessionReport, String> {
    let queue = state.need_queue_report(1000);
    let brain = state.clean_brain_prompt(prompt.to_string(), 6);
    let messages = need_queue_review_messages(&brain, &queue);
    let session_dir = next_need_queue_session_dir(state_path)?;
    fs::create_dir_all(&session_dir).map_err(|error| error.to_string())?;

    let prompt_path = session_dir.join("PROMPT.md");
    let messages_path = session_dir.join("messages.json");
    let review_path = session_dir.join("REVIEW.json");
    let draft_path = session_dir.join("DRAFT.json");
    let command_path = session_dir.join("COMMANDS.sh");
    let state_arg = shell_arg(state_path.to_string_lossy().as_ref());
    let review_arg = shell_arg(review_path.to_string_lossy().as_ref());
    let prompt_arg = shell_arg(prompt);
    let apply_command =
        format!("octopus --state {state_arg} brain --apply {review_arg} --save {prompt_arg}");
    let script_command = format!("octopus --state {state_arg} needs script");
    let review_template = serde_json::json!({
        "summary": "short Need queue review",
        "keep": [1],
        "drop": [],
        "needs": [
            {"kind": "verify", "query": "short cognitive request"}
        ]
    });

    fs::write(
        &prompt_path,
        need_queue_review_prompt_markdown(
            &brain,
            &queue,
            &messages,
            &apply_command,
            &script_command,
        ),
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        &messages_path,
        serde_json::to_string_pretty(&messages).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        &review_path,
        serde_json::to_string_pretty(&review_template).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let draft_path = if live {
        let mut client = clean_brain_llm_client()?;
        let response = client.chat(&messages)?;
        let draft = clean_brain_session_draft_json(&response.content)?;
        fs::write(&draft_path, draft).map_err(|error| error.to_string())?;
        Some(draft_path)
    } else {
        None
    };
    fs::write(
        &command_path,
        format!(
            "#!/usr/bin/env sh\nset -eu\n# Review REVIEW.json before running these commands.\n{apply_command}\n{script_command}\n"
        ),
    )
    .map_err(|error| error.to_string())?;
    make_executable(&command_path)?;
    let draft_path_string = draft_path
        .as_ref()
        .map(|path| path.to_string_lossy().to_string());
    let mut next = Vec::new();
    if draft_path_string.is_some() {
        next.push("review DRAFT.json, then copy accepted JSON into REVIEW.json".to_string());
    } else {
        next.push("paste PROMPT.md or messages.json into a chat model".to_string());
        next.push("replace REVIEW.json with the model JSON reply".to_string());
    }
    next.push(apply_command.clone());
    next.push(script_command.clone());

    Ok(NeedQueueReviewSessionReport {
        policy: queue.policy,
        prompt: prompt.to_string(),
        pending_count: queue.pending.len(),
        session_dir: session_dir.to_string_lossy().to_string(),
        prompt_path: prompt_path.to_string_lossy().to_string(),
        messages_path: messages_path.to_string_lossy().to_string(),
        review_path: review_path.to_string_lossy().to_string(),
        draft_path: draft_path_string,
        command_path: command_path.to_string_lossy().to_string(),
        next,
    })
}

fn need_queue_review_messages(
    brain: &BrainPromptReport,
    queue: &NeedQueueReport,
) -> Vec<ChatMessage> {
    let context = serde_json::json!({
        "policy": brain.policy,
        "slots": ["Goal", "Mem", "Need", "Feed"],
        "goal": brain.goal,
        "mem": brain.mem,
        "recent_need_feed": brain.recent,
        "pending_needs": queue.pending,
    });
    vec![
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain Need queue reviewer. You see only Goal, Mem, Need, Feed, and pending Needs. Keep, drop, or suggest cognitive Needs only. Do not choose tools, APIs, files, commands, routes, tentacles, or implementation. Return only JSON: {\"summary\":\"short queue review\",\"keep\":[1],\"drop\":[2],\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!(
                "Clean brain Need queue review prompt: {}\nClean brain context JSON: {context}",
                brain.prompt
            ),
        ),
    ]
}

fn need_queue_review_prompt_markdown(
    brain: &BrainPromptReport,
    queue: &NeedQueueReport,
    messages: &[ChatMessage],
    apply_command: &str,
    script_command: &str,
) -> String {
    let message_text = messages
        .iter()
        .map(|message| format!("{:?}:\n{}", message.role, message.content))
        .collect::<Vec<_>>()
        .join("\n\n");
    format!(
        "# Octopus Need Queue Review\n\npolicy: {}\nprompt: {}\npending: {}\n\n## Messages\n\n{}\n\n## Reply Schema\n\nPaste accepted model JSON into `REVIEW.json`.\n\n## Apply\n\n```sh\n{}\n{}\n```\n",
        brain.policy,
        brain.prompt,
        queue.pending.len(),
        message_text,
        apply_command,
        script_command
    )
}

fn next_need_queue_session_dir(state_path: &Path) -> Result<PathBuf, String> {
    let root = state_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new(".octopus"))
        .join("needs");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for index in 1..10000 {
        let candidate = root.join(format!("session-{index}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("no available Need queue session slot".to_string())
}

fn make_executable(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).map_err(|error| error.to_string())?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

fn brain_goal_session_messages(report: &BrainPromptReport) -> Vec<ChatMessage> {
    let context = serde_json::json!({
        "policy": report.policy,
        "slots": ["Goal", "Mem", "Need", "Feed"],
        "goal": report.goal,
        "mem": report.mem,
        "recent_need_feed": report.recent,
    });
    vec![
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain goal layer. You see only Goal, Mem, Need, and Feed. Refine the Goal and suggest cognitive Needs only. Do not choose tools, APIs, files, commands, routes, tentacles, or implementation. Return only JSON: {\"objective\":\"short goal\",\"constraints\":[\"short constraint\"],\"summary\":\"short goal update\",\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!(
                "Clean brain context JSON: {context}\nUser goal prompt: {}",
                report.prompt
            ),
        ),
    ]
}

fn brain_session_prompt_markdown(
    report: &BrainPromptReport,
    messages: &[ChatMessage],
    mode: &str,
    apply_command: &str,
) -> String {
    let message_text = messages
        .iter()
        .map(|message| format!("{:?}:\n{}", message.role, message.content))
        .collect::<Vec<_>>()
        .join("\n\n");
    format!(
        "# Octopus Brain Session\n\nmode: {mode}\npolicy: {}\nprompt: {}\n\n## Messages\n\n{}\n\n## Reply Schema\n\nPaste model JSON into `REPLY.json`.\n\n## Apply\n\n```sh\n{}\n```\n",
        report.policy, report.prompt, message_text, apply_command
    )
}

fn next_brain_session_dir(state_path: &Path) -> Result<PathBuf, String> {
    let root = state_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new(".octopus"))
        .join("brain");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for index in 1..10000 {
        let candidate = root.join(format!("session-{index}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("no available brain session slot".to_string())
}

fn parse_brain_reply<T>(payload: &str, label: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str::<T>(payload.trim())
        .or_else(|_| {
            extract_json_object(payload)
                .ok_or_else(|| format!("invalid {label} JSON: no JSON object found"))
                .and_then(|json| serde_json::from_str::<T>(json).map_err(|error| error.to_string()))
        })
        .map_err(|error| format!("invalid {label} JSON: {error}"))
}

fn extract_json_object(payload: &str) -> Option<&str> {
    let start = payload.find('{')?;
    let end = payload.rfind('}')?;
    (start <= end).then_some(&payload[start..=end])
}

fn usage() -> String {
    "usage: octopus [--version] [--state path] [--lang en|zh] [--json] init [tentacles-root] | bootstrap [tentacles-root] | need <kind> <query> | feedback <trace-index> <status> [summary] | repair [query] | repair score <trace-index> <status> [summary] | think <tentacle> <kind> <query> | context [kind query] | chat <message> | brain [--goal] [--live] [--save] [--session] [--apply path|-] [--apply-json json] [prompt] | explore [--save] [prompt] | needs [take|drop|script [path]|session [--live] [prompt]] | llm <message> | providers | provider <profile> [prefix] | provider save <profile> [prefix] [path] | provider status | provider check [prefix] [message] | bridge [addr] | demo [repo] | goal [set objective] | status | report | preflight [--live] | preflight script [path] | preflight record [path] | doctor | pet [state] | beat [memory_keep] | oauth <provider> <scope> [permissions...] | oauth revoke <grant> | self-iterate <repo> | self-iterate pr <repo> [objective] | evolve <tentacle> <objective> | evolve recommend <tentacle> [objective] | evolve apply <tentacle> <candidate> [objective] | evolve score <tentacle> <candidate> <status> [summary] | scaffold <tentacle> [runtime] | probe <tentacle> <kind> <query> | traces [limit] | routes [kind query] | catalog | starter [objective] | skills [root] | manifests [root] | env | adapt [root] | install <profile> | check <tentacle> [index] | installed".to_string()
}

fn parse_trace_index(value: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .ok()
        .filter(|index| *index > 0)
        .ok_or_else(|| format!("invalid feed trace index: {value}"))
}

fn parse_queue_index(value: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .ok()
        .filter(|index| *index > 0)
        .ok_or_else(|| format!("invalid Need queue index: {value}"))
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
        install_report, is_broken_pipe_panic, localize_summary, parse_bridge_env_overlay,
        percent_encode_path, pet_report, pet_report_for_state, preflight_report, product_report,
        provider_status_report, real_machine_record_status_from_parts, repair_report, run,
        skill_reports, starter_report, usage, Language,
    };
    use octopus_core::{
        default_tentacle_profiles, load_tentacle_manifests, CheckHistoryInput, Feed, Goal,
        GoalStatus, HarnessState, Need, NeedKind, NeedQueueStatus, Status,
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

    #[test]
    fn real_machine_record_accepts_current_or_docs_only_parent_record() {
        let current = real_machine_record_status_from_parts(
            Some("abc1234"),
            Some("def5678"),
            Some(&["crates/octopus-core/src/main.rs".to_string()]),
            "tested abc1234",
        );
        assert!(current.passed);
        assert!(current.evidence.contains("head abc1234 recorded=true"));

        let docs_only = real_machine_record_status_from_parts(
            Some("abc1234"),
            Some("def5678"),
            Some(&[
                "docs/real-machine-test.md".to_string(),
                "docs/product-gap.md".to_string(),
                "docs/version-plan.md".to_string(),
            ]),
            "tested def5678",
        );
        assert!(docs_only.passed);
        assert!(docs_only.evidence.contains("parent def5678 recorded=true"));

        let code_change = real_machine_record_status_from_parts(
            Some("abc1234"),
            Some("def5678"),
            Some(&[
                "docs/real-machine-test.md".to_string(),
                "crates/octopus-core/src/main.rs".to_string(),
            ]),
            "tested def5678",
        );
        assert!(!code_change.passed);
        assert!(code_change.evidence.contains("docs_only=false"));
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
    fn cli_goal_set_updates_goal_without_feed() {
        let _env = env_guard();
        let path = std::env::temp_dir().join(format!(
            "octopus-goal-set-state-{}.json",
            std::process::id()
        ));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "goal".to_string(),
            "set".to_string(),
            "keep".to_string(),
            "the".to_string(),
            "brain".to_string(),
            "clean".to_string(),
        ])
        .unwrap();

        let restored = HarnessState::load(&path).unwrap();
        assert_eq!(
            restored.goal.as_ref().unwrap().objective,
            "keep the brain clean"
        );
        assert!(restored.feed_traces.is_empty());
        assert!(restored.routes.scores.is_empty());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_explore_keeps_state_read_only() {
        let _env = env_guard();
        let path =
            std::env::temp_dir().join(format!("octopus-explore-state-{}.json", std::process::id()));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "goal".to_string(),
            "set".to_string(),
            "clean".to_string(),
            "brain".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "explore".to_string(),
            "next".to_string(),
            "need".to_string(),
        ])
        .unwrap();

        let restored = HarnessState::load(&path).unwrap();
        assert_eq!(restored.goal.as_ref().unwrap().objective, "clean brain");
        assert!(restored.feed_traces.is_empty());
        assert!(restored.routes.scores.is_empty());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_brain_prompt_keeps_state_read_only() {
        let _env = env_guard();
        let path =
            std::env::temp_dir().join(format!("octopus-brain-state-{}.json", std::process::id()));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "goal".to_string(),
            "set".to_string(),
            "clean".to_string(),
            "brain".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "brain".to_string(),
            "next".to_string(),
            "need".to_string(),
        ])
        .unwrap();

        let restored = HarnessState::load(&path).unwrap();
        assert_eq!(restored.goal.as_ref().unwrap().objective, "clean brain");
        assert!(restored.feed_traces.is_empty());
        assert!(restored.routes.scores.is_empty());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_brain_apply_json_queues_external_chat_needs_without_feed() {
        let _env = env_guard();
        let path = std::env::temp_dir().join(format!(
            "octopus-brain-apply-state-{}.json",
            std::process::id()
        ));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "brain".to_string(),
            "--apply-json".to_string(),
            "{\"summary\":\"external chat explored\",\"needs\":[{\"kind\":\"verify\",\"query\":\"goal evidence stays clean\"}]}".to_string(),
            "--save".to_string(),
            "what".to_string(),
            "next".to_string(),
        ])
        .unwrap();

        let restored = HarnessState::load(&path).unwrap();
        assert_eq!(restored.pending_need_queue_count(), 1);
        assert!(restored.goal.is_none());
        assert!(restored.feed_traces.is_empty());
        assert!(restored.routes.scores.is_empty());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("external chat explored"));
        assert!(content.contains("goal evidence stays clean"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_brain_apply_json_saves_only_clean_needs() {
        let _env = env_guard();
        let path = std::env::temp_dir().join(format!(
            "octopus-brain-clean-apply-state-{}.json",
            std::process::id()
        ));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "brain".to_string(),
            "--apply-json".to_string(),
            "{\"summary\":\"mixed external chat\",\"needs\":[{\"kind\":\"verify\",\"query\":\"goal evidence stays clean\"},{\"kind\":\"execute\",\"query\":\"cargo test -p octopus-core\"}]}".to_string(),
            "--save".to_string(),
            "what".to_string(),
            "next".to_string(),
        ])
        .unwrap();

        let restored = HarnessState::load(&path).unwrap();
        assert_eq!(restored.pending_need_queue_count(), 1);
        assert!(restored.feed_traces.is_empty());
        assert!(restored.routes.scores.is_empty());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("goal evidence stays clean"));
        assert!(!content.contains("cargo test"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_brain_apply_goal_file_refines_goal_without_feed() {
        let _env = env_guard();
        let dir =
            std::env::temp_dir().join(format!("octopus-brain-apply-goal-{}", std::process::id()));
        let state_path = dir.join("state.json");
        let reply_path = dir.join("reply.md");
        let state = state_path.to_string_lossy().to_string();
        let reply = reply_path.to_string_lossy().to_string();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            &reply_path,
            "```json\n{\"objective\":\"keep the LLM brain clean\",\"constraints\":[\"Need only\"],\"summary\":\"external goal refined\",\"needs\":[{\"kind\":\"observe\",\"query\":\"goal history\"}]}\n```",
        )
        .unwrap();

        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "brain".to_string(),
            "--goal".to_string(),
            "--apply".to_string(),
            reply,
            "--save".to_string(),
            "tighten".to_string(),
            "goal".to_string(),
        ])
        .unwrap();

        let restored = HarnessState::load(&state_path).unwrap();
        assert_eq!(
            restored.goal.as_ref().unwrap().objective,
            "keep the LLM brain clean"
        );
        assert_eq!(
            restored.goal.as_ref().unwrap().constraints,
            vec!["Need only".to_string()]
        );
        assert_eq!(restored.goal_turns.len(), 1);
        assert_eq!(restored.pending_need_queue_count(), 1);
        assert!(restored.feed_traces.is_empty());
        assert!(restored.routes.scores.is_empty());
        let content = fs::read_to_string(&state_path).unwrap();
        assert!(content.contains("external goal refined"));
        assert!(content.contains("goal history"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cli_brain_session_writes_external_chat_artifacts_without_feed() {
        let _env = env_guard();
        let dir =
            std::env::temp_dir().join(format!("octopus-brain-session-{}", std::process::id()));
        let state_path = dir.join("state.json");
        let state = state_path.to_string_lossy().to_string();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        run(vec![
            "--state".to_string(),
            state.clone(),
            "goal".to_string(),
            "set".to_string(),
            "clean".to_string(),
            "brain".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "brain".to_string(),
            "--session".to_string(),
            "--goal".to_string(),
            "tighten".to_string(),
            "goal".to_string(),
        ])
        .unwrap();

        let restored = HarnessState::load(&state_path).unwrap();
        assert_eq!(restored.goal.as_ref().unwrap().objective, "clean brain");
        assert!(restored.feed_traces.is_empty());
        assert!(restored.routes.scores.is_empty());
        let session = dir.join("brain").join("session-1");
        assert!(session.join("PROMPT.md").exists());
        assert!(session.join("messages.json").exists());
        assert!(session.join("REPLY.json").exists());
        assert!(session.join("COMMANDS.sh").exists());
        let command = fs::read_to_string(session.join("COMMANDS.sh")).unwrap();
        assert!(command.contains("brain --goal --apply"));
        let reply = fs::read_to_string(session.join("REPLY.json")).unwrap();
        assert!(reply.contains("\"objective\""));
        assert!(reply.contains("\"needs\""));
        let messages = fs::read_to_string(session.join("messages.json")).unwrap();
        assert!(messages.contains("Goal + Mem + Need + Feed"));
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn cli_brain_live_session_writes_draft_without_feed() {
        use std::os::unix::fs::PermissionsExt;

        let _env = env_guard();
        let dir =
            std::env::temp_dir().join(format!("octopus-brain-live-session-{}", std::process::id()));
        let state_path = dir.join("state.json");
        let state = state_path.to_string_lossy().to_string();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let curl = dir.join("fake-curl.sh");
        fs::write(
            &curl,
            r#"#!/bin/sh
printf '%s' '{"choices":[{"message":{"content":"{\"summary\":\"session draft explored\",\"needs\":[{\"kind\":\"verify\",\"query\":\"draft stays outside Feed\"}]}"}}]}'
"#,
        )
        .unwrap();
        let mut permissions = fs::metadata(&curl).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&curl, permissions).unwrap();

        let old_brain = std::env::var("OCTOPUS_BRAIN_LLM").ok();
        let old_prefix = std::env::var("OCTOPUS_BRAIN_LLM_PREFIX").ok();
        let old_model = std::env::var("OCTOPUS_BRAIN_SESSION_MODEL").ok();
        let old_base_url = std::env::var("OCTOPUS_BRAIN_SESSION_BASE_URL").ok();
        let old_api_key = std::env::var("OCTOPUS_BRAIN_SESSION_API_KEY").ok();
        let old_curl = std::env::var("OCTOPUS_BRAIN_SESSION_CURL").ok();
        std::env::set_var("OCTOPUS_BRAIN_LLM", "1");
        std::env::set_var("OCTOPUS_BRAIN_LLM_PREFIX", "OCTOPUS_BRAIN_SESSION");
        std::env::set_var("OCTOPUS_BRAIN_SESSION_MODEL", "test-model");
        std::env::set_var("OCTOPUS_BRAIN_SESSION_BASE_URL", "https://llm.example/v1");
        std::env::remove_var("OCTOPUS_BRAIN_SESSION_API_KEY");
        std::env::set_var(
            "OCTOPUS_BRAIN_SESSION_CURL",
            curl.to_string_lossy().to_string(),
        );

        run(vec![
            "--state".to_string(),
            state.clone(),
            "goal".to_string(),
            "set".to_string(),
            "clean".to_string(),
            "brain".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "brain".to_string(),
            "--session".to_string(),
            "--live".to_string(),
            "what".to_string(),
            "next".to_string(),
        ])
        .unwrap();

        restore_env("OCTOPUS_BRAIN_LLM", old_brain);
        restore_env("OCTOPUS_BRAIN_LLM_PREFIX", old_prefix);
        restore_env("OCTOPUS_BRAIN_SESSION_MODEL", old_model);
        restore_env("OCTOPUS_BRAIN_SESSION_BASE_URL", old_base_url);
        restore_env("OCTOPUS_BRAIN_SESSION_API_KEY", old_api_key);
        restore_env("OCTOPUS_BRAIN_SESSION_CURL", old_curl);

        let restored = HarnessState::load(&state_path).unwrap();
        assert_eq!(restored.goal.as_ref().unwrap().objective, "clean brain");
        assert_eq!(restored.pending_need_queue_count(), 0);
        assert!(restored.feed_traces.is_empty());
        assert!(restored.routes.scores.is_empty());
        let session = dir.join("brain").join("session-1");
        let draft = fs::read_to_string(session.join("DRAFT.json")).unwrap();
        assert!(draft.contains("session draft explored"));
        assert!(draft.contains("draft stays outside Feed"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cli_explore_save_and_needs_take_keep_feed_read_only() {
        let _env = env_guard();
        let path = std::env::temp_dir().join(format!(
            "octopus-need-queue-state-{}.json",
            std::process::id()
        ));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "goal".to_string(),
            "set".to_string(),
            "clean".to_string(),
            "brain".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "explore".to_string(),
            "--save".to_string(),
            "next".to_string(),
            "need".to_string(),
        ])
        .unwrap();

        let queued = HarnessState::load(&path).unwrap();
        assert!(queued.pending_need_queue_count() > 0);
        assert!(queued.feed_traces.is_empty());
        let index = queued
            .need_queue
            .iter()
            .find(|item| item.status == NeedQueueStatus::Pending)
            .unwrap()
            .index;

        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "needs".to_string(),
            "take".to_string(),
            index.to_string(),
        ])
        .unwrap();

        let restored = HarnessState::load(&path).unwrap();
        assert!(restored.pending_need_queue_count() < queued.pending_need_queue_count());
        assert!(restored.feed_traces.is_empty());
        assert!(restored.routes.scores.is_empty());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_needs_script_writes_reviewable_feed_script_without_execution() {
        let _env = env_guard();
        let dir = std::env::temp_dir().join(format!("octopus-needs-script-{}", std::process::id()));
        let state_path = dir.join("state.json");
        let script_path = dir.join("run-needs.sh");
        let state = state_path.to_string_lossy().to_string();
        let script = script_path.to_string_lossy().to_string();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        run(vec![
            "--state".to_string(),
            state.clone(),
            "goal".to_string(),
            "set".to_string(),
            "clean".to_string(),
            "brain".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "explore".to_string(),
            "--save".to_string(),
            "next".to_string(),
            "need".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "needs".to_string(),
            "script".to_string(),
            script,
        ])
        .unwrap();

        let restored = HarnessState::load(&state_path).unwrap();
        assert!(restored.pending_need_queue_count() > 0);
        assert!(restored.feed_traces.is_empty());
        assert!(restored.routes.scores.is_empty());
        let content = fs::read_to_string(&script_path).unwrap();
        assert!(content.contains("Review before running"));
        assert!(content.contains("octopus --state"));
        assert!(content.contains(" need "));
        assert!(content.contains(" needs take "));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cli_needs_session_writes_review_artifacts_without_execution() {
        let _env = env_guard();
        let dir =
            std::env::temp_dir().join(format!("octopus-needs-session-{}", std::process::id()));
        let state_path = dir.join("state.json");
        let state = state_path.to_string_lossy().to_string();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        run(vec![
            "--state".to_string(),
            state.clone(),
            "goal".to_string(),
            "set".to_string(),
            "clean".to_string(),
            "brain".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "explore".to_string(),
            "--save".to_string(),
            "next".to_string(),
            "need".to_string(),
        ])
        .unwrap();
        let queued = HarnessState::load(&state_path).unwrap();
        let pending = queued.pending_need_queue_count();

        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "needs".to_string(),
            "session".to_string(),
            "review".to_string(),
            "queue".to_string(),
        ])
        .unwrap();

        let restored = HarnessState::load(&state_path).unwrap();
        assert_eq!(restored.pending_need_queue_count(), pending);
        assert!(restored.feed_traces.is_empty());
        assert!(restored.routes.scores.is_empty());
        let session = dir.join("needs").join("session-1");
        assert!(session.join("PROMPT.md").exists());
        assert!(session.join("messages.json").exists());
        assert!(session.join("REVIEW.json").exists());
        assert!(session.join("COMMANDS.sh").exists());
        let prompt = fs::read_to_string(session.join("PROMPT.md")).unwrap();
        assert!(prompt.contains("pending:"));
        let messages = fs::read_to_string(session.join("messages.json")).unwrap();
        assert!(messages.contains("pending_needs"));
        let review = fs::read_to_string(session.join("REVIEW.json")).unwrap();
        assert!(review.contains("\"keep\""));
        assert!(review.contains("\"needs\""));
        let _ = fs::remove_dir_all(dir);
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
    fn starter_report_returns_gui_ready_first_need_without_feed() {
        let state = HarnessState::default();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let state_path = Path::new(".octopus/state.json");

        let report =
            starter_report(&state, state_path, root, Some("fix repo tests".to_string())).unwrap();

        assert!(report.groups.iter().any(|group| group.id == "repo"));
        assert!(report.groups.iter().any(|group| group.count > 0));
        let swe = report
            .recommendations
            .iter()
            .find(|item| item.id == "swe-agent")
            .unwrap();
        assert_eq!(swe.group, "repo");
        assert_eq!(swe.group_label, "Repo");
        assert!(swe.group_reason.contains("inspect"));
        assert!(swe.signals.iter().any(|signal| signal.contains("tools:")));
        assert!(swe
            .signals
            .iter()
            .any(|signal| signal.contains("first Need: execute")));
        assert_eq!(swe.first_need_kind, "execute");
        assert_eq!(swe.first_need_query, "fix repo tests");
        assert!(swe.first_need_command.contains(" need execute "));
        assert!(state.feed_traces.is_empty());
        assert!(state.routes.scores.is_empty());
    }

    #[test]
    fn cli_catalog_and_env_commands_run() {
        let _env = env_guard();
        run(vec!["catalog".to_string()]).unwrap();
        run(vec![
            "starter".to_string(),
            "fix".to_string(),
            "repo".to_string(),
            "tests".to_string(),
        ])
        .unwrap();
        run(vec![
            "--json".to_string(),
            "starter".to_string(),
            "use".to_string(),
            "desktop".to_string(),
        ])
        .unwrap();
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
            "--json".to_string(),
            "context".to_string(),
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
        assert!(usage().contains(
            "brain [--goal] [--live] [--save] [--session] [--apply path|-] [--apply-json json] [prompt]"
        ));
        assert!(usage().contains("explore [--save] [prompt]"));
        assert!(usage().contains("needs [take|drop|script [path]|session [--live] [prompt]]"));
        assert!(usage().contains("repair [query]"));
        assert!(usage().contains("repair score <trace-index>"));
        assert!(usage().contains("context [kind query]"));
        assert!(usage().contains("provider save <profile>"));
        assert!(usage().contains("provider status"));
        assert!(usage().contains("starter [objective]"));
        assert!(usage().contains("report"));
        assert!(usage().contains("preflight [--live]"));
        assert!(usage().contains("preflight script [path]"));
        assert!(usage().contains("preflight record [path]"));
        assert!(usage().contains("traces [limit]"));
        assert!(usage().contains("feedback <trace-index>"));
        assert!(usage().contains("bootstrap [tentacles-root]"));
    }

    #[test]
    fn cli_bootstrap_installs_seed_tentacles() {
        let _env = env_guard();
        let _cwd = CwdGuard::new();
        let dir = std::env::temp_dir().join(format!("octopus-bootstrap-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let state = dir
            .join(".octopus/state.json")
            .to_string_lossy()
            .to_string();

        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "bootstrap".to_string(),
        ])
        .unwrap();

        let restored = HarnessState::load(&state).unwrap();
        let installed = restored
            .installed_tentacles
            .iter()
            .map(|tentacle| tentacle.id.as_str())
            .collect::<Vec<_>>();
        assert!(installed.contains(&"swe-agent"));
        assert!(installed.contains(&"json-feed"));
        assert!(installed.contains(&"computer-use-agent"));
        assert!(installed.contains(&"harness-repair-agent"));
        assert!(dir.join(".octopus/llm.env.example").exists());
        assert!(restored.last_pet_event.is_some());
        std::env::set_current_dir(&_cwd.original).unwrap();
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cli_repair_queues_next_need_from_harness_feed() {
        let _env = env_guard();
        let _cwd = CwdGuard::new();
        let dir = std::env::temp_dir().join(format!("octopus-repair-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let state = dir
            .join(".octopus/state.json")
            .to_string_lossy()
            .to_string();

        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "repair".to_string(),
            ".".to_string(),
        ])
        .unwrap();

        let restored = HarnessState::load(&state).unwrap();
        assert!(restored
            .installed_tentacles
            .iter()
            .any(|tentacle| tentacle.id == "harness-repair-agent"));
        assert_eq!(restored.feed_traces.len(), 1);
        assert_eq!(
            restored.feed_traces[0].tentacle.as_deref(),
            Some("harness-repair-agent")
        );
        assert_eq!(restored.pending_need_queue_count(), 1);
        let queued = restored
            .need_queue
            .iter()
            .find(|item| item.status == NeedQueueStatus::Pending)
            .unwrap();
        assert_eq!(queued.source, "harness-repair-agent");
        assert_eq!(queued.need.kind, NeedKind::Verify);
        assert!(queued.need.query.contains("Feed trace"));
        std::env::set_current_dir(&_cwd.original).unwrap();
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cli_repair_report_exposes_latest_repair_plan() {
        let _env = env_guard();
        let _cwd = CwdGuard::new();
        let dir =
            std::env::temp_dir().join(format!("octopus-repair-plan-report-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let state_path = dir.join(".octopus/state.json");
        let state = state_path.to_string_lossy().to_string();

        run(vec![
            "--state".to_string(),
            state.clone(),
            "install".to_string(),
            "harness-repair-agent".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state,
            "need".to_string(),
            "execute".to_string(),
            dir.to_string_lossy().to_string(),
        ])
        .unwrap();

        let mut restored = HarnessState::load(&state_path).unwrap();
        let report = repair_report(&state_path, &mut restored, ".".to_string()).unwrap();
        assert_eq!(
            report.feed.metadata.get("tool").map(String::as_str),
            Some("heartbeat_repair")
        );
        let plan = report.repair_plan.expect("repair plan report");
        assert!(plan.path.ends_with("REPAIR_PLAN.json"));
        assert!(plan.review.ends_with("REVIEW.md"));
        assert_eq!(plan.status, "review_required");
        assert_eq!(plan.target_tool, "repair_session");
        assert!(plan.grant_command.contains("harness:write"));
        assert!(plan.score_command.contains("repair score 2"));
        assert!(!plan.score_command.contains("<trace-index>"));
        assert!(report
            .next
            .iter()
            .any(|command| command.contains("REVIEW.md")));
        assert!(report
            .next
            .iter()
            .any(|command| command == &plan.grant_command));
        std::env::set_current_dir(&_cwd.original).unwrap();
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cli_provider_save_writes_env_file() {
        let _env = env_guard();
        let _cwd = CwdGuard::new();
        let dir =
            std::env::temp_dir().join(format!("octopus-provider-save-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        std::env::set_current_dir(&dir).unwrap();

        run(vec![
            "--json".to_string(),
            "provider".to_string(),
            "save".to_string(),
            "openai".to_string(),
            "OCTOPUS_TEST".to_string(),
        ])
        .unwrap();

        let content = fs::read_to_string(dir.join(".octopus/llm.env")).unwrap();
        assert!(content.contains("export OCTOPUS_TEST_MODEL='gpt-4.1-mini'"));
        assert!(content.contains("export OCTOPUS_CHAT_LLM_PREFIX=OCTOPUS_TEST"));
        assert!(content.contains("export OCTOPUS_BRAIN_LLM=1"));
        assert!(content.contains("export OCTOPUS_BRAIN_LLM_PREFIX=OCTOPUS_TEST"));
        std::env::set_current_dir(&_cwd.original).unwrap();
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn bridge_provider_env_overlay_reads_octopus_exports() {
        let _env = env_guard();
        let old_key = std::env::var("OPENAI_API_KEY").ok();
        std::env::set_var("OPENAI_API_KEY", "sk-test");

        let overlay = parse_bridge_env_overlay(
            "export OCTOPUS_LLM_MODEL='gpt-4.1-mini'\n\
             export OCTOPUS_LLM_API_KEY=\"${OPENAI_API_KEY:-}\"\n\
             export OCTOPUS_CHAT_LLM=1\n\
             export OCTOPUS_BRAIN_LLM=1\n\
             export PATH='ignored'\n",
        );

        assert!(overlay.contains(&("OCTOPUS_LLM_MODEL".to_string(), "gpt-4.1-mini".to_string())));
        assert!(overlay.contains(&("OCTOPUS_LLM_API_KEY".to_string(), "sk-test".to_string())));
        assert!(overlay.contains(&("OCTOPUS_CHAT_LLM".to_string(), "1".to_string())));
        assert!(overlay.contains(&("OCTOPUS_BRAIN_LLM".to_string(), "1".to_string())));
        assert!(!overlay.iter().any(|(key, _)| key == "PATH"));

        restore_env("OPENAI_API_KEY", old_key);
    }

    #[test]
    fn provider_status_reports_all_llm_layers() {
        let _env = env_guard();
        let old_chat = std::env::var("OCTOPUS_CHAT_LLM").ok();
        let old_brain = std::env::var("OCTOPUS_BRAIN_LLM").ok();
        let old_manifest = std::env::var("OCTOPUS_LLM_MANIFEST").ok();
        let old_evolve = std::env::var("OCTOPUS_LLM_EVOLVE").ok();
        let old_chat_prefix = std::env::var("OCTOPUS_CHAT_LLM_PREFIX").ok();
        let old_brain_prefix = std::env::var("OCTOPUS_BRAIN_LLM_PREFIX").ok();
        let old_manifest_prefix = std::env::var("OCTOPUS_MANIFEST_LLM_PREFIX").ok();
        let old_evolve_prefix = std::env::var("OCTOPUS_EVOLVE_LLM_PREFIX").ok();
        let old_model = std::env::var("OCTOPUS_STATUS_TEST_MODEL").ok();
        let old_base_url = std::env::var("OCTOPUS_STATUS_TEST_BASE_URL").ok();
        let old_api_key = std::env::var("OCTOPUS_STATUS_TEST_API_KEY").ok();
        std::env::set_var("OCTOPUS_CHAT_LLM", "1");
        std::env::set_var("OCTOPUS_BRAIN_LLM", "1");
        std::env::set_var("OCTOPUS_LLM_MANIFEST", "1");
        std::env::set_var("OCTOPUS_LLM_EVOLVE", "1");
        std::env::set_var("OCTOPUS_CHAT_LLM_PREFIX", "OCTOPUS_STATUS_TEST");
        std::env::set_var("OCTOPUS_BRAIN_LLM_PREFIX", "OCTOPUS_STATUS_TEST");
        std::env::set_var("OCTOPUS_MANIFEST_LLM_PREFIX", "OCTOPUS_STATUS_TEST");
        std::env::set_var("OCTOPUS_EVOLVE_LLM_PREFIX", "OCTOPUS_STATUS_TEST");
        std::env::set_var("OCTOPUS_STATUS_TEST_MODEL", "test-model");
        std::env::set_var("OCTOPUS_STATUS_TEST_BASE_URL", "https://llm.example/v1");
        std::env::remove_var("OCTOPUS_STATUS_TEST_API_KEY");

        let report = provider_status_report();

        assert_eq!(report.layers.len(), 4);
        assert!(report
            .layers
            .iter()
            .all(|layer| layer.enabled && layer.configured));
        assert!(report.layers.iter().any(|layer| {
            layer.layer == "tentacle_planning"
                && layer.prefix == "OCTOPUS_STATUS_TEST"
                && layer.model.as_deref() == Some("test-model")
        }));
        assert!(report.layers.iter().any(|layer| {
            layer.layer == "clean_brain" && layer.prefix == "OCTOPUS_STATUS_TEST" && layer.enabled
        }));
        assert!(report
            .next
            .contains(&"octopus provider check OCTOPUS_STATUS_TEST".to_string()));

        restore_env("OCTOPUS_CHAT_LLM", old_chat);
        restore_env("OCTOPUS_BRAIN_LLM", old_brain);
        restore_env("OCTOPUS_LLM_MANIFEST", old_manifest);
        restore_env("OCTOPUS_LLM_EVOLVE", old_evolve);
        restore_env("OCTOPUS_CHAT_LLM_PREFIX", old_chat_prefix);
        restore_env("OCTOPUS_BRAIN_LLM_PREFIX", old_brain_prefix);
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
            "--json".to_string(),
            "bootstrap".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "starter".to_string(),
            "fix".to_string(),
            "repo".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "context".to_string(),
            "observe".to_string(),
            ".".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "brain".to_string(),
            "what".to_string(),
            "next".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "explore".to_string(),
            "what".to_string(),
            "next".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "needs".to_string(),
            "take".to_string(),
            "1".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "repair".to_string(),
            ".".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "repair".to_string(),
            "score".to_string(),
            "1".to_string(),
            "satisfied".to_string(),
            "reviewed from app".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "repair".to_string(),
            "score".to_string(),
            "0".to_string(),
            "satisfied".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "repair".to_string(),
            "score".to_string(),
            "1".to_string(),
            "unsupported".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "report".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "preflight".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "preflight".to_string(),
            "--live".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "preflight".to_string(),
            "script".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "preflight".to_string(),
            "record".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "preflight".to_string(),
            "script".to_string(),
            "/tmp/preflight.sh".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "preflight".to_string(),
            "record".to_string(),
            "/tmp/record.md".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "feedback".to_string(),
            "1".to_string(),
            "partial".to_string(),
            "reviewed from app".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "feedback".to_string(),
            "0".to_string(),
            "partial".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "feedback".to_string(),
            "1".to_string(),
            "unsupported".to_string()
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
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "check".to_string(),
            "harness-repair-agent".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--json".to_string(),
            "provider".to_string(),
            "save".to_string(),
            "openai".to_string(),
            "OCTOPUS_LLM".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--json".to_string(),
            "provider".to_string(),
            "save".to_string(),
            "openai".to_string(),
            "OCTOPUS_LLM".to_string(),
            "/tmp/llm.env".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--json".to_string(),
            "provider".to_string(),
            "save".to_string(),
            "openai".to_string(),
            "octopus-llm".to_string()
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
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "oauth".to_string(),
            "octopus".to_string(),
            "evolve:swe-agent".to_string(),
            "harness:write".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "oauth".to_string(),
            "octopus".to_string(),
            "evolve:harness-repair-agent".to_string(),
            "harness:write".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "evolve".to_string(),
            "score".to_string(),
            "swe-agent".to_string(),
            "03-runtime-code".to_string(),
            "partial".to_string(),
            "reviewed from app".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "evolve".to_string(),
            "apply".to_string(),
            "swe-agent".to_string(),
            "03-runtime-code".to_string()
        ]));
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "evolve".to_string(),
            "apply".to_string(),
            "swe-agent".to_string(),
            "03-runtime-code".to_string(),
            "apply harness beat recommendation".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "evolve".to_string(),
            "apply".to_string(),
            "custom-feed".to_string(),
            "03-runtime-code".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "evolve".to_string(),
            "apply".to_string(),
            "swe-agent".to_string(),
            "../patch".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "evolve".to_string(),
            "score".to_string(),
            "custom-feed".to_string(),
            "03-runtime-code".to_string(),
            "partial".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "evolve".to_string(),
            "score".to_string(),
            "swe-agent".to_string(),
            "../patch".to_string(),
            "partial".to_string()
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
        assert!(bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "--json".to_string(),
            "oauth".to_string(),
            "octopus".to_string(),
            "evolve:swe-agent".to_string(),
            "harness:write".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "oauth".to_string(),
            "octopus".to_string(),
            "evolve:custom-feed".to_string(),
            "harness:write".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "oauth".to_string(),
            "octopus".to_string(),
            "evolve:swe-agent".to_string(),
            "harness:read".to_string()
        ]));
        assert!(!bridge_command_allowed(&[
            "--state".to_string(),
            "state.json".to_string(),
            "oauth".to_string(),
            "octopus".to_string(),
            "evolve:swe-agent".to_string(),
            "harness:write".to_string(),
            "tool:execute".to_string()
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
        let repair = check_report("harness-repair-agent", None).unwrap();
        assert!(repair.passed);
        assert!(repair
            .results
            .iter()
            .any(|result| result.command.contains("repair_session.sh")));
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
        run(vec![
            "--state".to_string(),
            state.clone(),
            "report".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "report".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "preflight".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "preflight".to_string(),
        ])
        .unwrap();
        let script_path = path.with_file_name("preflight.sh");
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "preflight".to_string(),
            "script".to_string(),
            script_path.to_string_lossy().to_string(),
        ])
        .unwrap();
        let record_path = path.with_file_name("real-machine-record.md");
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "preflight".to_string(),
            "record".to_string(),
            record_path.to_string_lossy().to_string(),
        ])
        .unwrap();

        let loaded = HarnessState::load(&path).unwrap();
        let report = product_report(&loaded, Path::new(&state)).unwrap();
        assert!(report
            .capabilities
            .iter()
            .any(|item| item.id == "clean_brain"));
        assert!(report
            .capabilities
            .iter()
            .any(|item| item.id == "harness_self_repair"));
        assert!(report
            .gaps
            .iter()
            .any(|item| item.id == "provider_feedback"));
        let preflight = preflight_report(&loaded, Path::new(&state), false).unwrap();
        assert!(!preflight.release_ready);
        assert!(preflight
            .checks
            .iter()
            .any(|item| item.id == "live_provider" && item.status == "skipped"));
        assert!(preflight
            .next
            .iter()
            .any(|item| item.contains("preflight --live")));
        let script = fs::read_to_string(&script_path).unwrap();
        assert!(script.contains("octopus --state \"$STATE\" bootstrap"));
        assert!(script.contains("OCTOPUS_PREFLIGHT_LIVE"));
        let record = fs::read_to_string(&record_path).unwrap();
        assert!(record.contains("# Real-Machine Record"));
        assert!(record.contains("Package version"));
        assert!(record.contains("preflight --live"));
        assert!(record.contains("self-iterate pr"));

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("swe-agent"));
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(script_path);
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
    fn cli_brain_live_uses_clean_brain_provider_without_feed() {
        use std::os::unix::fs::PermissionsExt;

        let _env = env_guard();
        let dir =
            std::env::temp_dir().join(format!("octopus-cli-brain-llm-{}", std::process::id()));
        let state_path = dir.join("state.json");
        let state = state_path.to_string_lossy().to_string();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let curl = dir.join("fake-curl.sh");
        fs::write(
            &curl,
            r#"#!/bin/sh
printf '%s' '{"choices":[{"message":{"content":"{\"summary\":\"brain live explored\",\"needs\":[{\"kind\":\"verify\",\"query\":\"goal is still clean\"}]}"}}]}'
"#,
        )
        .unwrap();
        let mut permissions = fs::metadata(&curl).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&curl, permissions).unwrap();

        let old_brain = std::env::var("OCTOPUS_BRAIN_LLM").ok();
        let old_prefix = std::env::var("OCTOPUS_BRAIN_LLM_PREFIX").ok();
        let old_model = std::env::var("OCTOPUS_BRAIN_TEST_MODEL").ok();
        let old_base_url = std::env::var("OCTOPUS_BRAIN_TEST_BASE_URL").ok();
        let old_api_key = std::env::var("OCTOPUS_BRAIN_TEST_API_KEY").ok();
        let old_curl = std::env::var("OCTOPUS_BRAIN_TEST_CURL").ok();
        std::env::set_var("OCTOPUS_BRAIN_LLM", "1");
        std::env::set_var("OCTOPUS_BRAIN_LLM_PREFIX", "OCTOPUS_BRAIN_TEST");
        std::env::set_var("OCTOPUS_BRAIN_TEST_MODEL", "test-model");
        std::env::set_var("OCTOPUS_BRAIN_TEST_BASE_URL", "https://llm.example/v1");
        std::env::remove_var("OCTOPUS_BRAIN_TEST_API_KEY");
        std::env::set_var(
            "OCTOPUS_BRAIN_TEST_CURL",
            curl.to_string_lossy().to_string(),
        );

        run(vec![
            "--state".to_string(),
            state.clone(),
            "goal".to_string(),
            "set".to_string(),
            "keep".to_string(),
            "brain".to_string(),
            "clean".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "brain".to_string(),
            "--live".to_string(),
            "--save".to_string(),
            "what".to_string(),
            "next".to_string(),
        ])
        .unwrap();

        restore_env("OCTOPUS_BRAIN_LLM", old_brain);
        restore_env("OCTOPUS_BRAIN_LLM_PREFIX", old_prefix);
        restore_env("OCTOPUS_BRAIN_TEST_MODEL", old_model);
        restore_env("OCTOPUS_BRAIN_TEST_BASE_URL", old_base_url);
        restore_env("OCTOPUS_BRAIN_TEST_API_KEY", old_api_key);
        restore_env("OCTOPUS_BRAIN_TEST_CURL", old_curl);

        let loaded = HarnessState::load(&state_path).unwrap();
        assert_eq!(loaded.pending_need_queue_count(), 1);
        assert!(loaded.feed_traces.is_empty());
        assert!(loaded.routes.scores.is_empty());
        let content = fs::read_to_string(&state_path).unwrap();
        assert!(content.contains("brain live explored"));
        assert!(content.contains("goal is still clean"));
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn cli_brain_goal_uses_clean_brain_provider_without_feed() {
        use std::os::unix::fs::PermissionsExt;

        let _env = env_guard();
        let dir =
            std::env::temp_dir().join(format!("octopus-cli-brain-goal-{}", std::process::id()));
        let state_path = dir.join("state.json");
        let state = state_path.to_string_lossy().to_string();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let curl = dir.join("fake-curl.sh");
        fs::write(
            &curl,
            r#"#!/bin/sh
printf '%s' '{"choices":[{"message":{"content":"{\"objective\":\"build a clean goal brain\",\"constraints\":[\"Need only\"],\"summary\":\"brain goal refined\",\"needs\":[{\"kind\":\"observe\",\"query\":\"current goal evidence\"}]}"}}]}'
"#,
        )
        .unwrap();
        let mut permissions = fs::metadata(&curl).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&curl, permissions).unwrap();

        let old_brain = std::env::var("OCTOPUS_BRAIN_LLM").ok();
        let old_prefix = std::env::var("OCTOPUS_BRAIN_LLM_PREFIX").ok();
        let old_model = std::env::var("OCTOPUS_BRAIN_GOAL_MODEL").ok();
        let old_base_url = std::env::var("OCTOPUS_BRAIN_GOAL_BASE_URL").ok();
        let old_api_key = std::env::var("OCTOPUS_BRAIN_GOAL_API_KEY").ok();
        let old_curl = std::env::var("OCTOPUS_BRAIN_GOAL_CURL").ok();
        std::env::set_var("OCTOPUS_BRAIN_LLM", "1");
        std::env::set_var("OCTOPUS_BRAIN_LLM_PREFIX", "OCTOPUS_BRAIN_GOAL");
        std::env::set_var("OCTOPUS_BRAIN_GOAL_MODEL", "test-model");
        std::env::set_var("OCTOPUS_BRAIN_GOAL_BASE_URL", "https://llm.example/v1");
        std::env::remove_var("OCTOPUS_BRAIN_GOAL_API_KEY");
        std::env::set_var(
            "OCTOPUS_BRAIN_GOAL_CURL",
            curl.to_string_lossy().to_string(),
        );

        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "brain".to_string(),
            "--goal".to_string(),
            "--live".to_string(),
            "--save".to_string(),
            "tighten".to_string(),
            "goal".to_string(),
        ])
        .unwrap();

        restore_env("OCTOPUS_BRAIN_LLM", old_brain);
        restore_env("OCTOPUS_BRAIN_LLM_PREFIX", old_prefix);
        restore_env("OCTOPUS_BRAIN_GOAL_MODEL", old_model);
        restore_env("OCTOPUS_BRAIN_GOAL_BASE_URL", old_base_url);
        restore_env("OCTOPUS_BRAIN_GOAL_API_KEY", old_api_key);
        restore_env("OCTOPUS_BRAIN_GOAL_CURL", old_curl);

        let loaded = HarnessState::load(&state_path).unwrap();
        let goal = loaded.goal.as_ref().unwrap();
        assert_eq!(goal.objective, "build a clean goal brain");
        assert_eq!(goal.constraints, vec!["Need only".to_string()]);
        assert_eq!(loaded.goal_turns.len(), 1);
        assert_eq!(loaded.pending_need_queue_count(), 1);
        assert!(loaded.feed_traces.is_empty());
        assert!(loaded.routes.scores.is_empty());
        let content = fs::read_to_string(&state_path).unwrap();
        assert!(content.contains("brain goal refined"));
        assert!(content.contains("current goal evidence"));
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
    fn cli_feedback_scores_existing_feed_trace() {
        let _env = env_guard();
        let path = std::env::temp_dir().join(format!(
            "octopus-feed-feedback-state-{}.json",
            std::process::id()
        ));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "need".to_string(),
            "remember".to_string(),
            "feedback loop".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "feedback".to_string(),
            "1".to_string(),
            "failed".to_string(),
            "too noisy".to_string(),
        ])
        .unwrap();

        let restored = HarnessState::load(&path).unwrap();
        assert_eq!(restored.feed_traces[0].status, Status::Failed);
        assert_eq!(
            restored.last_pet_event.as_ref().unwrap().source,
            "feed feedback"
        );
        assert_eq!(restored.last_pet_event.as_ref().unwrap().state, "blocked");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_repair_score_records_outcome() {
        let _env = env_guard();
        let _cwd = CwdGuard::new();
        let workspace = std::env::temp_dir().join(format!(
            "octopus-repair-score-workspace-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&workspace);
        fs::create_dir_all(&workspace).unwrap();
        std::env::set_current_dir(&workspace).unwrap();
        let path = workspace.join("state.json");
        let state = path.to_string_lossy().to_string();

        run(vec![
            "--state".to_string(),
            state.clone(),
            "install".to_string(),
            "harness-repair-agent".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "need".to_string(),
            "execute".to_string(),
            workspace.to_string_lossy().to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "--json".to_string(),
            "repair".to_string(),
            "score".to_string(),
            "1".to_string(),
            "satisfied".to_string(),
            "repair improved harness".to_string(),
        ])
        .unwrap();

        let restored = HarnessState::load(&path).unwrap();
        assert_eq!(restored.repair_outcomes.len(), 1);
        assert_eq!(restored.repair_outcomes[0].trace_index, Some(1));
        assert_eq!(
            restored.last_pet_event.as_ref().unwrap().source,
            "repair score"
        );
        assert_eq!(restored.last_pet_event.as_ref().unwrap().state, "harness");
        let outcomes_file = workspace.join(".octopus/harness-repair/outcomes.jsonl");
        assert!(outcomes_file.exists());
        let outcomes = fs::read_to_string(&outcomes_file).unwrap();
        assert!(outcomes.contains("\"outcome_status\":\"satisfied\""));
        assert!(outcomes.contains("repair improved harness"));
        let outcome_markdown = workspace
            .join(".octopus/harness-repair")
            .read_dir()
            .unwrap()
            .find_map(|entry| {
                let entry = entry.unwrap();
                let candidate = entry.path().join("OUTCOME.md");
                candidate.exists().then_some(candidate)
            })
            .expect("repair outcome markdown");
        assert!(fs::read_to_string(outcome_markdown)
            .unwrap()
            .contains("repair improved harness"));
        let _ = fs::remove_dir_all(workspace);
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
