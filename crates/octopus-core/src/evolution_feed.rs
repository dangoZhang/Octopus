use crate::desktop_pet::{launch_desktop_pet, DesktopPetConfig, DesktopPetReport};
use crate::evolution_cycle::{
    record_stage_event, record_stage_event_with_error, EvolutionDriveStage,
};
use crate::need_runner::{
    empty_parallel_evolution_batch_report, record_parallel_evolution_action_event,
    run_queued_need_indices_with_observer_state, NeedRunBatchReport,
};
use crate::pet_events;
use octopus_core::{FieldTrajectoryReport, HarnessState, ParallelEvolutionRun, Status};
use std::path::Path;

pub(crate) struct EvolutionFeedCycleArgs {
    pub(crate) tentacle_id: String,
    pub(crate) objective: String,
    pub(crate) workers: usize,
    pub(crate) open_pet: bool,
}

pub(crate) struct EvolutionFeedCycleReport {
    pub(crate) stage: EvolutionDriveStage,
    pub(crate) run: ParallelEvolutionRun,
    pub(crate) auto_feed: NeedRunBatchReport,
    pub(crate) desktop_pet: Option<DesktopPetReport>,
    pub(crate) field_summary: Option<FieldTrajectoryReport>,
    pub(crate) next: Vec<String>,
}

pub(crate) fn run_field_mini_task_feed_cycle(
    state_path: &Path,
    mut state: HarnessState,
    args: EvolutionFeedCycleArgs,
) -> Result<EvolutionFeedCycleReport, String> {
    let mut run = state.start_parallel_evolution(args.objective, args.workers)?;
    state.save(state_path).map_err(|error| error.to_string())?;
    let desktop_pet = if args.open_pet {
        Some(launch_desktop_pet(
            state_path,
            DesktopPetConfig {
                worker_cap: run.worker_count.max(1),
            },
        )?)
    } else {
        None
    };
    let queued_indices = run
        .workers
        .iter()
        .filter_map(|worker| worker.queued_need_index)
        .collect::<Vec<_>>();
    if record_parallel_evolution_action_event(&mut state, &run).is_some() {
        state.save(state_path).map_err(|error| error.to_string())?;
        pet_events::append_latest(state_path, &state)?;
    }
    let (auto_feed, mut next_state) = if queued_indices.is_empty() {
        (empty_parallel_evolution_batch_report(&state), state)
    } else {
        run_queued_need_indices_with_observer_state(state, &queued_indices, Some(state_path))?
    };
    for feed_report in &auto_feed.reports {
        let status = feed_report
            .verifier_result
            .as_ref()
            .map(|result| result.status.clone())
            .unwrap_or_else(|| feed_report.feed.status.clone());
        if let Some(updated) = next_state.update_parallel_evolution_worker_result(
            run.index,
            feed_report.taken.item.index,
            feed_report.feed_trace_index,
            feed_report
                .verifier_result
                .as_ref()
                .map(|result| result.index),
            status,
        ) {
            run = updated;
        }
    }
    next_state
        .save(state_path)
        .map_err(|error| error.to_string())?;
    pet_events::append_latest(state_path, &next_state)?;
    let field_summary = next_state
        .field_trajectory_report_with_state(Some(state_path))
        .ok();
    let (stage, next) = if auto_feed.ran == 0 {
        let (pet_state, status, summary, error_class) = missing_feed_event_status(run.worker_count);
        if let Some(error_class) = error_class {
            record_stage_event_with_error(
                state_path,
                &mut next_state,
                EvolutionDriveStage::Feeding,
                pet_state,
                &args.tentacle_id,
                summary,
                status,
                Some(error_class.to_string()),
            )?;
        } else {
            record_stage_event(
                state_path,
                &mut next_state,
                EvolutionDriveStage::Feeding,
                pet_state,
                &args.tentacle_id,
                summary,
                status,
            )?;
        }
        (
            EvolutionDriveStage::Feeding,
            if auto_feed.next.is_empty() {
                vec!["octopus fields summary".to_string()]
            } else {
                auto_feed.next.clone()
            },
        )
    } else {
        let (pet_state, status, summary) = fed_event_status(&auto_feed);
        record_stage_event(
            state_path,
            &mut next_state,
            EvolutionDriveStage::Fed,
            pet_state,
            &args.tentacle_id,
            summary,
            status,
        )?;
        (
            EvolutionDriveStage::Fed,
            field_summary
                .as_ref()
                .map(|summary| summary.next.clone())
                .unwrap_or_else(|| vec!["octopus fields summary".to_string()]),
        )
    };
    Ok(EvolutionFeedCycleReport {
        stage,
        run,
        auto_feed,
        desktop_pet,
        field_summary,
        next,
    })
}

