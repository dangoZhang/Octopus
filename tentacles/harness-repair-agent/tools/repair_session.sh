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
    action_trace = item.get("action_trace") if isinstance(item.get("action_trace"), dict) else {}
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
        "action_trace_json": str(item.get("action_trace_json") or action_trace.get("json") or ""),
        "action_trace_status": str(item.get("action_trace_status") or action_trace.get("status") or ""),
        "action_trace_stage_count": str(item.get("action_trace_stage_count") or action_trace.get("stage_count") or ""),
        "action_trace_last_action": str(item.get("action_trace_last_action") or action_trace.get("last_action") or ""),
        "action_trace_repair_hint": str(item.get("action_trace_repair_hint") or action_trace.get("repair_hint") or ""),
        "action_trace_failed_stage": str(item.get("action_trace_failed_stage") or action_trace.get("failed_stage") or ""),
        "action_trace_recall_count": str(item.get("action_trace_recall_count") or action_trace.get("recall_count") or ""),
        "action_trace_recall_top_status": str(item.get("action_trace_recall_top_status") or action_trace.get("recall_top_status") or ""),
        "action_trace_recall_top_score": str(item.get("action_trace_recall_top_score") or action_trace.get("recall_top_score") or ""),
        "action_trace_recall_top_reasons": str(item.get("action_trace_recall_top_reasons") or action_trace.get("recall_top_reasons") or ""),
        "action_trace_recall_top_summary": str(item.get("action_trace_recall_top_summary") or action_trace.get("recall_top_summary") or ""),
        "action_trace_lesson_count": str(item.get("action_trace_lesson_count") or action_trace.get("lesson_count") or ""),
        "action_trace_lesson_reuse_count": str(item.get("action_trace_lesson_reuse_count") or action_trace.get("lesson_reuse_count") or ""),
        "action_trace_lesson_avoid_count": str(item.get("action_trace_lesson_avoid_count") or action_trace.get("lesson_avoid_count") or ""),
        "action_trace_lesson_top_reuse": str(item.get("action_trace_lesson_top_reuse") or action_trace.get("lesson_top_reuse") or ""),
        "action_trace_lesson_top_avoid": str(item.get("action_trace_lesson_top_avoid") or action_trace.get("lesson_top_avoid") or ""),
        "outcome_status": outcome_status(item),
        "summary": compact(item.get("summary") or item.get("content") or "", 500),
    }


def merge_repair_outcomes(state_items, journal_items, limit=8):
    merged = []
    positions = {}
    for origin, items in [("state", state_items), ("journal", journal_items)]:
        for item in items if isinstance(items, list) else []:
            if not isinstance(item, dict):
                continue
            outcome = normalized_outcome(item, origin)
            trace_key = str(outcome["trace_index"] or outcome["session"] or f"{origin}:{len(merged)}")
            key = (
                trace_key,
                outcome["target_tentacle"],
                outcome["candidate"],
                outcome["outcome_status"],
                outcome["summary"],
            )
            if key in positions:
                current = merged[positions[key]]
                for field in [
                    "session",
                    "timestamp",
                    "tool",
                    "source",
                    "next_need",
                    "draft_status",
                    "action_trace_json",
                    "action_trace_status",
                    "action_trace_stage_count",
                    "action_trace_last_action",
                    "action_trace_repair_hint",
                    "action_trace_failed_stage",
                    "action_trace_recall_count",
                    "action_trace_recall_top_status",
                    "action_trace_recall_top_score",
                    "action_trace_recall_top_reasons",
                    "action_trace_recall_top_summary",
                    "action_trace_lesson_count",
                    "action_trace_lesson_reuse_count",
                    "action_trace_lesson_avoid_count",
                    "action_trace_lesson_top_reuse",
                    "action_trace_lesson_top_avoid",
                ]:
                    if outcome.get(field) and not current.get(field):
                        current[field] = outcome[field]
                if origin == "journal":
                    current["origin"] = "journal"
                continue
            positions[key] = len(merged)
            merged.append(outcome)
    return merged[-limit:]


def outcome_memory_markdown(outcomes, workspace, outcomes_file, repair_recall=None):
    lines = [
        "# Harness Repair Outcome Memory",
        "",
        "This file is local tentacle memory. It is fed to repair-session planning, not to the clean brain.",
        "",
        f"journal: `{rel(outcomes_file, workspace)}`",
        f"count: `{len(outcomes)}`",
        "",
    ]
    matches = (repair_recall or {}).get("matches") or []
    if repair_recall:
        target = repair_recall.get("target") or {}
        lines.extend(
            [
                "## Similar Action Trace Experience",
                "",
                f"target: `{target.get('tentacle', 'unknown')}/{target.get('tool', 'unknown')}`",
                f"field: `{target.get('field', 'none')}` mini_task: `{target.get('mini_task', 'none')}`",
                f"matches: `{len(matches)}`",
                "",
            ]
        )
        if matches:
            for match in matches:
                item = match.get("outcome") or {}
                lines.extend(
                    [
                        (
                            f"- score=`{match.get('score', 0)}` "
                            f"status=`{item.get('outcome_status') or 'unknown'}` "
                            f"target=`{item.get('target_tentacle') or 'unknown'}` "
                            f"candidate=`{item.get('candidate') or 'none'}`"
                        ),
                        f"  why: {compact(', '.join(match.get('reasons') or []), 220)}",
                        f"  summary: {compact(item.get('summary', ''), 260)}",
                        f"  action_trace: status=`{item.get('action_trace_status') or 'unknown'}` failed_stage=`{compact(item.get('action_trace_failed_stage') or 'none', 120)}` hint=`{compact(item.get('action_trace_repair_hint') or 'none', 160)}`",
                    ]
                )
        else:
            lines.append("No similar reviewed action-trace outcomes yet.")
        lines.append("")
    if not outcomes:
        lines.append("No repair outcomes recorded yet.")
        return "\n".join(lines) + "\n"
    lines.extend(["## Recent Repair Outcomes", ""])
    for item in outcomes[-8:]:
        session = item.get("session") or "none"
        target = item.get("target_tentacle") or "unknown"
        candidate = item.get("candidate") or "none"
        status = item.get("outcome_status") or "unknown"
        origin = item.get("origin") or "unknown"
        action_status = item.get("action_trace_status") or "unknown"
        action_count = item.get("action_trace_stage_count") or "0"
        action_last = item.get("action_trace_last_action") or "none"
        action_hint = item.get("action_trace_repair_hint") or "none"
        recall_count = item.get("action_trace_recall_count") or "0"
        recall_status = item.get("action_trace_recall_top_status") or "none"
        recall_reasons = item.get("action_trace_recall_top_reasons") or "none"
        lesson_count = item.get("action_trace_lesson_count") or "0"
        lesson_reuse = item.get("action_trace_lesson_reuse_count") or "0"
        lesson_avoid = item.get("action_trace_lesson_avoid_count") or "0"
        lesson_top = item.get("action_trace_lesson_top_reuse") or item.get("action_trace_lesson_top_avoid") or "none"
        lines.extend(
            [
                f"- `{status}` target=`{target}` candidate=`{candidate}` origin=`{origin}` session=`{session}`",
                f"  summary: {compact(item.get('summary', ''), 320)}",
                f"  action_trace: status=`{action_status}` stages=`{action_count}` last=`{compact(action_last, 120)}` hint=`{compact(action_hint, 160)}`",
                f"  recall_used: matches=`{recall_count}` top=`{compact(recall_status, 80)}` reasons=`{compact(recall_reasons, 160)}`",
                f"  lesson_used: count=`{lesson_count}` reuse=`{lesson_reuse}` avoid=`{lesson_avoid}` top=`{compact(lesson_top, 160)}`",
            ]
        )
    return "\n".join(lines) + "\n"


