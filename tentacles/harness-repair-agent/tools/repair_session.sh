#!/usr/bin/env bash
set -euo pipefail

payload_file="$(mktemp)"
trap 'rm -f "$payload_file"' EXIT
if [ ! -t 0 ]; then
  cat > "$payload_file" || true
fi

tool_path="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
OCTOPUS_REPAIR_TOOL_PATH="$tool_path" python3 - "$payload_file" "$@" <<'PY'
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


def outcome_status(item):
    return str(
        item.get("outcome_status")
        or item.get("status")
        or item.get("state")
        or "unknown"
    ).lower()


def normalized_outcome(item, origin):
    return {
        "origin": origin,
        "index": item.get("index", ""),
        "trace_index": item.get("trace_index", ""),
        "timestamp": item.get("timestamp") or item.get("timestamp_secs") or "",
        "session": str(item.get("session") or ""),
        "target_tentacle": str(
            item.get("target_tentacle") or item.get("tentacle_id") or "unknown"
        ),
        "candidate": str(item.get("candidate") or item.get("candidate_id") or "none"),
        "tool": str(item.get("tool") or ""),
        "source": str(item.get("source") or origin),
        "next_need": str(item.get("next_need") or ""),
        "draft_status": str(item.get("draft_status") or ""),
        "outcome_status": outcome_status(item),
        "summary": compact(item.get("summary") or item.get("content") or "", 500),
    }


def merge_repair_outcomes(state_items, journal_items, limit=8):
    merged = []
    seen = set()
    for origin, items in [("state", state_items), ("journal", journal_items)]:
        for item in items if isinstance(items, list) else []:
            if not isinstance(item, dict):
                continue
            outcome = normalized_outcome(item, origin)
            key = (
                outcome["session"],
                str(outcome["trace_index"]),
                outcome["target_tentacle"],
                outcome["candidate"],
                outcome["outcome_status"],
                outcome["summary"],
            )
            if key in seen:
                continue
            seen.add(key)
            merged.append(outcome)
    return merged[-limit:]


def outcome_memory_markdown(outcomes, workspace, outcomes_file):
    lines = [
        "# Harness Repair Outcome Memory",
        "",
        "This file is local tentacle memory. It is fed to repair-session planning, not to the clean brain.",
        "",
        f"journal: `{rel(outcomes_file, workspace)}`",
        f"count: `{len(outcomes)}`",
        "",
    ]
    if not outcomes:
        lines.append("No repair outcomes recorded yet.")
        return "\n".join(lines) + "\n"
    for item in outcomes[-8:]:
        session = item.get("session") or "none"
        target = item.get("target_tentacle") or "unknown"
        candidate = item.get("candidate") or "none"
        status = item.get("outcome_status") or "unknown"
        origin = item.get("origin") or "unknown"
        lines.extend(
            [
                f"- `{status}` target=`{target}` candidate=`{candidate}` origin=`{origin}` session=`{session}`",
                f"  summary: {compact(item.get('summary', ''), 320)}",
            ]
        )
    return "\n".join(lines) + "\n"


def read_text(path, limit=12000):
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""
    if len(text) <= limit:
        return text
    return text[:limit] + "\n\n[truncated]\n"


def source_tentacle_roots(workspace):
    roots = []
    workspace_root = workspace / "tentacles"
    if workspace_root.is_dir():
        roots.append(workspace_root)
    tool_path = os.environ.get("OCTOPUS_REPAIR_TOOL_PATH", "")
    if tool_path:
        for parent in Path(tool_path).resolve().parents:
            if parent.name == "tentacles" and parent.is_dir():
                roots.append(parent)
            nested = parent / "tentacles"
            if nested.is_dir():
                roots.append(nested)
    unique = []
    seen = set()
    for root in roots:
        key = str(root)
        if key not in seen:
            unique.append(root)
            seen.add(key)
    return unique


def tool_from_trace(trace):
    if not isinstance(trace, dict):
        return ""
    metadata = trace.get("metadata") if isinstance(trace.get("metadata"), dict) else {}
    return str(trace.get("tool") or metadata.get("tool") or "")


def tool_from_check(record):
    if not isinstance(record, dict):
        return ""
    command = str(record.get("command") or "")
    match = re.search(r"tools/([A-Za-z0-9_-]+)\.(?:sh|py|js|ts|rb|go)", command)
    return match.group(1) if match else ""


