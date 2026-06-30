use crate::evolution_apply::EvolutionLiveApplyReport;
use crate::{
    propose_evolution_for_cli, write_tentacle_apply_artifacts, write_tentacle_evolution_artifacts,
};
use octopus_core::{
    recommend_tentacle_evolution_apply, EvolutionApplyArtifact, EvolutionArtifact,
    EvolutionRecommendation, HarnessState,
};
use std::path::Path;

pub(crate) struct EvolutionPatchPlanArtifacts {
    pub(crate) evolution_artifact: EvolutionArtifact,
    pub(crate) recommendation: EvolutionRecommendation,
    pub(crate) apply_artifact: EvolutionApplyArtifact,
}

pub(crate) fn recommend_current_patch(
    cwd: &Path,
    tentacle_id: &str,
    objective: &str,
    state: &HarnessState,
) -> Result<EvolutionPatchPlanArtifacts, String> {
    let proposal = propose_evolution_for_cli(tentacle_id, objective, state)?;
    let evolution_artifact = write_tentacle_evolution_artifacts(cwd, &proposal)?;
    let recommendation = recommend_tentacle_evolution_apply(&proposal, state)?;
    let apply_artifact = write_tentacle_apply_artifacts(cwd, &recommendation.apply)?;
    Ok(EvolutionPatchPlanArtifacts {
        evolution_artifact,
        recommendation,
        apply_artifact,
    })
}

pub(crate) fn retry_apply_objective(
    objective: &str,
    live_apply: &EvolutionLiveApplyReport,
) -> String {
    let stderr = compact_words(&live_apply.stderr, 220);
    if stderr.is_empty() {
        format!("{objective}; regenerate an authorized patch against the current files")
    } else {
        format!(
            "{objective}; regenerate an authorized patch against the current files after git apply failed: {stderr}"
        )
    }
}

fn compact_words(value: &str, limit: usize) -> String {
    let text = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() > limit {
        format!("{}...", text.chars().take(limit).collect::<String>())
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_objective_includes_apply_error_without_full_log() {
        let mut live_apply = EvolutionLiveApplyReport {
            applied: false,
            status: "check_failed".to_string(),
            command: Some("git apply --check candidate.patch".to_string()),
            patch_path: Some("candidate.patch".to_string()),
            stdout: String::new(),
            stderr: "error: patch failed: a very specific context mismatch".to_string(),
        };

        let retry = retry_apply_objective("improve harness", &live_apply);

        assert!(retry.contains("improve harness"));
        assert!(retry.contains("git apply failed"));
        assert!(retry.contains("specific context mismatch"));

        live_apply.stderr = " ".to_string();
        assert_eq!(
            retry_apply_objective("improve harness", &live_apply),
            "improve harness; regenerate an authorized patch against the current files"
        );
    }
}
