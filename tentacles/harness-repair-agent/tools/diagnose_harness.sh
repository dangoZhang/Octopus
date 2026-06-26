#!/usr/bin/env bash
set -euo pipefail

workspace="${1:-.}"
payload_file="$(mktemp)"
trap 'rm -f "$payload_file"' EXIT
if [ ! -t 0 ]; then
  cat > "$payload_file" || true
fi

python3 - "$workspace" "$payload_file" <<'PY'
import json
import os
import subprocess
import sys
from pathlib import Path


def compact(value, limit=160):
    text = " ".join(str(value).split())
    return text if len(text) <= limit else text[: limit - 3] + "..."


def read_payload():
    raw = ""
    if len(sys.argv) > 2:
        raw = Path(sys.argv[2]).read_text(encoding="utf-8", errors="replace").strip()
    if not raw:
        return {}
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        return {"raw_stdin": compact(raw)}


def resolve_workspace(argument, payload):
    query = payload.get("need", {}).get("query", "")
    token = argument or "."
    if argument == "." and query.strip():
        candidate = query.strip().split()[0]
        if Path(candidate).exists():
            token = candidate
    path = Path(token).expanduser()
    if not path.is_absolute():
        path = Path.cwd() / path
    return path.resolve()


def load_json(path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None


def run(command, cwd):
    try:
        output = subprocess.check_output(
            command,
            cwd=str(cwd),
            stderr=subprocess.STDOUT,
            text=True,
            timeout=3,
        )
        return compact(output, 400)
    except Exception as exc:
        return compact(exc, 180)


def ids(items):
    values = []
    for item in items if isinstance(items, list) else []:
        if isinstance(item, dict):
            values.append(str(item.get("id") or item.get("name") or item))
        else:
            values.append(str(item))
    return values


payload = read_payload()
root = resolve_workspace(sys.argv[1] if len(sys.argv) > 1 else ".", payload)
state_candidates = [root / ".octopus/state.json", root / "state.json"]
state_path = next((path for path in state_candidates if path.exists()), None)
state = load_json(state_path) if state_path else None

installed = ids((state or {}).get("installed_tentacles", []))
profiles = ids((state or {}).get("installed_profiles", []))
traces = (state or {}).get("feed_traces", [])
history = (state or {}).get("check_history", [])
routes = (state or {}).get("routes", {})
grants = (state or {}).get("capability_grants", [])
evolution_root = root / ".octopus/evolution"
evolution_dirs = sorted(path.name for path in evolution_root.iterdir() if path.is_dir()) if evolution_root.exists() else []
provider_env = root / ".octopus/llm.env"

git_status = "not a git repo"
if (root / ".git").exists() or run(["git", "rev-parse", "--is-inside-work-tree"], root) == "true":
    git_status = run(["git", "status", "--short"], root) or "clean"

state_label = "present" if state else "missing"
summary = (
    f"harness diagnosis: state={state_label}; tentacles={len(installed)}; "
    f"profiles={len(profiles)}; traces={len(traces)}; checks={len(history)}; "
    f"routes={len(routes) if isinstance(routes, dict) else 0}; grants={len(grants)}; "
    f"evolution={len(evolution_dirs)}; provider_env={'present' if provider_env.exists() else 'missing'}; "
    f"git={compact(git_status, 90)}"
)

next_need = "verify harness feedback loop"
status = "satisfied"
if not state:
    status = "partial"
    next_need = "bootstrap local state"
elif not installed:
    status = "partial"
    next_need = "install seed tentacles"
elif not traces and not history:
    next_need = "collect Feed trace or check history"

metadata = {
    "workspace": str(root),
    "state": state_label,
    "installed_tentacles": ",".join(installed),
    "installed_profiles": ",".join(profiles),
    "feed_trace_count": str(len(traces)),
    "check_history_count": str(len(history)),
    "evolution_tentacles": ",".join(evolution_dirs),
    "provider_env": "present" if provider_env.exists() else "missing",
    "next_need": next_need,
}

print(json.dumps({
    "status": status,
    "output": summary,
    "evidence": [{
        "source": "diagnose_harness",
        "content": summary,
        "confidence": 0.86,
        "metadata": metadata,
    }],
    "metadata": metadata,
}, ensure_ascii=True))
PY