def manifest_tool_entrypoint(manifest, tool_id):
    for tool in manifest.get("tools") or []:
        if not isinstance(tool, dict) or tool.get("id") != tool_id:
            continue
        implementation = tool.get("implementation") or {}
        return str(implementation.get("entrypoint") or "")
    return ""


def build_code_context(workspace, target_tentacle, target_tool, latest_trace, latest_check, latest_outcome):
    tentacle_id = target_tentacle if target_tentacle and target_tentacle != "unknown" else "harness-repair-agent"
    tool_id = target_tool or tool_from_trace(latest_trace) or tool_from_check(latest_check)
    if not tool_id and tentacle_id == "harness-repair-agent":
        tool_id = "repair_session"
    roots = source_tentacle_roots(workspace)
    tentacle_dir = next((root / tentacle_id for root in roots if (root / tentacle_id).is_dir()), None)
    manifest_path = tentacle_dir / "manifest.json" if tentacle_dir else None
    manifest = load_json(manifest_path) if manifest_path and manifest_path.exists() else {}
    entrypoint = manifest_tool_entrypoint(manifest, tool_id) if tool_id else ""
    tool_path = None
    if tentacle_dir and entrypoint:
        tool_path = (tentacle_dir / entrypoint).resolve()
    elif tentacle_dir and tool_id:
        for extension in ["sh", "py", "js", "ts"]:
            candidate = tentacle_dir / "tools" / f"{tool_id}.{extension}"
            if candidate.exists():
                tool_path = candidate.resolve()
                break
    manifest_text = read_text(manifest_path, 10000) if manifest_path and manifest_path.exists() else ""
    tool_text = read_text(tool_path, 14000) if tool_path and tool_path.exists() else ""
    lines = [
        "# Harness Repair Code Context",
        "",
        "This file is local tentacle code evidence for repair-session planning.",
        "It is used by the harness-repair tentacle, not by clean-brain context.",
        "",
        f"workspace: `{workspace}`",
        f"tentacle: `{tentacle_id}`",
        f"tool: `{tool_id or 'unknown'}`",
        f"manifest: `{rel(manifest_path, workspace) if manifest_path else 'missing'}`",
        f"tool_path: `{rel(tool_path, workspace) if tool_path else 'missing'}`",
        "",
        "latest signals:",
        f"- trace: {compact(latest_trace, 260)}",
        f"- check: {compact(latest_check, 260)}",
        f"- repair outcome: {compact(latest_outcome, 260)}",
        "",
    ]
    if manifest_text:
        lines.extend(["## Manifest", "", "```json", manifest_text.rstrip(), "```", ""])
    else:
        lines.extend(["## Manifest", "", "No manifest found for target tentacle.", ""])
    if tool_text:
        lines.extend(["## Target Tool Code", "", "```", tool_text.rstrip(), "```", ""])
    elif tool_id:
        lines.extend(["## Target Tool Code", "", f"No local code found for tool `{tool_id}`.", ""])
    prompt_excerpt = "\n".join(lines[:18])
    if tool_text:
        prompt_excerpt += "\n\nTARGET_TOOL_EXCERPT:\n" + tool_text[:3600]
    elif manifest_text:
        prompt_excerpt += "\n\nMANIFEST_EXCERPT:\n" + manifest_text[:2400]
    return {
        "tentacle": tentacle_id,
        "tool": tool_id or "unknown",
        "manifest": str(manifest_path) if manifest_path else "",
        "tool_path": str(tool_path) if tool_path else "",
        "roots": [str(root) for root in roots],
        "markdown": "\n".join(lines) + "\n",
        "prompt_excerpt": prompt_excerpt,
    }


def metadata_of(item):
    if isinstance(item, dict) and isinstance(item.get("metadata"), dict):
        return item["metadata"]
    return {}


def resolve_artifact(workspace, value):
    value = str(value or "").strip()
    if not value:
        return None
    path = Path(value).expanduser()
    if not path.is_absolute():
        path = workspace / path
    try:
        resolved = path.resolve()
        resolved.relative_to(workspace)
    except Exception:
        return None
    return resolved