def token_set(*values):
    tokens = set()
    for value in values:
        for token in re.findall(r"[a-z0-9_/-]+", str(value or "").lower()):
            if len(token) >= 3:
                tokens.add(token)
    return tokens


def repair_recall_target(target_tentacle, target_tool, latest_trace, latest_check, field_trajectory):
    metadata = metadata_of(latest_trace)
    return {
        "tentacle": target_tentacle or str(latest_trace.get("tentacle") or "unknown"),
        "tool": target_tool or tool_from_trace(latest_trace) or tool_from_check(latest_check) or "unknown",
        "field": field_trajectory.get("field") or str(latest_trace.get("field") or metadata.get("field_pack") or "none"),
        "mini_task": field_trajectory.get("mini_task") or str(metadata.get("field_mini_task") or "none"),
        "trace_status": str(latest_trace.get("status") or ""),
        "trace_summary": compact(latest_trace.get("summary") or metadata.get("summary") or "", 400),
        "check_status": str(latest_check.get("status") or ""),
        "check_command": compact(latest_check.get("command") or "", 300),
    }


def score_repair_outcome(outcome, target):
    score = 0
    reasons = []
    if outcome.get("target_tentacle") and outcome.get("target_tentacle") == target.get("tentacle"):
        score += 5
        reasons.append("same target tentacle")
    if outcome.get("tool") and outcome.get("tool") == target.get("tool"):
        score += 3
        reasons.append("same tool")
    if outcome.get("candidate") and outcome.get("candidate") not in {"none", "unknown"}:
        score += 1
        reasons.append("has candidate memory")
    if outcome.get("outcome_status") == "satisfied":
        score += 2
        reasons.append("satisfied repair")
    elif outcome.get("outcome_status") in {"partial", "failed"}:
        score += 1
        reasons.append("known failure mode")
    action_status = outcome.get("action_trace_status") or ""
    if action_status in {"failed", "missing_context", "partial"}:
        score += 2
        reasons.append(f"action trace blocker: {action_status}")
    elif action_status == "satisfied":
        score += 1
        reasons.append("satisfied action trace")
    field_tokens = token_set(target.get("field"), target.get("mini_task"))
    outcome_tokens = token_set(
        outcome.get("summary"),
        outcome.get("source"),
        outcome.get("next_need"),
        outcome.get("action_trace_last_action"),
        outcome.get("action_trace_repair_hint"),
        outcome.get("action_trace_failed_stage"),
    )
    overlap = sorted(field_tokens & outcome_tokens)
    if overlap:
        score += min(3, len(overlap))
        reasons.append("field overlap: " + ",".join(overlap[:4]))
    signal_tokens = token_set(
        target.get("trace_summary"),
        target.get("check_command"),
        target.get("trace_status"),
        target.get("check_status"),
    )
    signal_overlap = sorted(signal_tokens & outcome_tokens)
    if signal_overlap:
        score += min(4, len(signal_overlap))
        reasons.append("signal overlap: " + ",".join(signal_overlap[:4]))
    return score, reasons


def build_repair_recall(outcomes, target_tentacle, target_tool, latest_trace, latest_check, field_trajectory, limit=3):
    target = repair_recall_target(target_tentacle, target_tool, latest_trace, latest_check, field_trajectory)
    scored = []
    for index, outcome in enumerate(outcomes):
        score, reasons = score_repair_outcome(outcome, target)
        if score <= 0:
            continue
        scored.append({
            "score": score,
            "recency": index,
            "reasons": reasons,
            "outcome": outcome,
        })
    scored.sort(key=lambda item: (item["score"], item["recency"]), reverse=True)
    return {
        "schema_version": "octopus-harness-repair-recall-v1",
        "target": target,
        "match_count": len(scored[:limit]),
        "matches": scored[:limit],
    }


def lesson_direction(status):
    if status == "satisfied":
        return "reuse"
    if status in {"failed", "partial"}:
        return "avoid"
    return "inspect"


def lesson_from_outcome(outcome, source, reasons=None, score=""):
    status = outcome.get("outcome_status") or "unknown"
    hint = outcome.get("action_trace_repair_hint") or ""
    if hint.strip().lower() in {"review repair action plan", "review latest repair action plan"}:
        hint = ""
    hint = hint or outcome.get("summary") or ""
    failed_stage = outcome.get("action_trace_failed_stage") or ""
    recall_count = outcome.get("action_trace_recall_count") or "0"
    recall_status = outcome.get("action_trace_recall_top_status") or ""
    return {
        "direction": lesson_direction(status),
        "source": source,
        "score": str(score or ""),
        "status": status,
        "target_tentacle": outcome.get("target_tentacle") or "unknown",
        "candidate": outcome.get("candidate") or "none",
        "tool": outcome.get("tool") or "",
        "summary": compact(outcome.get("summary") or "", 360),
        "action_trace_status": outcome.get("action_trace_status") or "",
        "action_trace_hint": compact(hint, 280),
        "failed_stage": compact(failed_stage, 180),
        "recall_used": str(recall_count),
        "recall_top_status": str(recall_status),
        "reasons": [str(reason) for reason in reasons or []],
    }


