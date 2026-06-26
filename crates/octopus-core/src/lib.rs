use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{Error, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const OCTOPUS_JSON_CONTRACT: &str = "octopus-json-v1";
const OCTOPUS_TOOL_CALL_SCHEMA: &str = "octopus-tool-call-v1";
const FEED_TRACE_QUERY_BYTES: usize = 160;
const FEED_TRACE_SUMMARY_BYTES: usize = 240;
const CHECK_HISTORY_OUTPUT_BYTES: usize = 480;
const MAX_TENTACLE_ACTIONS: usize = 2;
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

impl GoalRefinement {
    pub fn local() -> Self {
        Self {
            objective: None,
            constraints: Vec::new(),
            summary: None,
            needs: Vec::new(),
        }
    }

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
    pub state: String,
    pub source: String,
    pub summary: String,
    pub status: Status,
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
pub struct FeedTraceRecord {
    pub index: u64,
    pub need_kind: NeedKind,
    pub need_query: String,
    pub status: Status,
    pub tentacle: Option<String>,
    pub tool: Option<String>,
    pub plan_source: Option<String>,
    pub route: Option<String>,
    pub evidence_count: usize,
    pub summary: String,
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
    pub feed_trace_count: usize,
    pub check_history_count: usize,
    pub installed_profiles: Vec<String>,
    pub tentacles: Vec<TentacleStatus>,
    pub goal: Option<GoalSnapshot>,
    pub active_grants: Vec<String>,
    pub last_pet_event: Option<PetEvent>,
    pub latest_feed_trace: Option<FeedTraceRecord>,
    pub latest_check: Option<CheckHistoryRecord>,
    pub warnings: Vec<String>,
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
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ContextReport {
    pub brain: BrainContextReport,
    pub tentacles: Vec<TentacleContextReport>,
    pub hearts: Vec<HeartBeat>,
    pub next: Vec<String>,
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

#[derive(Default)]
pub struct RulePlanner;

impl Planner for RulePlanner {
    fn plan(&mut self, need: &Need, tools: &[ToolSpec]) -> Plan {
        let calls = tools
            .first()
            .map(|tool| {
                vec![ToolCall {
                    tool: tool.name.clone(),
                    reason: format!("{} can feed {}", tool.name, kind_key(&need.kind)),
                    payload: BTreeMap::from([("query".to_string(), need.query.clone())]),
                }]
            })
            .unwrap_or_default();
        let summary = if calls.is_empty() {
            "no matching tool".to_string()
        } else {
            "selected first matching tool".to_string()
        };
        Plan { calls, summary }
    }
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct OpenAiCompatibleConfig {
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: String,
    pub timeout_seconds: u64,
    pub curl_command: String,
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
        let curl_command =
            env::var(format!("{prefix}_CURL")).unwrap_or_else(|_| "curl".to_string());
        Ok(Self {
            model,
            api_key,
            base_url,
            timeout_seconds,
            curl_command,
        })
    }

    pub fn endpoint(&self) -> String {
        format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
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
        serde_json::to_string(&serde_json::json!({
            "model": self.config.model.as_str(),
            "messages": messages,
        }))
        .map_err(|error| error.to_string())
    }
}

impl ChatClient for OpenAiCompatibleChatClient {
    fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
        let body = self.request_body(messages)?;
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
        let content = data
            .pointer("/choices/0/message/content")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "chat response missing choices[0].message.content".to_string())?;
        Ok(ChatResponse {
            content: content.to_string(),
            metadata: BTreeMap::from([
                ("provider".to_string(), "openai-compatible".to_string()),
                ("model".to_string(), self.config.model.clone()),
                ("base_url".to_string(), self.config.base_url.clone()),
            ]),
        })
    }
}

pub struct ChatPlanner<C>
where
    C: ChatClient,
{
    client: C,
    fallback: RulePlanner,
}

impl<C> ChatPlanner<C>
where
    C: ChatClient,
{
    pub fn new(client: C) -> Self {
        Self {
            client,
            fallback: RulePlanner,
        }
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
        self.client
            .chat(&messages)
            .ok()
            .and_then(|response| serde_json::from_str::<Plan>(&response.content).ok())
            .filter(|plan| !plan.calls.is_empty())
            .unwrap_or_else(|| self.fallback.plan(need, tools))
    }
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
    pub check_history: Vec<CheckHistoryRecord>,
    #[serde(default)]
    pub next_check_history_index: u64,
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
        fs::write(path, content)
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
        let compacted = trace_dropped + check_dropped;
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
        let mut next = vec!["octopus chat \"refine your goal\"".to_string()];
        if let Some(need) = &next_need {
            next.push(format!(
                "octopus need {} {}",
                kind_key(&need.kind),
                shell_arg(&need.query)
            ));
            if let Some(tentacle) = tentacles.first() {
                next.push(format!(
                    "octopus think {} {} {}",
                    tentacle.id,
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                ));
            }
        } else {
            next.push("octopus context observe .".to_string());
        }
        next.push("octopus traces".to_string());
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
            next,
        }
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
        let state_args = state_path
            .map(|path| format!(" --state {}", shell_arg(&path.to_string_lossy())))
            .unwrap_or_default();
        let next_action = if self.installed_tentacles.is_empty() {
            format!("octopus{state_args} adapt")
        } else if self.goal.is_none() {
            format!("octopus{state_args} chat \"describe your goal\"")
        } else if self.routes.scores.is_empty() {
            format!("octopus{state_args} need observe .")
        } else {
            format!("octopus{state_args} beat 200")
        };
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
            feed_trace_count: self.feed_traces.len(),
            check_history_count: self.check_history.len(),
            installed_profiles: self.installed_profiles.clone(),
            tentacles,
            goal,
            active_grants,
            last_pet_event: self.last_pet_event.clone(),
            latest_feed_trace: self.feed_traces.last().cloned(),
            latest_check: self.check_history.last().cloned(),
            warnings,
            next_action,
        }
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
            state: state.into(),
            source: source.into(),
            summary: summary.into(),
            status,
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
        };
        self.feed_traces.push(trace.clone());
        trace
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
        self.record_pet_event(
            "harness",
            "evolve score",
            outcome.summary.clone(),
            outcome.status.clone(),
        );
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
        SelfIterationPlan {
            repository,
            authorized,
            mode: if authorized {
                "pr-ready".to_string()
            } else {
                "report-only".to_string()
            },
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
    manifest_llm_config: Option<OpenAiCompatibleConfig>,
}

impl Harness {
    pub fn new() -> Self {
        Self {
            state: HarnessState::default(),
            tentacles: Vec::new(),
            manifest_llm_config: None,
        }
    }

    pub fn with_state(state: HarnessState) -> Self {
        let mut harness = Self {
            state,
            tentacles: Vec::new(),
            manifest_llm_config: None,
        };
        harness.add_installed_tentacles_with_grants();
        harness
    }

    pub fn with_state_and_manifest_llm(
        state: HarnessState,
        config: OpenAiCompatibleConfig,
    ) -> Self {
        let mut harness = Self {
            state,
            tentacles: Vec::new(),
            manifest_llm_config: Some(config),
        };
        harness.add_installed_tentacles_with_grants();
        harness
    }

    pub fn add_tentacle(&mut self, tentacle: Box<dyn Tentacle>) {
        self.tentacles.push(tentacle);
    }

    fn add_installed_tentacles_with_grants(&mut self) {
        let grants = self.state.grants.clone();
        for tentacle in self.state.installed_tentacles.clone() {
            self.add_tentacle(Box::new(ManifestTentacle::new_with_llm_and_grants(
                tentacle,
                self.manifest_llm_config.clone(),
                grants.clone(),
            )));
        }
    }

    pub fn feed_one(&mut self, need: &Need) -> Feed {
        if let Some(feed) = self.memory_feed(need) {
            self.state.record_pet_event_from_feed(&feed);
            self.state.record_feed_trace_from_feed(&feed);
            return feed;
        }
        let candidates = self
            .tentacles
            .iter()
            .enumerate()
            .filter(|(_, tentacle)| tentacle.supports(need))
            .map(|(index, tentacle)| (index, tentacle.name().to_string()))
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            let feed = Feed::unsupported(need, "no tentacle supports this need");
            self.state.record_pet_event_from_feed(&feed);
            self.state.record_feed_trace_from_feed(&feed);
            return feed;
        }

