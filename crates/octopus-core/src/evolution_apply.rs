use crate::shell_words::shell_arg;
use octopus_core::{
    evolution_patch_diff_paths, evolution_patch_unauthorized_diff_paths_for_plan,
    EvolutionApplyArtifact, EvolutionApplyPlan, EvolutionRecommendation,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct EvolutionLiveApplyReport {
    pub(crate) applied: bool,
    pub(crate) status: String,
    pub(crate) command: Option<String>,
    pub(crate) patch_path: Option<String>,
    pub(crate) changed_paths: Vec<String>,
    pub(crate) target_boundary_violations: Vec<String>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) fn apply_authorized_suggested_patch(
    cwd: &Path,
    plan: &EvolutionApplyPlan,
    artifact: &EvolutionApplyArtifact,
) -> EvolutionLiveApplyReport {
    if !plan.authorized {
        return EvolutionLiveApplyReport {
            applied: false,
            status: "skipped_not_authorized".to_string(),
            command: None,
            patch_path: artifact.patch_path.clone(),
            changed_paths: Vec::new(),
            target_boundary_violations: Vec::new(),
            stdout: String::new(),
            stderr: String::new(),
        };
    }
    if plan.suggested_patch.is_none() {
        return EvolutionLiveApplyReport {
            applied: false,
            status: "skipped_no_suggested_patch".to_string(),
            command: None,
            patch_path: artifact.patch_path.clone(),
            changed_paths: Vec::new(),
            target_boundary_violations: Vec::new(),
            stdout: String::new(),
            stderr: String::new(),
        };
    }
    let Some(patch_path) = artifact.patch_path.as_deref() else {
        return EvolutionLiveApplyReport {
            applied: false,
            status: "skipped_missing_patch_artifact".to_string(),
            command: None,
            patch_path: None,
            changed_paths: Vec::new(),
            target_boundary_violations: Vec::new(),
            stdout: String::new(),
            stderr: String::new(),
        };
    };
    let patch_text = match fs::read_to_string(patch_path) {
        Ok(patch_text) => patch_text,
        Err(error) => {
            return EvolutionLiveApplyReport {
                applied: false,
                status: "check_failed_to_start".to_string(),
                command: Some(format!(
                    "git apply --check --recount --unidiff-zero {}",
                    shell_arg(patch_path)
                )),
                patch_path: Some(patch_path.to_string()),
                changed_paths: Vec::new(),
                target_boundary_violations: Vec::new(),
                stdout: String::new(),
                stderr: format!("patch artifact unreadable: {error}"),
            };
        }
    };
    let changed_paths = evolution_patch_diff_paths(&patch_text);
    let target_boundary_violations =
        evolution_patch_unauthorized_diff_paths_for_plan(plan, &patch_text);
    if !target_boundary_violations.is_empty() {
        return EvolutionLiveApplyReport {
            applied: false,
            status: "target_boundary_failed".to_string(),
            command: Some(format!(
                "git apply --check --recount --unidiff-zero {}",
                shell_arg(patch_path)
            )),
            patch_path: Some(patch_path.to_string()),
            changed_paths,
            target_boundary_violations: target_boundary_violations.clone(),
            stdout: String::new(),
            stderr: format!(
                "patch target boundary violation: {}",
                target_boundary_violations.join(", ")
            ),
        };
    }
    let mut effective_patch_path = patch_path.to_string();
    let check_output = git_apply_check(cwd, &effective_patch_path, false);
    match check_output {
        Ok(check) if check.status.success() => {}
        Ok(check) => {
            let reverse_check = git_apply_check(cwd, patch_path, true);
            let (applied, status) = if reverse_check
                .as_ref()
                .is_ok_and(|reverse| reverse.status.success())
            {
                (true, "already_applied")
            } else {
                (false, "check_failed")
            };
            if applied {
                return EvolutionLiveApplyReport {
                    applied,
                    status: status.to_string(),
                    command: Some(format!(
                        "git apply --check --recount --unidiff-zero {}",
                        shell_arg(patch_path)
                    )),
                    patch_path: Some(patch_path.to_string()),
                    changed_paths: changed_paths.clone(),
                    target_boundary_violations: Vec::new(),
                    stdout: String::from_utf8_lossy(&check.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&check.stderr).to_string(),
                };
            }
            match apply_structured_field_pack_patch(cwd, plan, patch_path, &changed_paths) {
                Ok(Some(report)) => return report,
                Ok(None) => {}
                Err(error) => {
                    return EvolutionLiveApplyReport {
                        applied: false,
                        status: "structured_field_pack_apply_failed".to_string(),
                        command: Some(format!(
                            "field-pack structured apply {}",
                            shell_arg(patch_path)
                        )),
                        patch_path: Some(patch_path.to_string()),
                        changed_paths: changed_paths.clone(),
                        target_boundary_violations: Vec::new(),
                        stdout: String::new(),
                        stderr: error,
                    };
                }
            }
            match relocate_patch_hunks(cwd, patch_path) {
                Ok(Some(relocated_path)) => {
                    let relocated_path = relocated_path.to_string_lossy().to_string();
                    match git_apply_check(cwd, &relocated_path, false) {
                        Ok(relocated_check) if relocated_check.status.success() => {
                            effective_patch_path = relocated_path;
                        }
                        Ok(relocated_check) => {
                            return EvolutionLiveApplyReport {
                                applied: false,
                                status: "check_failed".to_string(),
                                command: Some(format!(
                                    "git apply --check --recount --unidiff-zero {}",
                                    shell_arg(&relocated_path)
                                )),
                                patch_path: Some(relocated_path),
                                changed_paths: changed_paths.clone(),
                                target_boundary_violations: Vec::new(),
                                stdout: join_stdout(
                                    &String::from_utf8_lossy(&check.stdout),
                                    &String::from_utf8_lossy(&relocated_check.stdout),
                                ),
                                stderr: join_stderr(
                                    &String::from_utf8_lossy(&check.stderr),
                                    &format!(
                                        "relocated patch check failed: {}",
                                        String::from_utf8_lossy(&relocated_check.stderr)
                                    ),
                                ),
                            };
                        }
                        Err(error) => {
                            return EvolutionLiveApplyReport {
                                applied: false,
                                status: "check_failed_to_start".to_string(),
                                command: Some(format!(
                                    "git apply --check --recount --unidiff-zero {}",
                                    shell_arg(&relocated_path)
                                )),
                                patch_path: Some(relocated_path),
                                changed_paths: changed_paths.clone(),
                                target_boundary_violations: Vec::new(),
                                stdout: String::from_utf8_lossy(&check.stdout).to_string(),
                                stderr: join_stderr(
                                    &String::from_utf8_lossy(&check.stderr),
                                    &format!("relocated patch check failed to start: {error}"),
                                ),
                            };
                        }
                    }
                }
                Ok(None) => {
                    return EvolutionLiveApplyReport {
                        applied: false,
                        status: "check_failed".to_string(),
                        command: Some(format!(
                            "git apply --check --recount --unidiff-zero {}",
                            shell_arg(patch_path)
                        )),
                        patch_path: Some(patch_path.to_string()),
                        changed_paths: changed_paths.clone(),
                        target_boundary_violations: Vec::new(),
                        stdout: String::from_utf8_lossy(&check.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&check.stderr).to_string(),
                    };
                }
                Err(error) => {
                    return EvolutionLiveApplyReport {
                        applied: false,
                        status: "check_failed".to_string(),
                        command: Some(format!(
                            "git apply --check --recount --unidiff-zero {}",
                            shell_arg(patch_path)
                        )),
                        patch_path: Some(patch_path.to_string()),
                        changed_paths: changed_paths.clone(),
                        target_boundary_violations: Vec::new(),
                        stdout: String::from_utf8_lossy(&check.stdout).to_string(),
                        stderr: join_stderr(
                            &String::from_utf8_lossy(&check.stderr),
                            &format!("relocated patch unavailable: {error}"),
                        ),
                    };
                }
            }
        }
        Err(error) => {
            return EvolutionLiveApplyReport {
                applied: false,
                status: "check_failed_to_start".to_string(),
                command: Some(format!(
                    "git apply --check --recount --unidiff-zero {}",
                    shell_arg(patch_path)
                )),
                patch_path: Some(patch_path.to_string()),
                changed_paths: changed_paths.clone(),
                target_boundary_violations: Vec::new(),
                stdout: String::new(),
                stderr: error.to_string(),
            };
        }
    }
    let command_text = format!(
        "git apply --recount --unidiff-zero {}",
        shell_arg(&effective_patch_path)
    );
    if let Err(error) = preflight_field_pack_patch(cwd, plan, &effective_patch_path, &changed_paths)
    {
        return EvolutionLiveApplyReport {
            applied: false,
            status: "pre_apply_validation_failed".to_string(),
            command: Some(command_text),
            patch_path: Some(effective_patch_path),
            changed_paths,
            target_boundary_violations: Vec::new(),
            stdout: String::new(),
            stderr: format!("field-pack dry-run validation failed: {error}"),
        };
    }
    let output = Command::new("git")
        .arg("apply")
        .arg("--recount")
        .arg("--unidiff-zero")
        .arg(&effective_patch_path)
        .current_dir(cwd)
        .output();
    match output {
        Ok(output) => {
            let mut applied = output.status.success();
            let mut status = if applied {
                "applied".to_string()
            } else {
                "failed".to_string()
            };
            let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let mut stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if applied {
                if let Err(error) = validate_field_pack_targets(cwd, plan) {
                    let reverse = Command::new("git")
                        .arg("apply")
                        .arg("--reverse")
                        .arg("--recount")
                        .arg("--unidiff-zero")
                        .arg(&effective_patch_path)
                        .current_dir(cwd)
                        .output();
                    applied = false;
                    match reverse {
                        Ok(reverse) if reverse.status.success() => {
                            status = "post_apply_validation_failed".to_string();
                            stderr = join_stderr(
                                &stderr,
                                &format!("field-pack schema validation failed: {error}; patch was reversed"),
                            );
                            stdout =
                                join_stdout(&stdout, &String::from_utf8_lossy(&reverse.stdout));
                        }
                        Ok(reverse) => {
                            status = "post_apply_validation_failed_reverse_failed".to_string();
                            stderr = join_stderr(
                                &stderr,
                                &format!(
                                    "field-pack schema validation failed: {error}; reverse failed: {}",
                                    String::from_utf8_lossy(&reverse.stderr)
                                ),
                            );
                            stdout =
                                join_stdout(&stdout, &String::from_utf8_lossy(&reverse.stdout));
                        }
                        Err(reverse_error) => {
                            status = "post_apply_validation_failed_reverse_unavailable".to_string();
                            stderr = join_stderr(
                                &stderr,
                                &format!(
                                    "field-pack schema validation failed: {error}; reverse unavailable: {reverse_error}",
                                ),
                            );
                        }
                    }
                }
            }
            let validation_failed = status.starts_with("post_apply_validation_failed");
            if !applied && !validation_failed {
                let reverse_check = Command::new("git")
                    .arg("apply")
                    .arg("--reverse")
                    .arg("--check")
                    .arg("--recount")
                    .arg(&effective_patch_path)
                    .current_dir(cwd)
                    .output();
                if reverse_check
                    .as_ref()
                    .is_ok_and(|reverse| reverse.status.success())
                {
                    applied = true;
                    status = "already_applied".to_string();
                }
            }
            EvolutionLiveApplyReport {
                applied,
                status,
                command: Some(command_text),
                patch_path: Some(effective_patch_path),
                changed_paths,
                target_boundary_violations: Vec::new(),
                stdout,
                stderr,
            }
        }
        Err(error) => EvolutionLiveApplyReport {
            applied: false,
            status: "failed_to_start".to_string(),
            command: Some(command_text),
            patch_path: Some(effective_patch_path),
            changed_paths,
            target_boundary_violations: Vec::new(),
            stdout: String::new(),
            stderr: error.to_string(),
        },
    }
}

fn git_apply_check(
    cwd: &Path,
    patch_path: &str,
    reverse: bool,
) -> std::io::Result<std::process::Output> {
    let mut command = Command::new("git");
    command.arg("apply");
    if reverse {
        command.arg("--reverse");
    }
    command
        .arg("--check")
        .arg("--recount")
        .arg("--unidiff-zero")
        .arg(patch_path)
        .current_dir(cwd)
        .output()
}

fn apply_structured_field_pack_patch(
    cwd: &Path,
    plan: &EvolutionApplyPlan,
    patch_path: &str,
    changed_paths: &[String],
) -> Result<Option<EvolutionLiveApplyReport>, String> {
    let pack_paths = field_pack_targets(plan)
        .into_iter()
        .filter(|path| changed_paths.iter().any(|changed| changed == path))
        .collect::<Vec<_>>();
    if pack_paths.is_empty() {
        return Ok(None);
    }
    let patch = fs::read_to_string(patch_path).map_err(|error| error.to_string())?;
    let template_paths = changed_paths
        .iter()
        .filter(|path| path.contains("/repair-templates/") && path.ends_with(".pyfrag"))
        .cloned()
        .collect::<Vec<_>>();
    let mut backups = BTreeMap::new();
    let mut created = Vec::new();
    let result = (|| -> Result<(), String> {
        for pack_path in &pack_paths {
            let added_tasks = added_field_pack_tasks(&patch, pack_path)?;
            if added_tasks.is_empty() {
                continue;
            }
            let full_path = cwd.join(pack_path);
            let original = fs::read_to_string(&full_path)
                .map_err(|error| format!("{}: {error}", full_path.display()))?;
            backups.insert(pack_path.clone(), Some(original.clone()));
            let mut value =
                serde_json::from_str::<serde_json::Value>(&original).map_err(|error| {
                    format!("{pack_path}: invalid JSON before structured apply: {error}")
                })?;
            let tasks = value
                .get_mut("mini_tasks")
                .and_then(serde_json::Value::as_array_mut)
                .ok_or_else(|| format!("{pack_path}: missing mini_tasks array"))?;
            let mut existing_ids = tasks
                .iter()
                .filter_map(|task| task.get("id").and_then(serde_json::Value::as_str))
                .map(str::to_string)
                .collect::<BTreeSet<_>>();
            let mut new_tasks = Vec::new();
            for task in added_tasks {
                let Some(id) = task.get("id").and_then(serde_json::Value::as_str) else {
                    continue;
                };
                if !existing_ids.contains(id) {
                    existing_ids.insert(id.to_string());
                    new_tasks.push(task);
                }
            }
            if new_tasks.is_empty() {
                continue;
            }
            let content = append_field_pack_tasks_text(&original, &new_tasks)
                .map_err(|error| format!("{pack_path}: {error}"))?;
            fs::write(&full_path, content)
                .map_err(|error| format!("{}: {error}", full_path.display()))?;
        }
        for template_path in &template_paths {
            let Some(content) = added_file_content(&patch, template_path) else {
                continue;
            };
            let full_path = cwd.join(template_path);
            if full_path.exists() {
                backups.insert(
                    template_path.clone(),
                    Some(
                        fs::read_to_string(&full_path)
                            .map_err(|error| format!("{}: {error}", full_path.display()))?,
                    ),
                );
            } else {
                backups.insert(template_path.clone(), None);
                created.push(template_path.clone());
            }
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("{}: {error}", parent.display()))?;
            }
            fs::write(&full_path, content)
                .map_err(|error| format!("{}: {error}", full_path.display()))?;
        }
        validate_field_pack_targets(cwd, plan)?;
        validate_field_pack_template_pairs(cwd, plan, changed_paths)?;
        Ok(())
    })();
    if let Err(error) = result {
        for (path, content) in backups.iter().rev() {
            let full_path = cwd.join(path);
            if let Some(content) = content {
                let _ = fs::write(&full_path, content);
            } else {
                let _ = fs::remove_file(&full_path);
            }
        }
        for path in created {
            let _ = fs::remove_file(cwd.join(path));
        }
        return Err(error);
    }
    Ok(Some(EvolutionLiveApplyReport {
        applied: true,
        status: "applied_structured_field_pack".to_string(),
        command: Some(format!(
            "field-pack structured apply {}",
            shell_arg(patch_path)
        )),
        patch_path: Some(patch_path.to_string()),
        changed_paths: changed_paths.to_vec(),
        target_boundary_violations: Vec::new(),
        stdout: "applied field-pack mini_tasks and repair templates from provider patch"
            .to_string(),
        stderr: String::new(),
    }))
}