def dedupe_lessons(lessons, limit=6):
    deduped = []
    seen = set()
    for lesson in lessons:
        key = (
            lesson.get("direction"),
            lesson.get("target_tentacle"),
            lesson.get("candidate"),
            lesson.get("status"),
            lesson.get("summary"),
            lesson.get("action_trace_hint"),
        )
        if key in seen:
            continue
        seen.add(key)
        deduped.append(lesson)
    return deduped[:limit]


def build_repair_lessons(outcomes, repair_recall):
    lessons = []
    for match in repair_recall.get("matches", []) if isinstance(repair_recall, dict) else []:
        if not isinstance(match, dict):
            continue
        outcome = match.get("outcome") if isinstance(match.get("outcome"), dict) else {}
        if outcome:
            lessons.append(
                lesson_from_outcome(
                    outcome,
                    "repair_recall",
                    match.get("reasons") if isinstance(match.get("reasons"), list) else [],
                    match.get("score"),
                )
            )
    for outcome in reversed(outcomes[-6:]):
        lessons.append(lesson_from_outcome(outcome, "outcome_memory"))
    lessons = dedupe_lessons(lessons)
    reuse = [lesson for lesson in lessons if lesson["direction"] == "reuse"]
    avoid = [lesson for lesson in lessons if lesson["direction"] == "avoid"]
    inspect = [lesson for lesson in lessons if lesson["direction"] == "inspect"]
    return {
        "schema_version": "octopus-harness-repair-lessons-v1",
        "lesson_count": len(lessons),
        "reuse_count": len(reuse),
        "avoid_count": len(avoid),
        "inspect_count": len(inspect),
        "top_reuse": reuse[0]["action_trace_hint"] or reuse[0]["summary"] if reuse else "",
        "top_avoid": avoid[0]["action_trace_hint"] or avoid[0]["summary"] if avoid else "",
        "lessons": lessons,
    }


def repair_lessons_markdown(lessons, workspace, lessons_json):
    lines = [
        "# Harness Repair Lessons",
        "",
        "This file compresses reviewed outcomes and similar recall into local tentacle lessons.",
        "It is Feed evidence for repair-session planning, not clean-brain context.",
        "",
        f"json: `{rel(lessons_json, workspace)}`",
        f"lesson_count: `{lessons.get('lesson_count', 0)}`",
        f"reuse_count: `{lessons.get('reuse_count', 0)}`",
        f"avoid_count: `{lessons.get('avoid_count', 0)}`",
        "",
    ]
    if not lessons.get("lessons"):
        lines.append("No reviewed repair lessons yet.")
        return "\n".join(lines) + "\n"
    lines.extend(["## Lessons", ""])
    for lesson in lessons["lessons"]:
        reasons = compact(", ".join(lesson.get("reasons") or []), 180)
        lines.extend(
            [
                (
                    f"- `{lesson.get('direction')}` status=`{lesson.get('status')}` "
                    f"target=`{lesson.get('target_tentacle')}` candidate=`{lesson.get('candidate')}` "
                    f"source=`{lesson.get('source')}` score=`{lesson.get('score')}`"
                ),
                f"  summary: {compact(lesson.get('summary'), 260)}",
                f"  carry_forward: {compact(lesson.get('action_trace_hint'), 240) or 'none'}",
                f"  failed_stage: {compact(lesson.get('failed_stage'), 160) or 'none'}",
                f"  recall_used: matches=`{lesson.get('recall_used') or '0'}` top=`{lesson.get('recall_top_status') or 'none'}`",
                f"  reasons: {reasons or 'none'}",
            ]
        )
    return "\n".join(lines) + "\n"


def int_count(value):
    try:
        return int(str(value).strip() or "0")
    except ValueError:
        return 0


def rate_label(numerator, denominator):
    if denominator <= 0:
        return "0.00"
    return f"{numerator / denominator:.2f}"


def add_effectiveness_hint(hints, direction, hint, status, outcome):
    hint = compact(hint or "", 240)
    if not hint:
        return
    key = (direction, hint)
    record = hints.setdefault(
        key,
        {
            "direction": direction,
            "hint": hint,
            "used_count": 0,
            "satisfied_count": 0,
            "partial_count": 0,
            "failed_count": 0,
            "unknown_count": 0,
            "latest_target": "",
            "latest_candidate": "",
            "latest_summary": "",
        },
    )
    record["used_count"] += 1
    if status == "satisfied":
        record["satisfied_count"] += 1
    elif status == "partial":
        record["partial_count"] += 1
    elif status == "failed":
        record["failed_count"] += 1
    else:
        record["unknown_count"] += 1
    record["latest_target"] = outcome.get("target_tentacle") or ""
    record["latest_candidate"] = outcome.get("candidate") or ""
    record["latest_summary"] = compact(outcome.get("summary") or "", 220)


def build_repair_lesson_effectiveness(outcomes):
    used = []
    hints = {}
    counts = {
        "satisfied": 0,
        "partial": 0,
        "failed": 0,
        "unknown": 0,
    }
    for outcome in outcomes:
        lesson_count = int_count(outcome.get("action_trace_lesson_count"))
        if lesson_count <= 0:
            continue
        status = outcome.get("outcome_status") or "unknown"
        if status not in counts:
            status = "unknown"
        counts[status] += 1
        used.append(outcome)
        if int_count(outcome.get("action_trace_lesson_reuse_count")) > 0:
            add_effectiveness_hint(
                hints,
                "reuse",
                outcome.get("action_trace_lesson_top_reuse"),
                status,
                outcome,
            )
        if int_count(outcome.get("action_trace_lesson_avoid_count")) > 0:
            add_effectiveness_hint(
                hints,
                "avoid",
                outcome.get("action_trace_lesson_top_avoid"),
                status,
                outcome,
            )
    used_count = len(used)
    hint_rows = []
    for record in hints.values():
        record["success_rate"] = rate_label(record["satisfied_count"], record["used_count"])
        record["failure_rate"] = rate_label(record["failed_count"], record["used_count"])
        hint_rows.append(record)
    hint_rows.sort(
        key=lambda item: (
            item["satisfied_count"],
            item["used_count"],
            -item["failed_count"],
            item["hint"],
        ),
        reverse=True,
    )
    reuse = [item for item in hint_rows if item["direction"] == "reuse"]
    avoid = [item for item in hint_rows if item["direction"] == "avoid"]
    avoid.sort(
        key=lambda item: (
            item["failed_count"] + item["partial_count"],
            item["used_count"],
            item["hint"],
        ),
        reverse=True,
    )
    return {
        "schema_version": "octopus-harness-repair-lesson-effectiveness-v1",
        "used_count": used_count,
        "satisfied_count": counts["satisfied"],
        "partial_count": counts["partial"],
        "failed_count": counts["failed"],
        "unknown_count": counts["unknown"],
        "success_rate": rate_label(counts["satisfied"], used_count),
        "partial_rate": rate_label(counts["partial"], used_count),
        "failure_rate": rate_label(counts["failed"], used_count),
        "top_reuse": reuse[0]["hint"] if reuse else "",
        "top_avoid": avoid[0]["hint"] if avoid else "",
        "hints": hint_rows[:8],
    }


