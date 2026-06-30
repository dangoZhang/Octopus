use crate::shell_words::shell_arg;
use octopus_core::{EvolutionApplyArtifact, EvolutionApplyPlan, EvolutionRecommendation};
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct EvolutionLiveApplyReport {
    pub(crate) applied: bool,
    pub(crate) status: String,
    pub(crate) command: Option<String>,
    pub(crate) patch_path: Option<String>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) fn apply_authorized_suggested_patch(
    cwd: &Path,
    plan: &EvolutionApplyPlan,
    artifact: &EvolutionApplyArtifact,
) -> EvolutionLiveApplyReport {
    if !plan.authorized {
        return EvolutionLiveApplyReport {
            applied: false,
            status: "skipped_not_authorized".to_string(),
            command: None,
            patch_path: artifact.patch_path.clone(),
            stdout: String::new(),
            stderr: String::new(),
        };
    }
    if plan.suggested_patch.is_none() {
        return EvolutionLiveApplyReport {
            applied: false,
            status: "skipped_no_suggested_patch".to_string(),
            command: None,
            patch_path: artifact.patch_path.clone(),
            stdout: String::new(),
            stderr: String::new(),
        };
    }
    let Some(patch_path) = artifact.patch_path.as_deref() else {
        return EvolutionLiveApplyReport {
            applied: false,
            status: "skipped_missing_patch_artifact".to_string(),
            command: None,
            patch_path: None,
            stdout: String::new(),
            stderr: String::new(),
        };
    };
    let command_text = format!(
        "git apply --recount --unidiff-zero {}",
        shell_arg(patch_path)
    );
    let check_output = Command::new("git")
        .arg("apply")
        .arg("--check")
        .arg("--recount")
        .arg("--unidiff-zero")
        .arg(patch_path)
        .current_dir(cwd)
        .output();
    match check_output {
        Ok(check) if check.status.success() => {}
        Ok(check) => {
            let reverse_check = Command::new("git")
                .arg("apply")
                .arg("--reverse")
                .arg("--check")
                .arg("--recount")
                .arg(patch_path)
                .current_dir(cwd)
                .output();
            let (applied, status) = if reverse_check
                .as_ref()
                .is_ok_and(|reverse| reverse.status.success())
            {
                (true, "already_applied")
            } else {
                (false, "check_failed")
            };
            return EvolutionLiveApplyReport {
                applied,
                status: status.to_string(),
                command: Some(format!(
                    "git apply --check --recount --unidiff-zero {}",
                    shell_arg(patch_path)
                )),
                patch_path: Some(patch_path.to_string()),
                stdout: String::from_utf8_lossy(&check.stdout).to_string(),
                stderr: String::from_utf8_lossy(&check.stderr).to_string(),
            };
        }
        Err(error) => {
            return EvolutionLiveApplyReport {
                applied: false,
                status: "check_failed_to_start".to_string(),
                command: Some(format!(
                    "git apply --check --recount --unidiff-zero {}",
                    shell_arg(patch_path)
                )),
                patch_path: Some(patch_path.to_string()),
                stdout: String::new(),
                stderr: error.to_string(),
            };
        }
    }
    let output = Command::new("git")
        .arg("apply")
        .arg("--recount")
        .arg("--unidiff-zero")
        .arg(patch_path)
        .current_dir(cwd)
        .output();
    match output {
        Ok(output) => {
            let mut applied = output.status.success();
            let mut status = if applied {
                "applied".to_string()
            } else {
                "failed".to_string()
            };
            if !applied {
                let reverse_check = Command::new("git")
                    .arg("apply")
                    .arg("--reverse")
                    .arg("--check")
                    .arg("--recount")
                    .arg(patch_path)
                    .current_dir(cwd)
                    .output();
                if reverse_check
                    .as_ref()
                    .is_ok_and(|reverse| reverse.status.success())
                {
                    applied = true;
                    status = "already_applied".to_string();
                }
            }
            EvolutionLiveApplyReport {
                applied,
                status,
                command: Some(command_text),
                patch_path: Some(patch_path.to_string()),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            }
        }
        Err(error) => EvolutionLiveApplyReport {
            applied: false,
            status: "failed_to_start".to_string(),
            command: Some(command_text),
            patch_path: Some(patch_path.to_string()),
            stdout: String::new(),
            stderr: error.to_string(),
        },
    }
}