        let names = candidates
            .iter()
            .map(|(_, name)| name.clone())
            .collect::<Vec<_>>();
        let decision = self
            .state
            .routes
            .choose(need, &names)
            .expect("candidates exist");
        let index = candidates
            .iter()
            .find(|(_, name)| name == &decision.tentacle)
            .map(|(index, _)| *index)
            .expect("chosen tentacle exists");
        let mut feed = self.tentacles[index].feed(need);
        feed.metadata
            .insert("tentacle".to_string(), decision.tentacle);
        feed.metadata.insert("route".to_string(), decision.reason);
        if !feed_is_authorization_blocked(&feed) {
            self.state.routes.learn(&feed);
        }
        self.state.record_pet_event_from_feed(&feed);
        self.state.record_feed_trace_from_feed(&feed);
        feed
    }

    pub fn feed(&mut self, needs: &[Need]) -> Feedback {
        let feeds = needs.iter().map(|need| self.feed_one(need)).collect();
        Feedback::from_feeds(feeds)
    }

    pub fn chat(&mut self, message: impl Into<String>) -> GoalChat {
        let message = message.into();
        self.chat_with_refinement(message, GoalRefinement::local())
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
        GoalChat {
            goal,
            turn,
            feedback,
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
    llm_config: Option<OpenAiCompatibleConfig>,
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
        Self::new_with_llm_and_grants(installed, llm_config, Vec::new())
    }

    pub fn new_with_llm_and_grants(
        installed: InstalledTentacle,
        llm_config: Option<OpenAiCompatibleConfig>,
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
            llm_config,
            grants,
        }
    }

    fn plan_tool(&mut self, need: &Need) -> Option<ManifestToolPlan> {
        let tools = self.available_tools(need);
        self.llm_plan_tool(need, &tools)
            .or_else(|| self.rule_plan_tool(need, &tools))
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

    fn available_tools(&self, need: &Need) -> Vec<InstalledToolRef> {
        if self.route_id == "computer-use-agent"
            && need.kind == NeedKind::Observe
            && !is_desktop_observe(&need.query)
        {
            return Vec::new();
        }
        if self.route_id == "computer-use-agent"
            && need.kind == NeedKind::Execute
            && !is_desktop_execute(&need.query)
        {
            return Vec::new();
        }
        if self.route_id == "visual"
            && need.kind == NeedKind::Observe
            && !is_visual_observe(&need.query)
        {
            return Vec::new();
        }
        self.tools
            .iter()
            .filter(|tool| self.can_feed_tool(tool))
            .cloned()
            .collect()
    }

    fn rule_plan_tool(&self, need: &Need, tools: &[InstalledToolRef]) -> Option<ManifestToolPlan> {
        let preferred = match need.kind {
            NeedKind::Observe
                if self.route_id == "computer-use-agent" && is_clipboard_query(&need.query) =>
            {
                ["clipboard_read", "window_status", "describe_screen", "mcp"].as_slice()
            }
            NeedKind::Observe
                if self.route_id == "computer-use-agent" && is_browser_observe(&need.query) =>
            {
                [
                    "browser_status",
                    "window_status",
                    "describe_screen",
                    "screenshot",
                    "mcp",
                ]
                .as_slice()
            }
            NeedKind::Observe => [
                "inspect_repo",
                "window_status",
                "describe_screen",
                "browser_status",
                "clipboard_read",
                "status_pet",
                "screenshot",
                "read",
                "mcp",
            ]
            .as_slice(),
            NeedKind::Verify | NeedKind::Reproduce => [
                "run_tests",
                "github_status",
                "inspect_repo",
                "read",
                "write_and_run",
                "bash",
                "mcp",
            ]
            .as_slice(),
            NeedKind::Compare => ["inspect_repo", "read"].as_slice(),
            NeedKind::Execute
                if self.route_id == "computer-use-agent" && is_clipboard_query(&need.query) =>
            {
                ["clipboard_write", "bash", "mcp", "open_url"].as_slice()
            }
            NeedKind::Execute => ["write_and_run", "bash", "mcp", "open_url"].as_slice(),
            _ => [].as_slice(),
        };
        let candidates = tools.iter().map(|tool| tool.id.clone()).collect::<Vec<_>>();
        let tool = preferred
            .iter()
            .find_map(|id| tools.iter().find(|tool| tool.id == *id))
            .or_else(|| tools.first())
            .cloned()?;
        let reason = manifest_plan_reason(need, &tool);
        Some(ManifestToolPlan {
            tool: tool.clone(),
            reason,
            candidates,
            source: "rule".to_string(),
            calls: vec![ManifestToolCall {
                tool,
                reason: "rule selected primary tool".to_string(),
                payload: BTreeMap::new(),
            }],
        })
    }

    fn llm_plan_tool(
        &mut self,
        need: &Need,
        tools: &[InstalledToolRef],
    ) -> Option<ManifestToolPlan> {
        let config = self.llm_config.clone()?;
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
                    tool.contract.as_deref().unwrap_or("legacy"),
                    tool_permission_label(tool.permission.as_ref()),
                    tool.input,
                    tool.output
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let mut client = OpenAiCompatibleChatClient::new(config);
        let response = client
            .chat(&[
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
                        "Need: {}: {}\nAvailable tools:\n{}",
                        kind_key(&need.kind),
                        need.query,
                        tool_sheet
                    ),
                ),
            ])
            .ok()?;
        let plan = serde_json::from_str::<Plan>(&response.content).ok()?;
        let calls = plan
            .calls
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
            .collect::<Vec<_>>();
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
            source: "llm".to_string(),
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

    fn run_tool(
        &self,
        tool: &InstalledToolRef,
        need: &Need,
        query_override: Option<&str>,
    ) -> ToolResult {
        let active_grant = match self.authorize_tool(tool) {
            Ok(grant) => grant.map(|grant| grant.id.clone()),
            Err(result) => return result,
        };
        let mut action_need = need.clone();
        if let Some(query) = clean_optional(query_override) {
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
        if uses_json_contract {
            command.stdin(Stdio::piped());
        } else {
            match tool.id.as_str() {
                "inspect_repo" | "run_tests" | "github_status" => {
                    command.arg(default_tool_query(&action_need.query));
                }
                "read" => {
                    for arg in read_args(&action_need.query) {
                        command.arg(arg);
                    }
                }
                "write_and_run" => {
                    command.arg(".");
                    command.stdin(Stdio::piped());
                }
                "bash" => {
                    command.arg(".");
                    command.stdin(Stdio::piped());
                }
                "mcp" => {
                    for arg in mcp_args(&action_need.query) {
                        command.arg(arg);
                    }
                }
                "clipboard_read" => {}
                "clipboard_write" => {
                    command.arg(default_tool_query(&action_need.query));
                }
                _ => {
                    command.arg(default_tool_query(&action_need.query));
                }
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
        } else if tool.id == "write_and_run" || tool.id == "bash" {
            if let Some(mut stdin) = child.stdin.take() {
                if let Err(error) = stdin.write_all(action_need.query.as_bytes()) {
                    return ToolResult::failed(format!("{} stdin failed: {error}", tool.id));
                }
            }
        }
        let output = match child.wait_with_output() {
            Ok(output) => output,
            Err(error) => return ToolResult::failed(format!("{} failed: {error}", tool.id)),
        };
        let text = trim_output(&String::from_utf8_lossy(if output.stdout.is_empty() {
            &output.stderr
        } else {
            &output.stdout
        }));
        let mut result = if output.status.success() {
            if uses_json_contract {
                tool_result_from_json(&tool.id, &text)
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
        if uses_json_contract {
            result.metadata.insert(
                "contract".to_string(),
                tool.contract
                    .clone()
                    .unwrap_or_else(|| OCTOPUS_JSON_CONTRACT.to_string()),
            );
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
        self.installed
            .needs
            .iter()
            .any(|need_key| need_key == kind_key(&need.kind))
            && self
                .rule_plan_tool(need, &self.available_tools(need))
                .is_some()
    }

    fn feed(&mut self, need: &Need) -> Feed {
        let Some(plan) = self.plan_tool(need) else {
            return Feed::unsupported(need, "no manifest tool supports this need");
        };
        let tool = plan.tool.clone();
        let mut action_results = Vec::new();
        for call in plan.calls.iter().take(MAX_TENTACLE_ACTIONS) {
            let query = tool_call_query(call);
            let result = self.run_tool(&call.tool, need, query.as_deref());
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
pub struct EvolutionPolicy {
    pub editable: Vec<String>,
    pub checks: Vec<String>,
    pub constraints: Vec<String>,
    #[serde(default = "default_evolution_surfaces")]
    pub surfaces: Vec<EvolutionSurface>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EvolutionSurface {
    pub id: String,
    pub description: String,
    pub targets: Vec<String>,
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleEvolutionProposal {
    pub tentacle_id: String,
    pub tentacle_name: String,
    pub objective: String,
    #[serde(default = "default_evolution_generator")]
    pub generator: String,
    #[serde(default)]
    pub planner_summary: String,
    pub manifest_path: String,
    pub brain_kind: String,
    pub current_brain_prompt: String,
    pub editable: Vec<String>,
    pub surfaces: Vec<EvolutionSurface>,
    pub checks: Vec<String>,
    pub constraints: Vec<String>,
    #[serde(default)]
    pub previous_outcomes: Vec<EvolutionOutcome>,
    #[serde(default)]
    pub recent_feed_traces: Vec<FeedTraceRecord>,
    #[serde(default)]
    pub recent_check_history: Vec<CheckHistoryRecord>,
    pub files: Vec<EvolutionFileTarget>,
    pub patch_candidates: Vec<EvolutionPatchCandidate>,
    pub next_steps: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EvolutionFileTarget {
    pub path: String,
    pub action: String,
    pub rationale: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EvolutionPatchCandidate {
    pub id: String,
    pub surface_id: String,
    pub title: String,
    pub target: String,
    pub rationale: String,
    #[serde(default)]
    pub feedback: Vec<EvolutionPatchFeedback>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_patch: Option<String>,
    pub change_plan: Vec<String>,
    pub checks: Vec<String>,
    pub draft: EvolutionPatchDraft,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EvolutionPatchFeedback {
    pub kind: String,
    pub source_index: u64,
    pub status: Status,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EvolutionPatchDraft {
    pub path: String,
    pub status: String,
    pub authorization_required: bool,
    pub apply_hint: String,
}

fn default_evolution_generator() -> String {
    "local".to_string()
}

#[derive(Debug, Deserialize)]
struct LlmEvolutionResponse {
    #[serde(default)]
    summary: String,
    #[serde(default)]
    candidates: Vec<LlmEvolutionCandidate>,
}

#[derive(Debug, Deserialize)]
struct LlmEvolutionCandidate {
    surface_id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    target: String,
    #[serde(default)]
    rationale: String,
    #[serde(default)]
    change_plan: Vec<String>,
    #[serde(default)]
    checks: Vec<String>,
    #[serde(default)]
    suggested_patch: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EvolutionArtifact {
    pub directory: String,
    pub proposal_path: String,
    pub candidates_path: String,
    pub drafts_path: String,
    pub patch_draft_paths: Vec<String>,
    pub json_path: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EvolutionApplyPlan {
    pub tentacle_id: String,
    pub candidate_id: String,
    pub objective: String,
    pub authorized: bool,
    pub status: String,
    pub required_grant: String,
    pub active_grant: Option<String>,
    pub target: String,
    pub draft_path: String,
    pub checks: Vec<String>,
    #[serde(default)]
    pub feedback: Vec<EvolutionPatchFeedback>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_patch: Option<String>,
    pub guardrails: Vec<String>,
    pub next_steps: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EvolutionRecommendation {
    pub tentacle_id: String,
    pub objective: String,
    pub candidate_id: String,
    pub candidate_title: String,
    pub surface_id: String,
    pub outcome_count: usize,
    pub feed_trace_count: usize,
    pub check_history_count: usize,
    pub recommendation_score: f32,
    pub reason: String,
    pub apply: EvolutionApplyPlan,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EvolutionApplyArtifact {
    pub directory: String,
    pub plan_path: String,
    pub patch_path: Option<String>,
    pub json_path: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HarnessBeatEvolution {
    pub source_check_index: u64,
    pub source_kind: String,
    pub source_index: u64,
    pub source_summary: String,
    pub tentacle_id: String,
    pub objective: String,
    pub candidate_id: String,
    pub status: String,
    pub reason: String,
    pub recommendation_score: f32,
    pub proposal_path: String,
    pub apply_plan_path: String,
    pub apply_plan_preview: String,
    pub patch_path: Option<String>,
    pub next_action: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleScaffold {
    pub tentacle_id: String,
    pub runtime: String,
    pub directory: String,
    pub manifest_path: String,
    pub tool_path: Option<String>,
    pub next_steps: Vec<String>,
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
    let root = root.as_ref();
    let loaded = load_tentacle_manifests(root)
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|loaded| loaded.manifest.id == tentacle_id)
        .ok_or_else(|| format!("unknown tentacle: {tentacle_id}"))?;
    let installed = installed_tentacle_from_manifest(root, loaded)?;
    let mut tentacle =
        ManifestTentacle::new_with_llm_and_grants(installed, llm_config, grants.to_vec());
    tentacle
        .thinking_plan(need)
        .ok_or_else(|| format!("{tentacle_id} cannot feed {}", kind_key(&need.kind)))
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

pub fn propose_tentacle_evolution(
    root: impl AsRef<Path>,
    tentacle_id: &str,
    objective: &str,
) -> Result<TentacleEvolutionProposal, String> {
    propose_tentacle_evolution_with_outcomes(root, tentacle_id, objective, &[])
}

pub fn propose_tentacle_evolution_with_state(
    root: impl AsRef<Path>,
    tentacle_id: &str,
    objective: &str,
    state: &HarnessState,
) -> Result<TentacleEvolutionProposal, String> {
    let outcomes = state.recent_evolution_outcomes(tentacle_id, 5);
    let feed_traces = state.recent_feed_traces_for_tentacle(tentacle_id, 8);
    let check_history = state.recent_check_history_for_tentacle(tentacle_id, 8);
    propose_tentacle_evolution_with_feedback(
        root,
        tentacle_id,
        objective,
        &outcomes,
        &feed_traces,
        &check_history,
    )
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
    let mut proposal = propose_tentacle_evolution_with_state(root, tentacle_id, objective, state)?;
    let plan = llm_evolution_plan(&proposal, client)?;
    proposal.generator = "llm".to_string();
    proposal.planner_summary = plan.summary;
    proposal.patch_candidates = plan.candidates;
    proposal
        .next_steps
        .insert(0, "review LLM-generated harness candidates".to_string());
    Ok(proposal)
}

struct ParsedLlmEvolutionPlan {
    summary: String,
    candidates: Vec<EvolutionPatchCandidate>,
}

struct EvolutionCandidateFeedback<'a> {
    feed_traces: &'a [FeedTraceRecord],
    check_history: &'a [CheckHistoryRecord],
}

fn llm_evolution_plan<C>(
    proposal: &TentacleEvolutionProposal,
    client: &mut C,
) -> Result<ParsedLlmEvolutionPlan, String>
where
    C: ChatClient,
{
    let messages = vec![
        ChatMessage::new(
            ChatRole::System,
            "You are an Octopus harness evolution brain. Preserve this context policy: clean-brain LLM context is only Goal, Mem, Need, and Feed; tentacle LLM context is Need, Tool, Action, Tool, Action, then Feed. Return only JSON and no hidden reasoning.",
        ),
        ChatMessage::new(ChatRole::User, llm_evolution_prompt(proposal)?),
    ];
    let response = client.chat(&messages)?;
    let content = extract_json_object(&response.content)
        .ok_or_else(|| "evolution LLM response did not contain a JSON object".to_string())?;
    let parsed = serde_json::from_str::<LlmEvolutionResponse>(content)
        .map_err(|error| format!("invalid evolution LLM JSON: {error}"))?;
    let candidates = parsed
        .candidates
        .into_iter()
        .map(|candidate| llm_candidate_to_evolution(proposal, candidate))
        .collect::<Result<Vec<_>, _>>()?;
    if candidates.is_empty() {
        return Err("evolution LLM returned no candidates".to_string());
    }
    Ok(ParsedLlmEvolutionPlan {
        summary: if parsed.summary.trim().is_empty() {
            "LLM-generated harness evolution candidates".to_string()
        } else {
            parsed.summary
        },
        candidates,
    })
}

fn llm_evolution_prompt(proposal: &TentacleEvolutionProposal) -> Result<String, String> {
    let payload = serde_json::json!({
        "tentacle_id": proposal.tentacle_id,
        "tentacle_name": proposal.tentacle_name,
        "objective": proposal.objective,
        "brain_kind": proposal.brain_kind,
        "current_brain_prompt": proposal.current_brain_prompt,
        "editable": proposal.editable,
        "surfaces": proposal.surfaces,
        "checks": proposal.checks,
        "constraints": proposal.constraints,
        "previous_outcomes": proposal.previous_outcomes,
        "recent_feed_traces": proposal.recent_feed_traces,
        "recent_check_history": proposal.recent_check_history,
        "files": proposal.files,
        "context_policy": {
            "clean_brain": ["Goal", "Mem", "Need", "Feed"],
            "tentacle_brain": ["Need", "Tool", "Action", "Tool", "Action", "Feed"],
            "harness_evolution": "modify prompt, metadata, runtime code, or policy without moving tool burden into the clean brain"
        },
        "return_schema": {
            "summary": "short reason for the selected harness changes",
            "candidates": [
                {
                    "surface_id": "one declared surface id",
                    "title": "short title",
                    "target": "relative manifest/tool target inside this tentacle",
                    "rationale": "why this improves Need-to-Feed",
                    "change_plan": ["concrete harness edit step"],
                    "checks": ["command to verify"],
                    "suggested_patch": "optional unified diff for the declared target only; omit if uncertain"
                }
            ]
        }
    });
    let payload = serde_json::to_string_pretty(&payload).map_err(|error| error.to_string())?;
    Ok(format!(
        "Generate code-as-harness evolution candidates from this JSON. Keep the clean brain out of tool details and preserve the declared context policy. Return only a JSON object matching return_schema.\n{payload}"
    ))
}

fn llm_candidate_to_evolution(
    proposal: &TentacleEvolutionProposal,
    candidate: LlmEvolutionCandidate,
) -> Result<EvolutionPatchCandidate, String> {
    let surface_index = proposal
        .surfaces
        .iter()
        .position(|surface| surface.id == candidate.surface_id)
        .ok_or_else(|| {
            format!(
                "evolution LLM returned unknown surface: {}",
                candidate.surface_id
            )
        })?;
    let surface = &proposal.surfaces[surface_index];
    let checks = if candidate.checks.is_empty() {
        proposal.checks.clone()
    } else {
        candidate.checks
    };
    let checks = if checks.is_empty() {
        vec!["cargo test".to_string()]
    } else {
        checks
    };
    let change_plan = if candidate.change_plan.is_empty() {
        evolution_candidate_plan(surface)
    } else {
        candidate.change_plan
    };
    let target = llm_candidate_target(proposal, surface, &candidate.target);
    let mut patch_candidate = EvolutionPatchCandidate {
        id: evolution_candidate_id(surface_index, &surface.id),
        surface_id: surface.id.clone(),
        title: if candidate.title.trim().is_empty() {
            evolution_candidate_title(&surface.id, &proposal.tentacle_id)
        } else {
            candidate.title
        },
        target,
        rationale: if candidate.rationale.trim().is_empty() {
            format!(
                "advance {} through {}",
                proposal.objective, surface.description
            )
        } else {
            candidate.rationale
        },
        feedback: Vec::new(),
        suggested_patch: clean_suggested_patch(candidate.suggested_patch),
        change_plan,
        checks,
        draft: evolution_patch_draft(surface_index, surface),
    };
    patch_candidate.feedback =
        evolution_candidate_feedback_from_proposal(&patch_candidate, proposal);
    Ok(patch_candidate)
}

fn llm_candidate_target(
    proposal: &TentacleEvolutionProposal,
    surface: &EvolutionSurface,
    target: &str,
) -> String {
    let manifest_dir = Path::new(&proposal.manifest_path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("tentacles").join(&proposal.tentacle_id));
    let target = target.trim();
    if target.is_empty() || target.contains("..") {
        return evolution_candidate_target(&manifest_dir, surface);
    }
    if target.starts_with("brain.")
        || target.starts_with("tools[]")
        || target.starts_with("evolution.")
    {
        return format!(
            "{}#{}",
            manifest_dir.join("manifest.json").to_string_lossy(),
            target
        );
    }
    if let Some(suffix) = target.strip_prefix("manifest.json") {
        return format!(
            "{}{}",
            manifest_dir.join("manifest.json").to_string_lossy(),
            suffix
        );
    }
    let path = Path::new(target);
    if path.is_absolute() {
        let manifest_dir_text = manifest_dir.to_string_lossy();
        return if target.starts_with(manifest_dir_text.as_ref()) {
            target.to_string()
        } else {
            evolution_candidate_target(&manifest_dir, surface)
        };
    }
    manifest_dir.join(target).to_string_lossy().to_string()
}

fn extract_json_object(value: &str) -> Option<&str> {
    let start = value.find('{')?;
    let end = value.rfind('}')?;
    (start <= end).then_some(&value[start..=end])
}

fn propose_tentacle_evolution_with_outcomes(
    root: impl AsRef<Path>,
    tentacle_id: &str,
    objective: &str,
    previous_outcomes: &[EvolutionOutcome],
) -> Result<TentacleEvolutionProposal, String> {
    propose_tentacle_evolution_with_feedback(
        root,
        tentacle_id,
        objective,
        previous_outcomes,
        &[],
        &[],
    )
}

fn propose_tentacle_evolution_with_feedback(
    root: impl AsRef<Path>,
    tentacle_id: &str,
    objective: &str,
    previous_outcomes: &[EvolutionOutcome],
    recent_feed_traces: &[FeedTraceRecord],
    recent_check_history: &[CheckHistoryRecord],
) -> Result<TentacleEvolutionProposal, String> {
    let root = root.as_ref();
    let loaded = load_tentacle_manifests(root)
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|loaded| loaded.manifest.id == tentacle_id)
        .ok_or_else(|| format!("unknown manifest: {tentacle_id}"))?;
    let objective = if objective.trim().is_empty() {
        "improve feed quality while keeping the clean brain tool-free"
    } else {
        objective.trim()
    };
    let manifest_dir = Path::new(&loaded.path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.join(tentacle_id));
    let files = loaded
        .manifest
        .evolution
        .editable
        .iter()
        .map(|target| evolution_file_target(&manifest_dir, target, objective))
        .collect::<Vec<_>>();
    let patch_candidates = evolution_patch_candidates(
        &manifest_dir,
        &loaded.manifest.id,
        objective,
        &loaded.manifest.evolution.surfaces,
        &loaded.manifest.evolution.checks,
        &loaded.manifest.tools,
        EvolutionCandidateFeedback {
            feed_traces: recent_feed_traces,
            check_history: recent_check_history,
        },
    );
    let mut next_steps = vec![
        "review PROPOSAL.md".to_string(),
        "review PATCH_CANDIDATES.md".to_string(),
        "edit only listed harness files".to_string(),
    ];
    next_steps.extend(
        loaded
            .manifest
            .evolution
            .checks
            .iter()
            .map(|check| format!("run: {check}")),
    );
    if !loaded
        .manifest
        .evolution
        .checks
        .iter()
        .any(|check| check.contains("cargo test"))
    {
        next_steps.push("run: cargo test".to_string());
    }
    Ok(TentacleEvolutionProposal {
        tentacle_id: loaded.manifest.id,
        tentacle_name: loaded.manifest.name,
        objective: objective.to_string(),
        generator: "local".to_string(),
        planner_summary: "local harness surface planner".to_string(),
        manifest_path: loaded.path,
        brain_kind: loaded.manifest.brain.kind,
        current_brain_prompt: loaded.manifest.brain.prompt,
        editable: loaded.manifest.evolution.editable,
        surfaces: loaded.manifest.evolution.surfaces,
        checks: loaded.manifest.evolution.checks,
        constraints: loaded.manifest.evolution.constraints,
        previous_outcomes: previous_outcomes.to_vec(),
        recent_feed_traces: recent_feed_traces.to_vec(),
        recent_check_history: recent_check_history.to_vec(),
        files,
        patch_candidates,
        next_steps,
    })
}

pub fn write_tentacle_evolution_artifacts(
    workspace_root: impl AsRef<Path>,
    proposal: &TentacleEvolutionProposal,
) -> Result<EvolutionArtifact, String> {
    let directory = workspace_root
        .as_ref()
        .join(".octopus")
        .join("evolution")
        .join(&proposal.tentacle_id);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let patches_dir = directory.join("patches");
    fs::create_dir_all(&patches_dir).map_err(|error| error.to_string())?;
    let proposal_path = directory.join("PROPOSAL.md");
    let candidates_path = directory.join("PATCH_CANDIDATES.md");
    let drafts_path = directory.join("PATCH_DRAFTS.md");
    let json_path = directory.join("proposal.json");
    fs::write(&proposal_path, render_tentacle_evolution_proposal(proposal))
        .map_err(|error| error.to_string())?;
    fs::write(&candidates_path, render_tentacle_patch_candidates(proposal))
        .map_err(|error| error.to_string())?;
    fs::write(&drafts_path, render_tentacle_patch_drafts(proposal))
        .map_err(|error| error.to_string())?;
    let mut patch_draft_paths = Vec::new();
    for candidate in &proposal.patch_candidates {
        let draft_path = directory.join(&candidate.draft.path);
        if let Some(parent) = draft_path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(&draft_path, render_single_patch_draft(proposal, candidate))
            .map_err(|error| error.to_string())?;
        patch_draft_paths.push(draft_path.to_string_lossy().to_string());
    }
    fs::write(
        &json_path,
        serde_json::to_string_pretty(proposal).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(EvolutionArtifact {
        directory: directory.to_string_lossy().to_string(),
        proposal_path: proposal_path.to_string_lossy().to_string(),
        candidates_path: candidates_path.to_string_lossy().to_string(),
        drafts_path: drafts_path.to_string_lossy().to_string(),
        patch_draft_paths,
        json_path: json_path.to_string_lossy().to_string(),
    })
}

pub fn plan_tentacle_evolution_apply(
    proposal: &TentacleEvolutionProposal,
    state: &HarnessState,
    candidate_id: &str,
) -> Result<EvolutionApplyPlan, String> {
    let candidate = proposal
        .patch_candidates
        .iter()
        .find(|candidate| candidate_matches(candidate, candidate_id))
        .ok_or_else(|| format!("unknown evolution candidate: {candidate_id}"))?;
    let required_grant = format!("octopus:evolve:{}", proposal.tentacle_id);
    let grant = state.active_grant("octopus", &format!("evolve:{}", proposal.tentacle_id));
    let authorized = grant.is_some_and(|grant| {
        grant
            .permissions
            .iter()
            .any(|permission| permission == "harness:write")
    });
    let active_grant = grant.map(|grant| grant.id.clone());
    let status = if authorized {
        "ready_for_authorized_patch"
    } else {
        "needs_authorization"
    };
    let mut next_steps = vec![format!("review {}", candidate.draft.path)];
    if authorized {
        next_steps.push("turn the draft into a narrow harness patch".to_string());
        next_steps.extend(candidate.checks.iter().map(|check| format!("run: {check}")));
    } else {
        next_steps.push(format!(
            "run: octopus oauth octopus evolve:{} harness:write",
            proposal.tentacle_id
        ));
    }
    Ok(EvolutionApplyPlan {
        tentacle_id: proposal.tentacle_id.clone(),
        candidate_id: candidate.id.clone(),
        objective: proposal.objective.clone(),
        authorized,
        status: status.to_string(),
        required_grant,
        active_grant,
        target: candidate.target.clone(),
        draft_path: candidate.draft.path.clone(),
        checks: candidate.checks.clone(),
        feedback: candidate.feedback.clone(),
        suggested_patch: candidate.suggested_patch.clone(),
        guardrails: vec![
            "do not modify kernel code from an evolution draft".to_string(),
            "patch only declared harness targets".to_string(),
            "run listed checks before commit".to_string(),
        ],
        next_steps,
    })
}

pub fn recommend_tentacle_evolution_apply(
    proposal: &TentacleEvolutionProposal,
    state: &HarnessState,
) -> Result<EvolutionRecommendation, String> {
    let Some(scored) = proposal
        .patch_candidates
        .iter()
        .map(|candidate| {
            let (score, count) = evolution_candidate_outcome_score(candidate, proposal);
            let (feed_score, feed_count) =
                evolution_candidate_feed_trace_score(candidate, proposal);
            let (check_score, check_count) =
                evolution_candidate_check_history_score(candidate, proposal);
            (
                candidate,
                score + feed_score + check_score,
                count,
                feed_count,
                check_count,
            )
        })
        .max_by(|left, right| {
            left.1
                .total_cmp(&right.1)
                .then_with(|| right.0.id.cmp(&left.0.id))
        })
    else {
        return Err("no evolution candidates to recommend".to_string());
    };
    let (candidate, score, outcome_count, feed_trace_count, check_history_count) = scored;
    let apply = plan_tentacle_evolution_apply(proposal, state, &candidate.id)?;
    let reason = if outcome_count > 0 {
        format!("selected from {outcome_count} previous outcomes with score {score:.2}")
    } else if feed_trace_count > 0 {
        format!(
            "selected from {feed_trace_count} matching Feed trace records with score {score:.2}"
        )
    } else if check_history_count > 0 {
        format!("selected from {check_history_count} matching check history records with score {score:.2}")
    } else if proposal.generator == "llm" {
        "selected an LLM-generated candidate after feeding previous outcomes to the planner"
            .to_string()
    } else {
        "selected candidate; no previous scored outcome matched these candidates".to_string()
    };
    Ok(EvolutionRecommendation {
        tentacle_id: proposal.tentacle_id.clone(),
        objective: proposal.objective.clone(),
        candidate_id: candidate.id.clone(),
        candidate_title: candidate.title.clone(),
        surface_id: candidate.surface_id.clone(),
        outcome_count,
        feed_trace_count,
        check_history_count,
        recommendation_score: score,
        reason,
        apply,
    })
}

pub fn write_tentacle_apply_artifacts(
    workspace_root: impl AsRef<Path>,
    plan: &EvolutionApplyPlan,
) -> Result<EvolutionApplyArtifact, String> {
    let directory = workspace_root
        .as_ref()
        .join(".octopus")
        .join("evolution")
        .join(&plan.tentacle_id)
        .join("apply");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let stem = plan.candidate_id.replace('/', "-");
    let plan_path = directory.join(format!("{stem}.md"));
    let patch_file_path = directory.join(format!("{stem}.patch"));
    let patch_path = if plan.authorized {
        fs::write(
            &patch_file_path,
            render_authorized_apply_patch(plan, &directory),
        )
        .map_err(|error| error.to_string())?;
        Some(patch_file_path)
    } else {
        if patch_file_path.exists() {
            fs::remove_file(&patch_file_path).map_err(|error| error.to_string())?;
        }
        None
    };
    let json_path = directory.join(format!("{stem}.json"));
    fs::write(&plan_path, render_tentacle_apply_plan(plan)).map_err(|error| error.to_string())?;
    fs::write(
        &json_path,
        serde_json::to_string_pretty(plan).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(EvolutionApplyArtifact {
        directory: directory.to_string_lossy().to_string(),
        plan_path: plan_path.to_string_lossy().to_string(),
        patch_path: patch_path.map(|path| path.to_string_lossy().to_string()),
        json_path: json_path.to_string_lossy().to_string(),
    })
}

pub fn write_harness_beat_evolution_artifacts(
    tentacles_root: impl AsRef<Path>,
    workspace_root: impl AsRef<Path>,
    state: &HarnessState,
) -> Result<Option<HarnessBeatEvolution>, String> {
    let tentacles_root = tentacles_root.as_ref();
    let workspace_root = workspace_root.as_ref();
    for record in state
        .check_history
        .iter()
        .rev()
        .filter(|record| evolution_actionable_check_status(&record.status))
    {
        let objective = check_history_evolution_objective(record);
        let Ok(proposal) = propose_tentacle_evolution_with_state(
            tentacles_root,
            &record.tentacle_id,
            &objective,
            state,
        ) else {
            continue;
        };
        return harness_beat_evolution_from_proposal(
            workspace_root,
            proposal,
            state,
            "check_history",
            record.index,
            record.index,
            short_text(&one_line(&check_history_evolution_detail(record)), 160),
        )
        .map(Some);
    }
    for trace in state
        .feed_traces
        .iter()
        .rev()
        .filter(|trace| evolution_actionable_feed_trace_status(&trace.status))
    {
        let Some(tentacle_id) = trace.tentacle.as_deref() else {
            continue;
        };
        let objective = feed_trace_evolution_objective(trace);
        let Ok(proposal) =
            propose_tentacle_evolution_with_state(tentacles_root, tentacle_id, &objective, state)
        else {
            continue;
        };
        return harness_beat_evolution_from_proposal(
            workspace_root,
            proposal,
            state,
            "feed_trace",
            trace.index,
            0,
            short_text(&one_line(&trace.summary), 160),
        )
        .map(Some);
    }
    Ok(None)
}

fn harness_beat_evolution_from_proposal(
    workspace_root: &Path,
    proposal: TentacleEvolutionProposal,
    state: &HarnessState,
    source_kind: &str,
    source_index: u64,
    source_check_index: u64,
    source_summary: String,
) -> Result<HarnessBeatEvolution, String> {
    let proposal_artifact = write_tentacle_evolution_artifacts(workspace_root, &proposal)?;
    let recommendation = recommend_tentacle_evolution_apply(&proposal, state)?;
    let apply_artifact = write_tentacle_apply_artifacts(workspace_root, &recommendation.apply)?;
    let apply_plan_preview = fs::read_to_string(&apply_artifact.plan_path)
        .map(|content| short_text(&content, 2000))
        .unwrap_or_default();
    let next_action = if recommendation.apply.authorized {
        format!(
            "octopus evolve apply {} {}",
            recommendation.tentacle_id, recommendation.candidate_id
        )
    } else {
        format!(
            "octopus oauth octopus evolve:{} harness:write",
            recommendation.tentacle_id
        )
    };
    Ok(HarnessBeatEvolution {
        source_check_index,
        source_kind: source_kind.to_string(),
        source_index,
        source_summary,
        tentacle_id: recommendation.tentacle_id,
        objective: recommendation.objective,
        candidate_id: recommendation.candidate_id,
        status: recommendation.apply.status,
        reason: recommendation.reason,
        recommendation_score: recommendation.recommendation_score,
        proposal_path: proposal_artifact.proposal_path,
        apply_plan_path: apply_artifact.plan_path,
        apply_plan_preview,
        patch_path: apply_artifact.patch_path,
        next_action,
    })
}

pub fn render_tentacle_apply_plan(plan: &EvolutionApplyPlan) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# Evolution Apply Plan: {}\n\n",
        plan.candidate_id
    ));
    markdown.push_str(&format!("tentacle: `{}`\n", plan.tentacle_id));
    markdown.push_str(&format!("objective: {}\n", plan.objective));
    markdown.push_str(&format!("status: `{}`\n", plan.status));
    markdown.push_str(&format!("authorized: {}\n", plan.authorized));
    markdown.push_str(&format!("required_grant: `{}`\n", plan.required_grant));
    if let Some(grant) = &plan.active_grant {
        markdown.push_str(&format!("active_grant: `{grant}`\n"));
    }
    markdown.push_str(&format!("target: `{}`\n", plan.target));
    markdown.push_str(&format!("draft: `{}`\n\n", plan.draft_path));
    if plan.authorized {
        markdown.push_str(&format!("patch: `{}.patch`\n\n", plan.candidate_id));
    }
    if !plan.feedback.is_empty() {
        markdown.push_str("feedback focus:\n");
        for feedback in &plan.feedback {
            markdown.push_str(&format!("- {}\n", evolution_feedback_line(feedback)));
        }
        markdown.push('\n');
    }
    if let Some(patch) = &plan.suggested_patch {
        markdown.push_str("provider patch draft:\n");
        markdown.push_str("```diff\n");
        markdown.push_str(patch.trim());
        markdown.push_str("\n```\n\n");
    }
    markdown.push_str("guardrails:\n");
    for guardrail in &plan.guardrails {
        markdown.push_str(&format!("- {guardrail}\n"));
    }
    markdown.push_str("\nchecks:\n");
    for check in &plan.checks {
        markdown.push_str(&format!("- `{check}`\n"));
    }
    markdown.push_str("\nnext steps:\n");
    for step in &plan.next_steps {
        markdown.push_str(&format!("- {step}\n"));
    }
    markdown
}

fn evolution_actionable_check_status(status: &Status) -> bool {
    matches!(status, Status::Failed | Status::Partial)
}

fn evolution_actionable_feed_trace_status(status: &Status) -> bool {
    matches!(status, Status::Failed | Status::Partial)
}

fn check_history_evolution_detail(record: &CheckHistoryRecord) -> String {
    if !record.stderr.trim().is_empty() {
        record.stderr.clone()
    } else if !record.stdout.trim().is_empty() {
        record.stdout.clone()
    } else {
        record.command.clone()
    }
}

fn check_history_evolution_objective(record: &CheckHistoryRecord) -> String {
    let detail = check_history_evolution_detail(record);
    format!(
        "repair check #{} for {}: {}",
        record.index,
        record.tentacle_id,
        short_text(&one_line(&detail), 160)
    )
}

fn feed_trace_evolution_objective(trace: &FeedTraceRecord) -> String {
    let tentacle = trace.tentacle.as_deref().unwrap_or("unknown");
    let tool = trace.tool.as_deref().unwrap_or("unknown tool");
    format!(
        "repair Feed trace #{} for {} via {}: {}",
        trace.index,
        tentacle,
        tool,
        short_text(&one_line(&trace.summary), 160)
    )
}

pub fn render_authorized_apply_patch(plan: &EvolutionApplyPlan, apply_dir: &Path) -> String {
    if let Some(patch) = plan
        .suggested_patch
        .as_deref()
        .and_then(|patch| provider_patch_for_plan(plan, patch))
    {
        return patch;
    }
    if let Some(target) = resolve_existing_patch_target(&plan.target) {
        if let Some(patch) = render_existing_file_patch(plan, &target) {
            return patch;
        }
    }
    render_apply_note_patch(plan, apply_dir)
}

fn candidate_matches(candidate: &EvolutionPatchCandidate, value: &str) -> bool {
    candidate.id == value
        || candidate.surface_id == value
        || candidate.id.replace('-', "_") == value
        || candidate.surface_id.replace('_', "-") == value
}

fn evolution_candidate_outcome_score(
    candidate: &EvolutionPatchCandidate,
    proposal: &TentacleEvolutionProposal,
) -> (f32, usize) {
    let outcomes = proposal
        .previous_outcomes
        .iter()
        .filter(|outcome| candidate_matches(candidate, &outcome.candidate_id))
        .collect::<Vec<_>>();
    if outcomes.is_empty() {
        return (0.0, 0);
    }
    let weighted = outcomes
        .iter()
        .enumerate()
        .map(|(index, outcome)| {
            let weight = (index + 1) as f32;
            (outcome.score * weight, weight)
        })
        .fold((0.0, 0.0), |acc, item| (acc.0 + item.0, acc.1 + item.1));
    (weighted.0 / weighted.1.max(1.0), outcomes.len())
}

fn evolution_candidate_check_history_score(
    candidate: &EvolutionPatchCandidate,
    proposal: &TentacleEvolutionProposal,
) -> (f32, usize) {
    let matched = proposal
        .recent_check_history
        .iter()
        .filter(|record| candidate_matches_check_history(candidate, record))
        .collect::<Vec<_>>();
    if matched.is_empty() {
        return (0.0, 0);
    }
    let weighted = matched
        .iter()
        .enumerate()
        .map(|(index, record)| {
            let weight = (index + 1) as f32;
            let score = match record.status {
                Status::Failed => 0.45,
                Status::Partial => 0.2,
                Status::Satisfied => 0.05,
                Status::Unsupported => 0.0,
            };
            (score * weight, weight)
        })
        .fold((0.0, 0.0), |acc, item| (acc.0 + item.0, acc.1 + item.1));
    (weighted.0 / weighted.1.max(1.0), matched.len())
}

fn evolution_candidate_feed_trace_score(
    candidate: &EvolutionPatchCandidate,
    proposal: &TentacleEvolutionProposal,
) -> (f32, usize) {
    let matched = proposal
        .recent_feed_traces
        .iter()
        .filter(|trace| candidate_matches_feed_trace(candidate, trace))
        .collect::<Vec<_>>();
    if matched.is_empty() {
        return (0.0, 0);
    }
    let weighted = matched
        .iter()
        .enumerate()
        .map(|(index, trace)| {
            let weight = (index + 1) as f32;
            let score = match trace.status {
                Status::Failed => 0.35,
                Status::Partial => 0.18,
                Status::Satisfied => 0.05,
                Status::Unsupported => 0.0,
            };
            (score * weight, weight)
        })
        .fold((0.0, 0.0), |acc, item| (acc.0 + item.0, acc.1 + item.1));
    (weighted.0 / weighted.1.max(1.0), matched.len())
}

fn evolution_candidate_feedback_from_proposal(
    candidate: &EvolutionPatchCandidate,
    proposal: &TentacleEvolutionProposal,
) -> Vec<EvolutionPatchFeedback> {
    let mut feedback = Vec::new();
    if let Some(record) = proposal
        .recent_check_history
        .iter()
        .rev()
        .find(|record| candidate_matches_check_history(candidate, record))
    {
        feedback.push(feedback_from_check_history(record, None));
    }
    if let Some(trace) = proposal
        .recent_feed_traces
        .iter()
        .rev()
        .find(|trace| candidate_matches_feed_trace(candidate, trace))
    {
        feedback.push(feedback_from_feed_trace(trace, None));
    }
    feedback
}

fn clean_suggested_patch(patch: Option<String>) -> Option<String> {
    let patch = patch?;
    let patch = patch.trim();
    (!patch.is_empty()).then(|| format!("{patch}\n"))
}

fn candidate_matches_check_history(
    candidate: &EvolutionPatchCandidate,
    record: &CheckHistoryRecord,
) -> bool {
    let target = candidate
        .target
        .split('#')
        .next()
        .unwrap_or(&candidate.target);
    if !target.contains('*') && !target.is_empty() && record.command.contains(target) {
        return true;
    }
    if let Some(name) = Path::new(target).file_name().and_then(|name| name.to_str()) {
        if !name.is_empty() && name != "*" && record.command.contains(name) {
            return true;
        }
    }
    candidate.surface_id == "runtime_code"
        && record.status == Status::Failed
        && record.command.contains("tools/")
}

fn candidate_matches_feed_trace(
    candidate: &EvolutionPatchCandidate,
    trace: &FeedTraceRecord,
) -> bool {
    let Some(tool) = trace.tool.as_deref() else {
        return false;
    };
    target_mentions_tool(&candidate.target, tool)
}

fn target_mentions_tool(target: &str, tool: &str) -> bool {
    if target.contains(tool) {
        return true;
    }
    Path::new(target.split('#').next().unwrap_or(target))
        .file_stem()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == tool)
}

fn evolution_candidate_feedback(
    trace_tool: Option<(&ManifestTool, &FeedTraceRecord)>,
    check_tool: Option<(&ManifestTool, &CheckHistoryRecord)>,
) -> Vec<EvolutionPatchFeedback> {
    let mut feedback = Vec::new();
    if let Some((tool, record)) = check_tool {
        feedback.push(feedback_from_check_history(record, Some(tool)));
    }
    if let Some((tool, trace)) = trace_tool {
        feedback.push(feedback_from_feed_trace(trace, Some(tool)));
    }
    feedback
}

fn feedback_from_check_history(
    record: &CheckHistoryRecord,
    tool: Option<&ManifestTool>,
) -> EvolutionPatchFeedback {
    let summary = if record.stderr.trim().is_empty() {
        one_line(&record.stdout)
    } else {
        one_line(&record.stderr)
    };
    EvolutionPatchFeedback {
        kind: "check_history".to_string(),
        source_index: record.index,
        status: record.status.clone(),
        summary,
        command: Some(record.command.clone()),
        target: tool.map(|tool| tool.implementation.entrypoint.clone()),
    }
}

fn feedback_from_feed_trace(
    trace: &FeedTraceRecord,
    tool: Option<&ManifestTool>,
) -> EvolutionPatchFeedback {
    EvolutionPatchFeedback {
        kind: "feed_trace".to_string(),
        source_index: trace.index,
        status: trace.status.clone(),
        summary: one_line(&trace.summary),
        command: None,
        target: tool
            .map(|tool| tool.implementation.entrypoint.clone())
            .or_else(|| trace.tool.clone()),
    }
}

fn resolve_existing_patch_target(target: &str) -> Option<PathBuf> {
    let path = target.split('#').next().unwrap_or(target);
    if path.contains('*') {
        return resolve_wildcard_target(path);
    }
    let path = PathBuf::from(path);
    path.exists().then_some(path)
}

fn resolve_wildcard_target(value: &str) -> Option<PathBuf> {
    let path = Path::new(value);
    let parent = path.parent()?;
    let pattern = path.file_name()?.to_string_lossy();
    let (prefix, suffix) = pattern.split_once('*')?;
    let mut matches = fs::read_dir(parent)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy())
                .is_some_and(|name| name.starts_with(prefix) && name.ends_with(suffix))
        })
        .collect::<Vec<_>>();
    matches.sort();
    matches.into_iter().next()
}

fn render_existing_file_patch(plan: &EvolutionApplyPlan, target: &Path) -> Option<String> {
    let content = fs::read_to_string(target).ok()?;
    let comment = patch_comment_for(target, plan)?;
    let context = content.lines().take(4).collect::<Vec<_>>();
    if context.is_empty() {
        return None;
    }
    let insert_at = if context.first().is_some_and(|line| line.starts_with("#!")) {
        1
    } else {
        0
    };
    let display = patch_display_path(target);
    let mut patch = String::new();
    patch.push_str(&format!("diff --git a/{display} b/{display}\n"));
    patch.push_str(&format!("--- a/{display}\n"));
    patch.push_str(&format!("+++ b/{display}\n"));
    patch.push_str(&format!(
        "@@ -1,{} +1,{} @@\n",
        context.len(),
        context.len() + 1
    ));
    for (index, line) in context.iter().enumerate() {
        if index == insert_at {
            patch.push_str(&format!("+{comment}\n"));
        }
        patch.push_str(&format!(" {line}\n"));
    }
    if insert_at >= context.len() {
        patch.push_str(&format!("+{comment}\n"));
    }
    Some(patch)
}

fn provider_patch_for_plan(plan: &EvolutionApplyPlan, patch: &str) -> Option<String> {
    if plan
        .target
        .split('#')
        .next()
        .unwrap_or(&plan.target)
        .contains('*')
    {
        return None;
    }
    let target = resolve_existing_patch_target(&plan.target)?;
    let target = patch_display_path(&target);
    let paths = diff_paths(patch);
    if paths.is_empty() || paths.iter().any(|path| path != &target) {
        return None;
    }
    let patch = patch.trim();
    (!patch.is_empty()).then(|| format!("{patch}\n"))
}

fn diff_paths(patch: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for line in patch.lines() {
        if let Some(value) = line.strip_prefix("diff --git ") {
            for part in value.split_whitespace().take(2) {
                if let Some(path) = normalize_diff_path(part) {
                    push_unique(&mut paths, path);
                }
            }
        } else if let Some(value) = line.strip_prefix("--- ") {
            if let Some(path) = normalize_diff_path(value) {
                push_unique(&mut paths, path);
            }
        } else if let Some(value) = line.strip_prefix("+++ ") {
            if let Some(path) = normalize_diff_path(value) {
                push_unique(&mut paths, path);
            }
        }
    }
    paths
}

fn normalize_diff_path(value: &str) -> Option<String> {
    let value = value.trim().trim_matches('"');
    if value == "/dev/null" {
        return None;
    }
    value
        .strip_prefix("a/")
        .or_else(|| value.strip_prefix("b/"))
        .map(|path| path.to_string())
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn patch_comment_for(target: &Path, plan: &EvolutionApplyPlan) -> Option<String> {
    let note = format!(
        "Octopus evolution candidate {}: {}",
        plan.candidate_id,
        one_line(&plan.objective)
    );
    let extension = target
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    match extension {
        "sh" | "py" | "rb" | "toml" | "yaml" | "yml" => Some(format!("# {note}")),
        "js" | "ts" | "rs" | "go" | "java" | "c" | "cpp" | "h" => Some(format!("// {note}")),
        "md" | "html" => Some(format!("<!-- {note} -->")),
        _ => None,
    }
}

fn render_apply_note_patch(plan: &EvolutionApplyPlan, apply_dir: &Path) -> String {
    let note_path = apply_dir.join(format!("{}-note.md", plan.candidate_id));
    let display = patch_display_path(&note_path);
    let lines = [
        format!("# Authorized Evolution Candidate {}", plan.candidate_id),
        String::new(),
        format!("tentacle: {}", plan.tentacle_id),
        format!("target: {}", plan.target),
        format!("objective: {}", one_line(&plan.objective)),
        "status: ready_for_authorized_patch".to_string(),
    ];
    let mut patch = String::new();
    patch.push_str(&format!("diff --git a/{display} b/{display}\n"));
    patch.push_str("new file mode 100644\n");
    patch.push_str("--- /dev/null\n");
    patch.push_str(&format!("+++ b/{display}\n"));
    patch.push_str(&format!("@@ -0,0 +1,{} @@\n", lines.len()));
    for line in lines {
        patch.push_str(&format!("+{line}\n"));
    }
    patch
}

fn patch_display_path(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    for marker in ["tentacles/", "docs/", ".octopus/"] {
        if let Some(index) = value.find(marker) {
            return value[index..].to_string();
        }
    }
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| value)
}

fn one_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn evolution_feedback_line(feedback: &EvolutionPatchFeedback) -> String {
    let mut parts = vec![
        format!("{} #{}", feedback.kind, feedback.source_index),
        format!("{:?}", feedback.status),
    ];
    if let Some(target) = &feedback.target {
        parts.push(format!("target `{target}`"));
    }
    if let Some(command) = &feedback.command {
        parts.push(format!("command `{}`", one_line(command)));
    }
    if !feedback.summary.trim().is_empty() {
        parts.push(format!("summary {}", one_line(&feedback.summary)));
    }
    parts.join(" · ")
}

pub fn render_tentacle_evolution_proposal(proposal: &TentacleEvolutionProposal) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# Tentacle Evolution: {}\n\n",
        proposal.tentacle_id
    ));
    markdown.push_str(&format!("objective: {}\n", proposal.objective));
    markdown.push_str(&format!("generator: `{}`\n", proposal.generator));
    if !proposal.planner_summary.trim().is_empty() {
        markdown.push_str(&format!("planner: {}\n", proposal.planner_summary));
    }
    markdown.push_str(&format!("manifest: {}\n", proposal.manifest_path));
    markdown.push_str(&format!("brain: {}\n\n", proposal.brain_kind));
    markdown.push_str("## Current Brain Prompt\n\n");
    markdown.push_str("```text\n");
    markdown.push_str(&proposal.current_brain_prompt);
    markdown.push_str("\n```\n\n");
    markdown.push_str("## Editable Targets\n\n");
    for file in &proposal.files {
        markdown.push_str(&format!(
            "- `{}`: {} ({})\n",
            file.path, file.action, file.rationale
        ));
    }
    markdown.push_str("\n## Evolution Surfaces\n\n");
    for surface in &proposal.surfaces {
        markdown.push_str(&format!(
            "- `{}`: {} [{}]\n",
            surface.id,
            surface.description,
            surface.targets.join(", ")
        ));
    }
    markdown.push_str("\n## Constraints\n\n");
    for constraint in &proposal.constraints {
        markdown.push_str(&format!("- {constraint}\n"));
    }
    markdown.push_str("\n## Checks\n\n");
    for check in &proposal.checks {
        markdown.push_str(&format!("- `{check}`\n"));
    }
    if !proposal.previous_outcomes.is_empty() {
        markdown.push_str("\n## Previous Outcomes\n\n");
        for outcome in &proposal.previous_outcomes {
            markdown.push_str(&format!(
                "- `{}`: {:?} score={:.2} {}\n",
                outcome.candidate_id, outcome.status, outcome.score, outcome.summary
            ));
        }
    }
    if !proposal.recent_feed_traces.is_empty() {
        markdown.push_str("\n## Recent Feed Traces\n\n");
        for trace in &proposal.recent_feed_traces {
            let tentacle = trace.tentacle.as_deref().unwrap_or("unknown");
            let tool = trace.tool.as_deref().unwrap_or("unknown");
            let plan = trace.plan_source.as_deref().unwrap_or("unknown");
            markdown.push_str(&format!(
                "- #{} {}:{} -> {:?} via `{}/{}` plan={} evidence={} :: {}\n",
                trace.index,
                kind_key(&trace.need_kind),
                trace.need_query,
                trace.status,
                tentacle,
                tool,
                plan,
                trace.evidence_count,
                trace.summary
            ));
        }
    }
    if !proposal.recent_check_history.is_empty() {
        markdown.push_str("\n## Recent Check History\n\n");
        for record in &proposal.recent_check_history {
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
            markdown.push_str(&format!(
                "- #{} {label} {} `{}` code={:?} :: {}\n",
                record.index,
                check,
                record.command,
                record.code,
                one_line(&record.stderr)
            ));
        }
    }
    markdown.push_str("\n## Patch Candidates\n\n");
    for candidate in &proposal.patch_candidates {
        markdown.push_str(&format!(
            "- `{}`: {} -> `{}`\n",
            candidate.id, candidate.title, candidate.target
        ));
    }
    markdown.push_str("\n## Next Steps\n\n");
    for step in &proposal.next_steps {
        markdown.push_str(&format!("- {step}\n"));
    }
    markdown
}

pub fn render_tentacle_patch_candidates(proposal: &TentacleEvolutionProposal) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!("# Patch Candidates: {}\n\n", proposal.tentacle_id));
    markdown.push_str(&format!("objective: {}\n\n", proposal.objective));
    markdown.push_str(&format!("generator: `{}`\n\n", proposal.generator));
    for candidate in &proposal.patch_candidates {
        markdown.push_str(&format!("## {}. {}\n\n", candidate.id, candidate.title));
        markdown.push_str(&format!("surface: `{}`\n", candidate.surface_id));
        markdown.push_str(&format!("target: `{}`\n", candidate.target));
        markdown.push_str(&format!("rationale: {}\n\n", candidate.rationale));
        if !candidate.feedback.is_empty() {
            markdown.push_str("feedback:\n");
            for feedback in &candidate.feedback {
                markdown.push_str(&format!("- {}\n", evolution_feedback_line(feedback)));
            }
            markdown.push('\n');
        }
        markdown.push_str(&format!("draft: `{}`\n", candidate.draft.path));
        if candidate.suggested_patch.is_some() {
            markdown.push_str("provider patch: attached\n");
        }
        markdown.push_str(&format!("status: `{}`\n", candidate.draft.status));
        markdown.push_str(&format!(
            "authorization required: {}\n\n",
            candidate.draft.authorization_required
        ));
        markdown.push_str("change plan:\n");
        for step in &candidate.change_plan {
            markdown.push_str(&format!("- {step}\n"));
        }
        markdown.push_str("\nchecks:\n");
        for check in &candidate.checks {
            markdown.push_str(&format!("- `{check}`\n"));
        }
        markdown.push('\n');
    }
    markdown
}

pub fn render_tentacle_patch_drafts(proposal: &TentacleEvolutionProposal) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!("# Patch Drafts: {}\n\n", proposal.tentacle_id));
    markdown.push_str("These drafts are local review artifacts. They do not modify harness files until a user or authorized repo tentacle turns one into a patch.\n\n");
    for candidate in &proposal.patch_candidates {
        markdown.push_str(&format!(
            "- `{}` -> `{}` ({})\n",
            candidate.id, candidate.draft.path, candidate.draft.status
        ));
    }
    markdown
}

pub fn render_single_patch_draft(
    proposal: &TentacleEvolutionProposal,
    candidate: &EvolutionPatchCandidate,
) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!("# Patch Draft: {}\n\n", candidate.id));
    markdown.push_str(&format!("tentacle: `{}`\n", proposal.tentacle_id));
    markdown.push_str(&format!("objective: {}\n", proposal.objective));
    markdown.push_str(&format!("surface: `{}`\n", candidate.surface_id));
    markdown.push_str(&format!("target: `{}`\n", candidate.target));
    markdown.push_str(&format!("status: `{}`\n", candidate.draft.status));
    markdown.push_str(&format!(
        "authorization_required: {}\n",
        candidate.draft.authorization_required
    ));
    markdown.push_str(&format!("apply_hint: {}\n\n", candidate.draft.apply_hint));
    if !candidate.feedback.is_empty() {
        markdown.push_str("feedback focus:\n");
        for feedback in &candidate.feedback {
            markdown.push_str(&format!("- {}\n", evolution_feedback_line(feedback)));
        }
        markdown.push('\n');
    }
    if let Some(patch) = &candidate.suggested_patch {
        markdown.push_str("provider patch draft:\n");
        markdown.push_str("```diff\n");
        markdown.push_str(patch.trim());
        markdown.push_str("\n```\n\n");
    }
    markdown.push_str("diff intent:\n");
    for step in &candidate.change_plan {
        markdown.push_str(&format!("- {step}\n"));
    }
    markdown.push_str("\nchecks:\n");
    for check in &candidate.checks {
        markdown.push_str(&format!("- `{check}`\n"));
    }
    markdown
}

pub fn scaffold_tentacle(
    workspace_root: impl AsRef<Path>,
    tentacle_id: &str,
    runtime: Option<&str>,
) -> Result<TentacleScaffold, String> {
    let tentacle_id = validate_tentacle_id(tentacle_id)?;
    let runtime = normalize_runtime(runtime)?;

    let tentacles_root = workspace_root.as_ref().join("tentacles");
    let directory = tentacles_root.join(&tentacle_id);
    if directory.exists() {
        return Err(format!("tentacle already exists: {}", directory.display()));
    }
    let tools_dir = directory.join("tools");
    fs::create_dir_all(&tools_dir).map_err(|error| error.to_string())?;
    let schema_path = tentacles_root.join("tentacle.schema.json");
    if !schema_path.exists() {
        fs::write(
            &schema_path,
            include_str!("../../../tentacles/tentacle.schema.json"),
        )
        .map_err(|error| error.to_string())?;
    }

    let (entrypoint, tool_path) = match runtime.as_str() {
        "python" => {
            let path = tools_dir.join("feed.py");
            fs::write(&path, python_scaffold_tool()).map_err(|error| error.to_string())?;
            make_executable(&path)?;
            ("tools/feed.py".to_string(), Some(path))
        }
        "node" => {
            let path = tools_dir.join("feed.js");
            fs::write(&path, node_scaffold_tool()).map_err(|error| error.to_string())?;
            make_executable(&path)?;
            ("tools/feed.js".to_string(), Some(path))
        }
        "shell" => {
            let path = tools_dir.join("feed.sh");
            fs::write(&path, shell_scaffold_tool()).map_err(|error| error.to_string())?;
            make_executable(&path)?;
            ("tools/feed.sh".to_string(), Some(path))
        }
        "http" => ("https://example.com/octopus-feed".to_string(), None),
        _ => ("tools/feed".to_string(), None),
    };

    let manifest_path = directory.join("manifest.json");
    let manifest = scaffold_manifest(&tentacle_id, &runtime, &entrypoint);
    fs::write(
        &manifest_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())?
        ),
    )
    .map_err(|error| error.to_string())?;

    let mut next_steps = vec![
        format!("review {}", manifest_path.display()),
        format!("octopus manifests {}", tentacles_root.display()),
        format!("octopus --state /tmp/octopus.json install {tentacle_id}"),
    ];
    if tool_path.is_none() && runtime != "http" {
        next_steps.insert(
            1,
            format!(
                "add executable {} that reads {OCTOPUS_JSON_CONTRACT} from stdin",
                directory.join("tools/feed").display()
            ),
        );
    }
    Ok(TentacleScaffold {
        tentacle_id,
        runtime,
        directory: directory.to_string_lossy().to_string(),
        manifest_path: manifest_path.to_string_lossy().to_string(),
        tool_path: tool_path.map(|path| path.to_string_lossy().to_string()),
        next_steps,
    })
}

fn normalize_runtime(runtime: Option<&str>) -> Result<String, String> {
    let value = runtime.unwrap_or("python").trim().to_ascii_lowercase();
    let value = match value.as_str() {
        "javascript" => "node".to_string(),
        "bash" => "shell".to_string(),
        _ => value,
    };
    if value.is_empty() {
        return Err("runtime cannot be empty".to_string());
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-' || character == '_')
    {
        return Err("runtime can only contain ASCII letters, numbers, '-' and '_'".to_string());
    }
    Ok(value)
}

fn validate_tentacle_id(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("tentacle id cannot be empty".to_string());
    }
    if value.starts_with('.') || value.contains("..") {
        return Err("tentacle id cannot contain path traversal".to_string());
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-' || character == '_')
    {
        return Err("tentacle id can only contain ASCII letters, numbers, '-' and '_'".to_string());
    }
    Ok(value.to_string())
}

fn scaffold_manifest(tentacle_id: &str, runtime: &str, entrypoint: &str) -> serde_json::Value {
    serde_json::json!({
        "$schema": "../tentacle.schema.json",
        "schema_version": "0.0.1",
        "id": tentacle_id,
        "name": tentacle_title(tentacle_id),
        "description": "User-owned code-as-harness tentacle.",
        "brain": {
            "kind": "llm",
            "model": null,
            "prompt": "Map a cognitive Need to this tentacle's tool metadata and implementation code. Execute only the needed feed step and return compact evidence.",
            "feedback_contract": "Return status, evidence, touched files or remote calls, and the next useful Need. Do not return hidden reasoning."
        },
        "skills": [
            {
                "id": format!("{tentacle_id}-feed"),
                "description": "Feed observe, execute, and verify Needs through custom harness code.",
                "needs": ["observe", "execute", "verify"]
            }
        ],
        "tools": [
            {
                "id": "feed",
                "description": "Run this tentacle's custom harness implementation.",
                "input": "octopus-json-v1 tool call",
                "output": "structured feedback",
                "implementation": {
                    "kind": runtime,
                    "entrypoint": entrypoint,
                    "contract": OCTOPUS_JSON_CONTRACT
                }
            }
        ],
        "evolution": {
            "editable": ["manifest.json", "brain.prompt", "tools/*"],
            "surfaces": [
                {
                    "id": "brain_prompt",
                    "description": "Tool-side LLM prompt and feedback contract.",
                    "targets": ["brain.prompt", "brain.feedback_contract"]
                },
                {
                    "id": "tool_meta",
                    "description": "Tool descriptions, inputs, outputs, and call contracts.",
                    "targets": ["tools[].description", "tools[].input", "tools[].output", "tools[].permission", "tools[].implementation.contract"]
                },
                {
                    "id": "runtime_code",
                    "description": "Executable harness code owned by the tentacle.",
                    "targets": ["tools/*"]
                },
                {
                    "id": "evolution_policy",
                    "description": "Checks, constraints, and the declared edit surface.",
                    "targets": ["evolution.checks", "evolution.constraints", "evolution.surfaces"]
                }
            ],
            "checks": ["python3 -m json.tool manifest.json > /dev/null"],
            "constraints": ["Keep octopus-json-v1 input and compact feedback output stable."]
        }
    })
}

fn tentacle_title(value: &str) -> String {
    value
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(capitalize_ascii)
        .collect::<Vec<_>>()
        .join(" ")
}

fn python_scaffold_tool() -> &'static str {
    r#"#!/usr/bin/env python3
import json
import sys

call = json.load(sys.stdin)
need = call["need"]
tool = call["tool"]
tentacle = call["tentacle"]

print(json.dumps({
    "status": "satisfied",
    "output": f"{need['kind']} feed for {need['query']}",
    "metadata": {
        "tentacle": tentacle["id"],
        "tool": tool["id"],
        "runtime": tool["runtime"],
    },
}))
"#
}

fn node_scaffold_tool() -> &'static str {
    r#"#!/usr/bin/env node
const fs = require("fs");

const call = JSON.parse(fs.readFileSync(0, "utf8"));
const need = call.need;
const tool = call.tool;
const tentacle = call.tentacle;

process.stdout.write(JSON.stringify({
  status: "satisfied",
  output: `${need.kind} feed for ${need.query}`,
  metadata: {
    tentacle: tentacle.id,
    tool: tool.id,
    runtime: tool.runtime
  }
}) + "\n");
"#
}

