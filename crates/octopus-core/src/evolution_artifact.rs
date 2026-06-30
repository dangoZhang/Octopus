use crate::{
    evolution_patch::authorized_apply_patch, kind_key, one_line, EvolutionApplyArtifact,
    EvolutionApplyPlan, EvolutionArtifact, EvolutionPatchCandidate, EvolutionPatchFeedback, Status,
    TentacleEvolutionProposal,
};
use std::fs;
use std::path::Path;

pub fn write_tentacle_evolution_artifacts(
    workspace_root: impl AsRef<Path>,
    proposal: &TentacleEvolutionProposal,
) -> Result<EvolutionArtifact, String> {
    let directory = workspace_root
        .as_ref()
        .join(".octopus")
        .join("evolution")
        .join(&proposal.tentacle_id);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let patches_dir = directory.join("patches");
    fs::create_dir_all(&patches_dir).map_err(|error| error.to_string())?;
    let proposal_path = directory.join("PROPOSAL.md");
    let candidates_path = directory.join("PATCH_CANDIDATES.md");
    let drafts_path = directory.join("PATCH_DRAFTS.md");
    let json_path = directory.join("proposal.json");
    fs::write(&proposal_path, render_tentacle_evolution_proposal(proposal))
        .map_err(|error| error.to_string())?;
    fs::write(&candidates_path, render_tentacle_patch_candidates(proposal))
        .map_err(|error| error.to_string())?;
    fs::write(&drafts_path, render_tentacle_patch_drafts(proposal))
        .map_err(|error| error.to_string())?;
    let mut patch_draft_paths = Vec::new();
    for candidate in &proposal.patch_candidates {
        let draft_path = directory.join(&candidate.draft.path);
        if let Some(parent) = draft_path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(&draft_path, render_single_patch_draft(proposal, candidate))
            .map_err(|error| error.to_string())?;
        patch_draft_paths.push(draft_path.to_string_lossy().to_string());
    }
    fs::write(
        &json_path,
        serde_json::to_string_pretty(proposal).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(EvolutionArtifact {
        directory: directory.to_string_lossy().to_string(),
        proposal_path: proposal_path.to_string_lossy().to_string(),
        candidates_path: candidates_path.to_string_lossy().to_string(),
        drafts_path: drafts_path.to_string_lossy().to_string(),
        patch_draft_paths,
        json_path: json_path.to_string_lossy().to_string(),
    })
}

pub fn write_tentacle_apply_artifacts(
    workspace_root: impl AsRef<Path>,
    plan: &EvolutionApplyPlan,
) -> Result<EvolutionApplyArtifact, String> {
    let directory = workspace_root
        .as_ref()
        .join(".octopus")
        .join("evolution")
        .join(&plan.tentacle_id)
        .join("apply");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let stem = plan.candidate_id.replace('/', "-");
    let plan_path = directory.join(format!("{stem}.md"));
    let patch_file_path = directory.join(format!("{stem}.patch"));
    let patch_path = if plan.authorized {
        if let Some(patch) = authorized_apply_patch(plan) {
            fs::write(&patch_file_path, patch).map_err(|error| error.to_string())?;
            Some(patch_file_path)
        } else {
            if patch_file_path.exists() {
                fs::remove_file(&patch_file_path).map_err(|error| error.to_string())?;
            }
            None
        }
    } else {
        if patch_file_path.exists() {
            fs::remove_file(&patch_file_path).map_err(|error| error.to_string())?;
        }
        None
    };
    let json_path = directory.join(format!("{stem}.json"));
    fs::write(&plan_path, render_tentacle_apply_plan(plan)).map_err(|error| error.to_string())?;
    fs::write(
        &json_path,
        serde_json::to_string_pretty(plan).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(EvolutionApplyArtifact {
        directory: directory.to_string_lossy().to_string(),
        plan_path: plan_path.to_string_lossy().to_string(),
        patch_path: patch_path.map(|path| path.to_string_lossy().to_string()),
        json_path: json_path.to_string_lossy().to_string(),
    })
}

pub fn render_tentacle_apply_plan(plan: &EvolutionApplyPlan) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# Evolution Apply Plan: {}\n\n",
        plan.candidate_id
    ));
    markdown.push_str(&format!("tentacle: `{}`\n", plan.tentacle_id));
    markdown.push_str(&format!("objective: {}\n", plan.objective));
    markdown.push_str(&format!("status: `{}`\n", plan.status));
    markdown.push_str(&format!("authorized: {}\n", plan.authorized));
    markdown.push_str(&format!("required_grant: `{}`\n", plan.required_grant));
    if let Some(grant) = &plan.active_grant {
        markdown.push_str(&format!("active_grant: `{grant}`\n"));
    }
    markdown.push_str(&format!("target: `{}`\n", plan.target));
    if !plan.target_files.is_empty() {
        markdown.push_str(&format!(
            "target_files: `{}`\n",
            plan.target_files.join("`, `")
        ));
    }
    markdown.push_str(&format!("draft: `{}`\n\n", plan.draft_path));
    if plan.authorized && authorized_apply_patch(plan).is_some() {
        markdown.push_str(&format!("patch: `{}.patch`\n\n", plan.candidate_id));
    }
    if !plan.feedback.is_empty() {
        markdown.push_str("feedback focus:\n");
        for feedback in &plan.feedback {
            markdown.push_str(&format!("- {}\n", evolution_feedback_line(feedback)));
        }
        markdown.push('\n');
    }
    if let Some(patch) = &plan.suggested_patch {
        markdown.push_str("suggested patch draft:\n");
        markdown.push_str("```diff\n");
        markdown.push_str(patch.trim());
        markdown.push_str("\n```\n\n");
    }
    markdown.push_str("guardrails:\n");
    for guardrail in &plan.guardrails {
        markdown.push_str(&format!("- {guardrail}\n"));
    }
    markdown.push_str("\nchecks:\n");
    for check in &plan.checks {
        markdown.push_str(&format!("- `{check}`\n"));
    }
    markdown.push_str("\nnext steps:\n");
    for step in &plan.next_steps {
        markdown.push_str(&format!("- {step}\n"));
    }
    markdown
}

