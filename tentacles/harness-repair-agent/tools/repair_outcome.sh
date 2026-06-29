#!/usr/bin/env bash
set -euo pipefail

payload_file="$(mktemp)"
trap 'rm -f "$payload_file"' EXIT
if [ ! -t 0 ]; then
  cat > "$payload_file" || true
fi

python3 - "$payload_file" "$@" <<'PY'
import json
import shlex
import sys
import time
from pathlib import Path


STATUSES = {"satisfied", "partial", "failed", "reviewed"}


def compact(value, limit=220):
    text = " ".join(str(value).split())
    return text if len(text) <= limit else text[: limit - 3] + "..."


def load_payload(path):
    try:
        raw = Path(path).read_text(encoding="utf-8", errors="replace").strip()
    except OSError:
        return {}
    if not raw:
        return {}
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        return {"raw": raw}


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


def newest_session(root):
    sessions = sorted((root / ".octopus" / "harness-repair").glob("*/SESSION.json"))
    if not sessions:
        return None
    return max(sessions, key=lambda path: path.stat().st_mtime)


def resolve_session_artifact(workspace, session_path, value, fallback_name):
    if value:
        path = Path(str(value)).expanduser()
        if not path.is_absolute():
            path = workspace / path
        if path.exists():
            return path
    fallback = session_path.parent / fallback_name
    return fallback if fallback.exists() else None


def action_trace_summary(workspace, session_path, session):
    path = resolve_session_artifact(
        workspace,
        session_path,
        session.get("action_trace_json"),
        "ACTION_TRACE.json",
    )
    data = load_json(path) if path else {}
    next_need = data.get("next_need") if isinstance(data.get("next_need"), dict) else {}
    repair_recall = data.get("repair_recall") if isinstance(data.get("repair_recall"), dict) else {}
    repair_lessons = data.get("repair_lessons") if isinstance(data.get("repair_lessons"), dict) else {}
    harness_adaptation = data.get("harness_adaptation") if isinstance(data.get("harness_adaptation"), dict) else {}
    harness_environment_drift = data.get("harness_environment_drift") if isinstance(data.get("harness_environment_drift"), dict) else {}
    failed_stage = ""
    for stage in data.get("stages") if isinstance(data.get("stages"), list) else []:
        if not isinstance(stage, dict):
            continue
        if str(stage.get("status") or "") in {"failed", "missing_context"}:
            failed_stage = str(stage.get("action") or "")
            break
    return {
        "json": rel(path, workspace) if path else "",
        "status": str(data.get("status") or "unknown"),
        "stage_count": str(data.get("stage_count") or ""),
        "last_action": str(data.get("last_action") or ""),
        "repair_hint": str(data.get("repair_hint") or ""),
        "failed_stage": failed_stage,
        "next_need_kind": str(next_need.get("kind") or ""),
        "next_need_query": str(next_need.get("query") or ""),
        "recall_count": str(repair_recall.get("match_count") or ""),
        "recall_top_status": str(repair_recall.get("top_status") or ""),
        "recall_top_score": str(repair_recall.get("top_score") or ""),
        "recall_top_reasons": str(repair_recall.get("top_reasons") or ""),
        "recall_top_summary": str(repair_recall.get("top_summary") or ""),
        "recall_top_action_hint": str(repair_recall.get("top_action_hint") or ""),
        "lesson_count": str(repair_lessons.get("lesson_count") or ""),
        "lesson_reuse_count": str(repair_lessons.get("reuse_count") or ""),
        "lesson_avoid_count": str(repair_lessons.get("avoid_count") or ""),
        "lesson_top_reuse": str(repair_lessons.get("top_reuse") or ""),
        "lesson_top_avoid": str(repair_lessons.get("top_avoid") or ""),
        "harness_environment_drift_status": str(harness_environment_drift.get("status") or data.get("harness_environment_drift_status") or ""),
        "harness_environment_drift_detail": str(harness_environment_drift.get("detail") or data.get("harness_environment_drift_detail") or ""),
        "harness_environment_drift_history_count": str(harness_environment_drift.get("history_count") or data.get("harness_environment_drift_history_count") or ""),
        "harness_environment_drift_gained_count": str(harness_environment_drift.get("gained_count") or data.get("harness_environment_drift_gained_count") or ""),
        "harness_environment_drift_lost_count": str(harness_environment_drift.get("lost_count") or data.get("harness_environment_drift_lost_count") or ""),
        "harness_environment_drift_next_need_kind": str(harness_environment_drift.get("next_need_kind") or data.get("harness_environment_drift_next_need_kind") or ""),
        "harness_environment_drift_next_need_query": str(harness_environment_drift.get("next_need_query") or data.get("harness_environment_drift_next_need_query") or ""),
        "harness_adaptation_status": str(harness_adaptation.get("status") or ""),
        "harness_adaptation_focus": str(harness_adaptation.get("focus") or ""),
        "harness_adaptation_next_need_kind": str(harness_adaptation.get("next_need_kind") or ""),
        "harness_adaptation_next_need_query": str(harness_adaptation.get("next_need_query") or ""),
    }


