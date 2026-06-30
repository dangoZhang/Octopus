use super::*;
use crate::evolution_drive_surface::{parse_evolution_drive_args, print_evolution_drive_report};
use crate::evolution_driver::drive_evolution_cycle;
use crate::observation_contract::{
    classify_apply_status, classify_planner_error, record_evolution_event, EvolutionStage,
};

pub(crate) fn handle_evolve_command(
    rest: &[String],
    state: &Path,
    json: bool,
    language: Language,
) -> Result<(), String> {
    if matches!(rest.get(1).map(String::as_str), Some("parallel" | "swarm")) {
        let (open_pet, worker_limit, objective) = parse_parallel_evolution_args(&rest[2..])?;
        let mut loaded = HarnessState::load(state).map_err(|error| error.to_string())?;
        let prepared_field_tentacle = ensure_field_mini_task_tentacle(&mut loaded)?;
        let objective = objective
            .or_else(|| loaded.goal.as_ref().map(|goal| goal.objective.clone()))
            .unwrap_or_else(|| "parallel harness evolution toward field adaptation".to_string());
        let mut run = loaded.start_parallel_evolution(objective, worker_limit)?;
        loaded.save(state).map_err(|error| error.to_string())?;
        let desktop_pet = if open_pet {
            Some(launch_desktop_pet(
                state,
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
        if record_parallel_evolution_action_event(&mut loaded, &run).is_some() {
            loaded.save(state).map_err(|error| error.to_string())?;
        }
        let (auto_feed, next_state) = if queued_indices.is_empty() {
            (empty_parallel_evolution_batch_report(&loaded), loaded)
        } else {
            run_queued_need_indices_with_observer_state(loaded, &queued_indices, Some(state))?
        };
        loaded = next_state;
        for report in &auto_feed.reports {
            let status = report
                .verifier_result
                .as_ref()
                .map(|result| result.status.clone())
                .unwrap_or_else(|| report.feed.status.clone());
            if let Some(updated) = loaded.update_parallel_evolution_worker_result(
                run.index,
                report.taken.item.index,
                report.feed_trace_index,
                report.verifier_result.as_ref().map(|result| result.index),
                status,
            ) {
                run = updated;
            }
        }
        loaded.save(state).map_err(|error| error.to_string())?;
        pet_events::append_latest(state, &loaded)?;
        let next = parallel_evolution_next_actions(&auto_feed);
        if json {
            let worker_policy = run.worker_policy.clone();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "run": run,
                    "auto_feed": auto_feed,
                    "desktop_pet": desktop_pet,
                    "prepared_field_tentacle": prepared_field_tentacle,
                    "mode": "0.1.x automatic peer-field worker step",
                    "worker_policy": worker_policy,
                    "next": next
                }))
                .map_err(|error| error.to_string())?
            );
        } else {
            print_parallel_evolution_run(&run, desktop_pet.as_ref(), language);
            print_need_run_batch_report(&auto_feed, language);
        }
        return Ok(());
    }
    if rest.get(1).map(String::as_str) == Some("drive") {
        let args = parse_evolution_drive_args(&rest[2..])?;
        let report = drive_evolution_cycle(state, args)?;
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_evolution_drive_report(&report, language);
        }
        return Ok(());
    }
    if rest.get(1).map(String::as_str) == Some("recommend") {
        let tentacle_id = rest
            .get(2)
            .ok_or_else(|| "evolve recommend requires a tentacle id".to_string())?;
        let objective = rest
            .get(3..)
            .filter(|values| !values.is_empty())
            .map(|values| values.join(" "))
            .unwrap_or_else(|| "recommend next evolution candidate".to_string());
        let mut loaded = HarnessState::load(state).map_err(|error| error.to_string())?;
        record_evolve_surface_event(
            state,
            &mut loaded,
            EvolutionStage::Planning,
            "evolution",
            format!("evolve recommend {tentacle_id}"),
            format!("planning {}", pet_events::summary_text(&objective)),
            Status::Partial,
            None,
        )?;
        let proposal = match propose_evolution_for_cli(tentacle_id, &objective, &loaded) {
            Ok(proposal) => proposal,
            Err(error) => {
                record_evolve_surface_event(
                    state,
                    &mut loaded,
                    EvolutionStage::Planning,
                    "blocked",
                    format!("evolve recommend {tentacle_id}"),
                    error.clone(),
                    Status::Failed,
                    Some(classify_planner_error(&error)),
                )?;
                return Err(error);
            }
        };
        let cwd = env::current_dir().map_err(|error| error.to_string())?;
        let artifact = match write_tentacle_evolution_artifacts(&cwd, &proposal) {
            Ok(artifact) => artifact,
            Err(error) => {
                record_evolve_surface_event(
                    state,
                    &mut loaded,
                    EvolutionStage::Planning,
                    "blocked",
                    format!("evolve recommend {tentacle_id}"),
                    error.clone(),
                    Status::Failed,
                    Some(classify_planner_error(&error)),
                )?;
                return Err(error);
            }
        };
        let recommendation = match recommend_tentacle_evolution_apply(&proposal, &loaded) {
            Ok(recommendation) => recommendation,
            Err(error) => {
                record_evolve_surface_event(
                    state,
                    &mut loaded,
                    EvolutionStage::Recommended,
                    "blocked",
                    format!("evolve recommend {tentacle_id}"),
                    error.clone(),
                    Status::Failed,
                    Some(classify_planner_error(&error)),
                )?;
                return Err(error);
            }
        };
        let apply_artifact = match write_tentacle_apply_artifacts(&cwd, &recommendation.apply) {
            Ok(apply_artifact) => apply_artifact,
            Err(error) => {
                record_evolve_surface_event(
                    state,
                    &mut loaded,
                    EvolutionStage::Recommended,
                    "blocked",
                    format!("evolve recommend {tentacle_id}"),
                    error.clone(),
                    Status::Failed,
                    Some(classify_planner_error(&error)),
                )?;
                return Err(error);
            }
        };
        record_evolve_surface_event(
            state,
            &mut loaded,
            EvolutionStage::Recommended,
            "evolution",
            format!("evolve recommend {tentacle_id}"),
            format!(
                "recommended {}: {}",
                recommendation.candidate_id,
                pet_events::summary_text(&recommendation.candidate_title)
            ),
            Status::Satisfied,
            None,
        )?;
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "recommendation": recommendation,
                    "apply_artifact": apply_artifact,
                    "evolution_artifact": artifact,
                }))
                .map_err(|error| error.to_string())?
            );
        } else {
            print_evolution_recommendation(&recommendation, &apply_artifact, language);
        }
        return Ok(());
    }
    if rest.get(1).map(String::as_str) == Some("apply") {
        let tentacle_id = rest
            .get(2)
            .ok_or_else(|| "evolve apply requires a tentacle id".to_string())?;
        let candidate_id = rest
            .get(3)
            .ok_or_else(|| "evolve apply requires a candidate id".to_string())?;
        let objective = rest
            .get(4..)
            .filter(|values| !values.is_empty())
            .map(|values| values.join(" "))
            .unwrap_or_else(|| "apply evolution candidate".to_string());
        let mut loaded = HarnessState::load(state).map_err(|error| error.to_string())?;
        record_evolve_surface_event(
            state,
            &mut loaded,
            EvolutionStage::Applying,
            "action",
            format!("evolve apply {tentacle_id}"),
            format!("applying {candidate_id}"),
            Status::Partial,
            None,
        )?;
        let cwd = env::current_dir().map_err(|error| error.to_string())?;
        let apply_result = (|| {
            let (plan, evolution_artifact) = if let Some(proposal) =
                maybe_load_evolution_proposal_artifact(&cwd, tentacle_id, candidate_id)?
            {
                let plan = plan_tentacle_evolution_apply(&proposal, &loaded, candidate_id)?;
                (plan, None)
            } else if let Some(plan) =
                maybe_load_evolution_apply_plan_artifact(&cwd, tentacle_id, candidate_id)?
            {
                (plan, None)
            } else {
                let proposal = propose_evolution_for_cli(tentacle_id, &objective, &loaded)?;
                let artifact = write_tentacle_evolution_artifacts(&cwd, &proposal)?;
                let plan = plan_tentacle_evolution_apply(&proposal, &loaded, candidate_id)?;
                (plan, Some(artifact))
            };
            let apply_artifact = write_tentacle_apply_artifacts(&cwd, &plan)?;
            let live_apply = apply_authorized_suggested_patch(&cwd, &plan, &apply_artifact);
            Ok::<_, String>((plan, evolution_artifact, apply_artifact, live_apply))
        })();
        let (plan, evolution_artifact, apply_artifact, live_apply) = match apply_result {
            Ok(result) => result,
            Err(error) => {
                record_evolve_surface_event(
                    state,
                    &mut loaded,
                    EvolutionStage::Applying,
                    "blocked",
                    format!("evolve apply {tentacle_id}"),
                    error.clone(),
                    Status::Failed,
                    Some(classify_planner_error(&error)),
                )?;
                return Err(error);
            }
        };
        let (pet_state, pet_status) = if live_apply.applied {
            ("success", Status::Satisfied)
        } else {
            ("blocked", Status::Failed)
        };
        let apply_summary = live_apply_candidate_summary(candidate_id, &live_apply);
        let error_class = (!live_apply.applied)
            .then(|| classify_apply_status(&live_apply.status, &live_apply.stderr));
        record_evolve_surface_event(
            state,
            &mut loaded,
            EvolutionStage::Applying,
            pet_state,
            format!("evolve apply {tentacle_id}"),
            apply_summary,
            pet_status,
            error_class,
        )?;
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "apply": plan,
                    "apply_artifact": apply_artifact,
                    "live_apply": live_apply,
                    "evolution_artifact": evolution_artifact,
                }))
                .map_err(|error| error.to_string())?
            );
        } else {
            print_evolution_apply_plan(&plan, &apply_artifact, language);
            print_evolution_live_apply(&live_apply, language);
        }
        return Ok(());
    }
    if rest.get(1).map(String::as_str) == Some("score") {
        let tentacle_id = rest
            .get(2)
            .ok_or_else(|| "evolve score requires a tentacle id".to_string())?;
        let candidate_id = rest
            .get(3)
            .ok_or_else(|| "evolve score requires a candidate id".to_string())?;
        let status = rest
            .get(4)
            .ok_or_else(|| "evolve score requires a status".to_string())
            .and_then(|value| parse_status(value))?;
        let summary = rest
            .get(5..)
            .filter(|values| !values.is_empty())
            .map(|values| values.join(" "))
            .unwrap_or_else(|| "recorded evolution outcome".to_string());
        let mut loaded = HarnessState::load(state).map_err(|error| error.to_string())?;
        let outcome = loaded.record_evolution_outcome(
            tentacle_id.as_str(),
            candidate_id.as_str(),
            status,
            summary,
        );
        loaded.save(state).map_err(|error| error.to_string())?;
        pet_events::append_latest(state, &loaded)?;
        let cwd = env::current_dir().map_err(|error| error.to_string())?;
        let objective_summary = outcome.summary.clone();
        let report =
            evolution_score_report(tentacle_id, &objective_summary, &loaded, &cwd, outcome);
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_evolution_score_report(&report, language);
        }
        return Ok(());
    }
    let tentacle_id = rest
        .get(1)
        .ok_or_else(|| "evolve requires a tentacle id".to_string())?;
    let objective = rest
        .get(2..)
        .filter(|values| !values.is_empty())
        .map(|values| values.join(" "))
        .unwrap_or_else(|| "improve feed quality".to_string());
    let mut loaded = HarnessState::load(state).map_err(|error| error.to_string())?;
    record_evolve_surface_event(
        state,
        &mut loaded,
        EvolutionStage::Planning,
        "evolution",
        format!("evolve {tentacle_id}"),
        format!("planning {}", pet_events::summary_text(&objective)),
        Status::Partial,
        None,
    )?;
    let proposal = match propose_evolution_for_cli(tentacle_id, &objective, &loaded) {
        Ok(proposal) => proposal,
        Err(error) => {
            record_evolve_surface_event(
                state,
                &mut loaded,
                EvolutionStage::Planning,
                "blocked",
                format!("evolve {tentacle_id}"),
                error.clone(),
                Status::Failed,
                Some(classify_planner_error(&error)),
            )?;
            return Err(error);
        }
    };
    let cwd = env::current_dir().map_err(|error| error.to_string())?;
    let artifact = match write_tentacle_evolution_artifacts(cwd, &proposal) {
        Ok(artifact) => artifact,
        Err(error) => {
            record_evolve_surface_event(
                state,
                &mut loaded,
                EvolutionStage::Planning,
                "blocked",
                format!("evolve {tentacle_id}"),
                error.clone(),
                Status::Failed,
                Some(classify_planner_error(&error)),
            )?;
            return Err(error);
        }
    };
    record_evolve_surface_event(
        state,
        &mut loaded,
        EvolutionStage::Recommended,
        "evolution",
        format!("evolve {tentacle_id}"),
        format!(
            "proposal with {} candidates",
            proposal.patch_candidates.len()
        ),
        Status::Satisfied,
        None,
    )?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "proposal": proposal,
                "artifact": artifact,
            }))
            .map_err(|error| error.to_string())?
        );
    } else {
        print_evolution_proposal(&proposal, &artifact, language);
    }
    Ok(())
}