def repair_lesson_effectiveness_markdown(effectiveness, workspace, effectiveness_json):
    lines = [
        "# Harness Repair Lesson Effectiveness",
        "",
        "This file measures whether prior repair lessons helped later reviewed outcomes.",
        "It is local tentacle Feed evidence, not clean-brain context.",
        "",
        f"json: `{rel(effectiveness_json, workspace)}`",
        f"used_count: `{effectiveness.get('used_count', 0)}`",
        f"satisfied_count: `{effectiveness.get('satisfied_count', 0)}`",
        f"partial_count: `{effectiveness.get('partial_count', 0)}`",
        f"failed_count: `{effectiveness.get('failed_count', 0)}`",
        f"success_rate: `{effectiveness.get('success_rate', '0.00')}`",
        f"failure_rate: `{effectiveness.get('failure_rate', '0.00')}`",
        "",
    ]
    if not effectiveness.get("hints"):
        lines.append("No scored repair outcome has used a repair lesson yet.")
        return "\n".join(lines) + "\n"
    lines.extend(["## Hints", ""])
    for hint in effectiveness["hints"]:
        lines.extend(
            [
                (
                    f"- `{hint.get('direction')}` used=`{hint.get('used_count')}` "
                    f"satisfied=`{hint.get('satisfied_count')}` partial=`{hint.get('partial_count')}` "
                    f"failed=`{hint.get('failed_count')}` success_rate=`{hint.get('success_rate')}`"
                ),
                f"  hint: {compact(hint.get('hint'), 260)}",
                f"  latest: target=`{hint.get('latest_target') or 'unknown'}` candidate=`{hint.get('latest_candidate') or 'none'}`",
                f"  summary: {compact(hint.get('latest_summary'), 220) or 'none'}",
            ]
        )
    return "\n".join(lines) + "\n"


def build_repair_decision(repair_lessons, effectiveness, repair_recall, target_tentacle, target_tool):
    used_count = int_count(effectiveness.get("used_count"))
    satisfied_count = int_count(effectiveness.get("satisfied_count"))
    partial_count = int_count(effectiveness.get("partial_count"))
    failed_count = int_count(effectiveness.get("failed_count"))
    lesson_count = int_count(repair_lessons.get("lesson_count"))
    recall_count = int_count(repair_recall.get("match_count"))
    top_reuse = effectiveness.get("top_reuse") or repair_lessons.get("top_reuse") or ""
    top_avoid = effectiveness.get("top_avoid") or repair_lessons.get("top_avoid") or ""
    if used_count <= 0:
        decision = "collect_lesson_outcomes" if lesson_count else "collect_repair_outcomes"
        status = "observe"
        focus = "score repair outcomes that use lessons" if lesson_count else "collect reviewed repair outcomes"
        next_kind = "verify"
        next_query = "score repair outcomes that used lessons" if lesson_count else "collect repair outcome memory"
    elif failed_count > satisfied_count:
        decision = "inspect_unreliable_lessons"
        status = "partial"
        focus = top_avoid or top_reuse or "repair lessons need inspection"
        next_kind = "execute"
        next_query = "octopus repair . after unreliable repair lessons"
    elif partial_count > 0 and satisfied_count == 0:
        decision = "tighten_partial_lessons"
        status = "partial"
        focus = top_avoid or top_reuse or "partial lesson outcomes need tighter evidence"
        next_kind = "execute"
        next_query = "octopus repair . after partial repair lessons"
    else:
        decision = "reuse_effective_lessons"
        status = "satisfied"
        focus = top_reuse or "reuse effective repair lessons"
        next_kind = "verify"
        next_query = "review repair plan with effective lessons"
    return {
        "schema_version": "octopus-harness-repair-decision-v1",
        "status": status,
        "decision": decision,
        "focus": compact(focus, 260),
        "target_tentacle": target_tentacle,
        "target_tool": target_tool,
        "next_need": {
            "kind": next_kind,
            "query": next_query,
        },
        "evidence": {
            "lesson_count": lesson_count,
            "recall_count": recall_count,
            "effectiveness_used_count": used_count,
            "effectiveness_satisfied_count": satisfied_count,
            "effectiveness_partial_count": partial_count,
            "effectiveness_failed_count": failed_count,
            "effectiveness_success_rate": effectiveness.get("success_rate") or "0.00",
            "effectiveness_failure_rate": effectiveness.get("failure_rate") or "0.00",
            "top_reuse": compact(top_reuse, 260),
            "top_avoid": compact(top_avoid, 260),
        },
    }


def repair_decision_markdown(decision, workspace, decision_json):
    evidence = decision.get("evidence") if isinstance(decision.get("evidence"), dict) else {}
    next_need = decision.get("next_need") if isinstance(decision.get("next_need"), dict) else {}
    lines = [
        "# Harness Repair Decision",
        "",
        "This file compresses repair lesson evidence into the next local harness focus.",
        "It is Feed evidence for the repair tentacle, not clean-brain context.",
        "",
        f"json: `{rel(decision_json, workspace)}`",
        f"status: `{decision.get('status')}`",
        f"decision: `{decision.get('decision')}`",
        f"focus: {compact(decision.get('focus'), 260)}",
        f"target: `{decision.get('target_tentacle')}/{decision.get('target_tool')}`",
        f"next_need: `{next_need.get('kind') or 'verify'} {next_need.get('query') or ''}`",
        "",
        "## Evidence",
        "",
        f"- lessons: `{evidence.get('lesson_count', 0)}`",
        f"- recall: `{evidence.get('recall_count', 0)}`",
        f"- effectiveness_used: `{evidence.get('effectiveness_used_count', 0)}`",
        f"- effectiveness_satisfied: `{evidence.get('effectiveness_satisfied_count', 0)}`",
        f"- effectiveness_partial: `{evidence.get('effectiveness_partial_count', 0)}`",
        f"- effectiveness_failed: `{evidence.get('effectiveness_failed_count', 0)}`",
        f"- success_rate: `{evidence.get('effectiveness_success_rate', '0.00')}`",
        f"- failure_rate: `{evidence.get('effectiveness_failure_rate', '0.00')}`",
        f"- top_reuse: {compact(evidence.get('top_reuse'), 260) or 'none'}",
        f"- top_avoid: {compact(evidence.get('top_avoid'), 260) or 'none'}",
    ]
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