def parse_input(payload, args):
    query = payload.get("need", {}).get("query", "")
    tokens = list(args)
    if not tokens and query:
        try:
            tokens = shlex.split(query)
        except ValueError:
            tokens = query.split()
    tokens = [token for token in tokens if token]

    workspace = Path(".")
    if tokens:
        first = Path(tokens[0]).expanduser()
        if first.exists() and first.is_dir():
            workspace = first
            tokens = tokens[1:]
    if not workspace.is_absolute():
        workspace = (Path.cwd() / workspace).resolve()

    while tokens and tokens[0].lower() in {"repair", "outcome", "score", "review", "session"}:
        tokens = tokens[1:]

    session_path = None
    if tokens:
        candidate = Path(tokens[0]).expanduser()
        resolved = candidate if candidate.is_absolute() else workspace / candidate
        if resolved.exists() and resolved.is_file():
            session_path = resolved
            tokens = tokens[1:]
    if session_path is None:
        session_path = newest_session(workspace)

    while tokens and tokens[0].lower() in {"repair", "outcome", "score", "review", "session"}:
        tokens = tokens[1:]

    status = None
    if tokens and tokens[0].lower() in STATUSES:
        status = tokens[0].lower()
        tokens = tokens[1:]
    summary = " ".join(tokens).strip()
    if not summary:
        summary = compact(payload.get("need", {}).get("query", "")) or "repair outcome recorded"
    return workspace, session_path, status, summary


payload = load_payload(sys.argv[1])
workspace, session_path, outcome_status, summary = parse_input(payload, sys.argv[2:])
root = workspace / ".octopus" / "harness-repair"
root.mkdir(parents=True, exist_ok=True)

if session_path is None or not session_path.exists():
    metadata = {
        "workspace": str(workspace),
        "next_need_kind": "execute",
        "next_need_query": "octopus repair .",
    }
    print(json.dumps({
        "status": "partial",
        "output": "repair outcome needs a SESSION.json path or an existing repair session",
        "evidence": [{
            "source": "harness-repair-agent/repair_outcome",
            "content": "missing SESSION.json",
            "confidence": 0.7,
            "metadata": metadata,
        }],
        "metadata": metadata,
    }, ensure_ascii=True))
    raise SystemExit(0)

if outcome_status is None:
    metadata = {
        "workspace": str(workspace),
        "session": rel(session_path, workspace),
        "next_need_kind": "verify",
        "next_need_query": f"repair outcome {rel(session_path, workspace)} satisfied|partial|failed",
    }
    print(json.dumps({
        "status": "partial",
        "output": "repair outcome needs status: satisfied, partial, failed, or reviewed",
        "evidence": [{
            "source": "harness-repair-agent/repair_outcome",
            "content": "missing outcome status",
            "confidence": 0.72,
            "metadata": metadata,
        }],
        "metadata": metadata,
    }, ensure_ascii=True))
    raise SystemExit(0)

