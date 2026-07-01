use crate::{
    evolution,
    evolution_target::{evolution_file_target, evolution_target_files},
    one_line, push_unique_limited, short_text, CheckHistoryRecord, EvolutionFileTarget,
    EvolutionOutcome, EvolutionSurface, FeedTraceRecord, TentacleEvolutionProposal,
};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const FIELD_PACK_SCHEMA_JSON: &str = include_str!("../../../field-packs/field-pack.schema.json");

pub(crate) fn llm_evolution_prompt(proposal: &TentacleEvolutionProposal) -> Result<String, String> {
    let required_surfaces = evolution::required_surfaces_for_objective(proposal);
    let payload = serde_json::json!({
        "tentacle_id": proposal.tentacle_id,
        "tentacle_name": proposal.tentacle_name,
        "objective": proposal.objective,
        "required_surfaces": required_surfaces,
        "brain_kind": proposal.brain_kind,
        "current_brain_prompt": short_text(&proposal.current_brain_prompt, 1200),
        "editable": proposal.editable,
        "surfaces": proposal.surfaces,
        "surface_requirements": proposal.requirements,
        "checks": proposal.checks,
        "constraints": proposal.constraints,
        "previous_outcomes": compact_evolution_outcomes(&proposal.previous_outcomes),
        "recent_feed_traces": compact_feed_trace_records(&proposal.recent_feed_traces),
        "recent_check_history": compact_check_history_records(&proposal.recent_check_history),
        "files": compact_evolution_file_targets(&proposal.files),
        "target_file_contents": evolution_target_file_contexts(proposal),
        "supporting_contracts": supporting_evolution_contracts(&required_surfaces),
        "context_policy": {
            "clean_brain": ["Goal", "Mem", "Need", "Feed"],
            "tentacle_brain": ["Need", "Tool", "Action", "Tool", "Action", "Feed"],
            "harness_evolution": "modify prompt, metadata, runtime code, or policy without moving tool burden into the clean brain"
        },
        "prompt_budget": {
            "policy": "compact summaries are intentional so local OpenAI-compatible models can evolve harnesses without context overflow",
            "target_file_limit": llm_evolution_file_limit(),
            "target_file_bytes_default": llm_evolution_file_bytes(),
            "target_file_bytes_for_small_scope": llm_evolution_file_bytes_for_path_count(8)
        },
        "patch_requirements": {
            "format": "suggested_patch must be a complete git unified diff that starts with diff --git, includes --- and +++ file headers, includes @@ -line,count +line,count @@ hunk headers, and can be applied by git apply. Use exact nearby lines from target_file_contents.content, target_file_contents.line_numbered_content, and target_file_contents.line_numbered_tail_content when present; do not invent line numbers or old context. On apply retry, treat line_numbered_content and line_numbered_tail_content as the current file. Do not return apply_patch, *** Begin Patch, or prose-only patches.",
            "field_pack_tasks": "suggested_patch is required for this surface. If required_surfaces contains field_pack_tasks, at least one candidate must use surface_id=field_pack_tasks. Patch only the named field-pack JSON files and matching tentacles/<tentacle>/repair-templates/<field>/<mini-task>.pyfrag files needed by the objective. New mini task layers must add both the field-pack entry and matching repair template. Field-pack mini_tasks is an array of objects with id, goal, and expected_feed; append the new object inside mini_tasks before the array closing bracket, preserve the comma pattern around the previous object, and keep the JSON schema valid. Use line_numbered_tail_content for end-of-array context when it is present. Do not create task_layers or convert mini_tasks to an object. Do not merely harden an existing mini task when the objective asks for a harder layer. Do not patch kernel Rust or unrelated runtime code from this surface.",
            "runtime_code": "suggested_patch is preferred when the objective names an executable harness gap. Patch declared harness targets only."
        },
        "return_schema": {
            "summary": "short reason for the selected harness changes",
            "candidates": [
                {
                    "surface_id": "one declared surface id",
                    "title": "short title",
                    "target": "one declared target or target-file path; field-pack surfaces may use field-packs/... paths outside the tentacle directory",
                    "rationale": "why this improves Need-to-Feed",
                    "change_plan": ["concrete harness edit step"],
                    "checks": ["command to verify"],
                    "suggested_patch": "complete git unified diff for declared target files only; must start with diff --git and include ---/+++ plus @@ -line,count +line,count @@ hunk headers; required for field_pack_tasks"
                }
            ]
        }
    });
    let payload = serde_json::to_string_pretty(&payload).map_err(|error| error.to_string())?;
    Ok(format!(
        "Generate code-as-harness evolution candidates from this JSON. Keep the clean brain out of tool details and preserve the declared context policy. Return only a JSON object matching return_schema. Any suggested_patch must be a complete git unified diff that starts with diff --git, includes --- and +++ file headers, includes @@ -line,count +line,count @@ hunk headers, and applies with git apply; never return apply_patch or *** Begin Patch format.\n{payload}"
    ))
}

