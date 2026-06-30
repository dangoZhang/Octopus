use octopus_core::{HarnessState, NeedQueueStatus, PetEvent, Status};
use serde::Serialize;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const FRESH_SECONDS: u64 = 300;
const HEALTHY_EVENT_STATES: &[&str] = &[
    "idle",
    "need",
    "action",
    "feed",
    "memory",
    "harness",
    "heartbeat",
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
        desktop_process_check(state_path),
        event_log_check(&event_log_path, &log),
        last_event_check(last_event.as_ref()),
        last_stage_check(last_event.as_ref()),
        error_category_check(last_event.as_ref()),
        event_log_contains_last_check(last_event.as_ref(), &log),
        event_state_check(last_event.as_ref()),
        event_freshness_check(last_event.as_ref()),
        active_work_check(state, last_event.as_ref()),
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

fn desktop_process_check(state_path: &Path) -> PetSupervisionCheck {
    if !cfg!(target_os = "macos") {
        return PetSupervisionCheck {
            id: "desktop_process".to_string(),
            status: "pass".to_string(),
            evidence: "desktop pet process check skipped outside macOS".to_string(),
            next: "run this check on macOS when validating the native desktop pet".to_string(),
        };
    }
    if !is_default_octopus_state_path(state_path) {
        return PetSupervisionCheck {
            id: "desktop_process".to_string(),
            status: "pass".to_string(),
            evidence: format!(
                "custom_state_path={}; desktop process check scoped to .octopus/state.json",
                state_path.display()
            ),
            next: "run octopus pet supervise on the project .octopus/state.json observer path"
                .to_string(),
        };
    }
    let expected = absolute_state_path(state_path);
    let output = Command::new("ps").arg("-axo").arg("pid=,args=").output();
    let Ok(output) = output else {
        return PetSupervisionCheck {
            id: "desktop_process".to_string(),
            status: "warn".to_string(),
            evidence: "failed to inspect local processes".to_string(),
            next: "open the native pet with octopus pet desktop --workers 1".to_string(),
        };
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let pet_lines = text
        .lines()
        .filter(|line| line.contains("OctopusDesktopPet"))
        .map(|line| line.trim().to_string())
        .collect::<Vec<_>>();
    let expected_text = expected.display().to_string();
    let matching = pet_lines
        .iter()
        .any(|line| line.contains("--state-path") && line.contains(&expected_text));
    let status = if matching {
        "pass"
    } else if pet_lines.is_empty() {
        "warn"
    } else {
        "warn"
    };
    let evidence = if pet_lines.is_empty() {
        format!("expected_state={expected_text}; process=none")
    } else {
        format!(
            "expected_state={expected_text}; matching={matching}; process_count={}",
            pet_lines.len()
        )
    };
    PetSupervisionCheck {
        id: "desktop_process".to_string(),
        status: status.to_string(),
        evidence,
        next:
            "restart the native observer with the same state path: octopus pet desktop --workers 1"
                .to_string(),
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

fn last_stage_check(last: Option<&PetEvent>) -> PetSupervisionCheck {
    let requires_stage = last
        .map(|event| event.source.starts_with("evolve "))
        .unwrap_or(false);
    let stage = last.and_then(|event| event.stage.as_deref());
    PetSupervisionCheck {
        id: "last_stage".to_string(),
        status: if last.is_none() {
            "fail"
        } else if !requires_stage || stage.is_some() {
            "pass"
        } else {
            "warn"
        }
        .to_string(),
        evidence: stage
            .map(|value| format!("stage={value}"))
            .unwrap_or_else(|| {
                format!("stage=missing; required_for_evolve_drive={requires_stage}")
            }),
        next: "write evolution and Goal runtime events through structured stage-aware pet events"
            .to_string(),
    }
}

fn error_category_check(last: Option<&PetEvent>) -> PetSupervisionCheck {
    let blocked = last
        .map(|event| event.state == "blocked" || event.status == octopus_core::Status::Failed)
        .unwrap_or(false);
    let category = last.and_then(|event| event.error_class.as_deref());
    let status = if last.is_none() {
        "fail"
    } else if !blocked || category.is_some() {
        "pass"
    } else {
        "warn"
    };
    PetSupervisionCheck {
        id: "error_category".to_string(),
        status: status.to_string(),
        evidence: category
            .map(|value| format!("error_class={value}"))
            .unwrap_or_else(|| format!("blocked={blocked}; error_class=missing")),
        next: "classify blocked pet events at the stage that produced the failure".to_string(),
    }
}

fn event_log_contains_last_check(
    last: Option<&PetEvent>,
    log: &EventLogRead,
) -> PetSupervisionCheck {
    let status = if last.is_none() {
        "fail"
    } else if log.contains_event(last.unwrap()) {
        "pass"
    } else {
        "warn"
    };
    PetSupervisionCheck {
        id: "event_log_contains_last".to_string(),
        status: status.to_string(),
        evidence: last
            .map(|event| {
                format!(
                    "last_timestamp={}; found_in_log={}",
                    event.timestamp_secs,
                    log.contains_event(event)
                )
            })
            .unwrap_or_else(|| "last=none; found_in_log=false".to_string()),
        next: "append the latest pet event to pet-events.jsonl whenever state.json is saved"
            .to_string(),
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

fn active_work_check(state: &HarnessState, last: Option<&PetEvent>) -> PetSupervisionCheck {
    let pending_needs = state
        .need_queue
        .iter()
        .filter(|item| item.status == NeedQueueStatus::Pending)
        .count();
    let latest_run = state.parallel_evolution_runs.last();
    let latest_partial_workers = latest_run
        .map(|run| {
            run.workers
                .iter()
                .filter(|worker| worker.status == Status::Partial)
                .count()
        })
        .unwrap_or_default();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let fresh_partial_workers = latest_run
        .map(|run| {
            run.workers
                .iter()
                .filter(|worker| worker.status == Status::Partial)
                .filter(|worker| worker.updated_at_secs > 0)
                .filter(|worker| now.saturating_sub(worker.updated_at_secs) <= FRESH_SECONDS)
                .count()
        })
        .unwrap_or_default();
    let latest_run_label = latest_run
        .map(|run| run.index.to_string())
        .unwrap_or_else(|| "none".to_string());
    let has_active_work = pending_needs > 0 || fresh_partial_workers > 0;
    let fresh = last.map(pet_event_fresh).unwrap_or(false);
    let status = if has_active_work || fresh {
        "pass"
    } else {
        "warn"
    };
    PetSupervisionCheck {
        id: "active_work".to_string(),
        status: status.to_string(),
        evidence: format!(
            "pending_needs={pending_needs}; latest_partial_workers={latest_partial_workers}; fresh_partial_workers={fresh_partial_workers}; latest_parallel_run={latest_run_label}; fresh_event={fresh}"
        ),
        next: "start a real Goal/Need/evolve loop; desktop pet must observe work, not refresh itself"
            .to_string(),
    }
}

#[derive(Clone, Debug, Default)]
struct EventLogRead {
    exists: bool,
    count: usize,
    corrupt_count: usize,
    events: Vec<PetEvent>,
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
        match serde_json::from_str::<PetEvent>(line) {
            Ok(event) => read.events.push(event),
            Err(_) => read.corrupt_count += 1,
        }
    }
    read
}

impl EventLogRead {
    fn contains_event(&self, expected: &PetEvent) -> bool {
        self.events.iter().any(|event| {
            event.timestamp_secs == expected.timestamp_secs
                && event.state == expected.state
                && event.source == expected.source
                && event.summary == expected.summary
                && event.status == expected.status
                && event.stage == expected.stage
                && event.error_class == expected.error_class
        })
    }
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

fn absolute_state_path(state_path: &Path) -> PathBuf {
    if state_path.is_absolute() {
        return state_path.to_path_buf();
    }
    env::current_dir()
        .map(|cwd| cwd.join(state_path))
        .unwrap_or_else(|_| state_path.to_path_buf())
}

fn is_default_octopus_state_path(state_path: &Path) -> bool {
    state_path
        .components()
        .rev()
        .take(2)
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        == vec!["state.json".to_string(), ".octopus".to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use octopus_core::{
        GoalNeedSuggestion, NeedKind, ParallelEvolutionRun, ParallelEvolutionWorker,
    };

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

        assert!(report
            .checks
            .iter()
            .filter(|check| check.id != "desktop_process")
            .all(|check| check.status == "pass"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn pet_supervision_warns_when_audit_log_misses_latest_event() {
        let dir = std::env::temp_dir().join(format!(
            "octopus-pet-supervision-missing-log-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let state_path = dir.join("state.json");
        let mut state = HarnessState::default();
        state.record_pet_event("feed", "field verifier", "latest feed", Status::Satisfied);
        state.save(&state_path).unwrap();

        let report = pet_supervision_report(&state_path, &state);

        assert_eq!(report.status, "warn");
        assert!(report
            .checks
            .iter()
            .any(|check| { check.id == "event_log_contains_last" && check.status == "warn" }));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn pet_supervision_accepts_heartbeat_without_evolution_stage() {
        let dir = std::env::temp_dir().join(format!(
            "octopus-pet-supervision-heartbeat-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let state_path = dir.join("state.json");
        let mut state = HarnessState::default();
        let event = state.record_pet_event("heartbeat", "heartbeat", "alive", Status::Satisfied);
        state.save(&state_path).unwrap();
        crate::pet_events::append_event(&state_path, &event).unwrap();

        let report = pet_supervision_report(&state_path, &state);

        assert!(report
            .checks
            .iter()
            .any(|check| check.id == "event_state" && check.status == "pass"));
        assert!(report
            .checks
            .iter()
            .any(|check| check.id == "last_stage" && check.status == "pass"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn pet_supervision_warns_when_stale_and_no_active_work_exists() {
        let dir = std::env::temp_dir().join(format!(
            "octopus-pet-supervision-idle-stale-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let state_path = dir.join("state.json");
        let mut state = HarnessState::default();
        let mut event =
            state.record_pet_event("heartbeat", "heartbeat", "alive", Status::Satisfied);
        event.timestamp_secs = 1;
        state.last_pet_event = Some(event.clone());
        state.save(&state_path).unwrap();
        crate::pet_events::append_event(&state_path, &event).unwrap();

        let report = pet_supervision_report(&state_path, &state);

        assert!(report
            .checks
            .iter()
            .any(|check| check.id == "active_work" && check.status == "warn"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn pet_supervision_sees_pending_need_as_active_work() {
        let dir = std::env::temp_dir().join(format!(
            "octopus-pet-supervision-active-need-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let state_path = dir.join("state.json");
        let mut state = HarnessState::default();
        let mut event =
            state.record_pet_event("need", "need queue", "queued verify", Status::Partial);
        event.timestamp_secs = 1;
        state.last_pet_event = Some(event.clone());
        state.need_queue.push(octopus_core::NeedQueueItem {
            index: 1,
            need: GoalNeedSuggestion {
                kind: NeedKind::Verify,
                query: "verify active work".to_string(),
            },
            context: Default::default(),
            source: "test".to_string(),
            prompt: "test".to_string(),
            summary: "test".to_string(),
            status: NeedQueueStatus::Pending,
        });
        state.save(&state_path).unwrap();
        crate::pet_events::append_event(&state_path, &event).unwrap();

        let report = pet_supervision_report(&state_path, &state);

        assert!(report
            .checks
            .iter()
            .any(|check| check.id == "active_work" && check.status == "pass"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn pet_supervision_ignores_stale_partial_workers_as_active_work() {
        let dir = std::env::temp_dir().join(format!(
            "octopus-pet-supervision-stale-worker-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let state_path = dir.join("state.json");
        let mut state = HarnessState::default();
        let mut event = state.record_pet_event(
            "harness",
            "parallel evolution",
            "old worker",
            Status::Partial,
        );
        event.timestamp_secs = 1;
        state.last_pet_event = Some(event.clone());
        state.parallel_evolution_runs.push(ParallelEvolutionRun {
            index: 1,
            objective: "old run".to_string(),
            field_pool_size: 1,
            field_pool_policy: "test".to_string(),
            candidate_fields: vec!["math".to_string()],
            requested_worker_count: 1,
            worker_count: 1,
            worker_policy: "test".to_string(),
            workers: vec![ParallelEvolutionWorker {
                id: "worker-1".to_string(),
                field: "math".to_string(),
                mini_task: None,
                goal: "test".to_string(),
                updated_at_secs: 1,
                queued_need_index: None,
                source_trace_index: None,
                verifier_result_index: None,
                status: Status::Partial,
                next_action: "test".to_string(),
            }],
            summary: "old run".to_string(),
        });
        state.save(&state_path).unwrap();
        crate::pet_events::append_event(&state_path, &event).unwrap();

        let report = pet_supervision_report(&state_path, &state);

        assert!(report
            .checks
            .iter()
            .any(|check| check.id == "active_work" && check.status == "warn"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn pet_supervision_accepts_structured_evolution_block() {
        let dir = std::env::temp_dir().join(format!(
            "octopus-pet-supervision-structured-block-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let state_path = dir.join("state.json");
        let mut state = HarnessState::default();
        let mut event = state.record_pet_event(
            "blocked",
            "evolve recommend field-mini-task",
            "codex provider timed out after 60s",
            Status::Failed,
        );
        event.stage = Some("planning".to_string());
        event.error_class = Some("provider_timeout".to_string());
        state.last_pet_event = Some(event.clone());
        state.save(&state_path).unwrap();
        crate::pet_events::append_event(&state_path, &event).unwrap();

        let report = pet_supervision_report(&state_path, &state);

        assert!(report
            .checks
            .iter()
            .any(|check| check.id == "last_stage" && check.status == "pass"));
        assert!(report
            .checks
            .iter()
            .any(|check| check.id == "error_category" && check.status == "pass"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn pet_supervision_flags_unstructured_evolution_event() {
        let dir = std::env::temp_dir().join(format!(
            "octopus-pet-supervision-unstructured-evolve-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let state_path = dir.join("state.json");
        let mut state = HarnessState::default();
        let event = state.record_pet_event(
            "blocked",
            "evolve recommend field-mini-task",
            "codex provider timed out after 60s",
            Status::Failed,
        );
        state.save(&state_path).unwrap();
        crate::pet_events::append_event(&state_path, &event).unwrap();

        let report = pet_supervision_report(&state_path, &state);

        assert!(report
            .checks
            .iter()
            .any(|check| check.id == "last_stage" && check.status == "warn"));
        assert!(report
            .checks
            .iter()
            .any(|check| check.id == "error_category" && check.status == "warn"));

        let _ = fs::remove_dir_all(dir);
    }
}