fn shell_scaffold_tool() -> &'static str {
    r#"#!/usr/bin/env bash
set -euo pipefail

payload="$(cat)"
OCTOPUS_PAYLOAD="$payload" python3 - <<'PY'
import json
import os

call = json.loads(os.environ["OCTOPUS_PAYLOAD"])
need = call["need"]
tool = call["tool"]
tentacle = call["tentacle"]

print(json.dumps({
    "status": "satisfied",
    "output": f"{need['kind']} feed for {need['query']}",
    "metadata": {
        "tentacle": tentacle["id"],
        "tool": tool["id"],
        "runtime": tool["runtime"],
    },
}))
PY
"#
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

pub fn default_tentacle_profiles() -> Vec<TentacleProfile> {
    vec![
        TentacleProfile {
            id: "research".to_string(),
            name: "Research Tentacle".to_string(),
            description: "Verifies, compares, retrieves, and returns compact evidence.".to_string(),
            brain: llm_brain(
                "Evidence planner",
                "Turn a Need into source checks, comparison steps, and compact evidence.",
            ),
            skills: vec![SkillManifest {
                id: "verify".to_string(),
                name: "Verify".to_string(),
                description: "Check claims against available sources.".to_string(),
                needs: vec![NeedKind::Verify, NeedKind::Compare, NeedKind::Observe],
                tools: vec![
                    "search".to_string(),
                    "browser".to_string(),
                    "citation".to_string(),
                ],
            }],
            tools: vec![
                tool_meta("search", "Find candidate sources.", "adapter", "search"),
                tool_meta("browser", "Inspect source pages.", "adapter", "browser"),
                tool_meta("citation", "Return compact citations.", "adapter", "citation"),
            ],
            evolution: evolution_policy(&["cargo test"], &["Return evidence, not hidden chain of thought."]),
            llm_ready: true,
        },
        TentacleProfile {
            id: "code".to_string(),
            name: "Code Tentacle".to_string(),
            description: "Inspects repositories, edits code, runs tests, and summarizes patches."
                .to_string(),
            brain: llm_brain(
                "Code planner",
                "Map a Need onto repository reads, edits, verification, and a concise patch summary.",
            ),
            skills: vec![SkillManifest {
                id: "harness-work".to_string(),
                name: "Harness Work".to_string(),
                description:
                    "Execute implementation work without loading the main brain with tools."
                        .to_string(),
                needs: vec![NeedKind::Execute, NeedKind::Reproduce, NeedKind::Verify],
                tools: vec![
                    "git".to_string(),
                    "shell".to_string(),
                    "test-runner".to_string(),
                ],
            }],
            tools: vec![
                tool_meta("git", "Inspect versioned state.", "adapter", "git"),
                tool_meta("shell", "Run local commands.", "adapter", "shell"),
                tool_meta("test-runner", "Verify changes.", "adapter", "test-runner"),
            ],
            evolution: evolution_policy(
                &["cargo test", "python -m unittest discover -s tests -q"],
                &["Keep edits scoped to the requested harness work."],
            ),
            llm_ready: true,
        },
        TentacleProfile {
            id: "memory".to_string(),
            name: "Memory Heart".to_string(),
            description: "Memory beat for remember, recall, forget, and compaction.".to_string(),
            brain: rule_brain(
                "Memory policy",
                "Store, recall, forget, and compact context outside the main brain.",
            ),
            skills: vec![SkillManifest {
                id: "memory".to_string(),
                name: "Memory".to_string(),
                description: "Manage long-running context outside the brain.".to_string(),
                needs: vec![NeedKind::Remember, NeedKind::Recall, NeedKind::Forget],
                tools: vec!["memory-store".to_string()],
            }],
            tools: vec![tool_meta(
                "memory-store",
                "Persist and retrieve compact memories.",
                "native",
                "HarnessState.memory",
            )],
            evolution: evolution_policy(&["cargo test"], &["Do not store secrets or noisy transcripts."]),
            llm_ready: false,
        },
        TentacleProfile {
            id: "visual".to_string(),
            name: "Color Pet Layer".to_string(),
            description: "Shows heartbeat, memory, harness, blocked, and success state as a color-changing pixel pet layer.".to_string(),
            brain: rule_brain(
                "Visual state translator",
                "Convert Feedback and harness status into user-visible pet state without changing the kernel contract.",
            ),
            skills: vec![SkillManifest {
                id: "color-change".to_string(),
                name: "Color Change".to_string(),
                description: "Render visual feedback without changing the core agent loop."
                    .to_string(),
                needs: vec![NeedKind::Observe],
                tools: vec!["status-pet".to_string()],
            }],
            tools: vec![tool_meta(
                "status-pet",
                "Render color-changing Octopus status.",
                "static-html",
                "docs/pet.html",
            )],
            evolution: evolution_policy(
                &["grep -q pixel-pet docs/pet.html"],
                &["Keep visual state outside Need, Feed, and Feedback."],
            ),
            llm_ready: false,
        },
        TentacleProfile {
            id: "repo-maintainer".to_string(),
            name: "Repo Maintainer Tentacle".to_string(),
            description: "Future self-iteration profile for branches, CI, and pull requests."
                .to_string(),
            brain: llm_brain(
                "Self-iteration planner",
                "Inspect repo health, propose small changes, and prepare PR-ready feedback after OAuth grant.",
            ),
            skills: vec![SkillManifest {
                id: "self-iteration".to_string(),
                name: "Self Iteration".to_string(),
                description:
                    "Inspect project health and prepare small pull requests after OAuth grant."
                        .to_string(),
                needs: vec![NeedKind::Observe, NeedKind::Verify, NeedKind::Execute],
                tools: vec![
                    "tentacles/repo-maintainer/tools/inspect_repo.sh".to_string(),
                    "tentacles/repo-maintainer/tools/github_status.sh".to_string(),
                    "tentacles/repo-maintainer/tools/patch_queue.sh".to_string(),
                    "tentacles/repo-maintainer/tools/draft_pr.sh".to_string(),
                    "tentacles/repo-maintainer/tools/publish_pr.sh".to_string(),
                ],
            }],
            tools: vec![
                tool_meta(
                    "inspect_repo",
                    "Inspect local repo health.",
                    "shell",
                    "tentacles/repo-maintainer/tools/inspect_repo.sh",
                ),
                tool_meta(
                    "github_status",
                    "Read GitHub issues and workflow runs when gh is available.",
                    "shell",
                    "tentacles/repo-maintainer/tools/github_status.sh",
                ),
                tool_meta(
                    "patch_queue",
                    "Generate candidate patch queue from GitHub and goal signals.",
                    "shell",
                    "tentacles/repo-maintainer/tools/patch_queue.sh",
                ),
                tool_meta(
                    "draft_pr",
                    "Write branch plan and pull request draft data.",
                    "shell",
                    "tentacles/repo-maintainer/tools/draft_pr.sh",
                ),
                tool_meta(
                    "publish_pr",
                    "Publish a draft pull request through gh after an explicit OAuth grant.",
                    "shell",
                    "tentacles/repo-maintainer/tools/publish_pr.sh",
                ),
            ],
            evolution: evolution_policy(
                &[
                    "tentacles/repo-maintainer/tools/inspect_repo.sh .",
                    "tentacles/repo-maintainer/tools/github_status.sh dangoZhang/Octopus",
                    "tentacles/repo-maintainer/tools/patch_queue.sh $(mktemp -d) dangoZhang/Octopus improve-usability",
                    "tentacles/repo-maintainer/tools/draft_pr.sh $(mktemp -d) dangoZhang/Octopus improve-usability",
                    "OCTOPUS_PR_DRY_RUN=1 tentacles/repo-maintainer/tools/publish_pr.sh $(mktemp -d) dangoZhang/Octopus octopus/improve-usability 'Improve usability' README.md",
                ],
                &["Wait for explicit OAuth/user grant before publishing branch or PR work."],
            ),
            llm_ready: true,
        },
        TentacleProfile {
            id: "json-feed".to_string(),
            name: "JSON Feed Runtime Seed".to_string(),
            description: "Python octopus-json-v1 seed runtime for structured non-shell feedback."
                .to_string(),
            brain: llm_brain(
                "JSON feed planner",
                "Map a Need onto a compact Python JSON feed and return structured evidence.",
            ),
            skills: vec![SkillManifest {
                id: "json-feed".to_string(),
                name: "JSON Feed".to_string(),
                description: "Feed observe, verify, compare, and reproduce Needs through a non-shell JSON runtime.".to_string(),
                needs: vec![
                    NeedKind::Observe,
                    NeedKind::Verify,
                    NeedKind::Compare,
                    NeedKind::Reproduce,
                ],
                tools: vec!["tentacles/json-feed/tools/feed.py".to_string()],
            }],
            tools: vec![tool_meta_with_contract(
                "feed",
                "Consume octopus-json-v1 and return structured feedback.",
                "python",
                "tentacles/json-feed/tools/feed.py",
                OCTOPUS_JSON_CONTRACT,
            )],
            evolution: evolution_policy(
                &["python3 -m py_compile tentacles/json-feed/tools/feed.py"],
                &["Keep octopus-json-v1 input and compact feedback output stable."],
            ),
            llm_ready: true,
        },
        TentacleProfile {
            id: "swe-agent".to_string(),
            name: "SWE Agent Tentacle".to_string(),
            description: "Code-as-harness repo tools for inspect, patch, and test workflows.".to_string(),
            brain: llm_brain(
                "SWE harness planner",
                "Choose repo tools from metadata, execute implementation code, and return patch/test evidence.",
            ),
            skills: vec![SkillManifest {
                id: "swe-workflow".to_string(),
                name: "SWE Workflow".to_string(),
                description: "Read/edit files, inspect a repository, prepare patches, and run tests through editable tool adapters.".to_string(),
                needs: vec![NeedKind::Observe, NeedKind::Execute, NeedKind::Verify],
                tools: vec![
                    "tentacles/swe-agent/tools/read.sh".to_string(),
                    "tentacles/swe-agent/tools/edit.sh".to_string(),
                    "tentacles/swe-agent/tools/inspect_repo.sh".to_string(),
                    "tentacles/swe-agent/tools/write_patch.sh".to_string(),
                    "tentacles/swe-agent/tools/run_tests.sh".to_string(),
                ],
            }],
            tools: vec![
                tool_meta("read", "Read a safe, line-numbered slice of a workspace file.", "shell", "tentacles/swe-agent/tools/read.sh"),
                tool_meta("edit", "Apply one scoped text replacement.", "shell", "tentacles/swe-agent/tools/edit.sh"),
                tool_meta("inspect_repo", "Summarize repo state and project type.", "shell", "tentacles/swe-agent/tools/inspect_repo.sh"),
                tool_meta("write_patch", "Write and optionally apply a patch.", "shell", "tentacles/swe-agent/tools/write_patch.sh"),
                tool_meta("run_tests", "Run detected project tests.", "shell", "tentacles/swe-agent/tools/run_tests.sh"),
            ],
            evolution: evolution_policy(
                &[
                    "tentacles/swe-agent/tools/read.sh README.md 1 2",
                    "cargo test",
                ],
                &["Keep path safety checks unless a stronger sandbox replaces them."],
            ),
            llm_ready: true,
        },
        TentacleProfile {
            id: "computer-use-agent".to_string(),
            name: "Computer Use Tentacle".to_string(),
            description: "Code-as-harness local UI tools for screenshots and desktop workflows.".to_string(),
            brain: llm_brain(
                "Computer-use planner",
                "Translate observation or execution Needs into MCP, local UI, shell, or URL actions.",
            ),
            skills: vec![SkillManifest {
                id: "computer-use".to_string(),
                name: "Computer Use".to_string(),
                description: "Use MCP, bash, URLs, screenshots, and desktop environment probes.".to_string(),
                needs: vec![NeedKind::Observe, NeedKind::Execute],
                tools: vec![
                    "tentacles/computer-use-agent/tools/mcp.sh".to_string(),
                    "tentacles/computer-use-agent/tools/bash.sh".to_string(),
                    "tentacles/computer-use-agent/tools/screenshot.sh".to_string(),
                    "tentacles/computer-use-agent/tools/open_url.sh".to_string(),
                    "tentacles/computer-use-agent/tools/browser_status.sh".to_string(),
                    "tentacles/computer-use-agent/tools/window_status.sh".to_string(),
                    "tentacles/computer-use-agent/tools/clipboard_read.sh".to_string(),
                    "tentacles/computer-use-agent/tools/clipboard_write.sh".to_string(),
                    "tentacles/computer-use-agent/tools/describe_screen.sh".to_string(),
                ],
            }],
            tools: vec![
                tool_meta_with_permission("mcp", "Call an MCP tool through a configured JSON-RPC client adapter.", "mcp", "tentacles/computer-use-agent/tools/mcp.sh", tool_permission("octopus", "tool:computer-use-agent", &["tool:mcp"], "external tool bridge")),
                tool_meta_with_permission("bash", "Write and run a local shell script.", "shell", "tentacles/computer-use-agent/tools/bash.sh", tool_permission("octopus", "tool:computer-use-agent", &["tool:execute"], "local command execution")),
                tool_meta_with_permission("screenshot", "Capture the current screen.", "shell", "tentacles/computer-use-agent/tools/screenshot.sh", tool_permission("octopus", "tool:computer-use-agent", &["tool:observe"], "screen capture")),
                tool_meta_with_permission("open_url", "Open a URL in the local desktop.", "shell", "tentacles/computer-use-agent/tools/open_url.sh", tool_permission("octopus", "tool:computer-use-agent", &["tool:ui"], "user-visible desktop action")),
                tool_meta_with_permission("browser_status", "Inspect local browser availability and current tab metadata.", "shell", "tentacles/computer-use-agent/tools/browser_status.sh", tool_permission("octopus", "tool:computer-use-agent", &["tool:observe"], "browser tab metadata")),
                tool_meta_with_permission("window_status", "Inspect front application, active window title, display session, and desktop process hints.", "shell", "tentacles/computer-use-agent/tools/window_status.sh", tool_permission("octopus", "tool:computer-use-agent", &["tool:observe"], "front window metadata")),
                tool_meta_with_permission("clipboard_read", "Read current clipboard text.", "shell", "tentacles/computer-use-agent/tools/clipboard_read.sh", tool_permission("octopus", "tool:computer-use-agent", &["tool:observe"], "clipboard text observation")),
                tool_meta_with_permission("clipboard_write", "Write provided text into the local clipboard.", "shell", "tentacles/computer-use-agent/tools/clipboard_write.sh", tool_permission("octopus", "tool:computer-use-agent", &["tool:ui"], "clipboard modification")),
                tool_meta("describe_screen", "Return lightweight desktop context.", "shell", "tentacles/computer-use-agent/tools/describe_screen.sh"),
            ],
            evolution: evolution_policy(
                &[
                    "tentacles/computer-use-agent/tools/window_status.sh | python3 -m json.tool > /dev/null",
                    "tentacles/computer-use-agent/tools/describe_screen.sh",
                    "tentacles/computer-use-agent/tools/browser_status.sh | python3 -m json.tool > /dev/null",
                    "OCTOPUS_CLIPBOARD_DRY_RUN=1 tentacles/computer-use-agent/tools/clipboard_read.sh | python3 -m json.tool > /dev/null",
                    "printf octopus | OCTOPUS_CLIPBOARD_DRY_RUN=1 tentacles/computer-use-agent/tools/clipboard_write.sh | python3 -m json.tool > /dev/null",
                ],
                &["Ask for user-visible actions only when the product flow grants it."],
            ),
            llm_ready: true,
        },
        TentacleProfile {
            id: "bash-only".to_string(),
            name: "Write-and-Run Tentacle".to_string(),
            description: "Agent tool-combo seed that stores generated actions as auditable scripts.".to_string(),
            brain: llm_brain(
                "Shell-only planner",
                "Represent each action as a transparent script before execution.",
            ),
            skills: vec![SkillManifest {
                id: "write-and-run".to_string(),
                name: "Write And Run".to_string(),
                description: "Represent execution as editable scripts under .octopus/harness.".to_string(),
                needs: vec![NeedKind::Execute, NeedKind::Reproduce, NeedKind::Verify],
                tools: vec!["tentacles/bash-only/tools/write_and_run.sh".to_string()],
            }],
            tools: vec![tool_meta_with_permission(
                "write_and_run",
                "Write stdin to an auditable script and execute it.",
                "shell",
                "tentacles/bash-only/tools/write_and_run.sh",
                tool_permission("octopus", "tool:bash-only", &["tool:execute"], "generated script execution"),
            )],
            evolution: evolution_policy(
                &["printf 'echo ok\\n' | tentacles/bash-only/tools/write_and_run.sh $(mktemp -d)"],
                &["Prefer generated scripts that are readable and replayable."],
            ),
            llm_ready: true,
        },
    ]
}