session = load_json(session_path)
action_trace = action_trace_summary(workspace, session_path, session)
record = {
    "schema_version": "octopus-harness-repair-outcome-v1",
    "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    "workspace": str(workspace),
    "session": rel(session_path, workspace),
    "target_tentacle": str(session.get("target_tentacle") or "unknown"),
    "candidate": str(session.get("candidate") or "none"),
    "source": str(session.get("source") or "none"),
    "next_need": str(session.get("next_need") or ""),
    "draft_status": str((session.get("draft") or {}).get("status") or "unknown"),
    "action_trace_json": action_trace["json"],
    "action_trace_status": action_trace["status"],
    "action_trace_stage_count": action_trace["stage_count"],
    "action_trace_last_action": action_trace["last_action"],
    "action_trace_repair_hint": action_trace["repair_hint"],
    "action_trace_failed_stage": action_trace["failed_stage"],
    "action_trace_recall_count": action_trace["recall_count"],
    "action_trace_recall_top_status": action_trace["recall_top_status"],
    "action_trace_recall_top_score": action_trace["recall_top_score"],
    "action_trace_recall_top_reasons": action_trace["recall_top_reasons"],
    "action_trace_recall_top_summary": action_trace["recall_top_summary"],
    "action_trace_recall_top_action_hint": action_trace["recall_top_action_hint"],
    "action_trace_lesson_count": action_trace["lesson_count"],
    "action_trace_lesson_reuse_count": action_trace["lesson_reuse_count"],
    "action_trace_lesson_avoid_count": action_trace["lesson_avoid_count"],
    "action_trace_lesson_top_reuse": action_trace["lesson_top_reuse"],
    "action_trace_lesson_top_avoid": action_trace["lesson_top_avoid"],
    "action_trace_harness_environment_drift_status": action_trace["harness_environment_drift_status"],
    "action_trace_harness_environment_drift_detail": action_trace["harness_environment_drift_detail"],
    "action_trace_harness_environment_drift_history_count": action_trace["harness_environment_drift_history_count"],
    "action_trace_harness_environment_drift_gained_count": action_trace["harness_environment_drift_gained_count"],
    "action_trace_harness_environment_drift_lost_count": action_trace["harness_environment_drift_lost_count"],
    "action_trace_harness_environment_drift_next_need_kind": action_trace["harness_environment_drift_next_need_kind"],
    "action_trace_harness_environment_drift_next_need_query": action_trace["harness_environment_drift_next_need_query"],
    "action_trace_harness_adaptation_status": action_trace["harness_adaptation_status"],
    "action_trace_harness_adaptation_focus": action_trace["harness_adaptation_focus"],
    "action_trace_harness_adaptation_next_need_kind": action_trace["harness_adaptation_next_need_kind"],
    "action_trace_harness_adaptation_next_need_query": action_trace["harness_adaptation_next_need_query"],
    "action_trace": action_trace,
    "outcome_status": outcome_status,
    "summary": compact(summary, 500),
}
outcomes_file = root / "outcomes.jsonl"
with outcomes_file.open("a", encoding="utf-8") as handle:
    handle.write(json.dumps(record, ensure_ascii=True, sort_keys=True) + "\n")

