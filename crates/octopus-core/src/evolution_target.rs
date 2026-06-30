use crate::{
    default_field_pack_aliases, default_field_pack_ids, default_field_pack_root,
    generic_evolution_scope_alias, push_unique_limited, text_contains_field_term,
    unicode_lowercase, EvolutionFileTarget, EvolutionSurface,
};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn evolution_file_target(
    manifest_dir: &Path,
    target: &str,
    objective: &str,
) -> EvolutionFileTarget {
    if let Some(path) = field_pack_evolution_target(target) {
        return EvolutionFileTarget {
            target_files: field_pack_evolution_targets_for_objective(target, objective),
            path,
            action: "extend editable peer-field task definitions".to_string(),
            rationale: format!("support {objective}"),
        };
    }
    match target {
        "brain.prompt" => EvolutionFileTarget {
            path: "manifest.json#brain.prompt".to_string(),
            target_files: vec![manifest_dir
                .join("manifest.json")
                .to_string_lossy()
                .to_string()],
            action: "tighten the tentacle brain prompt".to_string(),
            rationale: format!("align tool-side planning with {objective}"),
        },
        "manifest.json" => EvolutionFileTarget {
            path: manifest_dir
                .join("manifest.json")
                .to_string_lossy()
                .to_string(),
            target_files: vec![manifest_dir
                .join("manifest.json")
                .to_string_lossy()
                .to_string()],
            action: "review skills, tools, feedback contract, and evolution policy".to_string(),
            rationale: "keep prompt, metadata, code, checks, and constraints consistent"
                .to_string(),
        },
        value if value.contains('*') => {
            let path = manifest_dir.join(value).to_string_lossy().to_string();
            let scoped_repair_templates =
                repair_template_evolution_targets_for_objective(manifest_dir, value, objective);
            EvolutionFileTarget {
                target_files: if scoped_repair_templates.is_empty() {
                    evolution_target_files(&path)
                } else {
                    scoped_repair_templates
                },
                path,
                action: "inspect matching harness code before editing".to_string(),
                rationale: "wildcards require a narrow patch target".to_string(),
            }
        }
        value => {
            let path = manifest_dir.join(value).to_string_lossy().to_string();
            EvolutionFileTarget {
                target_files: evolution_target_files(&path),
                path,
                action: "prepare a scoped harness edit".to_string(),
                rationale: format!("support {objective}"),
            }
        }
    }
}

pub(crate) fn evolution_target_files(target: &str) -> Vec<String> {
    let path = target.split('#').next().unwrap_or(target).trim();
    if path.is_empty() {
        return Vec::new();
    }
    let field_pack_targets = field_pack_evolution_targets(path);
    if !field_pack_targets.is_empty() {
        return field_pack_targets;
    }
    if path.contains('*') {
        return resolve_wildcard_targets(path)
            .into_iter()
            .filter(|path| path.exists())
            .map(|path| path.to_string_lossy().to_string())
            .collect();
    }
    let path = PathBuf::from(path);
    path.exists()
        .then(|| vec![path.to_string_lossy().to_string()])
        .unwrap_or_default()
}

pub(crate) fn evolution_candidate_target_files(
    manifest_dir: &Path,
    surface: &EvolutionSurface,
    target: &str,
    objective: &str,
) -> Vec<String> {
    let mut files = if surface.id == "field_pack_tasks" {
        let scoped = field_pack_evolution_targets_for_objective(target, objective);
        if scoped.is_empty() {
            evolution_target_files(target)
        } else {
            scoped
        }
    } else {
        evolution_target_files(target)
    };
    if surface.id != "field_pack_tasks" {
        return files;
    }
    for declared_target in &surface.targets {
        let resolved = evolution_file_target(manifest_dir, declared_target, objective);
        for file in resolved.target_files {
            push_unique_limited(&mut files, file, usize::MAX);
        }
        if !field_pack_evolution_targets(&resolved.path).is_empty() {
            continue;
        }
        for file in evolution_target_files(&resolved.path) {
            push_unique_limited(&mut files, file, usize::MAX);
        }
    }
    files
}

pub(crate) fn field_pack_evolution_target(target: &str) -> Option<String> {
    let suffix = field_pack_evolution_suffix(target)?;
    Some(
        field_pack_evolution_root()
            .join(suffix)
            .to_string_lossy()
            .to_string(),
    )
}

pub(crate) fn field_pack_evolution_targets(target: &str) -> Vec<String> {
    field_pack_evolution_targets_with_fields(target, None)
}

fn field_pack_evolution_targets_for_objective(target: &str, objective: &str) -> Vec<String> {
    let fields = fields_mentioned_for_evolution_scope(objective);
    let selected = (!fields.is_empty()).then_some(fields.as_slice());
    field_pack_evolution_targets_with_fields(target, selected)
}