fn llm_brain(description: &str, prompt: &str) -> TentacleBrain {
    TentacleBrain {
        kind: "llm".to_string(),
        description: description.to_string(),
        model: None,
        prompt: prompt.to_string(),
    }
}

fn rule_brain(description: &str, prompt: &str) -> TentacleBrain {
    TentacleBrain {
        kind: "rule".to_string(),
        description: description.to_string(),
        model: None,
        prompt: prompt.to_string(),
    }
}

fn tool_meta(id: &str, description: &str, kind: &str, entrypoint: &str) -> ToolMetadata {
    ToolMetadata {
        id: id.to_string(),
        description: description.to_string(),
        implementation: ToolImplementation {
            kind: kind.to_string(),
            entrypoint: entrypoint.to_string(),
            contract: None,
        },
        permission: None,
    }
}

fn tool_meta_with_contract(
    id: &str,
    description: &str,
    kind: &str,
    entrypoint: &str,
    contract: &str,
) -> ToolMetadata {
    let mut meta = tool_meta(id, description, kind, entrypoint);
    meta.implementation.contract = Some(contract.to_string());
    meta
}

fn tool_meta_with_permission(
    id: &str,
    description: &str,
    kind: &str,
    entrypoint: &str,
    permission: ToolPermission,
) -> ToolMetadata {
    let mut meta = tool_meta(id, description, kind, entrypoint);
    meta.permission = Some(permission);
    meta
}