def latest_matching_field_verifier(state, latest_trace, field):
    results = state.get("field_verifier_results") if isinstance(state, dict) else []
    trace_index = str(latest_trace.get("index") or "")
    for result in reversed(results if isinstance(results, list) else []):
        if not isinstance(result, dict):
            continue
        if trace_index and str(result.get("trace_index") or "") == trace_index:
            return result
        if field and str(result.get("field") or "") == field:
            return result
    return {}


def build_field_trajectory_context(workspace, state, latest_trace):
    metadata = metadata_of(latest_trace)
    field = str(latest_trace.get("field") or metadata.get("field_pack") or "").strip()
    mini_task = str(metadata.get("field_mini_task") or "").strip()
    expected_feed = str(metadata.get("field_expected_feed") or "").strip()
    session_path = resolve_artifact(workspace, metadata.get("field_session"))
    artifact_keys = ["task_record", "prompt", "feed_draft"]
    artifacts = {
        key: resolve_artifact(workspace, metadata.get(key))
        for key in artifact_keys
    }
    verifier = latest_matching_field_verifier(state, latest_trace, field)
    lines = [
        "# Field Trajectory Context",
        "",
        "This file is local field-task evidence for harness-repair planning.",
        "It is used by the harness-repair tentacle, not by clean-brain context.",
        "",
        f"field: `{field or 'none'}`",
        f"mini_task: `{mini_task or 'none'}`",
        f"expected_feed: {expected_feed or 'not provided'}",
        f"trace_index: `{latest_trace.get('index', 'none')}`",
        f"trace_status: `{latest_trace.get('status', 'none')}`",
        f"trace_tentacle: `{latest_trace.get('tentacle', 'none')}`",
        f"trace_tool: `{latest_trace.get('tool', 'none')}`",
        f"field_session: `{rel(session_path, workspace) if session_path else 'missing'}`",
        "",
        "## Latest Trace",
        "",
        "```json",
        json.dumps(latest_trace, ensure_ascii=True, indent=2)[:8000],
        "```",
        "",
        "## Latest Field Verifier",
        "",
        "```json",
        json.dumps(verifier, ensure_ascii=True, indent=2)[:4000],
        "```",
        "",
    ]
    for key, path in artifacts.items():
        lines.extend([f"## {key}", ""])
        if path and path.exists():
            lines.extend(["```", read_text(path, 6000).rstrip(), "```", ""])
        else:
            lines.extend(["missing", ""])
    prompt_excerpt = "\n".join(lines[:30])
    if artifacts.get("task_record") and artifacts["task_record"].exists():
        prompt_excerpt += "\n\nTASK_RECORD_EXCERPT:\n" + read_text(artifacts["task_record"], 2400)
    return {
        "field": field or "none",
        "mini_task": mini_task or "none",
        "expected_feed": expected_feed,
        "session": str(session_path) if session_path else "",
        "verifier_status": str(verifier.get("status") or ""),
        "verifier_error": str(verifier.get("error_category") or ""),
        "markdown": "\n".join(lines) + "\n",
        "prompt_excerpt": prompt_excerpt,
    }


