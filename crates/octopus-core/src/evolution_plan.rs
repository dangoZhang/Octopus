use crate::evolution_apply::EvolutionLiveApplyReport;
use crate::evolution_surface::propose_evolution_for_cli;
use crate::{write_tentacle_apply_artifacts, write_tentacle_evolution_artifacts};
use octopus_core::{
    recommend_tentacle_evolution_apply, EvolutionApplyArtifact, EvolutionArtifact,
    EvolutionRecommendation, HarnessState,
};
use std::fs;
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
    let patch_excerpt = live_apply
        .patch_path
        .as_deref()
        .and_then(|path| failed_patch_excerpt(path, &live_apply.stderr));
    let mut details = Vec::new();
    if !stderr.is_empty() {
        details.push(format!("git apply failed: {stderr}"));
    }
    if let Some(excerpt) = patch_excerpt {
        details.push(format!("bad patch excerpt: {excerpt}"));
    }
    if details.is_empty() {
        return format!("{objective}; regenerate an authorized patch against the current files");
    }
    format!(
        "{objective}; regenerate an authorized patch against the current files; {}; output one complete valid unified diff only",
        details.join("; ")
    )
}

fn compact_words(value: &str, limit: usize) -> String {
    let text = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() > limit {
        format!("{}...", text.chars().take(limit).collect::<String>())
    } else {
        text
    }
}

fn failed_patch_excerpt(path: &str, stderr: &str) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let lines = content.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return None;
    }
    let line_number = patch_error_line(stderr).unwrap_or(1).clamp(1, lines.len());
    let start = line_number.saturating_sub(4).max(1);
    let end = (line_number + 3).min(lines.len());
    let excerpt = (start..=end)
        .map(|number| format!("{number}: {}", lines[number - 1]))
        .collect::<Vec<_>>()
        .join(" | ");
    Some(compact_words(&excerpt, 420))
}

fn patch_error_line(stderr: &str) -> Option<usize> {
    stderr.split("line ").skip(1).find_map(|tail| {
        tail.chars()
            .take_while(|ch| ch.is_ascii_digit())
            .collect::<String>()
            .parse::<usize>()
            .ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_objective_includes_apply_error_without_full_log() {
        let dir =
            std::env::temp_dir().join(format!("octopus-retry-objective-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let patch = dir.join("candidate.patch");
        fs::write(
            &patch,
            "diff --git a/a b/a\n--- a/a\n+++ b/a\n@@ -1,1 +1,1 @@\n-old\n+new\n@@\n",
        )
        .unwrap();
        let mut live_apply = EvolutionLiveApplyReport {
            applied: false,
            status: "check_failed".to_string(),
            command: Some("git apply --check candidate.patch".to_string()),
            patch_path: Some(patch.to_string_lossy().to_string()),
            changed_paths: Vec::new(),
            target_boundary_violations: Vec::new(),
            stdout: String::new(),
            stderr: "error: corrupt patch at line 7: a very specific context mismatch".to_string(),
        };

        let retry = retry_apply_objective("improve harness", &live_apply);

        assert!(retry.contains("improve harness"));
        assert!(retry.contains("git apply failed"));
        assert!(retry.contains("specific context mismatch"));
        assert!(retry.contains("bad patch excerpt"));
        assert!(retry.contains("7: @@"));
        assert!(retry.contains("valid unified diff"));

        live_apply.stderr = " ".to_string();
        live_apply.patch_path = Some("missing.patch".to_string());
        assert_eq!(
            retry_apply_objective("improve harness", &live_apply),
            "improve harness; regenerate an authorized patch against the current files"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn patch_error_line_reads_git_apply_line_number() {
        assert_eq!(
            patch_error_line("warning\nerror: corrupt patch at line 188\n"),
            Some(188)
        );
        assert_eq!(patch_error_line("no line number"), None);
    }
}