fn tool_permission(
    provider: &str,
    scope: &str,
    permissions: &[&str],
    reason: &str,
) -> ToolPermission {
    ToolPermission {
        provider: provider.to_string(),
        scope: scope.to_string(),
        permissions: permissions
            .iter()
            .map(|permission| permission.to_string())
            .collect(),
        reason: Some(reason.to_string()),
    }
}

fn evolution_policy(checks: &[&str], constraints: &[&str]) -> EvolutionPolicy {
    EvolutionPolicy {
        editable: vec![
            "manifest.json".to_string(),
            "brain.prompt".to_string(),
            "tools/*".to_string(),
        ],
        surfaces: default_evolution_surfaces(),
        checks: checks.iter().map(|item| item.to_string()).collect(),
        constraints: constraints.iter().map(|item| item.to_string()).collect(),
    }
}

fn default_evolution_surfaces() -> Vec<EvolutionSurface> {
    vec![
        EvolutionSurface {
            id: "brain_prompt".to_string(),
            description: "Tool-side LLM prompt and feedback contract.".to_string(),
            targets: vec![
                "brain.prompt".to_string(),
                "brain.feedback_contract".to_string(),
            ],
        },
        EvolutionSurface {
            id: "tool_meta".to_string(),
            description: "Tool descriptions, inputs, outputs, and call contracts.".to_string(),
            targets: vec![
                "tools[].description".to_string(),
                "tools[].input".to_string(),
                "tools[].output".to_string(),
                "tools[].permission".to_string(),
                "tools[].implementation.contract".to_string(),
            ],
        },
        EvolutionSurface {
            id: "runtime_code".to_string(),
            description: "Executable harness code owned by the tentacle.".to_string(),
            targets: vec!["tools/*".to_string()],
        },
        EvolutionSurface {
            id: "evolution_policy".to_string(),
            description: "Checks, constraints, and the declared edit surface.".to_string(),
            targets: vec![
                "evolution.checks".to_string(),
                "evolution.constraints".to_string(),
                "evolution.surfaces".to_string(),
            ],
        },
    ]
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

fn evolution_file_target(
    manifest_dir: &Path,
    target: &str,
    objective: &str,
) -> EvolutionFileTarget {
    match target {
        "brain.prompt" => EvolutionFileTarget {
            path: "manifest.json#brain.prompt".to_string(),
            action: "tighten the tentacle brain prompt".to_string(),
            rationale: format!("align tool-side planning with {objective}"),
        },
        "manifest.json" => EvolutionFileTarget {
            path: manifest_dir
                .join("manifest.json")
                .to_string_lossy()
                .to_string(),
            action: "review skills, tools, feedback contract, and evolution policy".to_string(),
            rationale: "keep prompt, metadata, code, checks, and constraints consistent"
                .to_string(),
        },
        value if value.contains('*') => EvolutionFileTarget {
            path: manifest_dir.join(value).to_string_lossy().to_string(),
            action: "inspect matching harness code before editing".to_string(),
            rationale: "wildcards require a narrow patch target".to_string(),
        },
        value => EvolutionFileTarget {
            path: manifest_dir.join(value).to_string_lossy().to_string(),
            action: "prepare a scoped harness edit".to_string(),
            rationale: format!("support {objective}"),
        },
    }
}

fn evolution_patch_candidates(
    manifest_dir: &Path,
    tentacle_id: &str,
    objective: &str,
    surfaces: &[EvolutionSurface],
    checks: &[String],
    tools: &[ManifestTool],
    feedback: EvolutionCandidateFeedback<'_>,
) -> Vec<EvolutionPatchCandidate> {
    surfaces
        .iter()
        .enumerate()
        .map(|(index, surface)| {
            let trace_tool = traced_tool_for_surface(surface, tools, feedback.feed_traces);
            let check_tool = checked_tool_for_surface(surface, tools, feedback.check_history);
            EvolutionPatchCandidate {
                id: evolution_candidate_id(index, &surface.id),
                surface_id: surface.id.clone(),
                title: evolution_candidate_title(&surface.id, tentacle_id),
                target: evolution_candidate_target_with_feedback(
                    manifest_dir,
                    surface,
                    trace_tool,
                    check_tool,
                ),
                rationale: evolution_candidate_rationale(
                    objective, surface, trace_tool, check_tool,
                ),
                feedback: evolution_candidate_feedback(trace_tool, check_tool),
                suggested_patch: None,
                change_plan: evolution_candidate_plan_with_feedback(
                    surface, trace_tool, check_tool,
                ),
                checks: evolution_candidate_checks(checks, surface, trace_tool, check_tool),
                draft: evolution_patch_draft(index, surface),
            }
        })
        .collect()
}

fn traced_tool_for_surface<'a>(
    surface: &EvolutionSurface,
    tools: &'a [ManifestTool],
    recent_feed_traces: &'a [FeedTraceRecord],
) -> Option<(&'a ManifestTool, &'a FeedTraceRecord)> {
    if surface.id != "runtime_code" {
        return None;
    }
    recent_feed_traces.iter().rev().find_map(|trace| {
        let tool_id = trace.tool.as_deref()?;
        tools
            .iter()
            .find(|tool| tool.id == tool_id)
            .map(|tool| (tool, trace))
    })
}