def action_trace_record(
    workspace,
    session_path,
    need_kind,
    need_query,
    source,
    code_context,
    adapter_context,
    field_trajectory,
    repair_recall,
    repair_lessons,
    repair_lesson_effectiveness,
    repair_decision,
    draft,
    repair_plan,
):
    recall_matches = repair_recall.get("matches") if isinstance(repair_recall.get("matches"), list) else []
    recall_top = recall_matches[0] if recall_matches and isinstance(recall_matches[0], dict) else {}
    recall_outcome = recall_top.get("outcome") if isinstance(recall_top.get("outcome"), dict) else {}
    recall_reasons = recall_top.get("reasons") if isinstance(recall_top.get("reasons"), list) else []
    recall_summary = {
        "match_count": str(repair_recall.get("match_count") or len(recall_matches)),
        "top_score": str(recall_top.get("score") or ""),
        "top_status": str(recall_outcome.get("outcome_status") or ""),
        "top_target": str(recall_outcome.get("target_tentacle") or ""),
        "top_candidate": str(recall_outcome.get("candidate") or ""),
        "top_reasons": compact(", ".join(str(reason) for reason in recall_reasons), 260),
        "top_summary": compact(recall_outcome.get("summary") or "", 320),
        "top_action_status": str(recall_outcome.get("action_trace_status") or ""),
        "top_action_hint": compact(recall_outcome.get("action_trace_repair_hint") or "", 240),
    }
    recall_count = recall_summary["match_count"]
    recall_top_status = recall_summary["top_status"] or "none"
    lessons_summary = {
        "lesson_count": str(repair_lessons.get("lesson_count") or 0),
        "reuse_count": str(repair_lessons.get("reuse_count") or 0),
        "avoid_count": str(repair_lessons.get("avoid_count") or 0),
        "top_reuse": compact(repair_lessons.get("top_reuse") or "", 240),
        "top_avoid": compact(repair_lessons.get("top_avoid") or "", 240),
    }
    effectiveness_summary = {
        "used_count": str(repair_lesson_effectiveness.get("used_count") or 0),
        "satisfied_count": str(repair_lesson_effectiveness.get("satisfied_count") or 0),
        "failed_count": str(repair_lesson_effectiveness.get("failed_count") or 0),
        "success_rate": str(repair_lesson_effectiveness.get("success_rate") or "0.00"),
        "top_reuse": compact(repair_lesson_effectiveness.get("top_reuse") or "", 240),
        "top_avoid": compact(repair_lesson_effectiveness.get("top_avoid") or "", 240),
    }
    decision_next = repair_decision.get("next_need") if isinstance(repair_decision.get("next_need"), dict) else {}
    decision_summary = {
        "status": str(repair_decision.get("status") or ""),
        "decision": str(repair_decision.get("decision") or ""),
        "focus": compact(repair_decision.get("focus") or "", 240),
        "next_need_kind": str(decision_next.get("kind") or ""),
        "next_need_query": compact(decision_next.get("query") or "", 240),
    }
    stages = [
        {
            "kind": "Need",
            "action": "read cognitive repair need",
            "result": f"{need_kind} {need_query}",
            "status": "satisfied",
        },
        {
            "kind": "Tool",
            "action": "probe local execution substrate",
            "result": f"adapter={adapter_context['status']} missing_core={adapter_context['missing_core'] or 'none'}",
            "status": adapter_context["status"] or "unknown",
        },
        {
            "kind": "Action",
            "action": "select target code evidence",
            "result": f"{code_context['tentacle']}/{code_context['tool']}",
            "status": "missing_context" if code_context["tentacle"] == "unknown" else "satisfied",
        },
        {
            "kind": "Action",
            "action": "merge field, recall, lessons, effectiveness, decision, and outcome evidence",
            "result": f"field={field_trajectory['field']} mini_task={field_trajectory['mini_task']} recalled={recall_count} top={recall_top_status} lessons={lessons_summary['lesson_count']} effectiveness_used={effectiveness_summary['used_count']} success_rate={effectiveness_summary['success_rate']} decision={decision_summary['decision']}",
            "status": "satisfied",
        },
        {
            "kind": "Tool",
            "action": "ask optional repair provider for a draft",
            "result": f"draft={draft['status']} prefix={draft['prefix']}",
            "status": draft["status"],
        },
        {
            "kind": "Feed",
            "action": "write reviewable repair plan",
            "result": f"plan={rel(session_path.parent / 'REPAIR_PLAN.json', workspace)} status={repair_plan['status']}",
            "status": repair_plan["status"],
        },
    ]
    status = "satisfied" if code_context["tentacle"] != "unknown" else "missing_context"
    repair_hint = "review repair action plan"
    if status == "missing_context":
        repair_hint = "repair harness action trace: missing target context"
    return {
        "schema_version": "octopus-harness-repair-action-trace-v1",
        "status": status,
        "stage_count": len(stages),
        "last_action": stages[-1]["action"],
        "session": rel(session_path, workspace),
        "source": source,
        "next_need": {
            "kind": "execute" if status == "missing_context" else need_kind,
            "query": repair_hint if status == "missing_context" else need_query,
        },
        "repair_hint": repair_hint,
        "repair_recall": recall_summary,
        "repair_lessons": lessons_summary,
        "repair_lesson_effectiveness": effectiveness_summary,
        "repair_decision": decision_summary,
        "stages": stages,
    }