fn added_field_pack_tasks(patch: &str, pack_path: &str) -> Result<Vec<serde_json::Value>, String> {
    let added = added_lines_for_path(patch, pack_path).join("\n");
    let mut tasks = Vec::new();
    for object in json_objects_in_text(&added) {
        let value = serde_json::from_str::<serde_json::Value>(&object)
            .map_err(|error| format!("{pack_path}: invalid added task JSON: {error}"))?;
        let is_task = value
            .get("id")
            .and_then(serde_json::Value::as_str)
            .is_some()
            && value
                .get("goal")
                .and_then(serde_json::Value::as_str)
                .is_some()
            && value
                .get("expected_feed")
                .and_then(serde_json::Value::as_str)
                .is_some();
        if is_task {
            tasks.push(value);
        }
    }
    Ok(tasks)
}

fn append_field_pack_tasks_text(
    original: &str,
    tasks: &[serde_json::Value],
) -> Result<String, String> {
    let rendered = tasks
        .iter()
        .map(render_field_pack_task)
        .collect::<Result<Vec<_>, _>>()?
        .join(",\n");
    if original.contains("\"mini_tasks\": []") {
        return Ok(original.replace(
            "\"mini_tasks\": []",
            &format!("\"mini_tasks\": [\n{rendered}\n  ]"),
        ));
    }
    let Some(index) = original.rfind("\n  ]") else {
        return Err("cannot locate mini_tasks closing bracket".to_string());
    };
    let mut output = String::new();
    output.push_str(&original[..index]);
    output.push_str(",\n");
    output.push_str(&rendered);
    output.push_str(&original[index..]);
    if !output.ends_with('\n') {
        output.push('\n');
    }
    Ok(output)
}