fn checked_tool_for_surface<'a>(
    surface: &EvolutionSurface,
    tools: &'a [ManifestTool],
    recent_check_history: &'a [CheckHistoryRecord],
) -> Option<(&'a ManifestTool, &'a CheckHistoryRecord)> {
    if surface.id != "runtime_code" {
        return None;
    }
    recent_check_history
        .iter()
        .rev()
        .filter(|record| record.status == Status::Failed)
        .find_map(|record| {
            tools
                .iter()
                .find(|tool| check_mentions_entrypoint(record, &tool.implementation.entrypoint))
                .map(|tool| (tool, record))
        })
}

fn check_mentions_entrypoint(record: &CheckHistoryRecord, entrypoint: &str) -> bool {
    let entrypoint = entrypoint.trim();
    if entrypoint.is_empty() {
        return false;
    }
    if record.command.contains(entrypoint) {
        return true;
    }
    Path::new(entrypoint)
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| record.command.contains(name))
}

fn evolution_candidate_target_with_feedback(
    manifest_dir: &Path,
    surface: &EvolutionSurface,
    trace_tool: Option<(&ManifestTool, &FeedTraceRecord)>,
    check_tool: Option<(&ManifestTool, &CheckHistoryRecord)>,
) -> String {
    if let Some((tool, _)) = trace_tool {
        if let Some(target) = trace_tool_entrypoint_target(manifest_dir, tool) {
            return target;
        }
    }
    if let Some((tool, _)) = check_tool {
        if let Some(target) = trace_tool_entrypoint_target(manifest_dir, tool) {
            return target;
        }
    }
    evolution_candidate_target(manifest_dir, surface)
}

fn trace_tool_entrypoint_target(manifest_dir: &Path, tool: &ManifestTool) -> Option<String> {
    let entrypoint = tool.implementation.entrypoint.trim();
    if entrypoint.is_empty()
        || entrypoint.contains("..")
        || entrypoint.starts_with("http://")
        || entrypoint.starts_with("https://")
    {
        return None;
    }
    let path = Path::new(entrypoint);
    if path.is_absolute() {
        return path.exists().then(|| path.to_string_lossy().to_string());
    }
    Some(manifest_dir.join(path).to_string_lossy().to_string())
}

fn evolution_candidate_rationale(
    objective: &str,
    surface: &EvolutionSurface,
    trace_tool: Option<(&ManifestTool, &FeedTraceRecord)>,
    check_tool: Option<(&ManifestTool, &CheckHistoryRecord)>,
) -> String {
    let base = format!("advance {objective} through {}", surface.description);
    let with_trace = if let Some((tool, trace)) = trace_tool {
        format!(
            "{base}; recent trace #{} used `{}` for {} `{}` and returned {} evidence items",
            trace.index,
            tool.id,
            kind_key(&trace.need_kind),
            trace.need_query,
            trace.evidence_count
        )
    } else {
        base
    };
    if let Some((tool, record)) = check_tool {
        return format!(
            "{with_trace}; failed check #{} points at `{}` with `{}`",
            record.index,
            tool.id,
            one_line(&record.command)
        );
    }
    with_trace
}

fn evolution_candidate_plan_with_feedback(
    surface: &EvolutionSurface,
    trace_tool: Option<(&ManifestTool, &FeedTraceRecord)>,
    check_tool: Option<(&ManifestTool, &CheckHistoryRecord)>,
) -> Vec<String> {
    let mut plan = evolution_candidate_plan(surface);
    if let Some((tool, trace)) = trace_tool {
        plan.insert(
            0,
            format!(
                "patch `{}` from trace #{} instead of a broad tools/* target",
                tool.implementation.entrypoint, trace.index
            ),
        );
        plan.insert(
            1,
            format!("preserve the observed `{}` Feed contract", tool.id),
        );
    }
    if let Some((tool, record)) = check_tool {
        plan.insert(
            0,
            format!(
                "repair failing check #{} around `{}` before broad harness edits",
                record.index, tool.implementation.entrypoint
            ),
        );
    }
    plan
}

fn evolution_candidate_checks(
    checks: &[String],
    surface: &EvolutionSurface,
    trace_tool: Option<(&ManifestTool, &FeedTraceRecord)>,
    check_tool: Option<(&ManifestTool, &CheckHistoryRecord)>,
) -> Vec<String> {
    let mut checks = if checks.is_empty() {
        vec!["cargo test".to_string()]
    } else {
        checks.to_vec()
    };
    if let Some((_, record)) = check_tool {
        if !checks.iter().any(|existing| existing == &record.command) {
            checks.insert(0, record.command.clone());
        }
    }
    if let Some((tool, trace)) = trace_tool {
        if let Some(check) = traced_tool_check(tool, trace) {
            if !checks
                .iter()
                .any(|existing| existing.contains(&tool.implementation.entrypoint))
            {
                checks.insert(0, check);
            }
        }
    }
    if surface.id == "evolution_policy" && !checks.iter().any(|check| check == "cargo test") {
        checks.push("cargo test".to_string());
    }
    checks
}

