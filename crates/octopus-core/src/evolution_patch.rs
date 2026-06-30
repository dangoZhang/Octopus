use crate::{
    evolution_target::{
        field_pack_evolution_targets, fields_mentioned_in_field_paths, resolve_wildcard_targets,
    },
    EvolutionApplyPlan,
};
use std::path::{Path, PathBuf};

pub(crate) fn authorized_apply_patch(plan: &EvolutionApplyPlan) -> Option<String> {
    plan.suggested_patch
        .as_deref()
        .and_then(|patch| provider_patch_for_plan(plan, patch))
}

pub(crate) fn clean_suggested_patch(patch: Option<String>) -> Option<String> {
    normalize_suggested_patch_text(patch?.as_str())
}

fn normalize_suggested_patch_text(patch: &str) -> Option<String> {
    let patch = patch.trim();
    (!patch.is_empty()).then(|| {
        let patch = strip_patch_wrappers(patch);
        let patch = normalize_patch_file_headers(&patch);
        format!("{}\n", normalize_new_file_hunks(&patch).trim())
    })
}

fn strip_patch_wrappers(patch: &str) -> String {
    patch
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if matches!(
                trimmed,
                "*** Begin Patch" | "*** End Patch" | "```" | "```diff" | "```patch"
            ) {
                None
            } else {
                Some(line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_patch_file_headers(patch: &str) -> String {
    patch
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("diff --git ") {
                line.trim_start()
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_new_file_hunks(patch: &str) -> String {
    let mut normalized = Vec::new();
    let mut pending_new_file = false;
    let mut current_file_is_new = false;
    let mut current_file_has_new_mode = false;
    let mut in_new_file_hunk = false;
    for line in patch.lines() {
        let line = if line.starts_with("+diff --git a/") && line.contains(" b/") {
            &line[1..]
        } else {
            line
        };
        if line.starts_with("diff --git ") {
            pending_new_file = false;
            current_file_is_new = false;
            current_file_has_new_mode = false;
            in_new_file_hunk = false;
            normalized.push(line.to_string());
            continue;
        }
        if line.starts_with("new file mode ") {
            current_file_has_new_mode = true;
            normalized.push(line.to_string());
            continue;
        }
        if line == "--- /dev/null" {
            pending_new_file = true;
            current_file_is_new = false;
            in_new_file_hunk = false;
            if !current_file_has_new_mode {
                normalized.push("new file mode 100644".to_string());
                current_file_has_new_mode = true;
            }
            normalized.push(line.to_string());
            continue;
        }
        if line.starts_with("+++ ") {
            current_file_is_new = pending_new_file;
            in_new_file_hunk = false;
            normalized.push(line.to_string());
            continue;
        }
        if line.starts_with("@@ ") {
            in_new_file_hunk = current_file_is_new;
            normalized.push(line.to_string());
            continue;
        }
        if in_new_file_hunk {
            if line.starts_with("\\ No newline") || line.starts_with('+') {
                normalized.push(line.to_string());
            } else {
                normalized.push(format!("+{line}"));
            }
            continue;
        }
        normalized.push(line.to_string());
    }
    normalized.join("\n")
}

fn provider_patch_for_plan(plan: &EvolutionApplyPlan, patch: &str) -> Option<String> {
    let patch = normalize_suggested_patch_text(patch)?;
    let paths = diff_paths(&patch);
    if paths.is_empty() {
        return None;
    }
    let mut allowed_paths = plan
        .target_files
        .iter()
        .map(|path| patch_display_path(Path::new(path)))
        .collect::<Vec<_>>();
    if allowed_paths.is_empty() {
        allowed_paths = resolve_existing_patch_targets(&plan.target)
            .iter()
            .map(|path| patch_display_path(path))
            .collect::<Vec<_>>();
    }
    if !plan.target.contains('*') {
        let target_path = plan.target.split('#').next().unwrap_or(&plan.target);
        if !target_path.trim().is_empty() {
            push_unique(
                &mut allowed_paths,
                patch_display_path(Path::new(target_path)),
            );
        }
    }
    let allowed_template_fields = fields_mentioned_in_field_paths(&plan.target_files.join(" "));
    if allowed_paths.is_empty()
        || paths.iter().any(|path| {
            !allowed_paths.iter().any(|allowed| allowed == path)
                && !allowed_field_repair_template_path(
                    path,
                    &plan.tentacle_id,
                    &allowed_template_fields,
                )
        })
    {
        return None;
    }
    Some(patch)
}

fn resolve_existing_patch_targets(target: &str) -> Vec<PathBuf> {
    let path = target.split('#').next().unwrap_or(target);
    let field_pack_targets = field_pack_evolution_targets(path)
        .into_iter()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    if !field_pack_targets.is_empty() {
        return field_pack_targets;
    }
    if path.contains('*') {
        return resolve_wildcard_targets(path);
    }
    let path = PathBuf::from(path);
    path.exists().then(|| vec![path]).unwrap_or_default()
}

fn allowed_field_repair_template_path(path: &str, tentacle_id: &str, fields: &[String]) -> bool {
    if fields.is_empty() || !path.ends_with(".pyfrag") || path.contains("..") {
        return false;
    }
    fields.iter().any(|field| {
        [
            format!("tentacles/{tentacle_id}/repair-templates/{field}/"),
            format!("repair-templates/{field}/"),
        ]
        .iter()
        .any(|prefix| {
            path.strip_prefix(prefix)
                .is_some_and(|rest| !rest.is_empty() && !rest.contains('/'))
        })
    })
}

pub(crate) fn diff_paths(patch: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for line in patch.lines() {
        if let Some(value) = line.strip_prefix("diff --git ") {
            for part in value.split_whitespace().take(2) {
                if let Some(path) = normalize_diff_path(part) {
                    push_unique(&mut paths, path);
                }
            }
        } else if let Some(value) = line.strip_prefix("--- ") {
            if let Some(path) = normalize_diff_path(value) {
                push_unique(&mut paths, path);
            }
        } else if let Some(value) = line.strip_prefix("+++ ") {
            if let Some(path) = normalize_diff_path(value) {
                push_unique(&mut paths, path);
            }
        }
    }
    paths
}

fn normalize_diff_path(value: &str) -> Option<String> {
    let value = value.trim().trim_matches('"');
    if value == "/dev/null" {
        return None;
    }
    value
        .strip_prefix("a/")
        .or_else(|| value.strip_prefix("b/"))
        .map(|path| path.to_string())
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

pub(crate) fn patch_display_path(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    for marker in ["tentacles/", "field-packs/", "docs/", ".octopus/"] {
        if let Some(index) = value.find(marker) {
            return collapse_repeated_tentacle_prefix(&value[index..]);
        }
    }
    collapse_repeated_tentacle_prefix(
        &path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| value),
    )
}

fn collapse_repeated_tentacle_prefix(value: &str) -> String {
    let parts = value.split('/').collect::<Vec<_>>();
    for index in 0..parts.len().saturating_sub(3) {
        if parts[index] == "tentacles"
            && parts.get(index + 2) == Some(&"tentacles")
            && parts.get(index + 1) == parts.get(index + 3)
        {
            let mut collapsed = Vec::new();
            collapsed.extend_from_slice(&parts[..index + 2]);
            collapsed.extend_from_slice(&parts[index + 4..]);
            return collapsed.join("/");
        }
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_suggested_patch_strips_apply_patch_wrappers() {
        let patch = r#"*** Begin Patch
diff --git a/tentacles/field-mini-task/repair-templates/swe/swe-mini-4.pyfrag b/tentacles/field-mini-task/repair-templates/swe/swe-mini-4.pyfrag
--- a/tentacles/field-mini-task/repair-templates/swe/swe-mini-4.pyfrag
+++ b/tentacles/field-mini-task/repair-templates/swe/swe-mini-4.pyfrag
@@ -1 +1 @@
-old
+new
*** End Patch
"#;

        let cleaned = clean_suggested_patch(Some(patch.to_string())).unwrap();

        assert!(!cleaned.contains("*** Begin Patch"));
        assert!(!cleaned.contains("*** End Patch"));
        assert_eq!(
            diff_paths(&cleaned),
            vec!["tentacles/field-mini-task/repair-templates/swe/swe-mini-4.pyfrag"]
        );
    }

    #[test]
    fn clean_suggested_patch_strips_markdown_fence_wrappers() {
        let patch = r#"```diff
diff --git a/field-packs/swe/field-pack.json b/field-packs/swe/field-pack.json
--- a/field-packs/swe/field-pack.json
+++ b/field-packs/swe/field-pack.json
@@ -1 +1 @@
-{}
+{"ok":true}
```
"#;

        let cleaned = clean_suggested_patch(Some(patch.to_string())).unwrap();

        assert!(!cleaned.contains("```"));
        assert_eq!(
            diff_paths(&cleaned),
            vec!["field-packs/swe/field-pack.json"]
        );
    }

    #[test]
    fn clean_suggested_patch_splits_embedded_diff_header() {
        let patch = r#"diff --git a/field-packs/write/field-pack.json b/field-packs/write/field-pack.json
--- a/field-packs/write/field-pack.json
+++ b/field-packs/write/field-pack.json
@@ -1,0 +2,1 @@
+{}
+diff --git a/tentacles/field-mini-task/repair-templates/write/write-mini-4.pyfrag b/tentacles/field-mini-task/repair-templates/write/write-mini-4.pyfrag
new file mode 100644
--- /dev/null
+++ b/tentacles/field-mini-task/repair-templates/write/write-mini-4.pyfrag
@@ -0,0 +1 @@
+if field == "write":
+    pass
"#;

        let cleaned = clean_suggested_patch(Some(patch.to_string())).unwrap();

        assert!(cleaned.contains("\ndiff --git a/tentacles/field-mini-task"));
        assert_eq!(
            diff_paths(&cleaned),
            vec![
                "field-packs/write/field-pack.json",
                "tentacles/field-mini-task/repair-templates/write/write-mini-4.pyfrag"
            ]
        );
    }
}