fn render_field_pack_task(task: &serde_json::Value) -> Result<String, String> {
    let id = task
        .get("id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "added task missing id".to_string())?;
    let goal = task
        .get("goal")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "added task missing goal".to_string())?;
    let expected_feed = task
        .get("expected_feed")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "added task missing expected_feed".to_string())?;
    Ok(format!(
        "    {{\n      \"id\": {},\n      \"goal\": {},\n      \"expected_feed\": {}\n    }}",
        serde_json::to_string(id).map_err(|error| error.to_string())?,
        serde_json::to_string(goal).map_err(|error| error.to_string())?,
        serde_json::to_string(expected_feed).map_err(|error| error.to_string())?
    ))
}

fn added_file_content(patch: &str, target_path: &str) -> Option<String> {
    let lines = added_lines_for_path(patch, target_path);
    (!lines.is_empty()).then(|| format!("{}\n", lines.join("\n")))
}

fn added_lines_for_path(patch: &str, target_path: &str) -> Vec<String> {
    let mut current_path: Option<String> = None;
    let mut in_hunk = false;
    let mut lines = Vec::new();
    for line in patch.lines() {
        if let Some(path) = diff_file_path(line).or_else(|| plus_file_path(line)) {
            current_path = Some(path);
            in_hunk = false;
            continue;
        }
        if line.starts_with("@@ ") {
            in_hunk = true;
            continue;
        }
        if line.starts_with("diff --git ") {
            in_hunk = false;
            continue;
        }
        if !in_hunk || current_path.as_deref() != Some(target_path) {
            continue;
        }
        if line.starts_with("+++") {
            continue;
        }
        if let Some(value) = line.strip_prefix('+') {
            lines.push(value.to_string());
        }
    }
    lines
}