fn supporting_evolution_contracts(required_surfaces: &[String]) -> Vec<serde_json::Value> {
    let mut contracts = Vec::new();
    if required_surfaces
        .iter()
        .any(|surface| surface == "field_pack_tasks")
    {
        contracts.push(serde_json::json!({
            "id": "field-pack-schema",
            "path": "field-packs/field-pack.schema.json",
            "contract": "mini_tasks is an array; each item has id, goal, expected_feed",
            "invalid_shapes": ["task_layers", "mini_tasks as object/map"],
            "content": short_text(FIELD_PACK_SCHEMA_JSON, 2400)
        }));
    }
    contracts
}

fn compact_evolution_outcomes(outcomes: &[EvolutionOutcome]) -> Vec<serde_json::Value> {
    outcomes
        .iter()
        .rev()
        .take(5)
        .rev()
        .map(|outcome| {
            serde_json::json!({
                "index": outcome.index,
                "tentacle_id": outcome.tentacle_id,
                "candidate_id": outcome.candidate_id,
                "status": outcome.status,
                "score": outcome.score,
                "summary": short_text(&one_line(&outcome.summary), 240)
            })
        })
        .collect()
}

fn compact_feed_trace_records(traces: &[FeedTraceRecord]) -> Vec<serde_json::Value> {
    traces
        .iter()
        .rev()
        .take(6)
        .rev()
        .map(|trace| {
            serde_json::json!({
                "index": trace.index,
                "need_kind": trace.need_kind,
                "need_query": short_text(&one_line(&trace.need_query), 180),
                "status": trace.status,
                "field": trace.field,
                "tentacle": trace.tentacle,
                "tool": trace.tool,
                "plan_source": trace.plan_source,
                "route": trace.route,
                "evidence_count": trace.evidence_count,
                "summary": short_text(&one_line(&trace.summary), 320),
                "metadata": compact_feed_trace_metadata(&trace.metadata)
            })
        })
        .collect()
}

fn compact_feed_trace_metadata(metadata: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut compact = BTreeMap::new();
    for key in [
        "field_pack",
        "field_mini_task",
        "field_expected_feed",
        "verifier_status",
        "error_category",
        "field_pass_evidence",
        "repair_template",
        "runtime_template",
        "tool",
    ] {
        if let Some(value) = metadata.get(key) {
            compact.insert(key.to_string(), short_text(&one_line(value), 240));
        }
    }
    compact
}

fn compact_check_history_records(records: &[CheckHistoryRecord]) -> Vec<serde_json::Value> {
    records
        .iter()
        .rev()
        .take(6)
        .rev()
        .map(|record| {
            serde_json::json!({
                "index": record.index,
                "tentacle_id": record.tentacle_id,
                "source_kind": record.source_kind,
                "command_index": record.command_index,
                "command": short_text(&one_line(&record.command), 220),
                "cwd": record.cwd,
                "status": record.status,
                "code": record.code,
                "stdout": short_text(&one_line(&record.stdout), 320),
                "stderr": short_text(&one_line(&record.stderr), 320)
            })
        })
        .collect()
}

