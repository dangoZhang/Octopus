use crate::pet_events::{self, PetEventDetails};
use octopus_core::{HarnessState, PetEvent, Status};
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvolutionStage {
    Planning,
    Recommended,
    Applying,
    ApplyRetryPlanning,
    Checking,
    Feeding,
    Fed,
}

impl EvolutionStage {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Planning => "planning",
            Self::Recommended => "recommended",
            Self::Applying => "applying",
            Self::ApplyRetryPlanning => "apply_retry_planning",
            Self::Checking => "checking",
            Self::Feeding => "feeding",
            Self::Fed => "fed",
        }
    }
}

pub(crate) fn record_evolution_event(
    state_path: &Path,
    state: &mut HarnessState,
    stage: EvolutionStage,
    pet_state: &str,
    source: impl Into<String>,
    summary: impl Into<String>,
    status: Status,
    error_class: Option<String>,
) -> Result<PetEvent, String> {
    pet_events::record_and_save_with_details(
        state_path,
        state,
        pet_state,
        source,
        summary,
        status,
        PetEventDetails {
            stage: Some(stage.as_str().to_string()),
            error_class,
        },
    )
}

pub(crate) fn classify_planner_error(error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("timed out") || lower.contains("timeout") {
        "provider_timeout".to_string()
    } else if lower.contains("provider") || lower.contains("llm") || lower.contains("codex") {
        "provider_error".to_string()
    } else if lower.contains("manifest") {
        "manifest_error".to_string()
    } else if lower.contains("artifact") || lower.contains("write") || lower.contains("path") {
        "artifact_error".to_string()
    } else {
        "planner_error".to_string()
    }
}

pub(crate) fn classify_apply_status(status: &str, stderr: &str) -> String {
    let status = status.to_ascii_lowercase();
    let stderr = stderr.to_ascii_lowercase();
    if status.contains("not_authorized") {
        "patch_not_authorized".to_string()
    } else if status.contains("no_suggested_patch") || status.contains("missing_patch") {
        "patch_missing".to_string()
    } else if status.contains("check_failed") || stderr.contains("corrupt patch") {
        "patch_check_failed".to_string()
    } else if status.contains("failed_to_start") {
        "patch_apply_unavailable".to_string()
    } else {
        "patch_apply_failed".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn evolution_event_records_stage_and_error_for_observer() {
        let dir = std::env::temp_dir().join(format!(
            "octopus-observation-contract-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let state_path = dir.join("state.json");
        let mut state = HarnessState::default();

        let event = record_evolution_event(
            &state_path,
            &mut state,
            EvolutionStage::Planning,
            "blocked",
            "evolve recommend field-mini-task",
            "codex provider timed out after 60s",
            Status::Failed,
            Some("provider_timeout".to_string()),
        )
        .unwrap();

        assert_eq!(event.stage.as_deref(), Some("planning"));
        assert_eq!(event.error_class.as_deref(), Some("provider_timeout"));
        let restored = HarnessState::load(&state_path).unwrap();
        assert_eq!(
            restored.last_pet_event.unwrap().stage.as_deref(),
            Some("planning")
        );
        assert!(crate::pet_events::event_log_path(&state_path).exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn planner_timeout_class_is_stable() {
        assert_eq!(
            classify_planner_error("codex provider timed out after 60s"),
            "provider_timeout"
        );
    }

    #[test]
    fn apply_check_failure_class_is_stable() {
        assert_eq!(
            classify_apply_status("check_failed", "error: corrupt patch"),
            "patch_check_failed"
        );
    }
}