fn traced_tool_check(tool: &ManifestTool, trace: &FeedTraceRecord) -> Option<String> {
    let entrypoint = tool.implementation.entrypoint.trim();
    if entrypoint.is_empty()
        || entrypoint.contains("..")
        || entrypoint.starts_with("http://")
        || entrypoint.starts_with("https://")
    {
        return None;
    }
    let args = trace
        .need_query
        .split_whitespace()
        .map(shell_arg)
        .collect::<Vec<_>>();
    let command = shell_arg(entrypoint);
    if args.is_empty() {
        Some(command)
    } else {
        Some(format!("{command} {}", args.join(" ")))
    }
}

fn evolution_candidate_id(index: usize, surface_id: &str) -> String {
    format!("{:02}-{}", index + 1, surface_id.replace('_', "-"))
}

fn evolution_patch_draft(index: usize, surface: &EvolutionSurface) -> EvolutionPatchDraft {
    let id = evolution_candidate_id(index, &surface.id);
    EvolutionPatchDraft {
        path: format!("patches/{id}.patch.md"),
        status: "draft_pending_authorization".to_string(),
        authorization_required: true,
        apply_hint: "review the draft, create a narrow harness patch, run listed checks, then commit through an explicit grant".to_string(),
    }
}

fn evolution_candidate_title(surface_id: &str, tentacle_id: &str) -> String {
    match surface_id {
        "brain_prompt" => format!("Retune {tentacle_id} tool-side thinking"),
        "tool_meta" => format!("Tighten {tentacle_id} tool metadata"),
        "runtime_code" => format!("Improve {tentacle_id} runtime harness code"),
        "evolution_policy" => format!("Strengthen {tentacle_id} evolution policy"),
        value => format!("Improve {tentacle_id} {value}"),
    }
}

fn evolution_candidate_target(manifest_dir: &Path, surface: &EvolutionSurface) -> String {
    let target = surface
        .targets
        .first()
        .map(String::as_str)
        .unwrap_or("manifest.json");
    if target.starts_with("brain.")
        || target.starts_with("tools[]")
        || target.starts_with("evolution.")
    {
        return format!(
            "{}#{}",
            manifest_dir.join("manifest.json").to_string_lossy(),
            target
        );
    }
    manifest_dir.join(target).to_string_lossy().to_string()
}

