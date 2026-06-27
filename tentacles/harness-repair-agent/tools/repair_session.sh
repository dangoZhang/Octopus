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
import re
import shlex
import shutil
import subprocess
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


def load_jsonl(path, limit=5):
    if not path.exists():
        return []
    rows = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return rows[-limit:]


def rel(path, root):
    try:
        return str(path.relative_to(root))
    except ValueError:
        return str(path)


def compact(value, limit=180):
    return " ".join(str(value).split())[:limit]


def enabled_flag(name):
    return os.environ.get(name, "").strip().lower() in {"1", "true", "yes", "on"}


def first_env(names, default=""):
    for name in names:
        value = os.environ.get(name, "").strip()
        if value:
            return value
    return default


def expand_env_value(value):
    def replace_default(match):
        return os.environ.get(match.group(1), match.group(2)) or ""

    value = re.sub(r"\$\{([A-Za-z_][A-Za-z0-9_]*):-([^}]*)\}", replace_default, value)
    return re.sub(
        r"\$([A-Za-z_][A-Za-z0-9_]*)",
        lambda match: os.environ.get(match.group(1), ""),
        value,
    )


def load_provider_env(workspace):
    path = workspace / ".octopus" / "llm.env"
    loaded = []
    if not path.exists():
        return loaded
    for raw in path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("export "):
            line = line[len("export ") :].strip()
        try:
            parts = shlex.split(line, comments=True, posix=True)
        except ValueError:
            continue
        for part in parts:
            if "=" not in part:
                continue
            key, value = part.split("=", 1)
            if not re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", key):
                continue
            if key in os.environ:
                continue
            os.environ[key] = expand_env_value(value)
            loaded.append(key)
    return loaded


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


def llm_draft(prompt, session, workspace):
    prefix = first_env(
        ["OCTOPUS_REPAIR_LLM_PREFIX", "OCTOPUS_EVOLVE_LLM_PREFIX"],
        "OCTOPUS_LLM",
    )
    if not enabled_flag("OCTOPUS_REPAIR_LLM"):
        return {
            "status": "disabled",
            "prefix": prefix,
            "model": "",
            "content": "Set `OCTOPUS_REPAIR_LLM=1` to let this tentacle ask the configured provider for a reviewable repair draft.",
        }

    model = os.environ.get(f"{prefix}_MODEL", "").strip()
    if not model:
        return {
            "status": "missing_config",
            "prefix": prefix,
            "model": "",
            "content": f"{prefix}_MODEL is required. Run `octopus provider save openai {prefix}` or source `.octopus/llm.env`.",
        }

    base_url = os.environ.get(f"{prefix}_BASE_URL", "https://api.openai.com/v1").strip()
    endpoint = f"{base_url.rstrip('/')}/chat/completions"
    api_key = os.environ.get(f"{prefix}_API_KEY", "").strip()
    curl_command = os.environ.get(f"{prefix}_CURL", "curl")
    try:
        curl_parts = shlex.split(curl_command) or ["curl"]
    except ValueError:
        curl_parts = ["curl"]
    if not shutil.which(curl_parts[0]) and not Path(curl_parts[0]).exists():
        return {
            "status": "missing_config",
            "prefix": prefix,
            "model": model,
            "content": f"{prefix}_CURL command is not available: {curl_parts[0]}",
        }
    try:
        timeout_seconds = int(os.environ.get(f"{prefix}_TIMEOUT", "60"))
    except ValueError:
        timeout_seconds = 60

    body = {
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": (
                    "You are the Octopus harness-repair-agent tentacle brain. "
                    "Use Need, Tool, Action, and Feed evidence only. "
                    "Return a compact review draft with diagnosis, proposed edit, checks, and next Need."
                ),
            },
            {
                "role": "user",
                "content": "\n\n".join(
                    [
                        prompt,
                        "SESSION_JSON:",
                        json.dumps(session, ensure_ascii=True, indent=2),
                    ]
                ),
            },
        ],
        "temperature": 0.2,
    }
    command = [
        *curl_parts,
        "-sS",
        "--max-time",
        str(timeout_seconds),
        "-X",
        "POST",
        endpoint,
        "-H",
        "Content-Type: application/json",
    ]
    if api_key:
        command.extend(["-H", f"Authorization: Bearer {api_key}"])
    command.extend(["--data-binary", "@-"])
    try:
        result = subprocess.run(
            command,
            input=json.dumps(body).encode("utf-8"),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout_seconds + 5,
            check=False,
        )
    except Exception as error:
        return {
            "status": "failed",
            "prefix": prefix,
            "model": model,
            "content": compact(error, 600),
        }
    if result.returncode != 0:
        return {
            "status": "failed",
            "prefix": prefix,
            "model": model,
            "content": compact(result.stderr.decode("utf-8", errors="replace"), 600),
        }
    try:
        data = json.loads(result.stdout.decode("utf-8", errors="replace"))
        content = data["choices"][0]["message"]["content"].strip()
    except Exception:
        return {
            "status": "failed",
            "prefix": prefix,
            "model": model,
            "content": compact(result.stdout.decode("utf-8", errors="replace"), 600),
        }
    if not content:
        content = "Provider returned an empty repair draft."
    return {"status": "generated", "prefix": prefix, "model": model, "content": content}


