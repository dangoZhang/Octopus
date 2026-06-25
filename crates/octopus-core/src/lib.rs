use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;

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
                let score = words
                    .iter()
                    .filter(|word| haystack.contains(word.as_str()))
                    .count() as f32
                    + record.weight * 0.01;
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

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct HarnessState {
    pub memory: MemoryStore,
    pub routes: RouteBook,
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
    fn invalid_state_file_returns_error() {
        let path =
            std::env::temp_dir().join(format!("octopus-invalid-state-{}.json", std::process::id()));
        fs::write(&path, "{bad json").unwrap();

        let error = HarnessState::load(&path).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidData);
        let _ = fs::remove_file(path);
    }
}
