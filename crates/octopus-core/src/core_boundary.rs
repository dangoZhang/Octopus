use super::{default_tentacles_root, repo_root};
use crate::profile_registry::state_profile_registry_path;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct CoreBoundaryReport {
    pub(crate) policy: String,
    pub(crate) stable_rust: Vec<BoundaryPathReport>,
    pub(crate) product_app: Vec<BoundaryPathReport>,
    pub(crate) editable_harness: Vec<BoundaryPathReport>,
    pub(crate) ok: bool,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct BoundaryPathReport {
    pub(crate) path: String,
    pub(crate) kind: String,
    pub(crate) exists: bool,
}

pub(crate) fn report(state_path: &Path) -> CoreBoundaryReport {
    let root = repo_root();
    let stable_rust = vec![
        path_report(root.join("crates/octopus-core/src/lib.rs"), "kernel"),
        path_report(
            root.join("crates/octopus-core/src/brain_loop.rs"),
            "clean-brain-loop",
        ),
        path_report(
            root.join("crates/octopus-core/src/llm_provider.rs"),
            "llm-provider",
        ),
        path_report(
            root.join("crates/octopus-core/src/llm_layers.rs"),
            "llm-layer-routing",
        ),
        path_report(
            root.join("crates/octopus-core/src/manifest_catalog.rs"),
            "manifest-catalog",
        ),
        path_report(
            root.join("crates/octopus-core/src/manifest_runtime.rs"),
            "manifest-tentacle-runtime",
        ),
        path_report(root.join("crates/octopus-core/src/main.rs"), "cli-dispatch"),
        path_report(
            root.join("crates/octopus-core/src/app_bridge.rs"),
            "product-app",
        ),
        path_report(
            root.join("crates/octopus-core/src/bundled_harness.rs"),
            "installed-bundle-materializer",
        ),
        path_report(
            root.join("crates/octopus-core/src/download.rs"),
            "download-manifest",
        ),
        path_report(
            root.join("crates/octopus-core/src/need_queue.rs"),
            "need-queue",
        ),
        path_report(root.join("crates/octopus-core/src/pet.rs"), "pixel-pet"),
        path_report(
            root.join("crates/octopus-core/src/profile_registry.rs"),
            "profile-registry-observer",
        ),
        path_report(
            root.join("crates/octopus-core/src/provider_surface.rs"),
            "provider-surface",
        ),
        path_report(
            root.join("crates/octopus-core/src/release_gate.rs"),
            "release-gate",
        ),
        path_report(
            root.join("crates/octopus-core/src/shell_words.rs"),
            "command-display",
        ),
        path_report(
            root.join("crates/octopus-core/src/state_report.rs"),
            "state-report",
        ),
    ];
    let product_app = vec![
        path_report(
            root.join("crates/octopus-core/src/app_bridge.rs"),
            "rust-server",
        ),
        path_report(
            root.join("crates/octopus-core/src/pet.rs"),
            "pixel-pet-rust",
        ),
        path_report(root.join("docs/app.html"), "native-html"),
        path_report(root.join("docs/pet.html"), "pixel-pet"),
    ];
    let editable_harness = vec![
        path_report(default_tentacles_root(), "tentacle-code"),
        path_report(
            root.join("tentacles/profile-registry/default.json"),
            "seed-profile-registry",
        ),
        path_report(
            state_profile_registry_path(state_path),
            "state-profile-registry",
        ),
    ];
    let mut warnings = Vec::new();
    collect_missing(&stable_rust, &mut warnings);
    collect_missing_except(&editable_harness, "state-profile-registry", &mut warnings);
    if root.join("src/octopus").exists() {
        warnings.push("old Python SDK path src/octopus still exists".to_string());
    }
    if root.join("pyproject.toml").exists() {
        warnings.push("old Python package pyproject.toml still exists".to_string());
    }
    CoreBoundaryReport {
        policy: "Rust owns stable kernel and product app; tentacles and profile registry own editable code-as-harness Feed supply.".to_string(),
        stable_rust,
        product_app,
        editable_harness,
        ok: warnings.is_empty(),
        warnings,
    }
}

fn path_report(path: PathBuf, kind: &str) -> BoundaryPathReport {
    BoundaryPathReport {
        exists: path.exists(),
        path: path.to_string_lossy().to_string(),
        kind: kind.to_string(),
    }
}

fn collect_missing(paths: &[BoundaryPathReport], warnings: &mut Vec<String>) {
    warnings.extend(
        paths
            .iter()
            .filter(|item| !item.exists)
            .map(|item| format!("missing {} {}", item.kind, item.path)),
    );
}

fn collect_missing_except(
    paths: &[BoundaryPathReport],
    optional_kind: &str,
    warnings: &mut Vec<String>,
) {
    warnings.extend(
        paths
            .iter()
            .filter(|item| item.kind != optional_kind && !item.exists)
            .map(|item| format!("missing {} {}", item.kind, item.path)),
    );
}