pub(crate) fn live_apply_summary(
    recommendation: &EvolutionRecommendation,
    live_apply: &EvolutionLiveApplyReport,
) -> String {
    live_apply_candidate_summary(&recommendation.candidate_id, live_apply)
}

pub(crate) fn live_apply_candidate_summary(
    candidate_id: &str,
    live_apply: &EvolutionLiveApplyReport,
) -> String {
    if live_apply.applied || live_apply.stderr.trim().is_empty() {
        return format!("{} {}", candidate_id, live_apply.status);
    }
    let detail = live_apply
        .stderr
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let detail = if detail.chars().count() > 160 {
        format!("{}...", detail.chars().take(160).collect::<String>())
    } else {
        detail
    };
    format!("{} {}: {}", candidate_id, live_apply.status, detail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_skips_when_plan_is_not_authorized() {
        let mut plan = test_plan();
        plan.authorized = false;
        let artifact = test_artifact(Some("candidate.patch"));

        let report = apply_authorized_suggested_patch(Path::new("."), &plan, &artifact);

        assert!(!report.applied);
        assert_eq!(report.status, "skipped_not_authorized");
        assert!(report.command.is_none());
        assert_eq!(report.patch_path.as_deref(), Some("candidate.patch"));
    }

    #[test]
    fn apply_skips_when_provider_did_not_return_patch() {
        let mut plan = test_plan();
        plan.suggested_patch = None;
        let artifact = test_artifact(Some("candidate.patch"));

        let report = apply_authorized_suggested_patch(Path::new("."), &plan, &artifact);

        assert!(!report.applied);
        assert_eq!(report.status, "skipped_no_suggested_patch");
        assert!(report.command.is_none());
    }

    #[test]
    fn live_apply_summary_compacts_stderr() {
        let recommendation = test_recommendation();
        let report = EvolutionLiveApplyReport {
            applied: false,
            status: "check_failed".to_string(),
            command: None,
            patch_path: Some("candidate.patch".to_string()),
            stdout: String::new(),
            stderr: "error: patch failed at current file context".to_string(),
        };

        let summary = live_apply_summary(&recommendation, &report);

        assert!(summary.contains("01-runtime-code check_failed"));
        assert!(summary.contains("patch failed"));
    }

    fn test_plan() -> EvolutionApplyPlan {
        EvolutionApplyPlan {
            tentacle_id: "field-mini-task".to_string(),
            candidate_id: "01-runtime-code".to_string(),
            objective: "repair harness".to_string(),
            authorized: true,
            status: "authorized".to_string(),
            required_grant: "octopus:evolve:field-mini-task".to_string(),
            active_grant: Some("octopus:evolve:field-mini-task".to_string()),
            target: "tentacles/field-mini-task/tools/run_field_mini_task.sh".to_string(),
            target_files: vec!["tentacles/field-mini-task/tools/run_field_mini_task.sh".to_string()],
            draft_path: "patches/01-runtime-code.patch".to_string(),
            checks: vec!["octopus check field-mini-task".to_string()],
            feedback: Vec::new(),
            suggested_patch: Some(
                "diff --git a/a b/a\n--- a/a\n+++ b/a\n@@ -1 +1 @@\n-a\n+b\n".to_string(),
            ),
            guardrails: vec!["declared target only".to_string()],
            next_steps: vec!["run manifest checks".to_string()],
        }
    }

    fn test_artifact(patch_path: Option<&str>) -> EvolutionApplyArtifact {
        EvolutionApplyArtifact {
            directory: ".octopus/evolution/field-mini-task".to_string(),
            plan_path: ".octopus/evolution/field-mini-task/APPLY_PLAN.md".to_string(),
            patch_path: patch_path.map(str::to_string),
            json_path: ".octopus/evolution/field-mini-task/apply.json".to_string(),
        }
    }

    fn test_recommendation() -> EvolutionRecommendation {
        EvolutionRecommendation {
            tentacle_id: "field-mini-task".to_string(),
            objective: "repair harness".to_string(),
            candidate_id: "01-runtime-code".to_string(),
            candidate_title: "Runtime patch".to_string(),
            surface_id: "runtime_code".to_string(),
            outcome_count: 0,
            feed_trace_count: 0,
            check_history_count: 0,
            recommendation_score: 1.0,
            reason: "test".to_string(),
            apply: test_plan(),
        }
    }
}