def build_adapter_context(workspace, state_path, loaded_provider_keys, repair_llm_prefix):
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
    missing_core = [
        command for command in ["bash", "python3", "git"] if not available.get(command)
    ]
    provider_env = workspace / ".octopus" / "llm.env"
    provider_text = read_text(provider_env, 6000) if provider_env.exists() else ""
    provider_keys = []
    for key in [
        "OPENAI_API_KEY",
        f"{repair_llm_prefix}_BASE_URL",
        f"{repair_llm_prefix}_API_KEY",
        f"{repair_llm_prefix}_MODEL",
        "OCTOPUS_LLM_BASE_URL",
        "OCTOPUS_LLM_API_KEY",
        "OCTOPUS_LLM_MODEL",
    ]:
        if key in loaded_provider_keys or key in provider_text or os.environ.get(key):
            provider_keys.append(key)
    provider_keys = sorted(set(provider_keys))
    desktop = [
        command
        for command in ["open", "osascript", "screencapture", "xdg-open"]
        if available.get(command)
    ]
    available_labels = [command for command, path in available.items() if path]
    status = "satisfied" if not missing_core else "partial"
    lines = [
        "# Adapter Context",
        "",
        "This file is local command, provider, desktop, and GitHub adapter evidence for repair-session planning.",
        "It is used by the harness-repair tentacle, not by clean-brain context.",
        "",
        f"workspace: `{workspace}`",
        f"state: `{rel(state_path, workspace)}`",
        f"status: `{status}`",
        f"available: `{','.join(available_labels) or 'none'}`",
        f"missing_core: `{','.join(missing_core) or 'none'}`",
        f"provider_env: `{'present' if provider_env.exists() else 'missing'}`",
        f"provider_keys: `{','.join(provider_keys) or 'none'}`",
        f"desktop_adapters: `{','.join(desktop) or 'none'}`",
        "",
        "## Command Paths",
        "",
    ]
    for command, path in available.items():
        lines.append(f"- `{command}`: `{path or 'missing'}`")
    lines.extend(
        [
            "",
            "## Provider Env",
            "",
            "```",
            provider_text.rstrip() if provider_text else "missing",
            "```",
            "",
        ]
    )
    prompt_excerpt = "\n".join(lines[:28])
    return {
        "status": status,
        "available": ",".join(available_labels),
        "missing_core": ",".join(missing_core),
        "provider_env": "present" if provider_env.exists() else "missing",
        "provider_keys": ",".join(provider_keys),
        "desktop_adapters": ",".join(desktop),
        "markdown": "\n".join(lines) + "\n",
        "prompt_excerpt": prompt_excerpt,
    }


def rel(path, root):
    if path is None:
        return "missing"
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


def shell_arg(value):
    return shlex.quote(str(value))


def build_action_plan(
    workspace,
    session_path,
    target_tentacle,
    candidate,
    source,
    next_need,
    commands,
    next_need_payload,
    code_context,
    adapter_context_path,
    adapter_context,
    outcome_memory_path,
    draft_path,
    plan_path,
):
    target = code_context.get("tentacle") or target_tentacle or "unknown"
    tool = code_context.get("tool") or "unknown"
    tool_path = code_context.get("tool_path") or ""
    check_commands = []
    if target and target != "unknown":
        check_commands.append(f"octopus check {shell_arg(target)}")
    if tool_path and tool_path.endswith(".sh"):
        check_commands.append(
            f"{shell_arg(tool_path)} {shell_arg(workspace)} | python3 -m json.tool > /dev/null"
        )
    if not check_commands:
        check_commands.append("octopus report")
    grant_command = (
        f"octopus oauth octopus evolve:{shell_arg(target)} harness:write"
        if target and target != "unknown"
        else "octopus oauth octopus harness:write"
    )
    apply_command = (
        f"octopus evolve apply {shell_arg(target)} {shell_arg(candidate)}"
        if target and target != "unknown" and candidate not in {"none", "recommended", "unknown"}
        else "octopus beat 200"
    )
    return {
        "schema_version": "octopus-harness-repair-plan-v1",
        "status": "review_required",
        "workspace": str(workspace),
        "session": rel(session_path, workspace),
        "target_tentacle": target,
        "target_tool": tool,
        "target_tool_path": tool_path,
        "candidate": candidate,
        "source": source,
        "next_need": next_need_payload,
        "inputs": {
            "adapter_context": rel(adapter_context_path, workspace),
            "code_context": rel(plan_path.parent / "CODE_CONTEXT.md", workspace),
            "outcome_memory": rel(outcome_memory_path, workspace),
            "draft": rel(draft_path, workspace),
        },
        "adapter_context_target": {
            "status": adapter_context.get("status", ""),
            "available": adapter_context.get("available", ""),
            "missing_core": adapter_context.get("missing_core", ""),
            "provider_env": adapter_context.get("provider_env", ""),
            "provider_keys": adapter_context.get("provider_keys", ""),
            "desktop_adapters": adapter_context.get("desktop_adapters", ""),
        },
        "review_boundary": "Review CODE_CONTEXT, OUTCOME_MEMORY, DRAFT, and this plan before running commands.",
        "commands": {
            "checks": check_commands,
            "grant": grant_command,
            "apply": apply_command,
            "score": "octopus repair score <trace-index> satisfied \"repair improved Feed\"",
            "suggested": commands,
        },
    }


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


