use crate::desktop_pet::{launch_desktop_pet, DesktopPetConfig, DesktopPetReport};
use crate::evolution_cycle::{
    record_stage_event, record_stage_event_with_error, EvolutionDriveStage,
};
use crate::{
    empty_parallel_evolution_batch_report, pet_events, record_parallel_evolution_action_event,
    run_queued_need_indices_with_observer_state, NeedRunBatchReport,
};
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
        record_stage_event_with_error(
            state_path,
            &mut next_state,
            EvolutionDriveStage::Feeding,
            "blocked",
            &args.tentacle_id,
            "no queued Need reached Feed",
            Status::Failed,
            Some("feed_missing_queued_need".to_string()),
        )?;
        (
            EvolutionDriveStage::Feeding,
            if auto_feed.next.is_empty() {
                vec!["octopus fields summary".to_string()]
            } else {
                auto_feed.next.clone()
            },
        )
    } else {
        record_stage_event(
            state_path,
            &mut next_state,
            EvolutionDriveStage::Fed,
            "feed",
            &args.tentacle_id,
            format!("fed {} queued Need(s)", auto_feed.ran),
            Status::Satisfied,
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
