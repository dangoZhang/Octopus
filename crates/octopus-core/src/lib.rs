mod evolution;
mod evolution_artifact;
mod evolution_candidate;
mod evolution_contract;
mod evolution_llm_plan;
mod evolution_patch;
mod evolution_prompt;
mod evolution_recommend;
mod evolution_target;
mod field_pack;
mod tentacle_scaffold;
pub mod user_surface;

pub use evolution_artifact::{
    render_authorized_apply_patch, render_single_patch_draft, render_tentacle_apply_plan,
    render_tentacle_evolution_proposal, render_tentacle_patch_candidates,
    render_tentacle_patch_drafts, write_tentacle_apply_artifacts,
    write_tentacle_evolution_artifacts,
};
pub use evolution_contract::{
    EvolutionApplyArtifact, EvolutionApplyPlan, EvolutionArtifact, EvolutionFileTarget,
    EvolutionPatchCandidate, EvolutionPatchDraft, EvolutionPatchFeedback, EvolutionPolicy,
    EvolutionRecommendation, EvolutionRequirement, EvolutionSurface, HarnessBeatEvolution,
    TentacleEvolutionProposal,
};
use evolution_recommend::tentacle_evolution_context;
pub use evolution_recommend::{
    plan_tentacle_evolution_apply, recommend_tentacle_evolution_apply,
    write_harness_beat_evolution_artifacts_with_client,
};
pub use field_pack::{
    annotate_need_with_field, default_field_pack_aliases, default_field_pack_catalog,
    default_field_pack_ids, default_field_pack_root, embedded_field_pack_catalog,
    field_pack_report, load_field_pack_catalog, select_field_pack, FieldMiniTask, FieldPack,
    FieldPackCatalog, FieldPackIndex, FieldPackReport, FieldPackSelection, FieldPackSummary,
    FieldPermissionBoundary, FieldTaskSchema, FieldVerifier,
};
pub use tentacle_scaffold::{scaffold_tentacle, TentacleScaffold};

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::{Error, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const OCTOPUS_JSON_CONTRACT: &str = "octopus-json-v1";
pub const OCTOPUS_STDIO_ARGV_CONTRACT: &str = "stdio-argv-v1";
pub const OCTOPUS_STATIC_HTML_CONTRACT: &str = "static-html-v1";
pub const OCTOPUS_ADAPTER_CONTRACT: &str = "adapter-v1";
pub const OCTOPUS_NATIVE_HARNESS_CONTRACT: &str = "native-harness-v1";
const OCTOPUS_TOOL_CALL_SCHEMA: &str = "octopus-tool-call-v1";
const FEED_TRACE_QUERY_BYTES: usize = 160;
const FEED_TRACE_SUMMARY_BYTES: usize = 240;
const CHECK_HISTORY_OUTPUT_BYTES: usize = 480;
const STARTER_FEEDBACK_OBJECTIVE_BYTES: usize = 180;
const STARTER_FEEDBACK_SUMMARY_BYTES: usize = 220;
const MAX_TENTACLE_ACTIONS: usize = 2;
const DEFAULT_PROFILE_REGISTRY: &str =
    include_str!("../../../tentacles/profile-registry/default.json");
const OCTOPUS_PROFILE_REGISTRY_ENV: &str = "OCTOPUS_PROFILE_REGISTRY";
const OCTOPUS_STATE_PATH_ENV: &str = "OCTOPUS_STATE_PATH";
const LOCAL_PROFILE_REGISTRY_PATH: &str = ".octopus/profile-registry/default.json";
pub const CLEAN_BRAIN_CONTEXT_POLICY: &str = "Goal + Mem + Need + Feed";
pub const TENTACLE_CONTEXT_POLICY: &str = "Need + Tool + Action + Tool + Action -> Feed";

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NeedKind {
    Verify,
    Reproduce,
    Compare,
    Remember,
    Forget,
    Execute,
    Recall,
    Observe,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Satisfied,
    Partial,
    Failed,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StarterFeedbackStatus {
    Accepted,
    Ignored,
    Failed,
}

impl StarterFeedbackStatus {
    pub fn score(&self) -> f32 {
        match self {
            StarterFeedbackStatus::Accepted => 6.0,
            StarterFeedbackStatus::Ignored => -1.5,
            StarterFeedbackStatus::Failed => -4.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct StarterFeedbackRecord {
    pub index: u64,
    pub tentacle_id: String,
    pub objective: String,
    pub group: Option<String>,
    pub status: StarterFeedbackStatus,
    pub score: f32,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct StarterFeedbackInput {
    pub tentacle_id: String,
    pub objective: String,
    pub group: Option<String>,
    pub status: StarterFeedbackStatus,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Need {
    pub kind: NeedKind,
    pub query: String,
    pub context: BTreeMap<String, String>,
    pub priority: f32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalStatus {
    Active,
    Satisfied,
    Blocked,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Goal {
    pub objective: String,
    pub constraints: Vec<String>,
    pub signals: BTreeMap<String, String>,
    pub status: GoalStatus,
}

impl Goal {
    pub fn new(objective: impl Into<String>) -> Self {
        Self {
            objective: objective.into(),
            constraints: Vec::new(),
            signals: BTreeMap::new(),
            status: GoalStatus::Active,
        }
    }

    pub fn refine(&mut self, note: impl Into<String>) {
        self.constraints.push(note.into());
    }

    pub fn need(&self, kind: NeedKind, query: impl Into<String>) -> Need {
        let mut need = Need::new(kind, query);
        need.context
            .insert("goal".to_string(), self.objective.clone());
        need
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GoalTurn {
    pub index: u64,
    pub message: String,
    pub summary: String,
    pub status: Status,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GoalChat {
    pub goal: Goal,
    pub turn: GoalTurn,
    pub feedback: Feedback,
    #[serde(default)]
    pub queued: Vec<NeedQueueItem>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GoalRefinement {
    pub objective: Option<String>,
    #[serde(default)]
    pub constraints: Vec<String>,
    pub summary: Option<String>,
    #[serde(default)]
    pub needs: Vec<GoalNeedSuggestion>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GoalNeedSuggestion {
    pub kind: NeedKind,
    pub query: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NeedQueueStatus {
    Pending,
    Taken,
    Dropped,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NeedQueueItem {
    pub index: u64,
    pub need: GoalNeedSuggestion,
    #[serde(default)]
    pub context: BTreeMap<String, String>,
    pub source: String,
    pub prompt: String,
    pub summary: String,
    pub status: NeedQueueStatus,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NeedQueueReport {
    pub policy: String,
    pub pending: Vec<NeedQueueItem>,
    pub history: Vec<NeedQueueItem>,
    pub agent_next: Vec<String>,
    pub user_goal_hints: Vec<String>,
    pub next: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NeedQueueSaveReport {
    pub explore: BrainExploreReport,
    pub queued: Vec<NeedQueueItem>,
    pub queue: NeedQueueReport,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainDeliberationSaveReport {
    pub deliberation: BrainDeliberationReport,
    pub queued: Vec<NeedQueueItem>,
    pub queue: NeedQueueReport,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainReflectionSaveReport {
    pub reflection: BrainReflectionReport,
    pub queued: Vec<NeedQueueItem>,
    pub queue: NeedQueueReport,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainSynthesisSaveReport {
    pub synthesis: BrainSynthesisReport,
    pub queued: Vec<NeedQueueItem>,
    pub queue: NeedQueueReport,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NeedQueueTakeReport {
    pub item: NeedQueueItem,
    pub command: String,
    pub next: Vec<String>,
}

impl GoalRefinement {
    fn source(&self) -> String {
        if self.objective.is_some()
            || self.summary.is_some()
            || !self.constraints.is_empty()
            || !self.needs.is_empty()
        {
            "llm".to_string()
        } else {
            "local".to_string()
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantStatus {
    Active,
    Revoked,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CapabilityGrant {
    pub id: String,
    pub provider: String,
    pub scope: String,
    pub permissions: Vec<String>,
    pub status: GrantStatus,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SelfIterationPlan {
    pub repository: String,
    pub authorized: bool,
    pub mode: String,
    pub next_action: String,
    pub grant_id: Option<String>,
    pub steps: Vec<String>,
    pub checks: Vec<String>,
    pub guardrails: Vec<String>,
    pub candidates: Vec<PatchCandidate>,
    pub draft: Option<PullRequestDraft>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PatchCandidate {
    pub id: String,
    pub title: String,
    pub source: String,
    pub reason: String,
    pub branch: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PullRequestDraft {
    pub branch: String,
    pub title: String,
    pub change_summary: String,
    pub body: String,
    pub check_results: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HeartBeat {
    pub name: String,
    pub changed: bool,
    pub summary: String,
    pub data: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HeartbeatReport {
    pub beats: Vec<HeartBeat>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PetEvent {
    #[serde(default)]
    pub timestamp_secs: u64,
    pub state: String,
    pub source: String,
    pub summary: String,
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_class: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EvolutionOutcome {
    pub index: u64,
    pub tentacle_id: String,
    pub candidate_id: String,
    pub status: Status,
    pub score: f32,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RepairOutcome {
    pub index: u64,
    pub trace_index: Option<u64>,
    pub tentacle_id: String,
    pub tool: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_tentacle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate: Option<String>,
    pub status: Status,
    pub score: f32,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FeedTraceRecord {
    pub index: u64,
    pub need_kind: NeedKind,
    pub need_query: String,
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    pub tentacle: Option<String>,
    pub tool: Option<String>,
    pub plan_source: Option<String>,
    pub route: Option<String>,
    pub evidence_count: usize,
    pub summary: String,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FeedFeedbackOutcome {
    pub trace: FeedTraceRecord,
    pub status: Status,
    pub route_key: Option<String>,
    pub route_score: Option<f32>,
    pub summary: String,
    pub pet_event: PetEvent,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FieldVerifierResult {
    pub index: u64,
    pub trace_index: u64,
    pub field: String,
    pub status: Status,
    pub error_category: Option<String>,
    pub artifact: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ParallelEvolutionWorker {
    pub id: String,
    pub field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mini_task: Option<String>,
    pub goal: String,
    #[serde(default)]
    pub updated_at_secs: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queued_need_index: Option<u64>,
    pub source_trace_index: Option<u64>,
    pub verifier_result_index: Option<u64>,
    pub status: Status,
    pub next_action: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ParallelEvolutionRun {
    pub index: u64,
    pub objective: String,
    #[serde(default = "default_parallel_field_pool_size")]
    pub field_pool_size: usize,
    #[serde(default = "default_parallel_field_pool_policy")]
    pub field_pool_policy: String,
    #[serde(default)]
    pub candidate_fields: Vec<String>,
    #[serde(default)]
    pub requested_worker_count: usize,
    pub worker_count: usize,
    #[serde(default = "default_parallel_worker_policy")]
    pub worker_policy: String,
    pub workers: Vec<ParallelEvolutionWorker>,
    pub summary: String,
}

pub fn parallel_field_pool_policy() -> &'static str {
    "peer field slots; workers only open execution capacity"
}

fn default_parallel_field_pool_policy() -> String {
    parallel_field_pool_policy().to_string()
}

fn default_parallel_field_pool_size() -> usize {
    default_field_pack_ids().len()
}

pub fn parallel_worker_policy() -> &'static str {
    "workers are execution slots from the peer field pool; fields stay peer"
}

fn default_parallel_worker_policy() -> String {
    parallel_worker_policy().to_string()
}

pub fn field_harder_layer_objective() -> &'static str {
    "add a harder mini task layer to the peer field packs and editable field-mini-task harness"
}

pub fn field_harder_layer_next_action(state_args: &str) -> String {
    format!(
        "octopus{state_args} evolve recommend field-mini-task {}",
        shell_arg(field_harder_layer_objective())
    )
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FieldTrajectorySummary {
    pub field: String,
    pub mini_task_count: usize,
    pub satisfied_mini_task_count: usize,
    pub next_mini_task: Option<String>,
    pub next_mini_task_goal: Option<String>,
    pub trace_count: usize,
    pub verifier_result_count: usize,
    pub satisfied_verifier_count: usize,
    pub unsatisfied_verifier_count: usize,
    pub latest_trace_index: Option<u64>,
    pub latest_trace_status: Option<Status>,
    pub latest_verifier_result_index: Option<u64>,
    pub latest_verifier_status: Option<Status>,
    pub latest_parallel_run_index: Option<u64>,
    pub latest_mini_task: Option<String>,
    pub latest_error_category: Option<String>,
    pub latest_pass_evidence: Option<String>,
    pub latest_summary: Option<String>,
    pub needs_repair: bool,
    pub ready_for_harder_task: bool,
    pub next_action: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FieldTrajectoryReport {
    pub field_count: usize,
    #[serde(default)]
    pub latest_worker_slot_count: usize,
    pub trace_count: usize,
    pub verifier_result_count: usize,
    pub parallel_evolution_run_count: usize,
    pub all_first_pass_satisfied: bool,
    pub all_pack_tasks_satisfied: bool,
    #[serde(default, alias = "next_field")]
    pub active_slot_field: Option<String>,
    #[serde(default)]
    pub active_slot_reason: String,
    pub fields: Vec<FieldTrajectorySummary>,
    pub next: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FieldPoolStatusReport {
    pub policy: String,
    pub field_count: usize,
    #[serde(default)]
    pub field_slot_count: usize,
    #[serde(default)]
    pub latest_worker_slot_count: usize,
    pub completed_fields: usize,
    pub active_slot_field: Option<String>,
    #[serde(default)]
    pub active_slot_reason: String,
    pub worker_slots: String,
    pub slots: Vec<FieldPoolSlotReport>,
    pub agent_next: Vec<String>,
    pub user_goal_hints: Vec<String>,
    pub next: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FieldPoolSlotReport {
    pub field: String,
    pub completed: bool,
    pub mini_task_count: usize,
    pub satisfied_mini_task_count: usize,
    pub next_mini_task: Option<String>,
    pub latest_worker_id: Option<String>,
    pub latest_mini_task: Option<String>,
    pub latest_worker_status: Option<Status>,
    pub latest_status: Option<Status>,
    pub latest_parallel_run_index: Option<u64>,
    #[serde(default)]
    pub latest_updated_at_secs: u64,
    pub needs_repair: bool,
    pub agent_next_action: String,
    pub user_goal_hint: String,
    pub next_action: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckHistoryInput {
    pub tentacle_id: String,
    pub source_kind: String,
    pub command_index: Option<usize>,
    pub command: String,
    pub cwd: String,
    pub status: Status,
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CheckHistoryRecord {
    pub index: u64,
    pub timestamp_secs: u64,
    pub tentacle_id: String,
    pub source_kind: String,
    pub command_index: Option<usize>,
    pub command: String,
    pub cwd: String,
    pub status: Status,
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AdaptReport {
    pub environment: EnvironmentReport,
    pub installed_profiles: Vec<String>,
    pub installed_tentacles: Vec<String>,
    pub skipped_manifests: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct StatusReport {
    pub hearts: Vec<HeartBeat>,
    pub memory_count: usize,
    pub route_count: usize,
    pub need_queue_count: usize,
    pub feed_trace_count: usize,
    pub field_verifier_result_count: usize,
    pub parallel_evolution_run_count: usize,
    pub check_history_count: usize,
    pub repair_outcome_count: usize,
    pub evolution_outcome_count: usize,
    pub starter_feedback_count: usize,
    pub installed_profiles: Vec<String>,
    pub tentacles: Vec<TentacleStatus>,
    pub goal: Option<GoalSnapshot>,
    pub active_grants: Vec<String>,
    pub last_pet_event: Option<PetEvent>,
    pub latest_feed_trace: Option<FeedTraceRecord>,
    pub latest_field_verifier_result: Option<FieldVerifierResult>,
    pub latest_parallel_evolution_run: Option<ParallelEvolutionRun>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_pool: Option<FieldPoolStatusReport>,
    pub latest_need_queue_item: Option<NeedQueueItem>,
    pub latest_check: Option<CheckHistoryRecord>,
    pub latest_repair_outcome: Option<RepairOutcome>,
    pub latest_evolution_outcome: Option<EvolutionOutcome>,
    pub harness_learning: HarnessLearningSummary,
    pub latest_starter_feedback: Option<StarterFeedbackRecord>,
    pub warnings: Vec<String>,
    pub agent_next_action: String,
    pub user_goal_hint: String,
    pub next_action: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HarnessLearningSummary {
    pub source: String,
    pub repair_outcomes: usize,
    pub evolution_outcomes: usize,
    pub target_tentacle: Option<String>,
    pub candidate: Option<String>,
    pub status: Option<Status>,
    pub score: Option<f32>,
    pub summary: Option<String>,
    pub next_action: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleStatus {
    pub id: String,
    pub name: String,
    pub brain_kind: String,
    pub runtime_kinds: Vec<String>,
    pub needs: Vec<String>,
    pub tool_count: usize,
    pub editable: Vec<String>,
    pub evolution_surfaces: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GoalSnapshot {
    pub objective: String,
    pub refinements: usize,
    pub status: GoalStatus,
    pub turns: Vec<GoalTurn>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ContextReport {
    pub brain: BrainContextReport,
    pub tentacles: Vec<TentacleContextReport>,
    pub hearts: Vec<HeartBeat>,
    pub agent_next: Vec<String>,
    pub user_goal_hints: Vec<String>,
    pub next: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainExploreReport {
    pub policy: String,
    pub source: String,
    pub prompt: String,
    pub goal: Option<Goal>,
    pub mem: Vec<MemoryContextRecord>,
    pub recent: Vec<BrainContextTurn>,
    pub summary: String,
    pub needs: Vec<GoalNeedSuggestion>,
    pub audit: BrainNeedAudit,
    pub next: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainDeliberationReport {
    pub policy: String,
    pub source: String,
    pub prompt: String,
    pub goal: Option<Goal>,
    pub mem: Vec<MemoryContextRecord>,
    pub recent: Vec<BrainContextTurn>,
    pub summary: String,
    pub observations: Vec<String>,
    pub questions: Vec<String>,
    pub options: Vec<String>,
    pub risks: Vec<String>,
    pub needs: Vec<GoalNeedSuggestion>,
    pub audit: BrainNeedAudit,
    pub next: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainReflectionReport {
    pub policy: String,
    pub source: String,
    pub prompt: String,
    pub goal: Option<Goal>,
    pub mem: Vec<MemoryContextRecord>,
    pub recent: Vec<BrainContextTurn>,
    pub summary: String,
    pub goal_state: String,
    pub evidence: Vec<String>,
    pub gaps: Vec<String>,
    pub questions: Vec<String>,
    pub needs: Vec<GoalNeedSuggestion>,
    pub audit: BrainNeedAudit,
    pub next: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainSynthesisReport {
    pub policy: String,
    pub source: String,
    pub prompt: String,
    pub goal: Option<Goal>,
    pub mem: Vec<MemoryContextRecord>,
    pub recent: Vec<BrainContextTurn>,
    pub summary: String,
    pub draft_count: usize,
    pub observations: Vec<String>,
    pub questions: Vec<String>,
    pub options: Vec<String>,
    pub risks: Vec<String>,
    pub needs: Vec<GoalNeedSuggestion>,
    pub audit: BrainNeedAudit,
    pub next: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainPromptReport {
    pub policy: String,
    pub prompt: String,
    pub goal: Option<Goal>,
    pub mem: Vec<MemoryContextRecord>,
    pub recent: Vec<BrainContextTurn>,
    pub messages: Vec<ChatMessage>,
    pub prompt_text: String,
    pub next: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainGoalReport {
    pub policy: String,
    pub source: String,
    pub prompt: String,
    pub previous_goal: Option<Goal>,
    pub goal: Goal,
    pub summary: String,
    pub needs: Vec<GoalNeedSuggestion>,
    pub audit: BrainNeedAudit,
    pub next: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainNeedAudit {
    pub status: Status,
    pub summary: String,
    pub clean_count: usize,
    pub issue_count: usize,
    pub clean_needs: Vec<GoalNeedSuggestion>,
    pub issues: Vec<BrainNeedAuditIssue>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainNeedAuditIssue {
    pub index: usize,
    pub kind: NeedKind,
    pub query: String,
    pub signal: String,
    pub guidance: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainGoalSaveReport {
    pub goal: BrainGoalReport,
    pub queued: Vec<NeedQueueItem>,
    pub queue: NeedQueueReport,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainContextReport {
    pub policy: String,
    pub slots: Vec<String>,
    pub goal: Option<Goal>,
    pub mem: Vec<MemoryContextRecord>,
    pub turns: Vec<BrainContextTurn>,
    pub next_need: Option<Need>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MemoryContextRecord {
    pub id: String,
    pub text: String,
    pub weight: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainContextTurn {
    pub need: NeedContext,
    pub feed: FeedContext,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NeedContext {
    pub kind: NeedKind,
    pub query: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FeedContext {
    pub status: Status,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleContextReport {
    pub id: String,
    pub brain_kind: String,
    pub policy: String,
    pub slots: Vec<String>,
    pub tools: Vec<ToolContext>,
    pub recent_actions: Vec<TentacleActionContext>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ToolContext {
    pub id: String,
    pub description: String,
    pub runtime: String,
    pub entrypoint: String,
    pub contract: Option<String>,
    pub permission: Option<String>,
    pub authorization_required: bool,
    pub active_grant: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleActionContext {
    pub index: u64,
    pub need: NeedContext,
    pub tool: Option<String>,
    pub plan_source: Option<String>,
    pub status: Status,
    pub summary: String,
}

impl Need {
    pub fn new(kind: NeedKind, query: impl Into<String>) -> Self {
        Self {
            kind,
            query: query.into(),
            context: BTreeMap::new(),
            priority: 0.5,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Evidence {
    pub source: String,
    #[serde(default)]
    pub content: String,
    pub confidence: f32,
    pub metadata: BTreeMap<String, String>,
}

impl Evidence {
    pub fn new(source: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            content: content.into(),
            confidence: 1.0,
            metadata: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Feed {
    pub need: Need,
    pub status: Status,
    pub evidence: Vec<Evidence>,
    pub summary: String,
    pub metadata: BTreeMap<String, String>,
}

impl Feed {
    pub fn satisfied(need: &Need, summary: impl Into<String>, source: impl Into<String>) -> Self {
        let summary = summary.into();
        Self {
            need: need.clone(),
            status: Status::Satisfied,
            evidence: vec![Evidence::new(source, summary.clone())],
            summary,
            metadata: BTreeMap::new(),
        }
    }

    pub fn unsupported(need: &Need, summary: impl Into<String>) -> Self {
        Self {
            need: need.clone(),
            status: Status::Unsupported,
            evidence: Vec::new(),
            summary: summary.into(),
            metadata: BTreeMap::new(),
        }
    }

    pub fn failed(need: &Need, summary: impl Into<String>, source: impl Into<String>) -> Self {
        let summary = summary.into();
        Self {
            need: need.clone(),
            status: Status::Failed,
            evidence: vec![Evidence::new(source, summary.clone())],
            summary,
            metadata: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Feedback {
    pub feeds: Vec<Feed>,
    pub summary: String,
    pub status: Status,
    pub signals: BTreeMap<String, String>,
}

impl Feedback {
    pub fn from_feeds(feeds: Vec<Feed>) -> Self {
        let status = if feeds.iter().all(|feed| feed.status == Status::Satisfied) {
            Status::Satisfied
        } else if feeds.iter().any(|feed| feed.status == Status::Failed) {
            Status::Failed
        } else if feeds.iter().any(|feed| feed.status == Status::Satisfied) {
            Status::Partial
        } else {
            Status::Unsupported
        };
        let summary = feeds
            .iter()
            .map(|feed| feed.summary.as_str())
            .filter(|summary| !summary.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        Self {
            feeds,
            summary,
            status,
            signals: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MemoryRecord {
    pub id: String,
    pub text: String,
    pub metadata: BTreeMap<String, String>,
    pub weight: f32,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MemoryStore {
    records: BTreeMap<String, MemoryRecord>,
    next_id: u64,
}

impl MemoryStore {
    pub fn remember(&mut self, text: impl Into<String>) -> MemoryRecord {
        self.next_id += 1;
        let record = MemoryRecord {
            id: format!("m{}", self.next_id),
            text: text.into(),
            metadata: BTreeMap::new(),
            weight: 1.0,
        };
        self.records.insert(record.id.clone(), record.clone());
        record
    }

    pub fn recall(&mut self, query: &str, limit: usize) -> Vec<MemoryRecord> {
        let words = query
            .split_whitespace()
            .map(str::to_lowercase)
            .collect::<Vec<_>>();
        let mut scored = self
            .records
            .values()
            .cloned()
            .map(|record| {
                let haystack = record.text.to_lowercase();
                let matches = words
                    .iter()
                    .filter(|word| haystack.contains(word.as_str()))
                    .count() as f32;
                let score = if words.is_empty() || matches > 0.0 {
                    matches + record.weight * 0.01
                } else {
                    0.0
                };
                (score, record)
            })
            .filter(|(score, _)| *score > 0.0)
            .collect::<Vec<_>>();
        scored.sort_by(|left, right| right.0.total_cmp(&left.0));
        let recalled = scored
            .into_iter()
            .take(limit)
            .map(|(_, record)| record)
            .collect::<Vec<_>>();
        for record in &recalled {
            if let Some(stored) = self.records.get_mut(&record.id) {
                stored.weight += 0.1;
            }
        }
        recalled
    }

    pub fn forget(&mut self, query: &str) -> usize {
        let query = query.to_lowercase();
        let ids = self
            .records
            .iter()
            .filter(|(_, record)| record.text.to_lowercase().contains(&query))
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        for id in &ids {
            self.records.remove(id);
        }
        ids.len()
    }

    pub fn compact(&mut self, keep: usize) -> usize {
        if self.records.len() <= keep {
            return 0;
        }
        let mut records = self
            .records
            .values()
            .map(|record| {
                (
                    record.weight,
                    memory_id_number(&record.id),
                    record.id.clone(),
                )
            })
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            left.0
                .total_cmp(&right.0)
                .then_with(|| left.1.cmp(&right.1))
                .then_with(|| left.2.cmp(&right.2))
        });
        let drop_count = self.records.len() - keep;
        for (_, _, id) in records.into_iter().take(drop_count) {
            self.records.remove(&id);
        }
        drop_count
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RouteDecision {
    pub tentacle: String,
    pub score: f32,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RouteOption {
    pub tentacle: String,
    pub route_key: String,
    pub score: f32,
    pub supported: bool,
    pub selected: bool,
    pub reason: String,
    pub trace_count: usize,
    pub last_trace: Option<FeedTraceRecord>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RouteReport {
    pub need: Need,
    pub selected: Option<RouteDecision>,
    pub candidates: Vec<RouteOption>,
    pub learned_scores: BTreeMap<String, f32>,
    pub next: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RouteBook {
    pub base_score: f32,
    pub scores: BTreeMap<String, f32>,
}

impl Default for RouteBook {
    fn default() -> Self {
        Self {
            base_score: 1.0,
            scores: BTreeMap::new(),
        }
    }
}

impl RouteBook {
    pub fn choose(&self, need: &Need, names: &[String]) -> Option<RouteDecision> {
        names
            .iter()
            .map(|name| {
                let score = self.score(&need.kind, name);
                RouteDecision {
                    tentacle: name.clone(),
                    score,
                    reason: format!("{}:{}={score:.2}", kind_key(&need.kind), name),
                }
            })
            .max_by(|left, right| left.score.total_cmp(&right.score))
    }

    pub fn learn(&mut self, feed: &Feed) {
        let Some(name) = feed.metadata.get("tentacle") else {
            return;
        };
        let key = route_key(&feed.need.kind, name);
        let current = self.scores.get(&key).copied().unwrap_or(self.base_score);
        let confidence = if feed.evidence.is_empty() {
            0.5
        } else {
            feed.evidence
                .iter()
                .map(|item| item.confidence)
                .sum::<f32>()
                / feed.evidence.len() as f32
        };
        let next = match feed.status {
            Status::Satisfied => current + 0.2 * confidence,
            Status::Partial => current + 0.05 * confidence,
            Status::Failed => current * 0.7,
            Status::Unsupported => current * 0.9,
        };
        self.scores.insert(key, next.clamp(0.05, 10.0));
    }

    pub fn score(&self, kind: &NeedKind, name: &str) -> f32 {
        self.scores
            .get(&route_key(kind, name))
            .copied()
            .unwrap_or(self.base_score)
    }

    pub fn decay(&mut self, rate: f32) -> usize {
        let rate = rate.clamp(0.0, 1.0);
        let mut changed = 0;
        for score in self.scores.values_mut() {
            let next = (*score * rate).clamp(0.05, 10.0);
            if (next - *score).abs() > f32::EPSILON {
                changed += 1;
            }
            *score = next;
        }
        changed
    }
}

pub trait Tentacle {
    fn name(&self) -> &str;
    fn supports(&self, need: &Need) -> bool;
    fn feed(&mut self, need: &Need) -> Feed;
}

pub struct FunctionTentacle<F>
where
    F: FnMut(&Need) -> Feed,
{
    name: String,
    kinds: Vec<NeedKind>,
    handler: F,
}

impl<F> FunctionTentacle<F>
where
    F: FnMut(&Need) -> Feed,
{
    pub fn new(name: impl Into<String>, kinds: Vec<NeedKind>, handler: F) -> Self {
        Self {
            name: name.into(),
            kinds,
            handler,
        }
    }
}

impl<F> Tentacle for FunctionTentacle<F>
where
    F: FnMut(&Need) -> Feed,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn supports(&self, need: &Need) -> bool {
        self.kinds.contains(&need.kind)
    }

    fn feed(&mut self, need: &Need) -> Feed {
        (self.handler)(need)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ToolResult {
    pub status: Status,
    pub output: String,
    pub evidence: Vec<Evidence>,
    pub metadata: BTreeMap<String, String>,
}

impl ToolResult {
    pub fn satisfied(tool: impl Into<String>, output: impl Into<String>) -> Self {
        let tool = tool.into();
        let output = output.into();
        Self {
            status: Status::Satisfied,
            output: output.clone(),
            evidence: vec![Evidence::new(tool, output.clone())],
            metadata: BTreeMap::new(),
        }
    }

    pub fn failed(output: impl Into<String>) -> Self {
        Self {
            status: Status::Failed,
            output: output.into(),
            evidence: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }
}

pub trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn supports(&self, need: &Need) -> bool;
    fn run(&mut self, need: &Need) -> ToolResult;
}

pub struct FunctionTool<F>
where
    F: FnMut(&Need) -> ToolResult,
{
    name: String,
    description: String,
    kinds: Vec<NeedKind>,
    handler: F,
}

impl<F> FunctionTool<F>
where
    F: FnMut(&Need) -> ToolResult,
{
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        kinds: Vec<NeedKind>,
        handler: F,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            kinds,
            handler,
        }
    }
}

impl<F> Tool for FunctionTool<F>
where
    F: FnMut(&Need) -> ToolResult,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn supports(&self, need: &Need) -> bool {
        self.kinds.contains(&need.kind)
    }

    fn run(&mut self, need: &Need) -> ToolResult {
        (self.handler)(need)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ToolCall {
    pub tool: String,
    pub reason: String,
    #[serde(default)]
    pub payload: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Plan {
    pub calls: Vec<ToolCall>,
    pub summary: String,
}

pub trait Planner {
    fn plan(&mut self, need: &Need, tools: &[ToolSpec]) -> Plan;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

impl ChatMessage {
    pub fn new(role: ChatRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChatResponse {
    pub content: String,
    pub metadata: BTreeMap<String, String>,
}

pub trait ChatClient {
    fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String>;
}

pub type ChatClientFactory = Arc<dyn Fn() -> Result<Box<dyn ChatClient>, String>>;

impl<T> ChatClient for Box<T>
where
    T: ChatClient + ?Sized,
{
    fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
        (**self).chat(messages)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct OpenAiCompatibleConfig {
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: String,
    pub timeout_seconds: u64,
    pub retry_count: u32,
    pub retry_delay_ms: u64,
    pub curl_command: String,
    #[serde(default)]
    pub tuning: OpenAiCompatibleTuning,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct OpenAiCompatibleTuning {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra_body: BTreeMap<String, serde_json::Value>,
}

impl OpenAiCompatibleConfig {
    pub fn from_env() -> Result<Self, String> {
        Self::from_env_prefix("OCTOPUS_LLM")
    }

    pub fn from_env_prefix(prefix: &str) -> Result<Self, String> {
        let model_key = format!("{prefix}_MODEL");
        let model = env::var(&model_key)
            .map_err(|_| format!("{model_key} is required for provider chat"))?;
        let api_key = env::var(format!("{prefix}_API_KEY"))
            .ok()
            .filter(|value| !value.is_empty());
        let base_url = env::var(format!("{prefix}_BASE_URL"))
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let timeout_seconds = env::var(format!("{prefix}_TIMEOUT"))
            .ok()
            .map(|value| {
                value
                    .parse::<u64>()
                    .map_err(|_| format!("{prefix}_TIMEOUT must be an integer"))
            })
            .transpose()?
            .unwrap_or(60);
        let retry_count = env::var(format!("{prefix}_RETRIES"))
            .ok()
            .map(|value| {
                value
                    .parse::<u32>()
                    .map_err(|_| format!("{prefix}_RETRIES must be an integer"))
            })
            .transpose()?
            .unwrap_or(1);
        let retry_delay_ms = env::var(format!("{prefix}_RETRY_MS"))
            .ok()
            .map(|value| {
                value
                    .parse::<u64>()
                    .map_err(|_| format!("{prefix}_RETRY_MS must be an integer"))
            })
            .transpose()?
            .unwrap_or(750);
        let curl_command =
            env::var(format!("{prefix}_CURL")).unwrap_or_else(|_| "curl".to_string());
        let tuning = OpenAiCompatibleTuning::from_env_prefix(prefix)?;
        Ok(Self {
            model,
            api_key,
            base_url,
            timeout_seconds,
            retry_count,
            retry_delay_ms,
            curl_command,
            tuning,
        })
    }

    pub fn endpoint(&self) -> String {
        format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
    }
}

impl OpenAiCompatibleTuning {
    fn from_env_prefix(prefix: &str) -> Result<Self, String> {
        Ok(Self {
            temperature: optional_f64_env(prefix, "TEMPERATURE")?,
            top_p: optional_f64_env(prefix, "TOP_P")?,
            max_tokens: optional_u64_env(prefix, "MAX_TOKENS")?,
            reasoning_effort: env::var(format!("{prefix}_REASONING_EFFORT"))
                .ok()
                .filter(|value| !value.trim().is_empty()),
            extra_body: optional_json_object_env(prefix, "EXTRA_BODY")?,
        })
    }
}

fn optional_f64_env(prefix: &str, suffix: &str) -> Result<Option<f64>, String> {
    let key = format!("{prefix}_{suffix}");
    env::var(&key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            let parsed = value
                .parse::<f64>()
                .map_err(|_| format!("{key} must be a number"))?;
            if parsed.is_finite() {
                Ok(parsed)
            } else {
                Err(format!("{key} must be finite"))
            }
        })
        .transpose()
}

fn optional_u64_env(prefix: &str, suffix: &str) -> Result<Option<u64>, String> {
    let key = format!("{prefix}_{suffix}");
    env::var(&key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|_| format!("{key} must be an integer"))
        })
        .transpose()
}

fn optional_json_object_env(
    prefix: &str,
    suffix: &str,
) -> Result<BTreeMap<String, serde_json::Value>, String> {
    let key = format!("{prefix}_{suffix}");
    let Some(payload) = env::var(&key).ok().filter(|value| !value.trim().is_empty()) else {
        return Ok(BTreeMap::new());
    };
    match serde_json::from_str::<serde_json::Value>(&payload)
        .map_err(|error| format!("{key} must be JSON: {error}"))?
    {
        serde_json::Value::Object(object) => Ok(object.into_iter().collect()),
        _ => Err(format!("{key} must be a JSON object")),
    }
}

pub struct OpenAiCompatibleChatClient {
    config: OpenAiCompatibleConfig,
}

impl OpenAiCompatibleChatClient {
    pub fn new(config: OpenAiCompatibleConfig) -> Self {
        Self { config }
    }

    pub fn from_env() -> Result<Self, String> {
        Ok(Self::new(OpenAiCompatibleConfig::from_env()?))
    }

    fn request_body(&self, messages: &[ChatMessage]) -> Result<String, String> {
        let mut body = serde_json::Map::new();
        for (key, value) in &self.config.tuning.extra_body {
            body.insert(key.clone(), value.clone());
        }
        body.insert(
            "model".to_string(),
            serde_json::Value::String(self.config.model.clone()),
        );
        body.insert(
            "messages".to_string(),
            serde_json::to_value(messages).map_err(|error| error.to_string())?,
        );
        if let Some(temperature) = self.config.tuning.temperature {
            body.insert(
                "temperature".to_string(),
                serde_json::Number::from_f64(temperature)
                    .map(serde_json::Value::Number)
                    .ok_or_else(|| "temperature must be finite".to_string())?,
            );
        }
        if let Some(top_p) = self.config.tuning.top_p {
            body.insert(
                "top_p".to_string(),
                serde_json::Number::from_f64(top_p)
                    .map(serde_json::Value::Number)
                    .ok_or_else(|| "top_p must be finite".to_string())?,
            );
        }
        if let Some(max_tokens) = self.config.tuning.max_tokens {
            body.insert(
                "max_tokens".to_string(),
                serde_json::Value::Number(max_tokens.into()),
            );
        }
        if let Some(reasoning_effort) = &self.config.tuning.reasoning_effort {
            body.insert(
                "reasoning_effort".to_string(),
                serde_json::Value::String(reasoning_effort.clone()),
            );
        }
        body.entry("stream".to_string())
            .or_insert(serde_json::Value::Bool(false));
        serde_json::to_string(&serde_json::Value::Object(body)).map_err(|error| error.to_string())
    }

    fn chat_once(&self, body: &str) -> Result<ChatResponse, String> {
        let mut command = Command::new(&self.config.curl_command);
        command
            .arg("-sS")
            .arg("--max-time")
            .arg(self.config.timeout_seconds.to_string())
            .arg("-X")
            .arg("POST")
            .arg(self.config.endpoint())
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("--data-binary")
            .arg(body);
        if let Some(api_key) = &self.config.api_key {
            command
                .arg("-H")
                .arg(format!("Authorization: Bearer {api_key}"));
        }
        let output = command.output().map_err(|error| {
            format!(
                "{} failed to start: {error}",
                self.config.curl_command.as_str()
            )
        })?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success() {
            let message = if stderr.trim().is_empty() {
                stdout
            } else {
                stderr
            };
            return Err(trim_output(&message));
        }
        let data = serde_json::from_str::<serde_json::Value>(&stdout)
            .map_err(|error| format!("invalid chat response JSON: {error}"))?;
        if let Some(error) = data.get("error") {
            return Err(chat_error_message(error));
        }
        let content = extract_chat_content(&data)?;
        Ok(ChatResponse {
            content,
            metadata: BTreeMap::from([
                ("provider".to_string(), "openai-compatible".to_string()),
                ("model".to_string(), self.config.model.clone()),
                ("base_url".to_string(), self.config.base_url.clone()),
            ]),
        })
    }
}

impl ChatClient for OpenAiCompatibleChatClient {
    fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
        let body = self.request_body(messages)?;
        let mut last_error = String::new();
        for attempt in 0..=self.config.retry_count {
            match self.chat_once(&body) {
                Ok(response) => return Ok(response),
                Err(error) => {
                    last_error = error;
                    if attempt < self.config.retry_count {
                        thread::sleep(Duration::from_millis(self.config.retry_delay_ms));
                    }
                }
            }
        }
        Err(last_error)
    }
}

pub fn chat_client_factory_from_openai_config(config: OpenAiCompatibleConfig) -> ChatClientFactory {
    Arc::new(move || Ok(Box::new(OpenAiCompatibleChatClient::new(config.clone()))))
}

fn chat_error_message(error: &serde_json::Value) -> String {
    error
        .get("message")
        .and_then(serde_json::Value::as_str)
        .or_else(|| error.get("code").and_then(serde_json::Value::as_str))
        .map(|value| format!("provider error: {value}"))
        .unwrap_or_else(|| format!("provider error: {error}"))
}

fn extract_chat_content(data: &serde_json::Value) -> Result<String, String> {
    let content = data.pointer("/choices/0/message/content");
    if let Some(text) = content.and_then(serde_json::Value::as_str) {
        let text = text.trim().to_string();
        if !text.is_empty() {
            return Ok(text);
        }
    }
    if let Some(parts) = content.and_then(serde_json::Value::as_array) {
        let text = parts
            .iter()
            .filter_map(|part| {
                part.get("text")
                    .or_else(|| part.get("content"))
                    .and_then(serde_json::Value::as_str)
            })
            .collect::<Vec<_>>()
            .join("");
        let text = text.trim().to_string();
        if !text.is_empty() {
            return Ok(text);
        }
    }
    if let Some(text) = data
        .pointer("/choices/0/text")
        .and_then(serde_json::Value::as_str)
    {
        let text = text.trim().to_string();
        if !text.is_empty() {
            return Ok(text);
        }
    }
    if data
        .pointer("/choices/0/message/reasoning_content")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
    {
        return Err(
            "chat response content is empty after reasoning_content; increase max_tokens or disable thinking"
                .to_string(),
        );
    }
    Err("chat response missing non-empty choices[0].message.content".to_string())
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CodexCliConfig {
    pub model: Option<String>,
    pub command: String,
    pub timeout_seconds: u64,
    pub retry_count: u32,
    pub retry_delay_ms: u64,
}

impl CodexCliConfig {
    pub fn from_env_prefix(prefix: &str) -> Result<Self, String> {
        let model = env::var(format!("{prefix}_MODEL"))
            .ok()
            .filter(|value| !value.trim().is_empty());
        let command =
            env::var(format!("{prefix}_CODEX_COMMAND")).unwrap_or_else(|_| "codex".to_string());
        let timeout_seconds = env::var(format!("{prefix}_TIMEOUT"))
            .ok()
            .map(|value| {
                value
                    .parse::<u64>()
                    .map_err(|_| format!("{prefix}_TIMEOUT must be an integer"))
            })
            .transpose()?
            .unwrap_or(120);
        let retry_count = env::var(format!("{prefix}_RETRIES"))
            .ok()
            .map(|value| {
                value
                    .parse::<u32>()
                    .map_err(|_| format!("{prefix}_RETRIES must be an integer"))
            })
            .transpose()?
            .unwrap_or(1);
        let retry_delay_ms = env::var(format!("{prefix}_RETRY_MS"))
            .ok()
            .map(|value| {
                value
                    .parse::<u64>()
                    .map_err(|_| format!("{prefix}_RETRY_MS must be an integer"))
            })
            .transpose()?
            .unwrap_or(750);
        Ok(Self {
            model,
            command,
            timeout_seconds,
            retry_count,
            retry_delay_ms,
        })
    }
}

pub struct CodexCliChatClient {
    config: CodexCliConfig,
}

impl CodexCliChatClient {
    pub fn new(config: CodexCliConfig) -> Self {
        Self { config }
    }

    fn prompt(messages: &[ChatMessage]) -> String {
        let mut prompt = String::from(
            "You are an Octopus LLM provider. Answer only the user's requested content. Do not run tools unless explicitly requested.\n\n",
        );
        for message in messages {
            let role = match message.role {
                ChatRole::System => "system",
                ChatRole::User => "user",
                ChatRole::Assistant => "assistant",
                ChatRole::Tool => "tool",
            };
            prompt.push_str(role);
            prompt.push_str(": ");
            prompt.push_str(&message.content);
            prompt.push('\n');
        }
        prompt
    }
}

impl ChatClient for CodexCliChatClient {
    fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
        let mut last_error = String::new();
        for attempt in 0..=self.config.retry_count {
            match self.chat_once(messages) {
                Ok(response) => return Ok(response),
                Err(error) => {
                    last_error = error;
                    if attempt < self.config.retry_count {
                        thread::sleep(Duration::from_millis(self.config.retry_delay_ms));
                    }
                }
            }
        }
        Err(last_error)
    }
}

impl CodexCliChatClient {
    fn chat_once(&self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
        let temp_suffix = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_millis())
                .unwrap_or_default()
        );
        let output_path = env::temp_dir().join(format!("octopus-codex-provider-{temp_suffix}.txt"));
        let prompt_path =
            env::temp_dir().join(format!("octopus-codex-provider-{temp_suffix}.prompt"));
        let stdout_path =
            env::temp_dir().join(format!("octopus-codex-provider-{temp_suffix}.stdout"));
        let stderr_path =
            env::temp_dir().join(format!("octopus-codex-provider-{temp_suffix}.stderr"));
        let prompt = Self::prompt(messages);
        fs::write(&prompt_path, prompt)
            .map_err(|error| format!("failed to write codex prompt file: {error}"))?;
        let mut command = Command::new(&self.config.command);
        let prompt_file = fs::File::open(&prompt_path)
            .map_err(|error| format!("failed to open codex prompt file: {error}"))?;
        let stdout_file = fs::File::create(&stdout_path)
            .map_err(|error| format!("failed to create codex stdout capture: {error}"))?;
        let stderr_file = fs::File::create(&stderr_path)
            .map_err(|error| format!("failed to create codex stderr capture: {error}"))?;
        command
            .arg("exec")
            .arg("--ephemeral")
            .arg("--skip-git-repo-check")
            .arg("--disable")
            .arg("image_generation")
            .arg("--sandbox")
            .arg("read-only")
            .arg("--output-last-message")
            .arg(&output_path);
        if let Some(model) = &self.config.model {
            command.arg("--model").arg(model);
        }
        let mut child = command
            .arg("-")
            .stdin(Stdio::from(prompt_file))
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .spawn()
            .map_err(|error| format!("{} failed to start: {error}", self.config.command))?;
        let deadline = Instant::now() + Duration::from_secs(self.config.timeout_seconds.max(1));
        let status = loop {
            if let Some(status) = child
                .try_wait()
                .map_err(|error| format!("{} failed: {error}", self.config.command))?
            {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                let _ = fs::remove_file(&prompt_path);
                let _ = fs::remove_file(&output_path);
                let _ = fs::remove_file(&stdout_path);
                let _ = fs::remove_file(&stderr_path);
                return Err(format!(
                    "codex provider timed out after {}s",
                    self.config.timeout_seconds.max(1)
                ));
            }
            thread::sleep(Duration::from_millis(50));
        };
        let _ = fs::remove_file(&prompt_path);
        let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
        let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
        let _ = fs::remove_file(&stdout_path);
        let _ = fs::remove_file(&stderr_path);
        if !status.success() {
            let message = if stderr.trim().is_empty() {
                stdout
            } else {
                stderr
            };
            let _ = fs::remove_file(&output_path);
            return Err(trim_output(&message));
        }
        let content = fs::read_to_string(&output_path)
            .unwrap_or_else(|_| stdout.clone())
            .trim()
            .to_string();
        let _ = fs::remove_file(&output_path);
        if content.is_empty() {
            return Err("codex provider returned empty content".to_string());
        }
        Ok(ChatResponse {
            content,
            metadata: BTreeMap::from([
                ("provider".to_string(), "codex-cli".to_string()),
                (
                    "model".to_string(),
                    self.config
                        .model
                        .clone()
                        .unwrap_or_else(|| "codex-default".to_string()),
                ),
                ("base_url".to_string(), "codex-cli".to_string()),
            ]),
        })
    }
}

pub struct ChatPlanner<C>
where
    C: ChatClient,
{
    client: C,
}

impl<C> ChatPlanner<C>
where
    C: ChatClient,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }
}

impl<C> Planner for ChatPlanner<C>
where
    C: ChatClient,
{
    fn plan(&mut self, need: &Need, tools: &[ToolSpec]) -> Plan {
        let tool_sheet = tools
            .iter()
            .map(|tool| format!("- {}: {}", tool.name, tool.description))
            .collect::<Vec<_>>()
            .join("\n");
        let messages = vec![
            ChatMessage::new(
                ChatRole::System,
                "You are an Octopus tentacle brain. Return only JSON for Plan.",
            ),
            ChatMessage::new(
                ChatRole::User,
                format!(
                    "Need: {}: {}\nTools:\n{}\nReturn JSON: {{\"calls\":[{{\"tool\":\"name\",\"reason\":\"short\",\"payload\":{{}}}}],\"summary\":\"short\"}}",
                    kind_key(&need.kind),
                    need.query,
                    tool_sheet
                ),
            ),
        ];
        match self.client.chat(&messages) {
            Ok(response) => match plan_from_llm_content(&response.content) {
                Ok(plan) if !plan.calls.is_empty() => plan,
                Ok(_) => Plan {
                    calls: Vec::new(),
                    summary: "llm planning returned no tool calls".to_string(),
                },
                Err(error) => Plan {
                    calls: Vec::new(),
                    summary: format!("llm planning returned invalid JSON: {error}"),
                },
            },
            Err(error) => Plan {
                calls: Vec::new(),
                summary: format!("llm planning failed: {error}"),
            },
        }
    }
}

fn plan_from_llm_content(content: &str) -> Result<Plan, String> {
    serde_json::from_str::<Plan>(content)
        .or_else(|direct_error| {
            extract_first_json_object(content)
                .ok_or_else(|| direct_error.to_string())
                .and_then(|json| {
                    serde_json::from_str::<Plan>(json)
                        .map_err(|extract_error| extract_error.to_string())
                })
        })
        .map_err(|error| format!("invalid Plan JSON: {error}"))
}

fn extract_first_json_object(content: &str) -> Option<&str> {
    let start = content.find('{')?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, ch) in content[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let end = start + offset + ch.len_utf8();
                    return Some(&content[start..end]);
                }
            }
            _ => {}
        }
    }
    None
}

fn goal_refinement_from_chat<C>(
    goal: Option<&Goal>,
    message: &str,
    client: &mut C,
) -> Result<GoalRefinement, String>
where
    C: ChatClient,
{
    let current_goal = goal
        .map(|goal| serde_json::to_string(goal).unwrap_or_else(|_| "null".to_string()))
        .unwrap_or_else(|| "null".to_string());
    let response = client.chat(&[
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain chat layer. Refine the Goal and suggest cognitive Needs only. Do not choose tools or implementation. Return only JSON: {\"objective\":\"short goal\",\"constraints\":[\"short constraint\"],\"summary\":\"short update\",\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!("Current goal JSON: {current_goal}\nUser message: {message}"),
        ),
    ])?;
    serde_json::from_str::<GoalRefinement>(&response.content)
        .map_err(|error| format!("invalid goal refinement JSON: {error}"))
}

fn brain_goal_from_chat<C>(
    brain: &BrainContextReport,
    prompt: &str,
    client: &mut C,
) -> Result<GoalRefinement, String>
where
    C: ChatClient,
{
    let context = serde_json::json!({
        "policy": brain.policy,
        "slots": brain.slots,
        "goal": brain.goal,
        "mem": brain.mem,
        "recent_need_feed": brain.turns,
    });
    let response = client.chat(&[
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain goal layer. You see only Goal, Mem, Need, and Feed. Refine the Goal and suggest cognitive Needs only. Do not choose tools, APIs, files, commands, routes, tentacles, or implementation. Return only JSON: {\"objective\":\"short goal\",\"constraints\":[\"short constraint\"],\"summary\":\"short goal update\",\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!("Clean brain context JSON: {context}\nUser goal prompt: {prompt}"),
        ),
    ])?;
    serde_json::from_str::<GoalRefinement>(&response.content)
        .map_err(|error| format!("invalid clean-brain goal JSON: {error}"))
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainExploreDraft {
    pub summary: String,
    #[serde(default)]
    pub needs: Vec<GoalNeedSuggestion>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainDeliberationDraft {
    pub summary: String,
    #[serde(default)]
    pub observations: Vec<String>,
    #[serde(default)]
    pub questions: Vec<String>,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub risks: Vec<String>,
    #[serde(default)]
    pub needs: Vec<GoalNeedSuggestion>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainReflectionDraft {
    pub summary: String,
    pub goal_state: String,
    #[serde(default)]
    pub evidence: Vec<String>,
    #[serde(default)]
    pub gaps: Vec<String>,
    #[serde(default)]
    pub questions: Vec<String>,
    #[serde(default)]
    pub needs: Vec<GoalNeedSuggestion>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainSynthesisInput {
    pub summary: Option<String>,
    #[serde(default)]
    pub observations: Vec<String>,
    #[serde(default)]
    pub questions: Vec<String>,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub risks: Vec<String>,
    #[serde(default)]
    pub needs: Vec<GoalNeedSuggestion>,
    #[serde(default)]
    pub drafts: Vec<BrainSynthesisDraft>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrainSynthesisDraft {
    pub source: Option<String>,
    pub summary: Option<String>,
    #[serde(default)]
    pub observations: Vec<String>,
    #[serde(default)]
    pub questions: Vec<String>,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub risks: Vec<String>,
    #[serde(default)]
    pub needs: Vec<GoalNeedSuggestion>,
}

fn brain_explore_from_chat<C>(
    brain: &BrainContextReport,
    prompt: &str,
    client: &mut C,
) -> Result<BrainExploreDraft, String>
where
    C: ChatClient,
{
    let context = serde_json::json!({
        "policy": brain.policy,
        "slots": brain.slots,
        "goal": brain.goal,
        "mem": brain.mem,
        "recent_need_feed": brain.turns,
    });
    let response = client.chat(&[
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain exploration layer. You see only Goal, Mem, Need, and Feed. Express cognitive Needs only. Do not choose tools, APIs, files, commands, routes, tentacles, or implementation. Return only JSON: {\"summary\":\"short exploration\",\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!("Clean brain context JSON: {context}\nUser exploration prompt: {prompt}"),
        ),
    ])?;
    serde_json::from_str::<BrainExploreDraft>(&response.content)
        .map_err(|error| format!("invalid clean-brain explore JSON: {error}"))
}

fn brain_brief_from_chat<C>(
    brain: &BrainContextReport,
    prompt: &str,
    client: &mut C,
) -> Result<BrainExploreDraft, String>
where
    C: ChatClient,
{
    let context = serde_json::json!({
        "policy": brain.policy,
        "slots": brain.slots,
        "goal": brain.goal,
        "mem": brain.mem,
        "recent_need_feed": brain.turns,
    });
    let response = client.chat(&[
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain brief layer. You see only Goal, Mem, Need, and Feed. Compress the current cognitive state into a compact brief for another strong model, then express the next cognitive Needs only. Do not choose tools, APIs, files, commands, routes, tentacles, or implementation. Return only JSON: {\"summary\":\"short cognitive brief\",\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!("Clean brain context JSON: {context}\nUser brief prompt: {prompt}"),
        ),
    ])?;
    serde_json::from_str::<BrainExploreDraft>(&response.content)
        .map_err(|error| format!("invalid clean-brain brief JSON: {error}"))
}

fn brain_intent_from_chat<C>(
    brain: &BrainContextReport,
    prompt: &str,
    client: &mut C,
) -> Result<BrainExploreDraft, String>
where
    C: ChatClient,
{
    let context = serde_json::json!({
        "policy": brain.policy,
        "slots": brain.slots,
        "goal": brain.goal,
        "mem": brain.mem,
        "recent_need_feed": brain.turns,
    });
    let response = client.chat(&[
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain intent layer. You see only Goal, Mem, Need, and Feed. First choose the cognitive intent the brain should express next: observe, verify, reproduce, compare, remember, forget, recall, or execute. Then emit compact cognitive Needs for those intents. Do not choose tools, APIs, files, commands, routes, tentacles, or implementation. Return only JSON: {\"summary\":\"short intent map\",\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!("Clean brain context JSON: {context}\nUser intent prompt: {prompt}"),
        ),
    ])?;
    serde_json::from_str::<BrainExploreDraft>(&response.content)
        .map_err(|error| format!("invalid clean-brain intent JSON: {error}"))
}

fn brain_focus_from_chat<C>(
    brain: &BrainContextReport,
    kind: &NeedKind,
    prompt: &str,
    client: &mut C,
) -> Result<BrainExploreDraft, String>
where
    C: ChatClient,
{
    let kind = kind_key(kind);
    let context = serde_json::json!({
        "policy": brain.policy,
        "slots": brain.slots,
        "goal": brain.goal,
        "mem": brain.mem,
        "recent_need_feed": brain.turns,
        "focus_kind": kind,
    });
    let response = client.chat(&[
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain Need focus layer. You see only Goal, Mem, Need, and Feed. Express cognitive Needs of the requested kind when possible. Do not choose tools, APIs, files, commands, routes, tentacles, or implementation. Return only JSON: {\"summary\":\"short focus\",\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!(
                "Clean brain focus kind: {kind}\nClean brain context JSON: {context}\nUser focus prompt: {prompt}"
            ),
        ),
    ])?;
    serde_json::from_str::<BrainExploreDraft>(&response.content)
        .map_err(|error| format!("invalid clean-brain focus JSON: {error}"))
}

fn brain_memory_from_chat<C>(
    brain: &BrainContextReport,
    prompt: &str,
    client: &mut C,
) -> Result<BrainExploreDraft, String>
where
    C: ChatClient,
{
    let context = serde_json::json!({
        "policy": brain.policy,
        "slots": brain.slots,
        "goal": brain.goal,
        "mem": brain.mem,
        "recent_need_feed": brain.turns,
    });
    let response = client.chat(&[
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain memory layer. You see only Goal, Mem, Need, and Feed. Express memory-shaped cognitive Needs only: remember durable context, recall relevant context, or forget stale context. Do not choose tools, APIs, files, commands, routes, tentacles, or implementation. Return only JSON: {\"summary\":\"short memory focus\",\"needs\":[{\"kind\":\"remember|recall|forget|verify\",\"query\":\"short cognitive memory request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!("Clean brain context JSON: {context}\nUser memory prompt: {prompt}"),
        ),
    ])?;
    serde_json::from_str::<BrainExploreDraft>(&response.content)
        .map_err(|error| format!("invalid clean-brain memory JSON: {error}"))
}

fn brain_clarification_from_chat<C>(
    brain: &BrainContextReport,
    prompt: &str,
    client: &mut C,
) -> Result<BrainDeliberationDraft, String>
where
    C: ChatClient,
{
    let context = serde_json::json!({
        "policy": brain.policy,
        "slots": brain.slots,
        "goal": brain.goal,
        "mem": brain.mem,
        "recent_need_feed": brain.turns,
    });
    let response = client.chat(&[
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain clarification layer. You see only Goal, Mem, Need, and Feed. Ask the few human-facing questions that would make the Goal and next cognitive Needs clearer. Express cognitive Needs only. Do not choose tools, APIs, files, commands, routes, tentacles, or implementation. Return only JSON: {\"summary\":\"short clarification focus\",\"observations\":[\"context fact\"],\"questions\":[\"question for the human\"],\"options\":[\"possible cognitive direction\"],\"risks\":[\"ambiguity risk\"],\"needs\":[{\"kind\":\"observe|verify|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!("Clean brain context JSON: {context}\nUser clarification prompt: {prompt}"),
        ),
    ])?;
    serde_json::from_str::<BrainDeliberationDraft>(&response.content)
        .map_err(|error| format!("invalid clean-brain clarification JSON: {error}"))
}

fn brain_agenda_from_chat<C>(
    brain: &BrainContextReport,
    prompt: &str,
    client: &mut C,
) -> Result<BrainDeliberationDraft, String>
where
    C: ChatClient,
{
    let context = serde_json::json!({
        "policy": brain.policy,
        "slots": brain.slots,
        "goal": brain.goal,
        "mem": brain.mem,
        "recent_need_feed": brain.turns,
    });
    let response = client.chat(&[
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain agenda layer. You see only Goal, Mem, Need, and Feed. Build a compact cognitive agenda: what matters now, which questions deserve attention, which cognitive options are plausible, which risks could mislead the brain, and which clean Needs should be asked next. Do not choose tools, APIs, files, commands, routes, tentacles, or implementation. Return only JSON: {\"summary\":\"short agenda focus\",\"observations\":[\"cognitive signal\"],\"questions\":[\"open cognitive question\"],\"options\":[\"possible cognitive priority\"],\"risks\":[\"reasoning risk\"],\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!("Clean brain context JSON: {context}\nUser agenda prompt: {prompt}"),
        ),
    ])?;
    serde_json::from_str::<BrainDeliberationDraft>(&response.content)
        .map_err(|error| format!("invalid clean-brain agenda JSON: {error}"))
}

fn brain_scout_from_chat<C>(
    brain: &BrainContextReport,
    prompt: &str,
    client: &mut C,
) -> Result<BrainDeliberationDraft, String>
where
    C: ChatClient,
{
    let context = serde_json::json!({
        "policy": brain.policy,
        "slots": brain.slots,
        "goal": brain.goal,
        "mem": brain.mem,
        "recent_need_feed": brain.turns,
    });
    let response = client.chat(&[
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain scout layer. You see only Goal, Mem, Need, and Feed. Scout the cognitive landscape before the next Need: name useful signals, hidden assumptions, unknowns, possible directions, risks of drift, and clean cognitive Needs that would improve the brain's map. Do not choose tools, APIs, files, commands, routes, tentacles, or implementation. Return only JSON: {\"summary\":\"short scout map\",\"observations\":[\"cognitive signal\"],\"questions\":[\"important unknown\"],\"options\":[\"possible cognitive direction\"],\"risks\":[\"drift or blind spot\"],\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!("Clean brain context JSON: {context}\nUser scout prompt: {prompt}"),
        ),
    ])?;
    serde_json::from_str::<BrainDeliberationDraft>(&response.content)
        .map_err(|error| format!("invalid clean-brain scout JSON: {error}"))
}

fn brain_deliberation_from_chat<C>(
    brain: &BrainContextReport,
    prompt: &str,
    client: &mut C,
) -> Result<BrainDeliberationDraft, String>
where
    C: ChatClient,
{
    let context = serde_json::json!({
        "policy": brain.policy,
        "slots": brain.slots,
        "goal": brain.goal,
        "mem": brain.mem,
        "recent_need_feed": brain.turns,
    });
    let response = client.chat(&[
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain deliberation layer. You see only Goal, Mem, Need, and Feed. Produce compact cognitive deliberation and cognitive Needs only. Do not choose tools, APIs, files, commands, routes, tentacles, or implementation. Return only JSON: {\"summary\":\"short deliberation\",\"observations\":[\"cognitive observation\"],\"questions\":[\"open cognitive question\"],\"options\":[\"possible cognitive direction\"],\"risks\":[\"reasoning risk\"],\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!("Clean brain context JSON: {context}\nUser deliberation prompt: {prompt}"),
        ),
    ])?;
    serde_json::from_str::<BrainDeliberationDraft>(&response.content)
        .map_err(|error| format!("invalid clean-brain deliberation JSON: {error}"))
}

fn brain_reflection_from_chat<C>(
    brain: &BrainContextReport,
    prompt: &str,
    client: &mut C,
) -> Result<BrainReflectionDraft, String>
where
    C: ChatClient,
{
    let context = serde_json::json!({
        "policy": brain.policy,
        "slots": brain.slots,
        "goal": brain.goal,
        "mem": brain.mem,
        "recent_need_feed": brain.turns,
    });
    let response = client.chat(&[
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain reflection layer. You see only Goal, Mem, Need, and Feed. Reflect on whether the Goal is satisfied, what evidence exists, what gaps remain, what questions matter, and what cognitive Needs should be asked next. Do not choose tools, APIs, files, commands, routes, tentacles, or implementation. Return only JSON: {\"summary\":\"short reflection\",\"goal_state\":\"satisfied|partial|uncertain|blocked\",\"evidence\":[\"evidence from Goal/Mem/Need/Feed\"],\"gaps\":[\"remaining cognitive gap\"],\"questions\":[\"open cognitive question\"],\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!("Clean brain context JSON: {context}\nUser reflection prompt: {prompt}"),
        ),
    ])?;
    serde_json::from_str::<BrainReflectionDraft>(&response.content)
        .map_err(|error| format!("invalid clean-brain reflection JSON: {error}"))
}

fn brain_alignment_from_chat<C>(
    brain: &BrainContextReport,
    prompt: &str,
    client: &mut C,
) -> Result<BrainReflectionDraft, String>
where
    C: ChatClient,
{
    let context = serde_json::json!({
        "policy": brain.policy,
        "slots": brain.slots,
        "goal": brain.goal,
        "mem": brain.mem,
        "recent_need_feed": brain.turns,
    });
    let response = client.chat(&[
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain alignment layer. You see only Goal, Mem, Need, and Feed. Judge whether the current cognitive direction follows the Goal and constraints, name supporting evidence, name alignment gaps, ask goal-facing questions, and express only cognitive Needs. Do not choose tools, APIs, files, commands, routes, tentacles, or implementation. Return only JSON: {\"summary\":\"short alignment check\",\"goal_state\":\"aligned|partial|uncertain|blocked\",\"evidence\":[\"evidence from Goal/Mem/Need/Feed\"],\"gaps\":[\"goal alignment gap\"],\"questions\":[\"open goal-facing question\"],\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!("Clean brain context JSON: {context}\nUser alignment prompt: {prompt}"),
        ),
    ])?;
    serde_json::from_str::<BrainReflectionDraft>(&response.content)
        .map_err(|error| format!("invalid clean-brain alignment JSON: {error}"))
}

fn brain_synthesis_from_chat<C>(
    brain: &BrainContextReport,
    prompt: &str,
    input: &BrainSynthesisInput,
    client: &mut C,
) -> Result<BrainSynthesisInput, String>
where
    C: ChatClient,
{
    let context = serde_json::json!({
        "policy": brain.policy,
        "slots": brain.slots,
        "goal": brain.goal,
        "mem": brain.mem,
        "recent_need_feed": brain.turns,
        "current_need": prompt,
        "current_feed": {
            "summary": "clean-brain draft bundle",
            "drafts": input,
        },
    });
    let response = client.chat(&[
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain synthesis layer. You see only Goal, Mem, Need, and Feed. The current Feed contains clean-brain draft replies from other models. Synthesize that Feed into compact cognitive observations, questions, options, risks, and cognitive Needs only. Do not choose tools, APIs, files, commands, routes, tentacles, or implementation. Return only JSON: {\"summary\":\"short synthesis\",\"observations\":[\"cognitive observation\"],\"questions\":[\"open cognitive question\"],\"options\":[\"possible cognitive direction\"],\"risks\":[\"reasoning risk\"],\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!("Clean brain synthesis prompt: {prompt}\nClean brain context JSON: {context}"),
        ),
    ])?;
    serde_json::from_str::<BrainSynthesisInput>(&response.content)
        .map_err(|error| format!("invalid clean-brain synthesis JSON: {error}"))
}

fn brain_rewrite_from_chat<C>(
    brain: &BrainContextReport,
    prompt: &str,
    draft: &BrainExploreDraft,
    audit: &BrainNeedAudit,
    client: &mut C,
) -> Result<BrainExploreDraft, String>
where
    C: ChatClient,
{
    let context = serde_json::json!({
        "policy": brain.policy,
        "slots": brain.slots,
        "goal": brain.goal,
        "mem": brain.mem,
        "recent_need_feed": brain.turns,
        "raw_summary": draft.summary,
        "raw_needs": draft.needs,
        "audit": audit,
    });
    let response = client.chat(&[
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain Need rewrite layer. You see only Goal, Mem, Need, Feed, and rejected Need candidates. Rewrite tool, API, command, file, route, or implementation details into cognitive Needs only. Do not choose tools or implementation. Return only JSON: {\"summary\":\"short rewrite\",\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!("Clean brain rewrite prompt: {prompt}\nClean brain context JSON: {context}"),
        ),
    ])?;
    serde_json::from_str::<BrainExploreDraft>(&response.content)
        .map_err(|error| format!("invalid clean-brain rewrite JSON: {error}"))
}

pub struct PlanningTentacle<P>
where
    P: Planner,
{
    name: String,
    kinds: Vec<NeedKind>,
    planner: P,
    tools: Vec<Box<dyn Tool>>,
}

impl<P> PlanningTentacle<P>
where
    P: Planner,
{
    pub fn new(
        name: impl Into<String>,
        kinds: Vec<NeedKind>,
        planner: P,
        tools: Vec<Box<dyn Tool>>,
    ) -> Self {
        Self {
            name: name.into(),
            kinds,
            planner,
            tools,
        }
    }
}

impl<P> Tentacle for PlanningTentacle<P>
where
    P: Planner,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn supports(&self, need: &Need) -> bool {
        self.kinds.contains(&need.kind) && self.tools.iter().any(|tool| tool.supports(need))
    }

    fn feed(&mut self, need: &Need) -> Feed {
        let specs = self
            .tools
            .iter()
            .filter(|tool| tool.supports(need))
            .map(|tool| ToolSpec {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
            })
            .collect::<Vec<_>>();
        let plan = self.planner.plan(need, &specs);
        if plan.calls.is_empty() {
            return Feed::unsupported(need, plan.summary);
        }

        let mut results = Vec::new();
        for call in &plan.calls {
            let Some(tool) = self.tools.iter_mut().find(|tool| tool.name() == call.tool) else {
                results.push(ToolResult::failed(format!("unknown tool: {}", call.tool)));
                continue;
            };
            if !tool.supports(need) {
                results.push(ToolResult::failed(format!(
                    "tool does not support need: {}",
                    call.tool
                )));
                continue;
            }
            results.push(tool.run(need));
        }

        let status = tool_status(&results);
        let summary = results
            .iter()
            .map(|result| result.output.as_str())
            .filter(|output| !output.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        let mut metadata = BTreeMap::from([
            ("plan".to_string(), plan.summary),
            (
                "tools".to_string(),
                plan.calls
                    .iter()
                    .map(|call| call.tool.as_str())
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        ]);
        metadata.insert("tentacle_brain".to_string(), self.name.clone());
        Feed {
            need: need.clone(),
            status,
            evidence: results
                .into_iter()
                .flat_map(|result| result.evidence)
                .collect(),
            summary,
            metadata,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct HarnessState {
    pub memory: MemoryStore,
    pub routes: RouteBook,
    #[serde(default)]
    pub installed_profiles: Vec<String>,
    #[serde(default)]
    pub installed_tentacles: Vec<InstalledTentacle>,
    #[serde(default)]
    pub goal: Option<Goal>,
    #[serde(default)]
    pub goal_turns: Vec<GoalTurn>,
    #[serde(default)]
    pub need_queue: Vec<NeedQueueItem>,
    #[serde(default)]
    pub next_need_queue_index: u64,
    #[serde(default)]
    pub grants: Vec<CapabilityGrant>,
    #[serde(default)]
    pub evolution_outcomes: Vec<EvolutionOutcome>,
    #[serde(default)]
    pub last_pet_event: Option<PetEvent>,
    #[serde(default)]
    pub feed_traces: Vec<FeedTraceRecord>,
    #[serde(default)]
    pub next_feed_trace_index: u64,
    #[serde(default)]
    pub field_verifier_results: Vec<FieldVerifierResult>,
    #[serde(default)]
    pub next_field_verifier_result_index: u64,
    #[serde(default)]
    pub parallel_evolution_runs: Vec<ParallelEvolutionRun>,
    #[serde(default)]
    pub next_parallel_evolution_run_index: u64,
    #[serde(default)]
    pub check_history: Vec<CheckHistoryRecord>,
    #[serde(default)]
    pub next_check_history_index: u64,
    #[serde(default)]
    pub repair_outcomes: Vec<RepairOutcome>,
    #[serde(default)]
    pub next_repair_outcome_index: u64,
    #[serde(default)]
    pub starter_feedback: Vec<StarterFeedbackRecord>,
    #[serde(default)]
    pub next_starter_feedback_index: u64,
}

impl HarnessState {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let state = serde_json::from_str(&content)
            .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        Ok(state)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self).expect("harness state must serialize");
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("state.json");
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let tmp_path =
            path.with_file_name(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce));
        let write_result = (|| -> Result<(), std::io::Error> {
            let mut file = fs::File::create(&tmp_path)?;
            file.write_all(content.as_bytes())?;
            file.write_all(b"\n")?;
            file.sync_all()?;
            drop(file);
            fs::rename(&tmp_path, path)?;
            if let Some(parent) = path.parent() {
                if let Ok(parent_dir) = fs::File::open(parent) {
                    let _ = parent_dir.sync_all();
                }
            }
            Ok(())
        })();
        if write_result.is_err() {
            let _ = fs::remove_file(&tmp_path);
        }
        write_result
    }

    pub fn install_profile(&mut self, profile_id: &str) -> Result<(), String> {
        let profiles = default_tentacle_profiles();
        if !profiles.iter().any(|profile| profile.id == profile_id) {
            return Err(format!("unknown profile: {profile_id}"));
        }
        if !self
            .installed_profiles
            .iter()
            .any(|installed| installed == profile_id)
        {
            self.installed_profiles.push(profile_id.to_string());
        }
        Ok(())
    }

    pub fn install_manifest(
        &mut self,
        root: impl AsRef<Path>,
        tentacle_id: &str,
    ) -> Result<InstalledTentacle, String> {
        let root = root.as_ref();
        let manifests = load_tentacle_manifests(root).map_err(|error| error.to_string())?;
        let loaded = manifests
            .into_iter()
            .find(|loaded| loaded.manifest.id == tentacle_id)
            .ok_or_else(|| format!("unknown manifest: {tentacle_id}"))?;
        let installed = installed_tentacle_from_manifest(root, loaded)?;
        if let Some(existing) = self
            .installed_tentacles
            .iter_mut()
            .find(|item| item.id == installed.id)
        {
            *existing = installed.clone();
        } else {
            self.installed_tentacles.push(installed.clone());
        }
        Ok(installed)
    }

    pub fn beat(&mut self, memory_keep: usize) -> HeartbeatReport {
        let dropped = self.memory.compact(memory_keep);
        let routes_changed = self.routes.decay(0.995);
        let trace_dropped = self.compact_feed_traces(memory_keep);
        let check_dropped = self.compact_check_history(memory_keep);
        let repair_dropped = self.compact_repair_outcomes(memory_keep);
        let starter_dropped = self.compact_starter_feedback(memory_keep);
        let compacted = trace_dropped + check_dropped + repair_dropped + starter_dropped;
        let harness_summary = if routes_changed > 0 && compacted > 0 {
            format!("evolved {routes_changed} routes, compacted {compacted} harness records")
        } else if compacted > 0 {
            format!("compacted {compacted} harness records")
        } else {
            format!("evolved {routes_changed} routes")
        };
        let report = HeartbeatReport {
            beats: vec![
                HeartBeat {
                    name: "heartbeat".to_string(),
                    changed: true,
                    summary: "alive".to_string(),
                    data: BTreeMap::new(),
                },
                HeartBeat {
                    name: "memory".to_string(),
                    changed: dropped > 0,
                    summary: format!("compacted {dropped} memories"),
                    data: BTreeMap::from([
                        ("dropped".to_string(), dropped.to_string()),
                        ("kept".to_string(), self.memory.len().to_string()),
                    ]),
                },
                HeartBeat {
                    name: "harness".to_string(),
                    changed: routes_changed > 0 || compacted > 0,
                    summary: harness_summary.clone(),
                    data: BTreeMap::from([
                        ("routes".to_string(), self.routes.scores.len().to_string()),
                        (
                            "feed_traces".to_string(),
                            self.feed_traces.len().to_string(),
                        ),
                        ("feed_traces_dropped".to_string(), trace_dropped.to_string()),
                        (
                            "check_history".to_string(),
                            self.check_history.len().to_string(),
                        ),
                        (
                            "check_history_dropped".to_string(),
                            check_dropped.to_string(),
                        ),
                        (
                            "repair_outcomes".to_string(),
                            self.repair_outcomes.len().to_string(),
                        ),
                        (
                            "repair_outcomes_dropped".to_string(),
                            repair_dropped.to_string(),
                        ),
                        (
                            "starter_feedback".to_string(),
                            self.starter_feedback.len().to_string(),
                        ),
                        (
                            "starter_feedback_dropped".to_string(),
                            starter_dropped.to_string(),
                        ),
                    ]),
                },
            ],
        };
        if dropped > 0 {
            self.record_pet_event(
                "memory",
                "memory beat",
                format!("compacted {dropped} memories"),
                Status::Satisfied,
            );
        } else if routes_changed > 0 || compacted > 0 {
            self.record_pet_event(
                "harness",
                "harness beat",
                harness_summary,
                Status::Satisfied,
            );
        } else {
            self.record_pet_event("heartbeat", "heartbeat", "alive", Status::Satisfied);
        }
        report
    }

    pub fn status_report(&self) -> StatusReport {
        self.status_report_with_state(None)
    }

    pub fn context_report(&self, next_need: Option<Need>, limit: usize) -> ContextReport {
        let limit = limit.max(1);
        let turns = self
            .recent_feed_traces(limit)
            .into_iter()
            .map(|trace| BrainContextTurn {
                need: NeedContext {
                    kind: trace.need_kind,
                    query: trace.need_query,
                },
                feed: FeedContext {
                    status: trace.status,
                    summary: trace.summary,
                },
            })
            .collect::<Vec<_>>();
        let tentacles = self
            .installed_tentacles
            .iter()
            .map(|tentacle| self.tentacle_context_report(tentacle, limit))
            .collect::<Vec<_>>();
        let mut agent_next = vec!["octopus chat \"refine your goal\"".to_string()];
        let mut user_goal_hints =
            vec!["Refine the Goal if the latest Feed changes direction.".to_string()];
        if let Some(need) = &next_need {
            user_goal_hints.push(format!(
                "Review the proposed {} Need: {}",
                kind_key(&need.kind),
                one_line(&need.query)
            ));
            agent_next.push(format!(
                "octopus need {} {}",
                kind_key(&need.kind),
                shell_arg(&need.query)
            ));
            if let Some(tentacle) = tentacles.first() {
                agent_next.push(format!(
                    "octopus think {} {} {}",
                    tentacle.id,
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                ));
            }
        } else {
            user_goal_hints.push("Wait for the next Need or refine the Goal.".to_string());
            agent_next.push("octopus context observe .".to_string());
        }
        agent_next.push("octopus traces".to_string());
        ContextReport {
            brain: BrainContextReport {
                policy: CLEAN_BRAIN_CONTEXT_POLICY.to_string(),
                slots: vec![
                    "Goal".to_string(),
                    "Mem".to_string(),
                    "Need".to_string(),
                    "Feed".to_string(),
                ],
                goal: self.goal.clone(),
                mem: self.memory_context_records(limit),
                turns,
                next_need,
            },
            tentacles,
            hearts: self.status_report().hearts,
            agent_next,
            user_goal_hints: user_goal_hints.clone(),
            next: user_goal_hints,
        }
    }

    pub fn clean_brain_explore_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainExploreReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_explore_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_explore_report(brain, prompt, "llm", draft.summary, draft.needs))
    }

    pub fn clean_brain_explore_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainExploreDraft,
    ) -> BrainExploreReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_explore_report(brain, prompt, "external_chat", draft.summary, draft.needs)
    }

    pub fn clean_brain_brief_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainExploreReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_brief_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_brief_report(brain, prompt, "llm_brief", draft.summary, draft.needs))
    }

    pub fn clean_brain_brief_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainExploreDraft,
    ) -> BrainExploreReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_brief_report(brain, prompt, "external_brief", draft.summary, draft.needs)
    }

    pub fn clean_brain_intent_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainExploreReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_intent_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_intent_report(brain, prompt, "llm_intent", draft.summary, draft.needs))
    }

    pub fn clean_brain_intent_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainExploreDraft,
    ) -> BrainExploreReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_intent_report(brain, prompt, "external_intent", draft.summary, draft.needs)
    }

    pub fn clean_brain_focus_with_client<C>(
        &self,
        kind: NeedKind,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainExploreReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_focus_from_chat(&brain, &kind, &prompt, client)?;
        Ok(self.brain_focus_report(
            brain,
            prompt,
            &kind,
            &format!("llm_focus_{}", kind_key(&kind)),
            draft.summary,
            draft.needs,
        ))
    }

    pub fn clean_brain_focus_from_draft(
        &self,
        kind: NeedKind,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainExploreDraft,
    ) -> BrainExploreReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_focus_report(
            brain,
            prompt,
            &kind,
            &format!("external_focus_{}", kind_key(&kind)),
            draft.summary,
            draft.needs,
        )
    }

    pub fn clean_brain_memory_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainExploreReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_memory_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_memory_report(brain, prompt, "llm_memory", draft.summary, draft.needs))
    }

    pub fn clean_brain_memory_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainExploreDraft,
    ) -> BrainExploreReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_memory_report(brain, prompt, "external_memory", draft.summary, draft.needs)
    }

    pub fn clean_brain_clarify_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainDeliberationReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_clarification_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_clarification_report(brain, prompt, "llm_clarify", draft))
    }

    pub fn clean_brain_clarify_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainDeliberationDraft,
    ) -> BrainDeliberationReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_clarification_report(brain, prompt, "external_clarify", draft)
    }

    pub fn clean_brain_agenda_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainDeliberationReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_agenda_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_agenda_report(brain, prompt, "llm_agenda", draft))
    }

    pub fn clean_brain_agenda_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainDeliberationDraft,
    ) -> BrainDeliberationReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_agenda_report(brain, prompt, "external_agenda", draft)
    }

    pub fn clean_brain_scout_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainDeliberationReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_scout_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_scout_report(brain, prompt, "llm_scout", draft))
    }

    pub fn clean_brain_scout_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainDeliberationDraft,
    ) -> BrainDeliberationReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_scout_report(brain, prompt, "external_scout", draft)
    }

    pub fn clean_brain_deliberate_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainDeliberationReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_deliberation_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_deliberation_report(brain, prompt, "llm_deliberation", draft))
    }

    pub fn clean_brain_deliberate_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainDeliberationDraft,
    ) -> BrainDeliberationReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_deliberation_report(brain, prompt, "external_deliberation", draft)
    }

    pub fn clean_brain_reflect_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainReflectionReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_reflection_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_reflection_report(brain, prompt, "llm_reflection", draft))
    }

    pub fn clean_brain_reflect_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainReflectionDraft,
    ) -> BrainReflectionReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_reflection_report(brain, prompt, "external_reflection", draft)
    }

    pub fn clean_brain_align_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainReflectionReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_alignment_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_alignment_report(brain, prompt, "llm_align", draft))
    }

    pub fn clean_brain_align_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainReflectionDraft,
    ) -> BrainReflectionReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_alignment_report(brain, prompt, "external_align", draft)
    }

    pub fn clean_brain_synthesize_from_input(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        input: BrainSynthesisInput,
    ) -> BrainSynthesisReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let (draft_count, summary, observations, questions, options, risks, needs) =
            merge_brain_synthesis_input(input);
        self.brain_synthesis_report(
            brain,
            prompt,
            "external_synthesis",
            draft_count,
            summary,
            observations,
            questions,
            options,
            risks,
            needs,
        )
    }

    pub fn clean_brain_synthesize_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        input: BrainSynthesisInput,
        client: &mut C,
    ) -> Result<BrainSynthesisReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft_count = brain_synthesis_draft_count(&input);
        let synthesis = brain_synthesis_from_chat(&brain, &prompt, &input, client)?;
        let (merged_count, summary, observations, questions, options, risks, needs) =
            merge_brain_synthesis_input(synthesis);
        Ok(self.brain_synthesis_report(
            brain,
            prompt,
            "llm_synthesis",
            draft_count.max(merged_count),
            summary,
            observations,
            questions,
            options,
            risks,
            needs,
        ))
    }

    pub fn clean_brain_rewrite_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainExploreDraft,
    ) -> BrainExploreReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let raw_audit = audit_clean_brain_needs(&draft.needs);
        let summary = if raw_audit.issue_count == 0 {
            draft.summary
        } else {
            format!(
                "rewrite review accepted {} clean Need(s); {} polluted Need(s) require live rewrite",
                raw_audit.clean_count, raw_audit.issue_count
            )
        };
        self.brain_explore_report(
            brain,
            prompt,
            "rewrite_review",
            summary,
            raw_audit.clean_needs,
        )
    }

    pub fn clean_brain_rewrite_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainExploreDraft,
        client: &mut C,
    ) -> Result<BrainExploreReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let raw_audit = audit_clean_brain_needs(&draft.needs);
        if raw_audit.issue_count == 0 {
            return Ok(self.brain_explore_report(
                brain,
                prompt,
                "rewrite_clean",
                draft.summary,
                draft.needs,
            ));
        }
        let rewrite = brain_rewrite_from_chat(&brain, &prompt, &draft, &raw_audit, client)?;
        Ok(self.brain_explore_report(brain, prompt, "llm_rewrite", rewrite.summary, rewrite.needs))
    }

    pub fn clean_brain_goal_with_client<C>(
        &mut self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainGoalReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let refinement = brain_goal_from_chat(&brain, &prompt, client)?;
        Ok(self.apply_clean_brain_goal(brain, prompt, "llm", refinement))
    }

    pub fn clean_brain_goal_from_refinement(
        &mut self,
        prompt: impl Into<String>,
        limit: usize,
        refinement: GoalRefinement,
    ) -> BrainGoalReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.apply_clean_brain_goal(brain, prompt, "external_chat", refinement)
    }

    pub fn clean_brain_prompt(&self, prompt: impl Into<String>, limit: usize) -> BrainPromptReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let context = serde_json::json!({
            "policy": brain.policy,
            "slots": brain.slots,
            "goal": brain.goal,
            "mem": brain.mem,
            "recent_need_feed": brain.turns,
        });
        let context_text = serde_json::to_string_pretty(&context)
            .unwrap_or_else(|_| "{\"policy\":\"Goal + Mem + Need + Feed\"}".to_string());
        let system = "You are the Octopus clean brain. Use only Goal, Mem, Need, and Feed. Express cognitive Needs only. Return JSON with {\"summary\":\"short\",\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}."
            .to_string();
        let user =
            format!("Clean brain prompt: {prompt}\nClean brain context JSON:\n{context_text}");
        let messages = vec![
            ChatMessage::new(ChatRole::System, system),
            ChatMessage::new(ChatRole::User, user),
        ];
        let prompt_text = messages
            .iter()
            .map(|message| format!("{:?}:\n{}", message.role, message.content))
            .collect::<Vec<_>>()
            .join("\n\n");
        let next = vec![
            "paste messages into any chat-completions-compatible model".to_string(),
            format!("octopus explore {}", shell_arg(&prompt)),
            "octopus context observe .".to_string(),
        ];
        BrainPromptReport {
            policy: brain.policy,
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            messages,
            prompt_text,
            next,
        }
    }

    fn apply_clean_brain_goal(
        &mut self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        refinement: GoalRefinement,
    ) -> BrainGoalReport {
        let previous_goal = self.goal.clone();
        let objective = clean_optional(refinement.objective.as_deref())
            .or_else(|| previous_goal.as_ref().map(|goal| goal.objective.as_str()))
            .or_else(|| clean_optional(Some(&prompt)))
            .unwrap_or("clean-brain goal")
            .to_string();
        let mut goal = previous_goal
            .clone()
            .unwrap_or_else(|| Goal::new(objective.clone()));
        goal.objective = objective;
        for constraint in &refinement.constraints {
            if let Some(constraint) = clean_optional(Some(constraint.as_str())) {
                if !goal.constraints.iter().any(|item| item == constraint) {
                    goal.refine(constraint.to_string());
                }
            }
        }
        goal.signals
            .insert("brain_goal_source".to_string(), source.to_string());
        let audit = audit_clean_brain_needs(&refinement.needs);
        if !audit.clean_needs.is_empty() {
            let suggested = audit
                .clean_needs
                .iter()
                .map(|need| format!("{}: {}", kind_key(&need.kind), need.query))
                .collect::<Vec<_>>()
                .join(" | ");
            goal.signals
                .insert("suggested_needs".to_string(), suggested);
        }
        let summary = clean_optional(refinement.summary.as_deref())
            .map(str::to_string)
            .unwrap_or_else(|| "goal refined by clean brain".to_string());
        let turn = GoalTurn {
            index: self.goal_turns.len() as u64 + 1,
            message: prompt.clone(),
            summary: summary.clone(),
            status: Status::Satisfied,
        };
        self.goal = Some(goal.clone());
        self.goal_turns.push(turn);
        let next = if audit.clean_needs.is_empty() {
            let mut next = vec![
                "octopus brain --live \"what should the brain ask next?\"".to_string(),
                "octopus context".to_string(),
            ];
            if audit.issue_count > 0 {
                next.insert(
                    0,
                    "revise Need suggestions as cognitive requests before Feed".to_string(),
                );
            }
            next
        } else {
            let mut next = audit
                .clean_needs
                .iter()
                .map(|need| {
                    format!(
                        "octopus need {} {}",
                        kind_key(&need.kind),
                        shell_arg(&need.query)
                    )
                })
                .collect::<Vec<_>>();
            next.push(format!(
                "octopus brain --goal --save {}",
                shell_arg(&prompt)
            ));
            next
        };
        BrainGoalReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            previous_goal,
            goal,
            summary,
            audit,
            needs: refinement.needs,
            next,
        }
    }

    fn brain_explore_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        summary: impl Into<String>,
        needs: Vec<GoalNeedSuggestion>,
    ) -> BrainExploreReport {
        let audit = audit_clean_brain_needs(&needs);
        let next = if audit.clean_needs.is_empty() {
            if audit.issue_count > 0 {
                vec![
                    "revise Need suggestions as cognitive requests before Feed".to_string(),
                    "octopus brain --session \"rewrite these Needs cleanly\"".to_string(),
                ]
            } else {
                vec!["octopus goal set \"describe your goal\"".to_string()]
            }
        } else {
            audit
                .clean_needs
                .iter()
                .map(|need| {
                    format!(
                        "octopus need {} {}",
                        kind_key(&need.kind),
                        shell_arg(&need.query)
                    )
                })
                .collect()
        };
        BrainExploreReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            summary: summary.into(),
            needs,
            audit,
            next,
        }
    }

    fn brain_intent_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        summary: impl Into<String>,
        needs: Vec<GoalNeedSuggestion>,
    ) -> BrainExploreReport {
        let mut report = self.brain_explore_report(brain, prompt.clone(), source, summary, needs);
        report
            .next
            .push("octopus brain --intent --session".to_string());
        if report.audit.clean_needs.is_empty() {
            if report.audit.issue_count == 0 {
                report
                    .next
                    .push(format!("octopus brain --intent {}", shell_arg(&prompt)));
            }
        } else {
            report.next.push(format!(
                "octopus brain --intent --save {}",
                shell_arg(&prompt)
            ));
        }
        report.next.sort();
        report.next.dedup();
        report
    }

    fn brain_brief_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        summary: impl Into<String>,
        needs: Vec<GoalNeedSuggestion>,
    ) -> BrainExploreReport {
        let mut report = self.brain_explore_report(brain, prompt.clone(), source, summary, needs);
        report
            .next
            .push("octopus brain --brief --session".to_string());
        if report.audit.clean_needs.is_empty() {
            if report.audit.issue_count == 0 {
                report
                    .next
                    .push(format!("octopus brain --brief {}", shell_arg(&prompt)));
            }
        } else {
            report.next.push(format!(
                "octopus brain --brief --save {}",
                shell_arg(&prompt)
            ));
        }
        report.next.sort();
        report.next.dedup();
        report
    }

    fn brain_memory_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        summary: impl Into<String>,
        needs: Vec<GoalNeedSuggestion>,
    ) -> BrainExploreReport {
        let mut report = self.brain_explore_report(brain, prompt.clone(), source, summary, needs);
        report
            .next
            .push("octopus brain --memory --session".to_string());
        if report.audit.clean_needs.is_empty() {
            if report.audit.issue_count == 0 {
                report
                    .next
                    .push("octopus brain --memory \"what should be remembered?\"".to_string());
            }
        } else {
            report.next.push(format!(
                "octopus brain --memory --save {}",
                shell_arg(&prompt)
            ));
        }
        report.next.sort();
        report.next.dedup();
        report
    }

    fn brain_focus_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        kind: &NeedKind,
        source: &str,
        summary: impl Into<String>,
        needs: Vec<GoalNeedSuggestion>,
    ) -> BrainExploreReport {
        let label = kind_key(kind);
        let mut report = self.brain_explore_report(brain, prompt.clone(), source, summary, needs);
        report
            .next
            .push(format!("octopus brain --focus {label} --session"));
        if report.audit.clean_needs.is_empty() {
            if report.audit.issue_count == 0 {
                report.next.push(format!(
                    "octopus brain --focus {label} {}",
                    shell_arg(&prompt)
                ));
            }
        } else {
            report.next.push(format!(
                "octopus brain --focus {label} --save {}",
                shell_arg(&prompt)
            ));
        }
        report.next.sort();
        report.next.dedup();
        report
    }

    fn brain_deliberation_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        draft: BrainDeliberationDraft,
    ) -> BrainDeliberationReport {
        let audit = audit_clean_brain_needs(&draft.needs);
        let mut next = vec!["octopus brain --deliberate --session".to_string()];
        if audit.clean_needs.is_empty() {
            if audit.issue_count > 0 {
                next.push(
                    "rewrite deliberation Needs as cognitive requests before Feed".to_string(),
                );
            } else {
                next.push(
                    "octopus brain --deliberate \"what should the brain examine next?\""
                        .to_string(),
                );
            }
        } else {
            next.extend(audit.clean_needs.iter().map(|need| {
                format!(
                    "octopus need {} {}",
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                )
            }));
            next.push(format!(
                "octopus brain --deliberate --save {}",
                shell_arg(&prompt)
            ));
        }
        next.sort();
        next.dedup();
        BrainDeliberationReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            summary: draft.summary,
            observations: draft.observations,
            questions: draft.questions,
            options: draft.options,
            risks: draft.risks,
            needs: draft.needs,
            audit,
            next,
        }
    }

    fn brain_clarification_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        draft: BrainDeliberationDraft,
    ) -> BrainDeliberationReport {
        let audit = audit_clean_brain_needs(&draft.needs);
        let mut next = vec!["octopus brain --clarify --session".to_string()];
        if audit.clean_needs.is_empty() {
            if audit.issue_count > 0 {
                next.push(
                    "rewrite clarification Needs as cognitive requests before Feed".to_string(),
                );
            } else {
                next.push(format!("octopus goal set {}", shell_arg(&prompt)));
            }
        } else {
            next.extend(audit.clean_needs.iter().map(|need| {
                format!(
                    "octopus need {} {}",
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                )
            }));
            next.push(format!(
                "octopus brain --clarify --save {}",
                shell_arg(&prompt)
            ));
        }
        next.sort();
        next.dedup();
        BrainDeliberationReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            summary: draft.summary,
            observations: draft.observations,
            questions: draft.questions,
            options: draft.options,
            risks: draft.risks,
            needs: draft.needs,
            audit,
            next,
        }
    }

    fn brain_agenda_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        draft: BrainDeliberationDraft,
    ) -> BrainDeliberationReport {
        let audit = audit_clean_brain_needs(&draft.needs);
        let mut next = vec!["octopus brain --agenda --session".to_string()];
        if audit.clean_needs.is_empty() {
            if audit.issue_count > 0 {
                next.push("rewrite agenda Needs as cognitive requests before Feed".to_string());
            } else {
                next.push("octopus brain --clarify \"what should the user clarify?\"".to_string());
            }
        } else {
            next.extend(audit.clean_needs.iter().map(|need| {
                format!(
                    "octopus need {} {}",
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                )
            }));
            next.push(format!(
                "octopus brain --agenda --save {}",
                shell_arg(&prompt)
            ));
        }
        next.sort();
        next.dedup();
        BrainDeliberationReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            summary: draft.summary,
            observations: draft.observations,
            questions: draft.questions,
            options: draft.options,
            risks: draft.risks,
            needs: draft.needs,
            audit,
            next,
        }
    }

    fn brain_scout_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        draft: BrainDeliberationDraft,
    ) -> BrainDeliberationReport {
        let audit = audit_clean_brain_needs(&draft.needs);
        let mut next = vec!["octopus brain --scout --session".to_string()];
        if audit.clean_needs.is_empty() {
            if audit.issue_count > 0 {
                next.push("rewrite scout Needs as cognitive requests before Feed".to_string());
            } else {
                next.push("octopus brain --scout \"what should the brain map next?\"".to_string());
            }
        } else {
            next.extend(audit.clean_needs.iter().map(|need| {
                format!(
                    "octopus need {} {}",
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                )
            }));
            next.push(format!(
                "octopus brain --scout --save {}",
                shell_arg(&prompt)
            ));
        }
        next.sort();
        next.dedup();
        BrainDeliberationReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            summary: draft.summary,
            observations: draft.observations,
            questions: draft.questions,
            options: draft.options,
            risks: draft.risks,
            needs: draft.needs,
            audit,
            next,
        }
    }

    fn brain_reflection_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        draft: BrainReflectionDraft,
    ) -> BrainReflectionReport {
        let audit = audit_clean_brain_needs(&draft.needs);
        let mut next = vec!["octopus brain --reflect --session".to_string()];
        if audit.clean_needs.is_empty() {
            if audit.issue_count > 0 {
                next.push("rewrite reflection Needs as cognitive requests before Feed".to_string());
            } else {
                next.push("octopus brain --reflect \"what goal evidence is missing?\"".to_string());
            }
        } else {
            next.extend(audit.clean_needs.iter().map(|need| {
                format!(
                    "octopus need {} {}",
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                )
            }));
            next.push(format!(
                "octopus brain --reflect --save {}",
                shell_arg(&prompt)
            ));
        }
        next.sort();
        next.dedup();
        BrainReflectionReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            summary: draft.summary,
            goal_state: draft.goal_state,
            evidence: draft.evidence,
            gaps: draft.gaps,
            questions: draft.questions,
            needs: draft.needs,
            audit,
            next,
        }
    }

    fn brain_alignment_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        draft: BrainReflectionDraft,
    ) -> BrainReflectionReport {
        let audit = audit_clean_brain_needs(&draft.needs);
        let mut next = vec!["octopus brain --align --session".to_string()];
        if audit.clean_needs.is_empty() {
            if audit.issue_count > 0 {
                next.push("rewrite alignment Needs as cognitive requests before Feed".to_string());
            } else {
                next.push("octopus brain --align \"does this still follow the goal?\"".to_string());
            }
        } else {
            next.extend(audit.clean_needs.iter().map(|need| {
                format!(
                    "octopus need {} {}",
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                )
            }));
            next.push(format!(
                "octopus brain --align --save {}",
                shell_arg(&prompt)
            ));
        }
        next.sort();
        next.dedup();
        BrainReflectionReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            summary: draft.summary,
            goal_state: draft.goal_state,
            evidence: draft.evidence,
            gaps: draft.gaps,
            questions: draft.questions,
            needs: draft.needs,
            audit,
            next,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn brain_synthesis_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        draft_count: usize,
        summary: String,
        observations: Vec<String>,
        questions: Vec<String>,
        options: Vec<String>,
        risks: Vec<String>,
        needs: Vec<GoalNeedSuggestion>,
    ) -> BrainSynthesisReport {
        let audit = audit_clean_brain_needs(&needs);
        let mut next = vec!["octopus brain --synthesize --session".to_string()];
        if audit.clean_needs.is_empty() {
            if audit.issue_count > 0 {
                next.push("rewrite synthesis Needs as cognitive requests before Feed".to_string());
            } else {
                next.push("octopus brain --synthesize --session --apply <drafts.json>".to_string());
            }
        } else {
            next.extend(audit.clean_needs.iter().map(|need| {
                format!(
                    "octopus need {} {}",
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                )
            }));
            next.push("octopus needs script".to_string());
        }
        next.sort();
        next.dedup();
        BrainSynthesisReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            summary,
            draft_count,
            observations,
            questions,
            options,
            risks,
            needs,
            audit,
            next,
        }
    }

    pub fn queue_goal_report(&mut self, report: &BrainGoalReport) -> BrainGoalSaveReport {
        let mut queued = Vec::new();
        for need in &report.audit.clean_needs {
            if let Some(existing) = self
                .need_queue
                .iter()
                .find(|item| item.status == NeedQueueStatus::Pending && item.need == *need)
            {
                queued.push(existing.clone());
                continue;
            }
            self.next_need_queue_index += 1;
            let item = NeedQueueItem {
                index: self.next_need_queue_index,
                need: need.clone(),
                context: BTreeMap::new(),
                source: report.source.clone(),
                prompt: report.prompt.clone(),
                summary: report.summary.clone(),
                status: NeedQueueStatus::Pending,
            };
            self.need_queue.push(item.clone());
            queued.push(item);
        }
        BrainGoalSaveReport {
            goal: report.clone(),
            queued,
            queue: self.need_queue_report(8),
        }
    }

    pub fn queue_exploration_report(&mut self, report: &BrainExploreReport) -> NeedQueueSaveReport {
        let mut queued = Vec::new();
        for need in &report.audit.clean_needs {
            queued.push(self.queue_need_suggestion(
                need.clone(),
                report.source.clone(),
                report.prompt.clone(),
                report.summary.clone(),
            ));
        }
        NeedQueueSaveReport {
            explore: report.clone(),
            queued,
            queue: self.need_queue_report(8),
        }
    }

    pub fn queue_deliberation_report(
        &mut self,
        report: &BrainDeliberationReport,
    ) -> BrainDeliberationSaveReport {
        let mut queued = Vec::new();
        for need in &report.audit.clean_needs {
            queued.push(self.queue_need_suggestion(
                need.clone(),
                report.source.clone(),
                report.prompt.clone(),
                report.summary.clone(),
            ));
        }
        BrainDeliberationSaveReport {
            deliberation: report.clone(),
            queued,
            queue: self.need_queue_report(8),
        }
    }

    pub fn queue_reflection_report(
        &mut self,
        report: &BrainReflectionReport,
    ) -> BrainReflectionSaveReport {
        let mut queued = Vec::new();
        for need in &report.audit.clean_needs {
            queued.push(self.queue_need_suggestion(
                need.clone(),
                report.source.clone(),
                report.prompt.clone(),
                report.summary.clone(),
            ));
        }
        BrainReflectionSaveReport {
            reflection: report.clone(),
            queued,
            queue: self.need_queue_report(8),
        }
    }

    pub fn queue_synthesis_report(
        &mut self,
        report: &BrainSynthesisReport,
    ) -> BrainSynthesisSaveReport {
        let mut queued = Vec::new();
        for need in &report.audit.clean_needs {
            queued.push(self.queue_need_suggestion(
                need.clone(),
                report.source.clone(),
                report.prompt.clone(),
                report.summary.clone(),
            ));
        }
        BrainSynthesisSaveReport {
            synthesis: report.clone(),
            queued,
            queue: self.need_queue_report(8),
        }
    }

    pub fn queue_need_suggestion(
        &mut self,
        need: GoalNeedSuggestion,
        source: impl Into<String>,
        prompt: impl Into<String>,
        summary: impl Into<String>,
    ) -> NeedQueueItem {
        self.queue_need_suggestion_with_context(need, BTreeMap::new(), source, prompt, summary)
    }

    pub fn queue_need_suggestion_with_context(
        &mut self,
        need: GoalNeedSuggestion,
        context: BTreeMap<String, String>,
        source: impl Into<String>,
        prompt: impl Into<String>,
        summary: impl Into<String>,
    ) -> NeedQueueItem {
        if let Some(existing) = self.need_queue.iter().find(|item| {
            item.status == NeedQueueStatus::Pending && item.need == need && item.context == context
        }) {
            return existing.clone();
        }
        self.next_need_queue_index += 1;
        let item = NeedQueueItem {
            index: self.next_need_queue_index,
            need,
            context,
            source: source.into(),
            prompt: prompt.into(),
            summary: summary.into(),
            status: NeedQueueStatus::Pending,
        };
        self.need_queue.push(item.clone());
        item
    }

    pub fn need_queue_report(&self, limit: usize) -> NeedQueueReport {
        let limit = limit.max(1);
        let pending = self
            .need_queue
            .iter()
            .filter(|item| item.status == NeedQueueStatus::Pending)
            .cloned()
            .collect::<Vec<_>>();
        let mut history = self
            .need_queue
            .iter()
            .filter(|item| item.status != NeedQueueStatus::Pending)
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>();
        history.reverse();
        let agent_next = if let Some(item) = pending.first() {
            vec![
                format!("octopus needs run {}", item.index),
                format!("octopus needs take {}", item.index),
                format!("octopus needs drop {}", item.index),
                format!("octopus {}", need_command(&item.need)),
            ]
        } else {
            vec![
                "octopus explore --save \"what should the brain ask next?\"".to_string(),
                "octopus goal set \"describe your goal\"".to_string(),
            ]
        };
        let user_goal_hints = user_surface::need_queue_hints(&pending);
        NeedQueueReport {
            policy: CLEAN_BRAIN_CONTEXT_POLICY.to_string(),
            pending,
            history,
            agent_next,
            user_goal_hints: user_goal_hints.clone(),
            next: user_goal_hints,
        }
    }

    pub fn take_queued_need(&mut self, index: u64) -> Result<NeedQueueTakeReport, String> {
        let item = self
            .need_queue
            .iter_mut()
            .find(|item| item.index == index)
            .ok_or_else(|| format!("unknown queued Need: {index}"))?;
        if item.status != NeedQueueStatus::Pending {
            return Err(format!(
                "queued Need is already {}",
                need_queue_status_key(&item.status)
            ));
        }
        item.status = NeedQueueStatus::Taken;
        let item = item.clone();
        let command = format!("octopus {}", need_command(&item.need));
        Ok(NeedQueueTakeReport {
            item,
            command: command.clone(),
            next: vec![command, "octopus needs".to_string()],
        })
    }

    pub fn drop_queued_need(&mut self, index: u64) -> Result<NeedQueueItem, String> {
        let item = self
            .need_queue
            .iter_mut()
            .find(|item| item.index == index)
            .ok_or_else(|| format!("unknown queued Need: {index}"))?;
        if item.status != NeedQueueStatus::Pending {
            return Err(format!(
                "queued Need is already {}",
                need_queue_status_key(&item.status)
            ));
        }
        item.status = NeedQueueStatus::Dropped;
        Ok(item.clone())
    }

    pub fn pending_need_queue_count(&self) -> usize {
        self.need_queue
            .iter()
            .filter(|item| item.status == NeedQueueStatus::Pending)
            .count()
    }

    pub fn status_report_with_state(&self, state_path: Option<&Path>) -> StatusReport {
        let tentacles = self
            .installed_tentacles
            .iter()
            .map(|tentacle| TentacleStatus {
                id: tentacle.id.clone(),
                name: tentacle.name.clone(),
                brain_kind: tentacle.brain_kind.clone(),
                runtime_kinds: tentacle.runtime_kinds.clone(),
                needs: tentacle.needs.clone(),
                tool_count: tentacle.tool_meta.len().max(tentacle.tools.len()),
                editable: tentacle.editable.clone(),
                evolution_surfaces: tentacle.evolution_surfaces.clone(),
            })
            .collect::<Vec<_>>();
        let active_grants = self
            .grants
            .iter()
            .filter(|grant| grant.status == GrantStatus::Active)
            .map(|grant| grant.id.clone())
            .collect::<Vec<_>>();
        let goal = self.goal.as_ref().map(|goal| GoalSnapshot {
            objective: goal.objective.clone(),
            refinements: goal.constraints.len(),
            status: goal.status.clone(),
            turns: {
                let mut turns = self
                    .goal_turns
                    .iter()
                    .rev()
                    .take(6)
                    .cloned()
                    .collect::<Vec<_>>();
                turns.reverse();
                turns
            },
        });
        let mut warnings = Vec::new();
        if self.installed_tentacles.is_empty() {
            warnings.push("no installed tentacle manifests".to_string());
        }
        if self.goal.is_none() {
            warnings.push("no active goal".to_string());
        }
        if active_grants.is_empty() {
            warnings.push("no active OAuth grants".to_string());
        }
        let pending_need_queue_count = self.pending_need_queue_count();
        let state_args = state_path
            .map(|path| format!(" --state {}", shell_arg(&path.to_string_lossy())))
            .unwrap_or_default();
        let harness_learning = self.harness_learning_summary(&state_args);
        let field_pool = self.field_pool_status_report(state_path);
        let completed_field_pool_next_action = field_pool.as_ref().and_then(|pool| {
            if pool.field_slot_count > 0 && pool.completed_fields == pool.field_slot_count {
                pool.agent_next
                    .iter()
                    .find(|action| action.contains("evolve recommend field-mini-task"))
                    .cloned()
                    .or_else(|| pool.agent_next.last().cloned())
            } else {
                None
            }
        });
        let pending_need = self
            .need_queue
            .iter()
            .find(|item| item.status == NeedQueueStatus::Pending);
        let has_active_parallel_field_goal = self.has_active_parallel_field_goal();
        let agent_next_action = if self.installed_tentacles.is_empty() {
            format!("octopus{state_args} adapt")
        } else if self.goal.is_none() {
            format!("octopus{state_args} chat \"describe your goal\"")
        } else if let Some(item) = pending_need {
            format!("octopus{state_args} needs run {}", item.index)
        } else if let Some(result) = self.latest_unsatisfied_field_verifier_result() {
            let tentacle = self
                .feed_traces
                .iter()
                .rev()
                .find(|trace| trace.index == result.trace_index)
                .and_then(|trace| trace.tentacle.clone())
                .unwrap_or_else(|| format!("field:{}", result.field));
            let reason = result
                .error_category
                .as_deref()
                .unwrap_or("field verifier gap");
            format!(
                "octopus{state_args} evolve recommend {} {}",
                shell_arg(&tentacle),
                shell_arg(&format!("improve {} harness after {reason}", result.field))
            )
        } else if let Some(action) = completed_field_pool_next_action {
            action
        } else if self.routes.scores.is_empty() {
            format!("octopus{state_args} need observe .")
        } else if self.has_active_parallel_field_goal() {
            format!(
                "octopus{state_args} evolve parallel --workers 1 {}",
                shell_arg("peer field objectives; open one worker slot from the peer field pool")
            )
        } else if harness_learning.source != "none" {
            harness_learning.next_action.clone()
        } else {
            format!("octopus{state_args} beat 200")
        };
        let user_goal_hint = user_surface::goal_hint(
            !self.installed_tentacles.is_empty(),
            self.goal.is_some(),
            pending_need,
            has_active_parallel_field_goal,
            &harness_learning,
        );
        StatusReport {
            hearts: vec![
                HeartBeat {
                    name: "heartbeat".to_string(),
                    changed: false,
                    summary: "ready".to_string(),
                    data: BTreeMap::new(),
                },
                HeartBeat {
                    name: "memory".to_string(),
                    changed: false,
                    summary: format!("{} memories", self.memory.len()),
                    data: BTreeMap::from([("records".to_string(), self.memory.len().to_string())]),
                },
                HeartBeat {
                    name: "harness".to_string(),
                    changed: false,
                    summary: format!("{} routes", self.routes.scores.len()),
                    data: BTreeMap::from([(
                        "routes".to_string(),
                        self.routes.scores.len().to_string(),
                    )]),
                },
            ],
            memory_count: self.memory.len(),
            route_count: self.routes.scores.len(),
            need_queue_count: pending_need_queue_count,
            feed_trace_count: self.feed_traces.len(),
            field_verifier_result_count: self.field_verifier_results.len(),
            parallel_evolution_run_count: self.parallel_evolution_runs.len(),
            check_history_count: self.check_history.len(),
            repair_outcome_count: self.repair_outcomes.len(),
            evolution_outcome_count: self.evolution_outcomes.len(),
            starter_feedback_count: self.starter_feedback.len(),
            installed_profiles: self.installed_profiles.clone(),
            tentacles,
            goal,
            active_grants,
            last_pet_event: self.last_pet_event.clone(),
            latest_feed_trace: self.feed_traces.last().cloned(),
            latest_field_verifier_result: self.field_verifier_results.last().cloned(),
            latest_parallel_evolution_run: self.parallel_evolution_runs.last().cloned(),
            field_pool,
            latest_need_queue_item: self.need_queue.last().cloned(),
            latest_check: self.check_history.last().cloned(),
            latest_repair_outcome: self.repair_outcomes.last().cloned(),
            latest_evolution_outcome: self.evolution_outcomes.last().cloned(),
            harness_learning,
            latest_starter_feedback: self.starter_feedback.last().cloned(),
            warnings,
            agent_next_action,
            user_goal_hint: user_goal_hint.clone(),
            next_action: user_goal_hint,
        }
    }

    pub fn field_trajectory_report(&self) -> Result<FieldTrajectoryReport, String> {
        self.field_trajectory_report_with_state(None)
    }

    pub fn field_trajectory_report_with_state(
        &self,
        state_path: Option<&Path>,
    ) -> Result<FieldTrajectoryReport, String> {
        let state_args = state_path
            .map(|path| format!(" --state {}", shell_arg(&path.to_string_lossy())))
            .unwrap_or_default();
        let catalog = default_field_pack_catalog()?;
        let mut fields = catalog
            .packs
            .iter()
            .into_iter()
            .map(|pack| pack.id.clone())
            .collect::<Vec<_>>();
        for field in self
            .feed_traces
            .iter()
            .filter_map(|trace| trace_field(trace))
            .chain(
                self.field_verifier_results
                    .iter()
                    .map(|result| result.field.clone()),
            )
            .chain(
                self.parallel_evolution_runs
                    .iter()
                    .flat_map(|run| run.workers.iter().map(|worker| worker.field.clone())),
            )
        {
            push_unique_limited(&mut fields, field, usize::MAX);
        }

        let mut summaries = Vec::new();
        for field in fields {
            let traces = self
                .feed_traces
                .iter()
                .filter(|trace| trace_field(trace).as_deref() == Some(field.as_str()))
                .collect::<Vec<_>>();
            let verifiers = self
                .field_verifier_results
                .iter()
                .filter(|result| result.field == field)
                .collect::<Vec<_>>();
            let latest_verifier = verifiers.last().copied();
            let latest_trace = latest_verifier
                .and_then(|result| {
                    self.feed_traces
                        .iter()
                        .find(|trace| trace.index == result.trace_index)
                })
                .or_else(|| traces.last().copied());
            let latest_parallel_run = self.parallel_evolution_runs.iter().rev().find(|run| {
                run.workers
                    .iter()
                    .any(|worker| worker.field.as_str() == field.as_str())
            });
            let satisfied_verifier_count = verifiers
                .iter()
                .filter(|result| result.status == Status::Satisfied)
                .count();
            let unsatisfied_verifier_count =
                verifiers.len().saturating_sub(satisfied_verifier_count);
            let pack = catalog.packs.iter().find(|pack| pack.id == field);
            let mini_task_count = pack.map(|pack| pack.mini_tasks.len()).unwrap_or_default();
            let satisfied_mini_task_count = pack
                .map(|pack| {
                    pack.mini_tasks
                        .iter()
                        .filter(|task| {
                            self.field_mini_task_status(&field, &task.id) == Some(Status::Satisfied)
                        })
                        .count()
                })
                .unwrap_or_default();
            let selected_mini_task = pack.and_then(|pack| self.next_field_mini_task(pack));
            let next_mini_task = selected_mini_task.map(|task| task.id.clone());
            let next_mini_task_goal = selected_mini_task.map(|task| task.goal.clone());
            let latest_mini_task = latest_trace.and_then(|trace| {
                trace
                    .metadata
                    .get("field_mini_task")
                    .filter(|value| !value.trim().is_empty())
                    .cloned()
            });
            let latest_pass_evidence = latest_trace.and_then(|trace| {
                trace
                    .metadata
                    .get("field_pass_evidence")
                    .filter(|value| !value.trim().is_empty())
                    .cloned()
            });
            let latest_error_category = latest_verifier
                .and_then(|result| result.error_category.clone())
                .or_else(|| {
                    latest_trace.and_then(|trace| {
                        trace
                            .metadata
                            .get("error_category")
                            .filter(|value| !value.trim().is_empty())
                            .cloned()
                    })
                });
            let latest_summary = latest_verifier
                .map(|result| result.summary.clone())
                .or_else(|| latest_trace.map(|trace| trace.summary.clone()));
            let latest_verifier_status = latest_verifier.map(|result| result.status.clone());
            let needs_repair = latest_verifier.is_some_and(|result| {
                result.status != Status::Satisfied
                    && self.field_verifier_result_has_real_mini_task(result)
            });
            let ready_for_harder_task = latest_verifier_status == Some(Status::Satisfied);
            let has_pending_mini_task = next_mini_task.as_ref().is_some_and(|task| {
                self.field_mini_task_status(&field, task) != Some(Status::Satisfied)
            });
            let field_pack_layer_complete =
                mini_task_count > 0 && satisfied_mini_task_count == mini_task_count;
            let next_action = if needs_repair {
                let tentacle = latest_trace
                    .and_then(|trace| trace.tentacle.clone())
                    .unwrap_or_else(|| "field-mini-task".to_string());
                let reason = latest_error_category
                    .as_deref()
                    .unwrap_or("field verifier gap");
                format!(
                    "octopus{state_args} evolve recommend {} {}",
                    shell_arg(&tentacle),
                    shell_arg(&format!("improve {field} harness after {reason}"))
                )
            } else if field_pack_layer_complete {
                field_harder_layer_next_action(&state_args)
            } else if ready_for_harder_task || has_pending_mini_task {
                format!(
                    "octopus{state_args} evolve parallel --workers 1 {}",
                    shell_arg(
                        "peer field objectives; open one harder-task worker slot from the peer field pool"
                    )
                )
            } else {
                format!(
                    "octopus{state_args} evolve parallel --workers 1 {}",
                    shell_arg(
                        "peer field objectives; open one worker slot from the peer field pool"
                    )
                )
            };
            summaries.push(FieldTrajectorySummary {
                field,
                mini_task_count,
                satisfied_mini_task_count,
                next_mini_task,
                next_mini_task_goal,
                trace_count: traces.len(),
                verifier_result_count: verifiers.len(),
                satisfied_verifier_count,
                unsatisfied_verifier_count,
                latest_trace_index: latest_trace.map(|trace| trace.index),
                latest_trace_status: latest_trace.map(|trace| trace.status.clone()),
                latest_verifier_result_index: latest_verifier.map(|result| result.index),
                latest_verifier_status,
                latest_parallel_run_index: latest_parallel_run.map(|run| run.index),
                latest_mini_task,
                latest_error_category,
                latest_pass_evidence,
                latest_summary,
                needs_repair,
                ready_for_harder_task,
                next_action,
            });
        }

        let active_slot_field = summaries
            .iter()
            .find(|summary| summary.needs_repair)
            .map(|summary| summary.field.clone())
            .or_else(|| {
                self.fair_parallel_field_pool(
                    &catalog,
                    self.next_parallel_evolution_run_index,
                    None,
                )
                .into_iter()
                .find(|field| {
                    summaries
                        .iter()
                        .find(|summary| summary.field == *field)
                        .is_some_and(|summary| {
                            let has_pending_mini_task =
                                summary.next_mini_task.as_ref().is_some_and(|task| {
                                    self.field_mini_task_status(&summary.field, task)
                                        != Some(Status::Satisfied)
                                });
                            has_pending_mini_task
                                || summary.latest_verifier_status != Some(Status::Satisfied)
                        })
                })
            })
            .or_else(|| {
                summaries
                    .iter()
                    .find(|summary| !summary.ready_for_harder_task)
                    .map(|summary| summary.field.clone())
            });
        let all_first_pass_satisfied = !catalog.packs.is_empty()
            && catalog.packs.iter().all(|pack| {
                pack.mini_tasks.first().is_some_and(|task| {
                    self.field_mini_task_status(&pack.id, &task.id) == Some(Status::Satisfied)
                })
            });
        let all_pack_tasks_satisfied = !catalog.packs.is_empty()
            && catalog.packs.iter().all(|pack| {
                !pack.mini_tasks.is_empty()
                    && pack.mini_tasks.iter().all(|task| {
                        self.field_mini_task_status(&pack.id, &task.id) == Some(Status::Satisfied)
                    })
            })
            && summaries.iter().all(|summary| !summary.needs_repair);
        let active_slot_reason = if all_pack_tasks_satisfied {
            "all peer field tasks satisfied; add a harder mini task layer".to_string()
        } else if let Some(field) = &active_slot_field {
            summaries
                .iter()
                .find(|summary| summary.field == *field)
                .map(|summary| {
                    if summary.needs_repair {
                        format!("{field} selected by latest verifier failure")
                    } else {
                        format!("{field} selected by field status and recent-run fairness")
                    }
                })
                .unwrap_or_else(|| format!("{field} selected from the peer field pool"))
        } else {
            "no runnable peer field slot selected".to_string()
        };
        let latest_worker_slot_count = self
            .parallel_evolution_runs
            .last()
            .map(|run| run.worker_count)
            .unwrap_or_default();
        let next = if all_pack_tasks_satisfied {
            vec![
                format!("octopus{state_args} fields summary"),
                field_harder_layer_next_action(&state_args),
            ]
        } else if let Some(field) = &active_slot_field {
            summaries
                .iter()
                .find(|summary| summary.field == *field)
                .map(|summary| vec![summary.next_action.clone()])
                .unwrap_or_default()
        } else {
            vec![format!("octopus{state_args} evolve parallel --workers 1")]
        };
        Ok(FieldTrajectoryReport {
            field_count: summaries.len(),
            latest_worker_slot_count,
            trace_count: self.feed_traces.len(),
            verifier_result_count: self.field_verifier_results.len(),
            parallel_evolution_run_count: self.parallel_evolution_runs.len(),
            all_first_pass_satisfied,
            all_pack_tasks_satisfied,
            active_slot_field,
            active_slot_reason,
            fields: summaries,
            next,
        })
    }

    pub fn field_pool_status_report(
        &self,
        state_path: Option<&Path>,
    ) -> Option<FieldPoolStatusReport> {
        let report = self.field_trajectory_report_with_state(state_path).ok()?;
        let slots = report
            .fields
            .iter()
            .map(|summary| {
                let latest_worker = self
                    .parallel_evolution_runs
                    .iter()
                    .rev()
                    .flat_map(|run| run.workers.iter())
                    .find(|worker| worker.field == summary.field);
                let completed = summary.mini_task_count > 0
                    && summary.satisfied_mini_task_count == summary.mini_task_count
                    && !summary.needs_repair;
                let user_goal_hint = user_surface::field_slot_hint(
                    &summary.field,
                    completed,
                    summary.next_mini_task.as_deref(),
                    summary.needs_repair,
                );
                FieldPoolSlotReport {
                    field: summary.field.clone(),
                    completed,
                    mini_task_count: summary.mini_task_count,
                    satisfied_mini_task_count: summary.satisfied_mini_task_count,
                    next_mini_task: summary.next_mini_task.clone(),
                    latest_worker_id: latest_worker.map(|worker| worker.id.clone()),
                    latest_mini_task: latest_worker
                        .and_then(|worker| worker.mini_task.clone())
                        .or_else(|| summary.latest_mini_task.clone()),
                    latest_worker_status: latest_worker.map(|worker| worker.status.clone()),
                    latest_status: summary
                        .latest_verifier_status
                        .clone()
                        .or_else(|| summary.latest_trace_status.clone())
                        .or_else(|| latest_worker.map(|worker| worker.status.clone())),
                    latest_parallel_run_index: summary.latest_parallel_run_index,
                    latest_updated_at_secs: latest_worker
                        .map(|worker| worker.updated_at_secs)
                        .unwrap_or_default(),
                    needs_repair: summary.needs_repair,
                    agent_next_action: summary.next_action.clone(),
                    user_goal_hint: user_goal_hint.clone(),
                    next_action: user_goal_hint,
                }
            })
            .collect::<Vec<_>>();
        let completed_fields = slots.iter().filter(|slot| slot.completed).count();
        let user_goal_hints = user_surface::field_pool_hints(
            report.field_count,
            completed_fields,
            report.active_slot_field.as_deref(),
        );
        Some(FieldPoolStatusReport {
            policy: parallel_field_pool_policy().to_string(),
            field_count: report.field_count,
            field_slot_count: report.field_count,
            latest_worker_slot_count: report.latest_worker_slot_count,
            completed_fields,
            active_slot_field: report.active_slot_field,
            active_slot_reason: report.active_slot_reason,
            worker_slots: parallel_worker_policy().to_string(),
            slots,
            agent_next: report.next,
            user_goal_hints: user_goal_hints.clone(),
            next: user_goal_hints,
        })
    }

    fn harness_learning_summary(&self, state_args: &str) -> HarnessLearningSummary {
        let prefer_evolution = self
            .last_pet_event
            .as_ref()
            .is_some_and(|event| event.source == "evolve score");
        if prefer_evolution {
            if let Some(summary) = self.evolution_learning_summary(state_args) {
                return summary;
            }
        }
        if let Some(outcome) = self.repair_outcomes.last() {
            let target = outcome
                .target_tentacle
                .clone()
                .unwrap_or_else(|| outcome.tentacle_id.clone());
            let candidate = outcome.candidate.clone();
            let next_action = if !target.trim().is_empty() {
                format!(
                    "octopus{state_args} evolve recommend {}",
                    shell_arg(&target)
                )
            } else {
                format!("octopus{state_args} beat 200")
            };
            return HarnessLearningSummary {
                source: "repair_outcome".to_string(),
                repair_outcomes: self.repair_outcomes.len(),
                evolution_outcomes: self.evolution_outcomes.len(),
                target_tentacle: Some(target),
                candidate,
                status: Some(outcome.status.clone()),
                score: Some(outcome.score),
                summary: Some(outcome.summary.clone()),
                next_action,
            };
        }
        if let Some(summary) = self.evolution_learning_summary(state_args) {
            return summary;
        }
        HarnessLearningSummary {
            source: "none".to_string(),
            repair_outcomes: 0,
            evolution_outcomes: 0,
            target_tentacle: None,
            candidate: None,
            status: None,
            score: None,
            summary: None,
            next_action: format!("octopus{state_args} beat 200"),
        }
    }

    fn evolution_learning_summary(&self, state_args: &str) -> Option<HarnessLearningSummary> {
        let outcome = self.evolution_outcomes.last()?;
        Some(HarnessLearningSummary {
            source: "evolution_outcome".to_string(),
            repair_outcomes: self.repair_outcomes.len(),
            evolution_outcomes: self.evolution_outcomes.len(),
            target_tentacle: Some(outcome.tentacle_id.clone()),
            candidate: Some(outcome.candidate_id.clone()),
            status: Some(outcome.status.clone()),
            score: Some(outcome.score),
            summary: Some(outcome.summary.clone()),
            next_action: format!(
                "octopus{state_args} evolve recommend {}",
                shell_arg(&outcome.tentacle_id)
            ),
        })
    }

    fn memory_context_records(&self, limit: usize) -> Vec<MemoryContextRecord> {
        let mut records = self.memory.records.values().cloned().collect::<Vec<_>>();
        records.sort_by(|left, right| {
            right
                .weight
                .total_cmp(&left.weight)
                .then_with(|| memory_id_number(&right.id).cmp(&memory_id_number(&left.id)))
        });
        records
            .into_iter()
            .take(limit)
            .map(|record| MemoryContextRecord {
                id: record.id,
                text: short_text(&record.text, FEED_TRACE_SUMMARY_BYTES),
                weight: record.weight,
            })
            .collect()
    }

    fn tentacle_context_report(
        &self,
        tentacle: &InstalledTentacle,
        limit: usize,
    ) -> TentacleContextReport {
        let tools = installed_tentacle_tools(tentacle)
            .into_iter()
            .map(|tool| self.tool_context(&tool))
            .collect::<Vec<_>>();
        let recent_actions = self
            .recent_feed_traces_for_tentacle(&tentacle.id, limit)
            .into_iter()
            .map(|trace| TentacleActionContext {
                index: trace.index,
                need: NeedContext {
                    kind: trace.need_kind,
                    query: trace.need_query,
                },
                tool: trace.tool,
                plan_source: trace.plan_source,
                status: trace.status,
                summary: trace.summary,
            })
            .collect::<Vec<_>>();
        TentacleContextReport {
            id: tentacle.id.clone(),
            brain_kind: tentacle.brain_kind.clone(),
            policy: TENTACLE_CONTEXT_POLICY.to_string(),
            slots: vec![
                "Need".to_string(),
                "Tool".to_string(),
                "Action".to_string(),
                "Feed".to_string(),
            ],
            tools,
            recent_actions,
        }
    }

    fn tool_context(&self, tool: &InstalledToolRef) -> ToolContext {
        let active_grant = active_grant_for_tool(tool, &self.grants).map(|grant| grant.id.clone());
        ToolContext {
            id: tool.id.clone(),
            description: tool.description.clone(),
            runtime: tool.kind.clone(),
            entrypoint: tool.entrypoint.clone(),
            contract: tool.contract.clone(),
            permission: tool.permission.as_ref().map(tool_permission_text),
            authorization_required: tool.permission.is_some(),
            active_grant,
        }
    }

    pub fn record_pet_event(
        &mut self,
        state: impl Into<String>,
        source: impl Into<String>,
        summary: impl Into<String>,
        status: Status,
    ) -> PetEvent {
        let event = PetEvent {
            timestamp_secs: unix_timestamp_secs(),
            state: state.into(),
            source: source.into(),
            summary: summary.into(),
            status,
            stage: None,
            error_class: None,
        };
        self.last_pet_event = Some(event.clone());
        event
    }

    pub fn record_pet_event_from_feed(&mut self, feed: &Feed) -> PetEvent {
        let state = match feed.status {
            Status::Failed | Status::Unsupported => "blocked",
            Status::Partial => "harness",
            Status::Satisfied => match feed.need.kind {
                NeedKind::Remember | NeedKind::Recall | NeedKind::Forget => "memory",
                _ => "success",
            },
        };
        let source = feed
            .metadata
            .get("tentacle")
            .cloned()
            .unwrap_or_else(|| kind_key(&feed.need.kind).to_string());
        let summary = if feed.summary.is_empty() {
            format!("{} {}", kind_key(&feed.need.kind), status_key(&feed.status))
        } else {
            feed.summary.clone()
        };
        self.record_pet_event(state, source, summary, feed.status.clone())
    }

    pub fn record_feed_trace_from_feed(&mut self, feed: &Feed) -> FeedTraceRecord {
        self.next_feed_trace_index += 1;
        let trace = FeedTraceRecord {
            index: self.next_feed_trace_index,
            need_kind: feed.need.kind.clone(),
            need_query: short_text(&feed.need.query, FEED_TRACE_QUERY_BYTES),
            status: feed.status.clone(),
            field: feed
                .metadata
                .get("field_pack")
                .or_else(|| feed.metadata.get("field"))
                .or_else(|| feed.need.context.get("field_pack"))
                .or_else(|| feed.need.context.get("field"))
                .cloned(),
            tentacle: feed
                .metadata
                .get("tentacle")
                .or_else(|| feed.metadata.get("tentacle_brain"))
                .cloned(),
            tool: feed.metadata.get("tool").cloned(),
            plan_source: feed.metadata.get("plan_source").cloned(),
            route: feed.metadata.get("route").cloned(),
            evidence_count: feed.evidence.len(),
            summary: short_text(&feed.summary, FEED_TRACE_SUMMARY_BYTES),
            metadata: feed.metadata.clone(),
        };
        self.feed_traces.push(trace.clone());
        trace
    }

    pub fn record_feed_feedback(
        &mut self,
        trace_index: u64,
        status: Status,
        summary: impl Into<String>,
    ) -> Result<FeedFeedbackOutcome, String> {
        let summary = summary.into();
        let Some(position) = self
            .feed_traces
            .iter()
            .position(|trace| trace.index == trace_index)
        else {
            return Err(format!("unknown feed trace: {trace_index}"));
        };
        let original = self.feed_traces[position].clone();
        let mut metadata =
            BTreeMap::from([("feedback_trace".to_string(), trace_index.to_string())]);
        if let Some(tentacle) = &original.tentacle {
            metadata.insert("tentacle".to_string(), tentacle.clone());
        }
        if let Some(tool) = &original.tool {
            metadata.insert("tool".to_string(), tool.clone());
        }
        if let Some(plan_source) = &original.plan_source {
            metadata.insert("plan_source".to_string(), plan_source.clone());
        }
        let feed = Feed {
            need: Need::new(original.need_kind.clone(), original.need_query.clone()),
            status: status.clone(),
            evidence: vec![Evidence::new("user_feedback", summary.clone())],
            summary: summary.clone(),
            metadata,
        };
        if original.tentacle.is_some() {
            self.routes.learn(&feed);
        }
        let route_key = original
            .tentacle
            .as_ref()
            .map(|tentacle| route_key(&original.need_kind, tentacle));
        let route_score = original
            .tentacle
            .as_ref()
            .map(|tentacle| self.routes.score(&original.need_kind, tentacle));
        let event_state = match status {
            Status::Satisfied => "success",
            Status::Partial => "harness",
            Status::Failed | Status::Unsupported => "blocked",
        };
        let pet_event = self.record_pet_event(
            event_state,
            "feed feedback",
            format!("#{trace_index} {summary}"),
            status.clone(),
        );
        let trace = &mut self.feed_traces[position];
        trace.status = status.clone();
        trace.summary = short_text(&format!("feedback: {summary}"), FEED_TRACE_SUMMARY_BYTES);
        trace.metadata.insert(
            "feedback_status".to_string(),
            status_key(&status).to_string(),
        );
        trace.metadata.insert(
            "feedback_summary".to_string(),
            short_text(&summary, FEED_TRACE_SUMMARY_BYTES),
        );
        let trace = trace.clone();
        Ok(FeedFeedbackOutcome {
            trace,
            status,
            route_key,
            route_score,
            summary,
            pet_event,
        })
    }

    fn latest_unsatisfied_field_verifier_result(&self) -> Option<&FieldVerifierResult> {
        let mut seen_fields = BTreeSet::new();
        for result in self.field_verifier_results.iter().rev() {
            if !seen_fields.insert(result.field.as_str()) {
                continue;
            }
            if !self.field_verifier_result_has_real_mini_task(result) {
                continue;
            }
            if result.status != Status::Satisfied {
                return Some(result);
            }
        }
        None
    }

    fn latest_field_verifier_status(&self, field: &str) -> Option<Status> {
        self.field_verifier_results
            .iter()
            .rev()
            .find(|result| result.field == field)
            .map(|result| result.status.clone())
    }

    fn field_verifier_result_has_real_mini_task(&self, result: &FieldVerifierResult) -> bool {
        self.feed_traces
            .iter()
            .rev()
            .find(|trace| trace.index == result.trace_index)
            .and_then(|trace| trace.metadata.get("field_mini_task"))
            .is_some_and(|mini_task| {
                let mini_task = mini_task.trim();
                !mini_task.is_empty() && mini_task != "ad-hoc"
            })
    }

    fn field_mini_task_status(&self, field: &str, mini_task: &str) -> Option<Status> {
        self.field_verifier_results
            .iter()
            .rev()
            .find(|result| {
                result.field == field
                    && self
                        .feed_traces
                        .iter()
                        .find(|trace| trace.index == result.trace_index)
                        .and_then(|trace| trace.metadata.get("field_mini_task"))
                        .is_some_and(|task| task == mini_task)
            })
            .map(|result| result.status.clone())
    }

    fn next_field_mini_task<'a>(&self, pack: &'a FieldPack) -> Option<&'a FieldMiniTask> {
        pack.mini_tasks
            .iter()
            .find(|task| self.field_mini_task_status(&pack.id, &task.id) != Some(Status::Satisfied))
    }

    fn annotate_field_mini_task_context(&self, packs: &[FieldPack], need: &mut Need) {
        if need.context.contains_key("field_mini_task") {
            return;
        }
        let Some(field) = need.context.get("field_pack").cloned() else {
            return;
        };
        let Some(pack) = packs.iter().find(|pack| pack.id == field) else {
            return;
        };
        let selected = pack
            .mini_tasks
            .iter()
            .find(|task| need.query.contains(&task.id))
            .or_else(|| self.next_field_mini_task(pack));
        if let Some(task) = selected {
            need.context
                .insert("field_mini_task".to_string(), task.id.clone());
            need.context.insert(
                "field_expected_feed".to_string(),
                task.expected_feed.clone(),
            );
        }
    }

    fn field_parallel_worker_ready(&self, catalog: &FieldPackCatalog, field: &str) -> bool {
        let has_pending_mini_task = catalog
            .packs
            .iter()
            .find(|pack| pack.id == field)
            .and_then(|pack| self.next_field_mini_task(pack))
            .is_some_and(|task| {
                self.field_mini_task_status(field, &task.id) != Some(Status::Satisfied)
            });
        has_pending_mini_task || self.latest_field_verifier_status(field) != Some(Status::Satisfied)
    }

    fn field_allowed_by_objective_pool(pool: Option<&[String]>, field: &str) -> bool {
        pool.map(|fields| fields.iter().any(|candidate| candidate == field))
            .unwrap_or(true)
    }

    fn has_active_parallel_field_goal(&self) -> bool {
        self.goal.as_ref().is_some_and(|goal| {
            let objective = goal.objective.to_ascii_lowercase();
            objective.contains("parallel field")
                || objective.contains("peer field")
                || fields_mentioned_in_text(&objective).len() > 1
        })
    }

    pub fn record_field_verifier_result(
        &mut self,
        trace_index: u64,
        status: Status,
        error_category: Option<String>,
        artifact: Option<String>,
        summary: impl Into<String>,
    ) -> Result<FieldVerifierResult, String> {
        let trace = self
            .feed_traces
            .iter()
            .find(|trace| trace.index == trace_index)
            .ok_or_else(|| format!("unknown feed trace: {trace_index}"))?;
        let field = trace
            .field
            .clone()
            .or_else(|| trace.metadata.get("field_pack").cloned())
            .or_else(|| trace.metadata.get("field").cloned())
            .ok_or_else(|| format!("feed trace {trace_index} has no field_pack"))?;
        self.next_field_verifier_result_index += 1;
        let summary = short_text(&summary.into(), FEED_TRACE_SUMMARY_BYTES);
        let result = FieldVerifierResult {
            index: self.next_field_verifier_result_index,
            trace_index,
            field,
            status: status.clone(),
            error_category: error_category
                .map(|value| short_text(value.trim(), 96))
                .filter(|value| !value.is_empty()),
            artifact: artifact
                .map(|value| short_text(value.trim(), 240))
                .filter(|value| !value.is_empty()),
            summary: summary.clone(),
        };
        let event_state = match status {
            Status::Satisfied => "feed",
            Status::Partial => "harness",
            Status::Failed | Status::Unsupported => "blocked",
        };
        self.record_pet_event(
            event_state,
            "field verifier",
            format!("{} #{} {}", result.field, trace_index, summary),
            status,
        );
        self.field_verifier_results.push(result.clone());
        Ok(result)
    }

    pub fn start_parallel_evolution(
        &mut self,
        objective: impl Into<String>,
        worker_limit: usize,
    ) -> Result<ParallelEvolutionRun, String> {
        let objective = objective.into();
        let catalog = default_field_pack_catalog()?;
        let limit = worker_limit.clamp(1, 8);
        let mut fields = Vec::new();
        let objective_fields = fields_mentioned_in_text(&objective);
        let objective_pool = if objective_fields.is_empty() {
            None
        } else {
            Some(objective_fields.clone())
        };
        if objective_fields.len() == 1
            && self.field_parallel_worker_ready(&catalog, &objective_fields[0])
        {
            push_unique_limited(&mut fields, objective_fields[0].clone(), limit);
        }
        if fields.len() < limit {
            for result in self.field_verifier_results.iter().rev() {
                if matches!(
                    result.status,
                    Status::Failed | Status::Partial | Status::Unsupported
                ) && self.latest_field_verifier_status(&result.field)
                    == Some(result.status.clone())
                    && self.field_verifier_result_has_real_mini_task(result)
                    && Self::field_allowed_by_objective_pool(
                        objective_pool.as_deref(),
                        &result.field,
                    )
                {
                    push_unique_limited(&mut fields, result.field.clone(), limit);
                }
                if fields.len() >= limit {
                    break;
                }
            }
        }
        if fields.len() < limit {
            for field in self.fair_parallel_field_pool(
                &catalog,
                self.next_parallel_evolution_run_index,
                objective_pool.as_deref(),
            ) {
                if self.field_parallel_worker_ready(&catalog, &field) {
                    push_unique_limited(&mut fields, field, limit);
                }
                if fields.len() >= limit {
                    break;
                }
            }
        }
        for trace in self.feed_traces.iter().rev() {
            if let Some(field) = &trace.field {
                if Self::field_allowed_by_objective_pool(objective_pool.as_deref(), field)
                    && self.field_parallel_worker_ready(&catalog, field)
                {
                    push_unique_limited(&mut fields, field.clone(), limit);
                }
            }
            if fields.len() >= limit {
                break;
            }
        }
        for pack in &catalog.packs {
            if Self::field_allowed_by_objective_pool(objective_pool.as_deref(), &pack.id)
                && self.field_parallel_worker_ready(&catalog, &pack.id)
            {
                push_unique_limited(&mut fields, pack.id.clone(), limit);
            }
            if fields.len() >= limit {
                break;
            }
        }

        let candidate_fields = if objective_fields.is_empty() {
            default_field_pack_ids()
        } else {
            objective_fields.clone()
        };
        self.next_parallel_evolution_run_index += 1;
        let run_index = self.next_parallel_evolution_run_index;
        let now = unix_timestamp_secs();
        let workers = fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let trace = self
                    .feed_traces
                    .iter()
                    .rev()
                    .find(|trace| trace.field.as_deref() == Some(field.as_str()));
                let verifier = self
                    .field_verifier_results
                    .iter()
                    .rev()
                    .find(|result| result.field == *field);
                let tentacle = trace
                    .and_then(|trace| trace.tentacle.clone())
                    .unwrap_or_else(|| format!("field:{field}"));
                ParallelEvolutionWorker {
                    id: format!("octopus-{run_index}-{}", index + 1),
                    field: field.clone(),
                    mini_task: None,
                    goal: format!(
                        "Improve {field} Feed supply from trajectories while keeping Brain clean: {objective}"
                    ),
                    updated_at_secs: now,
                    queued_need_index: None,
                    source_trace_index: trace.map(|trace| trace.index),
                    verifier_result_index: verifier.map(|result| result.index),
                    status: Status::Partial,
                    next_action: format!(
                        "octopus evolve recommend {} {}",
                        shell_arg(&tentacle),
                        shell_arg(&format!("improve {field} harness from field traces"))
                    ),
                }
            })
            .collect::<Vec<_>>();
        let summary = if workers.is_empty() {
            format!(
                "field evolution run #{run_index}: 0 active peer field slot(s); add a harder mini task layer"
            )
        } else {
            format!(
                "field evolution run #{run_index}: {} active peer field slot(s): {}",
                workers.len(),
                workers
                    .iter()
                    .map(|worker| worker.field.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let mut run = ParallelEvolutionRun {
            index: run_index,
            objective,
            field_pool_size: default_field_pack_ids().len(),
            field_pool_policy: parallel_field_pool_policy().to_string(),
            candidate_fields,
            requested_worker_count: worker_limit,
            worker_count: workers.len(),
            worker_policy: parallel_worker_policy().to_string(),
            workers,
            summary: summary.clone(),
        };
        for worker in &mut run.workers {
            let mini_task = catalog
                .packs
                .iter()
                .find(|pack| pack.id == worker.field)
                .and_then(|pack| self.next_field_mini_task(pack));
            let mut context = BTreeMap::from([("field_pack".to_string(), worker.field.clone())]);
            if let Some(task) = mini_task {
                context.insert("field_mini_task".to_string(), task.id.clone());
                context.insert(
                    "field_expected_feed".to_string(),
                    task.expected_feed.clone(),
                );
                worker.mini_task = Some(task.id.clone());
            }
            let query = mini_task
                .map(|task| {
                    format!(
                        "Run {} mini task {}: {}",
                        worker.field, task.id, task.goal
                    )
                })
                .unwrap_or_else(|| {
                    format!(
                        "Run {} in this peer field slot: run or define one mini task, record trajectory and verifier result, then propose harness repair if it fails",
                        worker.field
                    )
                });
            let summary = mini_task
                .map(|task| {
                    format!(
                        "field: {}; mini task: {}; expected Feed: {}",
                        worker.field, task.id, task.expected_feed
                    )
                })
                .unwrap_or_else(|| "selected from the peer field pool".to_string());
            let queued = self.queue_need_suggestion_with_context(
                GoalNeedSuggestion {
                    kind: NeedKind::Verify,
                    query,
                },
                context,
                "field evolution",
                run.objective.clone(),
                summary,
            );
            worker.queued_need_index = Some(queued.index);
        }
        self.record_pet_event("harness", "parallel evolution", summary, Status::Partial);
        self.parallel_evolution_runs.push(run.clone());
        Ok(run)
    }

    pub fn update_parallel_evolution_worker_result(
        &mut self,
        run_index: u64,
        queued_need_index: u64,
        source_trace_index: Option<u64>,
        verifier_result_index: Option<u64>,
        status: Status,
    ) -> Option<ParallelEvolutionRun> {
        let trace_tentacle = source_trace_index
            .and_then(|index| self.feed_traces.iter().find(|trace| trace.index == index))
            .and_then(|trace| trace.tentacle.clone())
            .unwrap_or_else(|| "field-mini-task".to_string());
        let run = self
            .parallel_evolution_runs
            .iter_mut()
            .find(|run| run.index == run_index)?;
        let worker = run
            .workers
            .iter_mut()
            .find(|worker| worker.queued_need_index == Some(queued_need_index))?;
        worker.source_trace_index = source_trace_index.or(worker.source_trace_index);
        worker.verifier_result_index = verifier_result_index.or(worker.verifier_result_index);
        worker.status = status.clone();
        worker.updated_at_secs = unix_timestamp_secs();
        worker.next_action = if status == Status::Satisfied {
            "octopus fields summary".to_string()
        } else {
            format!(
                "octopus evolve recommend {} {}",
                shell_arg(&trace_tentacle),
                shell_arg(&format!(
                    "improve {} harness from field traces",
                    worker.field
                ))
            )
        };
        Some(run.clone())
    }

    fn fair_parallel_field_pool(
        &self,
        catalog: &FieldPackCatalog,
        run_index: u64,
        candidates: Option<&[String]>,
    ) -> Vec<String> {
        let rotated = parallel_field_goal_pool(run_index);
        let mut fields = candidates
            .map(|fields| fields.to_vec())
            .unwrap_or_else(|| rotated.clone());
        fields.retain(|field| catalog.packs.iter().any(|pack| pack.id == *field));
        fields.sort_by_key(|field| {
            let has_pending_task = catalog
                .packs
                .iter()
                .find(|pack| pack.id == *field)
                .and_then(|pack| self.next_field_mini_task(pack))
                .is_some_and(|task| {
                    self.field_mini_task_status(field, &task.id) != Some(Status::Satisfied)
                });
            let needs_attention = has_pending_task
                || self.latest_field_verifier_status(field) != Some(Status::Satisfied);
            let latest_run = self.latest_parallel_run_index_for_field(field);
            let rotated_position = rotated
                .iter()
                .position(|candidate| candidate == field)
                .unwrap_or(usize::MAX);
            (
                !needs_attention,
                latest_run.unwrap_or(0),
                latest_run.is_some(),
                rotated_position,
            )
        });
        fields
    }

    fn latest_parallel_run_index_for_field(&self, field: &str) -> Option<u64> {
        self.parallel_evolution_runs
            .iter()
            .rev()
            .find(|run| {
                run.workers
                    .iter()
                    .any(|worker| worker.field.as_str() == field)
            })
            .map(|run| run.index)
    }

    pub fn record_starter_feedback(
        &mut self,
        input: StarterFeedbackInput,
    ) -> StarterFeedbackRecord {
        self.next_starter_feedback_index += 1;
        let record = StarterFeedbackRecord {
            index: self.next_starter_feedback_index,
            tentacle_id: short_text(&input.tentacle_id, 96),
            objective: short_text(&input.objective, STARTER_FEEDBACK_OBJECTIVE_BYTES),
            group: input.group.map(|group| short_text(&group, 64)),
            score: input.status.score(),
            status: input.status,
            summary: short_text(&input.summary, STARTER_FEEDBACK_SUMMARY_BYTES),
        };
        let event_state = match record.status {
            StarterFeedbackStatus::Accepted => "success",
            StarterFeedbackStatus::Ignored => "harness",
            StarterFeedbackStatus::Failed => "blocked",
        };
        self.record_pet_event(
            event_state,
            "starter feedback",
            format!("{} {}", record.tentacle_id, record.summary),
            match record.status {
                StarterFeedbackStatus::Accepted => Status::Satisfied,
                StarterFeedbackStatus::Ignored => Status::Partial,
                StarterFeedbackStatus::Failed => Status::Failed,
            },
        );
        self.starter_feedback.push(record.clone());
        record
    }

    pub fn recent_starter_feedback_for_tentacle(
        &self,
        tentacle_id: &str,
        limit: usize,
    ) -> Vec<StarterFeedbackRecord> {
        let mut records = self
            .starter_feedback
            .iter()
            .filter(|record| record.tentacle_id == tentacle_id)
            .cloned()
            .collect::<Vec<_>>();
        let start = records.len().saturating_sub(limit);
        records.drain(0..start);
        records
    }

    pub fn compact_starter_feedback(&mut self, keep: usize) -> usize {
        if self.starter_feedback.len() <= keep {
            return 0;
        }
        let dropped = self.starter_feedback.len() - keep;
        self.starter_feedback.drain(0..dropped);
        dropped
    }

    pub fn recent_feed_traces(&self, limit: usize) -> Vec<FeedTraceRecord> {
        let start = self.feed_traces.len().saturating_sub(limit);
        self.feed_traces[start..].to_vec()
    }

    pub fn recent_feed_traces_for_tentacle(
        &self,
        tentacle_id: &str,
        limit: usize,
    ) -> Vec<FeedTraceRecord> {
        let mut traces = self
            .feed_traces
            .iter()
            .rev()
            .filter(|trace| trace.tentacle.as_deref() == Some(tentacle_id))
            .take(limit)
            .cloned()
            .collect::<Vec<_>>();
        traces.reverse();
        traces
    }

    pub fn compact_feed_traces(&mut self, keep: usize) -> usize {
        if self.feed_traces.len() <= keep {
            return 0;
        }
        let dropped = self.feed_traces.len() - keep;
        self.feed_traces.drain(0..dropped);
        dropped
    }

    pub fn record_check_history(&mut self, input: CheckHistoryInput) -> CheckHistoryRecord {
        self.next_check_history_index += 1;
        let record = CheckHistoryRecord {
            index: self.next_check_history_index,
            timestamp_secs: unix_timestamp_secs(),
            tentacle_id: input.tentacle_id,
            source_kind: input.source_kind,
            command_index: input.command_index,
            command: input.command,
            cwd: input.cwd,
            status: input.status,
            code: input.code,
            stdout: short_text(&input.stdout, CHECK_HISTORY_OUTPUT_BYTES),
            stderr: short_text(&input.stderr, CHECK_HISTORY_OUTPUT_BYTES),
        };
        self.check_history.push(record.clone());
        record
    }

    pub fn recent_check_history_for_tentacle(
        &self,
        tentacle_id: &str,
        limit: usize,
    ) -> Vec<CheckHistoryRecord> {
        let mut history = self
            .check_history
            .iter()
            .rev()
            .filter(|record| record.tentacle_id == tentacle_id)
            .take(limit)
            .cloned()
            .collect::<Vec<_>>();
        history.reverse();
        history
    }

    pub fn compact_check_history(&mut self, keep: usize) -> usize {
        if self.check_history.len() <= keep {
            return 0;
        }
        let dropped = self.check_history.len() - keep;
        self.check_history.drain(0..dropped);
        dropped
    }

    pub fn record_repair_outcome(
        &mut self,
        trace_index: Option<u64>,
        status: Status,
        summary: impl Into<String>,
    ) -> Result<RepairOutcome, String> {
        let summary = summary.into();
        let trace = match trace_index {
            Some(index) => Some(
                self.feed_traces
                    .iter()
                    .find(|trace| trace.index == index)
                    .cloned()
                    .ok_or_else(|| format!("unknown feed trace: {index}"))?,
            ),
            None => None,
        };
        self.next_repair_outcome_index += 1;
        let tentacle_id = trace
            .as_ref()
            .and_then(|trace| trace.tentacle.clone())
            .unwrap_or_else(|| "harness-repair-agent".to_string());
        let tool = trace.as_ref().and_then(|trace| trace.tool.clone());
        let target_tentacle = trace.as_ref().and_then(|trace| {
            repair_outcome_metadata_value(trace.metadata.get("target_tentacle"), false)
        });
        let candidate = trace
            .as_ref()
            .and_then(|trace| repair_outcome_metadata_value(trace.metadata.get("candidate"), true));
        let outcome = RepairOutcome {
            index: self.next_repair_outcome_index,
            trace_index,
            tentacle_id: tentacle_id.clone(),
            tool: tool.clone(),
            target_tentacle: target_tentacle.clone(),
            candidate: candidate.clone(),
            status: status.clone(),
            score: evolution_status_score(&status),
            summary: summary.clone(),
        };
        let need = trace
            .as_ref()
            .map(|trace| Need::new(trace.need_kind.clone(), trace.need_query.clone()))
            .unwrap_or_else(|| Need::new(NeedKind::Verify, "harness repair outcome"));
        let mut metadata = BTreeMap::from([
            ("tentacle".to_string(), tentacle_id),
            ("tool".to_string(), "repair_outcome".to_string()),
            ("repair_outcome".to_string(), outcome.index.to_string()),
        ]);
        if let Some(trace_index) = trace_index {
            metadata.insert("feed_trace_index".to_string(), trace_index.to_string());
        }
        if let Some(tool) = tool {
            metadata.insert("source_tool".to_string(), tool);
        }
        if let Some(target_tentacle) = target_tentacle {
            metadata.insert("target_tentacle".to_string(), target_tentacle);
        }
        if let Some(candidate) = candidate {
            metadata.insert("candidate".to_string(), candidate);
        }
        let feed = Feed {
            need,
            status: status.clone(),
            evidence: vec![Evidence::new("repair_outcome", summary.clone())],
            summary: summary.clone(),
            metadata,
        };
        self.routes.learn(&feed);
        let event_state = match status {
            Status::Satisfied | Status::Partial => "harness",
            Status::Failed | Status::Unsupported => "blocked",
        };
        self.record_pet_event(
            event_state,
            "repair score",
            format!("#{} {summary}", outcome.index),
            status,
        );
        self.repair_outcomes.push(outcome.clone());
        Ok(outcome)
    }

    pub fn recent_repair_outcomes(&self, limit: usize) -> Vec<RepairOutcome> {
        let mut outcomes = self
            .repair_outcomes
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>();
        outcomes.reverse();
        outcomes
    }

    pub fn compact_repair_outcomes(&mut self, keep: usize) -> usize {
        if self.repair_outcomes.len() <= keep {
            return 0;
        }
        let dropped = self.repair_outcomes.len() - keep;
        self.repair_outcomes.drain(0..dropped);
        dropped
    }

    pub fn adapt_environment(
        &mut self,
        cwd: impl AsRef<Path>,
        tentacles_root: impl AsRef<Path>,
    ) -> AdaptReport {
        let environment = EnvironmentReport::detect(cwd);
        let tentacles_root = tentacles_root.as_ref();
        let mut installed_profiles = Vec::new();
        let mut installed_tentacles = Vec::new();
        let mut skipped_manifests = Vec::new();
        for profile in &environment.recommended_profiles {
            let profile_was_installed = self
                .installed_profiles
                .iter()
                .any(|installed| installed == profile);
            if self.install_profile(profile).is_ok() && !profile_was_installed {
                installed_profiles.push(profile.clone());
            }

            let tentacle_was_installed = self
                .installed_tentacles
                .iter()
                .any(|tentacle| tentacle.id == *profile);
            match self.install_manifest(tentacles_root, profile) {
                Ok(tentacle) if !tentacle_was_installed => installed_tentacles.push(tentacle.id),
                Ok(_) => {}
                Err(_) => skipped_manifests.push(profile.clone()),
            }
        }
        skipped_manifests.sort();
        skipped_manifests.dedup();
        AdaptReport {
            environment,
            installed_profiles,
            installed_tentacles,
            skipped_manifests,
        }
    }

    pub fn grant_oauth(
        &mut self,
        provider: &str,
        scope: &str,
        permissions: Vec<String>,
    ) -> CapabilityGrant {
        let permissions = normalize_permissions(permissions);
        let id = grant_id(provider, scope);
        if let Some(grant) = self.grants.iter_mut().find(|grant| grant.id == id) {
            grant.permissions = permissions;
            grant.status = GrantStatus::Active;
            return grant.clone();
        }
        let grant = CapabilityGrant {
            id,
            provider: provider.to_string(),
            scope: scope.to_string(),
            permissions,
            status: GrantStatus::Active,
        };
        self.grants.push(grant.clone());
        grant
    }

    pub fn revoke_grant(&mut self, grant_id: &str) -> bool {
        let Some(grant) = self.grants.iter_mut().find(|grant| grant.id == grant_id) else {
            return false;
        };
        grant.status = GrantStatus::Revoked;
        true
    }

    pub fn active_grant(&self, provider: &str, scope: &str) -> Option<&CapabilityGrant> {
        self.grants.iter().find(|grant| {
            grant.provider == provider
                && grant.scope == scope
                && grant.status == GrantStatus::Active
        })
    }

    pub fn record_evolution_outcome(
        &mut self,
        tentacle_id: impl Into<String>,
        candidate_id: impl Into<String>,
        status: Status,
        summary: impl Into<String>,
    ) -> EvolutionOutcome {
        self.record_evolution_outcome_inner(tentacle_id, candidate_id, status, summary, true)
    }

    pub fn record_repair_linked_evolution_outcome(
        &mut self,
        tentacle_id: impl Into<String>,
        candidate_id: impl Into<String>,
        status: Status,
        summary: impl Into<String>,
    ) -> EvolutionOutcome {
        self.record_evolution_outcome_inner(tentacle_id, candidate_id, status, summary, false)
    }

    fn record_evolution_outcome_inner(
        &mut self,
        tentacle_id: impl Into<String>,
        candidate_id: impl Into<String>,
        status: Status,
        summary: impl Into<String>,
        record_pet_event: bool,
    ) -> EvolutionOutcome {
        let tentacle_id = tentacle_id.into();
        let candidate_id = candidate_id.into();
        let summary = summary.into();
        let score = evolution_status_score(&status);
        let outcome = EvolutionOutcome {
            index: self.evolution_outcomes.len() as u64 + 1,
            tentacle_id: tentacle_id.clone(),
            candidate_id: candidate_id.clone(),
            status: status.clone(),
            score,
            summary: summary.clone(),
        };
        let feed = Feed {
            need: Need::new(
                NeedKind::Execute,
                format!("evolution outcome {tentacle_id} {candidate_id}"),
            ),
            status,
            evidence: vec![Evidence::new("evolution", summary)],
            summary: outcome.summary.clone(),
            metadata: BTreeMap::from([(
                "tentacle".to_string(),
                format!("evolve:{tentacle_id}:{candidate_id}"),
            )]),
        };
        self.routes.learn(&feed);
        if record_pet_event {
            self.record_pet_event(
                "harness",
                "evolve score",
                outcome.summary.clone(),
                outcome.status.clone(),
            );
        }
        self.evolution_outcomes.push(outcome.clone());
        outcome
    }

    pub fn recent_evolution_outcomes(
        &self,
        tentacle_id: &str,
        limit: usize,
    ) -> Vec<EvolutionOutcome> {
        let mut outcomes = self
            .evolution_outcomes
            .iter()
            .rev()
            .filter(|outcome| outcome.tentacle_id == tentacle_id)
            .take(limit)
            .cloned()
            .collect::<Vec<_>>();
        outcomes.reverse();
        outcomes
    }

    pub fn self_iteration_plan(
        &self,
        repository: impl Into<String>,
        objective: Option<&str>,
    ) -> SelfIterationPlan {
        let repository = repository.into();
        let objective = objective
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string)
            .or_else(|| self.goal.as_ref().map(|goal| goal.objective.clone()))
            .unwrap_or_else(|| "improve Octopus usability".to_string());
        let grant = self.active_grant("github", &repository);
        let authorized = grant.is_some();
        let checks = self_iteration_checks();
        let guardrails = self_iteration_guardrails();
        let candidates = vec![patch_candidate("goal", &objective)];
        let mut steps = vec![
            "inspect repository health".to_string(),
            "compare docs, manifests, and tests".to_string(),
            "propose the smallest useful patch".to_string(),
        ];
        if authorized {
            steps.extend([
                "create a branch inside the granted scope".to_string(),
                "run checks before publishing".to_string(),
                "open a pull request with compact evidence".to_string(),
            ]);
        } else {
            steps.push("report only until an OAuth grant exists".to_string());
        }
        let draft = authorized.then(|| PullRequestDraft {
            branch: format!("octopus/{}", slug(&objective)),
            title: draft_title(&objective),
            change_summary: objective.clone(),
            body: pull_request_body(&repository, &objective, &checks, &guardrails),
            check_results: checks
                .iter()
                .map(|check| format!("pending: {check}"))
                .collect(),
        });
        let next_action = if authorized {
            format!(
                "OCTOPUS_PR_DRY_RUN=1 octopus self-iterate pr {} {}",
                shell_arg(&repository),
                shell_arg(&objective)
            )
        } else {
            format!("octopus oauth github {}", shell_arg(&repository))
        };
        SelfIterationPlan {
            repository,
            authorized,
            mode: if authorized {
                "pr-ready".to_string()
            } else {
                "report-only".to_string()
            },
            next_action,
            grant_id: grant.map(|grant| grant.id.clone()),
            steps,
            checks,
            guardrails,
            candidates,
            draft,
        }
    }
}

pub struct Harness {
    pub state: HarnessState,
    tentacles: Vec<Box<dyn Tentacle>>,
    manifest_llm_factory: Option<ChatClientFactory>,
}

impl Harness {
    pub fn new() -> Self {
        Self {
            state: HarnessState::default(),
            tentacles: Vec::new(),
            manifest_llm_factory: None,
        }
    }

    pub fn with_state(state: HarnessState) -> Self {
        let mut harness = Self {
            state,
            tentacles: Vec::new(),
            manifest_llm_factory: None,
        };
        harness.add_installed_tentacles_with_grants();
        harness
    }

    pub fn with_state_and_manifest_llm(
        state: HarnessState,
        config: OpenAiCompatibleConfig,
    ) -> Self {
        Self::with_state_and_manifest_llm_factory(
            state,
            chat_client_factory_from_openai_config(config),
        )
    }

    pub fn with_state_and_manifest_llm_factory(
        state: HarnessState,
        factory: ChatClientFactory,
    ) -> Self {
        let mut harness = Self {
            state,
            tentacles: Vec::new(),
            manifest_llm_factory: Some(factory),
        };
        harness.add_installed_tentacles_with_grants();
        harness
    }

    pub fn add_tentacle(&mut self, tentacle: Box<dyn Tentacle>) {
        self.tentacles.push(tentacle);
    }

    pub fn route_report(&self, need: &Need) -> RouteReport {
        let need = self.need_with_field_context(need);
        let mut options = Vec::new();
        if matches!(
            need.kind,
            NeedKind::Remember | NeedKind::Recall | NeedKind::Forget
        ) {
            options.push(self.route_option_for("memory", &need, true));
        }
        options.extend(self.tentacles.iter().map(|tentacle| {
            self.route_option_for(tentacle.name(), &need, tentacle.supports(&need))
        }));

        let supported_names = options
            .iter()
            .filter(|option| option.supported)
            .map(|option| option.tentacle.clone())
            .collect::<Vec<_>>();
        let selected = self.state.routes.choose(&need, &supported_names);
        if let Some(decision) = &selected {
            for option in &mut options {
                option.selected = option.tentacle == decision.tentacle;
                if option.selected {
                    option.reason = decision.reason.clone();
                }
            }
        }
        options.sort_by(|left, right| {
            right
                .selected
                .cmp(&left.selected)
                .then_with(|| right.supported.cmp(&left.supported))
                .then_with(|| right.score.total_cmp(&left.score))
                .then_with(|| left.tentacle.cmp(&right.tentacle))
        });

        let score_prefix = format!("{}:", kind_key(&need.kind));
        let learned_scores = self
            .state
            .routes
            .scores
            .iter()
            .filter(|(key, _)| key.starts_with(&score_prefix))
            .map(|(key, score)| (key.clone(), *score))
            .collect::<BTreeMap<_, _>>();
        let mut next = Vec::new();
        if options.is_empty() {
            next.push("octopus adapt".to_string());
            next.push("octopus install swe-agent".to_string());
        } else if selected.is_none() {
            next.push(format!(
                "octopus install {}",
                default_tentacle_profiles()
                    .first()
                    .map(|profile| profile.id.as_str())
                    .unwrap_or("swe-agent")
            ));
        } else {
            next.push(format!(
                "octopus need {} {}",
                kind_key(&need.kind),
                need.query
            ));
            next.push("octopus traces".to_string());
            next.push("octopus feedback latest satisfied|partial|failed".to_string());
        }

        RouteReport {
            need,
            selected,
            candidates: options,
            learned_scores,
            next,
        }
    }

    fn route_option_for(&self, tentacle: &str, need: &Need, supported: bool) -> RouteOption {
        let route_key = route_key(&need.kind, tentacle);
        let score = self.state.routes.score(&need.kind, tentacle);
        let matching_traces = self
            .state
            .feed_traces
            .iter()
            .filter(|trace| {
                trace.need_kind == need.kind && trace.tentacle.as_deref() == Some(tentacle)
            })
            .collect::<Vec<_>>();
        let last_trace = matching_traces.last().map(|trace| (*trace).clone());
        let reason = if supported {
            format!("{route_key}={score:.2}")
        } else {
            "tentacle does not advertise this Need".to_string()
        };
        RouteOption {
            tentacle: tentacle.to_string(),
            route_key,
            score,
            supported,
            selected: false,
            reason,
            trace_count: matching_traces.len(),
            last_trace,
        }
    }

    fn add_installed_tentacles_with_grants(&mut self) {
        let grants = self.state.grants.clone();
        for tentacle in self.state.installed_tentacles.clone() {
            self.add_tentacle(Box::new(ManifestTentacle::new_with_llm_and_grants(
                tentacle,
                self.manifest_llm_factory.clone(),
                grants.clone(),
            )));
        }
    }

    fn need_with_field_context(&self, need: &Need) -> Need {
        let mut need = need.clone();
        if let Ok(catalog) = default_field_pack_catalog() {
            if let Some(selection) =
                select_field_pack(&catalog.packs, self.state.goal.as_ref(), &need)
            {
                annotate_need_with_field(&mut need, &selection);
                self.state
                    .annotate_field_mini_task_context(&catalog.packs, &mut need);
            }
        }
        need
    }

    pub fn feed_one(&mut self, need: &Need) -> Feed {
        let need = self.need_with_field_context(need);
        if let Some(feed) = self.memory_feed(&need) {
            let mut feed = feed;
            attach_need_context_metadata(&mut feed, &need);
            self.state.record_pet_event_from_feed(&feed);
            let trace = self.state.record_feed_trace_from_feed(&feed);
            feed.metadata
                .insert("feed_trace_index".to_string(), trace.index.to_string());
            return feed;
        }
        let candidates = self
            .tentacles
            .iter()
            .enumerate()
            .filter(|(_, tentacle)| tentacle.supports(&need))
            .map(|(index, tentacle)| (index, tentacle.name().to_string()))
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            let mut feed = Feed::unsupported(&need, "no tentacle supports this need");
            attach_need_context_metadata(&mut feed, &need);
            self.state.record_pet_event_from_feed(&feed);
            let trace = self.state.record_feed_trace_from_feed(&feed);
            feed.metadata
                .insert("feed_trace_index".to_string(), trace.index.to_string());
            return feed;
        }

        let names = candidates
            .iter()
            .map(|(_, name)| name.clone())
            .collect::<Vec<_>>();
        let decision = self
            .state
            .routes
            .choose(&need, &names)
            .expect("candidates exist");
        let index = candidates
            .iter()
            .find(|(_, name)| name == &decision.tentacle)
            .map(|(index, _)| *index)
            .expect("chosen tentacle exists");
        let mut feed = self.tentacles[index].feed(&need);
        attach_need_context_metadata(&mut feed, &need);
        feed.metadata
            .insert("tentacle".to_string(), decision.tentacle);
        feed.metadata.insert("route".to_string(), decision.reason);
        if !feed_is_authorization_blocked(&feed) {
            self.state.routes.learn(&feed);
        }
        self.state.record_pet_event_from_feed(&feed);
        let trace = self.state.record_feed_trace_from_feed(&feed);
        feed.metadata
            .insert("feed_trace_index".to_string(), trace.index.to_string());
        feed
    }

    pub fn feed(&mut self, needs: &[Need]) -> Feedback {
        let feeds = needs.iter().map(|need| self.feed_one(need)).collect();
        Feedback::from_feeds(feeds)
    }

    pub fn chat_with_client<C>(
        &mut self,
        message: impl Into<String>,
        client: &mut C,
    ) -> Result<GoalChat, String>
    where
        C: ChatClient,
    {
        let message = message.into();
        let refinement = goal_refinement_from_chat(self.state.goal.as_ref(), &message, client)?;
        Ok(self.chat_with_refinement(message, refinement))
    }

    pub fn chat_with_refinement(
        &mut self,
        message: impl Into<String>,
        refinement: GoalRefinement,
    ) -> GoalChat {
        let message = message.into();
        let mut goal = match self.state.goal.take() {
            Some(mut goal) => {
                let constraints = if refinement.constraints.is_empty() {
                    vec![message.clone()]
                } else {
                    refinement.constraints.clone()
                };
                for constraint in constraints {
                    if let Some(constraint) = clean_optional(Some(constraint.as_str())) {
                        goal.refine(constraint.to_string());
                    }
                }
                if let Some(objective) = clean_optional(refinement.objective.as_deref()) {
                    goal.objective = objective.to_string();
                }
                goal
            }
            None => {
                let mut goal = Goal::new(
                    clean_optional(refinement.objective.as_deref())
                        .unwrap_or(message.as_str())
                        .to_string(),
                );
                for constraint in &refinement.constraints {
                    if let Some(constraint) = clean_optional(Some(constraint.as_str())) {
                        goal.refine(constraint.to_string());
                    }
                }
                goal
            }
        };
        goal.signals
            .insert("chat_source".to_string(), refinement.source());
        if !refinement.needs.is_empty() {
            let suggested = refinement
                .needs
                .iter()
                .map(|need| format!("{}: {}", kind_key(&need.kind), need.query))
                .collect::<Vec<_>>()
                .join(" | ");
            goal.signals
                .insert("suggested_needs".to_string(), suggested);
        }
        let need = goal.need(NeedKind::Remember, format!("goal update: {message}"));
        let feed = self.feed_one(&need);
        let feedback = Feedback::from_feeds(vec![feed.clone()]);
        let summary = clean_optional(refinement.summary.as_deref())
            .map(str::to_string)
            .unwrap_or_else(|| feed.summary.clone());
        let turn = GoalTurn {
            index: self.state.goal_turns.len() as u64 + 1,
            message,
            summary,
            status: feed.status,
        };
        self.state.goal = Some(goal.clone());
        self.state.goal_turns.push(turn.clone());
        let queued = refinement
            .needs
            .iter()
            .map(|need| {
                self.state.queue_need_suggestion(
                    need.clone(),
                    "goal chat",
                    refinement.source(),
                    "queued clean-brain Need from chat",
                )
            })
            .collect::<Vec<_>>();
        GoalChat {
            goal,
            turn,
            feedback,
            queued,
        }
    }

    fn memory_feed(&mut self, need: &Need) -> Option<Feed> {
        match need.kind {
            NeedKind::Remember => {
                let record = self.state.memory.remember(need.query.clone());
                let mut feed = Feed::satisfied(need, format!("remembered {}", record.id), "memory");
                feed.metadata
                    .insert("tentacle".to_string(), "memory".to_string());
                self.state.routes.learn(&feed);
                Some(feed)
            }
            NeedKind::Recall => {
                let records = self.state.memory.recall(&need.query, 5);
                let summary = if records.is_empty() {
                    "nothing recalled".to_string()
                } else {
                    records
                        .iter()
                        .map(|record| record.text.as_str())
                        .collect::<Vec<_>>()
                        .join("\n")
                };
                let mut feed = Feed::satisfied(need, summary, "memory");
                feed.metadata
                    .insert("tentacle".to_string(), "memory".to_string());
                self.state.routes.learn(&feed);
                Some(feed)
            }
            NeedKind::Forget => {
                let count = self.state.memory.forget(&need.query);
                let mut feed = Feed::satisfied(need, format!("forgot {count} memories"), "memory");
                feed.metadata
                    .insert("tentacle".to_string(), "memory".to_string());
                self.state.routes.learn(&feed);
                Some(feed)
            }
            _ => None,
        }
    }
}

impl Default for Harness {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InstalledToolRef {
    id: String,
    description: String,
    input: String,
    output: String,
    kind: String,
    entrypoint: String,
    contract: Option<String>,
    permission: Option<ToolPermission>,
}

impl InstalledToolRef {
    fn parse(value: &str) -> Option<Self> {
        let mut parts = value.splitn(3, ':');
        Some(Self {
            id: parts.next()?.to_string(),
            description: String::new(),
            input: String::new(),
            output: String::new(),
            kind: parts.next()?.to_string(),
            entrypoint: parts.next()?.to_string(),
            contract: None,
            permission: None,
        })
    }

    fn from_installed(tool: &InstalledTool) -> Self {
        Self {
            id: tool.id.clone(),
            description: tool.description.clone(),
            input: tool.input.clone(),
            output: tool.output.clone(),
            kind: tool.kind.clone(),
            entrypoint: tool.entrypoint.clone(),
            contract: tool.contract.clone(),
            permission: tool.permission.clone(),
        }
    }
}

fn installed_tentacle_tools(tentacle: &InstalledTentacle) -> Vec<InstalledToolRef> {
    if tentacle.tool_meta.is_empty() {
        return tentacle
            .tools
            .iter()
            .filter_map(|tool| InstalledToolRef::parse(tool))
            .collect();
    }
    tentacle
        .tool_meta
        .iter()
        .map(InstalledToolRef::from_installed)
        .collect()
}

fn tool_contract_label(tool: &InstalledToolRef) -> &str {
    tool.contract.as_deref().unwrap_or("unspecified")
}

fn tool_result_contract(tool: &InstalledToolRef, uses_json_contract: bool) -> Option<String> {
    if uses_json_contract {
        Some(
            tool.contract
                .clone()
                .unwrap_or_else(|| OCTOPUS_JSON_CONTRACT.to_string()),
        )
    } else {
        tool.contract.clone()
    }
}

fn tool_permission_text(permission: &ToolPermission) -> String {
    let permissions = if permission.permissions.is_empty() {
        "none".to_string()
    } else {
        permission.permissions.join(",")
    };
    format!("{}:{}:{permissions}", permission.provider, permission.scope)
}

#[derive(Serialize)]
struct ToolInvocation<'a> {
    schema_version: &'static str,
    need: &'a Need,
    tool: ToolInvocationTool<'a>,
    tentacle: ToolInvocationTentacle<'a>,
}

#[derive(Serialize)]
struct ToolInvocationTool<'a> {
    id: &'a str,
    description: &'a str,
    input: &'a str,
    output: &'a str,
    runtime: &'a str,
    entrypoint: &'a str,
    contract: Option<&'a str>,
    permission: Option<&'a ToolPermission>,
}

#[derive(Serialize)]
struct ToolInvocationTentacle<'a> {
    id: &'a str,
    brain_kind: &'a str,
    brain_prompt: &'a str,
    feedback_contract: Option<&'a str>,
}

#[derive(Deserialize)]
struct ToolOutputEnvelope {
    status: Option<Status>,
    output: String,
    #[serde(default)]
    evidence: Vec<Evidence>,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ManifestToolCall {
    tool: InstalledToolRef,
    reason: String,
    payload: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ManifestToolPlan {
    tool: InstalledToolRef,
    reason: String,
    candidates: Vec<String>,
    source: String,
    calls: Vec<ManifestToolCall>,
}

fn manifest_tool_calls_from_plan(plan: &Plan, tools: &[InstalledToolRef]) -> Vec<ManifestToolCall> {
    plan.calls
        .iter()
        .filter_map(|call| {
            tools
                .iter()
                .find(|tool| tool.id == call.tool)
                .cloned()
                .map(|tool| ManifestToolCall {
                    tool,
                    reason: call.reason.clone(),
                    payload: call.payload.clone(),
                })
        })
        .take(MAX_TENTACLE_ACTIONS)
        .collect()
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleToolCandidate {
    pub id: String,
    pub description: String,
    pub runtime: String,
    pub entrypoint: String,
    pub contract: Option<String>,
    pub permission: Option<String>,
    pub authorization_required: bool,
    pub active_grant: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleToolAction {
    pub tool: TentacleToolCandidate,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleThinkingPlan {
    pub tentacle_id: String,
    pub brain_kind: String,
    pub brain_prompt: String,
    pub feedback_contract: Option<String>,
    pub context_policy: String,
    pub need: Need,
    pub selected_tool: TentacleToolCandidate,
    pub plan_source: String,
    pub reason: String,
    #[serde(default)]
    pub actions: Vec<TentacleToolAction>,
    pub candidates: Vec<TentacleToolCandidate>,
}

pub struct ManifestTentacle {
    route_id: String,
    installed: InstalledTentacle,
    tools: Vec<InstalledToolRef>,
    llm_factory: Option<ChatClientFactory>,
    grants: Vec<CapabilityGrant>,
}

impl ManifestTentacle {
    pub fn new(installed: InstalledTentacle) -> Self {
        Self::new_with_llm(installed, None)
    }

    pub fn new_with_llm(
        installed: InstalledTentacle,
        llm_config: Option<OpenAiCompatibleConfig>,
    ) -> Self {
        Self::new_with_llm_and_grants(
            installed,
            llm_config.map(chat_client_factory_from_openai_config),
            Vec::new(),
        )
    }

    pub fn new_with_llm_factory(
        installed: InstalledTentacle,
        llm_factory: Option<ChatClientFactory>,
    ) -> Self {
        Self::new_with_llm_and_grants(installed, llm_factory, Vec::new())
    }

    pub fn new_with_llm_and_grants(
        installed: InstalledTentacle,
        llm_factory: Option<ChatClientFactory>,
        grants: Vec<CapabilityGrant>,
    ) -> Self {
        let route_id = installed.id.clone();
        let tools = if installed.tool_meta.is_empty() {
            installed
                .tools
                .iter()
                .filter_map(|tool| InstalledToolRef::parse(tool))
                .collect()
        } else {
            installed
                .tool_meta
                .iter()
                .map(InstalledToolRef::from_installed)
                .collect()
        };
        Self {
            route_id,
            installed,
            tools,
            llm_factory,
            grants,
        }
    }

    fn plan_tool(&mut self, need: &Need) -> Option<ManifestToolPlan> {
        let tools = self.available_tools(need);
        if let Some(plan) = self.structured_field_mini_task_plan(need, &tools) {
            return Some(plan);
        }
        self.llm_plan_tool(need, &tools)
    }

    fn structured_field_mini_task_plan(
        &self,
        need: &Need,
        tools: &[InstalledToolRef],
    ) -> Option<ManifestToolPlan> {
        if self.installed.id != "field-mini-task" {
            return None;
        }
        let field = need.context.get("field_pack")?;
        let mini_task = need.context.get("field_mini_task")?;
        if field.trim().is_empty() || mini_task.trim().is_empty() {
            return None;
        }
        let tool = tools
            .iter()
            .find(|tool| tool.id == "run_field_mini_task")
            .cloned()?;
        let calls = vec![ManifestToolCall {
            tool: tool.clone(),
            reason: format!("run structured field mini task {field}/{mini_task}"),
            payload: BTreeMap::new(),
        }];
        Some(ManifestToolPlan {
            tool,
            reason: format!("structured Need selected field mini task {field}/{mini_task}"),
            candidates: tools.iter().map(|tool| tool.id.clone()).collect(),
            source: "structured-need".to_string(),
            calls,
        })
    }

    fn thinking_plan(&mut self, need: &Need) -> Option<TentacleThinkingPlan> {
        let plan = self.plan_tool(need)?;
        let candidate_ids = plan.candidates.clone();
        let actions = plan
            .calls
            .iter()
            .map(|call| TentacleToolAction {
                tool: self.thinking_candidate(&call.tool),
                reason: call.reason.clone(),
            })
            .collect::<Vec<_>>();
        let candidates = candidate_ids
            .iter()
            .filter_map(|id| self.tools.iter().find(|tool| &tool.id == id))
            .map(|tool| self.thinking_candidate(tool))
            .collect::<Vec<_>>();
        Some(TentacleThinkingPlan {
            tentacle_id: self.route_id.clone(),
            brain_kind: self.installed.brain_kind.clone(),
            brain_prompt: self.installed.brain_prompt.clone(),
            feedback_contract: self.installed.feedback_contract.clone(),
            context_policy: TENTACLE_CONTEXT_POLICY.to_string(),
            need: need.clone(),
            selected_tool: self.thinking_candidate(&plan.tool),
            plan_source: plan.source,
            reason: plan.reason,
            actions,
            candidates,
        })
    }

    fn available_tools(&self, _need: &Need) -> Vec<InstalledToolRef> {
        self.tools
            .iter()
            .filter(|tool| self.can_feed_tool(tool))
            .cloned()
            .collect()
    }

    fn llm_plan_tool(
        &mut self,
        need: &Need,
        tools: &[InstalledToolRef],
    ) -> Option<ManifestToolPlan> {
        let factory = self.llm_factory.clone()?;
        if tools.is_empty() {
            return None;
        }
        let tool_sheet = tools
            .iter()
            .map(|tool| {
                format!(
                    "- {}: {} | runtime={} | contract={} | permission={} | input={} | output={}",
                    tool.id,
                    tool.description,
                    tool.kind,
                    tool_contract_label(tool),
                    tool_permission_label(tool.permission.as_ref()),
                    tool.input,
                    tool.output
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let need_context =
            serde_json::to_string(&need.context).unwrap_or_else(|_| "{}".to_string());
        let mut client = (factory)().ok()?;
        let mut messages = vec![
            ChatMessage::new(
                ChatRole::System,
                format!(
                    "You are the '{}' Octopus tentacle brain. {}\nReturn only JSON: {{\"calls\":[{{\"tool\":\"id\",\"reason\":\"short\",\"payload\":{{\"query\":\"optional per-action input\"}}}}],\"summary\":\"short\"}}",
                    self.route_id, self.installed.brain_prompt
                ),
            ),
            ChatMessage::new(
                ChatRole::User,
                format!(
                    "Need: {}: {}\nNeed context JSON: {}\nAvailable tools:\n{}",
                    kind_key(&need.kind),
                    need.query,
                    need_context,
                    tool_sheet
                ),
            ),
        ];
        let response = client.chat(&messages).ok()?;
        let mut plan = plan_from_llm_content(&response.content).unwrap_or_else(|error| Plan {
            calls: Vec::new(),
            summary: error,
        });
        let mut calls = manifest_tool_calls_from_plan(&plan, tools);
        let mut source = "llm".to_string();
        if calls.is_empty() {
            messages.push(ChatMessage::new(ChatRole::Assistant, response.content));
            messages.push(ChatMessage::new(
                ChatRole::User,
                format!(
                    "The previous response did not select a valid tool from [{}]. Return only valid Plan JSON with at least one call.",
                    tools
                        .iter()
                        .map(|tool| tool.id.as_str())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            ));
            if let Ok(retry_response) = client.chat(&messages) {
                if let Ok(retry_plan) = plan_from_llm_content(&retry_response.content) {
                    let retry_calls = manifest_tool_calls_from_plan(&retry_plan, tools);
                    if !retry_calls.is_empty() {
                        plan = retry_plan;
                        calls = retry_calls;
                        source = "llm-retry".to_string();
                    }
                }
            }
        }
        let first = calls.first()?;
        let tool = first.tool.clone();
        let candidates = tools.iter().map(|tool| tool.id.clone()).collect::<Vec<_>>();
        let reason = if calls.len() > 1 {
            let steps = calls
                .iter()
                .map(|call| {
                    if call.reason.trim().is_empty() {
                        call.tool.id.clone()
                    } else {
                        format!("{}({})", call.tool.id, call.reason)
                    }
                })
                .collect::<Vec<_>>()
                .join(" -> ");
            if plan.summary.trim().is_empty() {
                format!(
                    "llm planned {} actions for {}",
                    calls.len(),
                    kind_key(&need.kind)
                )
            } else {
                format!(
                    "llm planned {} actions for {}: {} :: {}",
                    calls.len(),
                    kind_key(&need.kind),
                    plan.summary,
                    steps
                )
            }
        } else if first.reason.is_empty() {
            format!("llm selected {} for {}", tool.id, kind_key(&need.kind))
        } else {
            format!(
                "llm selected {} for {}: {}",
                tool.id,
                kind_key(&need.kind),
                first.reason
            )
        };
        Some(ManifestToolPlan {
            tool,
            reason,
            candidates,
            source,
            calls,
        })
    }

    fn tool_path(&self, tool: &InstalledToolRef) -> PathBuf {
        let entrypoint = Path::new(&tool.entrypoint);
        if entrypoint.is_absolute() {
            return entrypoint.to_path_buf();
        }
        Path::new(&self.installed.source)
            .parent()
            .map(|parent| parent.join(entrypoint))
            .unwrap_or_else(|| entrypoint.to_path_buf())
    }

    fn can_feed_tool(&self, tool: &InstalledToolRef) -> bool {
        tool.kind == "static-html"
            || (tool.kind == "http"
                && (tool.entrypoint.starts_with("https://")
                    || tool.entrypoint.starts_with("http://")))
            || self.tool_path(tool).exists()
    }

    fn run_tool(&self, call: &ManifestToolCall, need: &Need) -> ToolResult {
        let tool = &call.tool;
        let active_grant = match self.authorize_tool(tool) {
            Ok(grant) => grant.map(|grant| grant.id.clone()),
            Err(result) => return result,
        };
        let mut action_need = need.clone();
        if let Some(query) = tool_call_query(call) {
            action_need.query = query.to_string();
        }
        let path = self.tool_path(tool);
        if tool.kind == "static-html" {
            let target = if path.exists() {
                path.to_string_lossy().to_string()
            } else {
                tool.entrypoint.clone()
            };
            let mut result = ToolResult::satisfied(&tool.id, format!("static artifact: {target}"));
            result.metadata.insert("entrypoint".to_string(), target);
            result
                .metadata
                .insert("runtime".to_string(), tool.kind.clone());
            if let Some(contract) = tool.contract.as_ref() {
                result
                    .metadata
                    .insert("contract".to_string(), contract.clone());
            }
            self.record_tool_permission_metadata(tool, active_grant.as_deref(), &mut result);
            return result;
        }
        if tool.kind != "http" && !path.exists() {
            return ToolResult::failed(format!(
                "missing {} tool entrypoint: {}",
                tool.kind,
                path.display()
            ));
        }

        let uses_json_contract =
            tool.contract.as_deref() == Some(OCTOPUS_JSON_CONTRACT) || tool.kind == "http";
        let mut command = match tool_command(tool, &path) {
            Ok(command) => command,
            Err(error) => return ToolResult::failed(error),
        };
        let stdin_input = tool_stdin_input(call, &action_need.query);
        if uses_json_contract {
            command.stdin(Stdio::piped());
        } else {
            for arg in tool_argv(call, tool, &action_need.query) {
                command.arg(arg);
            }
            if stdin_input.is_some() {
                command.stdin(Stdio::piped());
            }
        }
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                return ToolResult::failed(format!("{} failed to start: {error}", tool.id))
            }
        };
        if uses_json_contract {
            let input = match self.tool_invocation_json(tool, &action_need) {
                Ok(input) => input,
                Err(error) => return ToolResult::failed(error),
            };
            if let Some(mut stdin) = child.stdin.take() {
                if let Err(error) = stdin.write_all(input.as_bytes()) {
                    return ToolResult::failed(format!("{} stdin failed: {error}", tool.id));
                }
            }
        } else if let Some(input) = stdin_input {
            if let Some(mut stdin) = child.stdin.take() {
                if let Err(error) = stdin.write_all(input.as_bytes()) {
                    return ToolResult::failed(format!("{} stdin failed: {error}", tool.id));
                }
            }
        }
        let output = match child.wait_with_output() {
            Ok(output) => output,
            Err(error) => return ToolResult::failed(format!("{} failed: {error}", tool.id)),
        };
        let raw_text = String::from_utf8_lossy(if output.stdout.is_empty() {
            &output.stderr
        } else {
            &output.stdout
        })
        .to_string();
        let text = trim_output(&raw_text);
        let mut result = if output.status.success() {
            if uses_json_contract {
                tool_result_from_json(&tool.id, raw_text.trim())
                    .or_else(|| tool_result_from_json(&tool.id, &text))
                    .unwrap_or_else(|| ToolResult::satisfied(&tool.id, text))
            } else {
                ToolResult::satisfied(&tool.id, text)
            }
        } else {
            ToolResult::failed(text)
        };
        result.metadata.insert("tool".to_string(), tool.id.clone());
        result
            .metadata
            .insert("entrypoint".to_string(), path.to_string_lossy().to_string());
        result
            .metadata
            .insert("runtime".to_string(), tool.kind.clone());
        if action_need.query != need.query {
            result
                .metadata
                .insert("action_query".to_string(), action_need.query);
        }
        if let Some(contract) = tool_result_contract(tool, uses_json_contract) {
            result.metadata.insert("contract".to_string(), contract);
        }
        self.record_tool_permission_metadata(tool, active_grant.as_deref(), &mut result);
        result.metadata.insert(
            "returncode".to_string(),
            exit_code(&output.status).to_string(),
        );
        result
    }

    fn authorize_tool<'a>(
        &'a self,
        tool: &InstalledToolRef,
    ) -> Result<Option<&'a CapabilityGrant>, ToolResult> {
        let Some(permission) = &tool.permission else {
            return Ok(None);
        };
        match active_grant_for_tool(tool, &self.grants) {
            Some(grant) => Ok(Some(grant)),
            None => Err(tool_authorization_result(tool, permission)),
        }
    }

    fn record_tool_permission_metadata(
        &self,
        tool: &InstalledToolRef,
        active_grant: Option<&str>,
        result: &mut ToolResult,
    ) {
        let Some(permission) = &tool.permission else {
            return;
        };
        result
            .metadata
            .insert("authorization_required".to_string(), "true".to_string());
        result.metadata.insert(
            "permission_provider".to_string(),
            permission.provider.clone(),
        );
        result
            .metadata
            .insert("permission_scope".to_string(), permission.scope.clone());
        result.metadata.insert(
            "required_permissions".to_string(),
            permission.permissions.join(","),
        );
        if let Some(reason) = &permission.reason {
            result
                .metadata
                .insert("permission_reason".to_string(), reason.clone());
        }
        if let Some(grant) = active_grant {
            result
                .metadata
                .insert("active_grant".to_string(), grant.to_string());
        }
    }

    fn thinking_candidate(&self, tool: &InstalledToolRef) -> TentacleToolCandidate {
        let active_grant = active_grant_for_tool(tool, &self.grants).map(|grant| grant.id.clone());
        TentacleToolCandidate {
            id: tool.id.clone(),
            description: tool.description.clone(),
            runtime: tool.kind.clone(),
            entrypoint: tool.entrypoint.clone(),
            contract: tool.contract.clone(),
            permission: tool
                .permission
                .as_ref()
                .map(|permission| tool_permission_label(Some(permission))),
            authorization_required: tool.permission.is_some(),
            active_grant,
        }
    }

    fn tool_invocation_json(&self, tool: &InstalledToolRef, need: &Need) -> Result<String, String> {
        serde_json::to_string(&ToolInvocation {
            schema_version: OCTOPUS_TOOL_CALL_SCHEMA,
            need,
            tool: ToolInvocationTool {
                id: &tool.id,
                description: &tool.description,
                input: &tool.input,
                output: &tool.output,
                runtime: &tool.kind,
                entrypoint: &tool.entrypoint,
                contract: tool.contract.as_deref(),
                permission: tool.permission.as_ref(),
            },
            tentacle: ToolInvocationTentacle {
                id: &self.installed.id,
                brain_kind: &self.installed.brain_kind,
                brain_prompt: &self.installed.brain_prompt,
                feedback_contract: self.installed.feedback_contract.as_deref(),
            },
        })
        .map_err(|error| error.to_string())
    }

    fn plan_evidence(&self, tool: &InstalledToolRef, plan: &ManifestToolPlan) -> Evidence {
        let mut evidence = Evidence::new(
            "tentacle_plan",
            format!(
                "{} selected {} with {} planning",
                self.route_id, tool.id, plan.source
            ),
        );
        evidence.confidence = if plan.source == "llm" { 0.9 } else { 0.7 };
        evidence.metadata.insert(
            "context_policy".to_string(),
            TENTACLE_CONTEXT_POLICY.to_string(),
        );
        evidence
            .metadata
            .insert("tentacle".to_string(), self.route_id.clone());
        evidence
            .metadata
            .insert("tool".to_string(), tool.id.clone());
        evidence
            .metadata
            .insert("plan_source".to_string(), plan.source.clone());
        evidence
            .metadata
            .insert("reason".to_string(), plan.reason.clone());
        evidence
            .metadata
            .insert("available_tools".to_string(), plan.candidates.join(","));
        evidence.metadata.insert(
            "tools".to_string(),
            plan.calls
                .iter()
                .map(|call| call.tool.id.as_str())
                .collect::<Vec<_>>()
                .join(","),
        );
        evidence
            .metadata
            .insert("action_count".to_string(), plan.calls.len().to_string());
        evidence
    }
}

fn tool_call_query(call: &ManifestToolCall) -> Option<String> {
    call.payload
        .get("query")
        .or_else(|| call.payload.get("input"))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn tool_stdin_input(call: &ManifestToolCall, default_query: &str) -> Option<String> {
    for key in ["stdin", "script", "patch", "content", "body"] {
        if let Some(value) = call
            .payload
            .get(key)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            return Some(value);
        }
    }
    if tool_declares_stdin(&call.tool) {
        Some(default_tool_query(default_query))
    } else {
        None
    }
}

fn tool_argv(call: &ManifestToolCall, tool: &InstalledToolRef, default_query: &str) -> Vec<String> {
    for key in ["argv", "args"] {
        if let Some(value) = call.payload.get(key) {
            let args = split_tool_args(value);
            if !args.is_empty() {
                return args;
            }
        }
    }
    let numbered = (1..=16)
        .filter_map(|index| {
            call.payload
                .get(&format!("arg{index}"))
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .collect::<Vec<_>>();
    if !numbered.is_empty() {
        return numbered;
    }
    if tool_declares_stdin(tool) || tool_declares_no_input(tool) {
        return Vec::new();
    }
    tool_call_query(call)
        .map(|query| split_tool_args(&query))
        .unwrap_or_else(|| split_tool_args(default_query))
}

fn tool_declares_stdin(tool: &InstalledToolRef) -> bool {
    tool.input.to_ascii_lowercase().contains("stdin")
}

fn tool_declares_no_input(tool: &InstalledToolRef) -> bool {
    matches!(
        tool.input.trim().to_ascii_lowercase().as_str(),
        "none" | "no input"
    )
}

fn split_tool_args(value: &str) -> Vec<String> {
    let value = value.trim();
    if value.is_empty() {
        return Vec::new();
    }
    if let Ok(args) = serde_json::from_str::<Vec<String>>(value) {
        return args
            .into_iter()
            .map(|arg| arg.trim().to_string())
            .filter(|arg| !arg.is_empty())
            .collect();
    }
    value.split_whitespace().map(str::to_string).collect()
}

fn action_results_status(results: &[(ManifestToolCall, ToolResult)]) -> Status {
    if results.is_empty() {
        return Status::Failed;
    }
    let tool_results = results
        .iter()
        .map(|(_, result)| result.clone())
        .collect::<Vec<_>>();
    tool_status(&tool_results)
}

fn action_results_summary(results: &[(ManifestToolCall, ToolResult)]) -> String {
    if results.is_empty() {
        return "no tentacle action executed".to_string();
    }
    if results.len() == 1 {
        return results[0].1.output.clone();
    }
    results
        .iter()
        .enumerate()
        .map(|(index, (call, result))| {
            format!(
                "== action {}: {} ({:?}) ==\n{}",
                index + 1,
                call.tool.id,
                result.status,
                result.output
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn action_results_evidence(results: &[(ManifestToolCall, ToolResult)]) -> Vec<Evidence> {
    let mut evidence = Vec::new();
    for (index, (call, result)) in results.iter().enumerate() {
        let mut action = Evidence::new(
            "tentacle_action",
            format!(
                "action {} ran {} -> {:?}",
                index + 1,
                call.tool.id,
                result.status
            ),
        );
        action.confidence = if result.status == Status::Satisfied {
            0.9
        } else {
            0.4
        };
        action
            .metadata
            .insert("tool".to_string(), call.tool.id.clone());
        action
            .metadata
            .insert("reason".to_string(), call.reason.clone());
        action
            .metadata
            .insert("status".to_string(), status_key(&result.status).to_string());
        evidence.push(action);
        evidence.extend(result.evidence.clone());
    }
    evidence
}

impl Tentacle for ManifestTentacle {
    fn name(&self) -> &str {
        &self.route_id
    }

    fn supports(&self, need: &Need) -> bool {
        if !self
            .installed
            .needs
            .iter()
            .any(|need_key| need_key == kind_key(&need.kind))
        {
            return false;
        }
        let tools = self.available_tools(need);
        !tools.is_empty()
            && (self.llm_factory.is_some()
                || self.structured_field_mini_task_plan(need, &tools).is_some())
    }

    fn feed(&mut self, need: &Need) -> Feed {
        let Some(plan) = self.plan_tool(need) else {
            return Feed::unsupported(need, "no manifest tool supports this need");
        };
        let tool = plan.tool.clone();
        let mut action_results = Vec::new();
        for call in plan.calls.iter().take(MAX_TENTACLE_ACTIONS) {
            let result = self.run_tool(call, need);
            let should_continue = result.status == Status::Satisfied;
            action_results.push((call.clone(), result));
            if !should_continue {
                break;
            }
        }
        let status = action_results_status(&action_results);
        let mut metadata = action_results
            .last()
            .map(|(_, result)| result.metadata.clone())
            .unwrap_or_default();
        metadata.insert("tool".to_string(), tool.id.clone());
        metadata.insert("tool_description".to_string(), tool.description.clone());
        metadata.insert(
            "tools".to_string(),
            action_results
                .iter()
                .map(|(call, _)| call.tool.id.as_str())
                .collect::<Vec<_>>()
                .join(","),
        );
        metadata.insert("action_count".to_string(), action_results.len().to_string());
        metadata.insert(
            "action_plan".to_string(),
            plan.calls
                .iter()
                .map(|call| call.tool.id.as_str())
                .collect::<Vec<_>>()
                .join(" -> "),
        );
        metadata.insert("plan".to_string(), plan.reason.clone());
        metadata.insert("plan_source".to_string(), plan.source.clone());
        metadata.insert("available_tools".to_string(), plan.candidates.join(","));
        metadata.insert(
            "context_policy".to_string(),
            TENTACLE_CONTEXT_POLICY.to_string(),
        );
        metadata.insert("runtime".to_string(), tool.kind.clone());
        metadata.insert("tentacle_brain".to_string(), self.route_id.clone());
        metadata.insert(
            "brain_prompt".to_string(),
            self.installed.brain_prompt.clone(),
        );
        if let Some(contract) = &self.installed.feedback_contract {
            metadata.insert("feedback_contract".to_string(), contract.clone());
        }
        let mut evidence = action_results_evidence(&action_results);
        evidence.insert(0, self.plan_evidence(&tool, &plan));
        Feed {
            need: need.clone(),
            status,
            evidence,
            summary: action_results_summary(&action_results),
            metadata,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SkillManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub needs: Vec<NeedKind>,
    pub tools: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleBrain {
    pub kind: String,
    pub description: String,
    pub model: Option<String>,
    pub prompt: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ToolImplementation {
    pub kind: String,
    pub entrypoint: String,
    #[serde(default)]
    pub contract: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ToolPermission {
    pub provider: String,
    pub scope: String,
    pub permissions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ToolMetadata {
    pub id: String,
    pub description: String,
    pub implementation: ToolImplementation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission: Option<ToolPermission>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub brain: TentacleBrain,
    pub skills: Vec<SkillManifest>,
    pub tools: Vec<ToolMetadata>,
    pub evolution: EvolutionPolicy,
    pub llm_ready: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct InstalledTentacle {
    pub id: String,
    pub name: String,
    pub source: String,
    pub brain_kind: String,
    #[serde(default)]
    pub brain_prompt: String,
    #[serde(default)]
    pub feedback_contract: Option<String>,
    pub runtime_kinds: Vec<String>,
    pub needs: Vec<String>,
    pub tools: Vec<String>,
    #[serde(default)]
    pub tool_meta: Vec<InstalledTool>,
    pub editable: Vec<String>,
    #[serde(default)]
    pub evolution_surfaces: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct InstalledTool {
    pub id: String,
    pub description: String,
    pub input: String,
    pub output: String,
    pub kind: String,
    pub entrypoint: String,
    #[serde(default)]
    pub contract: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission: Option<ToolPermission>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleManifest {
    #[serde(default, rename = "$schema")]
    pub schema: Option<String>,
    pub schema_version: String,
    pub id: String,
    pub name: String,
    pub description: String,
    pub brain: ManifestBrain,
    pub skills: Vec<ManifestSkill>,
    pub tools: Vec<ManifestTool>,
    pub evolution: EvolutionPolicy,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ManifestBrain {
    pub kind: String,
    pub model: Option<String>,
    pub prompt: String,
    #[serde(default)]
    pub feedback_contract: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ManifestSkill {
    pub id: String,
    pub description: String,
    pub needs: Vec<NeedKind>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ManifestTool {
    pub id: String,
    pub description: String,
    pub input: String,
    pub output: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission: Option<ToolPermission>,
    pub implementation: ToolImplementation,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct LoadedTentacleManifest {
    pub path: String,
    pub manifest: TentacleManifest,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleManifestReport {
    pub id: String,
    pub name: String,
    pub path: String,
    pub brain_kind: String,
    pub runtime_kinds: Vec<String>,
    pub needs: Vec<String>,
    pub tool_count: usize,
    pub editable: Vec<String>,
    pub evolution_surfaces: Vec<String>,
    pub missing_entrypoints: Vec<String>,
}

pub fn load_tentacle_manifests(
    root: impl AsRef<Path>,
) -> Result<Vec<LoadedTentacleManifest>, Error> {
    let root = root.as_ref();
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut manifests = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path().join("manifest.json");
        if !path.exists() {
            continue;
        }
        let content = fs::read_to_string(&path)?;
        let manifest = serde_json::from_str::<TentacleManifest>(&content)
            .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        let source = fs::canonicalize(&path).unwrap_or(path);
        manifests.push(LoadedTentacleManifest {
            path: source.to_string_lossy().to_string(),
            manifest,
        });
    }
    manifests.sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));
    Ok(manifests)
}

pub fn think_tentacle(
    root: impl AsRef<Path>,
    tentacle_id: &str,
    need: &Need,
    llm_config: Option<OpenAiCompatibleConfig>,
    grants: &[CapabilityGrant],
) -> Result<TentacleThinkingPlan, String> {
    think_tentacle_with_llm_factory(
        root,
        tentacle_id,
        need,
        llm_config.map(chat_client_factory_from_openai_config),
        grants,
    )
}

pub fn think_tentacle_with_llm_factory(
    root: impl AsRef<Path>,
    tentacle_id: &str,
    need: &Need,
    llm_factory: Option<ChatClientFactory>,
    grants: &[CapabilityGrant],
) -> Result<TentacleThinkingPlan, String> {
    let root = root.as_ref();
    let loaded = load_tentacle_manifests(root)
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|loaded| loaded.manifest.id == tentacle_id)
        .ok_or_else(|| format!("unknown tentacle: {tentacle_id}"))?;
    let installed = installed_tentacle_from_manifest(root, loaded)?;
    let mut tentacle =
        ManifestTentacle::new_with_llm_and_grants(installed, llm_factory, grants.to_vec());
    tentacle
        .thinking_plan(need)
        .ok_or_else(|| format!("{tentacle_id} cannot feed {}", kind_key(&need.kind)))
}

pub fn feed_tentacle(
    root: impl AsRef<Path>,
    tentacle_id: &str,
    need: &Need,
    llm_config: Option<OpenAiCompatibleConfig>,
    grants: &[CapabilityGrant],
) -> Result<Feed, String> {
    feed_tentacle_with_llm_factory(
        root,
        tentacle_id,
        need,
        llm_config.map(chat_client_factory_from_openai_config),
        grants,
    )
}

pub fn feed_tentacle_with_llm_factory(
    root: impl AsRef<Path>,
    tentacle_id: &str,
    need: &Need,
    llm_factory: Option<ChatClientFactory>,
    grants: &[CapabilityGrant],
) -> Result<Feed, String> {
    let root = root.as_ref();
    let loaded = load_tentacle_manifests(root)
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|loaded| loaded.manifest.id == tentacle_id)
        .ok_or_else(|| format!("unknown tentacle: {tentacle_id}"))?;
    let installed = installed_tentacle_from_manifest(root, loaded)?;
    let mut tentacle =
        ManifestTentacle::new_with_llm_and_grants(installed, llm_factory, grants.to_vec());
    Ok(tentacle.feed(need))
}

pub fn inspect_tentacle_manifests(
    root: impl AsRef<Path>,
) -> Result<Vec<TentacleManifestReport>, Error> {
    let root = root.as_ref();
    let manifests = load_tentacle_manifests(root)?;
    Ok(manifests
        .into_iter()
        .map(|loaded| manifest_report(root, loaded))
        .collect())
}

pub fn propose_tentacle_evolution_with_client<C>(
    root: impl AsRef<Path>,
    tentacle_id: &str,
    objective: &str,
    state: &HarnessState,
    client: &mut C,
) -> Result<TentacleEvolutionProposal, String>
where
    C: ChatClient,
{
    let mut proposal = tentacle_evolution_context(root, tentacle_id, objective, state)?;
    let plan = evolution_llm_plan::llm_evolution_plan(&proposal, client)?;
    proposal.generator = "llm".to_string();
    proposal.planner_summary = plan.summary;
    proposal.patch_candidates = plan.candidates;
    proposal
        .next_steps
        .insert(0, "review LLM-generated harness candidates".to_string());
    Ok(proposal)
}

fn one_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn repair_outcome_metadata_value(value: Option<&String>, candidate: bool) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() {
        return None;
    }
    let lower = value.to_ascii_lowercase();
    if matches!(lower.as_str(), "unknown" | "none") || (candidate && lower == "recommended") {
        return None;
    }
    Some(value.to_string())
}

pub fn default_tentacle_profiles() -> Vec<TentacleProfile> {
    external_profile_registry()
        .and_then(|path| load_tentacle_profiles_from_path(&path).ok())
        .unwrap_or_else(embedded_tentacle_profiles)
}

pub fn embedded_profile_registry_json() -> &'static str {
    DEFAULT_PROFILE_REGISTRY
}

pub fn load_tentacle_profiles_from_path(
    path: impl AsRef<Path>,
) -> Result<Vec<TentacleProfile>, String> {
    let path = path.as_ref();
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&content).map_err(|error| error.to_string())
}

fn external_profile_registry() -> Option<PathBuf> {
    env::var(OCTOPUS_PROFILE_REGISTRY_ENV)
        .ok()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .or_else(|| {
            env::var(OCTOPUS_STATE_PATH_ENV)
                .ok()
                .map(PathBuf::from)
                .and_then(|state_path| {
                    let directory = state_path
                        .parent()
                        .filter(|path| !path.as_os_str().is_empty())
                        .unwrap_or_else(|| Path::new("."));
                    let path = directory.join("profile-registry").join("default.json");
                    path.exists().then_some(path)
                })
        })
        .or_else(|| {
            let path = PathBuf::from(LOCAL_PROFILE_REGISTRY_PATH);
            path.exists().then_some(path)
        })
}

fn embedded_tentacle_profiles() -> Vec<TentacleProfile> {
    serde_json::from_str(DEFAULT_PROFILE_REGISTRY)
        .expect("embedded tentacle profile registry must be valid")
}

fn manifest_report(root: &Path, loaded: LoadedTentacleManifest) -> TentacleManifestReport {
    let mut runtime_kinds = loaded
        .manifest
        .tools
        .iter()
        .map(|tool| tool.implementation.kind.clone())
        .collect::<Vec<_>>();
    runtime_kinds.sort();
    runtime_kinds.dedup();

    let mut needs = loaded
        .manifest
        .skills
        .iter()
        .flat_map(|skill| skill.needs.iter().map(kind_key))
        .map(str::to_string)
        .collect::<Vec<_>>();
    needs.sort();
    needs.dedup();

    let manifest_path = Path::new(&loaded.path);
    let missing_entrypoints = loaded
        .manifest
        .tools
        .iter()
        .filter(|tool| !entrypoint_exists(root, manifest_path, &tool.implementation))
        .map(|tool| tool.implementation.entrypoint.clone())
        .collect::<Vec<_>>();

    let evolution_surfaces = loaded
        .manifest
        .evolution
        .surfaces
        .iter()
        .map(|surface| surface.id.clone())
        .collect::<Vec<_>>();

    TentacleManifestReport {
        id: loaded.manifest.id,
        name: loaded.manifest.name,
        path: loaded.path,
        brain_kind: loaded.manifest.brain.kind,
        runtime_kinds,
        needs,
        tool_count: loaded.manifest.tools.len(),
        editable: loaded.manifest.evolution.editable,
        evolution_surfaces,
        missing_entrypoints,
    }
}

fn installed_tentacle_from_manifest(
    root: &Path,
    loaded: LoadedTentacleManifest,
) -> Result<InstalledTentacle, String> {
    let report = manifest_report(root, loaded.clone());
    if !report.missing_entrypoints.is_empty() {
        return Err(format!(
            "manifest has missing entrypoints: {}",
            report.missing_entrypoints.join(", ")
        ));
    }
    let tools = loaded
        .manifest
        .tools
        .iter()
        .map(|tool| {
            format!(
                "{}:{}:{}",
                tool.id, tool.implementation.kind, tool.implementation.entrypoint
            )
        })
        .collect::<Vec<_>>();
    let tool_meta = loaded
        .manifest
        .tools
        .iter()
        .map(|tool| InstalledTool {
            id: tool.id.clone(),
            description: tool.description.clone(),
            input: tool.input.clone(),
            output: tool.output.clone(),
            kind: tool.implementation.kind.clone(),
            entrypoint: tool.implementation.entrypoint.clone(),
            contract: tool.implementation.contract.clone(),
            permission: tool.permission.clone(),
        })
        .collect::<Vec<_>>();
    Ok(InstalledTentacle {
        id: report.id,
        name: report.name,
        source: report.path,
        brain_kind: report.brain_kind,
        brain_prompt: loaded.manifest.brain.prompt,
        feedback_contract: loaded.manifest.brain.feedback_contract,
        runtime_kinds: report.runtime_kinds,
        needs: report.needs,
        tools,
        tool_meta,
        editable: report.editable,
        evolution_surfaces: report.evolution_surfaces,
    })
}

fn entrypoint_exists(
    root: &Path,
    manifest_path: &Path,
    implementation: &ToolImplementation,
) -> bool {
    if implementation.kind == "http" {
        return implementation.entrypoint.starts_with("https://")
            || implementation.entrypoint.starts_with("http://");
    }
    let entrypoint = Path::new(&implementation.entrypoint);
    if entrypoint.is_absolute() && entrypoint.exists() {
        return true;
    }
    let manifest_relative = manifest_path
        .parent()
        .map(|parent| parent.join(entrypoint))
        .is_some_and(|path| path.exists());
    if manifest_relative {
        return true;
    }
    root.parent()
        .map(|parent| parent.join(entrypoint).exists())
        .unwrap_or(false)
}

pub fn default_permissions(provider: &str) -> Vec<String> {
    match provider {
        "github" => vec![
            "repo:read".to_string(),
            "checks:read".to_string(),
            "branch:write".to_string(),
            "pull_request:write".to_string(),
        ],
        "octopus" => vec!["harness:read".to_string(), "harness:write".to_string()],
        "codex" => vec!["codex:execute".to_string()],
        _ => vec!["read".to_string()],
    }
}

fn grant_id(provider: &str, scope: &str) -> String {
    format!("{provider}:{scope}")
}

fn normalize_permissions(permissions: Vec<String>) -> Vec<String> {
    let mut permissions = permissions;
    permissions.sort();
    permissions.dedup();
    permissions
}

fn self_iteration_checks() -> Vec<String> {
    vec![
        "cargo test".to_string(),
        "cargo clippy --all-targets -- -D warnings".to_string(),
    ]
}

fn self_iteration_guardrails() -> Vec<String> {
    vec![
        "never push to main directly".to_string(),
        "prefer small pull requests".to_string(),
        "keep the kernel small enough to audit".to_string(),
    ]
}

fn patch_candidate(source: &str, objective: &str) -> PatchCandidate {
    let title = draft_title(objective);
    PatchCandidate {
        id: format!("{source}:{}", slug(objective)),
        title,
        source: source.to_string(),
        reason: format!("candidate derived from {source} signal"),
        branch: format!("octopus/{}", slug(objective)),
    }
}

fn pull_request_body(
    repository: &str,
    objective: &str,
    checks: &[String],
    guardrails: &[String],
) -> String {
    format!(
        "Repository: {repository}\nObjective: {objective}\n\nChecks:\n{}\n\nGuardrails:\n{}",
        checks
            .iter()
            .map(|check| format!("- [ ] {check}"))
            .collect::<Vec<_>>()
            .join("\n"),
        guardrails
            .iter()
            .map(|guardrail| format!("- {guardrail}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn draft_title(objective: &str) -> String {
    let objective = objective.trim();
    capitalize_ascii(objective)
}

fn capitalize_ascii(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!(
        "{}{}",
        first.to_ascii_uppercase(),
        chars.collect::<String>()
    )
}

fn slug(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
        if slug.len() >= 48 {
            break;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "update".to_string()
    } else {
        slug
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EnvironmentReport {
    pub manifests: Vec<String>,
    pub commands: Vec<String>,
    pub recommended_profiles: Vec<String>,
}

impl EnvironmentReport {
    pub fn detect(cwd: impl AsRef<Path>) -> Self {
        let cwd = cwd.as_ref();
        let manifests = [
            (cwd.join(".git"), "git"),
            (cwd.join("Cargo.toml"), "rust"),
            (cwd.join("pyproject.toml"), "python"),
            (cwd.join("package.json"), "javascript"),
            (cwd.join("docs"), "docs"),
        ]
        .into_iter()
        .filter(|(path, _)| path.exists())
        .map(|(_, name)| name.to_string())
        .collect::<Vec<_>>();
        let commands = [
            "git",
            "cargo",
            "python3",
            "gh",
            "bash",
            "open",
            "screencapture",
            "xdg-open",
            "gnome-screenshot",
        ]
        .into_iter()
        .filter(|command| command_available(command))
        .map(str::to_string)
        .collect::<Vec<_>>();
        let mut recommended_profiles = vec!["memory".to_string(), "visual".to_string()];
        if manifests.iter().any(|item| item == "git") {
            recommended_profiles.push("repo-maintainer".to_string());
            recommended_profiles.push("swe-agent".to_string());
        }
        if manifests
            .iter()
            .any(|item| item == "rust" || item == "python")
        {
            recommended_profiles.push("code".to_string());
        }
        if commands.iter().any(|item| item == "python3") {
            recommended_profiles.push("json-feed".to_string());
            recommended_profiles.push("field-mini-task".to_string());
        }
        if manifests.iter().any(|item| item == "docs") {
            recommended_profiles.push("research".to_string());
        }
        if commands.iter().any(|item| item == "bash") {
            recommended_profiles.push("bash-only".to_string());
        }
        if commands.iter().any(|item| {
            item == "open"
                || item == "xdg-open"
                || item == "screencapture"
                || item == "gnome-screenshot"
        }) {
            recommended_profiles.push("computer-use-agent".to_string());
        }
        Self {
            manifests,
            commands,
            recommended_profiles,
        }
    }
}

fn kind_key(kind: &NeedKind) -> &'static str {
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
}

fn need_command(need: &GoalNeedSuggestion) -> String {
    format!("need {} {}", kind_key(&need.kind), shell_arg(&need.query))
}

fn need_queue_status_key(status: &NeedQueueStatus) -> &'static str {
    match status {
        NeedQueueStatus::Pending => "pending",
        NeedQueueStatus::Taken => "taken",
        NeedQueueStatus::Dropped => "dropped",
    }
}

type BrainSynthesisParts = (
    usize,
    String,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<GoalNeedSuggestion>,
);

fn brain_synthesis_draft_count(input: &BrainSynthesisInput) -> usize {
    let direct_count = usize::from(
        !input.observations.is_empty()
            || !input.questions.is_empty()
            || !input.options.is_empty()
            || !input.risks.is_empty()
            || !input.needs.is_empty()
            || (input.drafts.is_empty() && input.summary.is_some()),
    );
    direct_count + input.drafts.len()
}

fn merge_brain_synthesis_input(input: BrainSynthesisInput) -> BrainSynthesisParts {
    let mut draft_count = 0;
    let mut summary_candidates = Vec::new();
    let mut observations = Vec::new();
    let mut questions = Vec::new();
    let mut options = Vec::new();
    let mut risks = Vec::new();
    let mut needs = Vec::new();
    let mut text_seen = BTreeMap::new();
    let mut need_seen = BTreeMap::new();

    if let Some(summary) = clean_optional(input.summary.as_deref()) {
        summary_candidates.push(summary.to_string());
    }
    let has_direct = !input.observations.is_empty()
        || !input.questions.is_empty()
        || !input.options.is_empty()
        || !input.risks.is_empty()
        || !input.needs.is_empty()
        || (input.drafts.is_empty() && input.summary.is_some());
    if has_direct {
        draft_count += 1;
        merge_brain_synthesis_draft_parts(
            &mut observations,
            &mut questions,
            &mut options,
            &mut risks,
            &mut needs,
            &mut text_seen,
            &mut need_seen,
            input.observations,
            input.questions,
            input.options,
            input.risks,
            input.needs,
        );
    }
    for draft in input.drafts {
        draft_count += 1;
        if let Some(summary) = clean_optional(draft.summary.as_deref()) {
            summary_candidates.push(summary.to_string());
        }
        merge_brain_synthesis_draft_parts(
            &mut observations,
            &mut questions,
            &mut options,
            &mut risks,
            &mut needs,
            &mut text_seen,
            &mut need_seen,
            draft.observations,
            draft.questions,
            draft.options,
            draft.risks,
            draft.needs,
        );
    }
    let summary = summary_candidates
        .first()
        .cloned()
        .unwrap_or_else(|| format!("synthesized {draft_count} clean-brain draft(s)"));
    (
        draft_count,
        summary,
        observations,
        questions,
        options,
        risks,
        needs,
    )
}

#[allow(clippy::too_many_arguments)]
fn merge_brain_synthesis_draft_parts(
    observations: &mut Vec<String>,
    questions: &mut Vec<String>,
    options: &mut Vec<String>,
    risks: &mut Vec<String>,
    needs: &mut Vec<GoalNeedSuggestion>,
    text_seen: &mut BTreeMap<String, ()>,
    need_seen: &mut BTreeMap<String, ()>,
    draft_observations: Vec<String>,
    draft_questions: Vec<String>,
    draft_options: Vec<String>,
    draft_risks: Vec<String>,
    draft_needs: Vec<GoalNeedSuggestion>,
) {
    for value in draft_observations {
        push_unique_clean_text(observations, text_seen, &value);
    }
    for value in draft_questions {
        push_unique_clean_text(questions, text_seen, &value);
    }
    for value in draft_options {
        push_unique_clean_text(options, text_seen, &value);
    }
    for value in draft_risks {
        push_unique_clean_text(risks, text_seen, &value);
    }
    for need in draft_needs {
        if let Some(query) = clean_optional(Some(need.query.as_str())) {
            let key = format!("{}:{}", kind_key(&need.kind), query.to_lowercase());
            if need_seen.insert(key, ()).is_none() {
                needs.push(GoalNeedSuggestion {
                    kind: need.kind,
                    query: query.to_string(),
                });
            }
        }
    }
}

fn push_unique_clean_text(target: &mut Vec<String>, seen: &mut BTreeMap<String, ()>, value: &str) {
    if let Some(value) = clean_optional(Some(value)) {
        let key = value.to_lowercase();
        if seen.insert(key, ()).is_none() {
            target.push(value.to_string());
        }
    }
}

fn audit_clean_brain_needs(needs: &[GoalNeedSuggestion]) -> BrainNeedAudit {
    let issues = needs
        .iter()
        .enumerate()
        .filter_map(|(index, need)| {
            implementation_signal(&need.query).map(|signal| BrainNeedAuditIssue {
                index: index + 1,
                kind: need.kind.clone(),
                query: need.query.clone(),
                signal: signal.to_string(),
                guidance: "rewrite as a cognitive request and let Feed choose implementation"
                    .to_string(),
            })
        })
        .collect::<Vec<_>>();
    let issue_count = issues.len();
    let clean_needs = needs
        .iter()
        .enumerate()
        .filter(|(index, _)| !issues.iter().any(|issue| issue.index == index + 1))
        .map(|(_, need)| need.clone())
        .collect::<Vec<_>>();
    let clean_count = clean_needs.len();
    let status = if issue_count == 0 {
        Status::Satisfied
    } else {
        Status::Partial
    };
    let summary = if issue_count == 0 {
        "all Needs stay cognitive".to_string()
    } else {
        format!("{issue_count} Need(s) carry implementation detail")
    };
    BrainNeedAudit {
        status,
        summary,
        clean_count,
        issue_count,
        clean_needs,
        issues,
    }
}

fn implementation_signal(query: &str) -> Option<&'static str> {
    let trimmed = query.trim();
    let lower = trimmed.to_ascii_lowercase();
    let command_prefixes = [
        "cargo ", "git ", "python ", "python3 ", "node ", "npm ", "pnpm ", "yarn ", "curl ",
        "bash ", "sh ", "gh ", "octopus ",
    ];
    if trimmed.contains("```")
        || trimmed.contains("$ ")
        || command_prefixes
            .iter()
            .any(|prefix| lower.starts_with(prefix))
    {
        return Some("command");
    }
    let implementation_terms = [
        " tool",
        " api",
        " mcp",
        " shell",
        "terminal",
        "clipboard",
        "browser",
        "endpoint",
    ];
    if implementation_terms.iter().any(|term| lower.contains(term)) {
        return Some("tool_or_api");
    }
    let file_markers = [
        ".rs", ".py", ".js", ".ts", ".json", ".toml", ".yaml", ".yml", ".md", "/", "\\",
    ];
    if file_markers.iter().any(|marker| lower.contains(marker)) {
        return Some("file_or_path");
    }
    None
}

fn status_key(status: &Status) -> &'static str {
    match status {
        Status::Satisfied => "satisfied",
        Status::Partial => "partial",
        Status::Failed => "failed",
        Status::Unsupported => "unsupported",
    }
}

fn evolution_status_score(status: &Status) -> f32 {
    match status {
        Status::Satisfied => 1.0,
        Status::Partial => 0.4,
        Status::Unsupported => -0.3,
        Status::Failed => -1.0,
    }
}

fn feed_is_authorization_blocked(feed: &Feed) -> bool {
    feed.status == Status::Failed
        && feed
            .metadata
            .get("authorization_required")
            .is_some_and(|value| value == "true")
        && !feed.metadata.contains_key("active_grant")
}

fn attach_need_context_metadata(feed: &mut Feed, need: &Need) {
    for key in [
        "field_pack",
        "field_score",
        "field_reason",
        "field_verifier",
        "field_pass_signal",
        "field_mini_task",
        "field_expected_feed",
        "field_original_need",
    ] {
        if let Some(value) = need.context.get(key) {
            feed.metadata
                .entry(key.to_string())
                .or_insert_with(|| value.clone());
        }
    }
}

fn memory_id_number(id: &str) -> u64 {
    id.strip_prefix('m')
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(u64::MAX)
}

fn route_key(kind: &NeedKind, name: &str) -> String {
    format!("{}:{name}", kind_key(kind))
}

fn default_tool_query(query: &str) -> String {
    let query = query.trim();
    if query.is_empty() {
        ".".to_string()
    } else {
        query.to_string()
    }
}

fn clean_optional(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn tool_command(tool: &InstalledToolRef, path: &Path) -> Result<Command, String> {
    match tool.kind.as_str() {
        "http" => {
            if !(tool.entrypoint.starts_with("https://") || tool.entrypoint.starts_with("http://"))
            {
                return Err(format!("http tool requires URL entrypoint: {}", tool.id));
            }
            let mut command = Command::new("curl");
            command
                .arg("-sS")
                .arg("-X")
                .arg("POST")
                .arg("-H")
                .arg("content-type: application/json")
                .arg("--data-binary")
                .arg("@-")
                .arg(&tool.entrypoint);
            Ok(command)
        }
        "python" => {
            let mut command = Command::new("python3");
            command.arg(path);
            Ok(command)
        }
        "node" | "javascript" => {
            let mut command = Command::new("node");
            command.arg(path);
            Ok(command)
        }
        _ => Ok(Command::new(path)),
    }
}

fn tool_result_from_json(tool_id: &str, text: &str) -> Option<ToolResult> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if let Ok(result) = serde_json::from_str::<ToolResult>(text) {
        return Some(result);
    }
    let envelope = serde_json::from_str::<ToolOutputEnvelope>(text).ok()?;
    let evidence = if envelope.evidence.is_empty() {
        vec![Evidence::new(tool_id, envelope.output.clone())]
    } else {
        envelope.evidence
    };
    Some(ToolResult {
        status: envelope.status.unwrap_or(Status::Satisfied),
        output: envelope.output,
        evidence,
        metadata: envelope.metadata,
    })
}

fn tool_authorization_result(tool: &InstalledToolRef, permission: &ToolPermission) -> ToolResult {
    let permissions = permission.permissions.join(" ");
    let hint = if permissions.is_empty() {
        format!("octopus oauth {} {}", permission.provider, permission.scope)
    } else {
        format!(
            "octopus oauth {} {} {}",
            permission.provider, permission.scope, permissions
        )
    };
    let mut result = ToolResult::failed(format!(
        "needs_authorization: {} requires {}. run: {}",
        tool.id,
        tool_permission_label(Some(permission)),
        hint
    ));
    result.metadata.insert("tool".to_string(), tool.id.clone());
    result
        .metadata
        .insert("authorization_required".to_string(), "true".to_string());
    result.metadata.insert(
        "permission_provider".to_string(),
        permission.provider.clone(),
    );
    result
        .metadata
        .insert("permission_scope".to_string(), permission.scope.clone());
    result.metadata.insert(
        "required_permissions".to_string(),
        permission.permissions.join(","),
    );
    result.metadata.insert("grant_hint".to_string(), hint);
    if let Some(reason) = &permission.reason {
        result
            .metadata
            .insert("permission_reason".to_string(), reason.clone());
    }
    result
}

fn tool_permission_label(permission: Option<&ToolPermission>) -> String {
    match permission {
        Some(permission) if permission.permissions.is_empty() => {
            format!("{}:{}", permission.provider, permission.scope)
        }
        Some(permission) => format!(
            "{}:{} [{}]",
            permission.provider,
            permission.scope,
            permission.permissions.join(",")
        ),
        None => "none".to_string(),
    }
}

fn active_grant_for_tool<'a>(
    tool: &InstalledToolRef,
    grants: &'a [CapabilityGrant],
) -> Option<&'a CapabilityGrant> {
    let permission = tool.permission.as_ref()?;
    grants.iter().find(|grant| {
        grant.status == GrantStatus::Active
            && grant.provider == permission.provider
            && grant.scope == permission.scope
            && permission
                .permissions
                .iter()
                .all(|required| grant.permissions.iter().any(|owned| owned == required))
    })
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

fn trim_output(output: &str) -> String {
    const MAX_BYTES: usize = 16_000;
    if output.len() <= MAX_BYTES {
        return output.trim().to_string();
    }
    let mut end = MAX_BYTES;
    while !output.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\n[truncated]", output[..end].trim())
}

fn short_text(value: &str, max_bytes: usize) -> String {
    let value = value.trim();
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", value[..end].trim())
}

fn push_unique_limited(values: &mut Vec<String>, value: String, limit: usize) {
    if values.len() < limit && !values.iter().any(|item| item == &value) {
        values.push(value);
    }
}

fn parallel_field_goal_candidates() -> Vec<String> {
    default_field_pack_ids()
}

fn parallel_field_goal_pool(run_index: u64) -> Vec<String> {
    let mut fields = parallel_field_goal_candidates();
    if !fields.is_empty() {
        let rotation = run_index as usize % fields.len();
        fields.rotate_left(rotation);
    }
    fields
}

fn fields_mentioned_in_text(value: &str) -> Vec<String> {
    let text = unicode_lowercase(value);
    let mut fields = Vec::new();
    let limit = default_field_pack_ids().len().max(1);
    for (field, aliases) in default_field_pack_aliases() {
        if aliases
            .iter()
            .chain(std::iter::once(&field))
            .map(|alias| unicode_lowercase(alias))
            .any(|alias| alias.chars().count() >= 2 && text.contains(&alias))
        {
            push_unique_limited(&mut fields, field, limit);
        }
    }
    fields
}

fn generic_evolution_scope_alias(alias: &str) -> bool {
    matches!(
        alias,
        "patch" | "edit" | "repo edit" | "local code" | "task" | "mini task"
    )
}

fn text_contains_field_term(text: &str, term: &str) -> bool {
    if term.chars().any(|ch| !ch.is_ascii()) {
        return text.contains(term);
    }
    text.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-'))
        .any(|part| part == term)
}

fn unicode_lowercase(value: &str) -> String {
    value.chars().flat_map(char::to_lowercase).collect()
}

fn trace_field(trace: &FeedTraceRecord) -> Option<String> {
    trace
        .field
        .clone()
        .or_else(|| trace.metadata.get("field_pack").cloned())
        .or_else(|| trace.metadata.get("field").cloned())
        .filter(|field| !field.trim().is_empty())
}

fn unix_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn exit_code(status: &std::process::ExitStatus) -> i32 {
    status.code().unwrap_or(-1)
}

fn tool_status(results: &[ToolResult]) -> Status {
    if results.is_empty() {
        Status::Failed
    } else if results
        .iter()
        .all(|result| result.status == Status::Satisfied)
    {
        Status::Satisfied
    } else if results
        .iter()
        .any(|result| matches!(result.status, Status::Satisfied | Status::Partial))
    {
        Status::Partial
    } else if results
        .iter()
        .all(|result| result.status == Status::Unsupported)
    {
        Status::Unsupported
    } else {
        Status::Failed
    }
}

fn command_available(command: &str) -> bool {
    Command::new(command).arg("--version").output().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_from_llm_content_extracts_fenced_json() {
        let plan = plan_from_llm_content(
            "```json\n{\"calls\":[{\"tool\":\"verify\",\"reason\":\"check\",\"payload\":{}}],\"summary\":\"ok\"}\n```",
        )
        .unwrap();

        assert_eq!(plan.calls[0].tool, "verify");
        assert_eq!(plan.summary, "ok");
    }

    #[test]
    fn tool_result_json_accepts_artifact_only_evidence() {
        let result = tool_result_from_json(
            "run_field_mini_task",
            r#"{
              "status": "satisfied",
              "output": "ok",
              "evidence": [
                {
                  "source": "field-mini-task/math/checks",
                  "artifact_path": ".octopus/field-mini-task/math/CHECKS.json",
                  "confidence": 0.95,
                  "metadata": {
                    "field_pass_evidence": "artifact-backed verifier evidence"
                  }
                }
              ],
              "metadata": {
                "verifier_status": "satisfied",
                "field_pass_evidence": "artifact-backed verifier evidence"
              }
            }"#,
        )
        .unwrap();

        assert_eq!(result.status, Status::Satisfied);
        assert_eq!(
            result
                .metadata
                .get("field_pass_evidence")
                .map(String::as_str),
            Some("artifact-backed verifier evidence")
        );
        assert_eq!(result.evidence[0].content, "");
    }

    #[test]
    fn manifest_tentacle_retries_invalid_llm_plan_once() {
        struct RetryChat {
            calls: usize,
        }

        impl ChatClient for RetryChat {
            fn chat(&mut self, _messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                self.calls += 1;
                let content = if self.calls == 1 {
                    "no valid plan".to_string()
                } else {
                    r#"{"calls":[{"tool":"artifact","reason":"retry selected tool","payload":{}}],"summary":"retry ok"}"#.to_string()
                };
                Ok(ChatResponse {
                    content,
                    metadata: BTreeMap::new(),
                })
            }
        }

        let mut state = HarnessState::default();
        state.installed_tentacles.push(InstalledTentacle {
            id: "retry-tentacle".to_string(),
            name: "Retry Tentacle".to_string(),
            source: "/tmp/retry-tentacle/manifest.json".to_string(),
            brain_kind: "llm".to_string(),
            brain_prompt: "Select a tool for the Need.".to_string(),
            feedback_contract: None,
            runtime_kinds: vec!["static-html".to_string()],
            needs: vec!["verify".to_string()],
            tools: Vec::new(),
            tool_meta: vec![InstalledTool {
                id: "artifact".to_string(),
                description: "Static artifact".to_string(),
                input: "none".to_string(),
                output: "artifact path".to_string(),
                kind: "static-html".to_string(),
                entrypoint: "/tmp/octopus-retry-artifact.html".to_string(),
                contract: None,
                permission: None,
            }],
            editable: Vec::new(),
            evolution_surfaces: Vec::new(),
        });
        let factory: ChatClientFactory = Arc::new(|| Ok(Box::new(RetryChat { calls: 0 })));
        let mut harness = Harness::with_state_and_manifest_llm_factory(state, factory);

        let feed = harness.feed_one(&Need::new(NeedKind::Verify, "claim"));

        assert_eq!(feed.status, Status::Satisfied, "{}", feed.summary);
        assert_eq!(
            feed.metadata.get("plan_source"),
            Some(&"llm-retry".to_string())
        );
        assert!(feed
            .metadata
            .get("plan")
            .is_some_and(|plan| plan.contains("retry selected tool")));
    }

    #[test]
    fn tool_status_preserves_single_partial_result() {
        let result = ToolResult {
            status: Status::Partial,
            output: "needs evolved field execution".to_string(),
            evidence: Vec::new(),
            metadata: BTreeMap::new(),
        };

        assert_eq!(tool_status(&[result]), Status::Partial);
    }

    #[test]
    fn memory_is_fed_without_brain_state() {
        let mut harness = Harness::new();
        harness.feed_one(&Need::new(
            NeedKind::Remember,
            "tools stay outside the brain",
        ));

        let feedback = harness.feed(&[Need::new(NeedKind::Recall, "tools")]);

        assert_eq!(feedback.status, Status::Satisfied, "{}", feedback.summary);
        assert!(feedback.summary.contains("tools stay outside the brain"));
        assert_eq!(
            harness.state.last_pet_event.as_ref().unwrap().state,
            "memory"
        );
    }

    #[test]
    fn memory_recall_ignores_unmatched_weighted_records() {
        let mut memory = MemoryStore::default();
        memory.remember("alpha route");
        memory.remember("beta tentacle");

        let recalled = memory.recall("tentacle", 5);

        assert_eq!(recalled.len(), 1);
        assert_eq!(recalled[0].text, "beta tentacle");
    }

    #[test]
    fn memory_compaction_keeps_higher_weight_records() {
        let mut memory = MemoryStore::default();
        memory.remember("alpha");
        memory.remember("beta");
        memory.remember("gamma");
        memory.recall("gamma", 1);

        let dropped = memory.compact(2);

        assert_eq!(dropped, 1);
        assert_eq!(memory.len(), 2);
        assert_eq!(memory.recall("alpha", 1), Vec::<MemoryRecord>::new());
        assert_eq!(memory.recall("gamma", 1)[0].text, "gamma");
    }

    #[test]
    fn router_learns_successful_tentacle() {
        let mut harness = Harness::new();
        harness.add_tentacle(Box::new(FunctionTentacle::new(
            "verifier",
            vec![NeedKind::Verify],
            |need| Feed::satisfied(need, "verified", "verifier"),
        )));

        harness.feed_one(&Need::new(NeedKind::Verify, "claim"));

        assert!(harness.state.routes.score(&NeedKind::Verify, "verifier") > 1.0);
    }

    #[test]
    fn route_report_explains_feedback_scores() {
        let mut harness = Harness::new();
        harness.add_tentacle(Box::new(FunctionTentacle::new(
            "verifier",
            vec![NeedKind::Verify],
            |need| Feed::satisfied(need, "verified", "verifier"),
        )));
        harness.add_tentacle(Box::new(FunctionTentacle::new(
            "backup",
            vec![NeedKind::Verify],
            |need| Feed::satisfied(need, "verified", "backup"),
        )));
        harness
            .state
            .routes
            .scores
            .insert(route_key(&NeedKind::Verify, "verifier"), 2.0);
        harness
            .state
            .routes
            .scores
            .insert(route_key(&NeedKind::Verify, "backup"), 0.5);
        harness.state.record_feed_trace_from_feed(&Feed {
            need: Need::new(NeedKind::Verify, "claim"),
            status: Status::Partial,
            evidence: vec![Evidence::new("review", "missing citation")],
            summary: "needs better evidence".to_string(),
            metadata: BTreeMap::from([("tentacle".to_string(), "verifier".to_string())]),
        });

        let report = harness.route_report(&Need::new(NeedKind::Verify, "claim"));

        assert_eq!(report.selected.as_ref().unwrap().tentacle, "verifier");
        assert_eq!(report.candidates[0].tentacle, "verifier");
        assert!(report.candidates[0].selected);
        assert_eq!(report.candidates[0].trace_count, 1);
        assert_eq!(
            report.candidates[0].last_trace.as_ref().unwrap().status,
            Status::Partial
        );
        assert_eq!(
            report
                .learned_scores
                .get(&route_key(&NeedKind::Verify, "verifier")),
            Some(&2.0)
        );
        assert!(report
            .next
            .contains(&"octopus feedback latest satisfied|partial|failed".to_string()));
    }

    #[test]
    fn context_report_keeps_brain_slots_clean() {
        let mut state = HarnessState {
            goal: Some(Goal::new("ship Octopus")),
            ..HarnessState::default()
        };
        state.memory.remember("clean brain keeps tools outside");
        let need = Need::new(NeedKind::Observe, "README.md");
        let mut feed = Feed::satisfied(&need, "observed README", "swe-agent");
        feed.metadata
            .insert("tentacle".to_string(), "swe-agent".to_string());
        feed.metadata.insert("tool".to_string(), "read".to_string());
        state.record_feed_trace_from_feed(&feed);

        let report = state.context_report(Some(Need::new(NeedKind::Verify, "claim")), 5);

        assert_eq!(report.brain.policy, CLEAN_BRAIN_CONTEXT_POLICY);
        assert_eq!(report.brain.slots, vec!["Goal", "Mem", "Need", "Feed"]);
        assert!(!report.brain.slots.iter().any(|slot| slot == "Tool"));
        assert_eq!(report.brain.mem[0].text, "clean brain keeps tools outside");
        assert_eq!(report.brain.turns[0].need.kind, NeedKind::Observe);
        assert_eq!(report.brain.turns[0].feed.summary, "observed README");
    }

    #[test]
    fn context_report_exposes_tentacle_tool_context() {
        let mut state = HarnessState::default();
        let permission = ToolPermission {
            provider: "octopus".to_string(),
            scope: "tool:swe-agent".to_string(),
            permissions: vec!["tool:observe".to_string()],
            reason: Some("repo observation".to_string()),
        };
        state.installed_tentacles.push(InstalledTentacle {
            id: "swe-agent".to_string(),
            name: "SWE Agent".to_string(),
            source: "test".to_string(),
            brain_kind: "llm".to_string(),
            brain_prompt: "turn needs into tool actions".to_string(),
            feedback_contract: None,
            runtime_kinds: vec!["shell".to_string()],
            needs: vec!["observe".to_string()],
            tools: Vec::new(),
            tool_meta: vec![InstalledTool {
                id: "read".to_string(),
                description: "Read files".to_string(),
                input: "path".to_string(),
                output: "lines".to_string(),
                kind: "shell".to_string(),
                entrypoint: "tools/read.sh".to_string(),
                contract: None,
                permission: Some(permission),
            }],
            editable: vec!["tools/*".to_string()],
            evolution_surfaces: vec!["runtime_code".to_string()],
        });
        let grant = state.grant_oauth(
            "octopus",
            "tool:swe-agent",
            vec!["tool:observe".to_string()],
        );

        let report = state.context_report(None, 5);

        assert_eq!(report.tentacles[0].policy, TENTACLE_CONTEXT_POLICY);
        assert_eq!(
            report.tentacles[0].slots,
            vec!["Need", "Tool", "Action", "Feed"]
        );
        assert_eq!(report.tentacles[0].tools[0].id, "read");
        assert!(report.tentacles[0].tools[0].authorization_required);
        assert_eq!(
            report.tentacles[0].tools[0].active_grant.as_deref(),
            Some(grant.id.as_str())
        );
    }

    #[test]
    fn feed_feedback_updates_trace_route_and_pet_state() {
        let mut harness = Harness::new();
        harness.add_tentacle(Box::new(FunctionTentacle::new(
            "verifier",
            vec![NeedKind::Verify],
            |need| Feed::satisfied(need, "verified", "verifier"),
        )));
        let feed = harness.feed_one(&Need::new(NeedKind::Verify, "claim"));
        let trace_index = feed
            .metadata
            .get("feed_trace_index")
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let route_before = harness.state.routes.score(&NeedKind::Verify, "verifier");

        let outcome = harness
            .state
            .record_feed_feedback(trace_index, Status::Failed, "missing evidence")
            .unwrap();

        assert_eq!(outcome.trace.status, Status::Failed);
        assert_eq!(outcome.trace.summary, "feedback: missing evidence");
        assert!(harness.state.routes.score(&NeedKind::Verify, "verifier") < route_before);
        assert_eq!(
            harness.state.last_pet_event.as_ref().unwrap().source,
            "feed feedback"
        );
        assert_eq!(
            harness.state.last_pet_event.as_ref().unwrap().state,
            "blocked"
        );
    }

    #[test]
    fn repair_outcome_feeds_harness_memory() {
        let mut state = HarnessState::default();
        let trace = state.record_feed_trace_from_feed(&Feed {
            need: Need::new(NeedKind::Verify, "harness repair loop"),
            status: Status::Partial,
            evidence: vec![Evidence::new("repair", "needs score")],
            summary: "repair session created".to_string(),
            metadata: BTreeMap::from([
                ("tentacle".to_string(), "harness-repair-agent".to_string()),
                ("tool".to_string(), "repair_session".to_string()),
                ("target_tentacle".to_string(), "swe-agent".to_string()),
                ("candidate".to_string(), "03-runtime-code".to_string()),
            ]),
        });
        let route_before = state
            .routes
            .score(&NeedKind::Verify, "harness-repair-agent");

        let outcome = state
            .record_repair_outcome(
                Some(trace.index),
                Status::Satisfied,
                "repair improved harness",
            )
            .unwrap();

        assert_eq!(outcome.index, 1);
        assert_eq!(outcome.trace_index, Some(trace.index));
        assert_eq!(outcome.tool.as_deref(), Some("repair_session"));
        assert_eq!(outcome.target_tentacle.as_deref(), Some("swe-agent"));
        assert_eq!(outcome.candidate.as_deref(), Some("03-runtime-code"));
        assert!(
            state
                .routes
                .score(&NeedKind::Verify, "harness-repair-agent")
                > route_before
        );
        assert_eq!(state.recent_repair_outcomes(1)[0].summary, outcome.summary);
        assert_eq!(
            state.last_pet_event.as_ref().unwrap().source,
            "repair score"
        );
        assert_eq!(state.last_pet_event.as_ref().unwrap().state, "harness");

        state
            .record_repair_outcome(None, Status::Failed, "still broken")
            .unwrap();
        assert_eq!(state.compact_repair_outcomes(1), 1);
        assert_eq!(state.repair_outcomes[0].summary, "still broken");
    }

    #[test]
    fn router_does_not_penalize_missing_tool_grant() {
        let mut harness = Harness::new();
        harness.add_tentacle(Box::new(FunctionTentacle::new(
            "guarded",
            vec![NeedKind::Execute],
            |need| Feed {
                need: need.clone(),
                status: Status::Failed,
                evidence: Vec::new(),
                summary: "needs_authorization".to_string(),
                metadata: BTreeMap::from([(
                    "authorization_required".to_string(),
                    "true".to_string(),
                )]),
            },
        )));

        harness.feed_one(&Need::new(NeedKind::Execute, "guarded action"));

        assert_eq!(
            harness.state.routes.score(&NeedKind::Execute, "guarded"),
            1.0
        );
    }

    #[test]
    fn evolution_outcomes_feed_harness_scores() {
        let mut state = HarnessState::default();

        let outcome = state.record_evolution_outcome(
            "swe-agent",
            "03-runtime-code",
            Status::Satisfied,
            "patch improved feed evidence",
        );

        assert_eq!(outcome.index, 1);
        assert_eq!(outcome.score, 1.0);
        assert_eq!(
            state.recent_evolution_outcomes("swe-agent", 1)[0].candidate_id,
            "03-runtime-code"
        );
        assert!(
            state
                .routes
                .score(&NeedKind::Execute, "evolve:swe-agent:03-runtime-code")
                > 1.0
        );
        let report = state.status_report();
        assert_eq!(report.harness_learning.source, "evolution_outcome");
        assert_eq!(
            report.harness_learning.target_tentacle.as_deref(),
            Some("swe-agent")
        );
        assert_eq!(
            report.harness_learning.candidate.as_deref(),
            Some("03-runtime-code")
        );
    }

    #[test]
    fn harness_state_beat_runs_three_hearts() {
        let mut harness = Harness::new();
        harness.feed_one(&Need::new(NeedKind::Remember, "older memory"));
        harness.feed_one(&Need::new(NeedKind::Remember, "newer memory"));
        harness.add_tentacle(Box::new(FunctionTentacle::new(
            "verifier",
            vec![NeedKind::Verify],
            |need| Feed::satisfied(need, "verified", "verifier"),
        )));
        harness.feed_one(&Need::new(NeedKind::Verify, "claim"));
        let route_before = harness.state.routes.score(&NeedKind::Verify, "verifier");

        let report = harness.state.beat(1);

        assert_eq!(
            report
                .beats
                .iter()
                .map(|beat| beat.name.as_str())
                .collect::<Vec<_>>(),
            vec!["heartbeat", "memory", "harness"]
        );
        assert_eq!(harness.state.memory.len(), 1);
        assert!(harness.state.routes.score(&NeedKind::Verify, "verifier") < route_before);
        assert_eq!(
            harness.state.last_pet_event.as_ref().unwrap().state,
            "memory"
        );
        assert!(
            report
                .beats
                .iter()
                .find(|beat| beat.name == "memory")
                .unwrap()
                .changed
        );
    }

    #[test]
    fn harness_records_and_compacts_check_history() {
        let mut state = HarnessState::default();
        state.record_check_history(CheckHistoryInput {
            tentacle_id: "swe-agent".to_string(),
            source_kind: "manifest".to_string(),
            command_index: Some(1),
            command: "python -m pytest".to_string(),
            cwd: "/repo".to_string(),
            status: Status::Failed,
            code: Some(1),
            stdout: "stdout".to_string(),
            stderr: "stderr".to_string(),
        });
        state.record_check_history(CheckHistoryInput {
            tentacle_id: "computer-use-agent".to_string(),
            source_kind: "manifest".to_string(),
            command_index: Some(1),
            command: "window_status".to_string(),
            cwd: "/repo/tentacles/computer-use-agent".to_string(),
            status: Status::Satisfied,
            code: Some(0),
            stdout: "{}".to_string(),
            stderr: String::new(),
        });

        let swe_history = state.recent_check_history_for_tentacle("swe-agent", 5);
        assert_eq!(swe_history.len(), 1);
        assert_eq!(swe_history[0].command_index, Some(1));
        assert_eq!(swe_history[0].stderr, "stderr");
        let status = state.status_report();
        assert_eq!(status.check_history_count, 2);
        assert_eq!(
            status.latest_check.as_ref().unwrap().tentacle_id,
            "computer-use-agent"
        );

        let dropped = state.compact_check_history(1);

        assert_eq!(dropped, 1);
        assert_eq!(state.check_history.len(), 1);
        assert_eq!(state.check_history[0].tentacle_id, "computer-use-agent");
    }

    #[test]
    fn feed_and_evolution_update_pet_event() {
        let mut harness = Harness::new();

        harness.feed_one(&Need::new(NeedKind::Verify, "claim"));
        let event = harness.state.last_pet_event.as_ref().unwrap();
        assert_eq!(event.state, "blocked");
        assert_eq!(event.status, Status::Unsupported);

        harness.add_tentacle(Box::new(FunctionTentacle::new(
            "verifier",
            vec![NeedKind::Verify],
            |need| Feed::satisfied(need, "verified", "verifier"),
        )));

        harness.feed_one(&Need::new(NeedKind::Verify, "claim"));

        let event = harness.state.last_pet_event.as_ref().unwrap();
        assert_eq!(event.state, "success");
        assert_eq!(event.source, "verifier");

        harness.state.record_evolution_outcome(
            "swe-agent",
            "03-runtime-code",
            Status::Satisfied,
            "patch improved feed",
        );

        let event = harness.state.last_pet_event.as_ref().unwrap();
        assert_eq!(event.state, "harness");
        assert_eq!(event.source, "evolve score");
    }

    #[test]
    fn harness_state_status_report_is_read_only_doctor() {
        let mut state = HarnessState::default();
        let empty = state.status_report();

        assert_eq!(empty.memory_count, 0);
        assert_eq!(empty.route_count, 0);
        assert_eq!(
            empty
                .hearts
                .iter()
                .map(|beat| beat.name.as_str())
                .collect::<Vec<_>>(),
            vec!["heartbeat", "memory", "harness"]
        );
        assert_eq!(empty.agent_next_action, "octopus adapt".to_string());
        assert_eq!(empty.next_action, empty.user_goal_hint);
        assert!(empty.user_goal_hint.contains("Set a Goal"));
        assert_eq!(empty.harness_learning.source, "none");
        assert!(empty
            .warnings
            .contains(&"no installed tentacle manifests".to_string()));

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        state.install_manifest(&root, "swe-agent").unwrap();
        state.grant_oauth(
            "github",
            "dangoZhang/Octopus",
            default_permissions("github"),
        );
        let mut harness = Harness::with_state(state);
        harness.chat_with_refinement(
            "build a clean-brain agent",
            GoalRefinement {
                objective: Some("build a clean-brain agent".to_string()),
                constraints: Vec::new(),
                summary: Some("goal refined by external model".to_string()),
                needs: Vec::new(),
            },
        );
        harness.feed_one(&Need::new(NeedKind::Observe, "."));
        let report = harness.state.status_report();

        assert_eq!(report.tentacles[0].id, "swe-agent");
        assert_eq!(report.tentacles[0].brain_kind, "llm");
        assert!(report.tentacles[0]
            .runtime_kinds
            .contains(&"shell".to_string()));
        assert_eq!(report.tentacles[0].tool_count, 5);
        assert_eq!(
            report.goal.as_ref().unwrap().objective,
            "build a clean-brain agent"
        );
        assert_eq!(report.goal.as_ref().unwrap().turns.len(), 1);
        assert_eq!(
            report.goal.as_ref().unwrap().turns[0].summary,
            "goal refined by external model"
        );
        assert_eq!(
            report.active_grants,
            vec!["github:dangoZhang/Octopus".to_string()]
        );
        assert_eq!(report.agent_next_action, "octopus beat 200".to_string());
        assert_eq!(report.next_action, report.user_goal_hint);
        assert!(!report.next_action.contains("octopus"));
        let trace = harness.state.record_feed_trace_from_feed(&Feed {
            need: Need::new(NeedKind::Verify, "repair swe-agent feed"),
            status: Status::Partial,
            evidence: vec![Evidence::new("repair", "targeted repair")],
            summary: "repair session".to_string(),
            metadata: BTreeMap::from([
                ("tentacle".to_string(), "harness-repair-agent".to_string()),
                ("tool".to_string(), "repair_session".to_string()),
                ("target_tentacle".to_string(), "swe-agent".to_string()),
                ("candidate".to_string(), "03-runtime-code".to_string()),
            ]),
        });
        harness
            .state
            .record_repair_outcome(
                Some(trace.index),
                Status::Partial,
                "line evidence still weak",
            )
            .unwrap();
        let report = harness.state.status_report();
        assert_eq!(report.evolution_outcome_count, 0);
        assert_eq!(report.harness_learning.source, "repair_outcome");
        assert_eq!(
            report.harness_learning.target_tentacle.as_deref(),
            Some("swe-agent")
        );
        assert_eq!(
            report.harness_learning.candidate.as_deref(),
            Some("03-runtime-code")
        );
        assert_eq!(report.harness_learning.status, Some(Status::Partial));
        assert_eq!(
            report.harness_learning.next_action,
            "octopus evolve recommend swe-agent"
        );
        assert_eq!(
            report.agent_next_action,
            "octopus evolve recommend swe-agent"
        );
        assert_eq!(report.next_action, report.user_goal_hint);
        assert!(!report.next_action.contains("evolve recommend"));
        let state_path = Path::new("/tmp/octopus state.json");
        let with_state = harness.state.status_report_with_state(Some(state_path));
        assert_eq!(
            with_state.agent_next_action,
            "octopus --state '/tmp/octopus state.json' evolve recommend swe-agent".to_string()
        );
        assert_eq!(
            with_state.harness_learning.next_action,
            "octopus --state '/tmp/octopus state.json' evolve recommend swe-agent"
        );
    }

    #[test]
    fn context_report_separates_user_hints_from_agent_commands() {
        let state = HarnessState::default();
        let report = state.context_report(Some(Need::new(NeedKind::Verify, "check Feed")), 3);

        assert!(report.next.iter().all(|item| !item.starts_with("octopus ")));
        assert_eq!(report.next, report.user_goal_hints);
        assert!(report
            .agent_next
            .iter()
            .any(|item| item.starts_with("octopus need verify")));
        assert!(report
            .agent_next
            .iter()
            .any(|item| item == "octopus traces"));
    }

    #[test]
    fn need_queue_report_separates_user_hints_from_agent_commands() {
        let mut state = HarnessState::default();
        state.queue_need_suggestion(
            GoalNeedSuggestion {
                kind: NeedKind::Verify,
                query: "check the desktop pet state freshness".to_string(),
            },
            "test",
            "Need queued by clean brain",
            "Goal asks for desktop state diagnosis",
        );

        let report = state.need_queue_report(8);

        assert_eq!(report.next, report.user_goal_hints);
        assert!(report.next.iter().all(|item| !item.starts_with("octopus ")));
        assert!(report
            .agent_next
            .iter()
            .any(|item| item == "octopus needs run 1"));
        assert!(report
            .agent_next
            .iter()
            .any(|item| item.starts_with("octopus need verify")));
    }

    #[test]
    fn status_ignores_ad_hoc_field_verifier_when_parallel_goal() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState {
            goal: Some(Goal::new(
                "Adapt Octopus tentacles across peer field objectives toward v0.2.0",
            )),
            ..HarnessState::default()
        };
        state.install_manifest(&root, "field-mini-task").unwrap();
        state
            .routes
            .scores
            .insert(route_key(&NeedKind::Verify, "field-mini-task"), 1.0);
        let trace = state.record_feed_trace_from_feed(&Feed {
            need: Need::new(NeedKind::Execute, "evolve recommend field-mini-task"),
            status: Status::Partial,
            evidence: vec![Evidence::new("field-mini-task", "old ad-hoc field trace")],
            summary: "old ad-hoc field trace".to_string(),
            metadata: BTreeMap::from([
                ("tentacle".to_string(), "field-mini-task".to_string()),
                ("tool".to_string(), "run_field_mini_task".to_string()),
                ("field_pack".to_string(), "computer-use".to_string()),
                ("field_mini_task".to_string(), "ad-hoc".to_string()),
            ]),
        });
        state
            .record_field_verifier_result(
                trace.index,
                Status::Partial,
                Some("field_mini_task_incomplete".to_string()),
                None,
                "old ad-hoc verifier gap",
            )
            .unwrap();

        let report = state.status_report();

        assert!(report
            .agent_next_action
            .contains("evolve parallel --workers 1"));
        assert!(!report.agent_next_action.contains("computer-use"));
        assert!(!report.next_action.contains("evolve parallel"));
    }

    #[test]
    fn field_trajectory_report_points_failed_field_to_harness_repair() {
        let mut state = HarnessState::default();
        let trace = state.record_feed_trace_from_feed(&Feed {
            need: Need::new(NeedKind::Verify, "Run math mini task math-mini-1"),
            status: Status::Partial,
            evidence: vec![Evidence::new("field-mini-task", "missing math runtime")],
            summary: "needs evolved math runtime".to_string(),
            metadata: BTreeMap::from([
                ("tentacle".to_string(), "field-mini-task".to_string()),
                ("tool".to_string(), "run_field_mini_task".to_string()),
                ("field_pack".to_string(), "math".to_string()),
                ("field_mini_task".to_string(), "math-mini-1".to_string()),
            ]),
        });
        state
            .record_field_verifier_result(
                trace.index,
                Status::Partial,
                Some("field_mini_task_incomplete".to_string()),
                None,
                "math mini task still needs explicit pass evidence",
            )
            .unwrap();

        let report = state.field_trajectory_report().unwrap();
        let math = report
            .fields
            .iter()
            .find(|field| field.field == "math")
            .unwrap();

        assert!(math.needs_repair);
        assert!(!math.ready_for_harder_task);
        assert_eq!(math.latest_mini_task.as_deref(), Some("math-mini-1"));
        assert!(math.next_action.contains("evolve recommend"));
        assert!(math.next_action.contains("improve math harness"));
        assert_eq!(report.active_slot_field.as_deref(), Some("math"));
    }

    #[test]
    fn field_trajectory_report_moves_parallel_pool_to_harder_tasks_after_first_pass() {
        let mut state = HarnessState::default();
        for field in parallel_field_goal_candidates() {
            let mini_task = format!("{field}-mini-1");
            let trace = state.record_feed_trace_from_feed(&Feed {
                need: Need::new(
                    NeedKind::Verify,
                    format!("Run {field} mini task {mini_task}"),
                ),
                status: Status::Satisfied,
                evidence: vec![Evidence::new("field-mini-task", "pass evidence")],
                summary: format!("{field} satisfied"),
                metadata: BTreeMap::from([
                    ("tentacle".to_string(), "field-mini-task".to_string()),
                    ("tool".to_string(), "run_field_mini_task".to_string()),
                    ("field_pack".to_string(), field.to_string()),
                    ("field_mini_task".to_string(), mini_task),
                    (
                        "field_pass_evidence".to_string(),
                        "explicit verifier evidence".to_string(),
                    ),
                ]),
            });
            state
                .record_field_verifier_result(
                    trace.index,
                    Status::Satisfied,
                    None,
                    None,
                    format!("{field} verifier passed"),
                )
                .unwrap();
        }

        let report = state.field_trajectory_report().unwrap();

        assert_eq!(report.field_count, default_field_pack_ids().len());
        assert!(report.all_first_pass_satisfied);
        assert!(!report.all_pack_tasks_satisfied);
        assert_eq!(report.active_slot_field.as_deref(), Some("math"));
        let math = report
            .fields
            .iter()
            .find(|field| field.field == "math")
            .unwrap();
        let catalog = default_field_pack_catalog().unwrap();
        let math_pack = catalog.packs.iter().find(|pack| pack.id == "math").unwrap();
        assert_eq!(math.mini_task_count, math_pack.mini_tasks.len());
        assert_eq!(math.satisfied_mini_task_count, 1);
        assert_eq!(math.next_mini_task.as_deref(), Some("math-mini-2"));
        assert!(report
            .fields
            .iter()
            .all(|field| field.ready_for_harder_task && !field.needs_repair));
        assert!(report
            .next
            .iter()
            .any(|item| item.contains("harder-task worker slot")));

        state.next_parallel_evolution_run_index = 35;
        let rotated_report = state.field_trajectory_report().unwrap();
        let active = rotated_report
            .active_slot_field
            .as_deref()
            .expect("active field");
        let active_summary = rotated_report
            .fields
            .iter()
            .find(|field| field.field == active)
            .expect("active field summary");
        assert!(active_summary.next_mini_task.is_some());
    }

    #[test]
    fn field_trajectory_report_clears_next_task_when_parallel_pool_layer_is_done() {
        let mut state = HarnessState::default();
        let catalog = default_field_pack_catalog().unwrap();
        for pack in &catalog.packs {
            for task in &pack.mini_tasks {
                let trace = state.record_feed_trace_from_feed(&Feed {
                    need: Need::new(
                        NeedKind::Verify,
                        format!("Run {} mini task {}", pack.id, task.id),
                    ),
                    status: Status::Satisfied,
                    evidence: vec![Evidence::new("field-mini-task", "pass evidence")],
                    summary: format!("{} satisfied", task.id),
                    metadata: BTreeMap::from([
                        ("tentacle".to_string(), "field-mini-task".to_string()),
                        ("tool".to_string(), "run_field_mini_task".to_string()),
                        ("field_pack".to_string(), pack.id.clone()),
                        ("field_mini_task".to_string(), task.id.clone()),
                    ]),
                });
                state
                    .record_field_verifier_result(
                        trace.index,
                        Status::Satisfied,
                        None,
                        None,
                        format!("{} verifier passed", task.id),
                    )
                    .unwrap();
            }
        }

        let report = state.field_trajectory_report().unwrap();

        assert!(report.all_first_pass_satisfied);
        assert!(report.all_pack_tasks_satisfied);
        assert_eq!(report.active_slot_field, None);
        assert!(report
            .fields
            .iter()
            .all(|field| field.next_mini_task.is_none()));
        assert!(report
            .next
            .iter()
            .any(|item| item.contains("evolve recommend field-mini-task")
                && item.contains("harder mini task layer")));
        assert!(!report
            .next
            .iter()
            .any(|item| item == "add the next harder mini task layer to field packs"));

        let stateful_report = state
            .field_trajectory_report_with_state(Some(Path::new("/tmp/octopus fields/state.json")))
            .unwrap();
        assert!(stateful_report.next.iter().any(|item| item.contains(
            "octopus --state '/tmp/octopus fields/state.json' evolve recommend field-mini-task"
        )));

        state.goal = Some(Goal::new(
            "Adapt Octopus tentacles across peer field objectives toward v0.2.0",
        ));
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        state.install_manifest(&root, "field-mini-task").unwrap();
        state
            .routes
            .scores
            .insert(route_key(&NeedKind::Verify, "field-mini-task"), 1.0);
        let status =
            state.status_report_with_state(Some(Path::new("/tmp/octopus fields/state.json")));
        assert!(status
            .agent_next_action
            .contains("evolve recommend field-mini-task"));
        assert!(!status.agent_next_action.contains("evolve parallel"));
        assert!(!status.next_action.contains("evolve recommend"));
    }

    #[test]
    fn field_trajectory_report_partial_harder_task_overrides_layer_complete() {
        let mut state = HarnessState::default();
        let catalog = default_field_pack_catalog().unwrap();
        for pack in &catalog.packs {
            for task in &pack.mini_tasks {
                let trace = state.record_feed_trace_from_feed(&Feed {
                    need: Need::new(
                        NeedKind::Verify,
                        format!("Run {} mini task {}", pack.id, task.id),
                    ),
                    status: Status::Satisfied,
                    evidence: vec![Evidence::new("field-mini-task", "pass evidence")],
                    summary: format!("{} satisfied", task.id),
                    metadata: BTreeMap::from([
                        ("tentacle".to_string(), "field-mini-task".to_string()),
                        ("tool".to_string(), "run_field_mini_task".to_string()),
                        ("field_pack".to_string(), pack.id.clone()),
                        ("field_mini_task".to_string(), task.id.clone()),
                    ]),
                });
                state
                    .record_field_verifier_result(
                        trace.index,
                        Status::Satisfied,
                        None,
                        None,
                        format!("{} verifier passed", task.id),
                    )
                    .unwrap();
            }
        }
        let trace = state.record_feed_trace_from_feed(&Feed {
            need: Need::new(NeedKind::Verify, "Run search mini task search-mini-4"),
            status: Status::Partial,
            evidence: vec![Evidence::new("field-mini-task", "missing search evidence")],
            summary: "search mini-4 partial".to_string(),
            metadata: BTreeMap::from([
                ("tentacle".to_string(), "field-mini-task".to_string()),
                ("tool".to_string(), "run_field_mini_task".to_string()),
                ("field_pack".to_string(), "search".to_string()),
                ("field_mini_task".to_string(), "search-mini-4".to_string()),
            ]),
        });
        state
            .record_field_verifier_result(
                trace.index,
                Status::Partial,
                Some("field_mini_task_incomplete".to_string()),
                None,
                "search mini-4 still needs explicit pass evidence",
            )
            .unwrap();

        let report = state.field_trajectory_report().unwrap();

        assert!(!report.all_pack_tasks_satisfied);
        assert_eq!(report.active_slot_field.as_deref(), Some("search"));
        assert!(report.active_slot_reason.contains("search"));
        let search = report
            .fields
            .iter()
            .find(|field| field.field == "search")
            .unwrap();
        assert!(search.needs_repair);
        assert_eq!(search.latest_mini_task.as_deref(), Some("search-mini-4"));
        assert!(report
            .next
            .iter()
            .any(|item| item.contains("improve search harness")));
        let pool = state.field_pool_status_report(None).unwrap();
        assert_eq!(pool.completed_fields, default_field_pack_ids().len() - 1);
        assert_eq!(pool.next, pool.user_goal_hints);
        assert!(pool.next.iter().all(|item| !item.starts_with("octopus ")));
        assert!(pool
            .agent_next
            .iter()
            .any(|item| item.contains("improve search harness")));
        let search_slot = pool
            .slots
            .iter()
            .find(|slot| slot.field == "search")
            .unwrap();
        assert!(!search_slot.completed);
        assert!(search_slot.needs_repair);
        assert!(search_slot
            .agent_next_action
            .contains("improve search harness"));
        assert!(!search_slot.next_action.contains("octopus"));
    }

    #[test]
    fn field_pool_status_prefers_verifier_over_stale_worker_status() {
        let mut state = HarnessState::default();
        let trace = state.record_feed_trace_from_feed(&Feed {
            need: Need::new(NeedKind::Verify, "Run math mini task math-mini-1"),
            status: Status::Satisfied,
            evidence: vec![Evidence::new("field-mini-task", "verified math")],
            summary: "math-mini-1 satisfied".to_string(),
            metadata: BTreeMap::from([
                ("tentacle".to_string(), "field-mini-task".to_string()),
                ("tool".to_string(), "run_field_mini_task".to_string()),
                ("field_pack".to_string(), "math".to_string()),
                ("field_mini_task".to_string(), "math-mini-1".to_string()),
            ]),
        });
        let verifier = state
            .record_field_verifier_result(
                trace.index,
                Status::Satisfied,
                None,
                None,
                "math-mini-1 explicit pass evidence",
            )
            .unwrap();
        state.parallel_evolution_runs.push(ParallelEvolutionRun {
            index: 1,
            objective: "math field evolution".to_string(),
            field_pool_size: default_parallel_field_pool_size(),
            field_pool_policy: default_parallel_field_pool_policy(),
            candidate_fields: vec!["math".to_string()],
            requested_worker_count: 1,
            worker_count: 1,
            worker_policy: default_parallel_worker_policy(),
            workers: vec![ParallelEvolutionWorker {
                id: "octopus-test-1".to_string(),
                field: "math".to_string(),
                mini_task: Some("math-mini-1".to_string()),
                goal: "old worker status".to_string(),
                updated_at_secs: 1,
                queued_need_index: None,
                source_trace_index: Some(trace.index),
                verifier_result_index: Some(verifier.index),
                status: Status::Partial,
                next_action: "octopus fields summary".to_string(),
            }],
            summary: "stale worker partial".to_string(),
        });

        let pool = state.field_pool_status_report(None).unwrap();
        let math = pool.slots.iter().find(|slot| slot.field == "math").unwrap();

        assert_eq!(math.latest_worker_status, Some(Status::Partial));
        assert_eq!(math.latest_status, Some(Status::Satisfied));
    }

    #[test]
    fn field_trajectory_report_samples_third_layer_after_two_layers_pass() {
        let mut state = HarnessState::default();
        let catalog = default_field_pack_catalog().unwrap();
        for pack in &catalog.packs {
            for task in pack.mini_tasks.iter().take(2) {
                let trace = state.record_feed_trace_from_feed(&Feed {
                    need: Need::new(
                        NeedKind::Verify,
                        format!("Run {} mini task {}", pack.id, task.id),
                    ),
                    status: Status::Satisfied,
                    evidence: vec![Evidence::new("field-mini-task", "pass evidence")],
                    summary: format!("{} satisfied", task.id),
                    metadata: BTreeMap::from([
                        ("tentacle".to_string(), "field-mini-task".to_string()),
                        ("tool".to_string(), "run_field_mini_task".to_string()),
                        ("field_pack".to_string(), pack.id.clone()),
                        ("field_mini_task".to_string(), task.id.clone()),
                    ]),
                });
                state
                    .record_field_verifier_result(
                        trace.index,
                        Status::Satisfied,
                        None,
                        None,
                        format!("{} verifier passed", task.id),
                    )
                    .unwrap();
            }
        }

        let report = state.field_trajectory_report().unwrap();
        let math = report
            .fields
            .iter()
            .find(|field| field.field == "math")
            .unwrap();

        assert!(report.all_first_pass_satisfied);
        assert!(!report.all_pack_tasks_satisfied);
        assert_eq!(report.active_slot_field.as_deref(), Some("math"));
        assert_eq!(math.satisfied_mini_task_count, 2);
        assert_eq!(math.next_mini_task.as_deref(), Some("math-mini-3"));
        assert!(math
            .next_mini_task_goal
            .as_deref()
            .unwrap()
            .contains("induction"));
    }

    #[test]
    fn parallel_evolution_samples_next_unsatisfied_mini_task_after_first_pass() {
        let mut state = HarnessState::default();
        for field in parallel_field_goal_candidates() {
            let mini_task = format!("{field}-mini-1");
            let trace = state.record_feed_trace_from_feed(&Feed {
                need: Need::new(
                    NeedKind::Verify,
                    format!("Run {field} mini task {mini_task}"),
                ),
                status: Status::Satisfied,
                evidence: vec![Evidence::new("field-mini-task", "pass evidence")],
                summary: format!("{field} satisfied"),
                metadata: BTreeMap::from([
                    ("tentacle".to_string(), "field-mini-task".to_string()),
                    ("tool".to_string(), "run_field_mini_task".to_string()),
                    ("field_pack".to_string(), field.to_string()),
                    ("field_mini_task".to_string(), mini_task),
                ]),
            });
            state
                .record_field_verifier_result(
                    trace.index,
                    Status::Satisfied,
                    None,
                    None,
                    format!("{field} verifier passed"),
                )
                .unwrap();
        }

        let run = state
            .start_parallel_evolution(
                "peer field objectives; open one harder-task worker slot from the peer field pool",
                1,
            )
            .unwrap();
        let queued = state.need_queue.last().unwrap();

        assert_eq!(run.worker_count, 1);
        assert_eq!(run.workers[0].field, "math");
        assert!(queued.need.query.contains("math-mini-2"));
        assert!(queued.summary.contains("mini task: math-mini-2"));
    }

    #[test]
    fn parallel_evolution_treats_multi_field_objective_as_candidate_pool() {
        let mut state = HarnessState::default();
        state.next_parallel_evolution_run_index = 3;
        for field in parallel_field_goal_candidates() {
            let mini_task = format!("{field}-mini-1");
            let trace = state.record_feed_trace_from_feed(&Feed {
                need: Need::new(
                    NeedKind::Verify,
                    format!("Run {field} mini task {mini_task}"),
                ),
                status: Status::Satisfied,
                evidence: vec![Evidence::new("field-mini-task", "pass evidence")],
                summary: format!("{field} satisfied"),
                metadata: BTreeMap::from([
                    ("tentacle".to_string(), "field-mini-task".to_string()),
                    ("tool".to_string(), "run_field_mini_task".to_string()),
                    ("field_pack".to_string(), field.to_string()),
                    ("field_mini_task".to_string(), mini_task),
                ]),
            });
            state
                .record_field_verifier_result(
                    trace.index,
                    Status::Satisfied,
                    None,
                    None,
                    format!("{field} verifier passed"),
                )
                .unwrap();
        }

        let run = state
            .start_parallel_evolution(
                "parallel fields: math search code swe research computer-use ib robotics",
                1,
            )
            .unwrap();
        let queued = state.need_queue.last().unwrap();

        assert_eq!(run.worker_count, 1);
        assert_eq!(run.workers[0].field, "swe");
        assert!(queued.need.query.contains("swe-mini-2"));
        assert!(queued.summary.contains("field: swe"));
    }

    #[test]
    fn parallel_evolution_does_not_queue_completed_single_field_without_next_task() {
        let mut state = HarnessState::default();
        let catalog = default_field_pack_catalog().unwrap();
        let math = catalog
            .packs
            .iter()
            .find(|pack| pack.id == "math")
            .expect("math field pack");
        for task in &math.mini_tasks {
            let trace = state.record_feed_trace_from_feed(&Feed {
                need: Need::new(NeedKind::Verify, format!("Run math mini task {}", task.id)),
                status: Status::Satisfied,
                evidence: vec![Evidence::new("field-mini-task", "pass evidence")],
                summary: format!("{} satisfied", task.id),
                metadata: BTreeMap::from([
                    ("tentacle".to_string(), "field-mini-task".to_string()),
                    ("tool".to_string(), "run_field_mini_task".to_string()),
                    ("field_pack".to_string(), "math".to_string()),
                    ("field_mini_task".to_string(), task.id.clone()),
                ]),
            });
            state
                .record_field_verifier_result(
                    trace.index,
                    Status::Satisfied,
                    None,
                    None,
                    format!("{} verifier passed", task.id),
                )
                .unwrap();
        }

        let run = state
            .start_parallel_evolution("math next harder mini task", 1)
            .unwrap();

        assert_eq!(run.candidate_fields, vec!["math".to_string()]);
        assert_eq!(run.worker_count, 0);
        assert!(run.workers.is_empty());
        assert_eq!(state.pending_need_queue_count(), 0);
    }

    #[test]
    fn parallel_evolution_queues_one_need_per_worker_slot() {
        let mut state = HarnessState::default();
        let run = state
            .start_parallel_evolution("peer field objectives", 3)
            .unwrap();

        assert_eq!(run.worker_count, 3);
        assert_eq!(state.pending_need_queue_count(), 3);
        assert!(run
            .workers
            .iter()
            .all(|worker| worker.queued_need_index.is_some()));
        let queued_fields = state
            .need_queue
            .iter()
            .map(|item| {
                item.summary
                    .split(';')
                    .find_map(|part| part.trim().strip_prefix("field: "))
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<_>>();
        let worker_fields = run
            .workers
            .iter()
            .map(|worker| worker.field.clone())
            .collect::<Vec<_>>();
        assert_eq!(queued_fields, worker_fields);
        assert_eq!(queued_fields, vec!["math", "search", "code"]);
    }

    #[test]
    fn parallel_evolution_worker_records_actual_feed_result() {
        let mut state = HarnessState::default();
        let run = state
            .start_parallel_evolution("peer field objectives", 1)
            .unwrap();
        let queued_need_index = run.workers[0].queued_need_index.unwrap();
        let trace = state.record_feed_trace_from_feed(&Feed {
            need: Need::new(NeedKind::Verify, "Run math mini task math-mini-1"),
            status: Status::Satisfied,
            evidence: vec![Evidence::new("field-mini-task", "pass evidence")],
            summary: "math satisfied".to_string(),
            metadata: BTreeMap::from([
                ("tentacle".to_string(), "field-mini-task".to_string()),
                ("tool".to_string(), "run_field_mini_task".to_string()),
                ("field_pack".to_string(), "math".to_string()),
                ("field_mini_task".to_string(), "math-mini-1".to_string()),
            ]),
        });
        let verifier = state
            .record_field_verifier_result(
                trace.index,
                Status::Satisfied,
                None,
                None,
                "math verifier passed".to_string(),
            )
            .unwrap();

        let updated = state
            .update_parallel_evolution_worker_result(
                run.index,
                queued_need_index,
                Some(trace.index),
                Some(verifier.index),
                Status::Satisfied,
            )
            .unwrap();

        assert_eq!(updated.workers[0].source_trace_index, Some(trace.index));
        assert_eq!(
            updated.workers[0].verifier_result_index,
            Some(verifier.index)
        );
        assert_eq!(updated.workers[0].status, Status::Satisfied);
        assert_eq!(updated.workers[0].next_action, "octopus fields summary");
    }

    #[test]
    fn parallel_evolution_worker_refreshes_failed_target_to_field_mini_task() {
        let mut state = HarnessState::default();
        let run = state
            .start_parallel_evolution("peer field objectives", 1)
            .unwrap();
        let queued_need_index = run.workers[0].queued_need_index.unwrap();
        let trace = state.record_feed_trace_from_feed(&Feed {
            need: Need::new(NeedKind::Verify, "Run math mini task math-mini-1"),
            status: Status::Partial,
            evidence: vec![Evidence::new("field-mini-task", "missing pass evidence")],
            summary: "math partial".to_string(),
            metadata: BTreeMap::from([
                ("tentacle".to_string(), "field-mini-task".to_string()),
                ("tool".to_string(), "run_field_mini_task".to_string()),
                ("field_pack".to_string(), "math".to_string()),
                ("field_mini_task".to_string(), "math-mini-1".to_string()),
            ]),
        });
        let verifier = state
            .record_field_verifier_result(
                trace.index,
                Status::Partial,
                Some("field_mini_task_incomplete".to_string()),
                None,
                "math verifier partial".to_string(),
            )
            .unwrap();

        let updated = state
            .update_parallel_evolution_worker_result(
                run.index,
                queued_need_index,
                Some(trace.index),
                Some(verifier.index),
                Status::Partial,
            )
            .unwrap();

        assert_eq!(updated.workers[0].status, Status::Partial);
        assert!(updated.workers[0]
            .next_action
            .contains("evolve recommend field-mini-task"));
        assert!(!updated.workers[0].next_action.contains("field:math"));
    }

    #[test]
    fn harness_state_roundtrips_as_json() {
        let mut harness = Harness::new();
        harness.feed_one(&Need::new(NeedKind::Remember, "persistent memory"));
        let json = serde_json::to_string(&harness.state).unwrap();
        let restored = serde_json::from_str::<HarnessState>(&json).unwrap();

        assert_eq!(restored.memory.records.len(), 1);
        assert!(restored.routes.score(&NeedKind::Remember, "memory") > 1.0);
    }

    #[test]
    fn chat_can_refine_goal_with_llm_without_tools() {
        struct FakeChat;

        impl ChatClient for FakeChat {
            fn chat(&mut self, _messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                Ok(ChatResponse {
                    content: r#"{"objective":"build Octopus","constraints":["keep tools outside the brain"],"summary":"goal refined by llm","needs":[{"kind":"observe","query":"inspect README"}]}"#.to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let mut harness = Harness::new();
        let mut client = FakeChat;

        let chat = harness
            .chat_with_client("make a clean-brain agent", &mut client)
            .unwrap();

        assert_eq!(chat.goal.objective, "build Octopus");
        assert_eq!(
            chat.goal.constraints,
            vec!["keep tools outside the brain".to_string()]
        );
        assert_eq!(chat.turn.summary, "goal refined by llm");
        assert_eq!(
            chat.goal.signals.get("suggested_needs").map(String::as_str),
            Some("observe: inspect README")
        );
        assert_eq!(
            chat.goal.signals.get("chat_source").map(String::as_str),
            Some("llm")
        );
        assert!(harness.state.routes.score(&NeedKind::Remember, "memory") > 1.0);
    }

    #[test]
    fn clean_brain_explore_can_use_llm_for_need_suggestions() {
        struct FakeChat;

        impl ChatClient for FakeChat {
            fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                let combined = messages
                    .iter()
                    .map(|message| message.content.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(combined.contains("Goal, Mem, Need, and Feed"));
                assert!(combined.contains("Do not choose tools"));
                Ok(ChatResponse {
                    content: r#"{"summary":"clean exploration","needs":[{"kind":"compare","query":"current evidence against goal"}]}"#.to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let state = HarnessState {
            goal: Some(Goal::new("keep tools outside the brain")),
            ..HarnessState::default()
        };
        let mut client = FakeChat;

        let report = state
            .clean_brain_explore_with_client("what next", 5, &mut client)
            .unwrap();

        assert_eq!(report.source, "llm");
        assert_eq!(report.summary, "clean exploration");
        assert_eq!(report.needs[0].kind, NeedKind::Compare);
        assert_eq!(report.audit.status, Status::Satisfied);
        assert_eq!(report.audit.summary, "all Needs stay cognitive");
    }

    #[test]
    fn clean_brain_brief_with_client_keeps_context_clean() {
        struct FakeChat;

        impl ChatClient for FakeChat {
            fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                let combined = messages
                    .iter()
                    .map(|message| message.content.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(combined.contains("clean-brain brief layer"));
                assert!(combined.contains("Goal, Mem, Need, and Feed"));
                assert!(combined.contains("Do not choose tools"));
                assert!(!combined.contains("ToolSpec"));
                Ok(ChatResponse {
                    content: r#"{"summary":"compact cognitive brief","needs":[{"kind":"verify","query":"which claims remain uncertain"},{"kind":"compare","query":"which goal direction deserves attention"}]}"#.to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let state = HarnessState {
            goal: Some(Goal::new("give the next LLM a clean brief")),
            ..HarnessState::default()
        };
        let mut client = FakeChat;
        let report = state
            .clean_brain_brief_with_client("compress the brain state", 5, &mut client)
            .unwrap();

        assert_eq!(report.source, "llm_brief");
        assert_eq!(report.summary, "compact cognitive brief");
        assert_eq!(report.needs[0].kind, NeedKind::Verify);
        assert_eq!(report.audit.status, Status::Satisfied);
        assert!(state.feed_traces.is_empty());
        assert!(state.routes.scores.is_empty());
    }

    #[test]
    fn clean_brain_intent_with_client_keeps_context_clean() {
        struct FakeChat;

        impl ChatClient for FakeChat {
            fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                let combined = messages
                    .iter()
                    .map(|message| message.content.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(combined.contains("clean-brain intent layer"));
                assert!(combined.contains("Goal, Mem, Need, and Feed"));
                assert!(combined.contains("Do not choose tools"));
                assert!(!combined.contains("ToolSpec"));
                Ok(ChatResponse {
                    content: r#"{"summary":"intent map","needs":[{"kind":"verify","query":"whether the goal has enough evidence"},{"kind":"remember","query":"why this intent was chosen"}]}"#.to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let state = HarnessState {
            goal: Some(Goal::new("free the clean brain to choose intent")),
            ..HarnessState::default()
        };
        let mut client = FakeChat;
        let report = state
            .clean_brain_intent_with_client("what should the brain intend next?", 5, &mut client)
            .unwrap();

        assert_eq!(report.source, "llm_intent");
        assert_eq!(report.summary, "intent map");
        assert_eq!(report.needs[0].kind, NeedKind::Verify);
        assert_eq!(report.audit.status, Status::Satisfied);
        assert!(state.feed_traces.is_empty());
        assert!(state.routes.scores.is_empty());
    }

    #[test]
    fn clean_brain_rewrite_uses_llm_without_feed() {
        struct FakeChat;

        impl ChatClient for FakeChat {
            fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                let combined = messages
                    .iter()
                    .map(|message| message.content.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(combined.contains("Need rewrite layer"));
                assert!(combined.contains("rejected Need candidates"));
                assert!(combined.contains("cargo test -p octopus-core"));
                Ok(ChatResponse {
                    content: r#"{"summary":"rewritten cleanly","needs":[{"kind":"verify","query":"whether test evidence supports the goal"}]}"#.to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let state = HarnessState {
            goal: Some(Goal::new("keep the main brain clean")),
            ..HarnessState::default()
        };
        let mut client = FakeChat;

        let report = state
            .clean_brain_rewrite_with_client(
                "rewrite polluted Needs",
                5,
                BrainExploreDraft {
                    summary: "dirty draft".to_string(),
                    needs: vec![GoalNeedSuggestion {
                        kind: NeedKind::Execute,
                        query: "cargo test -p octopus-core".to_string(),
                    }],
                },
                &mut client,
            )
            .unwrap();

        assert_eq!(report.source, "llm_rewrite");
        assert_eq!(report.summary, "rewritten cleanly");
        assert_eq!(report.needs.len(), 1);
        assert_eq!(report.audit.status, Status::Satisfied);
        assert_eq!(
            report.audit.clean_needs[0].query,
            "whether test evidence supports the goal"
        );
        assert!(state.feed_traces.is_empty());
        assert!(state.routes.scores.is_empty());
    }

    #[test]
    fn clean_brain_need_audit_flags_implementation_burden_without_feed() {
        let mut state = HarnessState {
            goal: Some(Goal::new("keep main brain clean")),
            ..HarnessState::default()
        };
        let report = state.clean_brain_explore_from_draft(
            "what next",
            5,
            BrainExploreDraft {
                summary: "external reply".to_string(),
                needs: vec![
                    GoalNeedSuggestion {
                        kind: NeedKind::Verify,
                        query: "check whether the goal is satisfied".to_string(),
                    },
                    GoalNeedSuggestion {
                        kind: NeedKind::Execute,
                        query: "cargo test -p octopus-core".to_string(),
                    },
                    GoalNeedSuggestion {
                        kind: NeedKind::Observe,
                        query: "read crates/octopus-core/src/lib.rs".to_string(),
                    },
                ],
            },
        );

        assert_eq!(report.audit.status, Status::Partial);
        assert_eq!(report.audit.clean_count, 1);
        assert_eq!(report.audit.issue_count, 2);
        assert_eq!(report.audit.clean_needs.len(), 1);
        assert_eq!(
            report.audit.clean_needs[0].query,
            "check whether the goal is satisfied"
        );
        assert_eq!(report.audit.issues[0].signal, "command");
        assert_eq!(report.audit.issues[1].signal, "file_or_path");
        assert_eq!(report.next.len(), 1);
        assert!(report.next[0].contains("check whether the goal is satisfied"));
        assert!(!report.next.iter().any(|next| next.contains("cargo test")));

        let saved = state.queue_exploration_report(&report);
        assert_eq!(saved.queued.len(), 1);
        assert_eq!(saved.queue.pending.len(), 1);
        assert_eq!(
            saved.queue.pending[0].need.query,
            "check whether the goal is satisfied"
        );
        assert!(state.feed_traces.is_empty());
        assert!(state.routes.scores.is_empty());
    }

    #[test]
    fn clean_brain_goal_refines_goal_without_feed() {
        struct FakeChat;

        impl ChatClient for FakeChat {
            fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                let combined = messages
                    .iter()
                    .map(|message| message.content.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(combined.contains("Goal, Mem, Need, and Feed"));
                assert!(combined.contains("Do not choose tools"));
                Ok(ChatResponse {
                    content: r#"{"objective":"build a focused Octopus","constraints":["keep execution outside the brain"],"summary":"goal tuned","needs":[{"kind":"verify","query":"goal boundary is clear"}]}"#.to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let mut state = HarnessState {
            goal: Some(Goal::new("build Octopus")),
            ..HarnessState::default()
        };
        let mut client = FakeChat;

        let report = state
            .clean_brain_goal_with_client("tighten goal", 5, &mut client)
            .unwrap();
        let saved = state.queue_goal_report(&report);

        assert_eq!(report.policy, CLEAN_BRAIN_CONTEXT_POLICY);
        assert_eq!(report.source, "llm");
        assert_eq!(report.goal.objective, "build a focused Octopus");
        assert_eq!(
            report.goal.constraints,
            vec!["keep execution outside the brain".to_string()]
        );
        assert_eq!(report.summary, "goal tuned");
        assert_eq!(report.needs[0].kind, NeedKind::Verify);
        assert_eq!(report.audit.status, Status::Satisfied);
        assert_eq!(state.goal_turns.len(), 1);
        assert_eq!(state.pending_need_queue_count(), 1);
        assert_eq!(saved.queued.len(), 1);
        assert!(state.feed_traces.is_empty());
        assert!(state.routes.scores.is_empty());
    }

    #[test]
    fn clean_brain_focus_with_client_keeps_context_clean() {
        struct FakeChat;

        impl ChatClient for FakeChat {
            fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                let combined = messages
                    .iter()
                    .map(|message| message.content.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(combined.contains("Goal, Mem, Need, and Feed"));
                assert!(combined.contains("Clean brain focus kind: verify"));
                assert!(combined.contains("Do not choose tools"));
                assert!(!combined.contains("ToolSpec"));
                Ok(ChatResponse {
                    content: r#"{"summary":"verify focus","needs":[{"kind":"verify","query":"whether evidence supports the goal"}]}"#.to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let state = HarnessState {
            goal: Some(Goal::new("keep focus clean")),
            ..HarnessState::default()
        };
        let mut client = FakeChat;
        let report = state
            .clean_brain_focus_with_client(NeedKind::Verify, "proof", 5, &mut client)
            .unwrap();

        assert_eq!(report.source, "llm_focus_verify");
        assert_eq!(report.summary, "verify focus");
        assert_eq!(report.needs[0].kind, NeedKind::Verify);
        assert_eq!(report.audit.status, Status::Satisfied);
        assert!(state.feed_traces.is_empty());
        assert!(state.routes.scores.is_empty());
    }

    #[test]
    fn need_queue_take_returns_command_without_execution() {
        let mut state = HarnessState {
            goal: Some(Goal::new("review Needs before Feed")),
            ..HarnessState::default()
        };
        let report = state.clean_brain_explore_from_draft(
            "review next",
            5,
            BrainExploreDraft {
                summary: "external model need".to_string(),
                needs: vec![GoalNeedSuggestion {
                    kind: NeedKind::Verify,
                    query: "whether the next Feed supports the Goal".to_string(),
                }],
            },
        );
        let saved = state.queue_exploration_report(&report);
        let index = saved.queued[0].index;

        let taken = state.take_queued_need(index).unwrap();

        assert_eq!(taken.item.status, NeedQueueStatus::Taken);
        assert!(taken.command.starts_with("octopus need "));
        assert_eq!(state.pending_need_queue_count(), report.needs.len() - 1);
        assert!(state.feed_traces.is_empty());
    }

    #[test]
    fn clean_brain_prompt_exports_messages_without_feed() {
        let mut state = HarnessState {
            goal: Some(Goal::new("let the brain ask clean Needs")),
            ..HarnessState::default()
        };
        state
            .memory
            .remember("keep tool details outside the main brain");

        let report = state.clean_brain_prompt("what should I ask next", 5);

        assert_eq!(report.policy, CLEAN_BRAIN_CONTEXT_POLICY);
        assert_eq!(report.messages.len(), 2);
        assert!(report.prompt_text.contains("Goal + Mem + Need + Feed"));
        assert!(report.prompt_text.contains("what should I ask next"));
        assert!(report
            .messages
            .iter()
            .all(|message| !message.content.contains("ToolSpec")));
        assert!(state.feed_traces.is_empty());
        assert!(state.routes.scores.is_empty());
    }

    #[test]
    fn clean_brain_scout_with_client_keeps_context_clean() {
        struct FakeChat;
        impl ChatClient for FakeChat {
            fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                let combined = messages
                    .iter()
                    .map(|message| message.content.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(combined.contains("clean-brain scout layer"));
                assert!(combined.contains("Goal, Mem, Need, and Feed"));
                assert!(combined.contains("Do not choose tools"));
                assert!(!combined.contains("ToolSpec"));
                Ok(ChatResponse {
                    content: "{\"summary\":\"scout map\",\"observations\":[\"signal\"],\"questions\":[\"unknown\"],\"options\":[\"direction\"],\"risks\":[\"drift\"],\"needs\":[{\"kind\":\"compare\",\"query\":\"candidate cognitive directions\"}]}".to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let state = HarnessState {
            goal: Some(Goal::new("keep the clean brain exploratory")),
            ..HarnessState::default()
        };
        let mut client = FakeChat;

        let report = state
            .clean_brain_scout_with_client("map the goal", 5, &mut client)
            .unwrap();

        assert_eq!(report.source, "llm_scout");
        assert_eq!(report.summary, "scout map");
        assert_eq!(report.audit.issue_count, 0);
        assert_eq!(report.needs[0].kind, NeedKind::Compare);
        assert!(state.feed_traces.is_empty());
        assert!(state.routes.scores.is_empty());
    }

    #[test]
    fn clean_brain_align_with_client_keeps_context_clean() {
        struct FakeChat;

        impl ChatClient for FakeChat {
            fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                let combined = messages
                    .iter()
                    .map(|message| message.content.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(combined.contains("clean-brain alignment layer"));
                assert!(combined.contains("Goal, Mem, Need, and Feed"));
                assert!(combined.contains("Do not choose tools"));
                assert!(!combined.contains("ToolSpec"));
                Ok(ChatResponse {
                    content: r#"{"summary":"alignment ok","goal_state":"aligned","evidence":["goal is explicit"],"gaps":["need proof"],"questions":["which Need best serves the goal?"],"needs":[{"kind":"verify","query":"whether the next Need follows the Goal"}]}"#.to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let state = HarnessState {
            goal: Some(Goal::new("align clean-brain Needs")),
            ..HarnessState::default()
        };
        let mut client = FakeChat;
        let report = state
            .clean_brain_align_with_client("check alignment", 5, &mut client)
            .unwrap();

        assert_eq!(report.source, "llm_align");
        assert_eq!(report.goal_state, "aligned");
        assert_eq!(report.audit.status, Status::Satisfied);
        assert!(state.feed_traces.is_empty());
        assert!(state.routes.scores.is_empty());
    }

    #[test]
    fn clean_brain_synthesizes_model_drafts_without_feed() {
        let mut state = HarnessState {
            goal: Some(Goal::new(
                "use model diversity while keeping brain context clean",
            )),
            ..HarnessState::default()
        };
        state
            .memory
            .remember("synthesis must still emit Needs only");
        let input = BrainSynthesisInput {
            summary: Some("model jury".to_string()),
            observations: Vec::new(),
            questions: Vec::new(),
            options: Vec::new(),
            risks: Vec::new(),
            needs: Vec::new(),
            drafts: vec![
                BrainSynthesisDraft {
                    source: Some("model-a".to_string()),
                    summary: Some("draft a".to_string()),
                    observations: vec!["goal needs evidence".to_string()],
                    questions: vec!["which evidence is missing?".to_string()],
                    options: vec!["verify user-facing behavior".to_string()],
                    risks: vec!["model may leak tool choices".to_string()],
                    needs: vec![
                        GoalNeedSuggestion {
                            kind: NeedKind::Verify,
                            query: "whether current evidence is enough".to_string(),
                        },
                        GoalNeedSuggestion {
                            kind: NeedKind::Execute,
                            query: "cargo test -p octopus-core".to_string(),
                        },
                    ],
                },
                BrainSynthesisDraft {
                    source: Some("model-b".to_string()),
                    summary: Some("draft b".to_string()),
                    observations: vec!["goal needs evidence".to_string()],
                    questions: vec!["what should be remembered?".to_string()],
                    options: vec!["compare recent Feed with goal".to_string()],
                    risks: vec!["next Need could be too broad".to_string()],
                    needs: vec![
                        GoalNeedSuggestion {
                            kind: NeedKind::Verify,
                            query: "whether current evidence is enough".to_string(),
                        },
                        GoalNeedSuggestion {
                            kind: NeedKind::Remember,
                            query: "draft diversity stayed cognitive".to_string(),
                        },
                    ],
                },
            ],
        };

        let report = state.clean_brain_synthesize_from_input("merge model drafts", 5, input);

        assert_eq!(report.policy, CLEAN_BRAIN_CONTEXT_POLICY);
        assert_eq!(report.source, "external_synthesis");
        assert_eq!(report.draft_count, 2);
        assert_eq!(report.needs.len(), 3);
        assert_eq!(report.audit.clean_count, 2);
        assert_eq!(report.audit.issue_count, 1);
        assert!(report
            .audit
            .clean_needs
            .iter()
            .any(|need| need.query == "whether current evidence is enough"));
        assert!(report
            .audit
            .clean_needs
            .iter()
            .any(|need| need.query == "draft diversity stayed cognitive"));
        assert!(state.feed_traces.is_empty());
        assert!(state.routes.scores.is_empty());

        let saved = state.queue_synthesis_report(&report);

        assert_eq!(saved.queued.len(), 2);
        assert_eq!(state.pending_need_queue_count(), 2);
        assert!(state.feed_traces.is_empty());
        assert!(state.routes.scores.is_empty());
    }

    #[test]
    fn self_iteration_requires_oauth_grant_for_pr_mode() {
        let mut state = HarnessState::default();

        let report_only =
            state.self_iteration_plan("dangoZhang/Octopus", Some("improve usability"));
        assert!(!report_only.authorized);
        assert_eq!(report_only.mode, "report-only");
        assert_eq!(
            report_only.next_action,
            "octopus oauth github dangoZhang/Octopus"
        );
        assert!(report_only.draft.is_none());
        assert!(report_only
            .steps
            .contains(&"report only until an OAuth grant exists".to_string()));

        let grant = state.grant_oauth(
            "github",
            "dangoZhang/Octopus",
            default_permissions("github"),
        );
        let pr_ready = state.self_iteration_plan("dangoZhang/Octopus", Some("improve usability"));

        assert!(pr_ready.authorized);
        assert_eq!(pr_ready.mode, "pr-ready");
        assert_eq!(
            pr_ready.next_action,
            "OCTOPUS_PR_DRY_RUN=1 octopus self-iterate pr dangoZhang/Octopus 'improve usability'"
        );
        assert_eq!(pr_ready.grant_id, Some(grant.id));
        assert_eq!(pr_ready.candidates[0].branch, "octopus/improve-usability");
        let draft = pr_ready.draft.as_ref().unwrap();
        assert_eq!(draft.branch, "octopus/improve-usability");
        assert_eq!(draft.title, "Improve usability");
        assert!(draft.body.contains("cargo test"));
        assert!(pr_ready
            .guardrails
            .contains(&"never push to main directly".to_string()));
    }

    #[test]
    fn invalid_state_file_returns_error() {
        let path =
            std::env::temp_dir().join(format!("octopus-invalid-state-{}.json", std::process::id()));
        fs::write(&path, "{bad json").unwrap();

        let error = HarnessState::load(&path).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidData);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn planning_tentacle_selects_tool_before_execution() {
        struct FakeChat;

        impl ChatClient for FakeChat {
            fn chat(&mut self, _messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                Ok(ChatResponse {
                    content: r#"{"calls":[{"tool":"verifier","reason":"model chose verifier","payload":{}}],"summary":"planned by model"}"#.to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let tool = FunctionTool::new(
            "verifier",
            "verifies claims",
            vec![NeedKind::Verify],
            |need| ToolResult::satisfied("verifier", format!("verified {}", need.query)),
        );
        let mut harness = Harness::new();
        harness.add_tentacle(Box::new(PlanningTentacle::new(
            "research",
            vec![NeedKind::Verify],
            ChatPlanner::new(FakeChat),
            vec![Box::new(tool)],
        )));

        let feed = harness.feed_one(&Need::new(NeedKind::Verify, "clean brain"));

        assert_eq!(feed.status, Status::Satisfied);
        assert_eq!(feed.summary, "verified clean brain");
        assert_eq!(feed.metadata.get("tools"), Some(&"verifier".to_string()));
        assert_eq!(
            feed.metadata.get("tentacle_brain"),
            Some(&"research".to_string())
        );
        assert!(harness.state.routes.score(&NeedKind::Verify, "research") > 1.0);
    }

    #[test]
    fn chat_planner_uses_chat_client_plan() {
        struct FakeChat;

        impl ChatClient for FakeChat {
            fn chat(&mut self, _messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                Ok(ChatResponse {
                    content: r#"{"calls":[{"tool":"verifier","reason":"best match","payload":{}}],"summary":"planned by chat"}"#.to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let mut planner = ChatPlanner::new(FakeChat);
        let plan = planner.plan(
            &Need::new(NeedKind::Verify, "claim"),
            &[ToolSpec {
                name: "verifier".to_string(),
                description: "checks claims".to_string(),
            }],
        );

        assert_eq!(plan.summary, "planned by chat");
        assert_eq!(plan.calls[0].tool, "verifier");
    }

    #[test]
    fn chat_planner_has_no_hidden_rule_execution() {
        struct BadChat;

        impl ChatClient for BadChat {
            fn chat(&mut self, _messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                Ok(ChatResponse {
                    content: "not json".to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let mut planner = ChatPlanner::new(BadChat);
        let plan = planner.plan(
            &Need::new(NeedKind::Verify, "claim"),
            &[ToolSpec {
                name: "verifier".to_string(),
                description: "checks claims".to_string(),
            }],
        );

        assert!(plan.calls.is_empty());
        assert!(plan.summary.contains("invalid JSON"));
    }

    #[test]
    fn openai_compatible_chat_client_builds_payload_and_parses_response() {
        let config = OpenAiCompatibleConfig {
            model: "test-model".to_string(),
            api_key: Some("token".to_string()),
            base_url: "https://llm.example/v1/".to_string(),
            timeout_seconds: 3,
            retry_count: 0,
            retry_delay_ms: 0,
            curl_command: "curl".to_string(),
            tuning: OpenAiCompatibleTuning {
                temperature: Some(0.2),
                top_p: Some(0.9),
                max_tokens: Some(512),
                reasoning_effort: Some("medium".to_string()),
                extra_body: BTreeMap::from([(
                    "response_format".to_string(),
                    serde_json::json!({"type": "json_object"}),
                )]),
            },
        };
        let client = OpenAiCompatibleChatClient::new(config.clone());
        let body = client
            .request_body(&[
                ChatMessage::new(ChatRole::System, "system"),
                ChatMessage::new(ChatRole::User, "user"),
            ])
            .unwrap();

        assert_eq!(config.endpoint(), "https://llm.example/v1/chat/completions");
        assert!(body.contains("\"model\":\"test-model\""));
        assert!(body.contains("\"role\":\"system\""));
        assert!(body.contains("\"role\":\"user\""));
        assert!(body.contains("\"temperature\":0.2"));
        assert!(body.contains("\"top_p\":0.9"));
        assert!(body.contains("\"max_tokens\":512"));
        assert!(body.contains("\"reasoning_effort\":\"medium\""));
        assert!(body.contains("\"response_format\":{\"type\":\"json_object\"}"));
    }

    #[test]
    fn openai_compatible_config_reads_tuning_env_without_context_override() {
        let prefix = format!("OCTOPUS_TEST_TUNING_{}", std::process::id());
        let keys = [
            "MODEL",
            "BASE_URL",
            "API_KEY",
            "RETRIES",
            "RETRY_MS",
            "TEMPERATURE",
            "TOP_P",
            "MAX_TOKENS",
            "REASONING_EFFORT",
            "EXTRA_BODY",
        ]
        .into_iter()
        .map(|suffix| format!("{prefix}_{suffix}"))
        .collect::<Vec<_>>();
        for key in &keys {
            std::env::remove_var(key);
        }
        std::env::set_var(format!("{prefix}_MODEL"), "clean-model");
        std::env::set_var(format!("{prefix}_BASE_URL"), "https://llm.example/v1");
        std::env::set_var(format!("{prefix}_API_KEY"), "token");
        std::env::set_var(format!("{prefix}_RETRIES"), "2");
        std::env::set_var(format!("{prefix}_RETRY_MS"), "5");
        std::env::set_var(format!("{prefix}_TEMPERATURE"), "0.3");
        std::env::set_var(format!("{prefix}_TOP_P"), "0.8");
        std::env::set_var(format!("{prefix}_MAX_TOKENS"), "1024");
        std::env::set_var(format!("{prefix}_REASONING_EFFORT"), "high");
        std::env::set_var(
            format!("{prefix}_EXTRA_BODY"),
            r#"{"metadata":{"purpose":"clean-brain"},"model":"dirty-model","messages":[{"role":"user","content":"dirty"}]}"#,
        );

        let config = OpenAiCompatibleConfig::from_env_prefix(&prefix).unwrap();
        let client = OpenAiCompatibleChatClient::new(config.clone());
        let body = client
            .request_body(&[ChatMessage::new(ChatRole::User, "clean need")])
            .unwrap();

        assert_eq!(config.tuning.temperature, Some(0.3));
        assert_eq!(config.retry_count, 2);
        assert_eq!(config.retry_delay_ms, 5);
        assert_eq!(config.tuning.top_p, Some(0.8));
        assert_eq!(config.tuning.max_tokens, Some(1024));
        assert_eq!(config.tuning.reasoning_effort.as_deref(), Some("high"));
        assert!(body.contains("\"model\":\"clean-model\""));
        assert!(body.contains("\"content\":\"clean need\""));
        assert!(!body.contains("dirty-model"));
        assert!(!body.contains("\"content\":\"dirty\""));
        assert!(body.contains("\"metadata\":{\"purpose\":\"clean-brain\"}"));
        for key in &keys {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn codex_cli_config_reads_retry_and_timeout_env() {
        let prefix = format!("OCTOPUS_TEST_CODEX_{}", std::process::id());
        let keys = ["MODEL", "CODEX_COMMAND", "TIMEOUT", "RETRIES", "RETRY_MS"]
            .into_iter()
            .map(|suffix| format!("{prefix}_{suffix}"))
            .collect::<Vec<_>>();
        for key in &keys {
            std::env::remove_var(key);
        }
        std::env::set_var(format!("{prefix}_MODEL"), "codex-model");
        std::env::set_var(format!("{prefix}_CODEX_COMMAND"), "codex-test");
        std::env::set_var(format!("{prefix}_TIMEOUT"), "9");
        std::env::set_var(format!("{prefix}_RETRIES"), "3");
        std::env::set_var(format!("{prefix}_RETRY_MS"), "15");

        let config = CodexCliConfig::from_env_prefix(&prefix).unwrap();

        assert_eq!(config.model.as_deref(), Some("codex-model"));
        assert_eq!(config.command, "codex-test");
        assert_eq!(config.timeout_seconds, 9);
        assert_eq!(config.retry_count, 3);
        assert_eq!(config.retry_delay_ms, 15);
        for key in &keys {
            std::env::remove_var(key);
        }
    }

    #[cfg(unix)]
    #[test]
    fn openai_compatible_chat_client_uses_curl_adapter() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!("octopus-fake-curl-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let curl = dir.join("fake-curl.sh");
        fs::write(
            &curl,
            "#!/bin/sh\nprintf '%s' '{\"choices\":[{\"message\":{\"content\":\"provider ok\"}}]}'\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&curl).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&curl, permissions).unwrap();
        let mut client = OpenAiCompatibleChatClient::new(OpenAiCompatibleConfig {
            model: "test-model".to_string(),
            api_key: None,
            base_url: "https://llm.example/v1".to_string(),
            timeout_seconds: 1,
            retry_count: 0,
            retry_delay_ms: 0,
            curl_command: curl.to_string_lossy().to_string(),
            tuning: OpenAiCompatibleTuning::default(),
        });

        let response = client
            .chat(&[ChatMessage::new(ChatRole::User, "hello")])
            .unwrap();

        assert_eq!(response.content, "provider ok");
        assert_eq!(
            response.metadata.get("provider"),
            Some(&"openai-compatible".to_string())
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn openai_compatible_chat_client_rejects_empty_content() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!("octopus-empty-curl-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let curl = dir.join("fake-curl.sh");
        fs::write(
            &curl,
            "#!/bin/sh\nprintf '%s' '{\"choices\":[{\"message\":{\"content\":\"\"}}]}'\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&curl).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&curl, permissions).unwrap();
        let mut client = OpenAiCompatibleChatClient::new(OpenAiCompatibleConfig {
            model: "test-model".to_string(),
            api_key: None,
            base_url: "https://llm.example/v1".to_string(),
            timeout_seconds: 1,
            retry_count: 0,
            retry_delay_ms: 0,
            curl_command: curl.to_string_lossy().to_string(),
            tuning: OpenAiCompatibleTuning::default(),
        });

        let error = client
            .chat(&[ChatMessage::new(ChatRole::User, "hello")])
            .unwrap_err();

        assert!(error.contains("non-empty"));
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn openai_compatible_chat_client_retries_transient_failures() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!("octopus-retry-curl-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let curl = dir.join("fake-curl.sh");
        let marker = dir.join("called");
        fs::write(
            &curl,
            format!(
                "#!/bin/sh\nif [ ! -f '{}' ]; then touch '{}'; exit 22; fi\nprintf '%s' '{{\"choices\":[{{\"message\":{{\"content\":\"retry ok\"}}}}]}}'\n",
                marker.display(),
                marker.display()
            ),
        )
        .unwrap();
        let mut permissions = fs::metadata(&curl).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&curl, permissions).unwrap();
        let mut client = OpenAiCompatibleChatClient::new(OpenAiCompatibleConfig {
            model: "test-model".to_string(),
            api_key: None,
            base_url: "https://llm.example/v1".to_string(),
            timeout_seconds: 1,
            retry_count: 1,
            retry_delay_ms: 1,
            curl_command: curl.to_string_lossy().to_string(),
            tuning: OpenAiCompatibleTuning::default(),
        });

        let response = client
            .chat(&[ChatMessage::new(ChatRole::User, "hello")])
            .unwrap();

        assert_eq!(response.content, "retry ok");
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn codex_cli_chat_client_retries_transient_failures() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!("octopus-retry-codex-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let marker = dir.join("called");
        let codex = dir.join("fake-codex.sh");
        fs::write(
            &codex,
            format!(
                "#!/bin/sh\nout=''\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = '--output-last-message' ]; then shift; out=\"$1\"; fi\n  shift || true\ndone\nif [ ! -f '{}' ]; then touch '{}'; echo transient >&2; exit 1; fi\nprintf '%s' 'codex retry ok' > \"$out\"\n",
                marker.display(),
                marker.display()
            ),
        )
        .unwrap();
        let mut permissions = fs::metadata(&codex).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&codex, permissions).unwrap();
        let mut client = CodexCliChatClient::new(CodexCliConfig {
            model: None,
            command: codex.to_string_lossy().to_string(),
            timeout_seconds: 1,
            retry_count: 1,
            retry_delay_ms: 1,
        });

        let response = client
            .chat(&[ChatMessage::new(ChatRole::User, "hello")])
            .unwrap();

        assert_eq!(response.content, "codex retry ok");
        assert_eq!(
            response.metadata.get("provider"),
            Some(&"codex-cli".to_string())
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn codex_cli_chat_client_disables_image_generation() {
        use std::os::unix::fs::PermissionsExt;

        let dir =
            std::env::temp_dir().join(format!("octopus-codex-text-only-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let args_path = dir.join("args.txt");
        let codex = dir.join("fake-codex.sh");
        fs::write(
            &codex,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" > '{}'\nout=''\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = '--output-last-message' ]; then shift; out=\"$1\"; fi\n  shift || true\ndone\nprintf '%s' 'codex text ok' > \"$out\"\n",
                args_path.display()
            ),
        )
        .unwrap();
        let mut permissions = fs::metadata(&codex).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&codex, permissions).unwrap();
        let mut client = CodexCliChatClient::new(CodexCliConfig {
            model: Some("codex-model".to_string()),
            command: codex.to_string_lossy().to_string(),
            timeout_seconds: 1,
            retry_count: 0,
            retry_delay_ms: 1,
        });

        let response = client
            .chat(&[ChatMessage::new(ChatRole::User, "hello")])
            .unwrap();

        let args = fs::read_to_string(&args_path).unwrap();
        assert_eq!(response.content, "codex text ok");
        assert!(args.contains("--disable image_generation"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn default_catalog_contains_installable_profiles() {
        let profiles = default_tentacle_profiles();

        assert!(profiles.iter().any(|profile| profile.id == "research"));
        assert!(profiles.iter().any(|profile| profile.id == "visual"));
        assert!(profiles
            .iter()
            .any(|profile| profile.id == "repo-maintainer"));
        assert!(profiles.iter().any(|profile| profile.id == "json-feed"));
        assert!(profiles.iter().any(|profile| profile.id == "swe-agent"));
        assert!(profiles
            .iter()
            .any(|profile| profile.id == "computer-use-agent"));
        assert!(profiles.iter().any(|profile| profile.id == "bash-only"));
        assert!(profiles
            .iter()
            .any(|profile| profile.skills.iter().any(|skill| skill.id == "memory")));
        assert!(profiles
            .iter()
            .all(|profile| !profile.brain.prompt.is_empty()));
        assert!(profiles.iter().all(|profile| !profile.tools.is_empty()));
        assert!(profiles
            .iter()
            .all(|profile| !profile.evolution.checks.is_empty()));
        assert!(profiles.iter().all(|profile| {
            profile
                .tools
                .iter()
                .all(|tool| tool.implementation.contract.is_some())
        }));
        assert!(profiles
            .iter()
            .find(|profile| profile.id == "visual")
            .unwrap()
            .tools
            .iter()
            .any(|tool| tool.implementation.entrypoint == "docs/pet.html"
                && tool.implementation.contract.as_deref() == Some(OCTOPUS_STATIC_HTML_CONTRACT)));
        assert!(profiles
            .iter()
            .find(|profile| profile.id == "research")
            .unwrap()
            .tools
            .iter()
            .any(|tool| tool.implementation.contract.as_deref() == Some(OCTOPUS_ADAPTER_CONTRACT)));
        assert!(profiles
            .iter()
            .find(|profile| profile.skills.iter().any(|skill| skill.id == "memory"))
            .unwrap()
            .tools
            .iter()
            .any(|tool| tool.implementation.contract.as_deref()
                == Some(OCTOPUS_NATIVE_HARNESS_CONTRACT)));
        assert!(profiles
            .iter()
            .find(|profile| profile.id == "repo-maintainer")
            .unwrap()
            .tools
            .iter()
            .any(|tool| tool.implementation.entrypoint
                == "tentacles/repo-maintainer/tools/draft_pr.sh"));
        assert!(profiles
            .iter()
            .find(|profile| profile.id == "json-feed")
            .unwrap()
            .tools
            .iter()
            .any(|tool| tool.implementation.kind == "python"
                && tool.implementation.contract.as_deref() == Some(OCTOPUS_JSON_CONTRACT)));
    }

    #[test]
    fn seed_manifests_declare_explicit_tool_contracts() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let manifests = load_tentacle_manifests(&root).unwrap();

        assert!(!manifests.is_empty());
        for loaded in manifests {
            for tool in &loaded.manifest.tools {
                assert!(
                    tool.implementation.contract.is_some(),
                    "{}:{} missing contract",
                    loaded.manifest.id,
                    tool.id
                );
            }
        }
    }

    #[test]
    fn profile_registry_file_supplies_profiles() {
        let dir =
            std::env::temp_dir().join(format!("octopus-profile-registry-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let registry = dir.join("default.json");
        fs::write(
            &registry,
            r#"[
  {
    "id": "registry-probe",
    "name": "Registry Probe",
    "description": "Loaded from an external profile registry.",
    "brain": {
      "kind": "llm",
      "description": "Probe",
      "model": null,
      "prompt": "Return compact Feed."
    },
    "skills": [
      {
        "id": "probe",
        "name": "Probe",
        "description": "Probe external registry loading.",
        "needs": ["observe"],
        "tools": ["probe"]
      }
    ],
    "tools": [
      {
        "id": "probe",
        "description": "External registry probe tool.",
        "implementation": { "kind": "adapter", "entrypoint": "probe" }
      }
    ],
    "evolution": {
      "editable": ["manifest.json", "brain.prompt", "tools/*"],
      "checks": ["echo ok"],
      "constraints": ["stay compact"]
    },
    "llm_ready": true
  }
]"#,
        )
        .unwrap();
        let profiles = load_tentacle_profiles_from_path(&registry).unwrap();

        let _ = fs::remove_dir_all(dir);
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].id, "registry-probe");
    }

    #[test]
    fn repo_tentacle_manifests_are_code_as_harness() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let reports = inspect_tentacle_manifests(&root).unwrap();

        assert!(reports.iter().any(|report| report.id == "swe-agent"));
        assert!(reports
            .iter()
            .filter(|report| report.id != "visual")
            .all(|report| report.brain_kind == "llm"));
        assert_eq!(
            reports
                .iter()
                .find(|report| report.id == "visual")
                .unwrap()
                .brain_kind,
            "rule"
        );
        assert!(reports
            .iter()
            .all(|report| report.missing_entrypoints.is_empty()));
        assert!(reports
            .iter()
            .find(|report| report.id == "computer-use-agent")
            .unwrap()
            .runtime_kinds
            .contains(&"mcp".to_string()));
        assert!(reports
            .iter()
            .find(|report| report.id == "json-feed")
            .unwrap()
            .runtime_kinds
            .contains(&"python".to_string()));
        assert!(reports
            .iter()
            .find(|report| report.id == "bash-only")
            .unwrap()
            .editable
            .contains(&"brain.prompt".to_string()));
        assert!(reports.iter().all(|report| report
            .evolution_surfaces
            .contains(&"brain_prompt".to_string())));
        assert!(reports
            .iter()
            .all(|report| report.evolution_surfaces.contains(&"tool_meta".to_string())));
        assert!(reports.iter().all(|report| report
            .evolution_surfaces
            .contains(&"runtime_code".to_string())));
    }

    #[test]
    fn loads_tentacle_manifest_prompts_and_needs() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let manifests = load_tentacle_manifests(&root).unwrap();
        let swe = manifests
            .iter()
            .find(|loaded| loaded.manifest.id == "swe-agent")
            .unwrap();
        let json_feed = manifests
            .iter()
            .find(|loaded| loaded.manifest.id == "json-feed")
            .unwrap();

        assert!(swe.manifest.brain.prompt.contains("cognitive Need"));
        assert!(swe.manifest.skills[0].needs.contains(&NeedKind::Verify));
        assert_eq!(swe.manifest.tools[0].implementation.kind, "shell");
        assert_eq!(
            json_feed.manifest.tools[0]
                .implementation
                .contract
                .as_deref(),
            Some(OCTOPUS_JSON_CONTRACT)
        );
    }

    #[test]
    fn manifest_report_accepts_http_runtime_url() {
        let root =
            std::env::temp_dir().join(format!("octopus-http-manifest-{}", std::process::id()));
        let manifest_dir = root.join("http-tentacle");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&manifest_dir).unwrap();
        fs::write(
            manifest_dir.join("manifest.json"),
            r#"{
  "schema_version": "0.0.1",
  "id": "http-tentacle",
  "name": "HTTP Tentacle",
  "description": "Remote tool runtime.",
  "brain": {
    "kind": "llm",
    "model": null,
    "prompt": "Plan a remote feed.",
    "feedback_contract": "Return compact evidence."
  },
  "skills": [
    { "id": "remote-observe", "description": "Remote observe.", "needs": ["observe"] }
  ],
  "tools": [
    {
      "id": "remote",
      "description": "Call a remote endpoint.",
      "input": "octopus-json-v1 tool call",
      "output": "structured feedback",
      "implementation": {
        "kind": "http",
        "entrypoint": "https://example.com/octopus",
        "contract": "octopus-json-v1"
      }
    }
  ],
  "evolution": {
    "editable": ["manifest.json"],
    "surfaces": [
      {
        "id": "brain_prompt",
        "description": "Remote planning prompt.",
        "targets": ["brain.prompt", "brain.feedback_contract"]
      },
      {
        "id": "tool_meta",
        "description": "Remote tool call metadata.",
        "targets": ["tools[].description", "tools[].input", "tools[].output", "tools[].implementation.contract"]
      },
      {
        "id": "runtime_code",
        "description": "Remote HTTP endpoint adapter.",
        "targets": ["tools[].implementation.entrypoint"]
      },
      {
        "id": "evolution_policy",
        "description": "Remote tool checks and constraints.",
        "targets": ["evolution.checks", "evolution.constraints", "evolution.surfaces"]
      }
    ],
    "checks": ["cargo test"],
    "constraints": ["Keep the JSON contract stable."]
  }
}
"#,
        )
        .unwrap();

        let reports = inspect_tentacle_manifests(&root).unwrap();

        assert_eq!(reports[0].runtime_kinds, vec!["http".to_string()]);
        assert!(reports[0].missing_entrypoints.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    struct EvolutionFakeChat {
        response: String,
        responses: Vec<String>,
        prompt: String,
    }

    impl ChatClient for EvolutionFakeChat {
        fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
            self.prompt = messages
                .iter()
                .map(|message| message.content.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            let content = if self.responses.is_empty() {
                self.response.clone()
            } else {
                self.responses.remove(0)
            };
            Ok(ChatResponse {
                content,
                metadata: BTreeMap::new(),
            })
        }
    }

    #[test]
    fn harness_beat_evolution_uses_llm_planner_for_failed_check() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let workspace =
            std::env::temp_dir().join(format!("octopus-llm-harness-beat-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
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
            stderr: "read check failed".to_string(),
        });
        let mut fake = EvolutionFakeChat {
            response: r#"{
              "summary": "repair read harness from failed check",
              "candidates": [
                {
                  "surface_id": "runtime_code",
                  "title": "Repair read command evidence",
                  "target": "tools/read.sh",
                  "rationale": "the failed check points at the read runtime harness",
                  "change_plan": ["patch read.sh output handling"],
                  "checks": ["tools/read.sh README.md 1 2"],
                  "suggested_patch": "diff --git a/tentacles/swe-agent/tools/read.sh b/tentacles/swe-agent/tools/read.sh\n--- a/tentacles/swe-agent/tools/read.sh\n+++ b/tentacles/swe-agent/tools/read.sh\n@@ -1,2 +1,3 @@\n #!/usr/bin/env bash\n+echo checked\n"
                }
              ]
            }"#
            .to_string(),
            responses: Vec::new(),
            prompt: String::new(),
        };

        let evolution = write_harness_beat_evolution_artifacts_with_client(
            &root, &workspace, &state, &mut fake,
        )
        .unwrap()
        .expect("LLM harness beat evolution");

        assert_eq!(evolution.source_kind, "check_history");
        assert_eq!(evolution.tentacle_id, "swe-agent");
        assert_eq!(evolution.candidate_id, "03-runtime-code");
        assert!(fake.prompt.contains("read check failed"));
        assert!(Path::new(&evolution.proposal_path).exists());
        assert!(Path::new(&evolution.apply_plan_path).exists());
        assert!(evolution
            .next_action
            .contains("octopus oauth octopus evolve:swe-agent"));
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn tentacle_evolution_can_use_llm_generated_candidates() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();
        state.record_evolution_outcome(
            "swe-agent",
            "03-runtime-code",
            Status::Satisfied,
            "runtime patch improved evidence",
        );
        state.record_feed_trace_from_feed(&Feed {
            need: Need::new(NeedKind::Observe, "README.md"),
            status: Status::Satisfied,
            evidence: vec![Evidence::new("read", "README evidence")],
            summary: "observed README before evolution".to_string(),
            metadata: BTreeMap::from([
                ("tentacle".to_string(), "swe-agent".to_string()),
                ("tool".to_string(), "read".to_string()),
                ("plan_source".to_string(), "llm".to_string()),
            ]),
        });
        state.record_check_history(CheckHistoryInput {
            tentacle_id: "swe-agent".to_string(),
            source_kind: "manifest".to_string(),
            command_index: Some(1),
            command: "tools/read.sh README.md 1 2".to_string(),
            cwd: "tentacles/swe-agent".to_string(),
            status: Status::Failed,
            code: Some(1),
            stdout: String::new(),
            stderr: "read check failed before evolution".to_string(),
        });
        let mut fake = EvolutionFakeChat {
            response: r#"{
              "summary": "select runtime code because previous runtime changes worked",
              "candidates": [
                {
                  "surface_id": "runtime_code",
                  "title": "Teach read tool to return denser evidence",
                  "target": "tools/read.sh",
                  "rationale": "Need-to-Feed improves when the tool returns compact file evidence",
                  "change_plan": ["keep clean brain context limited", "change only the read adapter"],
                  "checks": ["tentacles/swe-agent/tools/read.sh README.md 1 2"],
                  "suggested_patch": "diff --git a/tentacles/swe-agent/tools/read.sh b/tentacles/swe-agent/tools/read.sh\n--- a/tentacles/swe-agent/tools/read.sh\n+++ b/tentacles/swe-agent/tools/read.sh\n@@ -1,2 +1,3 @@\n #!/bin/sh\n+# provider-assisted draft\n set -eu\n"
                }
              ]
            }"#
            .to_string(),
            responses: Vec::new(),
            prompt: String::new(),
        };

        let proposal = propose_tentacle_evolution_with_client(
            &root,
            "swe-agent",
            "improve observe feed",
            &state,
            &mut fake,
        )
        .unwrap();
        let markdown = render_tentacle_evolution_proposal(&proposal);

        assert_eq!(proposal.generator, "llm");
        assert!(proposal
            .planner_summary
            .contains("previous runtime changes worked"));
        assert_eq!(proposal.patch_candidates.len(), 1);
        assert_eq!(proposal.patch_candidates[0].surface_id, "runtime_code");
        assert!(proposal.patch_candidates[0]
            .target
            .ends_with("tools/read.sh"));
        assert!(proposal.patch_candidates[0]
            .feedback
            .iter()
            .any(|feedback| feedback.kind == "check_history"));
        assert!(proposal.patch_candidates[0]
            .suggested_patch
            .as_deref()
            .is_some_and(|patch| patch.contains("provider-assisted draft")));
        let workspace =
            std::env::temp_dir().join(format!("octopus-llm-evolve-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        let artifact = write_tentacle_evolution_artifacts(&workspace, &proposal).unwrap();
        let draft = fs::read_to_string(
            artifact
                .patch_draft_paths
                .iter()
                .find(|path| path.contains("03-runtime-code"))
                .unwrap(),
        )
        .unwrap();
        assert!(draft.contains("suggested patch draft:"));
        assert!(draft.contains("provider-assisted draft"));
        state.grant_oauth(
            "octopus",
            "evolve:swe-agent",
            default_permissions("octopus"),
        );
        let plan = plan_tentacle_evolution_apply(&proposal, &state, "runtime_code").unwrap();
        let apply_artifact = write_tentacle_apply_artifacts(&workspace, &plan).unwrap();
        let patch = fs::read_to_string(apply_artifact.patch_path.as_ref().unwrap()).unwrap();
        assert!(patch.contains("provider-assisted draft"));
        assert!(patch.contains("b/tentacles/swe-agent/tools/read.sh"));
        assert!(markdown.contains("generator: `llm`"));
        assert!(fake.prompt.contains("Goal"));
        assert!(fake.prompt.contains("Need"));
        assert!(fake.prompt.contains("Tool"));
        assert!(fake.prompt.contains("harness_evolution"));
        assert!(fake.prompt.contains("recent_feed_traces"));
        assert!(fake.prompt.contains("recent_check_history"));
        assert!(fake.prompt.contains("observed README before evolution"));
        assert!(fake.prompt.contains("read check failed before evolution"));
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn llm_field_pack_task_patch_can_touch_multiple_resolved_pack_files() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();
        let mut fake = EvolutionFakeChat {
            response: r#"{
              "summary": "extend peer field pack tasks",
              "candidates": [
                {
                  "surface_id": "field_pack_tasks",
                  "title": "Add harder peer field tasks",
                  "target": "field-packs/*/field-pack.json",
                  "rationale": "harder tasks should live in editable field packs",
                  "change_plan": ["edit task definitions in the field packs"],
                  "checks": ["octopus check field-mini-task 2"],
                  "suggested_patch": "diff --git a/field-packs/math/field-pack.json b/field-packs/math/field-pack.json\n--- a/field-packs/math/field-pack.json\n+++ b/field-packs/math/field-pack.json\n@@ -1,2 +1,3 @@\n {\n+  \"llm_patch_probe\": true,\ndiff --git a/field-packs/search/field-pack.json b/field-packs/search/field-pack.json\n--- a/field-packs/search/field-pack.json\n+++ b/field-packs/search/field-pack.json\n@@ -1,2 +1,3 @@\n {\n+  \"llm_patch_probe\": true,\ndiff --git a/field-packs/index.json b/field-packs/index.json\n--- a/field-packs/index.json\n+++ b/field-packs/index.json\n@@ -1,2 +1,3 @@\n {\n+  \"llm_registry_probe\": true,\n"
                }
              ]
            }"#
            .to_string(),
            responses: Vec::new(),
            prompt: String::new(),
        };

        let proposal = propose_tentacle_evolution_with_client(
            &root,
            "field-mini-task",
            "add a harder mini task layer",
            &state,
            &mut fake,
        )
        .unwrap();
        let candidate = proposal
            .patch_candidates
            .iter()
            .find(|candidate| candidate.surface_id == "field_pack_tasks")
            .unwrap();
        assert!(candidate.target_files.len() >= default_field_pack_ids().len() + 1);
        assert!(candidate
            .target_files
            .iter()
            .any(|path| path.ends_with("field-packs/index.json")));
        assert!(fake.prompt.contains("declared target files"));
        assert!(fake
            .prompt
            .contains("field-pack surfaces may use field-packs"));
        assert!(fake.prompt.contains("target_file_contents"));
        assert!(fake.prompt.contains("field-packs/math/field-pack.json"));
        assert!(fake.prompt.contains("math-mini-1"));
        assert!(fake
            .prompt
            .contains("suggested_patch is required for this surface"));
        assert!(!fake.prompt.contains("inside this tentacle"));
        state.grant_oauth(
            "octopus",
            "evolve:field-mini-task",
            default_permissions("octopus"),
        );
        let plan = plan_tentacle_evolution_apply(&proposal, &state, "field_pack_tasks").unwrap();
        let workspace =
            std::env::temp_dir().join(format!("octopus-field-pack-apply-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        let artifact = write_tentacle_apply_artifacts(&workspace, &plan).unwrap();
        let patch = fs::read_to_string(artifact.patch_path.as_ref().unwrap()).unwrap();

        assert!(patch.contains("b/field-packs/math/field-pack.json"));
        assert!(patch.contains("b/field-packs/search/field-pack.json"));
        assert!(patch.contains("b/field-packs/index.json"));
        assert!(!patch.contains("Authorized Evolution Candidate"));
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn llm_field_pack_prompt_scopes_named_fields_only() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let state = HarnessState::default();
        let mut fake = EvolutionFakeChat {
            response: r#"{
              "summary": "extend named peer field pack tasks",
              "candidates": [
                {
                  "surface_id": "field_pack_tasks",
                  "title": "Add write and translate tasks",
                  "target": "field-packs/*/field-pack.json",
                  "rationale": "the objective names write and translate only",
                  "change_plan": ["edit only the named field packs"],
                  "checks": ["octopus fields summary"],
                  "suggested_patch": "diff --git a/field-packs/write/field-pack.json b/field-packs/write/field-pack.json\n--- a/field-packs/write/field-pack.json\n+++ b/field-packs/write/field-pack.json\n@@ -1,2 +1,3 @@\n {\n+  \"llm_patch_probe\": true,\ndiff --git a/field-packs/translate/field-pack.json b/field-packs/translate/field-pack.json\n--- a/field-packs/translate/field-pack.json\n+++ b/field-packs/translate/field-pack.json\n@@ -1,2 +1,3 @@\n {\n+  \"llm_patch_probe\": true,\n"
                }
              ]
            }"#
            .to_string(),
            responses: Vec::new(),
            prompt: String::new(),
        };

        let proposal = propose_tentacle_evolution_with_client(
            &root,
            "field-mini-task",
            "add the next harder mini task layer for write and 翻译",
            &state,
            &mut fake,
        )
        .unwrap();
        let candidate = proposal
            .patch_candidates
            .iter()
            .find(|candidate| candidate.surface_id == "field_pack_tasks")
            .unwrap();

        assert!(fake.prompt.contains("field-packs/write/field-pack.json"));
        assert!(fake
            .prompt
            .contains("field-packs/translate/field-pack.json"));
        assert!(!fake.prompt.contains("field-packs/math/field-pack.json"));
        assert!(candidate
            .target_files
            .iter()
            .any(|path| path.ends_with("field-packs/write/field-pack.json")));
        assert!(candidate
            .target_files
            .iter()
            .any(|path| path.ends_with("field-packs/translate/field-pack.json")));
        assert!(candidate
            .target_files
            .iter()
            .any(|path| path.ends_with("repair-templates/write/write-mini-1.pyfrag")));
        assert!(candidate
            .target_files
            .iter()
            .any(|path| path.ends_with("repair-templates/translate/translate-mini-1.pyfrag")));
        assert!(!candidate
            .target_files
            .iter()
            .any(|path| path.ends_with("field-packs/math/field-pack.json")));
        assert!(!candidate
            .target_files
            .iter()
            .any(|path| path.ends_with("field-packs/code/field-pack.json")));
        assert!(!candidate
            .target_files
            .iter()
            .any(|path| path.ends_with("repair-templates/math/math-mini-1.pyfrag")));
    }

    #[test]
    fn llm_evolution_assigns_unique_ids_for_same_surface_candidates() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let state = HarnessState::default();
        let mut fake = EvolutionFakeChat {
            response: r#"{
              "summary": "two runtime candidates should stay distinct",
              "candidates": [
                {
                  "surface_id": "runtime_code",
                  "title": "Patch read adapter",
                  "target": "tools/read.sh",
                  "rationale": "read evidence needs a narrow runtime change",
                  "change_plan": ["patch read.sh"],
                  "checks": ["tentacles/swe-agent/tools/read.sh README.md 1 2"],
                  "suggested_patch": "diff --git a/tentacles/swe-agent/tools/read.sh b/tentacles/swe-agent/tools/read.sh\n--- a/tentacles/swe-agent/tools/read.sh\n+++ b/tentacles/swe-agent/tools/read.sh\n@@ -1,2 +1,3 @@\n #!/bin/sh\n+# first candidate\n set -eu\n"
                },
                {
                  "surface_id": "runtime_code",
                  "title": "Patch edit adapter",
                  "target": "tools/edit.sh",
                  "rationale": "edit evidence needs a separate runtime change",
                  "change_plan": ["patch edit.sh"],
                  "checks": ["tentacles/swe-agent/tools/edit.sh README.md"],
                  "suggested_patch": "diff --git a/tentacles/swe-agent/tools/edit.sh b/tentacles/swe-agent/tools/edit.sh\n--- a/tentacles/swe-agent/tools/edit.sh\n+++ b/tentacles/swe-agent/tools/edit.sh\n@@ -1,2 +1,3 @@\n #!/bin/sh\n+# second candidate\n set -eu\n"
                }
              ]
            }"#
            .to_string(),
            responses: Vec::new(),
            prompt: String::new(),
        };

        let proposal = propose_tentacle_evolution_with_client(
            &root,
            "swe-agent",
            "improve observe feed",
            &state,
            &mut fake,
        )
        .unwrap();

        assert_eq!(proposal.patch_candidates[0].id, "03-runtime-code");
        assert_eq!(proposal.patch_candidates[1].id, "03-runtime-code-2");
        assert_eq!(
            proposal.patch_candidates[0].draft.path,
            "patches/03-runtime-code.patch.md"
        );
        assert_eq!(
            proposal.patch_candidates[1].draft.path,
            "patches/03-runtime-code-2.patch.md"
        );
    }

    #[test]
    fn field_pack_layer_objective_rejects_runtime_only_llm_candidate() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let state = HarnessState::default();
        let mut fake = EvolutionFakeChat {
            response: r#"{
              "summary": "incorrectly tries to harden existing runtime only",
              "candidates": [
                {
                  "surface_id": "runtime_code",
                  "title": "Harden write-mini-2",
                  "target": "repair-templates/write/write-mini-2.pyfrag",
                  "rationale": "existing task checks could be stricter",
                  "change_plan": ["patch write-mini-2 only"],
                  "checks": ["python3 tools/check_repair_templates.py"],
                  "suggested_patch": "diff --git a/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag b/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag\n--- a/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag\n+++ b/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag\n@@ -1,2 +1,3 @@\n if field == \"write\" and mini_task == \"write-mini-2\":\n+    checks[\"stricter\"] = True\n     pass\n"
                }
              ]
            }"#
            .to_string(),
            responses: Vec::new(),
            prompt: String::new(),
        };

        let error = propose_tentacle_evolution_with_client(
            &root,
            "field-mini-task",
            "add a harder mini task layer to write and translate field packs",
            &state,
            &mut fake,
        )
        .unwrap_err();

        assert!(error.contains("field_pack_tasks"));
    }

    #[test]
    fn field_pack_layer_objective_retries_llm_after_surface_rejection() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let state = HarnessState::default();
        let runtime_only = r#"{
          "summary": "incorrectly tries runtime only",
          "candidates": [
            {
              "surface_id": "runtime_code",
              "title": "Harden write-mini-2",
              "target": "repair-templates/write/write-mini-2.pyfrag",
              "rationale": "existing task checks could be stricter",
              "change_plan": ["patch write-mini-2 only"],
              "checks": ["python3 tools/check_repair_templates.py"],
              "suggested_patch": "diff --git a/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag b/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag\n--- a/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag\n+++ b/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag\n@@ -1,2 +1,3 @@\n if field == \"write\" and mini_task == \"write-mini-2\":\n+    checks[\"stricter\"] = True\n     pass\n"
            }
          ]
        }"#;
        let corrected = r#"{
          "summary": "adds write and translate third-layer field-pack tasks",
          "candidates": [
            {
              "surface_id": "field_pack_tasks",
              "title": "Add write and translate third mini tasks",
              "target": "field-packs/*/field-pack.json",
              "rationale": "harder layers belong in field-pack task definitions and matching repair templates",
              "change_plan": ["add write-mini-3 and translate-mini-3", "add matching repair templates"],
              "checks": ["octopus check field-mini-task 2"],
              "suggested_patch": "diff --git a/field-packs/write/field-pack.json b/field-packs/write/field-pack.json\n--- a/field-packs/write/field-pack.json\n+++ b/field-packs/write/field-pack.json\n@@ -1,2 +1,3 @@\n {\n+  \"write-mini-3-probe\": true,\ndiff --git a/field-packs/translate/field-pack.json b/field-packs/translate/field-pack.json\n--- a/field-packs/translate/field-pack.json\n+++ b/field-packs/translate/field-pack.json\n@@ -1,2 +1,3 @@\n {\n+  \"translate-mini-3-probe\": true,\ndiff --git a/tentacles/field-mini-task/repair-templates/write/write-mini-3.pyfrag b/tentacles/field-mini-task/repair-templates/write/write-mini-3.pyfrag\n--- /dev/null\n+++ b/tentacles/field-mini-task/repair-templates/write/write-mini-3.pyfrag\n@@ -0,0 +1,2 @@\n+if field == \"write\" and mini_task == \"write-mini-3\":\n+    pass\ndiff --git a/tentacles/field-mini-task/repair-templates/translate/translate-mini-3.pyfrag b/tentacles/field-mini-task/repair-templates/translate/translate-mini-3.pyfrag\n--- /dev/null\n+++ b/tentacles/field-mini-task/repair-templates/translate/translate-mini-3.pyfrag\n@@ -0,0 +1,2 @@\n+if field == \"translate\" and mini_task == \"translate-mini-3\":\n+    pass\n"
            }
          ]
        }"#;
        let mut fake = EvolutionFakeChat {
            response: String::new(),
            responses: vec![runtime_only.to_string(), corrected.to_string()],
            prompt: String::new(),
        };

        let proposal = propose_tentacle_evolution_with_client(
            &root,
            "field-mini-task",
            "add a harder mini task layer to write and translate field packs",
            &state,
            &mut fake,
        )
        .unwrap();

        assert!(fake.prompt.contains("previous candidate set was rejected"));
        assert_eq!(proposal.patch_candidates.len(), 1);
        assert_eq!(proposal.patch_candidates[0].surface_id, "field_pack_tasks");
        assert!(proposal.patch_candidates[0]
            .target_files
            .iter()
            .any(|path| path.ends_with("field-packs/write/field-pack.json")));
        assert!(proposal.patch_candidates[0]
            .target_files
            .iter()
            .any(|path| path.ends_with("field-packs/translate/field-pack.json")));
        assert!(proposal.patch_candidates[0]
            .target_files
            .iter()
            .any(|path| path.ends_with("repair-templates/write/write-mini-3.pyfrag")));
        assert!(proposal.patch_candidates[0]
            .target_files
            .iter()
            .any(|path| path.ends_with("repair-templates/translate/translate-mini-3.pyfrag")));
    }

    #[test]
    fn apply_artifact_accepts_collapsed_duplicate_tentacle_target_path() {
        let workspace =
            std::env::temp_dir().join(format!("octopus-duplicate-target-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        let plan = EvolutionApplyPlan {
            tentacle_id: "field-mini-task".to_string(),
            candidate_id: "01-brain-prompt".to_string(),
            objective: "tighten field harness prompt".to_string(),
            authorized: true,
            status: "ready_for_authorized_patch".to_string(),
            required_grant: "octopus:evolve:field-mini-task".to_string(),
            active_grant: Some("octopus:evolve:field-mini-task".to_string()),
            target: "/tmp/tentacles/field-mini-task/tentacles/field-mini-task/manifest.json"
                .to_string(),
            target_files: vec![],
            draft_path: "patches/01-brain-prompt.patch.md".to_string(),
            checks: vec![],
            feedback: vec![],
            suggested_patch: Some(
                "diff --git a/tentacles/field-mini-task/manifest.json b/tentacles/field-mini-task/manifest.json\n--- a/tentacles/field-mini-task/manifest.json\n+++ b/tentacles/field-mini-task/manifest.json\n@@ -1 +1 @@\n-old\n+new\n"
                    .to_string(),
            ),
            guardrails: vec![],
            next_steps: vec![],
        };

        let artifact = write_tentacle_apply_artifacts(&workspace, &plan).unwrap();
        let patch = fs::read_to_string(artifact.patch_path.as_ref().unwrap()).unwrap();

        assert!(patch.contains("b/tentacles/field-mini-task/manifest.json"));
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn apply_artifact_allows_new_matching_field_repair_template() {
        let mut plan = EvolutionApplyPlan {
            tentacle_id: "field-mini-task".to_string(),
            candidate_id: "04-field-pack-tasks".to_string(),
            objective: "add write mini task".to_string(),
            authorized: true,
            status: "ready_for_authorized_patch".to_string(),
            required_grant: "octopus:evolve:field-mini-task".to_string(),
            active_grant: Some("octopus:evolve:field-mini-task".to_string()),
            target: "field-packs/write/field-pack.json".to_string(),
            target_files: vec!["field-packs/write/field-pack.json".to_string()],
            draft_path: "patches/04-field-pack-tasks.patch.md".to_string(),
            checks: vec![],
            feedback: vec![],
            suggested_patch: Some(
                "diff --git a/field-packs/write/field-pack.json b/field-packs/write/field-pack.json\n--- a/field-packs/write/field-pack.json\n+++ b/field-packs/write/field-pack.json\n@@ -1,2 +1,3 @@\n {\n+  \"llm_patch_probe\": true,\ndiff --git a/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag b/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag\n--- /dev/null\n+++ b/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag\n@@ -0,0 +1,2 @@\n+if field == \"write\":\n+    pass\n"
                    .to_string(),
            ),
            guardrails: vec![],
            next_steps: vec![],
        };

        assert!(evolution_patch::authorized_apply_patch(&plan).is_some());
        plan.suggested_patch = Some(
            "diff --git a/field-packs/write/field-pack.json b/field-packs/write/field-pack.json\n--- a/field-packs/write/field-pack.json\n+++ b/field-packs/write/field-pack.json\n@@ -1,2 +1,3 @@\n {\n+  \"llm_patch_probe\": true,\ndiff --git a/tentacles/field-mini-task/repair-templates/math/math-mini-9.pyfrag b/tentacles/field-mini-task/repair-templates/math/math-mini-9.pyfrag\n--- /dev/null\n+++ b/tentacles/field-mini-task/repair-templates/math/math-mini-9.pyfrag\n@@ -0,0 +1,2 @@\n+if field == \"math\":\n+    pass\n"
                .to_string(),
        );
        assert!(evolution_patch::authorized_apply_patch(&plan).is_none());
    }

    #[test]
    fn clean_suggested_patch_normalizes_bare_new_file_hunk_lines() {
        let patch = evolution_patch::clean_suggested_patch(Some(
            "diff --git a/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag b/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag\nnew file mode 100644\n--- /dev/null\n+++ b/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag\n@@ -0,0 +1,2 @@\nif field == \"write\":\n    pass\n+diff --git a/tentacles/field-mini-task/repair-templates/translate/translate-mini-2.pyfrag b/tentacles/field-mini-task/repair-templates/translate/translate-mini-2.pyfrag\nnew file mode 100644\n--- /dev/null\n+++ b/tentacles/field-mini-task/repair-templates/translate/translate-mini-2.pyfrag\n@@ -0,0 +1 @@\nif field == \"translate\":\n"
                .to_string(),
        ))
        .unwrap();

        assert!(patch.contains("+if field == \"write\":"));
        assert!(patch.contains("+    pass"));
        assert!(patch.contains("\ndiff --git a/tentacles/field-mini-task/repair-templates/translate/translate-mini-2.pyfrag"));
        assert!(!patch.contains("\n+diff --git"));
    }

    #[test]
    fn clean_suggested_patch_adds_missing_new_file_mode() {
        if !command_available("git") {
            return;
        }
        let workspace =
            std::env::temp_dir().join(format!("octopus-new-file-mode-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        fs::create_dir_all(&workspace).unwrap();
        let patch = evolution_patch::clean_suggested_patch(Some(
            "diff --git a/new.txt b/new.txt\n--- /dev/null\n+++ b/new.txt\n@@ -0,0 +1 @@\nhello\n"
                .to_string(),
        ))
        .unwrap();
        let patch_path = workspace.join("patch.diff");
        fs::write(&patch_path, &patch).unwrap();
        let output = Command::new("git")
            .arg("apply")
            .arg("--check")
            .arg("--recount")
            .arg("--unidiff-zero")
            .arg(&patch_path)
            .current_dir(&workspace)
            .output()
            .unwrap();

        assert!(patch.contains("new file mode 100644\n--- /dev/null"));
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn clean_suggested_patch_normalizes_indented_diff_file_headers() {
        if !command_available("git") {
            return;
        }
        let workspace =
            std::env::temp_dir().join(format!("octopus-indented-diff-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        fs::create_dir_all(&workspace).unwrap();
        fs::write(workspace.join("a.txt"), "one\nthree\n").unwrap();

        let patch = evolution_patch::clean_suggested_patch(Some(
            "diff --git a/a.txt b/a.txt\n--- a/a.txt\n+++ b/a.txt\n@@ -1,2 +1,3 @@\n one\n+two\n three\n diff --git a/new.txt b/new.txt\nnew file mode 100644\n--- /dev/null\n+++ b/new.txt\n@@ -0,0 +1,2 @@\nhello\nworld\n"
                .to_string(),
        ))
        .unwrap();
        let patch_path = workspace.join("patch.diff");
        fs::write(&patch_path, &patch).unwrap();
        let output = Command::new("git")
            .arg("apply")
            .arg("--check")
            .arg("--recount")
            .arg("--unidiff-zero")
            .arg(&patch_path)
            .current_dir(&workspace)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(patch.contains("\ndiff --git a/new.txt b/new.txt"));
        assert!(!patch.contains("\n diff --git"));
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn runtime_candidate_recovers_target_files_from_provider_diff() {
        let surface = EvolutionSurface {
            id: "runtime_code".to_string(),
            description: "runtime".to_string(),
            targets: vec!["repair-templates/*/*.pyfrag".to_string()],
        };
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles/field-mini-task");
        let patch = "diff --git a/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag b/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag\n--- a/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag\n+++ b/tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag\n@@ -1 +1 @@\n-old\n+new\ndiff --git a/crates/octopus-core/src/lib.rs b/crates/octopus-core/src/lib.rs\n--- a/crates/octopus-core/src/lib.rs\n+++ b/crates/octopus-core/src/lib.rs\n@@ -1 +1 @@\n-old\n+new\n";

        let files =
            evolution_candidate::diff_target_files_for_surface(&manifest_dir, &surface, patch);

        assert_eq!(
            files,
            vec!["tentacles/field-mini-task/repair-templates/write/write-mini-2.pyfrag"]
        );
    }

    #[test]
    fn llm_evolution_tolerates_duplicate_json_fields() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let state = HarnessState::default();
        let mut fake = EvolutionFakeChat {
            response: r#"{
              "summary": "duplicate keys should not break planning",
              "candidates": [
                {
                  "surface_id": "brain_prompt",
                  "surface_id": "runtime_code",
                  "title": "Runtime patch",
                  "target": "tools/read.sh",
                  "rationale": "tool-side Feed repair",
                  "change_plan": ["patch runtime"],
                  "checks": ["tentacles/swe-agent/tools/read.sh README.md 1 2"]
                }
              ]
            }"#
            .to_string(),
            responses: Vec::new(),
            prompt: String::new(),
        };

        let proposal = propose_tentacle_evolution_with_client(
            &root,
            "swe-agent",
            "repair feed",
            &state,
            &mut fake,
        )
        .unwrap();

        assert_eq!(proposal.generator, "llm");
        assert_eq!(proposal.patch_candidates[0].surface_id, "runtime_code");
    }

    #[test]
    fn manifest_tentacle_runs_python_json_contract() {
        if !command_available("python3") {
            return;
        }
        let dir =
            std::env::temp_dir().join(format!("octopus-python-contract-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let script = dir.join("tool.py");
        fs::write(
            &script,
            r#"import json, sys
payload = json.load(sys.stdin)
print(json.dumps({
    "status": "satisfied",
    "output": f"{payload['tool']['runtime']}:{payload['need']['kind']}:{payload['need']['query']}",
    "metadata": {
        "schema": payload["schema_version"],
        "tentacle": payload["tentacle"]["id"]
    }
}))
"#,
        )
        .unwrap();

        struct FakeChat;

        impl ChatClient for FakeChat {
            fn chat(&mut self, _messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                Ok(ChatResponse {
                    content: r#"{"calls":[{"tool":"json_probe","reason":"inspect via json runtime"}],"summary":"planned json runtime"}"#.to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let factory: ChatClientFactory = Arc::new(|| Ok(Box::new(FakeChat)));
        let mut tentacle = ManifestTentacle::new_with_llm_and_grants(
            InstalledTentacle {
                id: "python-json".to_string(),
                name: "Python JSON".to_string(),
                source: dir.join("manifest.json").to_string_lossy().to_string(),
                brain_kind: "llm".to_string(),
                brain_prompt: "Use the JSON call envelope to feed the clean brain.".to_string(),
                feedback_contract: Some("Return compact structured feedback.".to_string()),
                runtime_kinds: vec!["python".to_string()],
                needs: vec!["observe".to_string()],
                tools: vec![format!("json_probe:python:{}", script.to_string_lossy())],
                tool_meta: vec![InstalledTool {
                    id: "json_probe".to_string(),
                    description: "Inspect a Need through a JSON runtime contract.".to_string(),
                    input: "octopus-json-v1 tool call".to_string(),
                    output: "structured feedback".to_string(),
                    kind: "python".to_string(),
                    entrypoint: script.to_string_lossy().to_string(),
                    contract: Some(OCTOPUS_JSON_CONTRACT.to_string()),
                    permission: None,
                }],
                editable: vec!["tools/tool.py".to_string()],
                evolution_surfaces: vec!["runtime_code".to_string()],
            },
            Some(factory),
            vec![],
        );

        let feed = tentacle.feed(&Need::new(NeedKind::Observe, "README.md"));

        assert_eq!(feed.status, Status::Satisfied);
        assert_eq!(feed.summary, "python:observe:README.md");
        assert_eq!(
            feed.metadata.get("contract").map(String::as_str),
            Some(OCTOPUS_JSON_CONTRACT)
        );
        assert_eq!(
            feed.metadata.get("schema").map(String::as_str),
            Some(OCTOPUS_TOOL_CALL_SCHEMA)
        );
        assert_eq!(
            feed.metadata.get("runtime").map(String::as_str),
            Some("python")
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn manifest_tool_permission_blocks_until_granted() {
        if command_available("python3") {
            let dir = std::env::temp_dir()
                .join(format!("octopus-tool-permission-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(dir.join("tools")).unwrap();
            let script = dir.join("tools").join("guarded.py");
            fs::write(
                &script,
                "import sys\nprint(f\"guarded:{sys.argv[1] if len(sys.argv) > 1 else ''}\")\n",
            )
            .unwrap();

            let permission = ToolPermission {
                provider: "octopus".to_string(),
                scope: "tool:guarded".to_string(),
                permissions: vec!["tool:execute".to_string()],
                reason: Some("test execution".to_string()),
            };
            let installed = InstalledTentacle {
                id: "guarded".to_string(),
                name: "Guarded".to_string(),
                source: dir.join("manifest.json").to_string_lossy().to_string(),
                brain_kind: "llm".to_string(),
                brain_prompt: "Execute only after permission.".to_string(),
                feedback_contract: Some("Return guarded output.".to_string()),
                runtime_kinds: vec!["python".to_string()],
                needs: vec!["execute".to_string()],
                tools: vec!["guarded:python:tools/guarded.py".to_string()],
                tool_meta: vec![InstalledTool {
                    id: "guarded".to_string(),
                    description: "Guarded local execution.".to_string(),
                    input: "query arg".to_string(),
                    output: "guarded output".to_string(),
                    kind: "python".to_string(),
                    entrypoint: "tools/guarded.py".to_string(),
                    contract: None,
                    permission: Some(permission.clone()),
                }],
                editable: vec!["tools/*".to_string()],
                evolution_surfaces: vec!["tool_meta".to_string(), "runtime_code".to_string()],
            };

            struct GuardedChat;

            impl ChatClient for GuardedChat {
                fn chat(&mut self, _messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                    Ok(ChatResponse {
                        content: r#"{"calls":[{"tool":"guarded","reason":"permission gate test"}],"summary":"guarded"}"#.to_string(),
                        metadata: BTreeMap::new(),
                    })
                }
            }

            let factory: ChatClientFactory = Arc::new(|| Ok(Box::new(GuardedChat)));
            let mut blocked = ManifestTentacle::new_with_llm_and_grants(
                installed.clone(),
                Some(factory.clone()),
                vec![],
            );
            let blocked_feed = blocked.feed(&Need::new(NeedKind::Execute, "ok"));
            assert_eq!(blocked_feed.status, Status::Failed);
            assert!(blocked_feed.summary.contains("needs_authorization"));
            assert_eq!(
                blocked_feed
                    .metadata
                    .get("authorization_required")
                    .map(String::as_str),
                Some("true")
            );

            let grant = CapabilityGrant {
                id: "octopus:tool:guarded".to_string(),
                provider: "octopus".to_string(),
                scope: "tool:guarded".to_string(),
                permissions: vec!["tool:execute".to_string()],
                status: GrantStatus::Active,
            };
            let mut allowed = ManifestTentacle::new_with_llm_and_grants(
                installed,
                Some(factory),
                vec![grant.clone()],
            );
            let allowed_feed = allowed.feed(&Need::new(NeedKind::Execute, "ok"));

            assert_eq!(allowed_feed.status, Status::Satisfied);
            assert!(allowed_feed.summary.contains("guarded:ok"));
            assert_eq!(
                allowed_feed
                    .metadata
                    .get("active_grant")
                    .map(String::as_str),
                Some(grant.id.as_str())
            );
            let _ = fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn builtin_json_feed_manifest_runs_python_contract() {
        if !command_available("python3") {
            return;
        }
        let repo = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
        let mut state = HarnessState::default();
        let installed = state
            .install_manifest(repo.join("tentacles"), "json-feed")
            .unwrap();
        assert!(installed.runtime_kinds.contains(&"python".to_string()));
        assert!(installed
            .tool_meta
            .iter()
            .any(|tool| tool.contract.as_deref() == Some(OCTOPUS_JSON_CONTRACT)));
        struct JsonFeedChat;

        impl ChatClient for JsonFeedChat {
            fn chat(&mut self, _messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                Ok(ChatResponse {
                    content: r#"{"calls":[{"tool":"feed","reason":"run json feed runtime"}],"summary":"planned json feed"}"#.to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let factory: ChatClientFactory = Arc::new(|| Ok(Box::new(JsonFeedChat)));
        let mut harness = Harness::with_state_and_manifest_llm_factory(state, factory);

        let feedback = harness.feed(&[Need::new(NeedKind::Observe, "Cargo.toml")]);

        assert_eq!(feedback.status, Status::Satisfied, "{}", feedback.summary);
        assert!(feedback
            .summary
            .contains("json-feed observe: file Cargo.toml"));
        assert_eq!(
            feedback.feeds[0]
                .metadata
                .get("runtime")
                .map(String::as_str),
            Some("python")
        );
        assert_eq!(
            feedback.feeds[0]
                .metadata
                .get("contract")
                .map(String::as_str),
            Some(OCTOPUS_JSON_CONTRACT)
        );
        assert_eq!(
            feedback.feeds[0].metadata.get("schema").map(String::as_str),
            Some(OCTOPUS_TOOL_CALL_SCHEMA)
        );
    }

    #[test]
    fn scaffolded_python_tentacle_installs_and_feeds_need() {
        if !command_available("python3") {
            return;
        }
        let workspace =
            std::env::temp_dir().join(format!("octopus-scaffold-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        fs::create_dir_all(&workspace).unwrap();

        let scaffold = scaffold_tentacle(&workspace, "custom_feed", Some("python")).unwrap();
        let tentacles_root = workspace.join("tentacles");
        let reports = inspect_tentacle_manifests(&tentacles_root).unwrap();

        assert_eq!(scaffold.runtime, "python");
        assert!(Path::new(&scaffold.manifest_path).exists());
        assert!(Path::new(scaffold.tool_path.as_ref().unwrap()).exists());
        assert!(tentacles_root.join("tentacle.schema.json").exists());
        assert_eq!(reports[0].id, "custom_feed");
        assert!(reports[0].missing_entrypoints.is_empty());
        assert!(reports[0]
            .evolution_surfaces
            .contains(&"runtime_code".to_string()));

        let mut state = HarnessState::default();
        state
            .install_manifest(&tentacles_root, "custom_feed")
            .unwrap();
        struct ScaffoldChat;

        impl ChatClient for ScaffoldChat {
            fn chat(&mut self, _messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                Ok(ChatResponse {
                    content: r#"{"calls":[{"tool":"feed","reason":"run scaffolded feed runtime"}],"summary":"planned scaffolded feed"}"#.to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let factory: ChatClientFactory = Arc::new(|| Ok(Box::new(ScaffoldChat)));
        let mut harness = Harness::with_state_and_manifest_llm_factory(state, factory);
        let feed = harness.feed_one(&Need::new(NeedKind::Observe, "README.md"));

        assert_eq!(feed.status, Status::Satisfied);
        assert_eq!(feed.summary, "observe feed for README.md");
        assert_eq!(
            feed.metadata.get("runtime").map(String::as_str),
            Some("python")
        );
        assert_eq!(
            feed.metadata.get("contract").map(String::as_str),
            Some(OCTOPUS_JSON_CONTRACT)
        );
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn scaffold_accepts_custom_runtime_manifest_only() {
        let workspace =
            std::env::temp_dir().join(format!("octopus-scaffold-custom-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        fs::create_dir_all(&workspace).unwrap();

        let scaffold = scaffold_tentacle(&workspace, "native_feed", Some("rust")).unwrap();
        let tentacles_root = workspace.join("tentacles");
        let reports = inspect_tentacle_manifests(&tentacles_root).unwrap();

        assert_eq!(scaffold.runtime, "rust");
        assert!(Path::new(&scaffold.manifest_path).exists());
        assert!(scaffold.tool_path.is_none());
        assert!(scaffold
            .next_steps
            .iter()
            .any(|step| step.contains("add executable")));
        assert_eq!(reports[0].runtime_kinds, vec!["rust".to_string()]);
        assert!(reports[0]
            .evolution_surfaces
            .contains(&"tool_meta".to_string()));
        assert_eq!(
            reports[0].missing_entrypoints,
            vec!["tools/feed".to_string()]
        );
        let mut state = HarnessState::default();
        assert!(state
            .install_manifest(&tentacles_root, "native_feed")
            .is_err());
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn environment_report_detects_manifests() {
        let dir = std::env::temp_dir().join(format!("octopus-env-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("docs")).unwrap();
        fs::write(dir.join("Cargo.toml"), "[workspace]\n").unwrap();

        let report = EnvironmentReport::detect(&dir);

        assert!(report.manifests.contains(&"rust".to_string()));
        assert!(report.manifests.contains(&"docs".to_string()));
        assert!(report.recommended_profiles.contains(&"code".to_string()));
        if command_available("python3") {
            assert!(report
                .recommended_profiles
                .contains(&"field-mini-task".to_string()));
        }
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn harness_state_adapts_environment_into_installed_tentacles() {
        let dir = std::env::temp_dir().join(format!("octopus-adapt-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".git")).unwrap();
        fs::create_dir_all(dir.join("docs")).unwrap();
        fs::write(dir.join("Cargo.toml"), "[workspace]\n").unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();

        let report = state.adapt_environment(&dir, &root);

        assert!(report
            .environment
            .recommended_profiles
            .contains(&"swe-agent".to_string()));
        assert!(report.installed_profiles.contains(&"swe-agent".to_string()));
        assert!(state
            .installed_tentacles
            .iter()
            .any(|tentacle| tentacle.id == "swe-agent"));
        assert!(state
            .installed_tentacles
            .iter()
            .any(|tentacle| tentacle.id == "repo-maintainer"));
        if command_available("python3") {
            assert!(state
                .installed_tentacles
                .iter()
                .any(|tentacle| tentacle.id == "field-mini-task"));
        }
        let second = state.adapt_environment(&dir, &root);
        assert!(second.installed_profiles.is_empty());
        assert!(second.installed_tentacles.is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn harness_state_installs_profiles_idempotently() {
        let mut state = HarnessState::default();

        state.install_profile("research").unwrap();
        state.install_profile("research").unwrap();

        assert_eq!(state.installed_profiles, vec!["research".to_string()]);
        assert!(state.install_profile("missing").is_err());
    }

    #[test]
    fn harness_state_installs_manifest_tentacle_package() {
        let mut state = HarnessState::default();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");

        let installed = state.install_manifest(&root, "swe-agent").unwrap();
        state.install_manifest(&root, "swe-agent").unwrap();

        assert_eq!(state.installed_tentacles.len(), 1);
        assert_eq!(installed.brain_kind, "llm");
        assert!(installed.brain_prompt.contains("cognitive Need"));
        assert!(installed.runtime_kinds.contains(&"shell".to_string()));
        assert!(installed
            .tools
            .iter()
            .any(|tool| tool.contains("read:shell")));
        assert!(installed
            .tool_meta
            .iter()
            .any(|tool| tool.id == "inspect_repo"
                && tool.description.contains("Summarize repo state")));
        assert!(installed
            .tool_meta
            .iter()
            .any(|tool| tool.id == "inspect_repo"
                && tool.contract.as_deref() == Some(OCTOPUS_STDIO_ARGV_CONTRACT)));
        assert!(installed
            .evolution_surfaces
            .contains(&"runtime_code".to_string()));
        assert!(state.install_manifest(&root, "missing").is_err());
    }

    #[test]
    fn harness_repair_manifest_installs_and_feeds_diagnostics() {
        struct HarnessRepairChat;

        impl ChatClient for HarnessRepairChat {
            fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                let prompt = messages
                    .iter()
                    .map(|message| message.content.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                let tool = if prompt.contains("Need: observe:") {
                    "diagnose_harness"
                } else if prompt.contains("repair outcome") {
                    "repair_outcome"
                } else if prompt.contains("Need: execute:") {
                    "repair_session"
                } else if prompt.contains("Need: verify:") {
                    "heartbeat_repair"
                } else {
                    "diagnose_harness"
                };
                Ok(ChatResponse {
                    content: format!(
                        r#"{{"calls":[{{"tool":"{tool}","reason":"repair test planner"}}],"summary":"planned harness repair"}}"#
                    ),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let mut state = HarnessState::default();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let installed = state
            .install_manifest(&root, "harness-repair-agent")
            .unwrap();
        assert_eq!(installed.brain_kind, "llm");
        assert!(installed.brain_prompt.contains("harness state"));
        assert!(installed.runtime_kinds.contains(&"shell".to_string()));
        assert!(installed.tool_meta.iter().any(|tool| {
            tool.id == "diagnose_harness" && tool.contract.as_deref() == Some(OCTOPUS_JSON_CONTRACT)
        }));
        assert!(installed
            .tool_meta
            .iter()
            .any(|tool| tool.id == "repair_outcome"
                && tool.contract.as_deref() == Some(OCTOPUS_JSON_CONTRACT)));

        let workspace =
            std::env::temp_dir().join(format!("octopus-repair-session-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        fs::create_dir_all(workspace.join(".octopus")).unwrap();
        fs::write(
            workspace.join(".octopus/state.json"),
            r#"{"installed_tentacles":[{"id":"harness-repair-agent"}],"feed_traces":[],"check_history":[]}"#,
        )
        .unwrap();

        let factory: ChatClientFactory = Arc::new(|| Ok(Box::new(HarnessRepairChat)));
        let mut tentacle =
            ManifestTentacle::new_with_llm_and_grants(installed, Some(factory.clone()), vec![]);
        let feed = tentacle.feed(&Need::new(
            NeedKind::Observe,
            workspace.to_string_lossy().to_string(),
        ));

        assert_eq!(feed.status, Status::Satisfied);
        assert!(feed.summary.contains("harness diagnosis"));
        assert_eq!(
            feed.metadata.get("tentacle_brain").map(String::as_str),
            Some("harness-repair-agent")
        );
        assert_eq!(
            feed.metadata.get("contract").map(String::as_str),
            Some(OCTOPUS_JSON_CONTRACT)
        );
        assert!(feed
            .evidence
            .iter()
            .any(|evidence| evidence.source == "diagnose_harness"));

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let installed = state
            .install_manifest(&root, "harness-repair-agent")
            .unwrap();
        let mut tentacle =
            ManifestTentacle::new_with_llm_and_grants(installed, Some(factory), vec![]);
        let old_repair_llm = std::env::var("OCTOPUS_REPAIR_LLM").ok();
        std::env::remove_var("OCTOPUS_REPAIR_LLM");
        let feed = tentacle.feed(&Need::new(
            NeedKind::Execute,
            workspace.to_string_lossy().to_string(),
        ));

        assert_eq!(feed.status, Status::Satisfied);
        assert!(feed.summary.contains("harness repair session"));
        assert_eq!(
            feed.metadata.get("tool").map(String::as_str),
            Some("repair_session")
        );
        assert!(workspace.join(".octopus/harness-repair").exists());
        for key in [
            "prompt",
            "draft",
            "review",
            "next_need_file",
            "command_script",
            "code_context",
            "repair_plan",
        ] {
            let path = feed
                .metadata
                .get(key)
                .map(|path| workspace.join(path))
                .expect("repair session metadata path");
            assert!(path.exists(), "{key} should exist at {}", path.display());
        }
        assert_eq!(
            feed.metadata.get("llm_draft_status").map(String::as_str),
            Some("disabled")
        );
        assert!(feed.metadata.contains_key("outcome_command"));
        assert_eq!(
            feed.metadata.get("next_need_kind").map(String::as_str),
            Some("verify")
        );
        assert_eq!(
            feed.metadata.get("code_context_tool").map(String::as_str),
            Some("repair_session")
        );
        let code_context = feed
            .metadata
            .get("code_context")
            .map(|path| workspace.join(path))
            .expect("code context metadata path");
        let code_context_text = fs::read_to_string(&code_context).unwrap();
        assert!(code_context_text.contains("Harness Repair Code Context"));
        assert!(code_context_text.contains("repair_session.sh"));
        let review = feed
            .metadata
            .get("review")
            .map(|path| workspace.join(path))
            .expect("review metadata path");
        let review_text = fs::read_to_string(&review).unwrap();
        assert!(review_text.contains("Harness Repair Review"));
        assert!(review_text.contains("harness:write"));
        assert_eq!(
            feed.metadata.get("repair_plan_status").map(String::as_str),
            Some("review_required")
        );
        let repair_plan = feed
            .metadata
            .get("repair_plan")
            .map(|path| workspace.join(path))
            .expect("repair plan metadata path");
        let repair_plan_text = fs::read_to_string(&repair_plan).unwrap();
        assert!(repair_plan_text.contains("octopus-harness-repair-plan-v1"));
        assert!(repair_plan_text.contains("\"target_tool\": \"repair_session\""));
        assert!(repair_plan_text.contains("\"checks\""));
        let heartbeat_feed = tentacle.feed(&Need::new(
            NeedKind::Verify,
            workspace.to_string_lossy().to_string(),
        ));
        assert_eq!(heartbeat_feed.status, Status::Satisfied);
        assert!(heartbeat_feed.summary.contains("repair_plan"));
        assert_eq!(
            heartbeat_feed.metadata.get("tool").map(String::as_str),
            Some("heartbeat_repair")
        );
        assert_eq!(
            heartbeat_feed
                .metadata
                .get("repair_plan_status")
                .map(String::as_str),
            Some("review_required")
        );
        assert_eq!(
            heartbeat_feed
                .metadata
                .get("target_tool")
                .map(String::as_str),
            Some("repair_session")
        );
        assert_eq!(
            heartbeat_feed
                .metadata
                .get("next_need_kind")
                .map(String::as_str),
            Some("verify")
        );
        let heartbeat_plan = heartbeat_feed
            .metadata
            .get("repair_plan")
            .map(|path| workspace.join(path))
            .expect("heartbeat repair plan metadata");
        assert_eq!(heartbeat_plan, repair_plan);
        assert!(heartbeat_feed
            .metadata
            .get("grant_command")
            .unwrap()
            .contains("harness:write"));
        assert_eq!(
            heartbeat_feed
                .metadata
                .get("review")
                .map(|path| workspace.join(path)),
            Some(review)
        );
        let session = feed
            .metadata
            .get("session")
            .expect("repair session path")
            .clone();
        let outcome_feed = tentacle.feed(&Need::new(
            NeedKind::Verify,
            format!(
                "{} repair outcome {} satisfied reviewed repair draft",
                workspace.to_string_lossy(),
                session
            ),
        ));
        assert_eq!(outcome_feed.status, Status::Satisfied);
        assert_eq!(
            outcome_feed.metadata.get("tool").map(String::as_str),
            Some("repair_outcome")
        );
        let outcomes_file = outcome_feed
            .metadata
            .get("outcomes_file")
            .map(|path| workspace.join(path))
            .expect("outcomes file metadata");
        assert!(outcomes_file.exists());
        assert!(fs::read_to_string(&outcomes_file)
            .unwrap()
            .contains("\"outcome_status\": \"satisfied\""));
        let outcome_markdown = outcome_feed
            .metadata
            .get("outcome")
            .map(|path| workspace.join(path))
            .expect("outcome markdown metadata");
        assert!(outcome_markdown.exists());
        let follow_up = tentacle.feed(&Need::new(
            NeedKind::Execute,
            workspace.to_string_lossy().to_string(),
        ));
        assert_eq!(follow_up.status, Status::Satisfied);
        let outcome_memory = follow_up
            .metadata
            .get("outcome_memory")
            .map(|path| workspace.join(path))
            .expect("outcome memory metadata");
        assert!(outcome_memory.exists());
        let memory = fs::read_to_string(&outcome_memory).unwrap();
        assert!(memory.contains("reviewed repair draft"));
        assert!(memory.contains("satisfied"));
        let prompt = follow_up
            .metadata
            .get("prompt")
            .map(|path| workspace.join(path))
            .expect("follow-up prompt metadata");
        assert!(fs::read_to_string(prompt)
            .unwrap()
            .contains("code context excerpt"));
        assert!(feed
            .evidence
            .iter()
            .any(|evidence| evidence.source == "harness-repair-agent/repair_session"));
        if let Some(value) = old_repair_llm {
            std::env::set_var("OCTOPUS_REPAIR_LLM", value);
        } else {
            std::env::remove_var("OCTOPUS_REPAIR_LLM");
        }
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn think_tentacle_requires_tool_side_llm_plan() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let need = Need::new(NeedKind::Observe, "current browser tab");

        let error = think_tentacle(&root, "computer-use-agent", &need, None, &[]).unwrap_err();

        assert!(error.contains("computer-use-agent cannot feed observe"));
    }

    #[cfg(unix)]
    #[test]
    fn think_tentacle_can_use_llm_plan_without_execution() {
        use std::os::unix::fs::PermissionsExt;

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let fake = std::env::temp_dir().join(format!("octopus-think-llm-{}", std::process::id()));
        let _ = fs::remove_dir_all(&fake);
        fs::create_dir_all(&fake).unwrap();
        let curl = fake.join("fake-curl.sh");
        fs::write(
            &curl,
            "#!/bin/sh\nprintf '%s' '{\"choices\":[{\"message\":{\"content\":\"{\\\"calls\\\":[{\\\"tool\\\":\\\"read\\\",\\\"reason\\\":\\\"inspect source without running tests\\\"}],\\\"summary\\\":\\\"planned by llm\\\"}\"}}]}'\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&curl).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&curl, permissions).unwrap();
        let config = OpenAiCompatibleConfig {
            model: "test-model".to_string(),
            api_key: None,
            base_url: "https://llm.example/v1".to_string(),
            timeout_seconds: 1,
            retry_count: 0,
            retry_delay_ms: 0,
            curl_command: curl.to_string_lossy().to_string(),
            tuning: OpenAiCompatibleTuning::default(),
        };

        let plan = think_tentacle(
            &root,
            "swe-agent",
            &Need::new(NeedKind::Observe, "Cargo.toml 1 1"),
            Some(config),
            &[],
        )
        .unwrap();

        assert_eq!(plan.plan_source, "llm");
        assert_eq!(plan.selected_tool.id, "read");
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0].tool.id, "read");
        assert!(plan.reason.contains("inspect source"));
        let _ = fs::remove_dir_all(fake);
    }

    #[test]
    fn think_tentacle_can_use_provider_factory_plan_without_execution() {
        struct FactoryChat;

        impl ChatClient for FactoryChat {
            fn chat(&mut self, _messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                Ok(ChatResponse {
                    content: r#"{"calls":[{"tool":"read","reason":"factory selected repo read"}],"summary":"planned by provider factory"}"#.to_string(),
                    metadata: BTreeMap::from([(
                        "provider".to_string(),
                        "factory-test".to_string(),
                    )]),
                })
            }
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let factory: ChatClientFactory = Arc::new(|| Ok(Box::new(FactoryChat)));

        let plan = think_tentacle_with_llm_factory(
            &root,
            "swe-agent",
            &Need::new(NeedKind::Observe, "Cargo.toml 1 1"),
            Some(factory),
            &[],
        )
        .unwrap();

        assert_eq!(plan.plan_source, "llm");
        assert_eq!(plan.selected_tool.id, "read");
        assert!(plan.reason.contains("factory selected repo read"));
    }

    #[test]
    fn installed_manifest_tentacle_without_llm_does_not_execute() {
        let repo = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
        let mut state = HarnessState::default();
        state
            .install_manifest(repo.join("tentacles"), "swe-agent")
            .unwrap();
        let mut harness = Harness::with_state(state);

        let feedback = harness.feed(&[Need::new(
            NeedKind::Observe,
            repo.to_string_lossy().to_string(),
        )]);

        assert_eq!(feedback.status, Status::Unsupported);
        assert!(feedback.summary.contains("no tentacle supports this need"));
        assert_eq!(feedback.feeds[0].metadata.get("tool"), None);
        assert_eq!(feedback.feeds[0].metadata.get("plan_source"), None);
        assert_eq!(harness.state.feed_traces.len(), 1);
        assert_eq!(harness.state.feed_traces[0].tool.as_deref(), None);
        assert_eq!(
            harness.state.routes.score(&NeedKind::Observe, "swe-agent"),
            1.0
        );
    }

    #[test]
    fn manifest_tentacle_supports_declared_need_with_llm_available() {
        struct FakeChat;

        impl ChatClient for FakeChat {
            fn chat(&mut self, _messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                Ok(ChatResponse {
                    content: r#"{"calls":[],"summary":"unused"}"#.to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();
        let installed = state.install_manifest(&root, "computer-use-agent").unwrap();
        let factory: ChatClientFactory = Arc::new(|| Ok(Box::new(FakeChat)));
        let tentacle =
            ManifestTentacle::new_with_llm_and_grants(installed.clone(), Some(factory), vec![]);

        assert!(tentacle.supports(&Need::new(NeedKind::Observe, ".")));
        assert!(tentacle.supports(&Need::new(NeedKind::Observe, "describe screen")));
        assert!(tentacle.supports(&Need::new(NeedKind::Observe, "current window")));
        assert!(tentacle.supports(&Need::new(NeedKind::Observe, "current browser tab")));
        assert!(tentacle.supports(&Need::new(NeedKind::Observe, "read clipboard")));
        assert!(tentacle.supports(&Need::new(NeedKind::Execute, "echo ok")));
        assert!(tentacle.supports(&Need::new(NeedKind::Execute, "open browser url")));
        assert!(tentacle.supports(&Need::new(NeedKind::Execute, "copy text to clipboard")));
    }

    #[test]
    fn field_mini_task_tentacle_supports_declared_needs_with_llm_available() {
        struct FakeChat;

        impl ChatClient for FakeChat {
            fn chat(&mut self, _messages: &[ChatMessage]) -> Result<ChatResponse, String> {
                Ok(ChatResponse {
                    content: r#"{"calls":[],"summary":"unused"}"#.to_string(),
                    metadata: BTreeMap::new(),
                })
            }
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();
        let installed = state.install_manifest(&root, "field-mini-task").unwrap();
        let factory: ChatClientFactory = Arc::new(|| Ok(Box::new(FakeChat)));
        let tentacle =
            ManifestTentacle::new_with_llm_and_grants(installed.clone(), Some(factory), vec![]);
        let mut task_need = Need::new(NeedKind::Verify, "Run math mini task math-mini-1");
        task_need
            .context
            .insert("field_pack".to_string(), "math".to_string());
        task_need
            .context
            .insert("field_mini_task".to_string(), "math-mini-1".to_string());

        let mut structured_tentacle = ManifestTentacle::new(installed.clone());
        let structured_plan = structured_tentacle
            .plan_tool(&task_need)
            .expect("structured field mini task plan");
        assert_eq!(structured_plan.source, "structured-need");
        assert_eq!(structured_plan.tool.id, "run_field_mini_task");
        assert!(structured_tentacle.supports(&task_need));
        assert!(!structured_tentacle.supports(&Need::new(
            NeedKind::Execute,
            "evolve recommend field-mini-task"
        )));

        assert!(tentacle.supports(&Need::new(
            NeedKind::Execute,
            "evolve recommend field-mini-task"
        )));
        assert!(tentacle.supports(&task_need));
    }
    #[test]
    fn beat_compacts_feed_trace_journal() {
        let repo = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
        let mut state = HarnessState::default();
        state
            .install_manifest(repo.join("tentacles"), "swe-agent")
            .unwrap();
        let mut harness = Harness::with_state(state);

        harness.feed(&[Need::new(NeedKind::Observe, ".")]);
        harness.feed(&[Need::new(NeedKind::Observe, "README.md")]);
        harness.feed(&[Need::new(NeedKind::Observe, "Cargo.toml")]);

        assert_eq!(harness.state.feed_traces.len(), 3);
        let report = harness.state.beat(2);

        assert_eq!(harness.state.feed_traces.len(), 2);
        assert!(report.beats.iter().any(|beat| {
            beat.name == "harness" && beat.data.get("feed_traces_dropped") == Some(&"1".to_string())
        }));
    }

    #[cfg(unix)]
    #[test]
    fn installed_manifest_tentacle_can_plan_with_llm_before_execution() {
        use std::os::unix::fs::PermissionsExt;

        let repo = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
        let fake =
            std::env::temp_dir().join(format!("octopus-manifest-llm-{}", std::process::id()));
        let _ = fs::remove_dir_all(&fake);
        fs::create_dir_all(&fake).unwrap();
        let curl = fake.join("fake-curl.sh");
        fs::write(
            &curl,
            "#!/bin/sh\nprintf '%s' '{\"choices\":[{\"message\":{\"content\":\"{\\\"calls\\\":[{\\\"tool\\\":\\\"read\\\",\\\"reason\\\":\\\"inspect requested file\\\"}],\\\"summary\\\":\\\"planned by llm\\\"}\"}}]}'\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&curl).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&curl, permissions).unwrap();
        let mut state = HarnessState::default();
        state
            .install_manifest(repo.join("tentacles"), "swe-agent")
            .unwrap();
        let config = OpenAiCompatibleConfig {
            model: "test-model".to_string(),
            api_key: None,
            base_url: "https://llm.example/v1".to_string(),
            timeout_seconds: 1,
            retry_count: 0,
            retry_delay_ms: 0,
            curl_command: curl.to_string_lossy().to_string(),
            tuning: OpenAiCompatibleTuning::default(),
        };
        let mut harness = Harness::with_state_and_manifest_llm(state, config);

        let feedback = harness.feed(&[Need::new(NeedKind::Observe, "Cargo.toml 1 1")]);

        assert_eq!(feedback.status, Status::Satisfied, "{}", feedback.summary);
        assert!(feedback.summary.contains("[package]"));
        assert_eq!(
            feedback.feeds[0].metadata.get("tool"),
            Some(&"read".to_string())
        );
        assert_eq!(
            feedback.feeds[0].metadata.get("plan_source"),
            Some(&"llm".to_string())
        );
        assert_eq!(
            feedback.feeds[0].metadata.get("action_count"),
            Some(&"1".to_string())
        );
        assert!(feedback.feeds[0]
            .metadata
            .get("plan")
            .is_some_and(|plan| plan.contains("inspect requested file")));
        assert!(feedback.feeds[0].evidence.iter().any(|evidence| {
            evidence.source == "tentacle_plan"
                && evidence.metadata.get("tool") == Some(&"read".to_string())
                && evidence.metadata.get("plan_source") == Some(&"llm".to_string())
        }));
        let _ = fs::remove_dir_all(fake);
    }

    #[cfg(unix)]
    #[test]
    fn installed_manifest_tentacle_executes_llm_action_sequence() {
        use std::os::unix::fs::PermissionsExt;

        let repo = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
        let fake = std::env::temp_dir().join(format!(
            "octopus-manifest-llm-sequence-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&fake);
        fs::create_dir_all(&fake).unwrap();
        let curl = fake.join("fake-curl.sh");
        fs::write(
            &curl,
            "#!/bin/sh\nprintf '%s' '{\"choices\":[{\"message\":{\"content\":\"{\\\"calls\\\":[{\\\"tool\\\":\\\"read\\\",\\\"reason\\\":\\\"inspect package\\\"},{\\\"tool\\\":\\\"read\\\",\\\"reason\\\":\\\"inspect version\\\",\\\"payload\\\":{\\\"query\\\":\\\"Cargo.toml 2 1\\\"}}],\\\"summary\\\":\\\"two-step observation\\\"}\"}}]}'\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&curl).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&curl, permissions).unwrap();
        let mut state = HarnessState::default();
        state
            .install_manifest(repo.join("tentacles"), "swe-agent")
            .unwrap();
        let config = OpenAiCompatibleConfig {
            model: "test-model".to_string(),
            api_key: None,
            base_url: "https://llm.example/v1".to_string(),
            timeout_seconds: 1,
            retry_count: 0,
            retry_delay_ms: 0,
            curl_command: curl.to_string_lossy().to_string(),
            tuning: OpenAiCompatibleTuning::default(),
        };
        let mut harness = Harness::with_state_and_manifest_llm(state, config);

        let feedback = harness.feed(&[Need::new(NeedKind::Observe, "Cargo.toml 1 1")]);

        assert_eq!(feedback.status, Status::Satisfied, "{}", feedback.summary);
        assert!(feedback.summary.contains("== action 1: read"));
        assert!(feedback.summary.contains("== action 2: read"));
        assert!(feedback.summary.contains("== read Cargo.toml:2-2 =="));
        assert_eq!(
            feedback.feeds[0].metadata.get("action_count"),
            Some(&"2".to_string())
        );
        assert_eq!(
            feedback.feeds[0].metadata.get("action_plan"),
            Some(&"read -> read".to_string())
        );
        assert_eq!(
            feedback.feeds[0].metadata.get("tools"),
            Some(&"read,read".to_string())
        );
        assert!(feedback.feeds[0].evidence.iter().any(|evidence| {
            evidence.source == "tentacle_plan"
                && evidence.metadata.get("action_count") == Some(&"2".to_string())
        }));
        assert_eq!(harness.state.feed_traces.len(), 1);
        assert_eq!(harness.state.feed_traces[0].tool.as_deref(), Some("read"));
        let _ = fs::remove_dir_all(fake);
    }
}
