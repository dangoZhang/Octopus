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
import shutil
import sys
from pathlib import Path


def compact(value, limit=180):
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
    path = path.resolve()
    if path.is_file():
        return path.parent
    return path


payload = read_payload()
root = resolve_workspace(sys.argv[1] if len(sys.argv) > 1 else ".", payload)
state_path = Path(os.environ["OCTOPUS_STATE_PATH"]).expanduser() if os.environ.get("OCTOPUS_STATE_PATH") else None
commands = [
    "bash",
    "python3",
    "git",
    "cargo",
    "curl",
    "node",
    "gh",
    "open",
    "osascript",
    "screencapture",
    "xdg-open",
]
available = {command: shutil.which(command) or "" for command in commands}
missing_core = [command for command in ["bash", "python3", "git"] if not available[command]]
provider_env = root / ".octopus/llm.env"
provider_text = provider_env.read_text(encoding="utf-8", errors="replace") if provider_env.exists() else ""
provider_keys = []
for key in ["OCTOPUS_LLM_BASE_URL", "OCTOPUS_LLM_API_KEY", "OCTOPUS_LLM_MODEL"]:
    if key in provider_text or os.environ.get(key):
        provider_keys.append(key)

desktop = []
for command in ["open", "osascript", "screencapture", "xdg-open"]:
    if available[command]:
        desktop.append(command)

status = "satisfied" if not missing_core else "partial"
next_need = "run provider check" if provider_keys else "save provider env"
next_need_kind = "verify" if provider_keys else "execute"
next_need_query = "octopus provider check" if provider_keys else "octopus provider save openai"
if missing_core:
    next_need = "install missing core adapters"
    next_need_kind = "execute"
    next_need_query = "install missing core adapters"

available_labels = [command for command, path in available.items() if path]
output = (
    f"adapter probe: available={','.join(available_labels)}; "
    f"missing_core={','.join(missing_core) or 'none'}; "
    f"provider_keys={','.join(provider_keys) or 'none'}; "
    f"desktop={','.join(desktop) or 'none'}"
)
metadata = {
    "workspace": str(root),
    "state_path": str(state_path) if state_path else "",
    "available": ",".join(available_labels),
    "missing_core": ",".join(missing_core),
    "provider_env": "present" if provider_env.exists() else "missing",
    "provider_keys": ",".join(provider_keys),
    "desktop_adapters": ",".join(desktop),
    "next_need": next_need,
    "next_need_kind": next_need_kind,
    "next_need_query": next_need_query,
}

print(json.dumps({
    "status": status,
    "output": output,
    "evidence": [{
        "source": "adapter_probe",
        "content": output,
        "confidence": 0.84,
        "metadata": metadata,
    }],
    "metadata": metadata,
}, ensure_ascii=True))
PY