fn fed_event_status(auto_feed: &NeedRunBatchReport) -> (&'static str, Status, String) {
    let statuses = auto_feed
        .reports
        .iter()
        .map(|report| {
            report
                .verifier_result
                .as_ref()
                .map(|result| result.status.clone())
                .unwrap_or_else(|| report.feed.status.clone())
        })
        .collect::<Vec<_>>();
    fed_status_from_statuses(auto_feed.ran, &statuses)
}

fn missing_feed_event_status(
    worker_count: usize,
) -> (&'static str, Status, &'static str, Option<&'static str>) {
    if worker_count == 0 {
        (
            "success",
            Status::Satisfied,
            "harness-only repair completed; no queued Need requested",
            None,
        )
    } else {
        (
            "blocked",
            Status::Failed,
            "no queued Need reached Feed",
            Some("feed_missing_queued_need"),
        )
    }
}

fn fed_status_from_statuses(ran: usize, statuses: &[Status]) -> (&'static str, Status, String) {
    if statuses
        .iter()
        .any(|status| matches!(status, Status::Failed | Status::Unsupported))
    {
        (
            "blocked",
            Status::Failed,
            format!("fed {ran} queued Need(s); some failed"),
        )
    } else if statuses.iter().all(|status| *status == Status::Satisfied) {
        (
            "feed",
            Status::Satisfied,
            format!("fed {ran} queued Need(s)"),
        )
    } else {
        (
            "harness",
            Status::Partial,
            format!("fed {ran} queued Need(s); some partial"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fed_event_status_keeps_partial_visible() {
        let (pet_state, status, summary) =
            fed_status_from_statuses(2, &[Status::Satisfied, Status::Partial]);

        assert_eq!(pet_state, "harness");
        assert_eq!(status, Status::Partial);
        assert!(summary.contains("some partial"));
    }

    #[test]
    fn fed_event_status_blocks_failed_batches() {
        let (pet_state, status, summary) = fed_status_from_statuses(1, &[Status::Failed]);

        assert_eq!(pet_state, "blocked");
        assert_eq!(status, Status::Failed);
        assert!(summary.contains("some failed"));
    }

    #[test]
    fn fed_event_status_marks_all_satisfied_as_feed() {
        let (pet_state, status, summary) =
            fed_status_from_statuses(2, &[Status::Satisfied, Status::Satisfied]);

        assert_eq!(pet_state, "feed");
        assert_eq!(status, Status::Satisfied);
        assert_eq!(summary, "fed 2 queued Need(s)");
    }

    #[test]
    fn zero_worker_repair_is_not_a_missing_feed_failure() {
        let (pet_state, status, summary, error_class) = missing_feed_event_status(0);

        assert_eq!(pet_state, "success");
        assert_eq!(status, Status::Satisfied);
        assert!(summary.contains("harness-only repair"));
        assert_eq!(error_class, None);
    }

    #[test]
    fn missing_feed_still_blocks_when_workers_were_opened() {
        let (pet_state, status, summary, error_class) = missing_feed_event_status(1);

        assert_eq!(pet_state, "blocked");
        assert_eq!(status, Status::Failed);
        assert_eq!(summary, "no queued Need reached Feed");
        assert_eq!(error_class, Some("feed_missing_queued_need"));
    }
}
