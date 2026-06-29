use crate::{Goal, Need, NeedKind};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const OCTOPUS_FIELD_PACKS_ENV: &str = "OCTOPUS_FIELD_PACKS";
const DEFAULT_FIELD_PACK_INDEX: &str = include_str!("../../../field-packs/index.json");
const EMBEDDED_FIELD_PACKS: &[(&str, &str)] = &[
    (
        "math",
        include_str!("../../../field-packs/math/field-pack.json"),
    ),
    (
        "search",
        include_str!("../../../field-packs/search/field-pack.json"),
    ),
    (
        "code",
        include_str!("../../../field-packs/code/field-pack.json"),
    ),
    (
        "swe",
        include_str!("../../../field-packs/swe/field-pack.json"),
    ),
    (
        "research",
        include_str!("../../../field-packs/research/field-pack.json"),
    ),
    (
        "computer-use",
        include_str!("../../../field-packs/computer-use/field-pack.json"),
    ),
    (
        "ib",
        include_str!("../../../field-packs/ib/field-pack.json"),
    ),
    (
        "robotics",
        include_str!("../../../field-packs/robotics/field-pack.json"),
    ),
];

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FieldPackIndex {
    pub version: String,
    pub schema: String,
    pub packs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FieldPack {
    pub id: String,
    pub version: String,
    pub description: String,
    pub task_schema: FieldTaskSchema,
    pub capability_hints: Vec<String>,
    pub permission_boundary: FieldPermissionBoundary,
    pub verifier: FieldVerifier,
    pub trajectory_labels: Vec<String>,
    pub mini_tasks: Vec<FieldMiniTask>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FieldTaskSchema {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    #[serde(default)]
    pub constraints: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FieldPermissionBoundary {
    pub safe: Vec<String>,
    pub requires_grant: Vec<String>,
    pub blocked_by_default: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FieldVerifier {
    pub method: String,
    pub pass_signal: String,
    pub error_categories: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FieldMiniTask {
    pub id: String,
    pub goal: String,
    pub expected_feed: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FieldPackCatalog {
    pub version: String,
    pub source: String,
    pub root: Option<String>,
    pub packs: Vec<FieldPack>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FieldPackSelection {
    pub field: String,
    pub score: f32,
    pub reason: String,
    pub signals: Vec<String>,
    pub verifier_method: String,
    pub pass_signal: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FieldPackSummary {
    pub id: String,
    pub version: String,
    pub description: String,
    pub capability_hints: Vec<String>,
    pub verifier_method: String,
    pub mini_tasks: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FieldPackReport {
    pub source: String,
    pub root: Option<String>,
    pub version: Option<String>,
    pub pack_count: usize,
    pub packs: Vec<FieldPackSummary>,
    pub errors: Vec<String>,
    pub next: Vec<String>,
}

pub fn default_field_pack_root() -> PathBuf {
    env::var_os(OCTOPUS_FIELD_PACKS_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("field-packs"))
}

pub fn default_field_pack_catalog() -> Result<FieldPackCatalog, String> {
    let root = default_field_pack_root();
    if root.join("index.json").exists() {
        return load_field_pack_catalog(&root);
    }
    embedded_field_pack_catalog()
}

pub fn load_field_pack_catalog(root: impl AsRef<Path>) -> Result<FieldPackCatalog, String> {
    let root = root.as_ref();
    let index_path = root.join("index.json");
    let index = parse_index(&fs::read_to_string(&index_path).map_err(|error| {
        format!(
            "field pack index missing or unreadable: {}: {error}",
            index_path.display()
        )
    })?)?;
    let mut packs = Vec::new();
    for id in &index.packs {
        let path = root.join(id).join("field-pack.json");
        let content = fs::read_to_string(&path)
            .map_err(|error| format!("field pack unreadable: {}: {error}", path.display()))?;
        let pack = parse_pack(id, &content)?;
        packs.push(pack);
    }
    Ok(FieldPackCatalog {
        version: index.version,
        source: "filesystem".to_string(),
        root: Some(root.display().to_string()),
        packs,
    })
}

pub fn embedded_field_pack_catalog() -> Result<FieldPackCatalog, String> {
    let index = parse_index(DEFAULT_FIELD_PACK_INDEX)?;
    let mut by_id = BTreeMap::new();
    for (id, content) in EMBEDDED_FIELD_PACKS {
        by_id.insert(*id, *content);
    }
    let mut packs = Vec::new();
    for id in &index.packs {
        let content = by_id
            .get(id.as_str())
            .ok_or_else(|| format!("embedded field pack missing: {id}"))?;
        packs.push(parse_pack(id, content)?);
    }
    Ok(FieldPackCatalog {
        version: index.version,
        source: "embedded".to_string(),
        root: None,
        packs,
    })
}

pub fn field_pack_report(root: Option<&Path>) -> FieldPackReport {
    let loaded = match root {
        Some(root) => load_field_pack_catalog(root),
        None => default_field_pack_catalog(),
    };
    match loaded {
        Ok(catalog) => FieldPackReport {
            source: catalog.source,
            root: catalog.root,
            version: Some(catalog.version),
            pack_count: catalog.packs.len(),
            packs: catalog
                .packs
                .iter()
                .map(|pack| FieldPackSummary {
                    id: pack.id.clone(),
                    version: pack.version.clone(),
                    description: pack.description.clone(),
                    capability_hints: pack.capability_hints.clone(),
                    verifier_method: pack.verifier.method.clone(),
                    mini_tasks: pack.mini_tasks.len(),
                })
                .collect(),
            errors: Vec::new(),
            next: vec![
                "octopus fields match verify <goal>".to_string(),
                "octopus traces 10".to_string(),
            ],
        },
        Err(error) => FieldPackReport {
            source: "unavailable".to_string(),
            root: root
                .map(|root| root.display().to_string())
                .or_else(|| Some(default_field_pack_root().display().to_string())),
            version: None,
            pack_count: 0,
            packs: Vec::new(),
            errors: vec![error],
            next: vec!["check field-packs/index.json".to_string()],
        },
    }
}

pub fn select_field_pack(
    packs: &[FieldPack],
    goal: Option<&Goal>,
    need: &Need,
) -> Option<FieldPackSelection> {
    if let Some(field) = explicit_field_signal(goal, need) {
        if let Some(pack) = packs.iter().find(|pack| pack.id == field) {
            return Some(selection_from_pack(
                pack,
                100.0,
                "explicit field signal".to_string(),
                vec![field],
            ));
        }
    }
    if is_harness_meta_need(need) {
        return None;
    }

    let signals = field_signal_tokens(goal, need);
    let mut best: Option<(f32, Vec<String>, &FieldPack)> = None;
    for pack in packs {
        let haystack = pack_tokens(pack);
        let matched = signals
            .iter()
            .filter(|signal| haystack.contains(*signal))
            .cloned()
            .collect::<Vec<_>>();
        if matched.is_empty() {
            continue;
        }
        let id_bonus = matched
            .iter()
            .filter(|signal| pack.id.split('-').any(|part| part == signal.as_str()))
            .count() as f32;
        let score = matched.len() as f32 + id_bonus;
        if best
            .as_ref()
            .map(|(best_score, _, _)| score > *best_score)
            .unwrap_or(true)
        {
            best = Some((score, matched, pack));
        }
    }

    best.map(|(score, matched, pack)| {
        selection_from_pack(
            pack,
            score,
            format!("metadata overlap: {}", matched.join(",")),
            matched,
        )
    })
}

fn is_harness_meta_need(need: &Need) -> bool {
    if !matches!(
        need.kind,
        NeedKind::Execute | NeedKind::Verify | NeedKind::Observe
    ) {
        return false;
    }
    let text = need.query.trim().to_ascii_lowercase();
    let meta_markers = [
        "evolve recommend",
        "octopus evolve",
        "octopus repair",
        "harness repair",
        "repair plan",
        "repair_session",
        "field-mini-task harness",
        "harness after",
    ];
    meta_markers.iter().any(|marker| text.contains(marker))
}

pub fn annotate_need_with_field(need: &mut Need, selection: &FieldPackSelection) {
    need.context
        .insert("field_pack".to_string(), selection.field.clone());
    need.context
        .insert("field_score".to_string(), format!("{:.2}", selection.score));
    need.context
        .insert("field_reason".to_string(), selection.reason.clone());
    need.context.insert(
        "field_verifier".to_string(),
        selection.verifier_method.clone(),
    );
    need.context.insert(
        "field_pass_signal".to_string(),
        selection.pass_signal.clone(),
    );
}

fn parse_index(content: &str) -> Result<FieldPackIndex, String> {
    serde_json::from_str(content).map_err(|error| format!("invalid field pack index: {error}"))
}

fn parse_pack(expected_id: &str, content: &str) -> Result<FieldPack, String> {
    let pack: FieldPack =
        serde_json::from_str(content).map_err(|error| format!("invalid field pack: {error}"))?;
    if pack.id != expected_id {
        return Err(format!(
            "field pack id mismatch: expected {expected_id}, found {}",
            pack.id
        ));
    }
    Ok(pack)
}

fn explicit_field_signal(goal: Option<&Goal>, need: &Need) -> Option<String> {
    ["field_pack", "field"]
        .iter()
        .find_map(|key| need.context.get(*key).cloned())
        .or_else(|| {
            goal.and_then(|goal| {
                ["field_pack", "field"]
                    .iter()
                    .find_map(|key| goal.signals.get(*key).cloned())
            })
        })
        .map(|field| field.trim().to_ascii_lowercase())
        .filter(|field| !field.is_empty())
}

fn field_signal_tokens(goal: Option<&Goal>, need: &Need) -> BTreeSet<String> {
    let mut text = format!("{:?} {}", need.kind, need.query);
    for value in need.context.values() {
        text.push(' ');
        text.push_str(value);
    }
    if let Some(goal) = goal {
        text.push(' ');
        text.push_str(&goal.objective);
        for value in &goal.constraints {
            text.push(' ');
            text.push_str(value);
        }
        for value in goal.signals.values() {
            text.push(' ');
            text.push_str(value);
        }
    }
    tokens(&text)
}

fn pack_tokens(pack: &FieldPack) -> BTreeSet<String> {
    let mut text = format!("{} {}", pack.id, pack.description);
    for value in pack
        .task_schema
        .inputs
        .iter()
        .chain(pack.task_schema.outputs.iter())
        .chain(pack.task_schema.constraints.iter())
        .chain(pack.capability_hints.iter())
        .chain(pack.permission_boundary.safe.iter())
        .chain(pack.permission_boundary.requires_grant.iter())
        .chain(pack.permission_boundary.blocked_by_default.iter())
        .chain(pack.verifier.error_categories.iter())
        .chain(pack.trajectory_labels.iter())
    {
        text.push(' ');
        text.push_str(value);
    }
    text.push(' ');
    text.push_str(&pack.verifier.method);
    text.push(' ');
    text.push_str(&pack.verifier.pass_signal);
    for task in &pack.mini_tasks {
        text.push(' ');
        text.push_str(&task.id);
        text.push(' ');
        text.push_str(&task.goal);
        text.push(' ');
        text.push_str(&task.expected_feed);
    }
    tokens(&text)
}

fn tokens(value: &str) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    let mut current = String::new();
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            current.push(character);
        } else {
            push_token(&mut tokens, &mut current);
        }
    }
    push_token(&mut tokens, &mut current);
    tokens
}

fn push_token(tokens: &mut BTreeSet<String>, current: &mut String) {
    if current.len() >= 2 && !TOKEN_STOPWORDS.contains(&current.as_str()) {
        tokens.insert(std::mem::take(current));
    } else {
        current.clear();
    }
}

const TOKEN_STOPWORDS: &[&str] = &[
    "a", "an", "and", "as", "by", "for", "from", "in", "into", "is", "it", "of", "on", "or", "the",
    "this", "to", "with",
];

fn selection_from_pack(
    pack: &FieldPack,
    score: f32,
    reason: String,
    signals: Vec<String>,
) -> FieldPackSelection {
    FieldPackSelection {
        field: pack.id.clone(),
        score,
        reason,
        signals,
        verifier_method: pack.verifier.method.clone(),
        pass_signal: pack.verifier.pass_signal.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Need, NeedKind};

    #[test]
    fn embedded_catalog_contains_initial_fields() {
        let catalog = embedded_field_pack_catalog().unwrap();
        let ids = catalog
            .packs
            .iter()
            .map(|pack| pack.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec![
                "math",
                "search",
                "code",
                "swe",
                "research",
                "computer-use",
                "ib",
                "robotics"
            ]
        );
    }

    #[test]
    fn selection_uses_explicit_field_signal() {
        let catalog = embedded_field_pack_catalog().unwrap();
        let mut need = Need::new(NeedKind::Verify, "check the result");
        need.context
            .insert("field_pack".to_string(), "robotics".to_string());

        let selection = select_field_pack(&catalog.packs, None, &need).unwrap();

        assert_eq!(selection.field, "robotics");
        assert_eq!(selection.score, 100.0);
    }

    #[test]
    fn selection_uses_pack_metadata_overlap() {
        let catalog = embedded_field_pack_catalog().unwrap();
        let need = Need::new(
            NeedKind::Verify,
            "dedupe search results and keep citations from sources",
        );

        let selection = select_field_pack(&catalog.packs, None, &need).unwrap();

        assert_eq!(selection.field, "search");
        assert!(selection.score > 0.0);
    }

    #[test]
    fn selection_ignores_harness_evolution_commands() {
        let catalog = embedded_field_pack_catalog().unwrap();
        let need = Need::new(NeedKind::Execute, "evolve recommend field-mini-task");

        let selection = select_field_pack(&catalog.packs, None, &need);

        assert!(selection.is_none());
    }

    #[test]
    fn annotation_keeps_field_signal_inside_need_context() {
        let catalog = embedded_field_pack_catalog().unwrap();
        let mut need = Need::new(NeedKind::Execute, "make a small patch and run tests");
        let selection = select_field_pack(&catalog.packs, None, &need).unwrap();

        annotate_need_with_field(&mut need, &selection);

        assert_eq!(
            need.context.get("field_pack").map(String::as_str),
            Some("code")
        );
        assert!(need.context.contains_key("field_verifier"));
    }
}
