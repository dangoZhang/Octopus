#!/usr/bin/env bash
set -euo pipefail

workspace="${1:-.}"
payload_file="$(mktemp)"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
trap 'rm -f "$payload_file"' EXIT

if [ ! -t 0 ]; then
  cat > "$payload_file" || true
fi

python3 - "$workspace" "$payload_file" "$script_dir" <<'PY'
import datetime as dt
import json
import re
import sys
from pathlib import Path


def compact(value, limit=240):
    text = " ".join(str(value or "").split())
    return text if len(text) <= limit else text[: limit - 3] + "..."


def slug(value):
    text = re.sub(r"[^a-zA-Z0-9_.-]+", "-", str(value or "").strip()).strip("-")
    return text[:64] or "unknown"


def read_payload(path):
    raw = Path(path).read_text(encoding="utf-8", errors="replace").strip()
    if not raw:
        return {}
    try:
        return json.loads(raw)
    except json.JSONDecodeError as exc:
        return {"invalid_payload": str(exc), "raw": compact(raw, 400)}


def rel(path, root):
    try:
        return str(path.relative_to(root))
    except ValueError:
        return str(path)


def repair_template_path(root, tentacle_root, field, mini_task):
    name = f"{mini_task}.pyfrag"
    candidates = [
        tentacle_root / "repair-templates" / field / name,
        root / "repair-templates" / field / name,
        root / "tentacles" / "field-mini-task" / "repair-templates" / field / name,
    ]
    return next((path for path in candidates if path.exists()), None)


def template_error_result(error, template_path):
    metadata = {
        "runtime": "shell",
        "contract": "octopus-json-v1",
        "tool": "run_field_mini_task",
        "tentacle": "field-mini-task",
        "field_pack": field,
        "field_mini_task": mini_task,
        "field_expected_feed": expected_feed,
        "field_session": rel(session, root),
        "task_record": rel(session / "TASK.json", root),
        "prompt": rel(session / "PROMPT.md", root),
        "feed_draft": rel(session / "FEED.md", root),
        "repair_template": rel(template_path, root),
        "verifier_status": "failed",
        "error_category": "repair_template_error",
    }
    output = f"field-mini-task template failed for {field}/{mini_task}: {compact(error, 360)}"
    return {
        "status": "failed",
        "output": output,
        "evidence": [{
            "source": "field-mini-task/repair-template-error",
            "content": output,
            "confidence": 0.9,
            "metadata": metadata,
        }],
        "metadata": metadata,
    }


def template_missing_result(template_path):
    metadata = {
        "runtime": "shell",
        "contract": "octopus-json-v1",
        "tool": "run_field_mini_task",
        "tentacle": "field-mini-task",
        "field_pack": field,
        "field_mini_task": mini_task,
        "field_expected_feed": expected_feed,
        "field_session": rel(session, root),
        "task_record": rel(session / "TASK.json", root),
        "prompt": rel(session / "PROMPT.md", root),
        "feed_draft": rel(session / "FEED.md", root),
        "repair_template": rel(template_path, root),
        "verifier_status": "failed",
        "error_category": "repair_template_no_result",
    }
    output = f"field-mini-task template produced no Feed for {field}/{mini_task}"
    return {
        "status": "failed",
        "output": output,
        "evidence": [{
            "source": "field-mini-task/repair-template-no-result",
            "content": output,
            "confidence": 0.9,
            "metadata": metadata,
        }],
        "metadata": metadata,
    }


def mark_template_result(result, template_path):
    if not isinstance(result, dict):
        return result
    metadata = result.setdefault("metadata", {})
    metadata["runtime_template"] = "repair-template"
    metadata["repair_template"] = rel(template_path, root)
    for evidence in result.get("evidence", []):
        if isinstance(evidence, dict):
            evidence_metadata = evidence.setdefault("metadata", {})
            evidence_metadata["runtime_template"] = "repair-template"
            evidence_metadata["repair_template"] = rel(template_path, root)
    return result