fn record_evolve_surface_event(
    state_path: &Path,
    state: &mut HarnessState,
    stage: EvolutionStage,
    pet_state: &str,
    source: impl Into<String>,
    summary: impl Into<String>,
    status: Status,
    error_class: Option<String>,
) -> Result<(), String> {
    record_evolution_event(
        state_path,
        state,
        stage,
        pet_state,
        source,
        summary,
        status,
        error_class,
    )
    .map(|_| ())
}

fn print_evolution_proposal(
    proposal: &TentacleEvolutionProposal,
    artifact: &EvolutionArtifact,
    language: Language,
) {
    let surfaces = proposal
        .surfaces
        .iter()
        .map(|surface| surface.id.clone())
        .collect::<Vec<_>>();
    match language {
        Language::En => {
            println!("tentacle: {}", proposal.tentacle_id);
            println!("objective: {}", proposal.objective);
            println!("generator: {}", proposal.generator);
            if !proposal.planner_summary.trim().is_empty() {
                println!("planner: {}", proposal.planner_summary);
            }
            println!("proposal: {}", artifact.proposal_path);
            println!("candidates: {}", artifact.candidates_path);
            println!("patch drafts: {}", artifact.drafts_path);
            println!("json: {}", artifact.json_path);
            println!("candidate count: {}", proposal.patch_candidates.len());
            println!("feed traces: {}", proposal.recent_feed_traces.len());
            println!("check history: {}", proposal.recent_check_history.len());
            println!("editable: {}", join_or_none(&proposal.editable));
            println!("surfaces: {}", join_or_none(&surfaces));
            println!("checks: {}", join_or_none(&proposal.checks));
        }
        Language::Zh => {
            println!("触手: {}", proposal.tentacle_id);
            println!("目标: {}", proposal.objective);
            println!("生成器: {}", proposal.generator);
            if !proposal.planner_summary.trim().is_empty() {
                println!("规划: {}", proposal.planner_summary);
            }
            println!("草案: {}", artifact.proposal_path);
            println!("候选: {}", artifact.candidates_path);
            println!("补丁草案: {}", artifact.drafts_path);
            println!("JSON: {}", artifact.json_path);
            println!("候选数量: {}", proposal.patch_candidates.len());
            println!("Feed轨迹: {}", proposal.recent_feed_traces.len());
            println!("检查历史: {}", proposal.recent_check_history.len());
            println!("可编辑: {}", join_or_none(&proposal.editable));
            println!("可进化面: {}", join_or_none(&surfaces));
            println!("检查: {}", join_or_none(&proposal.checks));
        }
    }
}

