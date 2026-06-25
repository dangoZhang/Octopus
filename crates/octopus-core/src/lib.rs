use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::Command;

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
    pub draft: Option<PullRequestDraft>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PullRequestDraft {
    pub branch: String,
    pub title: String,
    pub change_summary: String,
    pub body: String,
    pub check_results: Vec<String>,
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
    pub goal: Option<Goal>,
    #[serde(default)]
    pub goal_turns: Vec<GoalTurn>,
    #[serde(default)]
    pub grants: Vec<CapabilityGrant>,
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
            draft,
        }
    }
}

pub struct Harness {
    pub state: HarnessState,
    tentacles: Vec<Box<dyn Tentacle>>,
}

impl Harness {
    pub fn new() -> Self {
        Self {
            state: HarnessState::default(),
            tentacles: Vec::new(),
        }
    }

    pub fn with_state(state: HarnessState) -> Self {
        Self {
            state,
            tentacles: Vec::new(),
        }
    }

    pub fn add_tentacle(&mut self, tentacle: Box<dyn Tentacle>) {
        self.tentacles.push(tentacle);
    }

    pub fn feed_one(&mut self, need: &Need) -> Feed {
        if let Some(feed) = self.memory_feed(need) {
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
            return Feed::unsupported(need, "no tentacle supports this need");
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
        self.state.routes.learn(&feed);
        feed
    }

    pub fn feed(&mut self, needs: &[Need]) -> Feedback {
        let feeds = needs.iter().map(|need| self.feed_one(need)).collect();
        Feedback::from_feeds(feeds)
    }

    pub fn chat(&mut self, message: impl Into<String>) -> GoalChat {
        let message = message.into();
        let goal = match self.state.goal.take() {
            Some(mut goal) => {
                goal.refine(message.clone());
                goal
            }
            None => Goal::new(message.clone()),
        };
        let need = goal.need(NeedKind::Remember, format!("goal update: {message}"));
        let feed = self.feed_one(&need);
        let feedback = Feedback::from_feeds(vec![feed.clone()]);
        let turn = GoalTurn {
            index: self.state.goal_turns.len() as u64 + 1,
            message,
            summary: feed.summary,
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
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ToolMetadata {
    pub id: String,
    pub description: String,
    pub implementation: ToolImplementation,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EvolutionPolicy {
    pub editable: Vec<String>,
    pub checks: Vec<String>,
    pub constraints: Vec<String>,
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
            name: "Memory Tentacle".to_string(),
            description: "Remembers, recalls, forgets, and compacts context.".to_string(),
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
            name: "Color Pet Tentacle".to_string(),
            description: "Shows heartbeat, memory, harness, blocked, and success state as a color-changing pet.".to_string(),
            brain: llm_brain(
                "Visual state translator",
                "Convert Feedback into user-visible pet state without changing the kernel contract.",
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
                &["grep -q data-state docs/pet.html"],
                &["Keep visual state outside Need, Feed, and Feedback."],
            ),
            llm_ready: true,
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
                    "tentacles/repo-maintainer/tools/draft_pr.sh".to_string(),
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
                    "draft_pr",
                    "Write branch plan and pull request draft data.",
                    "shell",
                    "tentacles/repo-maintainer/tools/draft_pr.sh",
                ),
            ],
            evolution: evolution_policy(
                &[
                    "tentacles/repo-maintainer/tools/inspect_repo.sh .",
                    "tentacles/repo-maintainer/tools/draft_pr.sh $(mktemp -d) dangoZhang/Octopus improve-usability",
                ],
                &["Wait for explicit OAuth/user grant before preparing branch or PR work."],
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
                description: "Read/edit files, inspect a repository, prepare patches, and run tests through editable shell tools.".to_string(),
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
                tool_meta("read", "Read a safe slice of a workspace file.", "shell", "tentacles/swe-agent/tools/read.sh"),
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
                    "tentacles/computer-use-agent/tools/describe_screen.sh".to_string(),
                ],
            }],
            tools: vec![
                tool_meta("mcp", "Call an MCP tool through a client adapter.", "mcp", "tentacles/computer-use-agent/tools/mcp.sh"),
                tool_meta("bash", "Write and run a local shell script.", "shell", "tentacles/computer-use-agent/tools/bash.sh"),
                tool_meta("screenshot", "Capture the current screen.", "shell", "tentacles/computer-use-agent/tools/screenshot.sh"),
                tool_meta("open_url", "Open a URL in the local desktop.", "shell", "tentacles/computer-use-agent/tools/open_url.sh"),
                tool_meta("describe_screen", "Return lightweight desktop context.", "shell", "tentacles/computer-use-agent/tools/describe_screen.sh"),
            ],
            evolution: evolution_policy(
                &["tentacles/computer-use-agent/tools/describe_screen.sh"],
                &["Ask for user-visible actions only when the product flow grants it."],
            ),
            llm_ready: true,
        },
        TentacleProfile {
            id: "bash-only".to_string(),
            name: "Bash Only Tentacle".to_string(),
            description: "Writes every tool call into a .sh file and executes it with bash.".to_string(),
            brain: llm_brain(
                "Shell-only planner",
                "Represent each action as a transparent script before execution.",
            ),
            skills: vec![SkillManifest {
                id: "write-and-run".to_string(),
                name: "Write And Run".to_string(),
                description: "Represent execution as editable shell scripts under .octopus/harness.".to_string(),
                needs: vec![NeedKind::Execute, NeedKind::Reproduce, NeedKind::Verify],
                tools: vec!["tentacles/bash-only/tools/write_and_run.sh".to_string()],
            }],
            tools: vec![tool_meta(
                "write_and_run",
                "Write stdin to a .sh file and execute it.",
                "shell",
                "tentacles/bash-only/tools/write_and_run.sh",
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
        },
    }
}

fn evolution_policy(checks: &[&str], constraints: &[&str]) -> EvolutionPolicy {
    EvolutionPolicy {
        editable: vec![
            "manifest.json".to_string(),
            "brain.prompt".to_string(),
            "tools/*".to_string(),
        ],
        checks: checks.iter().map(|item| item.to_string()).collect(),
        constraints: constraints.iter().map(|item| item.to_string()).collect(),
    }
}

pub fn default_permissions(provider: &str) -> Vec<String> {
    match provider {
        "github" => vec![
            "repo:read".to_string(),
            "checks:read".to_string(),
            "pull_request:write".to_string(),
        ],
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

fn route_key(kind: &NeedKind, name: &str) -> String {
    format!("{}:{name}", kind_key(kind))
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

        assert_eq!(feedback.status, Status::Satisfied);
        assert!(feedback.summary.contains("tools stay outside the brain"));
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
    fn default_catalog_contains_installable_profiles() {
        let profiles = default_tentacle_profiles();

        assert!(profiles.iter().any(|profile| profile.id == "research"));
        assert!(profiles.iter().any(|profile| profile.id == "visual"));
        assert!(profiles
            .iter()
            .any(|profile| profile.id == "repo-maintainer"));
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
    fn harness_state_installs_profiles_idempotently() {
        let mut state = HarnessState::default();

        state.install_profile("research").unwrap();
        state.install_profile("research").unwrap();

        assert_eq!(state.installed_profiles, vec!["research".to_string()]);
        assert!(state.install_profile("missing").is_err());
    }
}
