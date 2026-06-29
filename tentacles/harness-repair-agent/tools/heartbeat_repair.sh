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


def newest(paths):
    existing = [path for path in paths if path.exists()]
    if not existing:
        return None
    return max(existing, key=lambda path: path.stat().st_mtime)


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


def resolve_artifact(root, value):
    value = str(value or "").strip()
    if not value:
        return None
    path = Path(value).expanduser()
    if not path.is_absolute():
        path = root / path
    try:
        resolved = path.resolve()
        resolved.relative_to(root)
    except Exception:
        return None
    return resolved


def clean_markdown_value(value):
    text = str(value or "").strip()
    if text.startswith("`") and text.endswith("`") and len(text) >= 2:
        text = text[1:-1].strip()
    return "" if text in {"none", "missing", "not provided"} else text


def clean_adapter_value(value, keep_missing=False):
    text = str(value or "").strip()
    if text.startswith("`") and text.endswith("`") and len(text) >= 2:
        text = text[1:-1].strip()
    empty_values = {"none", "not provided"}
    if not keep_missing:
        empty_values.add("missing")
    return "" if text in empty_values else text


def field_trajectory_metadata(root, value, target):
    path = resolve_artifact(root, value)
    metadata = {
        "field_trajectory_field": "",
        "field_trajectory_mini_task": "",
        "field_trajectory_verifier_status": "",
        "field_trajectory_verifier_error": "",
        "field_trajectory_preview": "",
    }
    if isinstance(target, dict):
        metadata.update({
            "field_trajectory_field": clean_markdown_value(target.get("field")),
            "field_trajectory_mini_task": clean_markdown_value(target.get("mini_task")),
            "field_trajectory_verifier_status": clean_markdown_value(target.get("verifier_status")),
            "field_trajectory_verifier_error": clean_markdown_value(target.get("verifier_error")),
        })
    if not path or not path.exists():
        return metadata
    text = path.read_text(encoding="utf-8", errors="replace")
    metadata["field_trajectory_preview"] = compact(text, 600)
    for line in text.splitlines():
        key, _, raw = line.partition(":")
        if not raw:
            continue
        key = key.strip()
        value = clean_markdown_value(raw)
        if key == "field" and not metadata["field_trajectory_field"]:
            metadata["field_trajectory_field"] = value
        elif key == "mini_task" and not metadata["field_trajectory_mini_task"]:
            metadata["field_trajectory_mini_task"] = value
    return metadata


def adapter_context_metadata(root, value, target):
    path = resolve_artifact(root, value)
    metadata = {
        "adapter_context_status": "",
        "adapter_context_available": "",
        "adapter_context_missing_core": "",
        "adapter_context_provider_env": "",
        "adapter_context_provider_keys": "",
        "adapter_context_desktop": "",
        "adapter_context_preview": "",
    }
    if isinstance(target, dict):
        metadata.update({
            "adapter_context_status": clean_adapter_value(target.get("status")),
            "adapter_context_available": clean_adapter_value(target.get("available")),
            "adapter_context_missing_core": clean_adapter_value(target.get("missing_core")),
            "adapter_context_provider_env": clean_adapter_value(target.get("provider_env"), keep_missing=True),
            "adapter_context_provider_keys": clean_adapter_value(target.get("provider_keys")),
            "adapter_context_desktop": clean_adapter_value(target.get("desktop_adapters")),
        })
    if not path or not path.exists():
        return metadata
    text = path.read_text(encoding="utf-8", errors="replace")
    metadata["adapter_context_preview"] = compact(text, 600)
    key_map = {
        "status": "adapter_context_status",
        "available": "adapter_context_available",
        "missing_core": "adapter_context_missing_core",
        "provider_env": "adapter_context_provider_env",
        "provider_keys": "adapter_context_provider_keys",
        "desktop_adapters": "adapter_context_desktop",
    }
    for line in text.splitlines():
        key, _, raw = line.partition(":")
        if not raw:
            continue
        target_key = key_map.get(key.strip())
        if target_key:
            metadata[target_key] = clean_adapter_value(
                raw,
                keep_missing=target_key == "adapter_context_provider_env",
            )
    return metadata


def repair_draft_metadata(root, value):
    path = resolve_artifact(root, value)
    metadata = {
        "draft_status": "",
        "draft_prefix": "",
        "draft_model": "",
        "draft_preview": "",
    }
    if not path or not path.exists():
        return metadata
    text = path.read_text(encoding="utf-8", errors="replace")
    metadata["draft_preview"] = compact(text, 600)
    key_map = {
        "status": "draft_status",
        "prefix": "draft_prefix",
        "model": "draft_model",
    }
    for line in text.splitlines():
        key, _, raw = line.partition(":")
        if not raw:
            continue
        target_key = key_map.get(key.strip().lower())
        if target_key and not metadata[target_key]:
            metadata[target_key] = clean_markdown_value(raw)
    return metadata


