use crate::evolution_apply::{
    apply_authorized_suggested_patch, live_apply_summary, EvolutionLiveApplyReport,
};
use crate::evolution_cycle::{
    classify_apply_status, classify_planner_error, record_stage_event,
    record_stage_event_with_error, EvolutionDriveStage,
};
use crate::evolution_drive_surface::{EvolutionDriveArgs, EvolutionDriveReport};
use crate::evolution_feed::{run_field_mini_task_feed_cycle, EvolutionFeedCycleArgs};
use crate::evolution_plan::{recommend_current_patch, retry_apply_objective};
use crate::shell_words::shell_arg;
use crate::{check_report, pet_events, record_check_report};
use octopus_core::{HarnessState, Status};
use std::env;
use std::path::Path;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_PLANNING_TIMEOUT_SECONDS: u64 = 90;
const DEFAULT_PLANNING_PROGRESS_SECONDS: u64 = 15;

pub(crate) fn drive_evolution_cycle(
    state_path: &Path,
    args: EvolutionDriveArgs,
) -> Result<EvolutionDriveReport, String> {
    let mut loaded = HarnessState::load(state_path).map_err(|error| error.to_string())?;
    record_drive_event(
        state_path,
        &mut loaded,
        EvolutionDriveStage::Planning,
        "evolution",
        &args.tentacle_id,
        format!("planning {}", pet_events::summary_text(&args.objective)),
        Status::Partial,
    )?;
    let cwd = env::current_dir().map_err(|error| error.to_string())?;
    let mut report = EvolutionDriveReport {
        tentacle_id: args.tentacle_id.clone(),
        objective: args.objective.clone(),
        stage: EvolutionDriveStage::Planning.as_str().to_string(),
        recommendation: None,
        evolution_artifact: None,
        apply_artifact: None,
        live_apply: None,
        check: None,
        run: None,
        auto_feed: None,
        desktop_pet: None,
        field_summary: None,
        next: vec![format!(
            "octopus evolve recommend {} {}",
            shell_arg(&args.tentacle_id),
            shell_arg(&args.objective)
        )],
    };

    let (evolution_artifact, recommendation, apply_artifact) =
        match recommend_current_patch_observed(
            state_path,
            &mut loaded,
            &cwd,
            &args.tentacle_id,
            &args.objective,
            EvolutionDriveStage::Planning,
        ) {
            Ok(result) => (
                result.evolution_artifact,
                result.recommendation,
                result.apply_artifact,
            ),
            Err(error) => {
                record_drive_error_event(
                    state_path,
                    &mut loaded,
                    EvolutionDriveStage::Planning,
                    "blocked",
                    &args.tentacle_id,
                    error.clone(),
                    Status::Failed,
                    classify_planner_error(&error),
                )?;
                return Err(error);
            }
        };
    record_drive_event(
        state_path,
        &mut loaded,
        EvolutionDriveStage::Recommended,
        "evolution",
        &args.tentacle_id,
        format!(
            "recommended {}: {}",
            recommendation.candidate_id,
            pet_events::summary_text(&recommendation.candidate_title)
        ),
        Status::Satisfied,
    )?;
    report.stage = EvolutionDriveStage::Recommended.as_str().to_string();
    report.evolution_artifact = Some(evolution_artifact);
    report.apply_artifact = Some(apply_artifact.clone());
    report.next = vec![format!(
        "octopus evolve apply {} {}",
        shell_arg(&args.tentacle_id),
        shell_arg(&recommendation.candidate_id)
    )];
    report.recommendation = Some(recommendation.clone());

    let mut live_apply =
        apply_authorized_suggested_patch(&cwd, &recommendation.apply, &apply_artifact);
    let apply_state = if live_apply.applied {
        "success"
    } else {
        "blocked"
    };
    record_apply_event(
        state_path,
        &mut loaded,
        &args.tentacle_id,
        apply_state,
        live_apply_summary(&recommendation, &live_apply),
        if live_apply.applied {
            Status::Satisfied
        } else {
            Status::Failed
        },
        &live_apply,
    )?;
    report.live_apply = Some(live_apply.clone());

    if !live_apply.applied {
        let retry_objective = retry_apply_objective(&args.objective, &live_apply);
        record_drive_event(
            state_path,
            &mut loaded,
            EvolutionDriveStage::ApplyRetryPlanning,
            "evolution",
            &args.tentacle_id,
            format!(
                "retrying patch after apply check failed: {}",
                live_apply.status
            ),
            Status::Partial,
        )?;
        match recommend_current_patch_observed(
            state_path,
            &mut loaded,
            &cwd,
            &args.tentacle_id,
            &retry_objective,
            EvolutionDriveStage::ApplyRetryPlanning,
        ) {
            Ok(result) => {
                report.stage = EvolutionDriveStage::ApplyRetryPlanning.as_str().to_string();
                report.objective = retry_objective;
                report.evolution_artifact = Some(result.evolution_artifact);
                report.apply_artifact = Some(result.apply_artifact.clone());
                report.next = vec![format!(
                    "octopus evolve apply {} {}",
                    shell_arg(&args.tentacle_id),
                    shell_arg(&result.recommendation.candidate_id)
                )];
                live_apply = apply_authorized_suggested_patch(
                    &cwd,
                    &result.recommendation.apply,
                    &result.apply_artifact,
                );
                let retry_state = if live_apply.applied {
                    "success"
                } else {
                    "blocked"
                };
                record_apply_event(
                    state_path,
                    &mut loaded,
                    &args.tentacle_id,
                    retry_state,
                    live_apply_summary(&result.recommendation, &live_apply),
                    if live_apply.applied {
                        Status::Satisfied
                    } else {
                        Status::Failed
                    },
                    &live_apply,
                )?;
                report.live_apply = Some(live_apply.clone());
                report.recommendation = Some(result.recommendation);
            }
            Err(error) => {
                record_drive_error_event(
                    state_path,
                    &mut loaded,
                    EvolutionDriveStage::ApplyRetryPlanning,
                    "blocked",
                    &args.tentacle_id,
                    format!("retry planning failed: {error}"),
                    Status::Failed,
                    classify_planner_error(&error),
                )?;
                report.stage = EvolutionDriveStage::ApplyRetryPlanning.as_str().to_string();
                report.objective = retry_objective;
                report.next = vec![format!(
                    "octopus evolve recommend {} {}",
                    shell_arg(&args.tentacle_id),
                    shell_arg("repair failed self-evolution apply retry")
                )];
                return Ok(report);
            }
        }
    }

    if !live_apply.applied {
        let candidate_id = report
            .recommendation
            .as_ref()
            .map(|recommendation| recommendation.candidate_id.as_str())
            .unwrap_or(recommendation.candidate_id.as_str());
        report.stage = EvolutionDriveStage::Applying.as_str().to_string();
        report.next = vec![format!(
            "octopus evolve apply {} {}",
            shell_arg(&args.tentacle_id),
            shell_arg(candidate_id)
        )];
        return Ok(report);
    }

    record_drive_event(
        state_path,
        &mut loaded,
        EvolutionDriveStage::Checking,
        "harness",
        &args.tentacle_id,
        "running configured harness checks".to_string(),
        Status::Partial,
    )?;
    let mut check = check_report(&args.tentacle_id, None)?;
    record_check_report(&mut loaded, &mut check);
    loaded.save(state_path).map_err(|error| error.to_string())?;
    pet_events::append_latest(state_path, &loaded)?;
    report.check = Some(check);
    if report.check.as_ref().is_some_and(|check| !check.passed) {
        record_drive_error_event(
            state_path,
            &mut loaded,
            EvolutionDriveStage::Checking,
            "blocked",
            &args.tentacle_id,
            "configured harness check failed",
            Status::Failed,
            "check_failed",
        )?;
        report.stage = EvolutionDriveStage::Checking.as_str().to_string();
        report.next = vec![
            format!("octopus check {}", shell_arg(&args.tentacle_id)),
            format!(
                "octopus evolve recommend {} {}",
                shell_arg(&args.tentacle_id),
                shell_arg("repair failed self-evolution check")
            ),
        ];
        return Ok(report);
    }

    if args.tentacle_id == "field-mini-task" {
        let feed = run_field_mini_task_feed_cycle(
            state_path,
            loaded,
            EvolutionFeedCycleArgs {
                tentacle_id: args.tentacle_id,
                objective: args.objective,
                workers: args.workers,
                open_pet: args.open_pet,
            },
        )?;
        report.stage = feed.stage.as_str().to_string();
        report.run = Some(feed.run);
        report.auto_feed = Some(feed.auto_feed);
        report.desktop_pet = feed.desktop_pet;
        report.field_summary = feed.field_summary;
        report.next = feed.next;
    } else {
        report.stage = EvolutionDriveStage::Checking.as_str().to_string();
        report.next = vec![
            format!("octopus check {}", shell_arg(&report.tentacle_id)),
            "octopus beat 200".to_string(),
        ];
    }

    Ok(report)
}