fn json_objects_in_text(text: &str) -> Vec<String> {
    let mut objects = Vec::new();
    let mut start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start = Some(index);
                }
                depth += 1;
            }
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    if let Some(start) = start.take() {
                        objects.push(text[start..index + ch.len_utf8()].to_string());
                    }
                }
            }
            _ => {}
        }
    }
    objects
}

fn relocate_patch_hunks(cwd: &Path, patch_path: &str) -> Result<Option<PathBuf>, String> {
    let patch = fs::read_to_string(patch_path).map_err(|error| error.to_string())?;
    let Some(relocated) = relocate_patch_hunks_text(cwd, &patch)? else {
        return Ok(None);
    };
    let relocated_path = PathBuf::from(format!("{patch_path}.relocated.patch"));
    fs::write(&relocated_path, relocated).map_err(|error| error.to_string())?;
    Ok(Some(relocated_path))
}

fn relocate_patch_hunks_text(cwd: &Path, patch: &str) -> Result<Option<String>, String> {
    let lines = patch.lines().map(str::to_string).collect::<Vec<_>>();
    let mut relocated = lines.clone();
    let mut current_path: Option<String> = None;
    let mut index = 0usize;
    let mut changed = false;
    while index < lines.len() {
        let line = &lines[index];
        if let Some(path) = diff_file_path(line) {
            current_path = Some(path);
            index += 1;
            continue;
        }
        if let Some(path) = plus_file_path(line) {
            current_path = Some(path);
            index += 1;
            continue;
        }
        if !line.starts_with("@@ ") {
            index += 1;
            continue;
        }
        let Some(path) = current_path.as_deref() else {
            return Err("hunk without target file".to_string());
        };
        let hunk_start = index;
        index += 1;
        let mut old_lines = Vec::new();
        let mut old_count = 0usize;
        let mut new_count = 0usize;
        while index < lines.len()
            && !lines[index].starts_with("@@ ")
            && !lines[index].starts_with("diff --git ")
        {
            let hunk_line = &lines[index];
            if hunk_line.starts_with("\\ No newline") {
                index += 1;
                continue;
            }
            if let Some(value) = hunk_line.strip_prefix('-') {
                old_lines.push(value.to_string());
                old_count += 1;
            } else if hunk_line.starts_with('+') {
                new_count += 1;
            } else if let Some(value) = hunk_line.strip_prefix(' ') {
                old_lines.push(value.to_string());
                old_count += 1;
                new_count += 1;
            }
            index += 1;
        }
        if old_lines.is_empty() {
            continue;
        }
        let target = cwd.join(path);
        let target_text = fs::read_to_string(&target)
            .map_err(|error| format!("{}: {error}", target.display()))?;
        let target_lines = target_text.lines().map(str::to_string).collect::<Vec<_>>();
        let Some(start) = find_line_sequence(&target_lines, &old_lines) else {
            continue;
        };
        let new_start = start + 1;
        let replacement = format!("@@ -{new_start},{old_count} +{new_start},{new_count} @@");
        if relocated[hunk_start] != replacement {
            relocated[hunk_start] = replacement;
            changed = true;
        }
    }
    Ok(changed.then(|| format!("{}\n", relocated.join("\n"))))
}

fn diff_file_path(line: &str) -> Option<String> {
    let value = line.strip_prefix("diff --git ")?;
    value
        .split_whitespace()
        .nth(1)
        .and_then(normalize_patch_path)
}

fn plus_file_path(line: &str) -> Option<String> {
    let value = line.strip_prefix("+++ ")?;
    normalize_patch_path(value)
}

fn normalize_patch_path(value: &str) -> Option<String> {
    let value = value.trim().trim_matches('"');
    if value == "/dev/null" {
        return None;
    }
    value
        .strip_prefix("b/")
        .or_else(|| value.strip_prefix("a/"))
        .map(str::to_string)
}