def action_trace_markdown(record, workspace, repair_plan):
    lines = [
        "# Tool-Side Action Trace",
        "",
        "This is the harness-repair tentacle's local thinking trace.",
        "It is Feed evidence for heartbeat repair, not clean-brain context.",
        "",
        f"status: `{record['status']}`",
        f"stage_count: `{record['stage_count']}`",
        f"last_action: `{record['last_action']}`",
        f"session: `{record['session']}`",
        f"source: `{record['source']}`",
        f"repair_hint: `{record['repair_hint']}`",
        f"recall_matches: `{record.get('repair_recall', {}).get('match_count', '0')}`",
        f"recall_top: `{record.get('repair_recall', {}).get('top_status', 'none') or 'none'}`",
        f"recall_reasons: `{record.get('repair_recall', {}).get('top_reasons', 'none') or 'none'}`",
        f"lesson_count: `{record.get('repair_lessons', {}).get('lesson_count', '0')}`",
        f"lesson_reuse: `{record.get('repair_lessons', {}).get('reuse_count', '0')}`",
        f"lesson_avoid: `{record.get('repair_lessons', {}).get('avoid_count', '0')}`",
        f"lesson_effectiveness_used: `{record.get('repair_lesson_effectiveness', {}).get('used_count', '0')}`",
        f"lesson_effectiveness_success_rate: `{record.get('repair_lesson_effectiveness', {}).get('success_rate', '0.00')}`",
        f"repair_decision: `{record.get('repair_decision', {}).get('decision', 'none') or 'none'}`",
        f"repair_decision_focus: `{record.get('repair_decision', {}).get('focus', 'none') or 'none'}`",
        "",
        "## Need -> Tool -> Action -> Feed",
        "",
    ]
    for index, stage in enumerate(record["stages"], start=1):
        lines.extend(
            [
                f"{index}. {stage['kind']}",
                f"   action: {stage['action']}",
                f"   result: {stage['result']}",
                f"   status: {stage['status']}",
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
repair_recall_json = session_dir / "REPAIR_RECALL.json"
repair_lessons_json = session_dir / "REPAIR_LESSONS.json"
repair_lessons_md = session_dir / "REPAIR_LESSONS.md"
repair_lesson_effectiveness_json = session_dir / "REPAIR_LESSON_EFFECTIVENESS.json"
repair_lesson_effectiveness_md = session_dir / "REPAIR_LESSON_EFFECTIVENESS.md"
repair_decision_json = session_dir / "REPAIR_DECISION.json"
repair_decision_md = session_dir / "REPAIR_DECISION.md"
adapter_context_md = session_dir / "ADAPTER_CONTEXT.md"
code_context_md = session_dir / "CODE_CONTEXT.md"
field_trajectory_md = session_dir / "FIELD_TRAJECTORY.md"
action_trace_md = session_dir / "ACTION_TRACE.md"
action_trace_json = session_dir / "ACTION_TRACE.json"
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
repair_recall = build_repair_recall(
    repair_outcomes,
    target_tentacle,
    target_tool,
    latest_trace,
    latest_check,
    field_trajectory,
)
repair_lessons = build_repair_lessons(repair_outcomes, repair_recall)
repair_lesson_effectiveness = build_repair_lesson_effectiveness(repair_outcomes)
repair_decision = build_repair_decision(
    repair_lessons,
    repair_lesson_effectiveness,
    repair_recall,
    target_tentacle,
    target_tool,
)
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
    "repair_recall": rel(repair_recall_json, workspace),
    "repair_lessons": rel(repair_lessons_md, workspace),
    "repair_lessons_json": rel(repair_lessons_json, workspace),
    "repair_lesson_effectiveness": rel(repair_lesson_effectiveness_md, workspace),
    "repair_lesson_effectiveness_json": rel(repair_lesson_effectiveness_json, workspace),
    "repair_decision": rel(repair_decision_md, workspace),
    "repair_decision_json": rel(repair_decision_json, workspace),
    "adapter_context": rel(adapter_context_md, workspace),
    "code_context": rel(code_context_md, workspace),
    "field_trajectory": rel(field_trajectory_md, workspace),
    "action_trace": rel(action_trace_md, workspace),
    "action_trace_json": rel(action_trace_json, workspace),
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
    "repair_recall_target": repair_recall["target"],
    "repair_lessons_summary": {
        "lesson_count": repair_lessons["lesson_count"],
        "reuse_count": repair_lessons["reuse_count"],
        "avoid_count": repair_lessons["avoid_count"],
        "top_reuse": repair_lessons["top_reuse"],
        "top_avoid": repair_lessons["top_avoid"],
    },
    "repair_lesson_effectiveness_summary": {
        "used_count": repair_lesson_effectiveness["used_count"],
        "satisfied_count": repair_lesson_effectiveness["satisfied_count"],
        "failed_count": repair_lesson_effectiveness["failed_count"],
        "success_rate": repair_lesson_effectiveness["success_rate"],
        "top_reuse": repair_lesson_effectiveness["top_reuse"],
        "top_avoid": repair_lesson_effectiveness["top_avoid"],
    },
    "repair_decision_summary": {
        "status": repair_decision["status"],
        "decision": repair_decision["decision"],
        "focus": repair_decision["focus"],
        "next_need": repair_decision["next_need"],
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
repair_recall_json.write_text(
    json.dumps(repair_recall, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_lessons_json.write_text(
    json.dumps(repair_lessons, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_lessons_md.write_text(
    repair_lessons_markdown(repair_lessons, workspace, repair_lessons_json),
    encoding="utf-8",
)
repair_lesson_effectiveness_json.write_text(
    json.dumps(repair_lesson_effectiveness, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_lesson_effectiveness_md.write_text(
    repair_lesson_effectiveness_markdown(
        repair_lesson_effectiveness,
        workspace,
        repair_lesson_effectiveness_json,
    ),
    encoding="utf-8",
)
repair_decision_json.write_text(
    json.dumps(repair_decision, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_decision_md.write_text(
    repair_decision_markdown(repair_decision, workspace, repair_decision_json),
    encoding="utf-8",
)
outcome_memory_md.write_text(
    outcome_memory_markdown(repair_outcomes, workspace, outcomes_file, repair_recall),
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
repair_plan["inputs"]["repair_recall"] = rel(repair_recall_json, workspace)
repair_plan["inputs"]["repair_lessons"] = rel(repair_lessons_md, workspace)
repair_plan["inputs"]["repair_lessons_json"] = rel(repair_lessons_json, workspace)
repair_plan["inputs"]["repair_lesson_effectiveness"] = rel(repair_lesson_effectiveness_md, workspace)
repair_plan["inputs"]["repair_lesson_effectiveness_json"] = rel(repair_lesson_effectiveness_json, workspace)
repair_plan["inputs"]["repair_decision"] = rel(repair_decision_md, workspace)
repair_plan["inputs"]["repair_decision_json"] = rel(repair_decision_json, workspace)
repair_plan["inputs"]["action_trace"] = rel(action_trace_md, workspace)
repair_plan["inputs"]["action_trace_json"] = rel(action_trace_json, workspace)
repair_plan["field_trajectory_target"] = {
    "field": field_trajectory["field"],
    "mini_task": field_trajectory["mini_task"],
    "session": field_trajectory["session"],
    "verifier_status": field_trajectory["verifier_status"],
    "verifier_error": field_trajectory["verifier_error"],
}
repair_plan["repair_recall_target"] = repair_recall["target"]
repair_plan["repair_lessons_summary"] = session["repair_lessons_summary"]
repair_plan["repair_lesson_effectiveness_summary"] = session["repair_lesson_effectiveness_summary"]
repair_plan["repair_decision_summary"] = session["repair_decision_summary"]
repair_plan["review_boundary"] = "Review ACTION_TRACE.md, ACTION_TRACE.json, REPAIR_RECALL.json, REPAIR_LESSONS, REPAIR_LESSON_EFFECTIVENESS, REPAIR_DECISION, ADAPTER_CONTEXT, FIELD_TRAJECTORY, CODE_CONTEXT, OUTCOME_MEMORY, DRAFT, and this plan before running commands."
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
recalled_outcome_lines = [
    (
        f"- score={match.get('score', 0)}"
        f" status={compact((match.get('outcome') or {}).get('outcome_status') or 'unknown')}"
        f" target={compact((match.get('outcome') or {}).get('target_tentacle') or 'unknown')}"
        f" reasons={compact(', '.join(match.get('reasons') or []), 180)}"
    )
    for match in repair_recall.get("matches", [])
] or ["- none"]
lesson_lines = [
    (
        f"- {lesson.get('direction')}"
        f" status={compact(lesson.get('status') or 'unknown')}"
        f" target={compact(lesson.get('target_tentacle') or 'unknown')}"
        f" carry={compact(lesson.get('action_trace_hint') or lesson.get('summary') or '', 220)}"
    )
    for lesson in repair_lessons.get("lessons", [])[:4]
] or ["- none"]
effectiveness_lines = [
    (
        f"- {hint.get('direction')}"
        f" used={hint.get('used_count')}"
        f" satisfied={hint.get('satisfied_count')}"
        f" failed={hint.get('failed_count')}"
        f" success={hint.get('success_rate')}"
        f" hint={compact(hint.get('hint') or '', 180)}"
    )
    for hint in repair_lesson_effectiveness.get("hints", [])[:4]
] or ["- none"]
decision_next = repair_decision.get("next_need") if isinstance(repair_decision.get("next_need"), dict) else {}
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
            "similar action-trace recall:",
            f"- recall artifact: `{rel(repair_recall_json, workspace)}`",
            f"- matches: `{repair_recall.get('match_count', 0)}`",
            *recalled_outcome_lines,
            "",
            "repair lessons:",
            f"- lessons artifact: `{rel(repair_lessons_md, workspace)}`",
            f"- lessons json: `{rel(repair_lessons_json, workspace)}`",
            f"- count: `{repair_lessons.get('lesson_count', 0)}` reuse: `{repair_lessons.get('reuse_count', 0)}` avoid: `{repair_lessons.get('avoid_count', 0)}`",
            *lesson_lines,
            "",
            "repair lesson effectiveness:",
            f"- effectiveness artifact: `{rel(repair_lesson_effectiveness_md, workspace)}`",
            f"- effectiveness json: `{rel(repair_lesson_effectiveness_json, workspace)}`",
            f"- used: `{repair_lesson_effectiveness.get('used_count', 0)}` satisfied: `{repair_lesson_effectiveness.get('satisfied_count', 0)}` failed: `{repair_lesson_effectiveness.get('failed_count', 0)}` success_rate: `{repair_lesson_effectiveness.get('success_rate', '0.00')}`",
            *effectiveness_lines,
            "",
            "repair decision:",
            f"- decision artifact: `{rel(repair_decision_md, workspace)}`",
            f"- decision json: `{rel(repair_decision_json, workspace)}`",
            f"- status: `{repair_decision.get('status')}` decision: `{repair_decision.get('decision')}`",
            f"- focus: {compact(repair_decision.get('focus'), 260)}",
            f"- decision next Need: `{decision_next.get('kind') or 'verify'} {decision_next.get('query') or ''}`",
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
            f"- repair recall: `{rel(repair_recall_json, workspace)}`",
            f"- repair lessons: `{rel(repair_lessons_md, workspace)}`",
            f"- repair lessons json: `{rel(repair_lessons_json, workspace)}`",
            f"- repair lesson effectiveness: `{rel(repair_lesson_effectiveness_md, workspace)}`",
            f"- repair lesson effectiveness json: `{rel(repair_lesson_effectiveness_json, workspace)}`",
            f"- repair decision: `{rel(repair_decision_md, workspace)}`",
            f"- repair decision json: `{rel(repair_decision_json, workspace)}`",
            f"- adapter context: `{rel(adapter_context_md, workspace)}`",
            f"- code context: `{rel(code_context_md, workspace)}`",
            f"- field trajectory: `{rel(field_trajectory_md, workspace)}`",
            f"- action trace: `{rel(action_trace_md, workspace)}`",
            f"- action trace json: `{rel(action_trace_json, workspace)}`",
            f"- repair plan: `{rel(repair_plan_json, workspace)}`",
        ]
    )
    + "\n",
    encoding="utf-8",
)
draft = llm_draft(prompt_md.read_text(encoding="utf-8"), session, workspace)
draft_md.write_text(draft_markdown(draft, prompt_md, session_json, workspace), encoding="utf-8")
action_trace = action_trace_record(
        workspace,
        session_json,
        next_need_kind,
        next_need_query,
        source,
        code_context,
        adapter_context,
        field_trajectory,
        repair_recall,
        repair_lessons,
        repair_lesson_effectiveness,
        repair_decision,
        draft,
        repair_plan,
)
action_trace_json.write_text(
    json.dumps(action_trace, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
action_trace_md.write_text(
    action_trace_markdown(action_trace, workspace, repair_plan),
    encoding="utf-8",
)
session["draft"] = {
    "status": draft["status"],
    "path": rel(draft_md, workspace),
    "prefix": draft["prefix"],
    "model": draft.get("model", ""),
}
session["action_trace"] = rel(action_trace_md, workspace)
session["action_trace_json"] = rel(action_trace_json, workspace)
repair_plan["inputs"]["draft"] = rel(draft_md, workspace)
repair_plan["inputs"]["review"] = rel(review_md, workspace)
repair_plan["inputs"]["action_trace"] = rel(action_trace_md, workspace)
repair_plan["inputs"]["action_trace_json"] = rel(action_trace_json, workspace)
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
            f"- action trace json: `{rel(action_trace_json, workspace)}`",
            f"- repair recall: `{rel(repair_recall_json, workspace)}`",
            f"- repair lessons: `{rel(repair_lessons_md, workspace)}`",
            f"- repair lessons json: `{rel(repair_lessons_json, workspace)}`",
            f"- repair lesson effectiveness: `{rel(repair_lesson_effectiveness_md, workspace)}`",
            f"- repair lesson effectiveness json: `{rel(repair_lesson_effectiveness_json, workspace)}`",
            f"- repair decision: `{rel(repair_decision_md, workspace)}`",
            f"- repair decision json: `{rel(repair_decision_json, workspace)}`",
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
            f"repair recall: `{rel(repair_recall_json, workspace)}`",
            f"repair lessons: `{rel(repair_lessons_md, workspace)}`",
            f"repair lessons json: `{rel(repair_lessons_json, workspace)}`",
            f"repair lesson effectiveness: `{rel(repair_lesson_effectiveness_md, workspace)}`",
            f"repair lesson effectiveness json: `{rel(repair_lesson_effectiveness_json, workspace)}`",
            f"repair decision: `{rel(repair_decision_md, workspace)}`",
            f"repair decision json: `{rel(repair_decision_json, workspace)}`",
            f"adapter context: `{rel(adapter_context_md, workspace)}`",
            f"code context: `{rel(code_context_md, workspace)}`",
            f"field trajectory: `{rel(field_trajectory_md, workspace)}`",
            f"action trace: `{rel(action_trace_md, workspace)}`",
            f"action trace json: `{rel(action_trace_json, workspace)}`",
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
    "repair_recall": rel(repair_recall_json, workspace),
    "repair_recall_count": str(repair_recall.get("match_count", 0)),
    "repair_recall_target": json.dumps(repair_recall.get("target", {}), ensure_ascii=True, sort_keys=True),
    "repair_lessons": rel(repair_lessons_md, workspace),
    "repair_lessons_json": rel(repair_lessons_json, workspace),
    "repair_lessons_count": str(repair_lessons.get("lesson_count", 0)),
    "repair_lessons_reuse_count": str(repair_lessons.get("reuse_count", 0)),
    "repair_lessons_avoid_count": str(repair_lessons.get("avoid_count", 0)),
    "repair_lessons_top_reuse": repair_lessons.get("top_reuse", ""),
    "repair_lessons_top_avoid": repair_lessons.get("top_avoid", ""),
    "repair_lesson_effectiveness": rel(repair_lesson_effectiveness_md, workspace),
    "repair_lesson_effectiveness_json": rel(repair_lesson_effectiveness_json, workspace),
    "repair_lesson_effectiveness_used_count": str(repair_lesson_effectiveness.get("used_count", 0)),
    "repair_lesson_effectiveness_satisfied_count": str(repair_lesson_effectiveness.get("satisfied_count", 0)),
    "repair_lesson_effectiveness_partial_count": str(repair_lesson_effectiveness.get("partial_count", 0)),
    "repair_lesson_effectiveness_failed_count": str(repair_lesson_effectiveness.get("failed_count", 0)),
    "repair_lesson_effectiveness_success_rate": repair_lesson_effectiveness.get("success_rate", "0.00"),
    "repair_lesson_effectiveness_failure_rate": repair_lesson_effectiveness.get("failure_rate", "0.00"),
    "repair_lesson_effectiveness_top_reuse": repair_lesson_effectiveness.get("top_reuse", ""),
    "repair_lesson_effectiveness_top_avoid": repair_lesson_effectiveness.get("top_avoid", ""),
    "repair_decision": rel(repair_decision_md, workspace),
    "repair_decision_json": rel(repair_decision_json, workspace),
    "repair_decision_status": repair_decision.get("status", ""),
    "repair_decision_kind": repair_decision.get("decision", ""),
    "repair_decision_focus": repair_decision.get("focus", ""),
    "repair_decision_next_need_kind": repair_decision.get("next_need", {}).get("kind", ""),
    "repair_decision_next_need_query": repair_decision.get("next_need", {}).get("query", ""),
    "adapter_context": rel(adapter_context_md, workspace),
    "code_context": rel(code_context_md, workspace),
    "field_trajectory": rel(field_trajectory_md, workspace),
    "action_trace": rel(action_trace_md, workspace),
    "action_trace_json": rel(action_trace_json, workspace),
    "repair_plan": rel(repair_plan_json, workspace),
    "action_trace_status": action_trace["status"],
    "action_trace_stage_count": str(action_trace["stage_count"]),
    "action_trace_last_action": action_trace["last_action"],
    "action_trace_repair_hint": action_trace["repair_hint"],
    "action_trace_recall_count": action_trace["repair_recall"]["match_count"],
    "action_trace_recall_top_status": action_trace["repair_recall"]["top_status"],
    "action_trace_recall_top_score": action_trace["repair_recall"]["top_score"],
    "action_trace_recall_top_reasons": action_trace["repair_recall"]["top_reasons"],
    "action_trace_recall_top_summary": action_trace["repair_recall"]["top_summary"],
    "action_trace_lesson_count": action_trace["repair_lessons"]["lesson_count"],
    "action_trace_lesson_reuse_count": action_trace["repair_lessons"]["reuse_count"],
    "action_trace_lesson_avoid_count": action_trace["repair_lessons"]["avoid_count"],
    "action_trace_lesson_top_reuse": action_trace["repair_lessons"]["top_reuse"],
    "action_trace_lesson_top_avoid": action_trace["repair_lessons"]["top_avoid"],
    "action_trace_repair_decision": action_trace["repair_decision"]["decision"],
    "action_trace_repair_decision_focus": action_trace["repair_decision"]["focus"],
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