def draft_markdown(draft, prompt_path, session_path, workspace):
    lines = [
        "# Harness Repair Draft",
        "",
        f"status: `{draft['status']}`",
        f"prefix: `{draft['prefix']}`",
    ]
    if draft.get("model"):
        lines.append(f"model: `{draft['model']}`")
    lines.extend(
        [
            f"prompt: `{rel(prompt_path, workspace)}`",
            f"session: `{rel(session_path, workspace)}`",
            "",
            draft.get("content", "").rstrip(),
            "",
        ]
    )
    return "\n".join(lines)


payload = load_payload(sys.argv[1])
workspace = workspace_from(sys.argv[2:], payload)
loaded_provider_keys = load_provider_env(workspace)
state_path = Path(
    os.environ.get("OCTOPUS_STATE_PATH")
    or os.environ.get("OCTOPUS_STATE")
    or workspace / ".octopus" / "state.json"
).expanduser()
state = load_json(state_path)
repair_root = workspace / ".octopus" / "harness-repair"
state_repair_outcomes = state.get("repair_outcomes") or []
file_repair_outcomes = load_jsonl(repair_root / "outcomes.jsonl", 6)
repair_outcomes = (state_repair_outcomes or file_repair_outcomes)[-6:]
latest_repair_outcome = repair_outcomes[-1] if repair_outcomes else {}
feed_traces = state.get("feed_traces") or []
check_history = state.get("check_history") or []
latest_trace = feed_traces[-1] if feed_traces else {}
latest_check = check_history[-1] if check_history else {}
evolution_root = workspace / ".octopus" / "evolution"
apply_plans = sorted(evolution_root.glob("*/apply/*.json")) if evolution_root.exists() else []
proposals = sorted(evolution_root.glob("*/proposal.json")) if evolution_root.exists() else []
available = {name: bool(shutil.which(name)) for name in ["git", "python3", "bash", "curl", "gh", "node"]}
repair_llm_prefix = first_env(
    ["OCTOPUS_REPAIR_LLM_PREFIX", "OCTOPUS_EVOLVE_LLM_PREFIX"],
    "OCTOPUS_LLM",
)
provider_ready = (
    bool(os.environ.get("OPENAI_API_KEY"))
    or (workspace / ".octopus" / "llm.env").exists()
    or bool(os.environ.get(f"{repair_llm_prefix}_MODEL"))
)

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
elif str(latest_repair_outcome.get("status") or latest_repair_outcome.get("outcome_status") or "").lower() in {"failed", "partial"}:
    target_tentacle = str(latest_repair_outcome.get("target_tentacle") or "unknown")
    candidate = str(latest_repair_outcome.get("candidate") or "none")
    source = f"repair_outcome:{compact(latest_repair_outcome.get('index') or latest_repair_outcome.get('session', 'session'))}"
    next_need = f"execute beat repair after {latest_repair_outcome.get('status') or latest_repair_outcome.get('outcome_status')} repair outcome"
    commands = ["octopus beat 200", "octopus repair .", "octopus report"]
elif not provider_ready:
    next_need = "verify provider setup"
    source = "provider_env_missing"
    commands = ["octopus provider save openai", "octopus provider status"]

session_root = repair_root
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
draft_md = session_dir / "DRAFT.md"
next_need_json = session_dir / "NEXT_NEED.json"
command_script = session_dir / "COMMANDS.sh"
outcome_command = (
    "octopus repair score <trace-index> satisfied \"repair improved Feed\""
)
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
        "repair_outcomes": len(repair_outcomes),
        "latest_repair_outcome": latest_repair_outcome,
        "provider_ready": provider_ready,
        "provider_env_loaded": loaded_provider_keys,
        "repair_llm_enabled": enabled_flag("OCTOPUS_REPAIR_LLM"),
        "repair_llm_prefix": repair_llm_prefix,
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
            "# After review, record the outcome so later repair sessions can learn from it.",
            outcome_command,
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
            f"- repair outcomes: {len(repair_outcomes)}",
            f"- provider ready: {provider_ready}",
            "",
            "recent repair outcomes:",
            *[
                f"- {compact(item.get('status') or item.get('outcome_status') or 'unknown')}: {compact(item.get('summary', ''))}"
                for item in repair_outcomes[-3:]
            ],
            "",
            "artifacts:",
            f"- session: `{rel(session_json, workspace)}`",
            f"- next need: `{rel(next_need_json, workspace)}`",
            f"- commands: `{rel(command_script, workspace)}`",
            f"- draft: `{rel(draft_md, workspace)}`",
        ]
    )
    + "\n",
    encoding="utf-8",
)
draft = llm_draft(prompt_md.read_text(encoding="utf-8"), session, workspace)
draft_md.write_text(draft_markdown(draft, prompt_md, session_json, workspace), encoding="utf-8")
session["draft"] = {
    "status": draft["status"],
    "path": rel(draft_md, workspace),
    "prefix": draft["prefix"],
    "model": draft.get("model", ""),
}
session_json.write_text(json.dumps(session, ensure_ascii=True, indent=2) + "\n", encoding="utf-8")
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
            f"draft: `{rel(draft_md, workspace)}`",
            f"next need: `{rel(next_need_json, workspace)}`",
            f"commands: `{rel(command_script, workspace)}`",
            f"outcome command: `{outcome_command}`",
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
    "draft": rel(draft_md, workspace),
    "llm_draft_status": draft["status"],
    "llm_prefix": draft["prefix"],
    "next_need_file": rel(next_need_json, workspace),
    "command_script": rel(command_script, workspace),
    "outcome_command": outcome_command,
    "repair_outcome_count": str(len(repair_outcomes)),
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
            "output": f"harness repair session: {rel(session_md, workspace)}; draft: {draft['status']}; next Need: {next_need}",
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
