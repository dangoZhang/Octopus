use crate::pet_events;
use octopus_core::{HarnessState, PetEvent, Status};
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvolutionDriveStage {
    Planning,
    Recommended,
    Applying,
    ApplyRetryPlanning,
    Checking,
    Feeding,
    Fed,
}

impl EvolutionDriveStage {
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

pub(crate) fn record_stage_event(
    state_path: &Path,
    state: &mut HarnessState,
    stage: EvolutionDriveStage,
    pet_state: &str,
    tentacle_id: &str,
    summary: impl Into<String>,
    status: Status,
) -> Result<PetEvent, String> {
    record_stage_event_with_error(
        state_path,
        state,
        stage,
        pet_state,
        tentacle_id,
        summary,
        status,
        None,
    )
}

pub(crate) fn record_stage_event_with_error(
    state_path: &Path,
    state: &mut HarnessState,
    stage: EvolutionDriveStage,
    pet_state: &str,
    tentacle_id: &str,
    summary: impl Into<String>,
    status: Status,
    error_class: Option<String>,
) -> Result<PetEvent, String> {
    let summary = summary.into();
    let mut event = state.record_pet_event(
        pet_state,
        format!("evolve drive {tentacle_id}"),
        pet_events::summary_text(&summary),
        status,
    );
    event.stage = Some(stage.as_str().to_string());
    event.error_class = error_class;
    state.last_pet_event = Some(event.clone());
    state.save(state_path).map_err(|error| error.to_string())?;
    pet_events::append_event(state_path, &event)?;
    Ok(event)
}

pub(crate) fn classify_planner_error(error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("timed out") || lower.contains("timeout") {
        "provider_timeout".to_string()
    } else if lower.contains("provider") || lower.contains("llm") || lower.contains("codex") {
        "provider_error".to_string()
    } else if lower.contains("manifest") {
        "manifest_error".to_string()
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
    use octopus_core::Status;
    use std::fs;

    #[test]
    fn stage_event_records_error_class_for_pet_observer() {
        let dir = std::env::temp_dir().join(format!(
            "octopus-evolution-cycle-stage-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let state_path = dir.join("state.json");
        let mut state = HarnessState::default();

        let event = record_stage_event_with_error(
            &state_path,
            &mut state,
            EvolutionDriveStage::Planning,
            "blocked",
            "field-mini-task",
            "codex provider timed out after 60s",
            Status::Failed,
            Some("provider_timeout".to_string()),
        )
        .unwrap();

        assert_eq!(event.stage.as_deref(), Some("planning"));
        assert_eq!(event.error_class.as_deref(), Some("provider_timeout"));
        let restored = HarnessState::load(&state_path).unwrap();
        assert_eq!(
            restored.last_pet_event.unwrap().error_class.as_deref(),
            Some("provider_timeout")
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
}