outcome_md = session_path.parent / "OUTCOME.md"
outcome_md.write_text(
    "\n".join([
        "# Harness Repair Outcome",
        "",
        f"status: `{outcome_status}`",
        f"session: `{rel(session_path, workspace)}`",
        f"target: `{record['target_tentacle']}`",
        f"candidate: `{record['candidate']}`",
        f"draft_status: `{record['draft_status']}`",
        f"action_trace_json: `{record['action_trace_json'] or 'missing'}`",
        f"action_trace_status: `{record['action_trace_status']}`",
        f"action_trace_stages: `{record['action_trace_stage_count'] or 'unknown'}`",
        f"action_trace_last: `{record['action_trace_last_action'] or 'unknown'}`",
        f"action_trace_recall: matches=`{record['action_trace_recall_count'] or '0'}` top=`{record['action_trace_recall_top_status'] or 'none'}` reasons=`{record['action_trace_recall_top_reasons'] or 'none'}`",
        f"action_trace_lessons: count=`{record['action_trace_lesson_count'] or '0'}` reuse=`{record['action_trace_lesson_reuse_count'] or '0'}` avoid=`{record['action_trace_lesson_avoid_count'] or '0'}` top_reuse=`{record['action_trace_lesson_top_reuse'] or 'none'}` top_avoid=`{record['action_trace_lesson_top_avoid'] or 'none'}`",
        f"action_trace_harness_environment_drift: status=`{record['action_trace_harness_environment_drift_status'] or 'none'}` detail=`{record['action_trace_harness_environment_drift_detail'] or 'none'}` history=`{record['action_trace_harness_environment_drift_history_count'] or '0'}` next=`{record['action_trace_harness_environment_drift_next_need_kind'] or 'verify'} {record['action_trace_harness_environment_drift_next_need_query'] or ''}`",
        f"action_trace_harness_adaptation: status=`{record['action_trace_harness_adaptation_status'] or 'none'}` focus=`{record['action_trace_harness_adaptation_focus'] or 'none'}` next=`{record['action_trace_harness_adaptation_next_need_kind'] or 'verify'} {record['action_trace_harness_adaptation_next_need_query'] or ''}`",
        "",
        summary,
        "",
        f"journal: `{rel(outcomes_file, workspace)}`",
    ]) + "\n",
    encoding="utf-8",
)

next_need_kind = "execute" if outcome_status in {"partial", "failed"} else "verify"
next_need_query = (
    f"octopus beat 200 after repair outcome {outcome_status}"
    if outcome_status in {"partial", "failed"}
    else "harness repair outcome retained"
)
metadata = {
    "workspace": str(workspace),
    "session": rel(session_path, workspace),
    "outcome": rel(outcome_md, workspace),
    "outcomes_file": rel(outcomes_file, workspace),
    "outcome_status": outcome_status,
    "target_tentacle": record["target_tentacle"],
    "candidate": record["candidate"],
    "action_trace_json": record["action_trace_json"],
    "action_trace_status": record["action_trace_status"],
    "action_trace_stage_count": record["action_trace_stage_count"],
    "action_trace_last_action": record["action_trace_last_action"],
    "action_trace_recall_count": record["action_trace_recall_count"],
    "action_trace_recall_top_status": record["action_trace_recall_top_status"],
    "action_trace_recall_top_reasons": record["action_trace_recall_top_reasons"],
    "action_trace_lesson_count": record["action_trace_lesson_count"],
    "action_trace_lesson_reuse_count": record["action_trace_lesson_reuse_count"],
    "action_trace_lesson_avoid_count": record["action_trace_lesson_avoid_count"],
    "action_trace_harness_environment_drift_status": record["action_trace_harness_environment_drift_status"],
    "action_trace_harness_environment_drift_detail": record["action_trace_harness_environment_drift_detail"],
    "action_trace_harness_environment_drift_history_count": record["action_trace_harness_environment_drift_history_count"],
    "action_trace_harness_environment_drift_gained_count": record["action_trace_harness_environment_drift_gained_count"],
    "action_trace_harness_environment_drift_lost_count": record["action_trace_harness_environment_drift_lost_count"],
    "action_trace_harness_environment_drift_next_need_kind": record["action_trace_harness_environment_drift_next_need_kind"],
    "action_trace_harness_environment_drift_next_need_query": record["action_trace_harness_environment_drift_next_need_query"],
    "action_trace_harness_adaptation_status": record["action_trace_harness_adaptation_status"],
    "action_trace_harness_adaptation_focus": record["action_trace_harness_adaptation_focus"],
    "next_need_kind": next_need_kind,
    "next_need_query": next_need_query,
}
print(json.dumps({
    "status": "satisfied",
    "output": f"repair outcome recorded: {outcome_status}; session: {rel(session_path, workspace)}",
    "evidence": [{
        "source": "harness-repair-agent/repair_outcome",
        "content": json.dumps(record, ensure_ascii=True, sort_keys=True),
        "confidence": 0.9,
        "metadata": metadata,
    }],
    "metadata": metadata,
}, ensure_ascii=True))
PY
