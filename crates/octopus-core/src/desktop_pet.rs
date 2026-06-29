use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DESKTOP_PET_SOURCE: &str = include_str!("../../../desktop/pet/OctopusDesktopPet.swift");

#[derive(Clone, Debug)]
pub(crate) struct DesktopPetConfig {
    pub(crate) worker_cap: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DesktopPetReport {
    pub(crate) app_path: String,
    pub(crate) executable_path: String,
    pub(crate) source_path: String,
    pub(crate) state_path: String,
    pub(crate) observer: bool,
    pub(crate) worker_cap: usize,
    pub(crate) launched: bool,
    pub(crate) command: Vec<String>,
}

pub(crate) fn launch_desktop_pet(
    state_path: &Path,
    config: DesktopPetConfig,
) -> Result<DesktopPetReport, String> {
    if !cfg!(target_os = "macos") {
        return Err("desktop pet currently requires macOS AppKit".to_string());
    }
    require_command("swiftc")?;
    require_command("open")?;

    let root = state_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .join("desktop-pet");
    let app_path = root.join("OctopusDesktopPet.app");
    let macos_dir = app_path.join("Contents").join("MacOS");
    let source_path = root.join("OctopusDesktopPet.swift");
    let executable_path = macos_dir.join("OctopusDesktopPet");
    fs::create_dir_all(&macos_dir).map_err(|error| error.to_string())?;
    fs::write(&source_path, DESKTOP_PET_SOURCE).map_err(|error| error.to_string())?;
    fs::write(app_path.join("Contents").join("Info.plist"), info_plist())
        .map_err(|error| error.to_string())?;

    let compile = Command::new("swiftc")
        .arg(&source_path)
        .arg("-o")
        .arg(&executable_path)
        .status()
        .map_err(|error| format!("failed to run swiftc: {error}"))?;
    if !compile.success() {
        return Err(format!("desktop pet compile failed with status {compile}"));
    }

    let worker_cap = config.worker_cap.clamp(1, 8);
    let state_path = absolute_or_current(state_path)?;
    let command = vec![
        "open".to_string(),
        "-n".to_string(),
        app_path.display().to_string(),
        "--args".to_string(),
        "--state-path".to_string(),
        state_path.display().to_string(),
        "--workers".to_string(),
        worker_cap.to_string(),
    ];
    let open = Command::new("open")
        .arg("-n")
        .arg(&app_path)
        .arg("--args")
        .arg("--state-path")
        .arg(&state_path)
        .arg("--workers")
        .arg(worker_cap.to_string())
        .status()
        .map_err(|error| format!("failed to open desktop pet: {error}"))?;
    if !open.success() {
        return Err(format!("desktop pet open failed with status {open}"));
    }

    Ok(DesktopPetReport {
        app_path: app_path.display().to_string(),
        executable_path: executable_path.display().to_string(),
        source_path: source_path.display().to_string(),
        state_path: state_path.display().to_string(),
        observer: true,
        worker_cap,
        launched: true,
        command,
    })
}

fn absolute_or_current(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    Ok(cwd.join(path))
}

fn require_command(command: &str) -> Result<(), String> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {command} >/dev/null 2>&1"))
        .status()
        .map_err(|error| format!("failed to check {command}: {error}"))?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("required command missing: {command}"))
}

fn info_plist() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key><string>OctopusDesktopPet</string>
  <key>CFBundleIdentifier</key><string>ai.octopus.desktoppet</string>
  <key>CFBundleName</key><string>Octopus Desktop Pet</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSUIElement</key><true/>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
"#
}
