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
    status = "satisfied"
    next_need = "review latest repair action plan"
    output = (
        f"heartbeat repair: repair_plan={rel(latest_repair_plan, root)}; "
        f"status={plan_status}; target={target_tentacle}/{target_tool}; "
        f"next={next_need_kind} {next_need_query}; review before grant/apply/score"
    )
    repair_metadata = {
        "repair_plan": rel(latest_repair_plan, root),
        "repair_plan_status": plan_status,
        "repair_plan_schema": str(repair_plan.get("schema_version") or ""),
        "repair_plan_session": str(repair_plan.get("session") or ""),
        "review": str(inputs.get("review") or ""),
        "field_trajectory": str(inputs.get("field_trajectory") or ""),
        "target_tentacle": target_tentacle,
        "target_tool": target_tool,
        "candidate": candidate,
        "review_boundary": str(repair_plan.get("review_boundary") or ""),
        "check_command": " && ".join(checks),
        "grant_command": command_value(commands, "grant"),
        "apply_command": command_value(commands, "apply"),
        "score_command": command_value(commands, "score"),
        "suggested_commands": " && ".join(suggested),
    }
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