fn field_pack_evolution_targets_with_fields(
    target: &str,
    selected_fields: Option<&[String]>,
) -> Vec<String> {
    let Some(suffix) = field_pack_evolution_suffix(target) else {
        return Vec::new();
    };
    let root = field_pack_evolution_root();
    if suffix == "*/field-pack.json" {
        return selected_fields
            .map(|fields| fields.to_vec())
            .unwrap_or_else(default_field_pack_ids)
            .into_iter()
            .map(|field| root.join(field).join("field-pack.json"))
            .filter(|path| path.exists())
            .map(|path| path.to_string_lossy().to_string())
            .collect();
    }
    if suffix.contains('*') {
        return Vec::new();
    }
    let path = root.join(suffix);
    path.exists()
        .then(|| vec![path.to_string_lossy().to_string()])
        .unwrap_or_default()
}

fn field_pack_evolution_suffix(target: &str) -> Option<String> {
    let target = target
        .split('#')
        .next()
        .unwrap_or(target)
        .trim()
        .replace('\\', "/");
    let suffix = target
        .strip_prefix("field-packs/")
        .map(str::to_string)
        .or_else(|| {
            target
                .find("/field-packs/")
                .map(|index| target[index + "/field-packs/".len()..].to_string())
        })?;
    if suffix.trim().is_empty() || suffix.contains("..") {
        return None;
    }
    Some(suffix)
}

fn repair_template_evolution_targets_for_objective(
    manifest_dir: &Path,
    target: &str,
    objective: &str,
) -> Vec<String> {
    let normalized = target.trim().replace('\\', "/");
    if normalized != "repair-templates/*/*.pyfrag" {
        return Vec::new();
    }
    let fields = fields_mentioned_for_evolution_scope(objective);
    if fields.is_empty() {
        return Vec::new();
    }
    let mut targets = Vec::new();
    for field in fields {
        let dir = manifest_dir.join("repair-templates").join(field);
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        let mut files = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "pyfrag"))
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        files.sort();
        for file in files {
            push_unique_limited(&mut targets, file, usize::MAX);
        }
    }
    targets
}

fn field_pack_evolution_root() -> PathBuf {
    let root = default_field_pack_root();
    if root.join("index.json").exists() {
        return root;
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("field-packs")
}

fn fields_mentioned_for_evolution_scope(value: &str) -> Vec<String> {
    let explicit = fields_mentioned_in_field_paths(value);
    if !explicit.is_empty() {
        return explicit;
    }
    let text = unicode_lowercase(value);
    let mut fields = Vec::new();
    let limit = default_field_pack_ids().len().max(1);
    for (field, aliases) in default_field_pack_aliases() {
        if aliases
            .iter()
            .chain(std::iter::once(&field))
            .filter(|alias| evolution_scope_alias_is_specific(&field, alias))
            .map(|alias| unicode_lowercase(alias))
            .any(|alias| text_contains_field_term(&text, &alias))
        {
            push_unique_limited(&mut fields, field, limit);
        }
    }
    fields
}

pub(crate) fn fields_mentioned_in_field_paths(value: &str) -> Vec<String> {
    let text = unicode_lowercase(&value.replace('\\', "/"));
    let mut fields = Vec::new();
    let limit = default_field_pack_ids().len().max(1);
    for field in default_field_pack_ids() {
        let field = unicode_lowercase(&field);
        for marker in [
            format!("field-packs/{field}/"),
            format!("field-packs/{field}/field-pack.json"),
            format!("repair-templates/{field}/"),
        ] {
            if text.contains(&marker) {
                push_unique_limited(&mut fields, field.clone(), limit);
                break;
            }
        }
    }
    fields
}

fn evolution_scope_alias_is_specific(field: &str, alias: &str) -> bool {
    let alias = alias.trim();
    if alias.is_empty() {
        return false;
    }
    let alias_lower = unicode_lowercase(alias);
    alias_lower == unicode_lowercase(field)
        || alias.chars().any(|ch| !ch.is_ascii())
        || alias.contains('-')
        || alias.split_whitespace().count() > 1 && !generic_evolution_scope_alias(&alias_lower)
}

pub(crate) fn resolve_wildcard_targets(value: &str) -> Vec<PathBuf> {
    let path = Path::new(value);
    let Some(parent) = path.parent() else {
        return Vec::new();
    };
    let Some(pattern) = path.file_name().map(|name| name.to_string_lossy()) else {
        return Vec::new();
    };
    let Some((prefix, suffix)) = pattern.split_once('*') else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(parent) else {
        return Vec::new();
    };
    let mut matches = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy())
                .is_some_and(|name| name.starts_with(prefix) && name.ends_with(suffix))
        })
        .collect::<Vec<_>>();
    matches.sort();
    matches
}