def repair_outcome_metadata(root, latest_repair_plan, repair_plan):
    session_path = resolve_artifact(root, repair_plan.get("session"))
    if not session_path and latest_repair_plan:
        session_path = latest_repair_plan.parent / "SESSION.json"
    if not session_path:
        return {}
    outcome_path = session_path.parent / "OUTCOME.md"
    if not outcome_path.exists():
        return {}
    text = outcome_path.read_text(encoding="utf-8", errors="replace")
    outcome_status = ""
    for line in text.splitlines():
        key, _, raw = line.partition(":")
        if key.strip().lower() == "status":
            outcome_status = clean_markdown_value(raw)
            break
    if not outcome_status:
        outcome_status = "reviewed"
    return {
        "outcome": rel(outcome_path, root),
        "outcome_status": outcome_status,
        "outcome_summary": compact(text, 500),
    }


def as_list(value):
    if isinstance(value, list):
        return [str(item) for item in value if str(item).strip()]
    if value:
        return [str(value)]
    return []


def command_value(commands, key):
    values = as_list(commands.get(key))
    return " && ".join(values)


def plan_next_need(plan):
    next_need = plan.get("next_need") or {}
    if isinstance(next_need, dict):
        kind = str(next_need.get("kind") or "verify").strip() or "verify"
        query = str(next_need.get("query") or "review repair action plan").strip()
        return kind, query
    text = compact(next_need or "review repair action plan", 240)
    first, _, rest = text.partition(" ")
    allowed = {"observe", "verify", "reproduce", "compare", "remember", "forget", "recall", "execute"}
    if first in allowed and rest.strip():
        return first, rest.strip()
    return "verify", text


payload = read_payload()
root = resolve_workspace(sys.argv[1] if len(sys.argv) > 1 else ".", payload)
evolution_root = root / ".octopus/evolution"
repair_root = root / ".octopus/harness-repair"
tentacles = []
plans = []
recommendations = []

if evolution_root.exists():
    for tentacle_dir in sorted(path for path in evolution_root.iterdir() if path.is_dir()):
        tentacles.append(tentacle_dir.name)
        proposal = tentacle_dir / "PROPOSAL.md"
        recommendation = tentacle_dir / "RECOMMENDATION.md"
        apply_dir = tentacle_dir / "apply"
        latest_apply = newest(list(apply_dir.glob("*")) if apply_dir.exists() else [])
        if proposal.exists():
            plans.append(str(proposal.relative_to(root)))
        if recommendation.exists():
            recommendations.append(str(recommendation.relative_to(root)))
        if latest_apply:
            plans.append(str(latest_apply.relative_to(root)))

latest_repair_plan = newest(
    list(repair_root.glob("*/REPAIR_PLAN.json")) if repair_root.exists() else []
)
repair_plan = load_json(latest_repair_plan) if latest_repair_plan else {}
repair_metadata = {}