fn recommend_current_patch_observed(
    state_path: &Path,
    state: &mut HarnessState,
    cwd: &Path,
    tentacle_id: &str,
    objective: &str,
    stage: EvolutionDriveStage,
) -> Result<crate::evolution_plan::EvolutionPatchPlanArtifacts, String> {
    let timeout = Duration::from_secs(planning_timeout_seconds());
    let progress_interval = Duration::from_secs(planning_progress_seconds());
    let started = Instant::now();
    let (tx, rx) = mpsc::channel();
    let cwd = cwd.to_path_buf();
    let tentacle_id_for_thread = tentacle_id.to_string();
    let objective_for_thread = objective.to_string();
    let state_for_thread = state.clone();

    thread::spawn(move || {
        let result = recommend_current_patch(
            &cwd,
            &tentacle_id_for_thread,
            &objective_for_thread,
            &state_for_thread,
        );
        let _ = tx.send(result);
    });

    loop {
        let elapsed = started.elapsed();
        if elapsed >= timeout {
            return Err(planning_timeout_error(timeout));
        }
        let remaining = timeout.saturating_sub(elapsed);
        let wait = remaining.min(progress_interval);
        match rx.recv_timeout(wait) {
            Ok(result) => return result,
            Err(RecvTimeoutError::Timeout) => {
                let elapsed_seconds = started.elapsed().as_secs();
                if elapsed_seconds >= timeout.as_secs() {
                    return Err(planning_timeout_error(timeout));
                }
                record_drive_event(
                    state_path,
                    state,
                    stage,
                    "evolution",
                    tentacle_id,
                    format!(
                        "still planning {} (elapsed={}s, timeout={}s)",
                        pet_events::summary_text(objective),
                        elapsed_seconds,
                        timeout.as_secs()
                    ),
                    Status::Partial,
                )?;
            }
            Err(RecvTimeoutError::Disconnected) => {
                return Err("evolution planner stopped before returning a plan".to_string());
            }
        }
    }
}

