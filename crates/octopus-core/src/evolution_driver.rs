use crate::desktop_pet::DesktopPetReport;
use crate::evolution_apply::{apply_authorized_suggested_patch, EvolutionLiveApplyReport};
use crate::evolution_cycle::{
    classify_apply_status, classify_planner_error, record_stage_event,
    record_stage_event_with_error, EvolutionDriveStage,
};
use crate::evolution_feed::{run_field_mini_task_feed_cycle, EvolutionFeedCycleArgs};
use crate::shell_words::shell_arg;
use crate::{
    check_report, parse_worker_count_1_to_8, pet_events, propose_evolution_for_cli,
    record_check_report, CheckReport, Language, NeedRunBatchReport,
};
use octopus_core::{
    recommend_tentacle_evolution_apply, write_tentacle_apply_artifacts,
    write_tentacle_evolution_artifacts, EvolutionApplyArtifact, EvolutionArtifact,
    EvolutionRecommendation, FieldTrajectoryReport, HarnessState, ParallelEvolutionRun, Status,
};
use std::env;
use std::path::Path;

#[derive(Debug, serde::Serialize)]
pub(crate) struct EvolutionDriveReport {
    tentacle_id: String,
    objective: String,
    stage: String,
    recommendation: Option<EvolutionRecommendation>,
    evolution_artifact: Option<EvolutionArtifact>,
    apply_artifact: Option<EvolutionApplyArtifact>,
    live_apply: Option<EvolutionLiveApplyReport>,
    check: Option<CheckReport>,
    run: Option<ParallelEvolutionRun>,
    auto_feed: Option<NeedRunBatchReport>,
    desktop_pet: Option<DesktopPetReport>,
    field_summary: Option<FieldTrajectoryReport>,
    next: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct EvolutionDriveArgs {
    pub(crate) open_pet: bool,
    pub(crate) workers: usize,
    pub(crate) tentacle_id: String,
    pub(crate) objective: String,
}

pub(crate) fn parse_evolution_drive_args(values: &[String]) -> Result<EvolutionDriveArgs, String> {
    let mut open_pet = false;
    let mut workers = 1usize;
    let mut rest = Vec::new();
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--open" => open_pet = true,
            "--workers" => {
                index += 1;
                let value = values
                    .get(index)
                    .ok_or_else(|| "evolve drive --workers requires 1..8".to_string())?;
                workers = parse_worker_count_1_to_8(value)?;
            }
            value => rest.push(value.to_string()),
        }
        index += 1;
    }
    let tentacle_id = rest
        .first()
        .cloned()
        .ok_or_else(|| "evolve drive requires a tentacle id".to_string())?;
    let objective = rest
        .get(1..)
        .filter(|values| !values.is_empty())
        .map(|values| values.join(" "))
        .unwrap_or_else(|| "drive one autonomous harness evolution cycle".to_string());
    Ok(EvolutionDriveArgs {
        open_pet,
        workers,
        tentacle_id,
        objective,
    })
}

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
        match recommend_current_patch(&cwd, &args.tentacle_id, &args.objective, &loaded) {
            Ok(result) => result,
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
        apply_summary(&recommendation, &live_apply),
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
        if let Ok((retry_evolution, retry_recommendation, retry_apply_artifact)) =
            recommend_current_patch(&cwd, &args.tentacle_id, &retry_objective, &loaded)
        {
            report.stage = EvolutionDriveStage::ApplyRetryPlanning.as_str().to_string();
            report.objective = retry_objective;
            report.evolution_artifact = Some(retry_evolution);
            report.apply_artifact = Some(retry_apply_artifact.clone());
            report.next = vec![format!(
                "octopus evolve apply {} {}",
                shell_arg(&args.tentacle_id),
                shell_arg(&retry_recommendation.candidate_id)
            )];
            live_apply = apply_authorized_suggested_patch(
                &cwd,
                &retry_recommendation.apply,
                &retry_apply_artifact,
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
                apply_summary(&retry_recommendation, &live_apply),
                if live_apply.applied {
                    Status::Satisfied
                } else {
                    Status::Failed
                },
                &live_apply,
            )?;
            report.live_apply = Some(live_apply.clone());
            report.recommendation = Some(retry_recommendation);
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

fn recommend_current_patch(
    cwd: &Path,
    tentacle_id: &str,
    objective: &str,
    state: &HarnessState,
) -> Result<
    (
        EvolutionArtifact,
        EvolutionRecommendation,
        EvolutionApplyArtifact,
    ),
    String,
> {
    let proposal = propose_evolution_for_cli(tentacle_id, objective, state)?;
    let evolution_artifact = write_tentacle_evolution_artifacts(cwd, &proposal)?;
    let recommendation = recommend_tentacle_evolution_apply(&proposal, state)?;
    let apply_artifact = write_tentacle_apply_artifacts(cwd, &recommendation.apply)?;
    Ok((evolution_artifact, recommendation, apply_artifact))
}

pub(crate) fn print_evolution_drive_report(report: &EvolutionDriveReport, language: Language) {
    match language {
        Language::En => {
            println!("tentacle: {}", report.tentacle_id);
            println!("stage: {}", report.stage);
            if let Some(recommendation) = &report.recommendation {
                println!("recommended: {}", recommendation.candidate_id);
            }
            if let Some(live_apply) = &report.live_apply {
                println!("apply: {}", live_apply.status);
            }
            if let Some(check) = &report.check {
                println!("check: {}", if check.passed { "pass" } else { "fail" });
            }
            if let Some(auto_feed) = &report.auto_feed {
                println!("feeds: {}", auto_feed.ran);
            }
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("触手: {}", report.tentacle_id);
            println!("阶段: {}", report.stage);
            if let Some(recommendation) = &report.recommendation {
                println!("推荐: {}", recommendation.candidate_id);
            }
            if let Some(live_apply) = &report.live_apply {
                println!("应用: {}", live_apply.status);
            }
            if let Some(check) = &report.check {
                println!("检查: {}", if check.passed { "通过" } else { "失败" });
            }
            if let Some(auto_feed) = &report.auto_feed {
                println!("Feed: {}", auto_feed.ran);
            }
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
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

fn apply_summary(
    recommendation: &EvolutionRecommendation,
    live_apply: &EvolutionLiveApplyReport,
) -> String {
    if live_apply.applied || live_apply.stderr.trim().is_empty() {
        return format!("{} {}", recommendation.candidate_id, live_apply.status);
    }
    let detail = live_apply
        .stderr
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let detail = if detail.chars().count() > 160 {
        format!("{}...", detail.chars().take(160).collect::<String>())
    } else {
        detail
    };
    format!(
        "{} {}: {}",
        recommendation.candidate_id, live_apply.status, detail
    )
}

fn retry_apply_objective(objective: &str, live_apply: &EvolutionLiveApplyReport) -> String {
    let stderr = compact_words(&live_apply.stderr, 220);
    if stderr.is_empty() {
        format!("{objective}; regenerate an authorized patch against the current files")
    } else {
        format!(
            "{objective}; regenerate an authorized patch against the current files after git apply failed: {stderr}"
        )
    }
}

fn compact_words(value: &str, limit: usize) -> String {
    let text = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() > limit {
        format!("{}...", text.chars().take(limit).collect::<String>())
    } else {
        text
    }
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join("; ")
    }
}