if latest_repair_plan:
    commands = repair_plan.get("commands") if isinstance(repair_plan.get("commands"), dict) else {}
    inputs = repair_plan.get("inputs") if isinstance(repair_plan.get("inputs"), dict) else {}
    checks = as_list(commands.get("checks"))
    suggested = as_list(commands.get("suggested"))
    next_need_kind, next_need_query = plan_next_need(repair_plan)
    plan_status = str(repair_plan.get("status") or "review_required")
    target_tentacle = str(repair_plan.get("target_tentacle") or "unknown")
    target_tool = str(repair_plan.get("target_tool") or "unknown")
    candidate = str(repair_plan.get("candidate") or "none")
    field_trajectory = str(inputs.get("field_trajectory") or "")
    adapter_context = str(inputs.get("adapter_context") or "")
    draft = str(inputs.get("draft") or "")
    code_context = str(inputs.get("code_context") or "")
    draft_metadata = repair_draft_metadata(root, draft)
    adapter_metadata = adapter_context_metadata(
        root,
        adapter_context,
        repair_plan.get("adapter_context_target") or {},
    )
    field_metadata = field_trajectory_metadata(
        root,
        field_trajectory,
        repair_plan.get("field_trajectory_target") or {},
    )
    field_label = "/".join(
        item for item in [
            field_metadata.get("field_trajectory_field", ""),
            field_metadata.get("field_trajectory_mini_task", ""),
        ]
        if item
    )
    outcome_metadata = repair_outcome_metadata(root, latest_repair_plan, repair_plan)
    has_outcome = bool(outcome_metadata.get("outcome"))
    adapter_status = adapter_metadata.get("adapter_context_status", "")
    adapter_missing_core = adapter_metadata.get("adapter_context_missing_core", "")
    adapter_blocked = (
        not has_outcome
        and (
            adapter_status in {"failed", "partial"}
            or bool(adapter_missing_core)
        )
    )
    if has_outcome:
        outcome_status = outcome_metadata.get("outcome_status", "reviewed")
        plan_status = f"outcome_{outcome_status}"
        if outcome_status in {"failed", "partial"}:
            status = "partial"
            next_need = f"continue repair after {outcome_status} outcome"
            next_need_kind = "execute"
            next_need_query = f"octopus repair . after {outcome_status} repair outcome"
        else:
            status = "satisfied"
            next_need = "retain reviewed repair outcome"
            next_need_kind = "verify"
            next_need_query = "harness repair outcome retained"
        output = (
            f"heartbeat repair: repair_plan={rel(latest_repair_plan, root)}; "
            f"outcome={outcome_metadata.get('outcome')}; status={outcome_status}; "
            f"target={target_tentacle}/{target_tool}; next={next_need_kind} {next_need_query}"
        )
        checks = []
        suggested = []
    elif adapter_blocked:
        plan_status = "adapter_blocked"
        status = "partial"
        missing = adapter_missing_core or "core adapters"
        next_need = f"repair adapter context: {missing}"
        next_need_kind = "execute"
        next_need_query = f"install missing core adapters: {missing}"
        output = (
            f"heartbeat repair: repair_plan={rel(latest_repair_plan, root)}; "
            f"adapter_status={adapter_status or 'unknown'}; missing_core={missing}; "
            f"target={target_tentacle}/{target_tool}; next={next_need_kind} {next_need_query}"
        )
        checks = []
        suggested = []
    else:
        status = "satisfied"
        next_need = "review latest repair action plan"
        output = (
            f"heartbeat repair: repair_plan={rel(latest_repair_plan, root)}; "
            f"status={plan_status}; target={target_tentacle}/{target_tool}; "
            f"next={next_need_kind} {next_need_query}; "
            f"field={field_label or 'none'}; review before grant/apply/score"
        )
    repair_metadata = {
        "repair_plan": rel(latest_repair_plan, root),
        "repair_plan_status": plan_status,
        "repair_plan_schema": str(repair_plan.get("schema_version") or ""),
        "repair_plan_session": str(repair_plan.get("session") or ""),
        "review": str(inputs.get("review") or ""),
        "draft": draft,
        "code_context": code_context,
        "adapter_context": adapter_context,
        "field_trajectory": field_trajectory,
        "target_tentacle": target_tentacle,
        "target_tool": target_tool,
        "candidate": candidate,
        "review_boundary": str(repair_plan.get("review_boundary") or ""),
        "check_command": " && ".join(checks),
        "adapter_blocker": adapter_missing_core if adapter_blocked else "",
        "grant_command": "" if has_outcome or adapter_blocked else command_value(commands, "grant"),
        "apply_command": "" if has_outcome or adapter_blocked else command_value(commands, "apply"),
        "score_command": "" if has_outcome or adapter_blocked else command_value(commands, "score"),
        "suggested_commands": " && ".join(suggested),
    }
    repair_metadata.update(draft_metadata)
    repair_metadata.update(adapter_metadata)
    repair_metadata.update(field_metadata)
    repair_metadata.update(outcome_metadata)
elif not tentacles:
    status = "partial"
    next_need = "run beat after failed check or scored Feed"
    next_need_kind = "execute"
    next_need_query = "octopus beat 200 after failed check or scored Feed"
    output = "heartbeat repair: no evolution artifacts yet; collect failed or partial feedback, then run octopus beat 200"
else:
    status = "satisfied"
    next_need = "review harness apply plan"
    next_need_kind = "verify"
    next_need_query = "review harness apply plan"
    output = (
        f"heartbeat repair: tentacles={','.join(tentacles)}; "
        f"plans={len(plans)}; recommendations={len(recommendations)}; "
        "next=grant harness:write only after reviewing apply artifacts"
    )

metadata = {
    "workspace": str(root),
    "evolution_tentacles": ",".join(tentacles),
    "plans": ",".join(plans),
    "recommendations": ",".join(recommendations),
    "next_need": next_need,
    "next_need_kind": next_need_kind,
    "next_need_query": next_need_query,
    "grant_boundary": "octopus oauth octopus evolve:<tentacle> harness:write",
}
metadata.update(repair_metadata)

print(json.dumps({
    "status": status,
    "output": output,
    "evidence": [{
        "source": "heartbeat_repair",
        "content": output,
        "confidence": 0.82,
        "metadata": metadata,
    }],
    "metadata": metadata,
}, ensure_ascii=True))
PY