def action_trace_markdown(
    workspace,
    session_path,
    need_kind,
    need_query,
    source,
    code_context,
    adapter_context,
    field_trajectory,
    draft,
    repair_plan,
):
    actions = [
        ("Need", "read cognitive repair need", f"{need_kind} {need_query}"),
        (
            "Tool",
            "probe local execution substrate",
            f"adapter={adapter_context['status']} missing_core={adapter_context['missing_core'] or 'none'}",
        ),
        (
            "Action",
            "select target code evidence",
            f"{code_context['tentacle']}/{code_context['tool']}",
        ),
        (
            "Action",
            "merge field and outcome evidence",
            f"field={field_trajectory['field']} mini_task={field_trajectory['mini_task']}",
        ),
        (
            "Tool",
            "ask optional repair provider for a draft",
            f"draft={draft['status']} prefix={draft['prefix']}",
        ),
        (
            "Feed",
            "write reviewable repair plan",
            f"plan={rel(session_path.parent / 'REPAIR_PLAN.json', workspace)} status={repair_plan['status']}",
        ),
    ]
    status = "satisfied" if code_context["tentacle"] != "unknown" else "partial"
    last_action = actions[-1][1]
    lines = [
        "# Tool-Side Action Trace",
        "",
        "This is the harness-repair tentacle's local thinking trace.",
        "It is Feed evidence for heartbeat repair, not clean-brain context.",
        "",
        f"status: `{status}`",
        f"stage_count: `{len(actions)}`",
        f"last_action: `{last_action}`",
        f"session: `{rel(session_path, workspace)}`",
        f"source: `{source}`",
        "",
        "## Need -> Tool -> Action -> Feed",
        "",
    ]
    for index, (kind, action, result) in enumerate(actions, start=1):
        lines.extend(
            [
                f"{index}. {kind}",
                f"   action: {action}",
                f"   result: {result}",
            ]
        )
    lines.extend(["", "## Review Boundary", "", repair_plan["review_boundary"], ""])
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
outcomes_file = repair_root / "outcomes.jsonl"
file_repair_outcomes = load_jsonl(outcomes_file, 12)
repair_outcomes = merge_repair_outcomes(state_repair_outcomes, file_repair_outcomes)
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
target_tool = ""
latest_trace_status = str(latest_trace.get("status", "")).lower() if latest_trace else ""
latest_trace_metadata = metadata_of(latest_trace)
latest_trace_is_field_gap = (
    latest_trace_status in {"failed", "partial"}
    and bool(latest_trace_metadata.get("field_mini_task"))
)

if latest_trace_is_field_gap:
    target_tentacle = str(latest_trace.get("tentacle") or latest_trace_metadata.get("tentacle") or "unknown")
    target_tool = tool_from_trace(latest_trace)
    field = str(latest_trace.get("field") or latest_trace_metadata.get("field_pack") or "field").strip() or "field"
    mini_task = str(latest_trace_metadata.get("field_mini_task") or "mini-task").strip()
    source = compact(latest_trace.get("summary", "field mini task trace"))
    next_need = f"execute evolve recommend {target_tentacle}"
    commands = [
        f"octopus evolve recommend {shell_arg(target_tentacle)} {shell_arg(f'improve {field} harness after {mini_task}')}",
        "octopus beat 200",
    ]
elif apply_plans:
    plan = apply_plans[-1]
    data = load_json(plan)
    target_tentacle = str(data.get("tentacle_id") or plan.parent.parent.name)
    candidate = str(data.get("candidate_id") or "runtime_code")
    target_tool = str(data.get("tool") or data.get("tool_id") or "")
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
    target_tool = tool_from_check(latest_check)
    source = compact(latest_check.get("command", "check"))
    next_need = f"execute beat repair for {target_tentacle}"