fn planning_timeout_error(timeout: Duration) -> String {
    format!(
        "evolution planner timed out after {}s; set OCTOPUS_EVOLVE_PLANNING_TIMEOUT to adjust",
        timeout.as_secs()
    )
}

fn planning_timeout_seconds() -> u64 {
    env_u64(
        "OCTOPUS_EVOLVE_PLANNING_TIMEOUT",
        DEFAULT_PLANNING_TIMEOUT_SECONDS,
    )
    .clamp(10, 600)
}

fn planning_progress_seconds() -> u64 {
    env_u64(
        "OCTOPUS_EVOLVE_PLANNING_PROGRESS_SECONDS",
        DEFAULT_PLANNING_PROGRESS_SECONDS,
    )
    .clamp(5, 120)
}

fn env_u64(key: &str, fallback: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(fallback)
}

fn record_drive_event(
    state_path: &Path,
    state: &mut HarnessState,
    stage: EvolutionDriveStage,
    pet_state: &str,
    tentacle_id: &str,
    summary: String,
    status: Status,
) -> Result<(), String> {
    record_stage_event(
        state_path,
        state,
        stage,
        pet_state,
        tentacle_id,
        summary,
        status,
    )
    .map(|_| ())
}

fn record_drive_error_event(
    state_path: &Path,
    state: &mut HarnessState,
    stage: EvolutionDriveStage,
    pet_state: &str,
    tentacle_id: &str,
    summary: impl Into<String>,
    status: Status,
    error_class: impl Into<String>,
) -> Result<(), String> {
    record_stage_event_with_error(
        state_path,
        state,
        stage,
        pet_state,
        tentacle_id,
        summary,
        status,
        Some(error_class.into()),
    )
    .map(|_| ())
}

fn record_apply_event(
    state_path: &Path,
    state: &mut HarnessState,
    tentacle_id: &str,
    pet_state: &str,
    summary: String,
    status: Status,
    live_apply: &EvolutionLiveApplyReport,
) -> Result<(), String> {
    let error_class = (!live_apply.applied)
        .then(|| classify_apply_status(&live_apply.status, &live_apply.stderr));
    record_stage_event_with_error(
        state_path,
        state,
        EvolutionDriveStage::Applying,
        pet_state,
        tentacle_id,
        summary,
        status,
        error_class,
    )
    .map(|_| ())
}