fn print_evolution_apply_plan(
    plan: &EvolutionApplyPlan,
    artifact: &EvolutionApplyArtifact,
    language: Language,
) {
    match language {
        Language::En => {
            println!("tentacle: {}", plan.tentacle_id);
            println!("candidate: {}", plan.candidate_id);
            println!("status: {}", plan.status);
            println!("authorized: {}", plan.authorized);
            println!("required grant: {}", plan.required_grant);
            println!("target: {}", plan.target);
            println!("target files: {}", join_or_none(&plan.target_files));
            println!("plan: {}", artifact.plan_path);
            if let Some(path) = &artifact.patch_path {
                println!("patch: {path}");
            }
            println!("json: {}", artifact.json_path);
            println!("next: {}", join_or_none(&plan.next_steps));
        }
        Language::Zh => {
            println!("触手: {}", plan.tentacle_id);
            println!("候选: {}", plan.candidate_id);
            println!("状态: {}", plan.status);
            println!("已授权: {}", plan.authorized);
            println!("所需授权: {}", plan.required_grant);
            println!("目标: {}", plan.target);
            println!("目标文件: {}", join_or_none(&plan.target_files));
            println!("计划: {}", artifact.plan_path);
            if let Some(path) = &artifact.patch_path {
                println!("补丁: {path}");
            }
            println!("JSON: {}", artifact.json_path);
            println!("下一步: {}", join_or_none(&plan.next_steps));
        }
    }
}

