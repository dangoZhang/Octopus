use octopus_core::{
    embedded_profile_registry_json, load_tentacle_profiles_from_path, TentacleProfile,
};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct ProfileRegistryReport {
    pub(crate) source: String,
    pub(crate) path: String,
    pub(crate) exists: bool,
    pub(crate) ok: bool,
    pub(crate) profile_count: usize,
    pub(crate) error: Option<String>,
    pub(crate) next: Vec<String>,
}

pub(crate) fn profile_registry_report(state_path: &Path) -> ProfileRegistryReport {
    let env_path = env::var("OCTOPUS_PROFILE_REGISTRY").ok().map(PathBuf::from);
    let state_path_candidate = state_profile_registry_path(state_path);
    let cwd_candidate = PathBuf::from(".octopus")
        .join("profile-registry")
        .join("default.json");
    let (source, path, embedded) = if let Some(path) = env_path {
        ("env".to_string(), path, false)
    } else if state_path_candidate.exists() {
        ("state".to_string(), state_path_candidate, false)
    } else if cwd_candidate.exists() {
        ("cwd".to_string(), cwd_candidate, false)
    } else {
        (
            "embedded".to_string(),
            PathBuf::from("embedded:tentacles/profile-registry/default.json"),
            true,
        )
    };

    let exists = embedded || path.exists();
    let loaded = if embedded {
        serde_json::from_str::<Vec<TentacleProfile>>(embedded_profile_registry_json())
            .map_err(|error| error.to_string())
    } else if exists {
        load_tentacle_profiles_from_path(&path)
    } else {
        Err(format!(
            "profile registry missing: {}",
            path.to_string_lossy()
        ))
    };
    let (ok, profile_count, error) = match loaded {
        Ok(profiles) => (true, profiles.len(), None),
        Err(error) => (false, 0, Some(error)),
    };
    let mut next = Vec::new();
    if !ok {
        if source == "env" {
            next.push("set OCTOPUS_PROFILE_REGISTRY to a valid JSON registry".to_string());
        }
        next.push("octopus bootstrap".to_string());
    }
    ProfileRegistryReport {
        source,
        path: path.to_string_lossy().to_string(),
        exists,
        ok,
        profile_count,
        error,
        next,
    }
}

pub(crate) fn state_profile_registry_path(state_path: &Path) -> PathBuf {
    let directory = state_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    directory.join("profile-registry").join("default.json")
}
