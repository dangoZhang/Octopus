use crate::desktop_pet::DesktopPetReport;
use crate::evolution_apply::EvolutionLiveApplyReport;
use crate::{parse_worker_count_1_to_8, CheckReport, Language, NeedRunBatchReport};
use octopus_core::{
    EvolutionApplyArtifact, EvolutionArtifact, EvolutionRecommendation, FieldTrajectoryReport,
    ParallelEvolutionRun,
};

#[derive(Debug, serde::Serialize)]
pub(crate) struct EvolutionDriveReport {
    pub(crate) tentacle_id: String,
    pub(crate) objective: String,
    pub(crate) stage: String,
    pub(crate) recommendation: Option<EvolutionRecommendation>,
    pub(crate) evolution_artifact: Option<EvolutionArtifact>,
    pub(crate) apply_artifact: Option<EvolutionApplyArtifact>,
    pub(crate) live_apply: Option<EvolutionLiveApplyReport>,
    pub(crate) check: Option<CheckReport>,
    pub(crate) run: Option<ParallelEvolutionRun>,
    pub(crate) auto_feed: Option<NeedRunBatchReport>,
    pub(crate) desktop_pet: Option<DesktopPetReport>,
    pub(crate) field_summary: Option<FieldTrajectoryReport>,
    pub(crate) next: Vec<String>,
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

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join("; ")
    }
}