fn maybe_load_evolution_apply_plan_artifact(
    cwd: &Path,
    tentacle_id: &str,
    candidate_id: &str,
) -> Result<Option<EvolutionApplyPlan>, String> {
    let stem = candidate_id.replace('/', "-");
    let path = cwd
        .join(".octopus")
        .join("evolution")
        .join(tentacle_id)
        .join("apply")
        .join(format!("{stem}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).map_err(|error| {
        format!(
            "apply artifact read failed {}: {error}",
            path.to_string_lossy()
        )
    })?;
    let plan = serde_json::from_str::<EvolutionApplyPlan>(&content).map_err(|error| {
        format!(
            "apply artifact parse failed {}: {error}",
            path.to_string_lossy()
        )
    })?;
    if plan.tentacle_id != tentacle_id {
        return Err(format!(
            "apply artifact tentacle mismatch: expected {tentacle_id}, got {}",
            plan.tentacle_id
        ));
    }
    if plan.candidate_id != candidate_id {
        return Err(format!(
            "apply artifact candidate mismatch: expected {candidate_id}, got {}",
            plan.candidate_id
        ));
    }
    Ok(Some(plan))
}

fn maybe_load_evolution_proposal_artifact(
    cwd: &Path,
    tentacle_id: &str,
    candidate_id: &str,
) -> Result<Option<TentacleEvolutionProposal>, String> {
    let path = cwd
        .join(".octopus")
        .join("evolution")
        .join(tentacle_id)
        .join("proposal.json");
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).map_err(|error| {
        format!(
            "evolution proposal read failed {}: {error}",
            path.to_string_lossy()
        )
    })?;
    let proposal =
        serde_json::from_str::<TentacleEvolutionProposal>(&content).map_err(|error| {
            format!(
                "evolution proposal parse failed {}: {error}",
                path.to_string_lossy()
            )
        })?;
    if proposal.tentacle_id != tentacle_id {
        return Err(format!(
            "evolution proposal tentacle mismatch: expected {tentacle_id}, got {}",
            proposal.tentacle_id
        ));
    }
    let candidate_present = proposal.patch_candidates.iter().any(|candidate| {
        candidate.id == candidate_id
            || candidate.surface_id == candidate_id
            || candidate.id.replace('-', "_") == candidate_id
            || candidate.surface_id.replace('_', "-") == candidate_id
    });
    if !candidate_present {
        return Ok(None);
    }
    Ok(Some(proposal))
}

