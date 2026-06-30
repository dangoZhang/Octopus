use crate::{
    evolution,
    evolution_patch::{clean_suggested_patch, diff_paths, patch_display_path},
    evolution_recommend::evolution_candidate_feedback_from_proposal,
    evolution_target::{evolution_candidate_target_files, field_pack_evolution_target},
    push_unique_limited, EvolutionPatchCandidate, EvolutionPatchDraft, EvolutionSurface,
    TentacleEvolutionProposal,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub(crate) struct ParsedLlmEvolutionPlan {
    pub(crate) summary: String,
    pub(crate) candidates: Vec<EvolutionPatchCandidate>,
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

pub(crate) fn parse_llm_evolution_plan(
    proposal: &TentacleEvolutionProposal,
    response_content: &str,
) -> Result<ParsedLlmEvolutionPlan, String> {
    let content = extract_json_object(response_content)
        .ok_or_else(|| "evolution LLM response did not contain a JSON object".to_string())?;
    let parsed_value = serde_json::from_str::<serde_json::Value>(content)
        .map_err(|error| format!("invalid evolution LLM JSON: {error}"))?;
    let parsed = serde_json::from_value::<LlmEvolutionResponse>(parsed_value)
        .map_err(|error| format!("invalid evolution LLM JSON shape: {error}"))?;
    let mut candidates = parsed
        .candidates
        .into_iter()
        .map(|candidate| llm_candidate_to_evolution(proposal, candidate))
        .collect::<Result<Vec<_>, _>>()?;
    if candidates.is_empty() {
        return Err("evolution LLM returned no candidates".to_string());
    }
    evolution::validate_candidates_for_objective(proposal, &candidates)?;
    evolution::uniquify_candidate_ids(&mut candidates);
    if parsed.summary.trim().is_empty() {
        return Err("evolution LLM response missing summary".to_string());
    }
    Ok(ParsedLlmEvolutionPlan {
        summary: parsed.summary,
        candidates,
    })
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
    if candidate.title.trim().is_empty() {
        return Err("evolution LLM candidate missing title".to_string());
    }
    if candidate.target.trim().is_empty() {
        return Err("evolution LLM candidate missing target".to_string());
    }
    if candidate.rationale.trim().is_empty() {
        return Err("evolution LLM candidate missing rationale".to_string());
    }
    if candidate.change_plan.is_empty() {
        return Err("evolution LLM candidate missing change_plan".to_string());
    }
    if candidate.checks.is_empty() {
        return Err("evolution LLM candidate missing checks".to_string());
    }
    let target = llm_candidate_target(proposal, surface, &candidate.target)?;
    let manifest_dir = Path::new(&proposal.manifest_path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("tentacles").join(&proposal.tentacle_id));
    let suggested_patch = clean_suggested_patch(candidate.suggested_patch);
    if let Some(patch) = &suggested_patch {
        validate_candidate_patch_contract(surface, patch)?;
    }
    let mut target_files =
        evolution_candidate_target_files(&manifest_dir, surface, &target, &proposal.objective);
    if let Some(patch) = &suggested_patch {
        let diff_files = diff_target_files_for_surface(&manifest_dir, surface, patch);
        if target_files.is_empty() {
            target_files = diff_files;
        } else if surface.id == "field_pack_tasks" {
            for file in diff_files {
                push_unique_limited(&mut target_files, file, usize::MAX);
            }
        }
    }
    let mut patch_candidate = EvolutionPatchCandidate {
        id: evolution_candidate_id(surface_index, &surface.id),
        surface_id: surface.id.clone(),
        title: candidate.title,
        target,
        target_files,
        rationale: candidate.rationale,
        feedback: Vec::new(),
        suggested_patch,
        change_plan: candidate.change_plan,
        checks: candidate.checks,
        draft: evolution_patch_draft(surface_index, surface),
    };
    patch_candidate.feedback =
        evolution_candidate_feedback_from_proposal(&patch_candidate, proposal);
    Ok(patch_candidate)
}

fn validate_candidate_patch_contract(
    surface: &EvolutionSurface,
    patch: &str,
) -> Result<(), String> {
    if surface.id != "field_pack_tasks" {
        return Ok(());
    }
    if patch.contains("\"task_layers\"") {
        return Err(
            "field_pack_tasks patch violates field-pack schema: use mini_tasks array, not task_layers"
                .to_string(),
        );
    }
    if patch.contains("\"mini_tasks\": {") {
        return Err(
            "field_pack_tasks patch violates field-pack schema: mini_tasks must remain an array of {id, goal, expected_feed} objects"
                .to_string(),
        );
    }
    Ok(())
}

pub(crate) fn diff_target_files_for_surface(
    manifest_dir: &Path,
    surface: &EvolutionSurface,
    patch: &str,
) -> Vec<String> {
    let manifest_prefix = patch_display_path(manifest_dir);
    let repair_prefix = format!("{manifest_prefix}/repair-templates/");
    let manifest_prefix = format!("{manifest_prefix}/");
    let mut files = Vec::new();
    for path in diff_paths(patch) {
        if path.contains("..") {
            continue;
        }
        let allowed = match surface.id.as_str() {
            "field_pack_tasks" => {
                path.starts_with("field-packs/") || path.starts_with(&repair_prefix)
            }
            "runtime_code" => path.starts_with(&manifest_prefix),
            _ => path.starts_with(&manifest_prefix),
        };
        if allowed {
            push_unique_limited(&mut files, path, usize::MAX);
        }
    }
    files
}

fn llm_candidate_target(
    proposal: &TentacleEvolutionProposal,
    surface: &EvolutionSurface,
    target: &str,
) -> Result<String, String> {
    let manifest_dir = Path::new(&proposal.manifest_path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("tentacles").join(&proposal.tentacle_id));
    let target = target.trim();
    if let Some(target) = field_pack_evolution_target(target) {
        return Ok(target);
    }
    if target.is_empty() || target.contains("..") {
        return Err(format!(
            "evolution LLM candidate target is invalid for surface {}",
            surface.id
        ));
    }
    if target.starts_with("brain.")
        || target.starts_with("tools[]")
        || target.starts_with("evolution.")
    {
        return Ok(format!(
            "{}#{}",
            manifest_dir.join("manifest.json").to_string_lossy(),
            target
        ));
    }
    if let Some(suffix) = target.strip_prefix("manifest.json") {
        return Ok(format!(
            "{}{}",
            manifest_dir.join("manifest.json").to_string_lossy(),
            suffix
        ));
    }
    let path = Path::new(target);
    if path.is_absolute() {
        let manifest_dir_text = manifest_dir.to_string_lossy();
        return if target.starts_with(manifest_dir_text.as_ref()) {
            Ok(target.to_string())
        } else {
            Err(
                "evolution LLM candidate target must stay inside declared harness targets"
                    .to_string(),
            )
        };
    }
    Ok(manifest_dir.join(target).to_string_lossy().to_string())
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

fn extract_json_object(value: &str) -> Option<&str> {
    let start = value.find('{')?;
    let end = value.rfind('}')?;
    (start <= end).then_some(&value[start..=end])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field_pack_surface() -> EvolutionSurface {
        EvolutionSurface {
            id: "field_pack_tasks".to_string(),
            description: "field tasks".to_string(),
            targets: vec!["field-packs/*/field-pack.json".to_string()],
        }
    }

    #[test]
    fn field_pack_candidate_rejects_task_layers_shape() {
        let error = validate_candidate_patch_contract(
            &field_pack_surface(),
            r#"diff --git a/field-packs/write/field-pack.json b/field-packs/write/field-pack.json
@@ -1 +1 @@
+  "task_layers": []
"#,
        )
        .unwrap_err();

        assert!(error.contains("mini_tasks array"));
    }

    #[test]
    fn field_pack_candidate_rejects_mini_tasks_object_shape() {
        let error = validate_candidate_patch_contract(
            &field_pack_surface(),
            r#"diff --git a/field-packs/write/field-pack.json b/field-packs/write/field-pack.json
@@ -1 +1 @@
+  "mini_tasks": {}
"#,
        )
        .unwrap_err();

        assert!(error.contains("mini_tasks must remain an array"));
    }
}