fn evolution_candidate_plan(surface: &EvolutionSurface) -> Vec<String> {
    match surface.id.as_str() {
        "brain_prompt" => vec![
            "rewrite the tentacle prompt around one Need-to-Feed responsibility".to_string(),
            "keep the feedback contract compact and evidence-first".to_string(),
        ],
        "tool_meta" => vec![
            "make tool inputs and outputs specific enough for tool-side planning".to_string(),
            "declare octopus-json-v1 when the runtime consumes the full Need envelope".to_string(),
        ],
        "runtime_code" => vec![
            "patch only the runtime adapter needed for this objective".to_string(),
            "preserve path safety, exit status, and compact evidence output".to_string(),
        ],
        "evolution_policy" => vec![
            "add the smallest check that proves the changed surface works".to_string(),
            "keep constraints aligned with user-visible safety boundaries".to_string(),
        ],
        _ => vec![
            "inspect the listed targets before editing".to_string(),
            "return a narrow harness patch with matching checks".to_string(),
        ],
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
        "PYTHONPATH=src python -m unittest discover -s tests -q".to_string(),
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

fn read_args(query: &str) -> Vec<String> {
    let mut args = query
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    if args.is_empty() {
        args.push("README.md".to_string());
    }
    args
}

fn manifest_plan_reason(need: &Need, tool: &InstalledToolRef) -> String {
    if tool.description.is_empty() {
        return format!("selected {} for {}", tool.id, kind_key(&need.kind));
    }
    format!(
        "selected {} for {}: {}",
        tool.id,
        kind_key(&need.kind),
        tool.description
    )
}

fn is_desktop_observe(query: &str) -> bool {
    let query = query.to_lowercase();
    [
        "screen",
        "desktop",
        "window",
        "browser",
        "url",
        "ui",
        "tab",
        "chrome",
        "safari",
        "firefox",
        "edge",
        "brave",
        "clipboard",
        "copy",
        "paste",
    ]
    .iter()
    .any(|word| query.contains(word))
}

fn is_clipboard_query(query: &str) -> bool {
    let query = query.to_lowercase();
    ["clipboard", "copy", "paste"]
        .iter()
        .any(|word| query.contains(word))
}

fn is_browser_observe(query: &str) -> bool {
    let query = query.to_lowercase();
    [
        "browser", "tab", "url", "chrome", "safari", "firefox", "edge", "brave",
    ]
    .iter()
    .any(|word| query.contains(word))
}

fn is_desktop_execute(query: &str) -> bool {
    let query = query.to_lowercase();
    [
        "screen",
        "desktop",
        "window",
        "browser",
        "url",
        "ui",
        "tab",
        "chrome",
        "safari",
        "firefox",
        "edge",
        "brave",
        "open ",
        "click",
        "type",
        "mcp",
        "screenshot",
        "clipboard",
        "copy",
        "paste",
    ]
    .iter()
    .any(|word| query.contains(word))
}

fn is_visual_observe(query: &str) -> bool {
    let query = query.to_lowercase();
    [
        "pet",
        "status",
        "color",
        "visual",
        "heartbeat",
        "memory",
        "harness",
        "blocked",
        "success",
    ]
    .iter()
    .any(|word| query.contains(word))
}

fn mcp_args(query: &str) -> Vec<String> {
    let mut parts = query
        .split_whitespace()
        .take(3)
        .map(str::to_string)
        .collect::<Vec<_>>();
    while parts.len() < 3 {
        parts.push(match parts.len() {
            0 => "default".to_string(),
            1 => "call".to_string(),
            _ => "{}".to_string(),
        });
    }
    parts
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
    if results
        .iter()
        .all(|result| result.status == Status::Satisfied)
    {
        Status::Satisfied
    } else if results
        .iter()
        .any(|result| result.status == Status::Satisfied)
    {
        Status::Partial
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
    }

    #[test]
    fn evolution_recommendation_uses_scored_outcomes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();
        state.record_evolution_outcome(
            "swe-agent",
            "03-runtime-code",
            Status::Satisfied,
            "runtime evidence improved",
        );
        let proposal =
            propose_tentacle_evolution_with_state(root, "swe-agent", "improve feed", &state)
                .unwrap();

        let recommendation = recommend_tentacle_evolution_apply(&proposal, &state).unwrap();

        assert_eq!(recommendation.candidate_id, "03-runtime-code");
        assert_eq!(recommendation.surface_id, "runtime_code");
        assert_eq!(recommendation.outcome_count, 1);
        assert!(recommendation.recommendation_score > 0.0);
        assert_eq!(recommendation.apply.status, "needs_authorization");
    }

    #[test]
    fn evolution_recommendation_avoids_failed_candidates() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();
        state.record_evolution_outcome(
            "swe-agent",
            "03-runtime-code",
            Status::Failed,
            "runtime change regressed feed",
        );
        let proposal =
            propose_tentacle_evolution_with_state(root, "swe-agent", "improve feed", &state)
                .unwrap();

        let recommendation = recommend_tentacle_evolution_apply(&proposal, &state).unwrap();

        assert_ne!(recommendation.candidate_id, "03-runtime-code");
        assert_eq!(recommendation.recommendation_score, 0.0);
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
        assert_eq!(empty.next_action, "octopus adapt".to_string());
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
        harness.chat("build a clean-brain agent");
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
        assert_eq!(
            report.active_grants,
            vec!["github:dangoZhang/Octopus".to_string()]
        );
        assert_eq!(report.next_action, "octopus beat 200".to_string());
        let state_path = Path::new("/tmp/octopus state.json");
        let with_state = harness.state.status_report_with_state(Some(state_path));
        assert_eq!(
            with_state.next_action,
            "octopus --state '/tmp/octopus state.json' beat 200".to_string()
        );
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
    fn chat_refines_goal_outside_brain_state() {
        let mut harness = Harness::new();

        let first = harness.chat("build a clean-brain agent");
        let second = harness.chat("make tools think through tentacles");

        let goal = harness.state.goal.as_ref().unwrap();
        assert_eq!(first.goal.objective, "build a clean-brain agent");
        assert_eq!(second.goal.constraints.len(), 1);
        assert_eq!(goal.objective, "build a clean-brain agent");
        assert_eq!(
            goal.constraints,
            vec!["make tools think through tentacles".to_string()]
        );
        assert_eq!(harness.state.goal_turns.len(), 2);
        assert!(harness.state.memory.recall("tentacles", 1)[0]
            .text
            .contains("goal update"));
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
    fn self_iteration_requires_oauth_grant_for_pr_mode() {
        let mut state = HarnessState::default();

        let report_only =
            state.self_iteration_plan("dangoZhang/Octopus", Some("improve usability"));
        assert!(!report_only.authorized);
        assert_eq!(report_only.mode, "report-only");
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
            RulePlanner,
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
    fn openai_compatible_chat_client_builds_payload_and_parses_response() {
        let config = OpenAiCompatibleConfig {
            model: "test-model".to_string(),
            api_key: Some("token".to_string()),
            base_url: "https://llm.example/v1/".to_string(),
            timeout_seconds: 3,
            curl_command: "curl".to_string(),
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
            curl_command: curl.to_string_lossy().to_string(),
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
        assert!(profiles
            .iter()
            .find(|profile| profile.id == "visual")
            .unwrap()
            .tools
            .iter()
            .any(|tool| tool.implementation.entrypoint == "docs/pet.html"));
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

    #[test]
    fn tentacle_evolution_proposal_writes_safe_artifacts() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let proposal = propose_tentacle_evolution(
            &root,
            "swe-agent",
            "improve repository observation feed quality",
        )
        .unwrap();

        assert_eq!(proposal.tentacle_id, "swe-agent");
        assert!(proposal.current_brain_prompt.contains("cognitive Need"));
        assert!(proposal.editable.contains(&"brain.prompt".to_string()));
        assert!(proposal
            .surfaces
            .iter()
            .any(|surface| surface.id == "runtime_code"));
        assert!(proposal
            .patch_candidates
            .iter()
            .any(|candidate| candidate.surface_id == "runtime_code"
                && candidate.title.contains("runtime harness")
                && candidate.draft.authorization_required));
        assert!(proposal
            .files
            .iter()
            .any(|file| file.path == "manifest.json#brain.prompt"));
        assert!(proposal
            .next_steps
            .iter()
            .any(|step| step.contains("cargo test")));

        let workspace = std::env::temp_dir().join(format!("octopus-evolve-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        let artifact = write_tentacle_evolution_artifacts(&workspace, &proposal).unwrap();
        let markdown = fs::read_to_string(&artifact.proposal_path).unwrap();
        let candidates = fs::read_to_string(&artifact.candidates_path).unwrap();
        let drafts = fs::read_to_string(&artifact.drafts_path).unwrap();
        let runtime_draft_path = artifact
            .patch_draft_paths
            .iter()
            .find(|path| path.contains("03-runtime-code"))
            .unwrap();
        let runtime_draft = fs::read_to_string(runtime_draft_path).unwrap();
        let json = fs::read_to_string(&artifact.json_path).unwrap();

        assert!(markdown.contains("# Tentacle Evolution: swe-agent"));
        assert!(markdown.contains("improve repository observation feed quality"));
        assert!(markdown.contains("## Evolution Surfaces"));
        assert!(markdown.contains("## Patch Candidates"));
        assert!(candidates.contains("# Patch Candidates: swe-agent"));
        assert!(candidates.contains("surface: `runtime_code`"));
        assert!(drafts.contains("# Patch Drafts: swe-agent"));
        assert!(runtime_draft.contains("authorization_required: true"));
        assert!(runtime_draft.contains("diff intent:"));
        assert!(json.contains("\"tentacle_id\": \"swe-agent\""));
        assert!(json.contains("\"patch_candidates\""));
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn tentacle_evolution_proposal_carries_previous_outcomes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();
        state.record_evolution_outcome(
            "swe-agent",
            "03-runtime-code",
            Status::Satisfied,
            "patch improved feed evidence",
        );

        let proposal = propose_tentacle_evolution_with_state(
            &root,
            "swe-agent",
            "improve repository observation feed quality",
            &state,
        )
        .unwrap();
        let markdown = render_tentacle_evolution_proposal(&proposal);

        assert_eq!(proposal.previous_outcomes.len(), 1);
        assert!(markdown.contains("Previous Outcomes"));
        assert!(markdown.contains("patch improved feed evidence"));
    }

    #[test]
    fn tentacle_evolution_proposal_carries_recent_feed_traces() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();
        let feed = Feed {
            need: Need::new(NeedKind::Observe, "README.md"),
            status: Status::Satisfied,
            evidence: vec![Evidence::new("read", "README evidence")],
            summary: "observed README through read tool".to_string(),
            metadata: BTreeMap::from([
                ("tentacle".to_string(), "swe-agent".to_string()),
                ("tool".to_string(), "read".to_string()),
                ("plan_source".to_string(), "llm".to_string()),
            ]),
        };
        state.record_feed_trace_from_feed(&feed);
        let other_feed = Feed {
            need: Need::new(NeedKind::Observe, "current browser tab"),
            status: Status::Satisfied,
            evidence: vec![Evidence::new("browser_status", "tab evidence")],
            summary: "browser trace".to_string(),
            metadata: BTreeMap::from([("tentacle".to_string(), "computer-use-agent".to_string())]),
        };
        state.record_feed_trace_from_feed(&other_feed);

        let proposal = propose_tentacle_evolution_with_state(
            &root,
            "swe-agent",
            "improve repository observation feed quality",
            &state,
        )
        .unwrap();
        let markdown = render_tentacle_evolution_proposal(&proposal);

        assert_eq!(proposal.recent_feed_traces.len(), 1);
        assert_eq!(proposal.recent_feed_traces[0].tool.as_deref(), Some("read"));
        let runtime_candidate = proposal
            .patch_candidates
            .iter()
            .find(|candidate| candidate.surface_id == "runtime_code")
            .unwrap();
        assert!(runtime_candidate.target.ends_with("tools/read.sh"));
        assert!(runtime_candidate.rationale.contains("trace #1"));
        assert!(runtime_candidate.change_plan[0].contains("tools/read.sh"));
        assert!(markdown.contains("Recent Feed Traces"));
        assert!(markdown.contains("observed README through read tool"));
        assert!(!markdown.contains("browser trace"));
    }

    #[test]
    fn evolution_recommendation_scores_feed_trace_feedback() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();
        state.record_feed_trace_from_feed(&Feed {
            need: Need::new(NeedKind::Observe, "README.md"),
            status: Status::Failed,
            evidence: vec![Evidence::new("read", "read output omitted evidence")],
            summary: "read output omitted evidence".to_string(),
            metadata: BTreeMap::from([
                ("tentacle".to_string(), "swe-agent".to_string()),
                ("tool".to_string(), "read".to_string()),
                ("plan_source".to_string(), "llm".to_string()),
            ]),
        });

        let proposal = propose_tentacle_evolution_with_state(
            &root,
            "swe-agent",
            "repair weak observe feed",
            &state,
        )
        .unwrap();
        let recommendation = recommend_tentacle_evolution_apply(&proposal, &state).unwrap();

        assert_eq!(recommendation.candidate_id, "03-runtime-code");
        assert_eq!(recommendation.surface_id, "runtime_code");
        assert_eq!(recommendation.feed_trace_count, 1);
        assert!(recommendation.reason.contains("Feed trace"));
        assert!(recommendation.recommendation_score > 0.0);
    }

    #[test]
    fn tentacle_evolution_uses_recent_check_history() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
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
            stderr: "line range failed".to_string(),
        });
        state.record_check_history(CheckHistoryInput {
            tentacle_id: "computer-use-agent".to_string(),
            source_kind: "manifest".to_string(),
            command_index: Some(1),
            command: "tools/window_status.sh".to_string(),
            cwd: "tentacles/computer-use-agent".to_string(),
            status: Status::Failed,
            code: Some(1),
            stdout: String::new(),
            stderr: "window unavailable".to_string(),
        });

        let proposal = propose_tentacle_evolution_with_state(
            &root,
            "swe-agent",
            "repair failing repo check",
            &state,
        )
        .unwrap();
        let markdown = render_tentacle_evolution_proposal(&proposal);
        let runtime_candidate = proposal
            .patch_candidates
            .iter()
            .find(|candidate| candidate.surface_id == "runtime_code")
            .unwrap();
        let recommendation = recommend_tentacle_evolution_apply(&proposal, &state).unwrap();

        assert_eq!(proposal.recent_check_history.len(), 1);
        assert_eq!(proposal.recent_check_history[0].command_index, Some(1));
        assert!(runtime_candidate.target.ends_with("tools/read.sh"));
        assert!(runtime_candidate.rationale.contains("failed check #1"));
        assert_eq!(runtime_candidate.feedback.len(), 1);
        assert_eq!(runtime_candidate.feedback[0].kind, "check_history");
        assert!(runtime_candidate.feedback[0]
            .summary
            .contains("line range failed"));
        assert!(runtime_candidate.change_plan[0].contains("repair failing check #1"));
        assert_eq!(recommendation.candidate_id, "03-runtime-code");
        assert_eq!(recommendation.check_history_count, 1);
        assert_eq!(recommendation.apply.feedback.len(), 1);
        assert!(recommendation.recommendation_score > 0.0);
        assert!(recommendation.reason.contains("check history"));
        assert!(markdown.contains("Recent Check History"));
        assert!(markdown.contains("line range failed"));
        assert!(!markdown.contains("window unavailable"));

        let workspace =
            std::env::temp_dir().join(format!("octopus-check-draft-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        let artifact = write_tentacle_evolution_artifacts(&workspace, &proposal).unwrap();
        let candidates = fs::read_to_string(&artifact.candidates_path).unwrap();
        let runtime_draft_path = artifact
            .patch_draft_paths
            .iter()
            .find(|path| path.contains("03-runtime-code"))
            .unwrap();
        let runtime_draft = fs::read_to_string(runtime_draft_path).unwrap();
        let apply_artifact =
            write_tentacle_apply_artifacts(&workspace, &recommendation.apply).unwrap();
        let apply_plan = fs::read_to_string(&apply_artifact.plan_path).unwrap();

        assert!(candidates.contains("feedback:"));
        assert!(runtime_draft.contains("feedback focus:"));
        assert!(runtime_draft.contains("line range failed"));
        assert!(apply_plan.contains("feedback focus:"));
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn harness_beat_evolution_writes_recommended_apply_plan() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
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
            stderr: "read output lost line numbers".to_string(),
        });
        state.record_check_history(CheckHistoryInput {
            tentacle_id: "missing-tentacle".to_string(),
            source_kind: "manifest".to_string(),
            command_index: Some(1),
            command: "tools/missing.sh".to_string(),
            cwd: "tentacles/missing-tentacle".to_string(),
            status: Status::Failed,
            code: Some(127),
            stdout: String::new(),
            stderr: "stale check".to_string(),
        });

        let workspace =
            std::env::temp_dir().join(format!("octopus-harness-beat-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        let evolution = write_harness_beat_evolution_artifacts(&root, &workspace, &state)
            .unwrap()
            .unwrap();

        assert_eq!(evolution.source_check_index, 1);
        assert_eq!(evolution.tentacle_id, "swe-agent");
        assert_eq!(evolution.candidate_id, "03-runtime-code");
        assert_eq!(evolution.status, "needs_authorization");
        assert!(evolution
            .objective
            .contains("read output lost line numbers"));
        assert!(evolution
            .next_action
            .contains("octopus oauth octopus evolve:swe-agent harness:write"));
        assert!(Path::new(&evolution.proposal_path).exists());
        assert!(Path::new(&evolution.apply_plan_path).exists());
        let plan = fs::read_to_string(&evolution.apply_plan_path).unwrap();
        assert!(plan.contains("feedback focus:"));
        assert!(plan.contains("read output lost line numbers"));
        assert!(evolution.apply_plan_preview.contains("feedback focus:"));
        assert!(evolution
            .apply_plan_preview
            .contains("read output lost line numbers"));
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn harness_beat_evolution_can_start_from_failed_feed_trace() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();
        state.record_feed_trace_from_feed(&Feed {
            need: Need::new(NeedKind::Observe, "README.md"),
            status: Status::Failed,
            evidence: vec![Evidence::new("read", "read feed lost the requested lines")],
            summary: "read feed lost the requested lines".to_string(),
            metadata: BTreeMap::from([
                ("tentacle".to_string(), "swe-agent".to_string()),
                ("tool".to_string(), "read".to_string()),
                ("plan_source".to_string(), "llm".to_string()),
            ]),
        });
        state.record_feed_trace_from_feed(&Feed {
            need: Need::new(NeedKind::Observe, "unknown"),
            status: Status::Failed,
            evidence: vec![Evidence::new("missing", "stale missing tentacle")],
            summary: "stale missing tentacle".to_string(),
            metadata: BTreeMap::from([("tentacle".to_string(), "missing-tentacle".to_string())]),
        });

        let workspace =
            std::env::temp_dir().join(format!("octopus-harness-beat-feed-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        let evolution = write_harness_beat_evolution_artifacts(&root, &workspace, &state)
            .unwrap()
            .unwrap();

        assert_eq!(evolution.source_kind, "feed_trace");
        assert_eq!(evolution.source_index, 1);
        assert_eq!(evolution.source_check_index, 0);
        assert_eq!(evolution.tentacle_id, "swe-agent");
        assert_eq!(evolution.candidate_id, "03-runtime-code");
        assert!(evolution
            .objective
            .contains("read feed lost the requested lines"));
        assert!(evolution.reason.contains("Feed trace"));
        assert!(Path::new(&evolution.proposal_path).exists());
        assert!(Path::new(&evolution.apply_plan_path).exists());
        let plan = fs::read_to_string(&evolution.apply_plan_path).unwrap();
        assert!(plan.contains("feedback focus:"));
        assert!(plan.contains("read feed lost the requested lines"));
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn tentacle_evolution_apply_uses_traced_runtime_tool_target() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();
        state.record_feed_trace_from_feed(&Feed {
            need: Need::new(NeedKind::Observe, "README.md"),
            status: Status::Satisfied,
            evidence: vec![Evidence::new("read", "README evidence")],
            summary: "observed README through read tool".to_string(),
            metadata: BTreeMap::from([
                ("tentacle".to_string(), "swe-agent".to_string()),
                ("tool".to_string(), "read".to_string()),
                ("plan_source".to_string(), "llm".to_string()),
            ]),
        });
        state.grant_oauth(
            "octopus",
            "evolve:swe-agent",
            default_permissions("octopus"),
        );
        let proposal = propose_tentacle_evolution_with_state(
            &root,
            "swe-agent",
            "improve repository observation feed quality",
            &state,
        )
        .unwrap();
        let plan = plan_tentacle_evolution_apply(&proposal, &state, "runtime_code").unwrap();

        assert!(plan.authorized);
        assert!(plan.target.ends_with("tools/read.sh"));

        let workspace =
            std::env::temp_dir().join(format!("octopus-traced-apply-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        let artifact = write_tentacle_apply_artifacts(&workspace, &plan).unwrap();
        let patch = fs::read_to_string(artifact.patch_path.as_ref().unwrap()).unwrap();

        assert!(patch.contains("b/tentacles/swe-agent/tools/read.sh"));
        assert!(!patch.contains("b/tentacles/swe-agent/tools/edit.sh"));
        let _ = fs::remove_dir_all(workspace);
    }

    struct EvolutionFakeChat {
        response: String,
        prompt: String,
    }

    impl ChatClient for EvolutionFakeChat {
        fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
            self.prompt = messages
                .iter()
                .map(|message| message.content.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            Ok(ChatResponse {
                content: self.response.clone(),
                metadata: BTreeMap::new(),
            })
        }
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
                ("plan_source".to_string(), "rule".to_string()),
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
        assert!(draft.contains("provider patch draft:"));
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
    fn tentacle_evolution_apply_plan_respects_authorization() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let proposal = propose_tentacle_evolution(&root, "swe-agent", "improve repo feed").unwrap();
        let mut state = HarnessState::default();

        let blocked = plan_tentacle_evolution_apply(&proposal, &state, "runtime_code").unwrap();
        assert!(!blocked.authorized);
        assert_eq!(blocked.status, "needs_authorization");
        assert_eq!(blocked.required_grant, "octopus:evolve:swe-agent");

        state.grant_oauth(
            "octopus",
            "evolve:swe-agent",
            default_permissions("octopus"),
        );
        let ready = plan_tentacle_evolution_apply(&proposal, &state, "03-runtime-code").unwrap();
        assert!(ready.authorized);
        assert_eq!(ready.status, "ready_for_authorized_patch");
        assert_eq!(
            ready.active_grant.as_deref(),
            Some("octopus:evolve:swe-agent")
        );

        let workspace = std::env::temp_dir().join(format!("octopus-apply-{}", std::process::id()));
        let _ = fs::remove_dir_all(&workspace);
        let blocked_artifact = write_tentacle_apply_artifacts(&workspace, &blocked).unwrap();
        assert!(blocked_artifact.patch_path.is_none());
        let expected_patch = workspace
            .join(".octopus")
            .join("evolution")
            .join("swe-agent")
            .join("apply")
            .join("03-runtime-code.patch");
        assert!(!expected_patch.exists());
        let artifact = write_tentacle_apply_artifacts(&workspace, &ready).unwrap();
        let markdown = fs::read_to_string(&artifact.plan_path).unwrap();
        let patch_path = artifact.patch_path.as_ref().unwrap();
        let patch = fs::read_to_string(patch_path).unwrap();
        let json = fs::read_to_string(&artifact.json_path).unwrap();
        assert!(markdown.contains("Evolution Apply Plan"));
        assert!(markdown.contains("authorized: true"));
        assert!(markdown.contains("patch: `03-runtime-code.patch`"));
        assert!(patch.contains("diff --git"));
        assert!(patch.contains("Octopus evolution candidate 03-runtime-code"));
        assert!(json.contains("\"authorized\": true"));
        let mut off_target = ready.clone();
        off_target.suggested_patch = Some(
            "diff --git a/tentacles/swe-agent/tools/edit.sh b/tentacles/swe-agent/tools/edit.sh\n--- a/tentacles/swe-agent/tools/edit.sh\n+++ b/tentacles/swe-agent/tools/edit.sh\n@@ -1,2 +1,3 @@\n #!/bin/sh\n+# off-target provider draft\n set -eu\n"
                .to_string(),
        );
        let off_target_artifact = write_tentacle_apply_artifacts(&workspace, &off_target).unwrap();
        let off_target_patch =
            fs::read_to_string(off_target_artifact.patch_path.as_ref().unwrap()).unwrap();
        assert!(!off_target_patch.contains("off-target provider draft"));
        assert!(off_target_patch.contains("Octopus evolution candidate 03-runtime-code"));
        let blocked_again = write_tentacle_apply_artifacts(&workspace, &blocked).unwrap();
        assert!(blocked_again.patch_path.is_none());
        assert!(!expected_patch.exists());
        let _ = fs::remove_dir_all(workspace);
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

        let mut tentacle = ManifestTentacle::new(InstalledTentacle {
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
        });

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
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let dir = std::env::temp_dir()
                .join(format!("octopus-tool-permission-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(dir.join("tools")).unwrap();
            let script = dir.join("tools").join("guarded.sh");
            fs::write(&script, "#!/bin/sh\nprintf 'guarded:%s\\n' \"$1\"\n").unwrap();
            let mut permissions = fs::metadata(&script).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&script, permissions).unwrap();

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
                runtime_kinds: vec!["shell".to_string()],
                needs: vec!["execute".to_string()],
                tools: vec!["guarded:shell:tools/guarded.sh".to_string()],
                tool_meta: vec![InstalledTool {
                    id: "guarded".to_string(),
                    description: "Guarded local execution.".to_string(),
                    input: "query arg".to_string(),
                    output: "guarded output".to_string(),
                    kind: "shell".to_string(),
                    entrypoint: "tools/guarded.sh".to_string(),
                    contract: None,
                    permission: Some(permission.clone()),
                }],
                editable: vec!["tools/*".to_string()],
                evolution_surfaces: vec!["tool_meta".to_string(), "runtime_code".to_string()],
            };

            let mut blocked = ManifestTentacle::new(installed.clone());
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
            let mut allowed =
                ManifestTentacle::new_with_llm_and_grants(installed, None, vec![grant.clone()]);
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
        let mut harness = Harness::with_state(state);

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
        let mut harness = Harness::with_state(state);
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
            .evolution_surfaces
            .contains(&"runtime_code".to_string()));
        assert!(state.install_manifest(&root, "missing").is_err());
    }

    #[test]
    fn think_tentacle_exposes_tool_side_plan_without_execution() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let need = Need::new(NeedKind::Observe, "current browser tab");

        let blocked = think_tentacle(&root, "computer-use-agent", &need, None, &[]).unwrap();

        assert_eq!(blocked.tentacle_id, "computer-use-agent");
        assert_eq!(blocked.selected_tool.id, "browser_status");
        assert_eq!(blocked.plan_source, "rule");
        assert!(blocked.selected_tool.authorization_required);
        assert!(blocked.selected_tool.active_grant.is_none());
        assert!(blocked
            .context_policy
            .contains("Need + Tool + Action + Tool + Action"));
        assert!(blocked
            .candidates
            .iter()
            .any(|candidate| candidate.id == "browser_status"));

        let grants = vec![CapabilityGrant {
            id: "octopus:tool:computer-use-agent".to_string(),
            provider: "octopus".to_string(),
            scope: "tool:computer-use-agent".to_string(),
            permissions: vec!["tool:observe".to_string()],
            status: GrantStatus::Active,
        }];
        let active = think_tentacle(&root, "computer-use-agent", &need, None, &grants).unwrap();

        assert_eq!(
            active.selected_tool.active_grant.as_deref(),
            Some("octopus:tool:computer-use-agent")
        );
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
            curl_command: curl.to_string_lossy().to_string(),
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
    fn installed_manifest_tentacle_feeds_observe_need() {
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

        assert_eq!(feedback.status, Status::Satisfied, "{}", feedback.summary);
        assert!(feedback.summary.contains("== project =="));
        assert_eq!(
            feedback.feeds[0].metadata.get("tool"),
            Some(&"inspect_repo".to_string())
        );
        assert!(feedback.feeds[0]
            .metadata
            .get("plan")
            .is_some_and(|plan| plan.contains("selected inspect_repo")));
        assert!(feedback.feeds[0]
            .metadata
            .get("available_tools")
            .is_some_and(|tools| tools.contains("inspect_repo")));
        assert!(feedback.feeds[0]
            .metadata
            .get("brain_prompt")
            .is_some_and(|prompt| prompt.contains("select tools from metadata")));
        assert!(feedback.feeds[0]
            .metadata
            .get("context_policy")
            .is_some_and(|context| context == "Need + Tool + Action + Tool + Action -> Feed"));
        assert!(feedback.feeds[0].evidence.iter().any(|evidence| {
            evidence.source == "tentacle_plan"
                && evidence.metadata.get("tool") == Some(&"inspect_repo".to_string())
                && evidence.metadata.get("plan_source") == Some(&"rule".to_string())
        }));
        assert_eq!(harness.state.feed_traces.len(), 1);
        assert_eq!(
            harness.state.feed_traces[0].tool.as_deref(),
            Some("inspect_repo")
        );
        assert!(harness.state.routes.score(&NeedKind::Observe, "swe-agent") > 1.0);
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
            curl_command: curl.to_string_lossy().to_string(),
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
            curl_command: curl.to_string_lossy().to_string(),
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

    #[test]
    fn computer_use_tentacle_does_not_take_repo_observe_need() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();
        let installed = state.install_manifest(&root, "computer-use-agent").unwrap();
        let tentacle = ManifestTentacle::new(installed);

        assert!(!tentacle.supports(&Need::new(NeedKind::Observe, ".")));
        assert!(tentacle.supports(&Need::new(NeedKind::Observe, "describe screen")));
        assert!(tentacle.supports(&Need::new(NeedKind::Observe, "current window")));
        assert!(tentacle.supports(&Need::new(NeedKind::Observe, "current browser tab")));
        assert!(tentacle.supports(&Need::new(NeedKind::Observe, "read clipboard")));
        assert!(!tentacle.supports(&Need::new(NeedKind::Execute, "echo ok")));
        assert!(tentacle.supports(&Need::new(NeedKind::Execute, "open browser url")));
        assert!(tentacle.supports(&Need::new(NeedKind::Execute, "copy text to clipboard")));
    }

    #[test]
    fn computer_use_browser_need_prefers_browser_status() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();
        let installed = state.install_manifest(&root, "computer-use-agent").unwrap();
        let mut tentacle = ManifestTentacle::new(installed);

        let plan = tentacle
            .plan_tool(&Need::new(NeedKind::Observe, "current browser tab"))
            .expect("browser observe need should plan a tool");

        assert_eq!(plan.tool.id, "browser_status");
    }

    #[test]
    fn computer_use_window_need_prefers_window_status() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();
        let installed = state.install_manifest(&root, "computer-use-agent").unwrap();
        let mut tentacle = ManifestTentacle::new(installed);

        let plan = tentacle
            .plan_tool(&Need::new(NeedKind::Observe, "current window"))
            .expect("window observe need should plan a tool");

        assert_eq!(plan.tool.id, "window_status");
    }

    #[test]
    fn computer_use_clipboard_needs_prefer_clipboard_tools() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tentacles");
        let mut state = HarnessState::default();
        let installed = state.install_manifest(&root, "computer-use-agent").unwrap();
        let mut tentacle = ManifestTentacle::new(installed);

        let read = tentacle
            .plan_tool(&Need::new(NeedKind::Observe, "read clipboard"))
            .expect("clipboard observe need should plan a tool");
        let write = tentacle
            .plan_tool(&Need::new(NeedKind::Execute, "copy octopus to clipboard"))
            .expect("clipboard execute need should plan a tool");

        assert_eq!(read.tool.id, "clipboard_read");
        assert_eq!(write.tool.id, "clipboard_write");
    }
}