fn find_line_sequence(haystack: &[String], needle: &[String]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn validate_field_pack_targets(cwd: &Path, plan: &EvolutionApplyPlan) -> Result<(), String> {
    for target in field_pack_targets(plan) {
        let path = cwd.join(&target);
        let text = fs::read_to_string(&path).map_err(|error| format!("{target}: {error}"))?;
        let value = serde_json::from_str::<serde_json::Value>(&text)
            .map_err(|error| format!("{target}: invalid JSON: {error}"))?;
        validate_field_pack_value(&target, &value)?;
    }
    Ok(())
}

fn preflight_field_pack_patch(
    cwd: &Path,
    plan: &EvolutionApplyPlan,
    patch_path: &str,
    changed_paths: &[String],
) -> Result<(), String> {
    let targets = field_pack_targets(plan);
    if targets.is_empty() {
        return Ok(());
    }
    let temp_dir = env_temp_dir("octopus-field-pack-preflight");
    let patch_path = if Path::new(patch_path).is_absolute() {
        PathBuf::from(patch_path)
    } else {
        cwd.join(patch_path)
    };
    let result = (|| {
        let _ = fs::remove_dir_all(&temp_dir);
        let input_paths = dry_run_input_paths(changed_paths, &targets);
        copy_patch_inputs_to_temp(cwd, &temp_dir, &input_paths)?;
        init_dry_run_git_repo(&temp_dir)?;
        let output = Command::new("git")
            .arg("apply")
            .arg("--recount")
            .arg("--unidiff-zero")
            .arg(&patch_path)
            .current_dir(&temp_dir)
            .output()
            .map_err(|error| format!("dry-run apply failed to start: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "dry-run apply failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        validate_field_pack_targets(&temp_dir, plan)?;
        validate_field_pack_template_pairs(&temp_dir, plan, changed_paths)?;
        Ok(())
    })();
    let _ = fs::remove_dir_all(&temp_dir);
    result
}

fn dry_run_input_paths(changed_paths: &[String], field_pack_targets: &[String]) -> Vec<String> {
    let mut paths = changed_paths.to_vec();
    for target in field_pack_targets {
        if !paths.iter().any(|path| path == target) {
            paths.push(target.clone());
        }
    }
    paths
}

fn init_dry_run_git_repo(temp_dir: &Path) -> Result<(), String> {
    let output = Command::new("git")
        .arg("init")
        .arg("-q")
        .current_dir(temp_dir)
        .output()
        .map_err(|error| format!("dry-run git init failed to start: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "dry-run git init failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn copy_patch_inputs_to_temp(
    cwd: &Path,
    temp_dir: &Path,
    changed_paths: &[String],
) -> Result<(), String> {
    fs::create_dir_all(temp_dir)
        .map_err(|error| format!("create dry-run directory {}: {error}", temp_dir.display()))?;
    for target in changed_paths {
        let source = cwd.join(target);
        let destination = temp_dir.join(target);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("create dry-run parent {}: {error}", parent.display()))?;
        }
        if source.is_file() {
            fs::copy(&source, &destination).map_err(|error| {
                format!(
                    "copy dry-run input {} to {}: {error}",
                    source.display(),
                    destination.display()
                )
            })?;
        }
    }
    Ok(())
}

fn validate_field_pack_template_pairs(
    cwd: &Path,
    plan: &EvolutionApplyPlan,
    changed_paths: &[String],
) -> Result<(), String> {
    for (field, task_id) in repair_template_field_task_ids(plan, changed_paths) {
        let target = format!("field-packs/{field}/field-pack.json");
        let path = cwd.join(&target);
        let text = fs::read_to_string(&path).map_err(|error| format!("{target}: {error}"))?;
        let value = serde_json::from_str::<serde_json::Value>(&text)
            .map_err(|error| format!("{target}: invalid JSON: {error}"))?;
        let Some(tasks) = value.get("mini_tasks").and_then(|value| value.as_array()) else {
            return Err(format!("{target}: missing mini_tasks array"));
        };
        let found = tasks.iter().any(|task| {
            task.get("id")
                .and_then(|value| value.as_str())
                .is_some_and(|id| id == task_id)
        });
        if !found {
            return Err(format!(
                "{target}: repair template {task_id}.pyfrag has no matching mini_tasks[].id"
            ));
        }
    }
    Ok(())
}

fn repair_template_field_task_ids(
    plan: &EvolutionApplyPlan,
    changed_paths: &[String],
) -> Vec<(String, String)> {
    let mut ids = BTreeSet::new();
    for path in changed_paths.iter().chain(plan.target_files.iter()) {
        if let Some(pair) = repair_template_field_task_id(path, &plan.tentacle_id) {
            ids.insert(pair);
        }
    }
    ids.into_iter().collect()
}

fn repair_template_field_task_id(path: &str, tentacle_id: &str) -> Option<(String, String)> {
    let path = path.replace('\\', "/");
    let manifest_prefix = format!("tentacles/{tentacle_id}/repair-templates/");
    let suffix = path
        .strip_prefix(&manifest_prefix)
        .or_else(|| path.strip_prefix("repair-templates/"))?;
    let mut parts = suffix.split('/');
    let field = parts.next()?.to_string();
    let file = parts.next()?;
    if parts.next().is_some() || !file.ends_with(".pyfrag") {
        return None;
    }
    let task_id = file.trim_end_matches(".pyfrag").to_string();
    Some((field, task_id))
}

fn env_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}-{counter}", std::process::id()))
}

fn field_pack_targets(plan: &EvolutionApplyPlan) -> Vec<String> {
    let mut targets = BTreeSet::new();
    for path in &plan.target_files {
        let path = path.replace('\\', "/");
        if path.starts_with("field-packs/") && path.ends_with("/field-pack.json") {
            targets.insert(path);
        }
    }
    let target = plan.target.split('#').next().unwrap_or(&plan.target);
    let target = target.replace('\\', "/");
    if target.starts_with("field-packs/") && target.ends_with("/field-pack.json") {
        targets.insert(target);
    }
    targets.into_iter().collect()
}

fn validate_field_pack_value(target: &str, value: &serde_json::Value) -> Result<(), String> {
    let Some(tasks) = value.get("mini_tasks") else {
        return Err(format!("{target}: missing mini_tasks array"));
    };
    let Some(tasks) = tasks.as_array() else {
        return Err(format!("{target}: mini_tasks must be an array"));
    };
    for (index, task) in tasks.iter().enumerate() {
        let Some(task) = task.as_object() else {
            return Err(format!("{target}: mini_tasks[{index}] must be an object"));
        };
        for key in ["id", "goal", "expected_feed"] {
            if !task.get(key).is_some_and(|value| {
                value
                    .as_str()
                    .map(|text| !text.trim().is_empty())
                    .unwrap_or(false)
            }) {
                return Err(format!(
                    "{target}: mini_tasks[{index}] missing non-empty {key}",
                ));
            }
        }
    }
    Ok(())
}

fn join_stderr(existing: &str, detail: &str) -> String {
    if existing.trim().is_empty() {
        detail.to_string()
    } else {
        format!("{}\n{}", existing.trim_end(), detail)
    }
}

fn join_stdout(existing: &str, detail: &str) -> String {
    if existing.trim().is_empty() {
        detail.to_string()
    } else if detail.trim().is_empty() {
        existing.to_string()
    } else {
        format!("{}\n{}", existing.trim_end(), detail)
    }
}

pub(crate) fn live_apply_summary(
    recommendation: &EvolutionRecommendation,
    live_apply: &EvolutionLiveApplyReport,
) -> String {
    live_apply_candidate_summary(&recommendation.candidate_id, live_apply)
}

pub(crate) fn live_apply_candidate_summary(
    candidate_id: &str,
    live_apply: &EvolutionLiveApplyReport,
) -> String {
    if live_apply.applied || live_apply.stderr.trim().is_empty() {
        return format!("{} {}", candidate_id, live_apply.status);
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
    format!("{} {}: {}", candidate_id, live_apply.status, detail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_skips_when_plan_is_not_authorized() {
        let mut plan = test_plan();
        plan.authorized = false;
        let artifact = test_artifact(Some("candidate.patch"));

        let report = apply_authorized_suggested_patch(Path::new("."), &plan, &artifact);

        assert!(!report.applied);
        assert_eq!(report.status, "skipped_not_authorized");
        assert!(report.command.is_none());
        assert_eq!(report.patch_path.as_deref(), Some("candidate.patch"));
    }

    #[test]
    fn apply_skips_when_provider_did_not_return_patch() {
        let mut plan = test_plan();
        plan.suggested_patch = None;
        let artifact = test_artifact(Some("candidate.patch"));

        let report = apply_authorized_suggested_patch(Path::new("."), &plan, &artifact);

        assert!(!report.applied);
        assert_eq!(report.status, "skipped_no_suggested_patch");
        assert!(report.command.is_none());
    }

    #[test]
    fn live_apply_summary_compacts_stderr() {
        let recommendation = test_recommendation();
        let report = EvolutionLiveApplyReport {
            applied: false,
            status: "check_failed".to_string(),
            command: None,
            patch_path: Some("candidate.patch".to_string()),
            changed_paths: Vec::new(),
            target_boundary_violations: Vec::new(),
            stdout: String::new(),
            stderr: "error: patch failed at current file context".to_string(),
        };

        let summary = live_apply_summary(&recommendation, &report);

        assert!(summary.contains("01-runtime-code check_failed"));
        assert!(summary.contains("patch failed"));
    }

    #[test]
    fn apply_blocks_patch_outside_target_files() {
        let dir = std::env::temp_dir().join(format!(
            "octopus-evolution-apply-boundary-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("tentacles/field-mini-task/tools")).unwrap();
        run_git(&dir, &["init"]);
        let allowed_path = dir.join("tentacles/field-mini-task/tools/run_field_mini_task.sh");
        fs::write(&allowed_path, "echo allowed\n").unwrap();
        let readme_path = dir.join("README.md");
        fs::write(&readme_path, "old\n").unwrap();
        let patch_path = dir.join("candidate.patch");
        fs::write(
            &patch_path,
            "diff --git a/README.md b/README.md\n--- a/README.md\n+++ b/README.md\n@@ -1,1 +1,1 @@\n-old\n+new\n",
        )
        .unwrap();
        let mut plan = test_plan();
        plan.suggested_patch = Some(fs::read_to_string(&patch_path).unwrap());
        let artifact = test_artifact(Some(patch_path.to_str().unwrap()));

        let report = apply_authorized_suggested_patch(&dir, &plan, &artifact);

        assert!(!report.applied);
        assert_eq!(report.status, "target_boundary_failed");
        assert_eq!(report.changed_paths, vec!["README.md".to_string()]);
        assert_eq!(
            report.target_boundary_violations,
            vec!["README.md".to_string()]
        );
        assert_eq!(fs::read_to_string(readme_path).unwrap(), "old\n");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn apply_blocks_field_pack_patch_that_breaks_json_before_live_apply() {
        let dir = std::env::temp_dir().join(format!(
            "octopus-evolution-apply-field-pack-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("field-packs/translate")).unwrap();
        run_git(&dir, &["init"]);
        let pack_path = dir.join("field-packs/translate/field-pack.json");
        let original = "{\n  \"id\": \"translate\",\n  \"mini_tasks\": []\n}\n";
        fs::write(&pack_path, original).unwrap();
        let patch_path = dir.join("candidate.patch");
        fs::write(
            &patch_path,
            "diff --git a/field-packs/translate/field-pack.json b/field-packs/translate/field-pack.json\n--- a/field-packs/translate/field-pack.json\n+++ b/field-packs/translate/field-pack.json\n@@ -2,0 +3,1 @@\n+  ,{\"id\":\"bad\",\"goal\":\"bad\",\"expected_feed\":\"bad\"}\n",
        )
        .unwrap();
        let mut plan = test_plan();
        plan.candidate_id = "04-field-pack-tasks".to_string();
        plan.target = "field-packs/translate/field-pack.json".to_string();
        plan.target_files = vec!["field-packs/translate/field-pack.json".to_string()];
        plan.suggested_patch = Some(fs::read_to_string(&patch_path).unwrap());
        let artifact = test_artifact(Some(patch_path.to_str().unwrap()));

        let report = apply_authorized_suggested_patch(&dir, &plan, &artifact);

        assert!(!report.applied);
        assert_eq!(report.status, "pre_apply_validation_failed");
        assert!(report.stderr.contains("invalid JSON"));
        assert_eq!(fs::read_to_string(&pack_path).unwrap(), original);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn apply_blocks_field_pack_template_id_without_matching_task() {
        let dir = std::env::temp_dir().join(format!(
            "octopus-evolution-apply-field-pack-pair-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("field-packs/translate")).unwrap();
        fs::create_dir_all(dir.join("tentacles/field-mini-task/repair-templates/translate"))
            .unwrap();
        run_git(&dir, &["init"]);
        let pack_path = dir.join("field-packs/translate/field-pack.json");
        let original = "{\n  \"id\": \"translate\",\n  \"mini_tasks\": []\n}\n";
        fs::write(&pack_path, original).unwrap();
        let template_path = dir
            .join("tentacles/field-mini-task/repair-templates/translate/translate-mini-8.pyfrag");
        let patch_path = dir.join("candidate.patch");
        fs::write(
            &patch_path,
            "diff --git a/field-packs/translate/field-pack.json b/field-packs/translate/field-pack.json\n--- a/field-packs/translate/field-pack.json\n+++ b/field-packs/translate/field-pack.json\n@@ -1,4 +1,10 @@\n {\n   \"id\": \"translate\",\n-  \"mini_tasks\": []\n+  \"mini_tasks\": [\n+    {\n+      \"id\": \"translate-mini-7\",\n+      \"goal\": \"g\",\n+      \"expected_feed\": \"f\"\n+    }\n+  ]\n }\ndiff --git a/tentacles/field-mini-task/repair-templates/translate/translate-mini-8.pyfrag b/tentacles/field-mini-task/repair-templates/translate/translate-mini-8.pyfrag\nnew file mode 100644\n--- /dev/null\n+++ b/tentacles/field-mini-task/repair-templates/translate/translate-mini-8.pyfrag\n@@ -0,0 +1,2 @@\n+if field == \"translate\" and mini_task == \"translate-mini-8\":\n+    pass\n",
        )
        .unwrap();
        let mut plan = test_plan();
        plan.candidate_id = "04-field-pack-tasks".to_string();
        plan.target = "field-packs/translate/field-pack.json".to_string();
        plan.target_files = vec![
            "field-packs/translate/field-pack.json".to_string(),
            "tentacles/field-mini-task/repair-templates/translate/translate-mini-8.pyfrag"
                .to_string(),
        ];
        plan.suggested_patch = Some(fs::read_to_string(&patch_path).unwrap());
        let artifact = test_artifact(Some(patch_path.to_str().unwrap()));

        let report = apply_authorized_suggested_patch(&dir, &plan, &artifact);

        assert!(!report.applied);
        assert_eq!(report.status, "pre_apply_validation_failed");
        assert!(
            report.stderr.contains("no matching mini_tasks[].id"),
            "{report:?}"
        );
        assert_eq!(fs::read_to_string(&pack_path).unwrap(), original);
        assert!(!template_path.exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn apply_allows_template_only_repair_when_field_pack_task_exists() {
        let dir = std::env::temp_dir().join(format!(
            "octopus-evolution-apply-template-only-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("field-packs/search")).unwrap();
        fs::create_dir_all(dir.join("tentacles/field-mini-task/repair-templates/search")).unwrap();
        run_git(&dir, &["init"]);
        fs::write(
            dir.join("field-packs/search/field-pack.json"),
            "{\n  \"id\": \"search\",\n  \"mini_tasks\": [\n    {\n      \"id\": \"search-mini-7\",\n      \"goal\": \"g\",\n      \"expected_feed\": \"f\"\n    }\n  ]\n}\n",
        )
        .unwrap();
        let template_path =
            dir.join("tentacles/field-mini-task/repair-templates/search/search-mini-7.pyfrag");
        fs::write(&template_path, "evidence.update({})\n").unwrap();
        let patch_path = dir.join("candidate.patch");
        fs::write(
            &patch_path,
            "diff --git a/tentacles/field-mini-task/repair-templates/search/search-mini-7.pyfrag b/tentacles/field-mini-task/repair-templates/search/search-mini-7.pyfrag\n--- a/tentacles/field-mini-task/repair-templates/search/search-mini-7.pyfrag\n+++ b/tentacles/field-mini-task/repair-templates/search/search-mini-7.pyfrag\n@@ -1,1 +1,2 @@\n+evidence = {}\n evidence.update({})\n",
        )
        .unwrap();
        let mut plan = test_plan();
        plan.candidate_id = "04-field-pack-tasks".to_string();
        plan.target =
            "tentacles/field-mini-task/repair-templates/search/search-mini-7.pyfrag".to_string();
        plan.target_files = vec![
            "field-packs/search/field-pack.json".to_string(),
            "tentacles/field-mini-task/repair-templates/search/search-mini-7.pyfrag".to_string(),
        ];
        plan.suggested_patch = Some(fs::read_to_string(&patch_path).unwrap());
        let artifact = test_artifact(Some(patch_path.to_str().unwrap()));

        let report = apply_authorized_suggested_patch(&dir, &plan, &artifact);

        assert!(report.applied, "{report:?}");
        assert_eq!(report.status, "applied");
        assert_eq!(
            fs::read_to_string(template_path).unwrap(),
            "evidence = {}\nevidence.update({})\n"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn apply_structures_field_pack_append_when_patch_context_is_stale() {
        let dir = std::env::temp_dir().join(format!(
            "octopus-evolution-apply-structured-field-pack-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("field-packs/code")).unwrap();
        fs::create_dir_all(dir.join("tentacles/field-mini-task/repair-templates/code")).unwrap();
        run_git(&dir, &["init"]);
        let pack_path = dir.join("field-packs/code/field-pack.json");
        fs::write(
            &pack_path,
            "{\n  \"id\": \"code\",\n  \"mini_tasks\": [\n    {\n      \"id\": \"code-mini-5\",\n      \"goal\": \"g\",\n      \"expected_feed\": \"current feed with verifier_status\"\n    }\n  ]\n}\n",
        )
        .unwrap();
        let template_path =
            dir.join("tentacles/field-mini-task/repair-templates/code/code-mini-7.pyfrag");
        let patch_path = dir.join("candidate.patch");
        fs::write(
            &patch_path,
            "diff --git a/field-packs/code/field-pack.json b/field-packs/code/field-pack.json\n--- a/field-packs/code/field-pack.json\n+++ b/field-packs/code/field-pack.json\n@@ -4,7 +4,12 @@\n     {\n       \"id\": \"code-mini-5\",\n       \"goal\": \"g\",\n       \"expected_feed\": \"stale feed without current words\"\n-    }\n+    },\n+    {\n+      \"id\": \"code-mini-7\",\n+      \"goal\": \"harder\",\n+      \"expected_feed\": \"artifact-backed\"\n+    }\n   ]\n }\ndiff --git a/tentacles/field-mini-task/repair-templates/code/code-mini-7.pyfrag b/tentacles/field-mini-task/repair-templates/code/code-mini-7.pyfrag\nnew file mode 100644\n--- /dev/null\n+++ b/tentacles/field-mini-task/repair-templates/code/code-mini-7.pyfrag\n@@ -0,0 +1,2 @@\n+if field == 'code' and mini_task == 'code-mini-7':\n+    field_result = {'status': 'satisfied'}\n",
        )
        .unwrap();
        let mut plan = test_plan();
        plan.candidate_id = "04-field-pack-tasks".to_string();
        plan.target = "field-packs/code/field-pack.json".to_string();
        plan.target_files = vec![
            "field-packs/code/field-pack.json".to_string(),
            "tentacles/field-mini-task/repair-templates/code/code-mini-7.pyfrag".to_string(),
        ];
        plan.suggested_patch = Some(fs::read_to_string(&patch_path).unwrap());
        let artifact = test_artifact(Some(patch_path.to_str().unwrap()));

        let report = apply_authorized_suggested_patch(&dir, &plan, &artifact);

        assert!(report.applied, "{report:?}");
        assert_eq!(report.status, "applied_structured_field_pack");
        let pack = fs::read_to_string(pack_path).unwrap();
        assert!(pack.contains("\"id\": \"code-mini-7\""));
        assert!(pack.contains("current feed with verifier_status"));
        assert!(fs::read_to_string(template_path)
            .unwrap()
            .contains("code-mini-7"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn apply_relocates_bad_line_number_hunk_by_current_file_content() {
        let dir = std::env::temp_dir().join(format!(
            "octopus-evolution-apply-relocate-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("tentacles/field-mini-task/repair-templates/ib")).unwrap();
        run_git(&dir, &["init"]);
        let template_path =
            dir.join("tentacles/field-mini-task/repair-templates/ib/ib-mini-4.pyfrag");
        fs::write(
            &template_path,
            "if field == \"ib\" and mini_task == \"ib-mini-4\":\n    assumptions = [\n        \"No guidance language such as buy/sell/target is allowed in the result summary or memo.\"\n    ]\n",
        )
        .unwrap();
        let patch_path = dir.join("candidate.patch");
        fs::write(
            &patch_path,
            "diff --git a/tentacles/field-mini-task/repair-templates/ib/ib-mini-4.pyfrag b/tentacles/field-mini-task/repair-templates/ib/ib-mini-4.pyfrag\n--- a/tentacles/field-mini-task/repair-templates/ib/ib-mini-4.pyfrag\n+++ b/tentacles/field-mini-task/repair-templates/ib/ib-mini-4.pyfrag\n@@ -1,1 +1,1 @@\n-        \"No guidance language such as buy/sell/target is allowed in the result summary or memo.\"\n+        \"No directional trading or price-target language is allowed in the result summary or memo.\"\n",
        )
        .unwrap();
        let relocated = relocate_patch_hunks_text(&dir, &fs::read_to_string(&patch_path).unwrap())
            .unwrap()
            .unwrap();
        assert!(relocated.contains("@@ -3,1 +3,1 @@"));
        let mut plan = test_plan();
        plan.target = "tentacles/field-mini-task/repair-templates/ib/ib-mini-4.pyfrag".to_string();
        plan.target_files =
            vec!["tentacles/field-mini-task/repair-templates/ib/ib-mini-4.pyfrag".to_string()];
        plan.suggested_patch = Some(fs::read_to_string(&patch_path).unwrap());
        let artifact = test_artifact(Some(patch_path.to_str().unwrap()));

        let report = apply_authorized_suggested_patch(&dir, &plan, &artifact);

        assert!(report.applied, "{report:?}");
        assert_eq!(report.status, "applied");
        let changed = fs::read_to_string(template_path).unwrap();
        assert!(changed.contains("No directional trading or price-target language"));
        assert!(!changed.contains("buy/sell/target"));

        let _ = fs::remove_dir_all(dir);
    }

    fn test_plan() -> EvolutionApplyPlan {
        EvolutionApplyPlan {
            tentacle_id: "field-mini-task".to_string(),
            candidate_id: "01-runtime-code".to_string(),
            objective: "repair harness".to_string(),
            authorized: true,
            status: "authorized".to_string(),
            required_grant: "octopus:evolve:field-mini-task".to_string(),
            active_grant: Some("octopus:evolve:field-mini-task".to_string()),
            target: "tentacles/field-mini-task/tools/run_field_mini_task.sh".to_string(),
            target_files: vec!["tentacles/field-mini-task/tools/run_field_mini_task.sh".to_string()],
            draft_path: "patches/01-runtime-code.patch".to_string(),
            checks: vec!["octopus check field-mini-task".to_string()],
            feedback: Vec::new(),
            suggested_patch: Some(
                "diff --git a/a b/a\n--- a/a\n+++ b/a\n@@ -1 +1 @@\n-a\n+b\n".to_string(),
            ),
            guardrails: vec!["declared target only".to_string()],
            next_steps: vec!["run manifest checks".to_string()],
        }
    }

    fn test_artifact(patch_path: Option<&str>) -> EvolutionApplyArtifact {
        EvolutionApplyArtifact {
            directory: ".octopus/evolution/field-mini-task".to_string(),
            plan_path: ".octopus/evolution/field-mini-task/APPLY_PLAN.md".to_string(),
            patch_path: patch_path.map(str::to_string),
            json_path: ".octopus/evolution/field-mini-task/apply.json".to_string(),
        }
    }

    fn test_recommendation() -> EvolutionRecommendation {
        EvolutionRecommendation {
            tentacle_id: "field-mini-task".to_string(),
            objective: "repair harness".to_string(),
            candidate_id: "01-runtime-code".to_string(),
            candidate_title: "Runtime patch".to_string(),
            surface_id: "runtime_code".to_string(),
            outcome_count: 0,
            feed_trace_count: 0,
            check_history_count: 0,
            recommendation_score: 1.0,
            reason: "test".to_string(),
            apply: test_plan(),
        }
    }

    fn run_git(cwd: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
