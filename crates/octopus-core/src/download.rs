use crate::app_bridge;
use crate::release_gate::{preflight_check, PreflightCheck};

#[derive(Debug, serde::Serialize)]
pub(crate) struct DownloadReport {
    pub(crate) current_version: String,
    pub(crate) repository: String,
    pub(crate) source_archive_url: String,
    pub(crate) cargo_package: String,
    pub(crate) binary: String,
    pub(crate) install_script_url: String,
    pub(crate) install_script_shell: String,
    pub(crate) install: DownloadCommand,
    pub(crate) update: DownloadCommand,
    pub(crate) verify: Vec<DownloadCommand>,
    pub(crate) start: String,
    pub(crate) docs: Vec<DownloadLink>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct DownloadCommand {
    pub(crate) label: String,
    pub(crate) command: Vec<String>,
    pub(crate) shell: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct DownloadLink {
    pub(crate) label: String,
    pub(crate) url: String,
}

pub(crate) fn download_report() -> DownloadReport {
    let install = install_command();
    let update = vec![
        "octopus".to_string(),
        "update".to_string(),
        "--run".to_string(),
    ];
    let verify_binary = vec!["octopus".to_string(), "--version".to_string()];
    let verify_app = vec![
        "octopus".to_string(),
        "start".to_string(),
        "--check".to_string(),
        "127.0.0.1:18765".to_string(),
    ];
    let repository = "https://github.com/dangoZhang/Octopus".to_string();
    let install_script_url = "https://dangozhang.github.io/Octopus/install.sh".to_string();
    DownloadReport {
        current_version: env!("CARGO_PKG_VERSION").to_string(),
        source_archive_url: format!("{repository}/archive/refs/heads/main.zip"),
        repository,
        cargo_package: "octopus-core".to_string(),
        binary: "octopus".to_string(),
        install_script_shell: format!("curl -fsSL {install_script_url} | sh"),
        install_script_url,
        install: DownloadCommand {
            label: "Install from GitHub with Cargo".to_string(),
            shell: shell_command(&install),
            command: install,
        },
        update: DownloadCommand {
            label: "Update existing install".to_string(),
            shell: shell_command(&update),
            command: update,
        },
        verify: vec![
            DownloadCommand {
                label: "Check installed binary".to_string(),
                shell: shell_command(&verify_binary),
                command: verify_binary,
            },
            DownloadCommand {
                label: "Check local app startup".to_string(),
                shell: shell_command(&verify_app),
                command: verify_app,
            },
        ],
        start: "octopus start --open".to_string(),
        docs: vec![
            DownloadLink {
                label: "Try app".to_string(),
                url: "https://dangozhang.github.io/Octopus/app.html".to_string(),
            },
            DownloadLink {
                label: "Quick Install & Use".to_string(),
                url: "https://dangozhang.github.io/Octopus/quickstart.html".to_string(),
            },
            DownloadLink {
                label: "Recipes".to_string(),
                url: "https://dangozhang.github.io/Octopus/recipes.html".to_string(),
            },
        ],
        next: vec![
            "octopus --version".to_string(),
            "octopus start --check 127.0.0.1:18765".to_string(),
            "octopus start --open".to_string(),
            "octopus first-run \"make this repo easier to use\"".to_string(),
        ],
    }
}

pub(crate) fn download_artifacts_preflight_check() -> PreflightCheck {
    let report = download_report();
    let report_value = serde_json::to_value(&report).ok();
    let manifest_value =
        serde_json::from_str::<serde_json::Value>(include_str!("../../../docs/download.json")).ok();
    let manifest_matches = manifest_value.as_ref() == report_value.as_ref();
    let static_manifest_matches = app_bridge::static_page("/download.json")
        .ok()
        .and_then(|(content_type, body)| {
            let value = serde_json::from_slice::<serde_json::Value>(&body).ok();
            Some(content_type == "application/json" && value.as_ref() == report_value.as_ref())
        })
        .unwrap_or(false);
    let install_script = include_str!("../../../docs/install.sh");
    let install_script_matches = install_script.starts_with("#!/usr/bin/env sh")
        && install_script.contains("cargo install")
        && install_script.contains("--version")
        && install_script.contains("octopus start --open")
        && install_script.contains(&report.repository)
        && install_script.contains(&report.cargo_package)
        && install_script.contains(&report.binary);
    let static_install_matches = app_bridge::static_page("/install.sh")
        .ok()
        .map(|(content_type, body)| {
            content_type == "text/x-shellscript" && body.as_slice() == install_script.as_bytes()
        })
        .unwrap_or(false);

    preflight_check(
        "download_artifacts",
        manifest_matches
            && static_manifest_matches
            && install_script_matches
            && static_install_matches,
        true,
        format!(
            "manifest={}, static_manifest={}, install_script={}, static_install={}",
            manifest_matches,
            static_manifest_matches,
            install_script_matches,
            static_install_matches
        ),
        "octopus download; docs/download.json; docs/install.sh",
    )
}

pub(crate) fn install_command() -> Vec<String> {
    [
        "cargo",
        "install",
        "--git",
        "https://github.com/dangoZhang/Octopus",
        "octopus-core",
        "--locked",
        "--bin",
        "octopus",
        "--force",
    ]
    .iter()
    .map(|value| value.to_string())
    .collect()
}

fn shell_command(command: &[String]) -> String {
    command
        .iter()
        .map(|part| shell_arg(part))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_arg(value: &str) -> String {
    if !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || "/._-=".contains(ch))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}
