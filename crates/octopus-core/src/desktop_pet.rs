use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DesktopPetSourceCheck {
    pub(crate) status: String,
    pub(crate) evidence: String,
    pub(crate) next: String,
}

pub(crate) fn desktop_pet_source_check() -> DesktopPetSourceCheck {
    if !cfg!(target_os = "macos") {
        return DesktopPetSourceCheck {
            status: "skipped".to_string(),
            evidence: "desktop pet source typecheck requires macOS AppKit".to_string(),
            next: "run octopus preflight on macOS before release".to_string(),
        };
    }
    if let Err(error) = require_command("swiftc") {
        return DesktopPetSourceCheck {
            status: "fail".to_string(),
            evidence: error,
            next: "install Xcode command line tools, then rerun octopus preflight".to_string(),
        };
    }

    let source_path = std::env::temp_dir().join(format!(
        "octopus-desktop-pet-typecheck-{}-{}.swift",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|time| time.as_nanos())
            .unwrap_or_default()
    ));
    if let Err(error) = fs::write(&source_path, DESKTOP_PET_SOURCE) {
        return DesktopPetSourceCheck {
            status: "fail".to_string(),
            evidence: format!("failed to write desktop pet source for typecheck: {error}"),
            next: "check temporary directory permissions, then rerun octopus preflight".to_string(),
        };
    }
    let output = Command::new("swiftc")
        .arg("-typecheck")
        .arg(&source_path)
        .output();
    let _ = fs::remove_file(&source_path);

    match output {
        Ok(output) if output.status.success() => DesktopPetSourceCheck {
            status: "pass".to_string(),
            evidence: "embedded Swift desktop pet source typechecked".to_string(),
            next: "octopus pet desktop".to_string(),
        },
        Ok(output) => DesktopPetSourceCheck {
            status: "fail".to_string(),
            evidence: format!(
                "swiftc -typecheck failed: {}",
                compact_command_output(&output.stderr)
            ),
            next: "fix desktop/pet/OctopusDesktopPet.swift, then rerun octopus preflight"
                .to_string(),
        },
        Err(error) => DesktopPetSourceCheck {
            status: "fail".to_string(),
            evidence: format!("failed to run swiftc -typecheck: {error}"),
            next: "install Xcode command line tools, then rerun octopus preflight".to_string(),
        },
    }
}

