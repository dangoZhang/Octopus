use std::fs;
use std::path::{Path, PathBuf};

const BUNDLED_TENTACLE_FILES: &[(&str, &[u8])] = &[
    (
        "bash-only/manifest.json",
        include_bytes!("../../../tentacles/bash-only/manifest.json"),
    ),
    (
        "bash-only/tools/write_and_run.sh",
        include_bytes!("../../../tentacles/bash-only/tools/write_and_run.sh"),
    ),
    (
        "computer-use-agent/manifest.json",
        include_bytes!("../../../tentacles/computer-use-agent/manifest.json"),
    ),
    (
        "computer-use-agent/tools/bash.sh",
        include_bytes!("../../../tentacles/computer-use-agent/tools/bash.sh"),
    ),
    (
        "computer-use-agent/tools/browser_status.sh",
        include_bytes!("../../../tentacles/computer-use-agent/tools/browser_status.sh"),
    ),
    (
        "computer-use-agent/tools/clipboard_read.sh",
        include_bytes!("../../../tentacles/computer-use-agent/tools/clipboard_read.sh"),
    ),
    (
        "computer-use-agent/tools/clipboard_write.sh",
        include_bytes!("../../../tentacles/computer-use-agent/tools/clipboard_write.sh"),
    ),
    (
        "computer-use-agent/tools/describe_screen.sh",
        include_bytes!("../../../tentacles/computer-use-agent/tools/describe_screen.sh"),
    ),
    (
        "computer-use-agent/tools/mcp.sh",
        include_bytes!("../../../tentacles/computer-use-agent/tools/mcp.sh"),
    ),
    (
        "computer-use-agent/tools/open_url.sh",
        include_bytes!("../../../tentacles/computer-use-agent/tools/open_url.sh"),
    ),
    (
        "computer-use-agent/tools/screenshot.sh",
        include_bytes!("../../../tentacles/computer-use-agent/tools/screenshot.sh"),
    ),
    (
        "computer-use-agent/tools/window_status.sh",
        include_bytes!("../../../tentacles/computer-use-agent/tools/window_status.sh"),
    ),
    (
        "harness-repair-agent/manifest.json",
        include_bytes!("../../../tentacles/harness-repair-agent/manifest.json"),
    ),
    (
        "harness-repair-agent/tools/adapter_probe.sh",
        include_bytes!("../../../tentacles/harness-repair-agent/tools/adapter_probe.sh"),
    ),
    (
        "harness-repair-agent/tools/diagnose_harness.sh",
        include_bytes!("../../../tentacles/harness-repair-agent/tools/diagnose_harness.sh"),
    ),
    (
        "harness-repair-agent/tools/heartbeat_repair.sh",
        include_bytes!("../../../tentacles/harness-repair-agent/tools/heartbeat_repair.sh"),
    ),
    (
        "harness-repair-agent/tools/repair_outcome.sh",
        include_bytes!("../../../tentacles/harness-repair-agent/tools/repair_outcome.sh"),
    ),
    (
        "harness-repair-agent/tools/repair_session.sh",
        include_bytes!("../../../tentacles/harness-repair-agent/tools/repair_session.sh"),
    ),
    (
        "json-feed/manifest.json",
        include_bytes!("../../../tentacles/json-feed/manifest.json"),
    ),
    (
        "json-feed/tools/feed.py",
        include_bytes!("../../../tentacles/json-feed/tools/feed.py"),
    ),
    (
        "repo-maintainer/manifest.json",
        include_bytes!("../../../tentacles/repo-maintainer/manifest.json"),
    ),
    (
        "repo-maintainer/tools/draft_pr.sh",
        include_bytes!("../../../tentacles/repo-maintainer/tools/draft_pr.sh"),
    ),
    (
        "repo-maintainer/tools/github_status.sh",
        include_bytes!("../../../tentacles/repo-maintainer/tools/github_status.sh"),
    ),
    (
        "repo-maintainer/tools/codex_status.sh",
        include_bytes!("../../../tentacles/repo-maintainer/tools/codex_status.sh"),
    ),
    (
        "repo-maintainer/tools/codex_maintain.sh",
        include_bytes!("../../../tentacles/repo-maintainer/tools/codex_maintain.sh"),
    ),
    (
        "repo-maintainer/tools/inspect_repo.sh",
        include_bytes!("../../../tentacles/repo-maintainer/tools/inspect_repo.sh"),
    ),
    (
        "repo-maintainer/tools/patch_queue.sh",
        include_bytes!("../../../tentacles/repo-maintainer/tools/patch_queue.sh"),
    ),
    (
        "repo-maintainer/tools/publish_pr.sh",
        include_bytes!("../../../tentacles/repo-maintainer/tools/publish_pr.sh"),
    ),
    (
        "swe-agent/manifest.json",
        include_bytes!("../../../tentacles/swe-agent/manifest.json"),
    ),
    (
        "swe-agent/tools/edit.sh",
        include_bytes!("../../../tentacles/swe-agent/tools/edit.sh"),
    ),
    (
        "swe-agent/tools/inspect_repo.sh",
        include_bytes!("../../../tentacles/swe-agent/tools/inspect_repo.sh"),
    ),
    (
        "swe-agent/tools/read.sh",
        include_bytes!("../../../tentacles/swe-agent/tools/read.sh"),
    ),
    (
        "swe-agent/tools/run_tests.sh",
        include_bytes!("../../../tentacles/swe-agent/tools/run_tests.sh"),
    ),
    (
        "swe-agent/tools/write_patch.sh",
        include_bytes!("../../../tentacles/swe-agent/tools/write_patch.sh"),
    ),
    (
        "visual/manifest.json",
        include_bytes!("../../../tentacles/visual/manifest.json"),
    ),
];

const BUNDLED_PET_HTML: &[u8] = include_bytes!("../../../docs/pet.html");

pub(crate) fn tentacles_root(cwd: &Path) -> PathBuf {
    cwd.join(".octopus").join("bundled-tentacles")
}

pub(crate) fn materialize_tentacles_root(cwd: &Path) -> Result<PathBuf, String> {
    let root = tentacles_root(cwd);
    for (relative, bytes) in BUNDLED_TENTACLE_FILES {
        let path = root.join(relative);
        write_bundled_file_if_missing(&path, bytes)?;
        if bundled_file_executable(relative) {
            make_executable(&path)?;
        }
    }
    write_bundled_file_if_missing(
        &cwd.join(".octopus").join("docs").join("pet.html"),
        BUNDLED_PET_HTML,
    )?;
    Ok(root)
}

fn write_bundled_file_if_missing(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, bytes).map_err(|error| error.to_string())
}

fn bundled_file_executable(relative: &str) -> bool {
    relative.ends_with(".sh") || relative.ends_with(".py")
}

fn make_executable(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).map_err(|error| error.to_string())?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}