pub fn render_authorized_apply_patch(plan: &EvolutionApplyPlan, _apply_dir: &Path) -> String {
    authorized_apply_patch(plan).unwrap_or_default()
}

pub fn render_tentacle_evolution_proposal(proposal: &TentacleEvolutionProposal) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# Tentacle Evolution: {}\n\n",
        proposal.tentacle_id
    ));
    markdown.push_str(&format!("objective: {}\n", proposal.objective));
    markdown.push_str(&format!("generator: `{}`\n", proposal.generator));
    if !proposal.planner_summary.trim().is_empty() {
        markdown.push_str(&format!("planner: {}\n", proposal.planner_summary));
    }
    markdown.push_str(&format!("manifest: {}\n", proposal.manifest_path));
    markdown.push_str(&format!("brain: {}\n\n", proposal.brain_kind));
    markdown.push_str("## Current Brain Prompt\n\n");
    markdown.push_str("```text\n");
    markdown.push_str(&proposal.current_brain_prompt);
    markdown.push_str("\n```\n\n");
    markdown.push_str("## Editable Targets\n\n");
    for file in &proposal.files {
        markdown.push_str(&format!(
            "- `{}`: {} ({})\n",
            file.path, file.action, file.rationale
        ));
        if !file.target_files.is_empty() {
            markdown.push_str(&format!(
                "  target_files: `{}`\n",
                file.target_files.join("`, `")
            ));
        }
    }
    markdown.push_str("\n## Evolution Surfaces\n\n");
    for surface in &proposal.surfaces {
        markdown.push_str(&format!(
            "- `{}`: {} [{}]\n",
            surface.id,
            surface.description,
            surface.targets.join(", ")
        ));
    }
    if !proposal.requirements.is_empty() {
        markdown.push_str("\n## Surface Requirements\n\n");
        for requirement in &proposal.requirements {
            markdown.push_str(&format!(
                "- `{}`: {} -> `{}`\n",
                requirement.id,
                requirement.description,
                requirement.required_surfaces.join("`, `")
            ));
        }
    }
    markdown.push_str("\n## Constraints\n\n");
    for constraint in &proposal.constraints {
        markdown.push_str(&format!("- {constraint}\n"));
    }
    markdown.push_str("\n## Checks\n\n");
    for check in &proposal.checks {
        markdown.push_str(&format!("- `{check}`\n"));
    }
    if !proposal.previous_outcomes.is_empty() {
        markdown.push_str("\n## Previous Outcomes\n\n");
        for outcome in &proposal.previous_outcomes {
            markdown.push_str(&format!(
                "- `{}`: {:?} score={:.2} {}\n",
                outcome.candidate_id, outcome.status, outcome.score, outcome.summary
            ));
        }
    }
    if !proposal.recent_feed_traces.is_empty() {
        markdown.push_str("\n## Recent Feed Traces\n\n");
        for trace in &proposal.recent_feed_traces {
            let tentacle = trace.tentacle.as_deref().unwrap_or("unknown");
            let tool = trace.tool.as_deref().unwrap_or("unknown");
            let plan = trace.plan_source.as_deref().unwrap_or("unknown");
            markdown.push_str(&format!(
                "- #{} {}:{} -> {:?} via `{}/{}` plan={} evidence={} :: {}\n",
                trace.index,
                kind_key(&trace.need_kind),
                trace.need_query,
                trace.status,
                tentacle,
                tool,
                plan,
                trace.evidence_count,
                trace.summary
            ));
        }
    }
    if !proposal.recent_check_history.is_empty() {
        markdown.push_str("\n## Recent Check History\n\n");
        for record in &proposal.recent_check_history {
            let label = match record.status {
                Status::Satisfied => "ok",
                Status::Failed => "failed",
                Status::Partial => "partial",
                Status::Unsupported => "unsupported",
            };
            let check = record
                .command_index
                .map(|index| format!("check {index}"))
                .unwrap_or_else(|| "check".to_string());
            markdown.push_str(&format!(
                "- #{} {label} {} `{}` code={:?} :: {}\n",
                record.index,
                check,
                record.command,
                record.code,
                one_line(&record.stderr)
            ));
        }
    }
    markdown.push_str("\n## Patch Candidates\n\n");
    for candidate in &proposal.patch_candidates {
        markdown.push_str(&format!(
            "- `{}`: {} -> `{}`\n",
            candidate.id, candidate.title, candidate.target
        ));
    }
    markdown.push_str("\n## Next Steps\n\n");
    for step in &proposal.next_steps {
        markdown.push_str(&format!("- {step}\n"));
    }
    markdown
}