fn print_evolution_live_apply(report: &EvolutionLiveApplyReport, language: Language) {
    match language {
        Language::En => {
            println!("live apply: {}", report.status);
            if let Some(command) = &report.command {
                println!("command: {command}");
            }
            if !report.stderr.trim().is_empty() {
                println!("stderr: {}", report.stderr.trim());
            }
        }
        Language::Zh => {
            println!("实时应用: {}", report.status);
            if let Some(command) = &report.command {
                println!("命令: {command}");
            }
            if !report.stderr.trim().is_empty() {
                println!("stderr: {}", report.stderr.trim());
            }
        }
    }
}

fn print_evolution_recommendation(
    recommendation: &EvolutionRecommendation,
    artifact: &EvolutionApplyArtifact,
    language: Language,
) {
    match language {
        Language::En => {
            println!("tentacle: {}", recommendation.tentacle_id);
            println!("recommended: {}", recommendation.candidate_id);
            println!("surface: {}", recommendation.surface_id);
            println!("title: {}", recommendation.candidate_title);
            println!("score: {:.2}", recommendation.recommendation_score);
            println!("outcomes: {}", recommendation.outcome_count);
            println!("feed traces: {}", recommendation.feed_trace_count);
            println!("check history: {}", recommendation.check_history_count);
            println!("reason: {}", recommendation.reason);
            println!("status: {}", recommendation.apply.status);
            println!("target: {}", recommendation.apply.target);
            println!(
                "target files: {}",
                join_or_none(&recommendation.apply.target_files)
            );
            println!("plan: {}", artifact.plan_path);
            if let Some(path) = &artifact.patch_path {
                println!("patch: {path}");
            }
        }
        Language::Zh => {
            println!("触手: {}", recommendation.tentacle_id);
            println!("推荐: {}", recommendation.candidate_id);
            println!("可进化面: {}", recommendation.surface_id);
            println!("标题: {}", recommendation.candidate_title);
            println!("分数: {:.2}", recommendation.recommendation_score);
            println!("历史结果: {}", recommendation.outcome_count);
            println!("Feed轨迹: {}", recommendation.feed_trace_count);
            println!("检查历史: {}", recommendation.check_history_count);
            println!("原因: {}", recommendation.reason);
            println!("状态: {}", recommendation.apply.status);
            println!("目标: {}", recommendation.apply.target);
            println!(
                "目标文件: {}",
                join_or_none(&recommendation.apply.target_files)
            );
            println!("计划: {}", artifact.plan_path);
            if let Some(path) = &artifact.patch_path {
                println!("补丁: {path}");
            }
        }
    }
}

