use octopus_core::{HarnessState, PetEvent};
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const FRESH_SECONDS: u64 = 300;
const HEALTHY_EVENT_STATES: &[&str] = &[
    "idle",
    "need",
    "action",
    "feed",
    "memory",
    "harness",
    "evolution",
    "blocked",
    "success",
];

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PetSupervisionReport {
    pub(crate) status: String,
    pub(crate) state_path: String,
    pub(crate) event_log_path: String,
    pub(crate) event_log_count: usize,
    pub(crate) last_event: Option<PetEvent>,
    pub(crate) checks: Vec<PetSupervisionCheck>,
    pub(crate) next: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PetSupervisionCheck {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) evidence: String,
    pub(crate) next: String,
}

pub(crate) fn pet_supervision_report(
    state_path: &Path,
    state: &HarnessState,
) -> PetSupervisionReport {
    let event_log_path = crate::pet_events::event_log_path(state_path);
    let log = read_event_log(&event_log_path);
    let last_event = state.last_pet_event.clone();
    let checks = vec![
        state_file_check(state_path),
        event_log_check(&event_log_path, &log),
        last_event_check(last_event.as_ref()),
        event_state_check(last_event.as_ref()),
        event_freshness_check(last_event.as_ref()),
    ];
    let status = aggregate_status(&checks);
    let next = aggregate_next(&checks);
    PetSupervisionReport {
        status,
        state_path: state_path.display().to_string(),
        event_log_path: event_log_path.display().to_string(),
        event_log_count: log.count,
        last_event,
        checks,
        next,
    }
}

pub(crate) fn strategy_evidence(report: &PetSupervisionReport) -> String {
    let check_statuses = report
        .checks
        .iter()
        .map(|check| format!("{}={}", check.id, check.status))
        .collect::<Vec<_>>()
        .join(",");
    let last = report
        .last_event
        .as_ref()
        .map(|event| format!("last={} {}/{}", event.state, event.source, event.summary))
        .unwrap_or_else(|| "last=none".to_string());
    format!(
        "state_path={}; event_log={} count={}; {}; {}",
        report.state_path, report.event_log_path, report.event_log_count, check_statuses, last
    )
}

pub(crate) fn pet_event_fresh(event: &PetEvent) -> bool {
    match event_age_seconds(event) {
        Some(age) => (0..=FRESH_SECONDS as i64).contains(&age),
        None => false,
    }
}

fn state_file_check(state_path: &Path) -> PetSupervisionCheck {
    let pass = state_path.exists();
    PetSupervisionCheck {
        id: "state_file".to_string(),
        status: pass_fail(pass),
        evidence: format!("path={}; exists={pass}", state_path.display()),
        next: "start Octopus with the same --state path used by the desktop pet".to_string(),
    }
}

fn event_log_check(path: &Path, log: &EventLogRead) -> PetSupervisionCheck {
    let status = if log.corrupt_count > 0 {
        "fail"
    } else if !log.exists || log.count == 0 {
        "warn"
    } else {
        "pass"
    };
    PetSupervisionCheck {
        id: "event_log".to_string(),
        status: status.to_string(),
        evidence: format!(
            "path={}; exists={}; count={}; corrupt={}",
            path.display(),
            log.exists,
            log.count,
            log.corrupt_count
        ),
        next: "run a real Goal step and keep pet event writes on the unified pet_events path"
            .to_string(),
    }
}

fn last_event_check(last: Option<&PetEvent>) -> PetSupervisionCheck {
    PetSupervisionCheck {
        id: "last_event".to_string(),
        status: pass_fail(last.is_some()),
        evidence: last
            .map(|event| {
                format!(
                    "timestamp={}; state={}; source={}",
                    event.timestamp_secs, event.state, event.source
                )
            })
            .unwrap_or_else(|| "none".to_string()),
        next: "drive one Need through the system so state.json records last_pet_event".to_string(),
    }
}