payload = read_payload(sys.argv[2])
root = Path(sys.argv[1] or ".").expanduser()
if not root.is_absolute():
    root = Path.cwd() / root
root = root.resolve()
script_dir = Path(sys.argv[3]).expanduser().resolve()
tentacle_root = script_dir.parent

need = payload.get("need") if isinstance(payload.get("need"), dict) else {}
context = need.get("context") if isinstance(need.get("context"), dict) else {}
field = slug(context.get("field_pack") or "unknown")
mini_task = slug(context.get("field_mini_task") or "ad-hoc")
expected_feed = compact(context.get("field_expected_feed") or "")
query = compact(need.get("query") or "")
now = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
session = root / ".octopus" / "field-mini-task" / field / f"{now}-{mini_task}"
session.mkdir(parents=True, exist_ok=True)

task_record = {
    "schema_version": "octopus-field-mini-task-v1",
    "field": field,
    "mini_task": mini_task,
    "need_kind": need.get("kind") or "",
    "need_query": query,
    "expected_feed": expected_feed,
    "context": context,
    "created_at": now,
}
(session / "TASK.json").write_text(json.dumps(task_record, indent=2, ensure_ascii=True) + "\n", encoding="utf-8")
(session / "PROMPT.md").write_text(
    "\n".join([
        "# Field Mini Task",
        "",
        f"Field: `{field}`",
        f"Mini task: `{mini_task}`",
        "",
        "## Need",
        query or "(empty)",
        "",
        "## Expected Feed",
        expected_feed or "(not provided)",
        "",
        "## Harness Work",
        "Evolve this tentacle's prompt, metadata, or runtime code so future runs can observe, act, verify, and return compact Feed for this field.",
        "Keep the clean brain context limited to Goal, Mem, Need, and Feed.",
        "",
    ]),
    encoding="utf-8",
)
(session / "FEED.md").write_text(
    "\n".join([
        "# Current Feed",
        "",
        "Status: partial",
        "",
        "This generic harness recorded the mini task and preserved the evidence surface. It has not yet evolved field-specific execution logic.",
        "",
    ]),
    encoding="utf-8",
)

field_result = None
template_path = repair_template_path(root, tentacle_root, field, mini_task)
if template_path is not None:
    try:
        source = template_path.read_text(encoding="utf-8", errors="replace")
        exec(compile(source, str(template_path), "exec"), globals(), globals())
    except Exception as exc:
        print(json.dumps(template_error_result(str(exc), template_path), ensure_ascii=True))
        raise SystemExit(0)
    if field_result is not None:
        print(json.dumps(mark_template_result(field_result, template_path), ensure_ascii=True))
        raise SystemExit(0)
    print(json.dumps(template_missing_result(template_path), ensure_ascii=True))
    raise SystemExit(0)

if field_result is not None:
    print(json.dumps(field_result, ensure_ascii=True))
    raise SystemExit(0)

next_query = f"evolve field-mini-task harness for {field} after {mini_task}"
output = (
    f"field-mini-task: recorded {field}/{mini_task}; "
    f"session={rel(session, root)}; needs evolved field execution before verifier pass"
)
metadata = {
    "runtime": "shell",
    "contract": "octopus-json-v1",
    "tool": "run_field_mini_task",
    "tentacle": "field-mini-task",
    "field_pack": field,
    "field_mini_task": mini_task,
    "field_expected_feed": expected_feed,
    "field_session": rel(session, root),
    "task_record": rel(session / "TASK.json", root),
    "prompt": rel(session / "PROMPT.md", root),
    "feed_draft": rel(session / "FEED.md", root),
    "next_need_kind": "execute",
    "next_need_query": next_query,
    "verifier_status": "partial",
    "error_category": "field_execution_not_evolved",
}

print(json.dumps({
    "status": "partial",
    "output": output,
    "evidence": [{
        "source": "field-mini-task",
        "content": output,
        "confidence": 0.72,
        "metadata": metadata,
    }],
    "metadata": metadata,
}, ensure_ascii=True))
PY
