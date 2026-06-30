use super::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SkillManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub needs: Vec<NeedKind>,
    pub tools: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleBrain {
    pub kind: String,
    pub description: String,
    pub model: Option<String>,
    pub prompt: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ToolImplementation {
    pub kind: String,
    pub entrypoint: String,
    #[serde(default)]
    pub contract: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ToolPermission {
    pub provider: String,
    pub scope: String,
    pub permissions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ToolMetadata {
    pub id: String,
    pub description: String,
    pub implementation: ToolImplementation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission: Option<ToolPermission>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub brain: TentacleBrain,
    pub skills: Vec<SkillManifest>,
    pub tools: Vec<ToolMetadata>,
    pub evolution: EvolutionPolicy,
    pub llm_ready: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct InstalledTentacle {
    pub id: String,
    pub name: String,
    pub source: String,
    pub brain_kind: String,
    #[serde(default)]
    pub brain_prompt: String,
    #[serde(default)]
    pub feedback_contract: Option<String>,
    pub runtime_kinds: Vec<String>,
    pub needs: Vec<String>,
    pub tools: Vec<String>,
    #[serde(default)]
    pub tool_meta: Vec<InstalledTool>,
    pub editable: Vec<String>,
    #[serde(default)]
    pub evolution_surfaces: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct InstalledTool {
    pub id: String,
    pub description: String,
    pub input: String,
    pub output: String,
    pub kind: String,
    pub entrypoint: String,
    #[serde(default)]
    pub contract: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission: Option<ToolPermission>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleManifest {
    #[serde(default, rename = "$schema")]
    pub schema: Option<String>,
    pub schema_version: String,
    pub id: String,
    pub name: String,
    pub description: String,
    pub brain: ManifestBrain,
    pub skills: Vec<ManifestSkill>,
    pub tools: Vec<ManifestTool>,
    pub evolution: EvolutionPolicy,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ManifestBrain {
    pub kind: String,
    pub model: Option<String>,
    pub prompt: String,
    #[serde(default)]
    pub feedback_contract: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ManifestSkill {
    pub id: String,
    pub description: String,
    pub needs: Vec<NeedKind>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ManifestTool {
    pub id: String,
    pub description: String,
    pub input: String,
    pub output: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission: Option<ToolPermission>,
    pub implementation: ToolImplementation,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct LoadedTentacleManifest {
    pub path: String,
    pub manifest: TentacleManifest,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleManifestReport {
    pub id: String,
    pub name: String,
    pub path: String,
    pub brain_kind: String,
    pub runtime_kinds: Vec<String>,
    pub needs: Vec<String>,
    pub tool_count: usize,
    pub editable: Vec<String>,
    pub evolution_surfaces: Vec<String>,
    pub missing_entrypoints: Vec<String>,
}

pub fn load_tentacle_manifests(
    root: impl AsRef<Path>,
) -> Result<Vec<LoadedTentacleManifest>, Error> {
    let root = root.as_ref();
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut manifests = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path().join("manifest.json");
        if !path.exists() {
            continue;
        }
        let content = fs::read_to_string(&path)?;
        let manifest = serde_json::from_str::<TentacleManifest>(&content)
            .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        let source = fs::canonicalize(&path).unwrap_or(path);
        manifests.push(LoadedTentacleManifest {
            path: source.to_string_lossy().to_string(),
            manifest,
        });
    }
    manifests.sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));
    Ok(manifests)
}

pub fn inspect_tentacle_manifests(
    root: impl AsRef<Path>,
) -> Result<Vec<TentacleManifestReport>, Error> {
    let root = root.as_ref();
    let manifests = load_tentacle_manifests(root)?;
    Ok(manifests
        .into_iter()
        .map(|loaded| manifest_report(root, loaded))
        .collect())
}

pub fn default_tentacle_profiles() -> Vec<TentacleProfile> {
    external_profile_registry()
        .and_then(|path| load_tentacle_profiles_from_path(&path).ok())
        .unwrap_or_else(embedded_tentacle_profiles)
}

pub fn embedded_profile_registry_json() -> &'static str {
    DEFAULT_PROFILE_REGISTRY
}

pub fn load_tentacle_profiles_from_path(
    path: impl AsRef<Path>,
) -> Result<Vec<TentacleProfile>, String> {
    let path = path.as_ref();
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&content).map_err(|error| error.to_string())
}

fn external_profile_registry() -> Option<PathBuf> {
    env::var(OCTOPUS_PROFILE_REGISTRY_ENV)
        .ok()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .or_else(|| {
            env::var(OCTOPUS_STATE_PATH_ENV)
                .ok()
                .map(PathBuf::from)
                .and_then(|state_path| {
                    let directory = state_path
                        .parent()
                        .filter(|path| !path.as_os_str().is_empty())
                        .unwrap_or_else(|| Path::new("."));
                    let path = directory.join("profile-registry").join("default.json");
                    path.exists().then_some(path)
                })
        })
        .or_else(|| {
            let path = PathBuf::from(LOCAL_PROFILE_REGISTRY_PATH);
            path.exists().then_some(path)
        })
}

fn embedded_tentacle_profiles() -> Vec<TentacleProfile> {
    serde_json::from_str(DEFAULT_PROFILE_REGISTRY)
        .expect("embedded tentacle profile registry must be valid")
}

fn manifest_report(root: &Path, loaded: LoadedTentacleManifest) -> TentacleManifestReport {
    let mut runtime_kinds = loaded
        .manifest
        .tools
        .iter()
        .map(|tool| tool.implementation.kind.clone())
        .collect::<Vec<_>>();
    runtime_kinds.sort();
    runtime_kinds.dedup();

    let mut needs = loaded
        .manifest
        .skills
        .iter()
        .flat_map(|skill| skill.needs.iter().map(kind_key))
        .map(str::to_string)
        .collect::<Vec<_>>();
    needs.sort();
    needs.dedup();

    let manifest_path = Path::new(&loaded.path);
    let missing_entrypoints = loaded
        .manifest
        .tools
        .iter()
        .filter(|tool| !entrypoint_exists(root, manifest_path, &tool.implementation))
        .map(|tool| tool.implementation.entrypoint.clone())
        .collect::<Vec<_>>();

    let evolution_surfaces = loaded
        .manifest
        .evolution
        .surfaces
        .iter()
        .map(|surface| surface.id.clone())
        .collect::<Vec<_>>();

    TentacleManifestReport {
        id: loaded.manifest.id,
        name: loaded.manifest.name,
        path: loaded.path,
        brain_kind: loaded.manifest.brain.kind,
        runtime_kinds,
        needs,
        tool_count: loaded.manifest.tools.len(),
        editable: loaded.manifest.evolution.editable,
        evolution_surfaces,
        missing_entrypoints,
    }
}

pub(crate) fn installed_tentacle_from_manifest(
    root: &Path,
    loaded: LoadedTentacleManifest,
) -> Result<InstalledTentacle, String> {
    let report = manifest_report(root, loaded.clone());
    if !report.missing_entrypoints.is_empty() {
        return Err(format!(
            "manifest has missing entrypoints: {}",
            report.missing_entrypoints.join(", ")
        ));
    }
    let tools = loaded
        .manifest
        .tools
        .iter()
        .map(|tool| {
            format!(
                "{}:{}:{}",
                tool.id, tool.implementation.kind, tool.implementation.entrypoint
            )
        })
        .collect::<Vec<_>>();
    let tool_meta = loaded
        .manifest
        .tools
        .iter()
        .map(|tool| InstalledTool {
            id: tool.id.clone(),
            description: tool.description.clone(),
            input: tool.input.clone(),
            output: tool.output.clone(),
            kind: tool.implementation.kind.clone(),
            entrypoint: tool.implementation.entrypoint.clone(),
            contract: tool.implementation.contract.clone(),
            permission: tool.permission.clone(),
        })
        .collect::<Vec<_>>();
    Ok(InstalledTentacle {
        id: report.id,
        name: report.name,
        source: report.path,
        brain_kind: report.brain_kind,
        brain_prompt: loaded.manifest.brain.prompt,
        feedback_contract: loaded.manifest.brain.feedback_contract,
        runtime_kinds: report.runtime_kinds,
        needs: report.needs,
        tools,
        tool_meta,
        editable: report.editable,
        evolution_surfaces: report.evolution_surfaces,
    })
}

fn entrypoint_exists(
    root: &Path,
    manifest_path: &Path,
    implementation: &ToolImplementation,
) -> bool {
    if implementation.kind == "http" {
        return implementation.entrypoint.starts_with("https://")
            || implementation.entrypoint.starts_with("http://");
    }
    let entrypoint = Path::new(&implementation.entrypoint);
    if entrypoint.is_absolute() && entrypoint.exists() {
        return true;
    }
    let manifest_relative = manifest_path
        .parent()
        .map(|parent| parent.join(entrypoint))
        .is_some_and(|path| path.exists());
    if manifest_relative {
        return true;
    }
    root.parent()
        .map(|parent| parent.join(entrypoint).exists())
        .unwrap_or(false)
}
