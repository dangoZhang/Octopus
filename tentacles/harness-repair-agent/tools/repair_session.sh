#!/usr/bin/env bash
set -euo pipefail

payload_file="$(mktemp)"
trap 'rm -f "$payload_file"' EXIT
if [ ! -t 0 ]; then
  cat > "$payload_file" || true
fi

python3 - "$payload_file" "$@" <<'PY'
import json
import os
import shutil
import sys
import time
from pathlib import Path


def load_payload(path):
    try:
        text = Path(path).read_text(encoding="utf-8", errors="replace").strip()
    except OSError:
        return {}
    if not text:
        return {}
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        return {}


def workspace_from(args, payload):
    query = args[0] if args else payload.get("need", {}).get("query", "")
    token = query.strip().split()[0] if query.strip() else "."
    path = Path(token).expanduser()
    if not path.is_absolute():
        path = Path.cwd() / path
    if not path.exists() or not path.is_dir():
        path = Path.cwd()
    return path.resolve()


def load_json(path):
    try:
        return json.loads(path.read_text(encoding="utf-8", errors="replace"))
    except Exception:
        return {}


def rel(path, root):
    try:
        return str(path.relative_to(root))
    except ValueError:
        return str(path)


def compact(value, limit=180):
    return " ".join(str(value).split())[:limit]


def need_parts(value):
    text = compact(value, 240)
    if not text:
        return "execute", "repair harness"
    first, _, rest = text.partition(" ")
    allowed = {"observe", "verify", "reproduce", "compare", "remember", "forget", "recall", "execute"}
    if first in allowed and rest.strip():
        return first, rest.strip()
    if first in allowed:
        return first, text
    return "execute", text


payload = load_payload(sys.argv[1])
workspace = workspace_from(sys.argv[2:], payload)
state_path = Path(os.environ.get("OCTOPUS_STATE", workspace / ".octopus" / "state.json")).expanduser()
state = load_json(state_path)
feed_traces = state.get("feed_traces") or []
check_history = state.get("check_history") or []
latest_trace = feed_traces[-1] if feed_traces else {}
latest_check = check_history[-1] if check_history else {}
evolution_root = workspace / ".octopus" / "evolution"
apply_plans = sorted(evolution_root.glob("*/apply/*.json")) if evolution_root.exists() else []
proposals = sorted(evolution_root.glob("*/proposal.json")) if evolution_root.exists() else []
available = {name: bool(shutil.which(name)) for name in ["git", "python3", "bash", "curl", "gh", "node"]}
provider_ready = bool(os.environ.get("OPENAI_API_KEY")) or (workspace / ".octopus" / "llm.env").exists()

target_tentacle = "unknown"
candidate = "none"
source = "none"
next_need = "execute octopus beat 200"
commands = ["octopus beat 200", "octopus report", "octopus traces"]

if apply_plans:
    plan = apply_plans[-1]
    data = load_json(plan)
    target_tentacle = str(data.get("tentacle_id") or plan.parent.parent.name)
    candidate = str(data.get("candidate_id") or "runtime_code")
    source = rel(plan, workspace)
    next_need = f"verify apply plan {target_tentacle}/{candidate}"
    commands = [
        f"octopus oauth octopus evolve:{target_tentacle} harness:write",
        f"octopus evolve apply {target_tentacle} {candidate}",
        f"octopus evolve score {target_tentacle} {candidate} satisfied \"repair improved Feed\"",
    ]
elif proposals:
    proposal = proposals[-1]
    data = load_json(proposal)
    target_tentacle = str(data.get("tentacle_id") or proposal.parent.name)
    candidate = "recommended"
    source = rel(proposal, workspace)
    next_need = f"execute evolve recommend {target_tentacle}"
    commands = [f"octopus evolve recommend {target_tentacle}", "octopus beat 200"]
elif latest_check and str(latest_check.get("status", "")).lower() in {"failed", "partial"}:
    target_tentacle = str(latest_check.get("tentacle_id") or "unknown")
    source = compact(latest_check.get("command", "check"))
    next_need = f"execute beat repair for {target_tentacle}"
elif latest_trace and str(latest_trace.get("status", "")).lower() in {"failed", "partial"}:
    target_tentacle = str(latest_trace.get("tentacle") or "unknown")
    source = compact(latest_trace.get("summary", "feed_trace"))
    next_need = f"execute beat repair for {target_tentacle}"
elif not provider_ready:
    next_need = "verify provider setup"
    source = "provider_env_missing"
    commands = ["octopus provider save openai", "octopus provider status"]

