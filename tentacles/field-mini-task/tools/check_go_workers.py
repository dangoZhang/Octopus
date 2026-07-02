#!/usr/bin/env python3
"""Check the editable Go worker substrate without faking runtime availability."""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path


def tentacle_root(start: Path) -> Path:
    for candidate in [start, *start.parents]:
        if (candidate / "manifest.json").is_file() and (candidate / "tools").is_dir():
            return candidate
    raise SystemExit("could not find field-mini-task root")


def rel(path: Path, root: Path) -> str:
    try:
        return str(path.relative_to(root))
    except ValueError:
        return str(path)


def main() -> int:
    root = tentacle_root(Path.cwd().resolve())
    required = [
        root / "go.mod",
        root / "internal" / "fieldworker" / "feed.go",
        root / "workers" / "default" / "main.go",
    ]
    worker_files = sorted((root / "workers").glob("*/*/main.go"))
    missing = [rel(path, root) for path in required if not path.exists()]
    go_bin = shutil.which("go")
    report: dict[str, object] = {
        "status": "ok" if not missing else "failed",
        "go_available": bool(go_bin),
        "go_runtime_status": "available" if go_bin else "partial",
        "missing": missing,
        "checked": [rel(path, root) for path in required if path.exists()],
        "worker_count": len(worker_files),
        "workers": [rel(path, root) for path in worker_files],
    }
    if missing:
        print(json.dumps(report, indent=2, sort_keys=True))
        return 1
    if not go_bin:
        report["error_category"] = "go_runtime_missing"
        report["note"] = "Go worker substrate exists; live Go execution will return honest partial until Go is installed."
        print(json.dumps(report, indent=2, sort_keys=True))
        return 0
    completed = subprocess.run(
        [go_bin, "test", "./..."],
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
    )
    report["go_test_status"] = "ok" if completed.returncode == 0 else "failed"
    report["stdout"] = completed.stdout[-1200:]
    report["stderr"] = completed.stderr[-1200:]
    if completed.returncode != 0:
        report["status"] = "failed"
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if report["status"] == "ok" else 1


if __name__ == "__main__":
    raise SystemExit(main())
