#!/usr/bin/env bash
set -euo pipefail

workspace="${1:-.}"
payload_file="$(mktemp)"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
trap 'rm -f "$payload_file"' EXIT

if [ ! -t 0 ]; then
  cat > "$payload_file" || true
fi

python3 - "$script_dir" <<'PY'
import ast
import sys
from pathlib import Path

tentacle_root = Path(sys.argv[1])
syntax_errors = []
for path in sorted(tentacle_root.rglob("*.pyfrag")):
    try:
        ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    except SyntaxError as exc:
        syntax_errors.append((str(path), exc.lineno or 0, exc.msg))

if syntax_errors:
    print("repair-template pre-validation failed:", file=sys.stderr)
    for path, line, msg in syntax_errors:
        print(f"{path}:{line}: {msg}", file=sys.stderr)
    raise SystemExit(2)
PY

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


def load_field_pack_task(root, query, context):
    packs_root = root / "field-packs"
    if not packs_root.is_dir():
        return None
    context = context if isinstance(context, dict) else {}
    requested_field = slug(context.get("field_pack") or "")
    requested_task = slug(context.get("field_mini_task") or "")
    query_text = str(query or "")
    query_fold = query_text.lower()
    matches = []
    for pack_path in sorted(packs_root.glob("*/field-pack.json")):
        try:
            pack = json.loads(pack_path.read_text(encoding="utf-8"))
        except Exception:
            continue
        pack_id = slug(pack.get("id") or pack_path.parent.name)
        if requested_field and requested_field != "unknown" and pack_id != requested_field:
            continue
        aliases = [pack_id, *pack.get("aliases", [])]
        field_matches = bool(requested_field and requested_field != "unknown")
        field_matches = field_matches or any(
            str(alias or "").lower() in query_fold for alias in aliases
        )
        for task in pack.get("mini_tasks", []):
            task_id = slug(task.get("id") or "")
            if not task_id:
                continue
            task_matches = (
                requested_task
                and requested_task != "ad-hoc"
                and task_id == requested_task
            )
            task_matches = task_matches or task_id.lower() in query_fold
            if task_matches and field_matches:
                return {
                    "field": pack_id,
                    "mini_task": task_id,
                    "expected_feed": compact(task.get("expected_feed") or ""),
                    "goal": compact(task.get("goal") or ""),
                }
            if task_matches:
                matches.append({
                    "field": pack_id,
                    "mini_task": task_id,
                    "expected_feed": compact(task.get("expected_feed") or ""),
                    "goal": compact(task.get("goal") or ""),
                })
    if len(matches) == 1:
        return matches[0]
    return None


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


def base_metadata(template_path=None):
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
    }
    if template_path is not None:
        metadata["runtime_template"] = "repair-template"
        metadata["repair_template"] = rel(template_path, root)
    return metadata


def artifact_path_from_metadata(metadata):
    for key in (
        "trajectory_artifact",
        "answer",
        "checks",
        "result",
        "artifact",
        "artifact_path",
    ):
        value = metadata.get(key)
        if isinstance(value, str) and value.strip():
            return value.strip()
    return None


def normalize_field_result(result, template_path=None):
    if not isinstance(result, dict):
        return result
    metadata = result.get("metadata")
    if not isinstance(metadata, dict):
        metadata = result.get("field_metadata") if isinstance(result.get("field_metadata"), dict) else {}
    merged_metadata = base_metadata(template_path)
    merged_metadata.update(metadata)
    if isinstance(result.get("verifier_status"), str):
        merged_metadata.setdefault("verifier_status", result["verifier_status"])
    if isinstance(result.get("field_pass_evidence"), str):
        merged_metadata.setdefault("field_pass_evidence", result["field_pass_evidence"])
    status = result.get("status")
    if status not in {"satisfied", "partial", "failed", "unsupported"}:
        if result.get("satisfied") is True or merged_metadata.get("verifier_status") == "satisfied":
            status = "satisfied"
        elif result.get("satisfied") is False or merged_metadata.get("verifier_status") == "partial":
            status = "partial"
        else:
            status = "partial"
    output = result.get("output")
    if not isinstance(output, str) or not output.strip():
        output = (
            result.get("field_pass_evidence")
            or result.get("summary")
            or compact(result.get("checks") or merged_metadata)
        )
    evidence = result.get("evidence")
    if not isinstance(evidence, list) or not evidence:
        artifact = artifact_path_from_metadata(merged_metadata)
        evidence = [{
            "source": f"field-mini-task/{field}/normalized-template",
            "content": artifact or output,
            "confidence": 0.86,
            "metadata": merged_metadata,
        }]
    normalized = dict(result)
    normalized["status"] = status
    normalized["output"] = output
    normalized["evidence"] = evidence
    normalized["metadata"] = merged_metadata
    return normalized


def mark_template_result(result, template_path):
    result = normalize_field_result(result, template_path)
    if not isinstance(result, dict):
        return result
    metadata = result["metadata"]
    for evidence in result.get("evidence", []):
        if isinstance(evidence, dict):
            evidence_metadata = evidence.setdefault("metadata", {})
            evidence_metadata["runtime_template"] = "repair-template"
            evidence_metadata["repair_template"] = rel(template_path, root)
            for key, value in metadata.items():
                evidence_metadata.setdefault(key, value)
    return result


payload = read_payload(sys.argv[2])
root = Path(sys.argv[1] or ".").expanduser()
if not root.is_absolute():
    root = Path.cwd() / root
root = root.resolve()
script_dir = Path(sys.argv[3]).expanduser().resolve()
tentacle_root = script_dir.parent
if not ((root / "field-packs").is_dir() and (root / "tentacles").is_dir()):
    for candidate in [root, *root.parents, tentacle_root, *tentacle_root.parents]:
        if (candidate / "field-packs").is_dir() and (candidate / "tentacles").is_dir():
            root = candidate.resolve()
            break

need = payload.get("need") if isinstance(payload.get("need"), dict) else {}
context = need.get("context") if isinstance(need.get("context"), dict) else {}
query = compact(need.get("query") or "")
resolved_task = load_field_pack_task(root, query, context)
field = slug(context.get("field_pack") or (resolved_task or {}).get("field") or "unknown")
mini_task = slug(context.get("field_mini_task") or (resolved_task or {}).get("mini_task") or "ad-hoc")
expected_feed = compact(
    context.get("field_expected_feed") or (resolved_task or {}).get("expected_feed") or ""
)
if resolved_task:
    context = dict(context)
    context.setdefault("field_pack", field)
    context.setdefault("field_mini_task", mini_task)
    context.setdefault("field_expected_feed", expected_feed)
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