pub fn render_tentacle_patch_candidates(proposal: &TentacleEvolutionProposal) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!("# Patch Candidates: {}\n\n", proposal.tentacle_id));
    markdown.push_str(&format!("objective: {}\n\n", proposal.objective));
    markdown.push_str(&format!("generator: `{}`\n\n", proposal.generator));
    for candidate in &proposal.patch_candidates {
        markdown.push_str(&format!("## {}. {}\n\n", candidate.id, candidate.title));
        markdown.push_str(&format!("surface: `{}`\n", candidate.surface_id));
        markdown.push_str(&format!("target: `{}`\n", candidate.target));
        if !candidate.target_files.is_empty() {
            markdown.push_str(&format!(
                "target_files: `{}`\n",
                candidate.target_files.join("`, `")
            ));
        }
        markdown.push_str(&format!("rationale: {}\n\n", candidate.rationale));
        if !candidate.feedback.is_empty() {
            markdown.push_str("feedback:\n");
            for feedback in &candidate.feedback {
                markdown.push_str(&format!("- {}\n", evolution_feedback_line(feedback)));
            }
            markdown.push('\n');
        }
        markdown.push_str(&format!("draft: `{}`\n", candidate.draft.path));
        if candidate.suggested_patch.is_some() {
            markdown.push_str("provider patch: attached\n");
        }
        markdown.push_str(&format!("status: `{}`\n", candidate.draft.status));
        markdown.push_str(&format!(
            "authorization required: {}\n\n",
            candidate.draft.authorization_required
        ));
        markdown.push_str("change plan:\n");
        for step in &candidate.change_plan {
            markdown.push_str(&format!("- {step}\n"));
        }
        markdown.push_str("\nchecks:\n");
        for check in &candidate.checks {
            markdown.push_str(&format!("- `{check}`\n"));
        }
        markdown.push('\n');
    }
    markdown
}

