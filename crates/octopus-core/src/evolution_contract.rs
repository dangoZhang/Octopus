use crate::{CheckHistoryRecord, EvolutionOutcome, FeedTraceRecord, Status};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EvolutionPolicy {
    pub editable: Vec<String>,
    pub checks: Vec<String>,
    pub constraints: Vec<String>,
    #[serde(default = "default_evolution_surfaces")]
    pub surfaces: Vec<EvolutionSurface>,
    #[serde(default)]
    pub requirements: Vec<EvolutionRequirement>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EvolutionSurface {
    pub id: String,
    pub description: String,
    pub targets: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EvolutionRequirement {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub objective_markers_any: Vec<String>,
    #[serde(default)]
    pub required_surfaces: Vec<String>,
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
    #[serde(default)]
    pub requirements: Vec<EvolutionRequirement>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_files: Vec<String>,
    pub action: String,
    pub rationale: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EvolutionPatchCandidate {
    pub id: String,
    pub surface_id: String,
    pub title: String,
    pub target: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_files: Vec<String>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_files: Vec<String>,
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

fn default_evolution_generator() -> String {
    "llm_required".to_string()
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