pub(crate) fn launch_desktop_pet(
    state_path: &Path,
    config: DesktopPetConfig,
) -> Result<DesktopPetReport, String> {
    if !(1..=8).contains(&config.worker_cap) {
        return Err("desktop pet workers must be between 1 and 8".to_string());
    }
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

    let worker_cap = config.worker_cap;
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

fn compact_command_output(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes).trim().replace('\n', " ");
    if text.is_empty() {
        return "no compiler output".to_string();
    }
    const LIMIT: usize = 360;
    if text.chars().count() > LIMIT {
        format!("{}...", text.chars().take(LIMIT).collect::<String>())
    } else {
        text
    }
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

#[cfg(test)]
mod tests {
    use super::{launch_desktop_pet, DesktopPetConfig, DESKTOP_PET_SOURCE};
    use std::path::Path;

    #[test]
    fn desktop_pet_keeps_observation_bubbles_ephemeral() {
        assert!(DESKTOP_PET_SOURCE.contains("func freshEvent"));
        assert!(DESKTOP_PET_SOURCE.contains("let observationFreshSeconds: TimeInterval = 8"));
        assert!(DESKTOP_PET_SOURCE.contains("func freshTimestamp"));
        assert!(DESKTOP_PET_SOURCE.contains("age >= 0 && age <= observationFreshSeconds"));
        assert!(DESKTOP_PET_SOURCE.contains("showNeedBubble(text: needText)"));
        assert!(DESKTOP_PET_SOURCE.contains("drawNeedBubble(text: needText, color: palette.head)"));
        assert!(DESKTOP_PET_SOURCE.contains("func needBubblePulse"));
        assert!(DESKTOP_PET_SOURCE.contains("color.withAlphaComponent(0.10 + pulse * 0.08)"));
        assert!(DESKTOP_PET_SOURCE
            .contains("let freshNeedEvent = eventFresh && eventState == \"need\""));
        assert!(DESKTOP_PET_SOURCE.contains("?? (freshNeedEvent ? eventSummary : nil)"));
        assert!(DESKTOP_PET_SOURCE.contains("return snapshot.showNeedBubble"));
        assert!(DESKTOP_PET_SOURCE.contains("snapshot.showActionBubbles = eventFresh"));
    }

    #[test]
    fn desktop_pet_rejects_invalid_worker_cap_before_platform_check() {
        let error = launch_desktop_pet(Path::new("state.json"), DesktopPetConfig { worker_cap: 9 })
            .unwrap_err();

        assert!(error.contains("workers must be between 1 and 8"));
    }

    #[test]
    fn desktop_pet_source_rejects_direct_worker_arg_instead_of_clamping() {
        assert!(DESKTOP_PET_SOURCE.contains("desktop pet workers must be between 1 and 8"));
        assert!(DESKTOP_PET_SOURCE.contains("usageError("));
        assert!(DESKTOP_PET_SOURCE.contains("Darwin.exit(64)"));
        assert!(DESKTOP_PET_SOURCE.contains("config.workerCap = count"));
    }

    #[test]
    fn desktop_pet_source_opens_without_stealing_focus() {
        assert!(DESKTOP_PET_SOURCE.contains("window.orderFrontRegardless()"));
        assert!(DESKTOP_PET_SOURCE.contains("app.setActivationPolicy(.accessory)"));
        assert!(!DESKTOP_PET_SOURCE.contains("makeKeyAndOrderFront"));
        assert!(!DESKTOP_PET_SOURCE.contains("activate(ignoringOtherApps: true)"));
    }

    #[test]
    fn desktop_pet_source_maps_worker_windows_to_their_queued_needs() {
        assert!(DESKTOP_PET_SOURCE.contains("workerNeeds"));
        assert!(DESKTOP_PET_SOURCE.contains("latestWorkerNeeds"));
        assert!(DESKTOP_PET_SOURCE.contains("pendingQueuedNeedQueries"));
        assert!(
            DESKTOP_PET_SOURCE.contains("(text(item[\"status\"]) ?? \"pending\") == \"pending\"")
        );
        assert!(DESKTOP_PET_SOURCE.contains("queued_need_index"));
        assert!(DESKTOP_PET_SOURCE.contains("func observerWindowCount"));
        assert!(DESKTOP_PET_SOURCE
            .contains("observerWindowCount(runWorkers: activeRunWorkerCount, config: config)"));
        assert!(DESKTOP_PET_SOURCE.contains("return max(requested, active)"));
        assert!(DESKTOP_PET_SOURCE.contains("waitingObserverSlot()"));
        assert!(DESKTOP_PET_SOURCE.contains("observer-slot"));
        assert!(DESKTOP_PET_SOURCE.contains("drawNeedBubble(text: needText, color: palette.head)"));
        assert!(DESKTOP_PET_SOURCE.contains("field:\\(field)"));
        assert!(DESKTOP_PET_SOURCE.contains("text(worker[\"mini_task\"])"));
        assert!(DESKTOP_PET_SOURCE.contains("field:\\(field)/\\(task)"));
    }

    #[test]
    fn desktop_pet_source_keeps_worker_goal_out_of_need_bubble() {
        assert!(!DESKTOP_PET_SOURCE.contains("latestWorkerGoal"));
        assert!(!DESKTOP_PET_SOURCE.contains("text(worker[\"goal\"])"));
        assert!(!DESKTOP_PET_SOURCE.contains("|| latestWorkerGoal != nil"));
        assert!(!DESKTOP_PET_SOURCE.contains("return snapshot.showNeedBubble || workerFresh()"));
        assert!(DESKTOP_PET_SOURCE.contains("workerNeedLabel"));
        assert!(!DESKTOP_PET_SOURCE.contains("Run \\(field)"));
        assert!(DESKTOP_PET_SOURCE.contains("\\(field) · \\(task)"));
        assert!(DESKTOP_PET_SOURCE.contains("\\(field) · peer field"));
        assert!(DESKTOP_PET_SOURCE.contains("return \"\""));
    }

    #[test]
    fn desktop_pet_source_maps_worker_windows_to_their_status() {
        assert!(DESKTOP_PET_SOURCE.contains("workerStates"));
        assert!(DESKTOP_PET_SOURCE.contains("latestWorkerStates"));
        assert!(DESKTOP_PET_SOURCE.contains("workerUpdatedAt"));
        assert!(DESKTOP_PET_SOURCE.contains("updated_at_secs"));
        assert!(DESKTOP_PET_SOURCE.contains("text(worker[\"status\"])?"));
        assert!(DESKTOP_PET_SOURCE.contains("case \"failed\", \"unsupported\""));
        assert!(DESKTOP_PET_SOURCE.contains("colors(for: displayState())"));
        assert!(DESKTOP_PET_SOURCE.contains("showWorkBubbles()"));
        assert!(DESKTOP_PET_SOURCE.contains("workerFresh()"));
        assert!(DESKTOP_PET_SOURCE.contains("func workerDisplayState"));
        assert!(DESKTOP_PET_SOURCE.contains("guard workerFresh() else { return nil }"));
        assert!(DESKTOP_PET_SOURCE
            .contains("if workerIndex == 0 || workerFresh() { return \"action\" }"));
        assert!(DESKTOP_PET_SOURCE.contains("return workerState ?? \"heartbeat\""));
        assert!(DESKTOP_PET_SOURCE.contains(
            "if snapshot.showActionBubbles { return workerIndex == 0 || workerFresh() }"
        ));
        assert!(!DESKTOP_PET_SOURCE.contains("if snapshot.showActionBubbles { return true }"));
        assert!(DESKTOP_PET_SOURCE
            .contains("return freshTimestamp(snapshot.workerUpdatedAt[workerIndex])"));
        assert!(DESKTOP_PET_SOURCE.contains("[\"harness\", \"evolution\", \"feed\", \"success\", \"blocked\"].contains(displayState())"));
    }

    #[test]
    fn desktop_pet_source_observes_peer_field_pool_without_control() {
        assert!(DESKTOP_PET_SOURCE.contains("struct FieldPoolObservation"));
        assert!(DESKTOP_PET_SOURCE.contains("let peerFieldIds = [\"math\", \"search\", \"code\", \"swe\", \"research\", \"computer-use\", \"ib\", \"robotics\"]"));
        assert!(DESKTOP_PET_SOURCE.contains("observeFieldPool(root)"));
        assert!(DESKTOP_PET_SOURCE.contains("dict(root[\"field_pool\"])"));
        assert!(DESKTOP_PET_SOURCE.contains("observeSerializedFieldPool"));
        assert!(DESKTOP_PET_SOURCE.contains("observeLegacyFieldPool"));
        assert!(DESKTOP_PET_SOURCE.contains("active_slot_field"));
        assert!(DESKTOP_PET_SOURCE.contains("next_mini_task"));
        assert!(DESKTOP_PET_SOURCE.contains("completed_fields"));
        assert!(DESKTOP_PET_SOURCE.contains("field_slot_count"));
        assert!(DESKTOP_PET_SOURCE.contains("latest_worker_slot_count"));
        assert!(DESKTOP_PET_SOURCE.contains("\\(workers) worker slots"));
        assert!(DESKTOP_PET_SOURCE.contains("parallelRunPoolDetail"));
        assert!(DESKTOP_PET_SOURCE.contains("requested_worker_count"));
        assert!(DESKTOP_PET_SOURCE.contains("candidate_fields"));
        assert!(DESKTOP_PET_SOURCE.contains("stringArray(run[\"candidate_fields\"])"));
        assert!(DESKTOP_PET_SOURCE.contains(
            "requested \\(requested) · active \\(active) · candidates \\(candidateText)"
        ));
        assert!(DESKTOP_PET_SOURCE.contains("text(pool[\"worker_slots\"])"));
        assert!(DESKTOP_PET_SOURCE.contains("let fallbackWorkerPolicy = \"workers are execution slots from the peer field pool; fields stay peer\""));
        assert!(DESKTOP_PET_SOURCE
            .contains("text(latestRun?[\"worker_policy\"]) ?? fallbackWorkerPolicy"));
        assert!(DESKTOP_PET_SOURCE.contains("func bool"));
        assert!(DESKTOP_PET_SOURCE.contains("latestWorkerByField"));
        assert!(DESKTOP_PET_SOURCE.contains("latestVerifierByField"));
        assert!(DESKTOP_PET_SOURCE.contains("latestMiniTaskForField"));
        assert!(DESKTOP_PET_SOURCE.contains("let runHasActiveWorker"));
        assert!(DESKTOP_PET_SOURCE.contains("firstNonEmpty(runWorkerNeeds) != nil"));
        assert!(DESKTOP_PET_SOURCE.contains("runWorkerUpdatedAt.contains { freshTimestamp($0) }"));
        assert!(DESKTOP_PET_SOURCE.contains("let runDetail = parallelRunPoolDetail"));
        assert!(DESKTOP_PET_SOURCE.contains("summary: \"\\(peerFieldIds.count) peer fields · \\(completed) complete · active \\(activeField ?? \"none\")\\n\\(policy)\\(runDetail)\""));
        assert!(DESKTOP_PET_SOURCE.contains(
            "map { active in [active] + peerFieldIds.filter { field in field != active } }"
        ));
        assert!(DESKTOP_PET_SOURCE.contains("pool:\\(field)\\(taskSuffix)"));
        assert!(DESKTOP_PET_SOURCE.contains("snapshot.fieldPool = fieldPool.summary"));
        assert!(DESKTOP_PET_SOURCE.contains("(eventFresh ? text(lastEvent?[\"source\"]) : nil)"));
        assert!(!DESKTOP_PET_SOURCE.contains("evolve parallel --workers"));
    }
}