pub fn render_tentacle_patch_drafts(proposal: &TentacleEvolutionProposal) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!("# Patch Drafts: {}\n\n", proposal.tentacle_id));
    markdown.push_str("These drafts are local review artifacts. They do not modify harness files until a user or authorized repo tentacle turns one into a patch.\n\n");
    for candidate in &proposal.patch_candidates {
        markdown.push_str(&format!(
            "- `{}` -> `{}` ({})\n",
            candidate.id, candidate.draft.path, candidate.draft.status
        ));
    }
    markdown
}

pub fn render_single_patch_draft(
    proposal: &TentacleEvolutionProposal,
    candidate: &EvolutionPatchCandidate,
) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!("# Patch Draft: {}\n\n", candidate.id));
    markdown.push_str(&format!("tentacle: `{}`\n", proposal.tentacle_id));
    markdown.push_str(&format!("objective: {}\n", proposal.objective));
    markdown.push_str(&format!("surface: `{}`\n", candidate.surface_id));
    markdown.push_str(&format!("target: `{}`\n", candidate.target));
    if !candidate.target_files.is_empty() {
        markdown.push_str(&format!(
            "target_files: `{}`\n",
            candidate.target_files.join("`, `")
        ));
    }
    markdown.push_str(&format!("status: `{}`\n", candidate.draft.status));
    markdown.push_str(&format!(
        "authorization_required: {}\n",
        candidate.draft.authorization_required
    ));
    markdown.push_str(&format!("apply_hint: {}\n\n", candidate.draft.apply_hint));
    if !candidate.feedback.is_empty() {
        markdown.push_str("feedback focus:\n");
        for feedback in &candidate.feedback {
            markdown.push_str(&format!("- {}\n", evolution_feedback_line(feedback)));
        }
        markdown.push('\n');
    }
    if let Some(patch) = &candidate.suggested_patch {
        markdown.push_str("suggested patch draft:\n");
        markdown.push_str("```diff\n");
        markdown.push_str(patch.trim());
        markdown.push_str("\n```\n\n");
    }
    markdown.push_str("diff intent:\n");
    for step in &candidate.change_plan {
        markdown.push_str(&format!("- {step}\n"));
    }
    markdown.push_str("\nchecks:\n");
    for check in &candidate.checks {
        markdown.push_str(&format!("- `{check}`\n"));
    }
    markdown
}

fn evolution_feedback_line(feedback: &EvolutionPatchFeedback) -> String {
    let mut parts = vec![
        format!("{} #{}", feedback.kind, feedback.source_index),
        format!("{:?}", feedback.status),
    ];
    if let Some(target) = &feedback.target {
        parts.push(format!("target `{target}`"));
    }
    if let Some(command) = &feedback.command {
        parts.push(format!("command `{}`", one_line(command)));
    }
    if !feedback.summary.trim().is_empty() {
        parts.push(format!("summary {}", one_line(&feedback.summary)));
    }
    parts.join(" · ")
}
