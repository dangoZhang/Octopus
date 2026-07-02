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

pub fn unauthorized_diff_paths_for_plan(plan: &EvolutionApplyPlan, patch: &str) -> Vec<String> {
    let Some(patch) = normalize_suggested_patch_text(patch) else {
        return Vec::new();
    };
    let patch = rewrite_tentacle_relative_patch_paths(plan, &patch);
    let paths = diff_paths(&patch);
    let allowed_paths = allowed_patch_paths(plan);
    let allowed_template_fields = fields_mentioned_in_field_paths(&plan.target_files.join(" "));
    paths
        .into_iter()
        .filter(|path| {
            !allowed_paths.iter().any(|allowed| allowed == path)
                && !allowed_field_task_harness_path(
                    path,
                    &plan.tentacle_id,
                    &allowed_template_fields,
                )
        })
        .collect()
}

pub(crate) fn clean_suggested_patch(patch: Option<String>) -> Option<String> {
    normalize_suggested_patch_text(patch?.as_str())
}

fn normalize_suggested_patch_text(patch: &str) -> Option<String> {
    let patch = patch.trim();
    (!patch.is_empty()).then(|| {
        let patch = strip_patch_wrappers(patch);
        let patch = normalize_patch_file_headers(&patch);
        let patch = normalize_prefixed_patch_control_lines(&patch);
        let patch = normalize_missing_existing_file_headers(&patch);
        let patch = normalize_new_file_hunks(&patch);
        let patch = normalize_existing_file_hunk_context(&patch);
        let patch = normalize_diff_boundaries(&patch);
        format!("{}\n", recount_hunk_headers(&patch).trim())
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

fn normalize_prefixed_patch_control_lines(patch: &str) -> String {
    let mut normalized = Vec::new();
    let mut before_hunk = false;
    for line in patch.lines() {
        if line.starts_with("diff --git ") {
            before_hunk = true;
            normalized.push(line.to_string());
            continue;
        }
        if before_hunk {
            if let Some(stripped) = line.strip_prefix("+new file mode ") {
                normalized.push(format!("new file mode {stripped}"));
                continue;
            }
            if let Some(stripped) = line.strip_prefix("+--- /dev/null") {
                normalized.push(format!("--- /dev/null{stripped}"));
                continue;
            }
            if let Some(stripped) = line.strip_prefix("++++ ") {
                normalized.push(format!("+++ {stripped}"));
                continue;
            }
            if let Some(stripped) = line.strip_prefix("+@@ ") {
                normalized.push(format!("@@ {stripped}"));
                before_hunk = false;
                continue;
            }
            if line.starts_with("@@ ") {
                before_hunk = false;
            }
        }
        normalized.push(line.to_string());
    }
    normalized.join("\n")
}

fn normalize_missing_existing_file_headers(patch: &str) -> String {
    let lines = patch.lines().map(str::to_string).collect::<Vec<_>>();
    let mut normalized = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = &lines[index];
        if let Some((old_path, new_path)) = parse_diff_git_paths(line) {
            normalized.push(line.to_string());
            let mut lookahead = index + 1;
            while lookahead < lines.len() && lines[lookahead].trim().is_empty() {
                normalized.push(lines[lookahead].clone());
                lookahead += 1;
            }
            if lookahead < lines.len() && lines[lookahead].starts_with("@@ ") {
                normalized.push(format!("--- {old_path}"));
                normalized.push(format!("+++ {new_path}"));
            }
            index = lookahead;
            continue;
        }
        normalized.push(line.to_string());
        index += 1;
    }
    normalized.join("\n")
}

fn parse_diff_git_paths(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix("diff --git ")?;
    let mut parts = rest.split_whitespace();
    let old_path = parts.next()?.trim_matches('"');
    let new_path = parts.next()?.trim_matches('"');
    (old_path.starts_with("a/") && new_path.starts_with("b/"))
        .then(|| (old_path.to_string(), new_path.to_string()))
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
        let trimmed_line = line.trim_start();
        if trimmed_line.starts_with("new file mode ") {
            if current_file_has_new_mode {
                continue;
            }
            current_file_has_new_mode = true;
            normalized.push(trimmed_line.to_string());
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

fn normalize_existing_file_hunk_context(patch: &str) -> String {
    let mut normalized = Vec::new();
    let mut pending_new_file = false;
    let mut current_file_is_new = false;
    let mut in_hunk = false;
    for line in patch.lines() {
        if line.starts_with("diff --git ") {
            pending_new_file = false;
            current_file_is_new = false;
            in_hunk = false;
            normalized.push(line.to_string());
            continue;
        }
        if line == "--- /dev/null" {
            pending_new_file = true;
            current_file_is_new = false;
            in_hunk = false;
            normalized.push(line.to_string());
            continue;
        }
        if line.starts_with("--- ") {
            pending_new_file = false;
            current_file_is_new = false;
            in_hunk = false;
            normalized.push(line.to_string());
            continue;
        }
        if line.starts_with("+++ ") {
            current_file_is_new = pending_new_file;
            in_hunk = false;
            normalized.push(line.to_string());
            continue;
        }
        if line.starts_with("@@ ") {
            in_hunk = true;
            normalized.push(line.to_string());
            continue;
        }
        if in_hunk && !current_file_is_new {
            if line.starts_with("\\ No newline")
                || line.starts_with('+')
                || line.starts_with('-')
                || line.starts_with(' ')
            {
                normalized.push(line.to_string());
            } else {
                normalized.push(format!(" {line}"));
            }
            continue;
        }
        normalized.push(line.to_string());
    }
    normalized.join("\n")
}

fn normalize_diff_boundaries(patch: &str) -> String {
    let mut normalized: Vec<String> = Vec::new();
    for line in patch.lines() {
        if line.trim_start().starts_with("diff --git ") {
            while normalized
                .last()
                .is_some_and(|previous| previous.trim().is_empty())
            {
                normalized.pop();
            }
            normalized.push(line.trim_start().to_string());
        } else {
            normalized.push(line.to_string());
        }
    }
    normalized.join("\n")
}

fn recount_hunk_headers(patch: &str) -> String {
    let mut lines = patch.lines().map(str::to_string).collect::<Vec<_>>();
    let mut index = 0;
    while index < lines.len() {
        if !lines[index].starts_with("@@ ") {
            index += 1;
            continue;
        }
        let Some(header) = ParsedHunkHeader::parse(&lines[index]) else {
            index += 1;
            continue;
        };
        let header_index = index;
        index += 1;
        let mut old_count = 0usize;
        let mut new_count = 0usize;
        while index < lines.len()
            && !lines[index].starts_with("@@ ")
            && !lines[index].starts_with("diff --git ")
        {
            let line = &lines[index];
            if line.starts_with("\\ No newline") {
                index += 1;
                continue;
            }
            if line.starts_with('-') {
                old_count += 1;
            } else if line.starts_with('+') {
                new_count += 1;
            } else if line.starts_with(' ') {
                old_count += 1;
                new_count += 1;
            }
            index += 1;
        }
        lines[header_index] = header.with_counts(old_count, new_count);
    }
    lines.join("\n")
}

struct ParsedHunkHeader {
    old_start: String,
    new_start: String,
    tail: String,
}

impl ParsedHunkHeader {
    fn parse(line: &str) -> Option<Self> {
        let rest = line.strip_prefix("@@")?;
        let close = rest.find("@@")?;
        let inside = rest[..close].trim();
        let tail = rest[close + 2..].to_string();
        let mut old_start = None;
        let mut new_start = None;
        for token in inside.split_whitespace() {
            if let Some(range) = token.strip_prefix('-') {
                old_start = Some(range_start(range));
            } else if let Some(range) = token.strip_prefix('+') {
                new_start = Some(range_start(range));
            }
        }
        Some(Self {
            old_start: old_start?,
            new_start: new_start?,
            tail,
        })
    }

    fn with_counts(&self, old_count: usize, new_count: usize) -> String {
        format!(
            "@@ -{},{} +{},{} @@{}",
            self.old_start, old_count, self.new_start, new_count, self.tail
        )
    }
}

fn range_start(range: &str) -> String {
    range
        .split_once(',')
        .map(|(start, _)| start)
        .unwrap_or(range)
        .to_string()
}

fn provider_patch_for_plan(plan: &EvolutionApplyPlan, patch: &str) -> Option<String> {
    let patch = normalize_suggested_patch_text(patch)?;
    let patch = rewrite_tentacle_relative_patch_paths(plan, &patch);
    let paths = diff_paths(&patch);
    if paths.is_empty() {
        return None;
    }
    let allowed_paths = allowed_patch_paths(plan);
    let allowed_template_fields = fields_mentioned_in_field_paths(&plan.target_files.join(" "));
    if allowed_paths.is_empty()
        || paths.iter().any(|path| {
            !allowed_paths.iter().any(|allowed| allowed == path)
                && !allowed_field_task_harness_path(
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

fn rewrite_tentacle_relative_patch_paths(plan: &EvolutionApplyPlan, patch: &str) -> String {
    let allowed_paths = allowed_patch_paths(plan);
    if allowed_paths.is_empty() {
        return patch.to_string();
    }
    let mut mapping = Vec::new();
    for path in diff_paths(patch) {
        if allowed_paths.iter().any(|allowed| allowed == &path) {
            continue;
        }
        if path.starts_with("tentacles/")
            || path.starts_with("field-packs/")
            || path.starts_with("docs/")
            || path.starts_with(".octopus/")
            || path.contains("..")
        {
            continue;
        }
        let mut matches = allowed_paths
            .iter()
            .filter(|allowed| {
                allowed == &&format!("tentacles/{}/{}", plan.tentacle_id, path)
                    || allowed.ends_with(&format!("/{path}"))
            })
            .cloned()
            .collect::<Vec<_>>();
        matches.sort();
        matches.dedup();
        if matches.len() == 1 {
            mapping.push((path, matches.remove(0)));
        }
    }
    if mapping.is_empty() {
        return patch.to_string();
    }
    patch
        .lines()
        .map(|line| rewrite_patch_control_line(line, &mapping))
        .collect::<Vec<_>>()
        .join("\n")
}

fn rewrite_patch_control_line(line: &str, mapping: &[(String, String)]) -> String {
    if let Some(rest) = line.strip_prefix("diff --git ") {
        let mut parts = rest
            .split_whitespace()
            .map(str::to_string)
            .collect::<Vec<_>>();
        if parts.len() >= 2 {
            parts[0] = rewrite_patch_path_token(&parts[0], mapping);
            parts[1] = rewrite_patch_path_token(&parts[1], mapping);
            return format!("diff --git {}", parts.join(" "));
        }
    }
    for prefix in ["--- ", "+++ "] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return format!("{prefix}{}", rewrite_patch_path_token(rest, mapping));
        }
    }
    line.to_string()
}

fn rewrite_patch_path_token(token: &str, mapping: &[(String, String)]) -> String {
    let trimmed = token.trim_matches('"');
    if trimmed == "/dev/null" {
        return token.to_string();
    }
    let (prefix, path) = trimmed
        .strip_prefix("a/")
        .map(|path| ("a/", path))
        .or_else(|| trimmed.strip_prefix("b/").map(|path| ("b/", path)))
        .unwrap_or(("", trimmed));
    let rewritten = mapping
        .iter()
        .find_map(|(from, to)| (path == from).then(|| to.as_str()))
        .unwrap_or(path);
    let value = format!("{prefix}{rewritten}");
    if token.starts_with('"') && token.ends_with('"') {
        format!("\"{value}\"")
    } else {
        value
    }
}

fn allowed_patch_paths(plan: &EvolutionApplyPlan) -> Vec<String> {
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
    allowed_paths
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

fn allowed_field_task_harness_path(path: &str, tentacle_id: &str, fields: &[String]) -> bool {
    if fields.is_empty() || path.contains("..") {
        return false;
    }
    fields.iter().any(|field| {
        allowed_field_go_worker_path(path, tentacle_id, field)
            || allowed_legacy_field_template_path(path, tentacle_id, field)
    })
}

fn allowed_field_go_worker_path(path: &str, tentacle_id: &str, field: &str) -> bool {
    [
        format!("tentacles/{tentacle_id}/workers/{field}/"),
        format!("workers/{field}/"),
    ]
    .iter()
    .filter_map(|prefix| path.strip_prefix(prefix))
    .any(|rest| {
        let mut parts = rest.split('/');
        let Some(task_id) = parts.next() else {
            return false;
        };
        !task_id.is_empty() && parts.next() == Some("main.go") && parts.next().is_none()
    })
}

fn allowed_legacy_field_template_path(path: &str, tentacle_id: &str, field: &str) -> bool {
    if !path.ends_with(".pyfrag") {
        return false;
    }
    [
        format!("tentacles/{tentacle_id}/repair-templates/{field}/"),
        format!("repair-templates/{field}/"),
    ]
    .iter()
    .any(|prefix| {
        path.strip_prefix(prefix)
            .is_some_and(|rest| !rest.is_empty() && !rest.contains('/'))
    })
}

pub fn diff_paths(patch: &str) -> Vec<String> {
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
    use std::{fs, process::Command};

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

    #[test]
    fn clean_suggested_patch_recounts_short_hunks_and_removes_diff_separator_blank() {
        let patch = r#"diff --git a/field-packs/computer-use/field-pack.json b/field-packs/computer-use/field-pack.json
--- a/field-packs/computer-use/field-pack.json
+++ b/field-packs/computer-use/field-pack.json
@@ -7,2 +7,9 @@
-    }
-  ]
+    },
+    {
+      "id": "computer-use-mini-4",
+      "goal": "Validate placement.",
+      "expected_feed": "Artifact-backed Feed evidence."
+    }
+  ]

diff --git a/tentacles/field-mini-task/repair-templates/computer-use/computer-use-mini-4.pyfrag b/tentacles/field-mini-task/repair-templates/computer-use/computer-use-mini-4.pyfrag
new file mode 100644
--- /dev/null
+++ b/tentacles/field-mini-task/repair-templates/computer-use/computer-use-mini-4.pyfrag
@@ -0,0 +1,5 @@
+if field == "computer-use" and mini_task == "computer-use-mini-4":
+    field_result = {
+        "verifier_status": "satisfied",
+    }
"#;

        let cleaned = clean_suggested_patch(Some(patch.to_string())).unwrap();

        assert!(cleaned.contains("@@ -7,2 +7,7 @@"));
        assert!(cleaned.contains("@@ -0,0 +1,4 @@"));
        assert!(!cleaned.contains("]\n\ndiff --git"));

        let dir = std::env::temp_dir().join(format!(
            "octopus-evolution-patch-recount-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("field-packs/computer-use")).unwrap();
        fs::create_dir_all(dir.join("tentacles/field-mini-task/repair-templates/computer-use"))
            .unwrap();
        run_git(&dir, &["init"]);
        fs::write(
            dir.join("field-packs/computer-use/field-pack.json"),
            "{\n  \"id\": \"computer-use\",\n  \"mini_tasks\": [\n    {\n      \"id\": \"computer-use-mini-3\",\n      \"goal\": \"Observe.\",\n      \"expected_feed\": \"Verify.\"\n    }\n  ]\n}\n",
        )
        .unwrap();
        let patch_path = dir.join("candidate.patch");
        fs::write(&patch_path, cleaned).unwrap();

        run_git(
            &dir,
            &[
                "apply",
                "--check",
                "--recount",
                "--unidiff-zero",
                patch_path.to_str().unwrap(),
            ],
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn clean_suggested_patch_adds_missing_existing_file_headers() {
        let patch = r#"diff --git a/field-packs/math/field-pack.json b/field-packs/math/field-pack.json
@@ -1,0 +2,1 @@
+{}
diff --git a/tentacles/field-mini-task/repair-templates/math/math-mini-6.pyfrag b/tentacles/field-mini-task/repair-templates/math/math-mini-6.pyfrag
new file mode 100644
--- /dev/null
+++ b/tentacles/field-mini-task/repair-templates/math/math-mini-6.pyfrag
@@ -0,0 +1,1 @@
+if field == "math":
+    pass
"#;

        let cleaned = clean_suggested_patch(Some(patch.to_string())).unwrap();

        assert!(cleaned.contains("--- a/field-packs/math/field-pack.json"));
        assert!(cleaned.contains("+++ b/field-packs/math/field-pack.json"));
        assert_eq!(
            diff_paths(&cleaned),
            vec![
                "field-packs/math/field-pack.json",
                "tentacles/field-mini-task/repair-templates/math/math-mini-6.pyfrag"
            ]
        );
    }

    #[test]
    fn clean_suggested_patch_strips_plus_prefixed_new_file_headers() {
        let patch = r#"diff --git a/tentacles/field-mini-task/repair-templates/write/write-mini-6.pyfrag b/tentacles/field-mini-task/repair-templates/write/write-mini-6.pyfrag
+new file mode 100644
+--- /dev/null
++++ b/tentacles/field-mini-task/repair-templates/write/write-mini-6.pyfrag
+@@ -0,0 +1,2 @@
+if field == "write":
+    pass
"#;

        let cleaned = clean_suggested_patch(Some(patch.to_string())).unwrap();

        assert!(cleaned.contains("\nnew file mode 100644\n"));
        assert!(cleaned.contains("\n--- /dev/null\n"));
        assert!(cleaned.contains(
            "\n+++ b/tentacles/field-mini-task/repair-templates/write/write-mini-6.pyfrag\n"
        ));
        assert!(cleaned.contains("\n@@ -0,0 +1,2 @@\n"));
        assert_eq!(
            diff_paths(&cleaned),
            vec!["tentacles/field-mini-task/repair-templates/write/write-mini-6.pyfrag"]
        );
    }

    #[test]
    fn clean_suggested_patch_deduplicates_indented_new_file_mode() {
        let patch = r#"diff --git a/tentacles/field-mini-task/workers/swe/swe-go-default-smoke/fallback_worker.py b/tentacles/field-mini-task/workers/swe/swe-go-default-smoke/fallback_worker.py
 new file mode 100644
new file mode 100644
--- /dev/null
+++ b/tentacles/field-mini-task/workers/swe/swe-go-default-smoke/fallback_worker.py
@@ -0,0 +1 @@
+print("ok")
"#;

        let cleaned = clean_suggested_patch(Some(patch.to_string())).unwrap();

        assert_eq!(cleaned.matches("new file mode 100644").count(), 1);
        assert!(cleaned.contains("\nnew file mode 100644\n--- /dev/null\n"));
    }

    #[test]
    fn clean_suggested_patch_prefixes_bare_existing_hunk_context() {
        let patch = r#"diff --git a/a.py b/a.py
--- a/a.py
+++ b/a.py
@@ -1,2 +1,3 @@
def rel(path, root):
+    fallback()
    return path
"#;

        let cleaned = clean_suggested_patch(Some(patch.to_string())).unwrap();

        assert!(cleaned.contains("\n def rel(path, root):\n"));
        assert!(cleaned.contains("\n+    fallback()\n"));
        assert!(cleaned.contains("\n    return path\n"));

        let dir = std::env::temp_dir().join(format!(
            "octopus-evolution-patch-existing-context-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        run_git(&dir, &["init"]);
        fs::write(dir.join("a.py"), "def rel(path, root):\n   return path\n").unwrap();
        let patch_path = dir.join("candidate.patch");
        fs::write(&patch_path, cleaned).unwrap();
        run_git(
            &dir,
            &[
                "apply",
                "--check",
                "--recount",
                "--unidiff-zero",
                patch_path.to_str().unwrap(),
            ],
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn authorized_apply_patch_rewrites_tentacle_relative_target_path() {
        let patch = r#"diff --git a/tools/run_field_mini_task.sh b/tools/run_field_mini_task.sh
--- a/tools/run_field_mini_task.sh
+++ b/tools/run_field_mini_task.sh
@@ -1 +1 @@
-old
+new
"#;
        let plan = test_apply_plan(
            "tentacles/field-mini-task/tools/run_field_mini_task.sh",
            patch,
        );

        let rewritten = authorized_apply_patch(&plan).unwrap();

        assert_eq!(
            diff_paths(&rewritten),
            vec!["tentacles/field-mini-task/tools/run_field_mini_task.sh"]
        );
        assert!(rewritten.contains(
            "diff --git a/tentacles/field-mini-task/tools/run_field_mini_task.sh b/tentacles/field-mini-task/tools/run_field_mini_task.sh"
        ));
        assert!(unauthorized_diff_paths_for_plan(&plan, patch).is_empty());
    }

    #[test]
    fn authorized_apply_patch_keeps_ambiguous_relative_target_blocked() {
        let patch = r#"diff --git a/main.go b/main.go
--- a/main.go
+++ b/main.go
@@ -1 +1 @@
-old
+new
"#;
        let mut plan = test_apply_plan("tentacles/field-mini-task/workers/swe/a/main.go", patch);
        plan.target_files
            .push("tentacles/field-mini-task/workers/code/b/main.go".to_string());

        assert!(authorized_apply_patch(&plan).is_none());
        assert_eq!(
            unauthorized_diff_paths_for_plan(&plan, patch),
            vec!["main.go"]
        );
    }

    fn test_apply_plan(target: &str, suggested_patch: &str) -> EvolutionApplyPlan {
        EvolutionApplyPlan {
            tentacle_id: "field-mini-task".to_string(),
            candidate_id: "03-runtime-code".to_string(),
            objective: "repair field mini task harness".to_string(),
            authorized: true,
            status: "ready_for_authorized_patch".to_string(),
            required_grant: "octopus:evolve:field-mini-task".to_string(),
            active_grant: Some("octopus:evolve:field-mini-task".to_string()),
            target: target.to_string(),
            target_files: vec![target.to_string()],
            draft_path: "patches/03-runtime-code.patch.md".to_string(),
            checks: vec![],
            feedback: vec![],
            suggested_patch: Some(suggested_patch.to_string()),
            guardrails: vec![],
            next_steps: vec![],
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
