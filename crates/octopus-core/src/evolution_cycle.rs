pub(crate) use crate::observation_contract::{classify_apply_status, classify_planner_error};
use crate::observation_contract::{record_evolution_event, EvolutionStage};
use octopus_core::{HarnessState, PetEvent, Status};
use std::path::Path;

pub(crate) type EvolutionDriveStage = EvolutionStage;

pub(crate) fn record_stage_event(
    state_path: &Path,
    state: &mut HarnessState,
    stage: EvolutionDriveStage,
    pet_state: &str,
    tentacle_id: &str,
    summary: impl Into<String>,
    status: Status,
) -> Result<PetEvent, String> {
    record_evolution_event(
        state_path,
        state,
        stage,
        pet_state,
        format!("evolve drive {tentacle_id}"),
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
    record_evolution_event(
        state_path,
        state,
        stage,
        pet_state,
        format!("evolve drive {tentacle_id}"),
        summary,
        status,
        error_class,
    )
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
