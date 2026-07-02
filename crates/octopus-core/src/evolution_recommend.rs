use crate::{
    evolution_target::evolution_file_target, load_tentacle_manifests, one_line,
    propose_tentacle_evolution_with_client, short_text, write_tentacle_apply_artifacts,
    write_tentacle_evolution_artifacts, ChatClient, CheckHistoryRecord, EvolutionApplyPlan,
    EvolutionPatchCandidate, EvolutionPatchFeedback, EvolutionRecommendation, FeedTraceRecord,
    HarnessBeatEvolution, HarnessState, ManifestTool, RepairOutcome, Status,
    TentacleEvolutionProposal,
};
use std::fs;
use std::path::Path;

pub(crate) fn tentacle_evolution_context(
    root: impl AsRef<Path>,
    tentacle_id: &str,
    objective: &str,
    state: &HarnessState,
) -> Result<TentacleEvolutionProposal, String> {
    let root = root.as_ref();
    let loaded = load_tentacle_manifests(root)
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|loaded| loaded.manifest.id == tentacle_id)
        .ok_or_else(|| format!("unknown manifest: {tentacle_id}"))?;
    let objective = objective.trim();
    if objective.is_empty() {
        return Err("evolution objective cannot be empty".to_string());
    }
    let manifest_dir = Path::new(&loaded.path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.join(tentacle_id));
    let files = loaded
        .manifest
        .evolution
        .editable
        .iter()
        .map(|target| evolution_file_target(&manifest_dir, target, objective))
        .collect::<Vec<_>>();
    let previous_outcomes = state.recent_evolution_outcomes(tentacle_id, 5);
    let recent_feed_traces = state.recent_feed_traces_for_tentacle(tentacle_id, 8);
    let recent_check_history = state.recent_check_history_for_tentacle(tentacle_id, 8);
    let environment_gaps = state
        .field_trajectory_report()
        .map(|report| {
            report
                .fields
                .into_iter()
                .filter(|summary| summary.needs_environment)
                .map(|summary| crate::EvolutionEnvironmentGap {
                    field: summary.field,
                    mini_task: summary.latest_mini_task.or(summary.next_mini_task),
                    error_category: summary.latest_error_category,
                    latest_trace_index: summary.latest_trace_index,
                    latest_verifier_result_index: summary.latest_verifier_result_index,
                    latest_summary: summary.latest_summary.map(|value| short_text(&one_line(&value), 320)),
                    guidance: "treat this as an environment adaptation gap; add real fallback evidence or runtime guidance, and keep the Feed partial when the missing runtime remains unavailable".to_string(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut next_steps = vec![
        "run the configured LLM harness planner".to_string(),
        "review LLM-generated harness candidates".to_string(),
        "apply only provider-supplied patches for declared harness targets".to_string(),
    ];
    next_steps.extend(
        loaded
            .manifest
            .evolution
            .checks
            .iter()
            .map(|check| format!("run: {check}")),
    );
    if !loaded
        .manifest
        .evolution
        .checks
        .iter()
        .any(|check| check.contains("cargo test"))
    {
        next_steps.push("run: cargo test".to_string());
    }
    Ok(TentacleEvolutionProposal {
        tentacle_id: loaded.manifest.id,
        tentacle_name: loaded.manifest.name,
        objective: objective.to_string(),
        generator: "llm_required".to_string(),
        planner_summary: "awaiting LLM harness planner".to_string(),
        manifest_path: loaded.path,
        brain_kind: loaded.manifest.brain.kind,
        current_brain_prompt: loaded.manifest.brain.prompt,
        editable: loaded.manifest.evolution.editable,
        surfaces: loaded.manifest.evolution.surfaces,
        requirements: loaded.manifest.evolution.requirements,
        checks: loaded.manifest.evolution.checks,
        constraints: loaded.manifest.evolution.constraints,
        previous_outcomes: previous_outcomes.to_vec(),
        recent_feed_traces: recent_feed_traces.to_vec(),
        recent_check_history: recent_check_history.to_vec(),
        environment_gaps,
        files,
        patch_candidates: Vec::new(),
        next_steps,
    })
}

pub fn plan_tentacle_evolution_apply(
    proposal: &TentacleEvolutionProposal,
    state: &HarnessState,
    candidate_id: &str,
) -> Result<EvolutionApplyPlan, String> {
    let candidate = proposal
        .patch_candidates
        .iter()
        .find(|candidate| candidate_matches(candidate, candidate_id))
        .ok_or_else(|| format!("unknown evolution candidate: {candidate_id}"))?;
    let required_grant = format!("octopus:evolve:{}", proposal.tentacle_id);
    let grant = state.active_grant("octopus", &format!("evolve:{}", proposal.tentacle_id));
    let authorized = grant.is_some_and(|grant| {
        grant
            .permissions
            .iter()
            .any(|permission| permission == "harness:write")
    });
    let active_grant = grant.map(|grant| grant.id.clone());
    let needs_suggested_patch = candidate.suggested_patch.is_none();
    let status = if authorized && needs_suggested_patch {
        "needs_suggested_patch"
    } else if authorized {
        "ready_for_authorized_patch"
    } else {
        "needs_authorization"
    };
    let mut next_steps = vec![format!("review {}", candidate.draft.path)];
    if authorized && needs_suggested_patch {
        next_steps.push(
            "rerun evolution with an LLM planner that returns a suggested_patch or choose a concrete target"
                .to_string(),
        );
        next_steps.push("do not apply a review-note patch as harness evolution".to_string());
    } else if authorized {
        next_steps.push("turn the draft into a narrow harness patch".to_string());
        next_steps.extend(candidate.checks.iter().map(|check| format!("run: {check}")));
    } else {
        next_steps.push(format!(
            "run: octopus oauth octopus evolve:{} harness:write",
            proposal.tentacle_id
        ));
    }
    Ok(EvolutionApplyPlan {
        tentacle_id: proposal.tentacle_id.clone(),
        candidate_id: candidate.id.clone(),
        objective: proposal.objective.clone(),
        authorized,
        status: status.to_string(),
        required_grant,
        active_grant,
        target: candidate.target.clone(),
        target_files: candidate.target_files.clone(),
        draft_path: candidate.draft.path.clone(),
        checks: candidate.checks.clone(),
        feedback: candidate.feedback.clone(),
        suggested_patch: candidate.suggested_patch.clone(),
        guardrails: vec![
            "do not modify kernel code from an evolution draft".to_string(),
            "patch only declared harness targets".to_string(),
            "run listed checks before commit".to_string(),
        ],
        next_steps,
    })
}

pub fn recommend_tentacle_evolution_apply(
    proposal: &TentacleEvolutionProposal,
    state: &HarnessState,
) -> Result<EvolutionRecommendation, String> {
    recommend_tentacle_evolution_apply_with_preference(proposal, state, None)
}

fn recommend_tentacle_evolution_apply_with_preference(
    proposal: &TentacleEvolutionProposal,
    state: &HarnessState,
    preferred_candidate: Option<&str>,
) -> Result<EvolutionRecommendation, String> {
    if let Some(preferred_candidate) = clean_preferred_candidate(preferred_candidate) {
        if let Some(candidate) = proposal
            .patch_candidates
            .iter()
            .find(|candidate| candidate_matches(candidate, preferred_candidate))
        {
            let apply = plan_tentacle_evolution_apply(proposal, state, &candidate.id)?;
            return Ok(EvolutionRecommendation {
                tentacle_id: proposal.tentacle_id.clone(),
                objective: proposal.objective.clone(),
                candidate_id: candidate.id.clone(),
                candidate_title: candidate.title.clone(),
                surface_id: candidate.surface_id.clone(),
                outcome_count: 1,
                feed_trace_count: 0,
                check_history_count: 0,
                recommendation_score: 1.0,
                reason: format!(
                    "selected from repair outcome candidate hint {preferred_candidate}"
                ),
                apply,
            });
        }
    }
    let Some(scored) = proposal
        .patch_candidates
        .iter()
        .map(|candidate| {
            let (score, count) = evolution_candidate_outcome_score(candidate, proposal);
            let (feed_score, feed_count) =
                evolution_candidate_feed_trace_score(candidate, proposal);
            let (check_score, check_count) =
                evolution_candidate_check_history_score(candidate, proposal);
            (
                candidate,
                score + feed_score + check_score,
                count,
                feed_count,
                check_count,
            )
        })
        .max_by(|left, right| {
            left.1
                .total_cmp(&right.1)
                .then_with(|| right.0.id.cmp(&left.0.id))
        })
    else {
        return Err("no evolution candidates to recommend".to_string());
    };
    let (candidate, score, outcome_count, feed_trace_count, check_history_count) = scored;
    let apply = plan_tentacle_evolution_apply(proposal, state, &candidate.id)?;
    let reason = if outcome_count > 0 {
        format!("selected from {outcome_count} previous outcomes with score {score:.2}")
    } else if feed_trace_count > 0 {
        format!(
            "selected from {feed_trace_count} matching Feed trace records with score {score:.2}"
        )
    } else if check_history_count > 0 {
        format!(
            "selected from {check_history_count} matching check history records with score {score:.2}"
        )
    } else if proposal.generator == "llm" {
        "selected an LLM-generated candidate after feeding previous outcomes to the planner"
            .to_string()
    } else {
        "selected candidate; no previous scored outcome matched these candidates".to_string()
    };
    Ok(EvolutionRecommendation {
        tentacle_id: proposal.tentacle_id.clone(),
        objective: proposal.objective.clone(),
        candidate_id: candidate.id.clone(),
        candidate_title: candidate.title.clone(),
        surface_id: candidate.surface_id.clone(),
        outcome_count,
        feed_trace_count,
        check_history_count,
        recommendation_score: score,
        reason,
        apply,
    })
}

pub fn write_harness_beat_evolution_artifacts_with_client<C>(
    tentacles_root: impl AsRef<Path>,
    workspace_root: impl AsRef<Path>,
    state: &HarnessState,
    client: &mut C,
) -> Result<Option<HarnessBeatEvolution>, String>
where
    C: ChatClient,
{
    let tentacles_root = tentacles_root.as_ref();
    let workspace_root = workspace_root.as_ref();
    for record in state
        .check_history
        .iter()
        .rev()
        .filter(|record| evolution_actionable_status(&record.status))
    {
        let objective = check_history_evolution_objective(record);
        let proposal = match propose_tentacle_evolution_with_client(
            tentacles_root,
            &record.tentacle_id,
            &objective,
            state,
            client,
        ) {
            Ok(proposal) => proposal,
            Err(error) if error.starts_with("unknown manifest:") => continue,
            Err(error) => return Err(error),
        };
        return harness_beat_evolution_from_proposal(
            workspace_root,
            proposal,
            state,
            "check_history",
            record.index,
            record.index,
            short_text(&one_line(&check_history_evolution_detail(record)), 160),
            None,
        )
        .map(Some);
    }
    for trace in state
        .feed_traces
        .iter()
        .rev()
        .filter(|trace| evolution_actionable_status(&trace.status))
    {
        let Some(tentacle_id) = trace.tentacle.as_deref() else {
            continue;
        };
        let objective = feed_trace_evolution_objective(trace);
        let proposal = match propose_tentacle_evolution_with_client(
            tentacles_root,
            tentacle_id,
            &objective,
            state,
            client,
        ) {
            Ok(proposal) => proposal,
            Err(error) if error.starts_with("unknown manifest:") => continue,
            Err(error) => return Err(error),
        };
        return harness_beat_evolution_from_proposal(
            workspace_root,
            proposal,
            state,
            "feed_trace",
            trace.index,
            0,
            short_text(&one_line(&trace.summary), 160),
            None,
        )
        .map(Some);
    }
    for outcome in state
        .repair_outcomes
        .iter()
        .rev()
        .filter(|outcome| evolution_actionable_status(&outcome.status))
    {
        let tentacle_id = repair_outcome_evolution_tentacle(outcome);
        let objective = repair_outcome_evolution_objective(outcome);
        let proposal = match propose_tentacle_evolution_with_client(
            tentacles_root,
            tentacle_id,
            &objective,
            state,
            client,
        ) {
            Ok(proposal) => proposal,
            Err(error) if error.starts_with("unknown manifest:") => continue,
            Err(error) => return Err(error),
        };
        return harness_beat_evolution_from_proposal(
            workspace_root,
            proposal,
            state,
            "repair_outcome",
            outcome.index,
            0,
            short_text(&one_line(&outcome.summary), 160),
            outcome.candidate.as_deref(),
        )
        .map(Some);
    }
    Ok(None)
}

fn harness_beat_evolution_from_proposal(
    workspace_root: &Path,
    proposal: TentacleEvolutionProposal,
    state: &HarnessState,
    source_kind: &str,
    source_index: u64,
    source_check_index: u64,
    source_summary: String,
    preferred_candidate: Option<&str>,
) -> Result<HarnessBeatEvolution, String> {
    let proposal_artifact = write_tentacle_evolution_artifacts(workspace_root, &proposal)?;
    let recommendation =
        recommend_tentacle_evolution_apply_with_preference(&proposal, state, preferred_candidate)?;
    let apply_artifact = write_tentacle_apply_artifacts(workspace_root, &recommendation.apply)?;
    let apply_plan_preview = fs::read_to_string(&apply_artifact.plan_path)
        .map(|content| short_text(&content, 2000))
        .unwrap_or_default();
    let next_action = if recommendation.apply.authorized {
        format!(
            "octopus evolve apply {} {}",
            recommendation.tentacle_id, recommendation.candidate_id
        )
    } else {
        format!(
            "octopus oauth octopus evolve:{} harness:write",
            recommendation.tentacle_id
        )
    };
    Ok(HarnessBeatEvolution {
        source_check_index,
        source_kind: source_kind.to_string(),
        source_index,
        source_summary,
        tentacle_id: recommendation.tentacle_id,
        objective: recommendation.objective,
        candidate_id: recommendation.candidate_id,
        status: recommendation.apply.status,
        reason: recommendation.reason,
        recommendation_score: recommendation.recommendation_score,
        proposal_path: proposal_artifact.proposal_path,
        apply_plan_path: apply_artifact.plan_path,
        apply_plan_preview,
        patch_path: apply_artifact.patch_path,
        next_action,
    })
}

fn evolution_actionable_status(status: &Status) -> bool {
    matches!(status, Status::Failed | Status::Partial)
}

fn check_history_evolution_detail(record: &CheckHistoryRecord) -> String {
    if record.stderr.trim().is_empty() {
        record
            .stdout
            .trim()
            .is_empty()
            .then(|| record.command.clone())
            .unwrap_or_else(|| record.stdout.clone())
    } else {
        record.stderr.clone()
    }
}

fn check_history_evolution_objective(record: &CheckHistoryRecord) -> String {
    let detail = check_history_evolution_detail(record);
    format!(
        "repair check #{} for {}: {}",
        record.index,
        record.tentacle_id,
        short_text(&one_line(&detail), 160)
    )
}

fn feed_trace_evolution_objective(trace: &FeedTraceRecord) -> String {
    let tentacle = trace.tentacle.as_deref().unwrap_or("unknown");
    let tool = trace.tool.as_deref().unwrap_or("unknown tool");
    format!(
        "repair Feed trace #{} for {} via {}: {}",
        trace.index,
        tentacle,
        tool,
        short_text(&one_line(&trace.summary), 160)
    )
}

fn repair_outcome_evolution_tentacle(outcome: &RepairOutcome) -> &str {
    outcome
        .target_tentacle
        .as_deref()
        .unwrap_or(&outcome.tentacle_id)
}

fn repair_outcome_evolution_objective(outcome: &RepairOutcome) -> String {
    let tool = outcome.tool.as_deref().unwrap_or("unknown tool");
    let target = repair_outcome_evolution_tentacle(outcome);
    let candidate = outcome.candidate.as_deref().unwrap_or("unknown candidate");
    format!(
        "repair outcome #{} for {} via {} candidate {} ({:?}): {}",
        outcome.index,
        target,
        tool,
        candidate,
        outcome.status,
        short_text(&one_line(&outcome.summary), 160)
    )
}

fn clean_preferred_candidate(value: Option<&str>) -> Option<&str> {
    let value = value?.trim();
    (!value.is_empty()).then_some(value)
}

fn candidate_matches(candidate: &EvolutionPatchCandidate, value: &str) -> bool {
    candidate.id == value
        || candidate.surface_id == value
        || candidate.id.replace('-', "_") == value
        || candidate.surface_id.replace('_', "-") == value
}

fn evolution_candidate_outcome_score(
    candidate: &EvolutionPatchCandidate,
    proposal: &TentacleEvolutionProposal,
) -> (f32, usize) {
    let outcomes = proposal
        .previous_outcomes
        .iter()
        .filter(|outcome| candidate_matches(candidate, &outcome.candidate_id))
        .collect::<Vec<_>>();
    if outcomes.is_empty() {
        return (0.0, 0);
    }
    let weighted = outcomes
        .iter()
        .enumerate()
        .map(|(index, outcome)| {
            let weight = (index + 1) as f32;
            (outcome.score * weight, weight)
        })
        .fold((0.0, 0.0), |acc, item| (acc.0 + item.0, acc.1 + item.1));
    (weighted.0 / weighted.1.max(1.0), outcomes.len())
}

fn evolution_candidate_check_history_score(
    candidate: &EvolutionPatchCandidate,
    proposal: &TentacleEvolutionProposal,
) -> (f32, usize) {
    let matched = proposal
        .recent_check_history
        .iter()
        .filter(|record| candidate_matches_check_history(candidate, record))
        .collect::<Vec<_>>();
    if matched.is_empty() {
        return (0.0, 0);
    }
    let weighted = matched
        .iter()
        .enumerate()
        .map(|(index, record)| {
            let weight = (index + 1) as f32;
            let score = match record.status {
                Status::Failed => 0.45,
                Status::Partial => 0.2,
                Status::Satisfied => 0.05,
                Status::Unsupported => 0.0,
            };
            (score * weight, weight)
        })
        .fold((0.0, 0.0), |acc, item| (acc.0 + item.0, acc.1 + item.1));
    (weighted.0 / weighted.1.max(1.0), matched.len())
}

fn evolution_candidate_feed_trace_score(
    candidate: &EvolutionPatchCandidate,
    proposal: &TentacleEvolutionProposal,
) -> (f32, usize) {
    let matched = proposal
        .recent_feed_traces
        .iter()
        .filter(|trace| candidate_matches_feed_trace(candidate, trace))
        .collect::<Vec<_>>();
    if matched.is_empty() {
        return (0.0, 0);
    }
    let weighted = matched
        .iter()
        .enumerate()
        .map(|(index, trace)| {
            let weight = (index + 1) as f32;
            let score = match trace.status {
                Status::Failed => 0.35,
                Status::Partial => 0.18,
                Status::Satisfied => 0.05,
                Status::Unsupported => 0.0,
            };
            (score * weight, weight)
        })
        .fold((0.0, 0.0), |acc, item| (acc.0 + item.0, acc.1 + item.1));
    (weighted.0 / weighted.1.max(1.0), matched.len())
}

pub(crate) fn evolution_candidate_feedback_from_proposal(
    candidate: &EvolutionPatchCandidate,
    proposal: &TentacleEvolutionProposal,
) -> Vec<EvolutionPatchFeedback> {
    let mut feedback = Vec::new();
    if let Some(record) = proposal
        .recent_check_history
        .iter()
        .rev()
        .find(|record| candidate_matches_check_history(candidate, record))
    {
        feedback.push(feedback_from_check_history(record, None));
    }
    if let Some(trace) = proposal
        .recent_feed_traces
        .iter()
        .rev()
        .find(|trace| candidate_matches_feed_trace(candidate, trace))
    {
        feedback.push(feedback_from_feed_trace(trace, None));
    }
    feedback
}

fn candidate_matches_check_history(
    candidate: &EvolutionPatchCandidate,
    record: &CheckHistoryRecord,
) -> bool {
    let target = candidate
        .target
        .split('#')
        .next()
        .unwrap_or(&candidate.target);
    if !target.contains('*') && !target.is_empty() && record.command.contains(target) {
        return true;
    }
    if let Some(name) = Path::new(target).file_name().and_then(|name| name.to_str()) {
        if !name.is_empty() && name != "*" && record.command.contains(name) {
            return true;
        }
    }
    candidate.surface_id == "runtime_code"
        && record.status == Status::Failed
        && record.command.contains("tools/")
}

fn candidate_matches_feed_trace(
    candidate: &EvolutionPatchCandidate,
    trace: &FeedTraceRecord,
) -> bool {
    if field_mini_task_template_candidate_matches_trace(candidate, trace) {
        return true;
    }
    let Some(tool) = trace.tool.as_deref() else {
        return false;
    };
    target_mentions_tool(&candidate.target, tool)
}

fn field_mini_task_template_candidate_matches_trace(
    candidate: &EvolutionPatchCandidate,
    trace: &FeedTraceRecord,
) -> bool {
    if candidate.surface_id != "runtime_code" || trace.status == Status::Satisfied {
        return false;
    }
    let Some(field) = trace.metadata.get("field_pack") else {
        return false;
    };
    let Some(mini_task) = trace.metadata.get("field_mini_task") else {
        return false;
    };
    let worker = format!("workers/{field}/{mini_task}/main.go");
    let legacy_template = format!("repair-templates/{field}/{mini_task}.pyfrag");
    candidate.target.contains(&worker)
        || candidate.target.contains(&legacy_template)
        || candidate
            .suggested_patch
            .as_deref()
            .is_some_and(|patch| patch.contains(&worker) || patch.contains(&legacy_template))
}

fn target_mentions_tool(target: &str, tool: &str) -> bool {
    if target.contains(tool) {
        return true;
    }
    Path::new(target.split('#').next().unwrap_or(target))
        .file_stem()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == tool)
}

fn feedback_from_check_history(
    record: &CheckHistoryRecord,
    tool: Option<&ManifestTool>,
) -> EvolutionPatchFeedback {
    let summary = if record.stderr.trim().is_empty() {
        one_line(&record.stdout)
    } else {
        one_line(&record.stderr)
    };
    EvolutionPatchFeedback {
        kind: "check_history".to_string(),
        source_index: record.index,
        status: record.status.clone(),
        summary,
        command: Some(record.command.clone()),
        target: tool.map(|tool| tool.implementation.entrypoint.clone()),
    }
}

fn feedback_from_feed_trace(
    trace: &FeedTraceRecord,
    tool: Option<&ManifestTool>,
) -> EvolutionPatchFeedback {
    EvolutionPatchFeedback {
        kind: "feed_trace".to_string(),
        source_index: trace.index,
        status: trace.status.clone(),
        summary: one_line(&trace.summary),
        command: None,
        target: tool
            .map(|tool| tool.implementation.entrypoint.clone())
            .or_else(|| trace.tool.clone()),
    }
}