session_root = workspace / ".octopus" / "harness-repair"
session_root.mkdir(parents=True, exist_ok=True)
session_id = time.strftime("%Y%m%d-%H%M%S")
session_dir = session_root / session_id
suffix = 1
while session_dir.exists():
    session_dir = session_root / f"{session_id}-{suffix}"
    suffix += 1
session_dir.mkdir(parents=True)
session_json = session_dir / "SESSION.json"
session_md = session_dir / "SESSION.md"
prompt_md = session_dir / "PROMPT.md"
next_need_json = session_dir / "NEXT_NEED.json"
command_script = session_dir / "COMMANDS.sh"
next_need_kind, next_need_query = need_parts(next_need)
session = {
    "schema_version": "octopus-harness-repair-session-v1",
    "workspace": str(workspace),
    "state_path": str(state_path),
    "target_tentacle": target_tentacle,
    "candidate": candidate,
    "source": source,
    "next_need": next_need,
    "commands": commands,
    "signals": {
        "feed_traces": len(feed_traces),
        "check_history": len(check_history),
        "apply_plans": len(apply_plans),
        "proposals": len(proposals),
        "provider_ready": provider_ready,
        "commands": available,
    },
}
next_need_payload = {
    "kind": next_need_kind,
    "query": next_need_query,
    "source": "harness-repair-agent/repair_session",
    "session": rel(session_json, workspace),
}
session_json.write_text(json.dumps(session, ensure_ascii=True, indent=2) + "\n", encoding="utf-8")
next_need_json.write_text(json.dumps(next_need_payload, ensure_ascii=True, indent=2) + "\n", encoding="utf-8")
command_script.write_text(
    "\n".join(
        [
            "#!/usr/bin/env bash",
            "set -euo pipefail",
            "",
            "# Review this generated harness repair script before running it.",
            *commands,
            "",
        ]
    ),
    encoding="utf-8",
)
command_script.chmod(0o755)
prompt_md.write_text(
    "\n".join(
        [
            "# Harness Repair Prompt",
            "",
            "You are the Octopus harness-repair-agent tentacle brain.",
            "Clean-brain context stays Goal + Mem + Need + Feed.",
            "Use only this session's Need, Tool, Action, and Feed evidence to plan the next repair step.",
            "",
            "Return compact Feed with:",
            "- status",
            "- evidence",
            "- next Need",
            "- required grant or review boundary",
            "",
            f"workspace: `{workspace}`",
            f"target tentacle: `{target_tentacle}`",
            f"candidate: `{candidate}`",
            f"source: `{source}`",
            f"next Need: `{next_need_kind} {next_need_query}`",
            "",
            "signals:",
            f"- feed traces: {len(feed_traces)}",
            f"- check history: {len(check_history)}",
            f"- apply plans: {len(apply_plans)}",
            f"- proposals: {len(proposals)}",
            f"- provider ready: {provider_ready}",
            "",
            "artifacts:",
            f"- session: `{rel(session_json, workspace)}`",
            f"- next need: `{rel(next_need_json, workspace)}`",
            f"- commands: `{rel(command_script, workspace)}`",
        ]
    )
    + "\n",
    encoding="utf-8",
)
session_md.write_text(
    "\n".join(
        [
            "# Harness Repair Session",
            "",
            f"target: `{target_tentacle}`",
            f"candidate: `{candidate}`",
            f"source: `{source}`",
            f"next Need: `{next_need}`",
            "",
            "commands:",
            *[f"- `{command}`" for command in commands],
            "",
            f"json: `{rel(session_json, workspace)}`",
            f"prompt: `{rel(prompt_md, workspace)}`",
            f"next need: `{rel(next_need_json, workspace)}`",
            f"commands: `{rel(command_script, workspace)}`",
        ]
    )
    + "\n",
    encoding="utf-8",
)

metadata = {
    "tool": "repair_session",
    "tentacle": "harness-repair-agent",
    "workspace": str(workspace),
    "session": rel(session_json, workspace),
    "session_markdown": rel(session_md, workspace),
    "prompt": rel(prompt_md, workspace),
    "next_need_file": rel(next_need_json, workspace),
    "command_script": rel(command_script, workspace),
    "target_tentacle": target_tentacle,
    "candidate": candidate,
    "next_need_kind": next_need_kind,
    "next_need_query": next_need_query,
    "next_need": next_need,
}

print(
    json.dumps(
        {
            "status": "satisfied",
            "output": f"harness repair session: {rel(session_md, workspace)}; next Need: {next_need}",
            "evidence": [
                {
                    "source": "harness-repair-agent/repair_session",
                    "content": json.dumps(session, sort_keys=True),
                    "confidence": 0.86,
                    "metadata": metadata,
                }
            ],
            "metadata": metadata,
        },
        ensure_ascii=True,
    )
)
PY