fn compact_evolution_file_targets(files: &[EvolutionFileTarget]) -> Vec<serde_json::Value> {
    files
        .iter()
        .take(24)
        .map(|file| {
            serde_json::json!({
                "path": file.path,
                "target_files": file.target_files.iter().take(12).collect::<Vec<_>>(),
                "target_file_count": file.target_files.len(),
                "action": file.action,
                "rationale": short_text(&one_line(&file.rationale), 180)
            })
        })
        .collect()
}

fn evolution_target_file_contexts(proposal: &TentacleEvolutionProposal) -> Vec<serde_json::Value> {
    let mut paths = Vec::new();
    let file_limit = llm_evolution_file_limit();
    for path in objective_path_hints(&proposal.objective) {
        push_unique_limited(&mut paths, path, file_limit);
    }
    let manifest_dir = Path::new(&proposal.manifest_path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("tentacles").join(&proposal.tentacle_id));
    for surface in prioritized_evolution_surfaces(&proposal.surfaces) {
        for target in &surface.targets {
            let file = evolution_file_target(&manifest_dir, target, &proposal.objective);
            push_unique_limited(&mut paths, file.path, file_limit);
            for path in file.target_files {
                push_unique_limited(&mut paths, path, file_limit);
            }
        }
    }
    for candidate in &proposal.patch_candidates {
        for path in &candidate.target_files {
            push_unique_limited(&mut paths, path.clone(), file_limit);
        }
    }
    for file in &proposal.files {
        push_unique_limited(&mut paths, file.path.clone(), file_limit);
        for path in evolution_target_files(&file.path) {
            push_unique_limited(&mut paths, path, file_limit);
        }
        for path in &file.target_files {
            push_unique_limited(&mut paths, path.clone(), file_limit);
        }
    }
    let file_bytes = llm_evolution_file_bytes_for_path_count(paths.len());
    paths
        .into_iter()
        .filter_map(|path| {
            let content = fs::read_to_string(&path).ok()?;
            let line_numbered_content = line_numbered_text(&content);
            if line_numbered_content.len() > file_bytes {
                Some(serde_json::json!({
                    "path": path,
                    "content": short_text(&content, file_bytes),
                    "line_numbered_content": short_text(&line_numbered_content, file_bytes),
                    "line_numbered_tail_content": line_numbered_tail_text(&content, 40),
                    "truncated": true
                }))
            } else {
                Some(serde_json::json!({
                    "path": path,
                    "content": short_text(&content, file_bytes),
                    "line_numbered_content": line_numbered_content,
                    "truncated": content.len() > file_bytes
                }))
            }
        })
        .collect()
}

fn objective_path_hints(objective: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for token in objective
        .split(|ch: char| ch.is_whitespace() || matches!(ch, ':' | ',' | ';' | '"' | '\''))
        .map(|value| value.trim_matches(|ch: char| matches!(ch, '(' | ')' | '[' | ']')))
    {
        if token.contains('/') && Path::new(token).exists() {
            push_unique_limited(&mut paths, token.to_string(), 16);
        }
    }
    paths
}

