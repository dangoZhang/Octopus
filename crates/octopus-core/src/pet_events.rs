use octopus_core::{HarnessState, PetEvent, Status};
use std::path::Path;

pub(crate) fn record_and_save(
    state_path: &Path,
    state: &mut HarnessState,
    pet_state: &str,
    source: impl Into<String>,
    summary: impl Into<String>,
    status: Status,
) -> Result<PetEvent, String> {
    let summary = summary.into();
    let event = state.record_pet_event(pet_state, source, summary_text(&summary), status);
    state.save(state_path).map_err(|error| error.to_string())?;
    Ok(event)
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
