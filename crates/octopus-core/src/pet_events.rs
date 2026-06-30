use octopus_core::{HarnessState, PetEvent, Status};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub(crate) struct PetEventDetails {
    pub(crate) stage: Option<String>,
    pub(crate) error_class: Option<String>,
}

pub(crate) fn record_and_save_with_details(
    state_path: &Path,
    state: &mut HarnessState,
    pet_state: &str,
    source: impl Into<String>,
    summary: impl Into<String>,
    status: Status,
    details: PetEventDetails,
) -> Result<PetEvent, String> {
    let summary = summary.into();
    let mut event = state.record_pet_event(pet_state, source, summary_text(&summary), status);
    event.stage = details.stage;
    event.error_class = details.error_class;
    state.last_pet_event = Some(event.clone());
    state.save(state_path).map_err(|error| error.to_string())?;
    append_event(state_path, &event)?;
    Ok(event)
}

pub(crate) fn append_latest(state_path: &Path, state: &HarnessState) -> Result<(), String> {
    if let Some(event) = &state.last_pet_event {
        append_event(state_path, event)?;
    }
    Ok(())
}

pub(crate) fn append_event(state_path: &Path, event: &PetEvent) -> Result<(), String> {
    let path = event_log_path(state_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| error.to_string())?;
    let line = serde_json::to_string(event).map_err(|error| error.to_string())?;
    writeln!(file, "{line}").map_err(|error| error.to_string())
}

pub(crate) fn event_log_path(state_path: &Path) -> PathBuf {
    state_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("pet-events.jsonl")
}

pub(crate) fn summary_text(value: &str) -> String {
    const LIMIT: usize = 180;
    let mut summary = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if summary.chars().count() > LIMIT {
        summary = summary.chars().take(LIMIT).collect::<String>();
        summary.push_str("...");
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pet_event_log_path_sits_next_to_state() {
        assert_eq!(
            event_log_path(Path::new(".octopus/state.json")),
            PathBuf::from(".octopus/pet-events.jsonl")
        );
        assert_eq!(
            event_log_path(Path::new("state.json")),
            PathBuf::from("pet-events.jsonl")
        );
    }

    #[test]
    fn detailed_event_persists_stage_and_error_class() {
        let dir =
            std::env::temp_dir().join(format!("octopus-pet-events-details-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let state_path = dir.join("state.json");
        let mut state = HarnessState::default();

        let event = record_and_save_with_details(
            &state_path,
            &mut state,
            "blocked",
            "evolve recommend field-mini-task",
            "codex provider timed out after 60s",
            Status::Failed,
            PetEventDetails {
                stage: Some("planning".to_string()),
                error_class: Some("provider_timeout".to_string()),
            },
        )
        .unwrap();

        assert_eq!(event.stage.as_deref(), Some("planning"));
        assert_eq!(event.error_class.as_deref(), Some("provider_timeout"));
        let restored = HarnessState::load(&state_path).unwrap();
        assert_eq!(
            restored.last_pet_event.unwrap().error_class.as_deref(),
            Some("provider_timeout")
        );

        let _ = fs::remove_dir_all(dir);
    }
}