fn print_evolution_score_report(report: &EvolutionScoreReport, language: Language) {
    print_evolution_outcome(&report.outcome, language);
    match language {
        Language::En => {
            println!("recent outcomes: {}", report.recent.len());
            if let Some(recommendation) = &report.recommendation {
                println!("next recommended: {}", recommendation.candidate_id);
                println!("next score: {:.2}", recommendation.recommendation_score);
                println!("next reason: {}", recommendation.reason);
                println!("next target: {}", recommendation.apply.target);
                println!(
                    "next target files: {}",
                    join_or_none(&recommendation.apply.target_files)
                );
                if let Some(artifact) = &report.apply_artifact {
                    println!("next plan: {}", artifact.plan_path);
                    if let Some(path) = &artifact.patch_path {
                        println!("next patch: {path}");
                    }
                }
            }
            if let Some(error) = &report.recommendation_error {
                println!("recommendation error: {error}");
            }
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("近期结果: {}", report.recent.len());
            if let Some(recommendation) = &report.recommendation {
                println!("下一推荐: {}", recommendation.candidate_id);
                println!("下一分数: {:.2}", recommendation.recommendation_score);
                println!("下一原因: {}", recommendation.reason);
                println!("下一目标: {}", recommendation.apply.target);
                println!(
                    "下一目标文件: {}",
                    join_or_none(&recommendation.apply.target_files)
                );
                if let Some(artifact) = &report.apply_artifact {
                    println!("下一计划: {}", artifact.plan_path);
                    if let Some(path) = &artifact.patch_path {
                        println!("下一补丁: {path}");
                    }
                }
            }
            if let Some(error) = &report.recommendation_error {
                println!("推荐错误: {error}");
            }
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

fn print_evolution_outcome(outcome: &EvolutionOutcome, language: Language) {
    match language {
        Language::En => {
            println!("tentacle: {}", outcome.tentacle_id);
            println!("candidate: {}", outcome.candidate_id);
            println!("status: {:?}", outcome.status);
            println!("score: {:.2}", outcome.score);
            println!("summary: {}", outcome.summary);
        }
        Language::Zh => {
            println!("触手: {}", outcome.tentacle_id);
            println!("候选: {}", outcome.candidate_id);
            println!("状态: {:?}", outcome.status);
            println!("分数: {:.2}", outcome.score);
            println!("摘要: {}", outcome.summary);
        }
    }
}

pub(crate) fn propose_evolution_for_cli(
    tentacle_id: &str,
    objective: &str,
    state: &HarnessState,
) -> Result<TentacleEvolutionProposal, String> {
    let root = resolve_tentacle_manifest_root(tentacle_id)?;
    if evolve_llm_enabled() {
        let (_, _, _, _, mut client) = provider_client(&evolve_llm_prefix())?;
        return propose_tentacle_evolution_with_client(
            root,
            tentacle_id,
            objective,
            state,
            &mut client,
        );
    }
    let _ = (root, state);
    Err(
        "harness evolution requires an LLM planner; set an evolve provider before running evolve"
            .to_string(),
    )
}

pub(crate) fn evolution_score_report(
    tentacle_id: &str,
    summary: &str,
    state: &HarnessState,
    cwd: &Path,
    outcome: EvolutionOutcome,
) -> EvolutionScoreReport {
    let objective = format!("continue from scored evolution feedback: {summary}");
    let recent = state.recent_evolution_outcomes(tentacle_id, 5);
    let mut next = vec![
        format!(
            "octopus evolve recommend {tentacle_id} {}",
            shell_arg(&objective)
        ),
        "review the generated apply plan before granting harness write".to_string(),
    ];
    let mut recommendation = None;
    let mut apply_artifact = None;
    let mut evolution_artifact = None;
    let mut recommendation_error = None;
    match propose_evolution_for_cli(tentacle_id, &objective, state).and_then(|proposal| {
        let artifact = write_tentacle_evolution_artifacts(cwd, &proposal)?;
        let recommendation = recommend_tentacle_evolution_apply(&proposal, state)?;
        let apply_artifact = write_tentacle_apply_artifacts(cwd, &recommendation.apply)?;
        Ok((recommendation, apply_artifact, artifact))
    }) {
        Ok((recommended, apply, artifact)) => {
            next = recommended.apply.next_steps.clone();
            next.push(format!(
                "octopus evolve apply {} {}",
                recommended.tentacle_id, recommended.candidate_id
            ));
            next.push(format!(
                "octopus evolve score {} {} satisfied {}",
                recommended.tentacle_id,
                recommended.candidate_id,
                shell_arg("reviewed follow-up harness change")
            ));
            recommendation = Some(recommended);
            apply_artifact = Some(apply);
            evolution_artifact = Some(artifact);
        }
        Err(error) => {
            recommendation_error = Some(error);
        }
    }
    EvolutionScoreReport {
        outcome,
        recent,
        recommendation,
        apply_artifact,
        evolution_artifact,
        recommendation_error,
        next,
    }
}