elif latest_trace and str(latest_trace.get("status", "")).lower() in {"failed", "partial"}:
    target_tentacle = str(latest_trace.get("tentacle") or "unknown")
    target_tool = tool_from_trace(latest_trace)
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
review_md = session_dir / "REVIEW.md"
next_need_json = session_dir / "NEXT_NEED.json"
command_script = session_dir / "COMMANDS.sh"
outcome_memory_md = session_dir / "OUTCOME_MEMORY.md"
adapter_context_md = session_dir / "ADAPTER_CONTEXT.md"
code_context_md = session_dir / "CODE_CONTEXT.md"
field_trajectory_md = session_dir / "FIELD_TRAJECTORY.md"
action_trace_md = session_dir / "ACTION_TRACE.md"
repair_plan_json = session_dir / "REPAIR_PLAN.json"
outcome_command = (
    "octopus repair score <trace-index> satisfied \"repair improved Feed\""
)
next_need_kind, next_need_query = need_parts(next_need)
outcome_sources = sorted({str(item.get("origin") or "unknown") for item in repair_outcomes})
code_context = build_code_context(
    workspace,
    target_tentacle,
    target_tool,
    latest_trace,
    latest_check,
    latest_repair_outcome,
)
adapter_context = build_adapter_context(
    workspace,
    state_path,
    loaded_provider_keys,
    repair_llm_prefix,
)
field_trajectory = build_field_trajectory_context(workspace, state, latest_trace)
session = {
    "schema_version": "octopus-harness-repair-session-v1",
    "workspace": str(workspace),
    "state_path": str(state_path),
    "target_tentacle": target_tentacle,
    "candidate": candidate,
    "source": source,
    "next_need": next_need,
    "commands": commands,
    "outcome_memory": rel(outcome_memory_md, workspace),
    "adapter_context": rel(adapter_context_md, workspace),
    "code_context": rel(code_context_md, workspace),
    "field_trajectory": rel(field_trajectory_md, workspace),
    "action_trace": rel(action_trace_md, workspace),
    "review": rel(review_md, workspace),
    "repair_plan": rel(repair_plan_json, workspace),
    "code_context_target": {
        "tentacle": code_context["tentacle"],
        "tool": code_context["tool"],
        "manifest": code_context["manifest"],
        "tool_path": code_context["tool_path"],
    },
    "adapter_context_target": {
        "status": adapter_context["status"],
        "available": adapter_context["available"],
        "missing_core": adapter_context["missing_core"],
        "provider_env": adapter_context["provider_env"],
        "provider_keys": adapter_context["provider_keys"],
        "desktop_adapters": adapter_context["desktop_adapters"],
    },
    "field_trajectory_target": {
        "field": field_trajectory["field"],
        "mini_task": field_trajectory["mini_task"],
        "session": field_trajectory["session"],
        "verifier_status": field_trajectory["verifier_status"],
        "verifier_error": field_trajectory["verifier_error"],
    },
    "signals": {
        "feed_traces": len(feed_traces),
        "check_history": len(check_history),
        "apply_plans": len(apply_plans),
        "proposals": len(proposals),
        "repair_outcomes": len(repair_outcomes),
        "repair_outcome_sources": outcome_sources,
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
outcome_memory_md.write_text(
    outcome_memory_markdown(repair_outcomes, workspace, outcomes_file),
    encoding="utf-8",
)
adapter_context_md.write_text(adapter_context["markdown"], encoding="utf-8")
code_context_md.write_text(code_context["markdown"], encoding="utf-8")
field_trajectory_md.write_text(field_trajectory["markdown"], encoding="utf-8")
repair_plan = build_action_plan(
    workspace,
    session_json,
    target_tentacle,
    candidate,
    source,
    next_need,
    commands,
    next_need_payload,
    code_context,
    adapter_context_md,
    adapter_context,
    outcome_memory_md,
    draft_md,
    repair_plan_json,
)
repair_plan["inputs"]["field_trajectory"] = rel(field_trajectory_md, workspace)
repair_plan["inputs"]["adapter_context"] = rel(adapter_context_md, workspace)
repair_plan["inputs"]["action_trace"] = rel(action_trace_md, workspace)
repair_plan["field_trajectory_target"] = {
    "field": field_trajectory["field"],
    "mini_task": field_trajectory["mini_task"],
    "session": field_trajectory["session"],
    "verifier_status": field_trajectory["verifier_status"],
    "verifier_error": field_trajectory["verifier_error"],
}
repair_plan["review_boundary"] = "Review ACTION_TRACE, ADAPTER_CONTEXT, FIELD_TRAJECTORY, CODE_CONTEXT, OUTCOME_MEMORY, DRAFT, and this plan before running commands."
repair_plan_json.write_text(
    json.dumps(repair_plan, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
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
recent_outcome_lines = [
    (
        f"- {compact(item.get('outcome_status') or 'unknown')}"
        f" target={compact(item.get('target_tentacle') or 'unknown')}"
        f" candidate={compact(item.get('candidate') or 'none')}"
        f" origin={compact(item.get('origin') or 'unknown')}:"
        f" {compact(item.get('summary', ''), 260)}"
    )
    for item in repair_outcomes[-4:]
] or ["- none"]
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
            f"- repair outcome sources: {', '.join(outcome_sources) if outcome_sources else 'none'}",
            f"- provider ready: {provider_ready}",
            "",
            "repair outcome memory:",
            f"- memory artifact: `{rel(outcome_memory_md, workspace)}`",
            *recent_outcome_lines,
            "",
            "code context:",
            f"- artifact: `{rel(code_context_md, workspace)}`",
            f"- tentacle: `{code_context['tentacle']}`",
            f"- tool: `{code_context['tool']}`",
            f"- tool path: `{rel(Path(code_context['tool_path']), workspace) if code_context['tool_path'] else 'missing'}`",
            "",
            "code context excerpt:",
            code_context["prompt_excerpt"],
            "",
            "adapter context:",
            f"- artifact: `{rel(adapter_context_md, workspace)}`",
            f"- status: `{adapter_context['status']}`",
            f"- available: `{adapter_context['available'] or 'none'}`",
            f"- missing core: `{adapter_context['missing_core'] or 'none'}`",
            f"- provider env: `{adapter_context['provider_env']}`",
            f"- provider keys: `{adapter_context['provider_keys'] or 'none'}`",
            f"- desktop adapters: `{adapter_context['desktop_adapters'] or 'none'}`",
            "",
            "adapter context excerpt:",
            adapter_context["prompt_excerpt"],
            "",
            "field trajectory:",
            f"- artifact: `{rel(field_trajectory_md, workspace)}`",
            f"- field: `{field_trajectory['field']}`",
            f"- mini task: `{field_trajectory['mini_task']}`",
            f"- verifier: `{field_trajectory['verifier_status'] or 'none'}` `{field_trajectory['verifier_error'] or ''}`",
            "",
            "field trajectory excerpt:",
            field_trajectory["prompt_excerpt"],
            "",
            "repair action plan:",
            f"- plan artifact: `{rel(repair_plan_json, workspace)}`",
            f"- review boundary: {repair_plan['review_boundary']}",
            f"- checks: {', '.join(repair_plan['commands']['checks'])}",
            "",
            "artifacts:",
            f"- session: `{rel(session_json, workspace)}`",
            f"- next need: `{rel(next_need_json, workspace)}`",
            f"- commands: `{rel(command_script, workspace)}`",
            f"- draft: `{rel(draft_md, workspace)}`",
            f"- outcome memory: `{rel(outcome_memory_md, workspace)}`",
            f"- adapter context: `{rel(adapter_context_md, workspace)}`",
            f"- code context: `{rel(code_context_md, workspace)}`",
            f"- field trajectory: `{rel(field_trajectory_md, workspace)}`",
            f"- action trace: `{rel(action_trace_md, workspace)}`",
            f"- repair plan: `{rel(repair_plan_json, workspace)}`",
        ]
    )
    + "\n",
    encoding="utf-8",
)
draft = llm_draft(prompt_md.read_text(encoding="utf-8"), session, workspace)
draft_md.write_text(draft_markdown(draft, prompt_md, session_json, workspace), encoding="utf-8")
action_trace_md.write_text(
    action_trace_markdown(
        workspace,
        session_json,
        next_need_kind,
        next_need_query,
        source,
        code_context,
        adapter_context,
        field_trajectory,
        draft,
        repair_plan,
    ),
    encoding="utf-8",
)
session["draft"] = {
    "status": draft["status"],
    "path": rel(draft_md, workspace),
    "prefix": draft["prefix"],
    "model": draft.get("model", ""),
}
session["action_trace"] = rel(action_trace_md, workspace)
repair_plan["inputs"]["draft"] = rel(draft_md, workspace)
repair_plan["inputs"]["review"] = rel(review_md, workspace)
repair_plan["inputs"]["action_trace"] = rel(action_trace_md, workspace)
repair_plan_json.write_text(
    json.dumps(repair_plan, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
review_md.write_text(
    "\n".join(
        [
            "# Harness Repair Review",
            "",
            "This is the human-facing review bundle for the repair session.",
            "The clean brain still receives only compact Need and Feed.",
            "",
            f"session: `{rel(session_json, workspace)}`",
            f"target: `{code_context['tentacle']}/{code_context['tool']}`",
            f"candidate: `{candidate}`",
            f"source: `{source}`",
            f"next Need: `{next_need_kind} {next_need_query}`",
            "",
            "## Evidence",
            "",
            f"- adapter context: `{rel(adapter_context_md, workspace)}`",
            f"- action trace: `{rel(action_trace_md, workspace)}`",
            f"- field trajectory: `{rel(field_trajectory_md, workspace)}`",
            f"- code context: `{rel(code_context_md, workspace)}`",
            f"- outcome memory: `{rel(outcome_memory_md, workspace)}`",
            f"- provider draft: `{rel(draft_md, workspace)}`",
            f"- repair plan: `{rel(repair_plan_json, workspace)}`",
            f"- commands: `{rel(command_script, workspace)}`",
            "",
            "## Review Boundary",
            "",
            repair_plan["review_boundary"],
            "",
            "## Commands",
            "",
            "checks:",
            *[f"- `{command}`" for command in repair_plan["commands"]["checks"]],
            f"- grant: `{repair_plan['commands']['grant']}`",
            f"- apply: `{repair_plan['commands']['apply']}`",
            f"- score: `{repair_plan['commands']['score']}`",
            "",
            "## Draft Status",
            "",
            f"- status: `{draft['status']}`",
            f"- prefix: `{draft['prefix']}`",
            f"- model: `{draft.get('model', '') or 'none'}`",
            "",
        ]
    ),
    encoding="utf-8",
)
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
            f"review: `{rel(review_md, workspace)}`",
            f"next need: `{rel(next_need_json, workspace)}`",
            f"commands: `{rel(command_script, workspace)}`",
            f"outcome memory: `{rel(outcome_memory_md, workspace)}`",
            f"adapter context: `{rel(adapter_context_md, workspace)}`",
            f"code context: `{rel(code_context_md, workspace)}`",
            f"field trajectory: `{rel(field_trajectory_md, workspace)}`",
            f"action trace: `{rel(action_trace_md, workspace)}`",
            f"repair plan: `{rel(repair_plan_json, workspace)}`",
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
    "review": rel(review_md, workspace),
    "llm_draft_status": draft["status"],
    "llm_prefix": draft["prefix"],
    "next_need_file": rel(next_need_json, workspace),
    "command_script": rel(command_script, workspace),
    "outcome_memory": rel(outcome_memory_md, workspace),
    "adapter_context": rel(adapter_context_md, workspace),
    "code_context": rel(code_context_md, workspace),
    "field_trajectory": rel(field_trajectory_md, workspace),
    "action_trace": rel(action_trace_md, workspace),
    "repair_plan": rel(repair_plan_json, workspace),
    "repair_plan_status": repair_plan["status"],
    "code_context_tentacle": code_context["tentacle"],
    "code_context_tool": code_context["tool"],
    "code_context_tool_path": code_context["tool_path"],
    "adapter_context_status": adapter_context["status"],
    "adapter_context_available": adapter_context["available"],
    "adapter_context_missing_core": adapter_context["missing_core"],
    "adapter_context_provider_env": adapter_context["provider_env"],
    "adapter_context_provider_keys": adapter_context["provider_keys"],
    "adapter_context_desktop": adapter_context["desktop_adapters"],
    "field_trajectory_field": field_trajectory["field"],
    "field_trajectory_mini_task": field_trajectory["mini_task"],
    "field_trajectory_verifier_status": field_trajectory["verifier_status"],
    "field_trajectory_verifier_error": field_trajectory["verifier_error"],
    "outcome_command": outcome_command,
    "repair_outcome_count": str(len(repair_outcomes)),
    "repair_outcome_sources": ",".join(outcome_sources),
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
            "output": f"harness repair session: {rel(session_md, workspace)}; review: {rel(review_md, workspace)}; draft: {draft['status']}; next Need: {next_need}",
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