fn line_numbered_text(content: &str) -> String {
    content
        .lines()
        .enumerate()
        .map(|(index, line)| format!("{:>4}: {line}", index + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn line_numbered_tail_text(content: &str, max_lines: usize) -> String {
    let lines = content.lines().collect::<Vec<_>>();
    let start = lines.len().saturating_sub(max_lines);
    lines
        .iter()
        .enumerate()
        .skip(start)
        .map(|(index, line)| format!("{:>4}: {line}", index + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn prioritized_evolution_surfaces(surfaces: &[EvolutionSurface]) -> Vec<&EvolutionSurface> {
    let mut ordered = Vec::new();
    for preferred in ["field_pack_tasks", "runtime_code"] {
        ordered.extend(surfaces.iter().filter(|surface| surface.id == preferred));
    }
    ordered.extend(
        surfaces
            .iter()
            .filter(|surface| surface.id != "field_pack_tasks" && surface.id != "runtime_code"),
    );
    ordered
}

fn llm_evolution_file_limit() -> usize {
    env_usize("OCTOPUS_LLM_EVOLVE_FILE_LIMIT", 12).clamp(1, 64)
}

fn llm_evolution_file_bytes() -> usize {
    env_usize("OCTOPUS_LLM_EVOLVE_FILE_BYTES", 1200).clamp(400, 6000)
}

fn llm_evolution_file_bytes_for_path_count(path_count: usize) -> usize {
    if let Some(value) = env_usize_optional("OCTOPUS_LLM_EVOLVE_FILE_BYTES") {
        return value.clamp(400, 6000);
    }
    if path_count <= 8 {
        return 4000;
    }
    1200
}

fn env_usize(key: &str, default: usize) -> usize {
    env_usize_optional(key).unwrap_or(default)
}

fn env_usize_optional(key: &str) -> Option<usize> {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn line_numbered_text_preserves_current_patch_context() {
        let numbered = line_numbered_text("alpha\nbeta\n");

        assert!(numbered.contains("   1: alpha"));
        assert!(numbered.contains("   2: beta"));
    }

    #[test]
    fn line_numbered_tail_text_preserves_end_of_file_context() {
        let content = (1..=80)
            .map(|index| format!("line-{index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let tail = line_numbered_tail_text(&content, 4);

        assert!(!tail.contains("   1: line-1"));
        assert!(tail.contains("  77: line-77"));
        assert!(tail.contains("  80: line-80"));
    }

    #[test]
    fn objective_path_hints_prioritize_existing_retry_targets() {
        let dir = env::temp_dir().join(format!(
            "octopus-evolution-prompt-paths-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("retry.pyfrag");
        fs::write(&file, "if field:\n    pass\n").unwrap();
        let objective = format!(
            "regenerate patch after git apply failed: error: patch failed: {}:18",
            file.display()
        );

        let paths = objective_path_hints(&objective);

        assert_eq!(paths, vec![file.to_string_lossy().to_string()]);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn field_pack_task_prompt_includes_schema_contract() {
        let contracts = supporting_evolution_contracts(&["field_pack_tasks".to_string()]);

        let text = serde_json::to_string(&contracts).unwrap();
        assert!(text.contains("field-pack.schema.json"));
        assert!(text.contains("mini_tasks is an array"));
        assert!(text.contains("task_layers"));
    }

    #[test]
    fn field_pack_task_prompt_points_to_tail_context() {
        let proposal = TentacleEvolutionProposal {
            tentacle_id: "field-mini-task".to_string(),
            tentacle_name: "Field Mini Task".to_string(),
            objective: "add a harder math mini task layer".to_string(),
            generator: "test".to_string(),
            planner_summary: String::new(),
            manifest_path: "tentacles/field-mini-task/manifest.json".to_string(),
            brain_kind: "llm".to_string(),
            current_brain_prompt: String::new(),
            editable: Vec::new(),
            surfaces: vec![EvolutionSurface {
                id: "field_pack_tasks".to_string(),
                description: "field tasks".to_string(),
                targets: vec!["field-packs/*/field-pack.json".to_string()],
            }],
            requirements: Vec::new(),
            checks: Vec::new(),
            constraints: Vec::new(),
            previous_outcomes: Vec::new(),
            recent_feed_traces: Vec::new(),
            recent_check_history: Vec::new(),
            files: Vec::new(),
            patch_candidates: Vec::new(),
            next_steps: Vec::new(),
        };

        let prompt = llm_evolution_prompt(&proposal).unwrap();

        assert!(prompt.contains("line_numbered_tail_content"));
        assert!(prompt.contains("before the array closing bracket"));
    }
}