fn event_state_check(last: Option<&PetEvent>) -> PetSupervisionCheck {
    let valid = last
        .map(|event| HEALTHY_EVENT_STATES.contains(&event.state.as_str()))
        .unwrap_or(false);
    PetSupervisionCheck {
        id: "event_state".to_string(),
        status: pass_fail(valid),
        evidence: last
            .map(|event| {
                format!(
                    "state={}; allowed={}",
                    event.state,
                    HEALTHY_EVENT_STATES.join("|")
                )
            })
            .unwrap_or_else(|| format!("state=none; allowed={}", HEALTHY_EVENT_STATES.join("|"))),
        next: "write pet states through the known Goal/Need/action/Feed/harness state names"
            .to_string(),
    }
}

fn event_freshness_check(last: Option<&PetEvent>) -> PetSupervisionCheck {
    let age = last.and_then(event_age_seconds);
    let fresh = last.map(pet_event_fresh).unwrap_or(false);
    let status = if last.is_none() {
        "fail"
    } else if fresh {
        "pass"
    } else {
        "warn"
    };
    PetSupervisionCheck {
        id: "event_freshness".to_string(),
        status: status.to_string(),
        evidence: format!(
            "fresh={fresh}; age_seconds={}",
            age.map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        ),
        next: "confirm the active Octopus loop is still writing pet events within five minutes"
            .to_string(),
    }
}

#[derive(Clone, Debug, Default)]
struct EventLogRead {
    exists: bool,
    count: usize,
    corrupt_count: usize,
}

fn read_event_log(path: &PathBuf) -> EventLogRead {
    let Ok(content) = fs::read_to_string(path) else {
        return EventLogRead::default();
    };
    let mut read = EventLogRead {
        exists: true,
        ..EventLogRead::default()
    };
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        read.count += 1;
        if serde_json::from_str::<PetEvent>(line).is_err() {
            read.corrupt_count += 1;
        }
    }
    read
}

fn aggregate_status(checks: &[PetSupervisionCheck]) -> String {
    if checks.iter().any(|check| check.status == "fail") {
        "fail".to_string()
    } else if checks.iter().any(|check| check.status == "warn") {
        "warn".to_string()
    } else {
        "pass".to_string()
    }
}

fn aggregate_next(checks: &[PetSupervisionCheck]) -> Vec<String> {
    checks
        .iter()
        .filter(|check| check.status != "pass")
        .map(|check| check.next.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn pass_fail(pass: bool) -> String {
    if pass {
        "pass".to_string()
    } else {
        "fail".to_string()
    }
}

fn event_age_seconds(event: &PetEvent) -> Option<i64> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    Some(now as i64 - event.timestamp_secs as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use octopus_core::Status;

    #[test]
    fn pet_supervision_pinpoints_missing_observation_chain() {
        let report =
            pet_supervision_report(Path::new("missing-state.json"), &HarnessState::default());

        assert_eq!(report.status, "fail");
        assert!(report
            .checks
            .iter()
            .any(|check| check.id == "state_file" && check.status == "fail"));
        assert!(report
            .checks
            .iter()
            .any(|check| check.id == "last_event" && check.status == "fail"));
    }

    #[test]
    fn pet_supervision_passes_with_fresh_state_and_audit_log() {
        let dir =
            std::env::temp_dir().join(format!("octopus-pet-supervision-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let state_path = dir.join("state.json");
        let mut state = HarnessState::default();
        let event = state.record_pet_event("harness", "test", "evolved routes", Status::Satisfied);
        state.save(&state_path).unwrap();
        crate::pet_events::append_event(&state_path, &event).unwrap();

        let report = pet_supervision_report(&state_path, &state);

        assert_eq!(report.status, "pass");
        assert!(report.checks.iter().all(|check| check.status == "pass"));

        let _ = fs::remove_dir_all(dir);
    }
}
