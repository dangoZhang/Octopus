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
import platform
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
    action_decision = action_trace.get("repair_decision") if isinstance(action_trace.get("repair_decision"), dict) else {}
    action_patch_draft = action_trace.get("repair_patch_draft") if isinstance(action_trace.get("repair_patch_draft"), dict) else {}
    action_patch_review = action_trace.get("repair_patch_review") if isinstance(action_trace.get("repair_patch_review"), dict) else {}
    action_patch_apply = action_trace.get("repair_patch_apply") if isinstance(action_trace.get("repair_patch_apply"), dict) else {}
    action_patch_verify = action_trace.get("repair_patch_verify") if isinstance(action_trace.get("repair_patch_verify"), dict) else {}
    action_patch_learning = action_trace.get("repair_patch_learning") if isinstance(action_trace.get("repair_patch_learning"), dict) else {}
    action_patch_strategy = action_trace.get("repair_patch_strategy") if isinstance(action_trace.get("repair_patch_strategy"), dict) else {}
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
        "draft_prefix": str(item.get("draft_prefix") or ""),
        "draft_model": str(item.get("draft_model") or ""),
        "repair_command_check": str(item.get("repair_command_check") or ""),
        "repair_command_grant": str(item.get("repair_command_grant") or ""),
        "repair_command_apply": str(item.get("repair_command_apply") or ""),
        "repair_command_score": str(item.get("repair_command_score") or ""),
        "repair_command_score_options": str(item.get("repair_command_score_options") or ""),
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
        "action_trace_command_strategy_status": str(item.get("action_trace_command_strategy_status") or action_trace.get("command_strategy_status") or ""),
        "action_trace_command_strategy_focus": str(item.get("action_trace_command_strategy_focus") or action_trace.get("command_strategy_focus") or ""),
        "action_trace_command_strategy_learned_reuse": str(item.get("action_trace_command_strategy_learned_reuse") or action_trace.get("command_strategy_learned_reuse") or ""),
        "action_trace_command_strategy_learned_avoid": str(item.get("action_trace_command_strategy_learned_avoid") or action_trace.get("command_strategy_learned_avoid") or ""),
        "action_trace_command_strategy_next_need_kind": str(item.get("action_trace_command_strategy_next_need_kind") or action_trace.get("command_strategy_next_need_kind") or ""),
        "action_trace_command_strategy_next_need_query": str(item.get("action_trace_command_strategy_next_need_query") or action_trace.get("command_strategy_next_need_query") or ""),
        "action_trace_repair_decision": str(item.get("action_trace_repair_decision") or action_decision.get("decision") or ""),
        "action_trace_repair_decision_focus": str(item.get("action_trace_repair_decision_focus") or action_decision.get("focus") or ""),
        "action_trace_repair_patch_draft_status": str(item.get("action_trace_repair_patch_draft_status") or action_patch_draft.get("status") or ""),
        "action_trace_repair_patch_draft_has_patch": str(item.get("action_trace_repair_patch_draft_has_patch") or action_patch_draft.get("has_patch") or ""),
        "action_trace_repair_patch_draft_target": str(item.get("action_trace_repair_patch_draft_target") or action_patch_draft.get("target") or ""),
        "action_trace_repair_patch_draft_preview": str(item.get("action_trace_repair_patch_draft_preview") or action_patch_draft.get("preview") or ""),
        "action_trace_repair_patch_review_status": str(item.get("action_trace_repair_patch_review_status") or action_patch_review.get("status") or ""),
        "action_trace_repair_patch_review_check_status": str(item.get("action_trace_repair_patch_review_check_status") or action_patch_review.get("check_status") or ""),
        "action_trace_repair_patch_review_has_patch": str(item.get("action_trace_repair_patch_review_has_patch") or action_patch_review.get("has_patch") or ""),
        "action_trace_repair_patch_review_target": str(item.get("action_trace_repair_patch_review_target") or action_patch_review.get("target") or ""),
        "action_trace_repair_patch_review_summary": str(item.get("action_trace_repair_patch_review_summary") or action_patch_review.get("summary") or ""),
        "action_trace_repair_patch_apply_status": str(item.get("action_trace_repair_patch_apply_status") or action_patch_apply.get("status") or ""),
        "action_trace_repair_patch_apply_applied": str(item.get("action_trace_repair_patch_apply_applied") or action_patch_apply.get("applied") or ""),
        "action_trace_repair_patch_apply_target": str(item.get("action_trace_repair_patch_apply_target") or action_patch_apply.get("target") or ""),
        "action_trace_repair_patch_apply_summary": str(item.get("action_trace_repair_patch_apply_summary") or action_patch_apply.get("summary") or ""),
        "action_trace_repair_patch_verify_status": str(item.get("action_trace_repair_patch_verify_status") or action_patch_verify.get("status") or ""),
        "action_trace_repair_patch_verify_passed": str(item.get("action_trace_repair_patch_verify_passed") or action_patch_verify.get("passed") or ""),
        "action_trace_repair_patch_verify_target": str(item.get("action_trace_repair_patch_verify_target") or action_patch_verify.get("target") or ""),
        "action_trace_repair_patch_verify_command": str(item.get("action_trace_repair_patch_verify_command") or action_patch_verify.get("command") or ""),
        "action_trace_repair_patch_verify_summary": str(item.get("action_trace_repair_patch_verify_summary") or action_patch_verify.get("summary") or ""),
        "action_trace_repair_patch_learning_status": str(item.get("action_trace_repair_patch_learning_status") or action_patch_learning.get("status") or ""),
        "action_trace_repair_patch_learning_used_count": str(item.get("action_trace_repair_patch_learning_used_count") or action_patch_learning.get("used_count") or ""),
        "action_trace_repair_patch_learning_verified_count": str(item.get("action_trace_repair_patch_learning_verified_count") or action_patch_learning.get("verified_count") or ""),
        "action_trace_repair_patch_learning_verified_success_rate": str(item.get("action_trace_repair_patch_learning_verified_success_rate") or action_patch_learning.get("verified_success_rate") or ""),
        "action_trace_repair_patch_learning_top_reuse": str(item.get("action_trace_repair_patch_learning_top_reuse") or action_patch_learning.get("top_reuse") or ""),
        "action_trace_repair_patch_learning_top_avoid": str(item.get("action_trace_repair_patch_learning_top_avoid") or action_patch_learning.get("top_avoid") or ""),
        "action_trace_repair_patch_learning_next_need_kind": str(item.get("action_trace_repair_patch_learning_next_need_kind") or action_patch_learning.get("next_need_kind") or ""),
        "action_trace_repair_patch_learning_next_need_query": str(item.get("action_trace_repair_patch_learning_next_need_query") or action_patch_learning.get("next_need_query") or ""),
        "action_trace_repair_patch_strategy_status": str(item.get("action_trace_repair_patch_strategy_status") or action_patch_strategy.get("status") or ""),
        "action_trace_repair_patch_strategy_focus": str(item.get("action_trace_repair_patch_strategy_focus") or action_patch_strategy.get("focus") or ""),
        "action_trace_repair_patch_strategy_learned_reuse": str(item.get("action_trace_repair_patch_strategy_learned_reuse") or action_patch_strategy.get("learned_reuse") or ""),
        "action_trace_repair_patch_strategy_learned_avoid": str(item.get("action_trace_repair_patch_strategy_learned_avoid") or action_patch_strategy.get("learned_avoid") or ""),
        "action_trace_repair_patch_strategy_next_need_kind": str(item.get("action_trace_repair_patch_strategy_next_need_kind") or action_patch_strategy.get("next_need_kind") or ""),
        "action_trace_repair_patch_strategy_next_need_query": str(item.get("action_trace_repair_patch_strategy_next_need_query") or action_patch_strategy.get("next_need_query") or ""),
        "action_trace_harness_adaptation_status": str(item.get("action_trace_harness_adaptation_status") or action_trace.get("harness_adaptation_status") or ""),
        "action_trace_harness_adaptation_focus": str(item.get("action_trace_harness_adaptation_focus") or action_trace.get("harness_adaptation_focus") or ""),
        "action_trace_harness_environment_drift_status": str(item.get("action_trace_harness_environment_drift_status") or action_trace.get("harness_environment_drift_status") or ""),
        "action_trace_harness_environment_drift_detail": str(item.get("action_trace_harness_environment_drift_detail") or action_trace.get("harness_environment_drift_detail") or ""),
        "action_trace_harness_environment_drift_history_count": str(item.get("action_trace_harness_environment_drift_history_count") or action_trace.get("harness_environment_drift_history_count") or ""),
        "action_trace_harness_environment_drift_gained_count": str(item.get("action_trace_harness_environment_drift_gained_count") or action_trace.get("harness_environment_drift_gained_count") or ""),
        "action_trace_harness_environment_drift_lost_count": str(item.get("action_trace_harness_environment_drift_lost_count") or action_trace.get("harness_environment_drift_lost_count") or ""),
        "action_trace_harness_environment_drift_next_need_kind": str(item.get("action_trace_harness_environment_drift_next_need_kind") or action_trace.get("harness_environment_drift_next_need_kind") or ""),
        "action_trace_harness_environment_drift_next_need_query": str(item.get("action_trace_harness_environment_drift_next_need_query") or action_trace.get("harness_environment_drift_next_need_query") or ""),
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
                    "draft_prefix",
                    "draft_model",
                    "repair_command_check",
                    "repair_command_grant",
                    "repair_command_apply",
                    "repair_command_score",
                    "repair_command_score_options",
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
                    "action_trace_command_strategy_status",
                    "action_trace_command_strategy_focus",
                    "action_trace_command_strategy_learned_reuse",
                    "action_trace_command_strategy_learned_avoid",
                    "action_trace_command_strategy_next_need_kind",
                    "action_trace_command_strategy_next_need_query",
                    "action_trace_repair_decision",
                    "action_trace_repair_decision_focus",
                    "action_trace_repair_patch_draft_status",
                    "action_trace_repair_patch_draft_has_patch",
                    "action_trace_repair_patch_draft_target",
                    "action_trace_repair_patch_draft_preview",
                    "action_trace_repair_patch_review_status",
                    "action_trace_repair_patch_review_check_status",
                    "action_trace_repair_patch_review_has_patch",
                    "action_trace_repair_patch_review_target",
                    "action_trace_repair_patch_review_summary",
                    "action_trace_repair_patch_apply_status",
                    "action_trace_repair_patch_apply_applied",
                    "action_trace_repair_patch_apply_target",
                    "action_trace_repair_patch_apply_summary",
                    "action_trace_repair_patch_verify_status",
                    "action_trace_repair_patch_verify_passed",
                    "action_trace_repair_patch_verify_target",
                    "action_trace_repair_patch_verify_command",
                    "action_trace_repair_patch_verify_summary",
                    "action_trace_repair_patch_learning_status",
                    "action_trace_repair_patch_learning_used_count",
                    "action_trace_repair_patch_learning_verified_count",
                    "action_trace_repair_patch_learning_verified_success_rate",
                    "action_trace_repair_patch_learning_top_reuse",
                    "action_trace_repair_patch_learning_top_avoid",
                    "action_trace_repair_patch_learning_next_need_kind",
                    "action_trace_repair_patch_learning_next_need_query",
                    "action_trace_repair_patch_strategy_status",
                    "action_trace_repair_patch_strategy_focus",
                    "action_trace_repair_patch_strategy_learned_reuse",
                    "action_trace_repair_patch_strategy_learned_avoid",
                    "action_trace_repair_patch_strategy_next_need_kind",
                    "action_trace_repair_patch_strategy_next_need_query",
                    "action_trace_harness_adaptation_status",
                    "action_trace_harness_adaptation_focus",
                    "action_trace_harness_environment_drift_status",
                    "action_trace_harness_environment_drift_detail",
                    "action_trace_harness_environment_drift_history_count",
                    "action_trace_harness_environment_drift_gained_count",
                    "action_trace_harness_environment_drift_lost_count",
                    "action_trace_harness_environment_drift_next_need_kind",
                    "action_trace_harness_environment_drift_next_need_query",
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
        draft_status = item.get("draft_status") or "none"
        draft_model = item.get("draft_model") or item.get("draft_prefix") or "none"
        command_hint = (
            item.get("repair_command_apply")
            or item.get("repair_command_check")
            or item.get("repair_command_score")
            or "none"
        )
        recall_count = item.get("action_trace_recall_count") or "0"
        recall_status = item.get("action_trace_recall_top_status") or "none"
        recall_reasons = item.get("action_trace_recall_top_reasons") or "none"
        lesson_count = item.get("action_trace_lesson_count") or "0"
        lesson_reuse = item.get("action_trace_lesson_reuse_count") or "0"
        lesson_avoid = item.get("action_trace_lesson_avoid_count") or "0"
        lesson_top = item.get("action_trace_lesson_top_reuse") or item.get("action_trace_lesson_top_avoid") or "none"
        patch_strategy_status = item.get("action_trace_repair_patch_strategy_status") or "none"
        patch_strategy_focus = item.get("action_trace_repair_patch_strategy_focus") or "none"
        command_strategy_status = item.get("action_trace_command_strategy_status") or "none"
        command_strategy_focus = item.get("action_trace_command_strategy_focus") or "none"
        adaptation_status = item.get("action_trace_harness_adaptation_status") or "none"
        adaptation_focus = item.get("action_trace_harness_adaptation_focus") or "none"
        drift_status = item.get("action_trace_harness_environment_drift_status") or "none"
        drift_detail = item.get("action_trace_harness_environment_drift_detail") or "none"
        drift_history = item.get("action_trace_harness_environment_drift_history_count") or "0"
        lines.extend(
            [
                f"- `{status}` target=`{target}` candidate=`{candidate}` origin=`{origin}` session=`{session}`",
                f"  summary: {compact(item.get('summary', ''), 320)}",
                f"  draft_used: status=`{draft_status}` model=`{compact(draft_model, 120)}`",
                f"  command_used: {compact(command_hint, 220)}",
                f"  action_trace: status=`{action_status}` stages=`{action_count}` last=`{compact(action_last, 120)}` hint=`{compact(action_hint, 160)}`",
                f"  recall_used: matches=`{recall_count}` top=`{compact(recall_status, 80)}` reasons=`{compact(recall_reasons, 160)}`",
                f"  lesson_used: count=`{lesson_count}` reuse=`{lesson_reuse}` avoid=`{lesson_avoid}` top=`{compact(lesson_top, 160)}`",
                f"  patch_strategy_used: status=`{patch_strategy_status}` focus=`{compact(patch_strategy_focus, 180)}`",
                f"  command_strategy_used: status=`{command_strategy_status}` focus=`{compact(command_strategy_focus, 180)}`",
                f"  adaptation_used: status=`{adaptation_status}` focus=`{compact(adaptation_focus, 180)}`",
                f"  environment_drift_used: status=`{drift_status}` history=`{drift_history}` detail=`{compact(drift_detail, 180)}`",
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


def build_action_trace_effectiveness(outcomes):
    used = []
    hints = {}
    counts = {
        "satisfied": 0,
        "partial": 0,
        "failed": 0,
        "unknown": 0,
    }
    for outcome in outcomes:
        action_status = str(outcome.get("action_trace_status") or "").strip()
        action_hint = compact(outcome.get("action_trace_repair_hint") or "", 180)
        failed_stage = compact(outcome.get("action_trace_failed_stage") or "", 120)
        last_action = compact(outcome.get("action_trace_last_action") or "", 160)
        action_parts = [
            f"status={action_status}" if action_status else "",
            f"failed_stage={failed_stage}" if failed_stage else "",
            f"hint={action_hint}" if action_hint else "",
            f"last={last_action}" if last_action else "",
        ]
        action_hint = compact(" | ".join(part for part in action_parts if part), 320)
        if not action_hint:
            continue
        status = outcome.get("outcome_status") or "unknown"
        if status not in counts:
            status = "unknown"
        counts[status] += 1
        used.append(outcome)
        if status == "satisfied":
            direction = "reuse"
        elif status in {"partial", "failed"}:
            direction = "avoid"
        else:
            direction = "observe"
        add_effectiveness_hint(
            hints,
            direction,
            action_hint,
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
        "schema_version": "octopus-harness-action-trace-effectiveness-v1",
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


def build_repair_draft_effectiveness(outcomes):
    used = []
    hints = {}
    counts = {
        "satisfied": 0,
        "partial": 0,
        "failed": 0,
        "unknown": 0,
    }
    for outcome in outcomes:
        draft_status = str(outcome.get("draft_status") or "").strip()
        if draft_status != "generated":
            continue
        status = outcome.get("outcome_status") or "unknown"
        if status not in counts:
            status = "unknown"
        counts[status] += 1
        used.append(outcome)
        if status == "satisfied":
            direction = "reuse"
        elif status in {"partial", "failed"}:
            direction = "avoid"
        else:
            direction = "observe"
        add_effectiveness_hint(
            hints,
            direction,
            outcome.get("draft_model") or outcome.get("draft_prefix") or draft_status,
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
        "schema_version": "octopus-harness-repair-draft-effectiveness-v1",
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


def build_repair_draft_strategy(draft, draft_effectiveness, target_tentacle, target_tool):
    used_count = int_count(draft_effectiveness.get("used_count"))
    satisfied_count = int_count(draft_effectiveness.get("satisfied_count"))
    partial_count = int_count(draft_effectiveness.get("partial_count"))
    failed_count = int_count(draft_effectiveness.get("failed_count"))
    learned_reuse = compact(draft_effectiveness.get("top_reuse") or "", 260)
    learned_avoid = compact(draft_effectiveness.get("top_avoid") or "", 260)
    draft_status = str(draft.get("status") or "")
    draft_prefix = str(draft.get("prefix") or "")
    draft_model = str(draft.get("model") or "")
    if draft_status in {"missing_config", "failed"}:
        status = "repair_provider_draft_config"
        focus = compact(draft.get("content") or "provider repair draft is unavailable", 260)
        next_kind = "execute"
        next_query = f"configure repair provider draft: {draft_prefix or 'OCTOPUS_LLM'}"
    elif used_count > 0 and failed_count > satisfied_count:
        status = "inspect_unreliable_repair_draft"
        focus = learned_avoid or learned_reuse or "provider repair draft needs inspection"
        next_kind = "execute"
        next_query = "octopus repair . after unreliable provider repair draft"
    elif used_count > 0 and partial_count > 0 and satisfied_count == 0:
        status = "tighten_partial_repair_draft"
        focus = learned_avoid or learned_reuse or "partial provider repair draft needs tighter evidence"
        next_kind = "execute"
        next_query = "octopus repair . after partial provider repair draft"
    elif used_count > 0 and satisfied_count >= failed_count and learned_reuse:
        status = "reuse_effective_repair_draft"
        focus = learned_reuse
        next_kind = "verify"
        next_query = "review repair plan with effective provider draft"
    elif draft_status == "generated":
        status = "review_generated_repair_draft"
        focus = compact(draft.get("content") or draft_model or draft_prefix, 260)
        next_kind = "verify"
        next_query = "review generated provider repair draft"
    else:
        status = "collect_provider_draft_outcomes"
        focus = "score repair outcomes that used provider-generated drafts"
        next_kind = "verify"
        next_query = "collect reviewed provider repair draft outcomes"
    return {
        "schema_version": "octopus-harness-repair-draft-strategy-v1",
        "status": status,
        "focus": compact(focus, 260),
        "target_tentacle": target_tentacle,
        "target_tool": target_tool,
        "current_draft": {
            "status": draft_status,
            "prefix": draft_prefix,
            "model": draft_model,
            "has_content": "true" if bool(draft.get("content")) else "false",
        },
        "learned_reuse": learned_reuse,
        "learned_avoid": learned_avoid,
        "next_need": {
            "kind": next_kind,
            "query": next_query,
        },
        "evidence": {
            "repair_draft_effectiveness_used_count": used_count,
            "repair_draft_effectiveness_satisfied_count": satisfied_count,
            "repair_draft_effectiveness_partial_count": partial_count,
            "repair_draft_effectiveness_failed_count": failed_count,
            "repair_draft_effectiveness_success_rate": draft_effectiveness.get("success_rate") or "0.00",
            "repair_draft_effectiveness_failure_rate": draft_effectiveness.get("failure_rate") or "0.00",
        },
    }


def build_repair_patch_draft_effectiveness(outcomes):
    used = []
    hints = {}
    counts = {
        "satisfied": 0,
        "partial": 0,
        "failed": 0,
        "unknown": 0,
    }
    for outcome in outcomes:
        patch_status = str(outcome.get("action_trace_repair_patch_draft_status") or "").strip()
        if patch_status not in {"patch_attached", "narrative_only"}:
            continue
        patch_hint = compact(
            " | ".join(
                part
                for part in [
                    patch_status,
                    outcome.get("action_trace_repair_patch_draft_target"),
                    outcome.get("action_trace_repair_patch_draft_preview"),
                ]
                if part
            ),
            320,
        )
        if not patch_hint:
            continue
        status = outcome.get("outcome_status") or "unknown"
        if status not in counts:
            status = "unknown"
        counts[status] += 1
        used.append(outcome)
        if status == "satisfied":
            direction = "reuse"
        elif status in {"partial", "failed"}:
            direction = "avoid"
        else:
            direction = "observe"
        add_effectiveness_hint(
            hints,
            direction,
            patch_hint,
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
        "schema_version": "octopus-harness-repair-patch-draft-effectiveness-v1",
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


def build_repair_patch_review_effectiveness(outcomes):
    used = []
    hints = {}
    counts = {
        "satisfied": 0,
        "partial": 0,
        "failed": 0,
        "unknown": 0,
    }
    for outcome in outcomes:
        review_status = str(outcome.get("action_trace_repair_patch_review_status") or "").strip()
        check_status = str(outcome.get("action_trace_repair_patch_review_check_status") or "").strip()
        if review_status not in {"check_passed", "check_failed", "unchecked"}:
            continue
        review_hint = compact(
            " | ".join(
                part
                for part in [
                    review_status,
                    check_status,
                    outcome.get("action_trace_repair_patch_review_target"),
                    outcome.get("action_trace_repair_patch_review_summary"),
                ]
                if part
            ),
            320,
        )
        if not review_hint:
            continue
        status = outcome.get("outcome_status") or "unknown"
        if status not in counts:
            status = "unknown"
        counts[status] += 1
        used.append(outcome)
        if status == "satisfied":
            direction = "reuse"
        elif status in {"partial", "failed"}:
            direction = "avoid"
        else:
            direction = "observe"
        add_effectiveness_hint(
            hints,
            direction,
            review_hint,
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
        "schema_version": "octopus-harness-repair-patch-review-effectiveness-v1",
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


def build_repair_patch_learning(outcomes):
    used = []
    hints = {}
    counts = {
        "satisfied": 0,
        "partial": 0,
        "failed": 0,
        "unknown": 0,
    }
    verified_count = 0
    verified_satisfied_count = 0
    verified_failed_count = 0
    for outcome in outcomes:
        apply_status = str(outcome.get("action_trace_repair_patch_apply_status") or "").strip()
        apply_applied = str(outcome.get("action_trace_repair_patch_apply_applied") or "").strip()
        verify_status = str(outcome.get("action_trace_repair_patch_verify_status") or "").strip()
        verify_passed = str(outcome.get("action_trace_repair_patch_verify_passed") or "").strip()
        target = (
            outcome.get("action_trace_repair_patch_verify_target")
            or outcome.get("action_trace_repair_patch_apply_target")
            or outcome.get("action_trace_repair_patch_review_target")
            or ""
        )
        if not any([apply_status, apply_applied, verify_status, verify_passed]):
            continue
        is_verified = verify_status == "passed" and verify_passed == "true"
        if is_verified:
            verified_count += 1
        patch_hint = compact(
            " | ".join(
                part
                for part in [
                    f"apply={apply_status or 'unknown'}",
                    f"applied={apply_applied or 'unknown'}",
                    f"verify={verify_status or 'unknown'}",
                    f"passed={verify_passed or 'unknown'}",
                    target,
                    outcome.get("action_trace_repair_patch_verify_summary"),
                    outcome.get("action_trace_repair_patch_apply_summary"),
                ]
                if part
            ),
            360,
        )
        if not patch_hint:
            continue
        status = outcome.get("outcome_status") or "unknown"
        if status not in counts:
            status = "unknown"
        counts[status] += 1
        used.append(outcome)
        if is_verified and status == "satisfied":
            verified_satisfied_count += 1
            direction = "reuse"
        elif status in {"partial", "failed"} or verify_status in {"failed", "failed_to_start"}:
            if is_verified:
                verified_failed_count += 1
            direction = "avoid"
        else:
            direction = "observe"
        add_effectiveness_hint(
            hints,
            direction,
            patch_hint,
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
    top_reuse = reuse[0]["hint"] if reuse else ""
    top_avoid = avoid[0]["hint"] if avoid else ""
    if top_reuse:
        status = "reuse_verified_patch"
        next_need = {
            "kind": "remember",
            "query": f"reuse verified repair patch pattern: {compact(top_reuse, 180)}",
        }
    elif top_avoid:
        status = "inspect_patch_risk"
        next_need = {
            "kind": "verify",
            "query": f"inspect repair patch learning risk: {compact(top_avoid, 180)}",
        }
    else:
        status = "collect_verified_patch_outcomes"
        next_need = {
            "kind": "observe",
            "query": "collect scored repair patch verification outcomes",
        }
    return {
        "schema_version": "octopus-harness-repair-patch-learning-v1",
        "status": status,
        "used_count": used_count,
        "verified_count": verified_count,
        "verified_satisfied_count": verified_satisfied_count,
        "verified_failed_count": verified_failed_count,
        "satisfied_count": counts["satisfied"],
        "partial_count": counts["partial"],
        "failed_count": counts["failed"],
        "unknown_count": counts["unknown"],
        "success_rate": rate_label(counts["satisfied"], used_count),
        "verified_success_rate": rate_label(verified_satisfied_count, verified_count),
        "failure_rate": rate_label(counts["failed"], used_count),
        "top_reuse": top_reuse,
        "top_avoid": top_avoid,
        "next_need": next_need,
        "hints": hint_rows[:8],
    }


def build_repair_patch_learning_effectiveness(outcomes):
    used = []
    hints = {}
    counts = {
        "satisfied": 0,
        "partial": 0,
        "failed": 0,
        "unknown": 0,
    }
    for outcome in outcomes:
        learning_status = str(outcome.get("action_trace_repair_patch_learning_status") or "").strip()
        learning_used = str(outcome.get("action_trace_repair_patch_learning_used_count") or "").strip()
        learning_verified = str(outcome.get("action_trace_repair_patch_learning_verified_count") or "").strip()
        top_reuse = outcome.get("action_trace_repair_patch_learning_top_reuse") or ""
        top_avoid = outcome.get("action_trace_repair_patch_learning_top_avoid") or ""
        if int_count(learning_used) <= 0 and int_count(learning_verified) <= 0 and not top_reuse and not top_avoid:
            continue
        learning_hint = compact(
            " | ".join(
                part
                for part in [
                    learning_status,
                    f"used={learning_used or '0'}",
                    f"verified={learning_verified or '0'}",
                    top_reuse,
                    top_avoid,
                ]
                if part
            ),
            360,
        )
        if not learning_hint:
            continue
        status = outcome.get("outcome_status") or "unknown"
        if status not in counts:
            status = "unknown"
        counts[status] += 1
        used.append(outcome)
        if status == "satisfied" and learning_status == "reuse_verified_patch":
            direction = "reuse"
        elif status in {"partial", "failed"}:
            direction = "avoid"
        else:
            direction = "observe"
        add_effectiveness_hint(
            hints,
            direction,
            learning_hint,
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
        "schema_version": "octopus-harness-repair-patch-learning-effectiveness-v1",
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


def build_repair_command_effectiveness(outcomes):
    used = []
    hints = {}
    counts = {
        "satisfied": 0,
        "partial": 0,
        "failed": 0,
        "unknown": 0,
    }
    for outcome in outcomes:
        command_hint = (
            outcome.get("repair_command_apply")
            or outcome.get("repair_command_check")
            or outcome.get("repair_command_score")
            or outcome.get("repair_command_score_options")
            or ""
        )
        if not command_hint:
            continue
        status = outcome.get("outcome_status") or "unknown"
        if status not in counts:
            status = "unknown"
        counts[status] += 1
        used.append(outcome)
        if status == "satisfied":
            direction = "reuse"
        elif status in {"partial", "failed"}:
            direction = "avoid"
        else:
            direction = "observe"
        add_effectiveness_hint(
            hints,
            direction,
            command_hint,
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
        "schema_version": "octopus-harness-repair-command-effectiveness-v1",
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


def build_repair_command_strategy_effectiveness(outcomes):
    used = []
    hints = {}
    counts = {
        "satisfied": 0,
        "partial": 0,
        "failed": 0,
        "unknown": 0,
    }
    for outcome in outcomes:
        strategy_parts = [
            outcome.get("action_trace_command_strategy_status"),
            outcome.get("action_trace_command_strategy_focus"),
            outcome.get("action_trace_command_strategy_learned_reuse"),
            outcome.get("action_trace_command_strategy_learned_avoid"),
        ]
        strategy_hint = compact(
            " | ".join(str(part) for part in strategy_parts if part),
            320,
        )
        if not strategy_hint:
            continue
        status = outcome.get("outcome_status") or "unknown"
        if status not in counts:
            status = "unknown"
        counts[status] += 1
        used.append(outcome)
        if status == "satisfied":
            direction = "reuse"
        elif status in {"partial", "failed"}:
            direction = "avoid"
        else:
            direction = "observe"
        add_effectiveness_hint(
            hints,
            direction,
            strategy_hint,
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
        "schema_version": "octopus-harness-repair-command-strategy-effectiveness-v1",
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


def build_repair_patch_strategy_effectiveness(outcomes):
    used = []
    hints = {}
    counts = {
        "satisfied": 0,
        "partial": 0,
        "failed": 0,
        "unknown": 0,
    }
    for outcome in outcomes:
        strategy_status = str(outcome.get("action_trace_repair_patch_strategy_status") or "").strip()
        strategy_focus = outcome.get("action_trace_repair_patch_strategy_focus") or ""
        learned_reuse = outcome.get("action_trace_repair_patch_strategy_learned_reuse") or ""
        learned_avoid = outcome.get("action_trace_repair_patch_strategy_learned_avoid") or ""
        next_query = outcome.get("action_trace_repair_patch_strategy_next_need_query") or ""
        if not any([strategy_status, strategy_focus, learned_reuse, learned_avoid, next_query]):
            continue
        strategy_hint = compact(
            " | ".join(
                part
                for part in [
                    strategy_status,
                    strategy_focus,
                    learned_reuse,
                    learned_avoid,
                    next_query,
                ]
                if part
            ),
            360,
        )
        if not strategy_hint:
            continue
        status = outcome.get("outcome_status") or "unknown"
        if status not in counts:
            status = "unknown"
        counts[status] += 1
        used.append(outcome)
        if status == "satisfied":
            direction = "reuse"
        elif status in {"partial", "failed"}:
            direction = "avoid"
        else:
            direction = "observe"
        add_effectiveness_hint(
            hints,
            direction,
            strategy_hint,
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
        "schema_version": "octopus-harness-repair-patch-strategy-effectiveness-v1",
        "used_count": used_count,
        "satisfied_count": counts["satisfied"],
        "partial_count": counts["partial"],
        "failed_count": counts["failed"],
        "unknown_count": counts["unknown"],
        "success_rate": rate_label(counts["satisfied"], used_count),
        "failure_rate": rate_label(counts["failed"], used_count),
        "top_reuse": reuse[0]["hint"] if reuse else "",
        "top_avoid": avoid[0]["hint"] if avoid else "",
        "hints": hint_rows[:8],
    }


def build_repair_decision_effectiveness(outcomes):
    used = []
    hints = {}
    counts = {
        "satisfied": 0,
        "partial": 0,
        "failed": 0,
        "unknown": 0,
    }
    for outcome in outcomes:
        decision_parts = [
            outcome.get("action_trace_repair_decision"),
            outcome.get("action_trace_repair_decision_focus"),
        ]
        decision_hint = compact(
            " | ".join(str(part) for part in decision_parts if part),
            320,
        )
        if not decision_hint:
            continue
        status = outcome.get("outcome_status") or "unknown"
        if status not in counts:
            status = "unknown"
        counts[status] += 1
        used.append(outcome)
        if status == "satisfied":
            direction = "reuse"
        elif status in {"partial", "failed"}:
            direction = "avoid"
        else:
            direction = "observe"
        add_effectiveness_hint(
            hints,
            direction,
            decision_hint,
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
        "schema_version": "octopus-harness-repair-decision-effectiveness-v1",
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


def build_harness_adaptation_effectiveness(outcomes):
    used = []
    hints = {}
    counts = {
        "satisfied": 0,
        "partial": 0,
        "failed": 0,
        "unknown": 0,
    }
    for outcome in outcomes:
        adaptation_status = outcome.get("action_trace_harness_adaptation_status") or ""
        adaptation_focus = outcome.get("action_trace_harness_adaptation_focus") or ""
        if not adaptation_status and not adaptation_focus:
            continue
        status = outcome.get("outcome_status") or "unknown"
        if status not in counts:
            status = "unknown"
        counts[status] += 1
        used.append(outcome)
        if status == "satisfied":
            direction = "reuse"
        elif status in {"partial", "failed"}:
            direction = "avoid"
        else:
            direction = "observe"
        add_effectiveness_hint(
            hints,
            direction,
            adaptation_focus or adaptation_status,
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
        "schema_version": "octopus-harness-adaptation-effectiveness-v1",
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


def build_harness_environment_drift_effectiveness(outcomes):
    used = []
    hints = {}
    counts = {
        "satisfied": 0,
        "partial": 0,
        "failed": 0,
        "unknown": 0,
    }
    for outcome in outcomes:
        drift_status = outcome.get("action_trace_harness_environment_drift_status") or ""
        drift_detail = outcome.get("action_trace_harness_environment_drift_detail") or ""
        if not drift_status and not drift_detail:
            continue
        status = outcome.get("outcome_status") or "unknown"
        if status not in counts:
            status = "unknown"
        counts[status] += 1
        used.append(outcome)
        if status == "satisfied":
            direction = "reuse"
        elif status in {"partial", "failed"}:
            direction = "avoid"
        else:
            direction = "observe"
        add_effectiveness_hint(
            hints,
            direction,
            drift_detail or drift_status,
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
        "schema_version": "octopus-harness-environment-drift-effectiveness-v1",
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


def harness_adaptation_effectiveness_markdown(effectiveness, workspace, effectiveness_json):
    lines = [
        "# Harness Adaptation Effectiveness",
        "",
        "This file measures whether prior harness adaptation focus helped later reviewed outcomes.",
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
        lines.append("No scored repair outcome has used a harness adaptation focus yet.")
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
                f"  focus: {compact(hint.get('hint'), 260)}",
                f"  latest: target=`{hint.get('latest_target') or 'unknown'}` candidate=`{hint.get('latest_candidate') or 'none'}`",
                f"  summary: {compact(hint.get('latest_summary'), 220) or 'none'}",
            ]
        )
    return "\n".join(lines) + "\n"


def repair_draft_effectiveness_markdown(effectiveness, workspace, effectiveness_json):
    lines = [
        "# Harness Repair Draft Effectiveness",
        "",
        "This file measures whether provider-generated repair drafts helped later reviewed outcomes.",
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
        lines.append("No scored repair outcome has used a provider-generated repair draft yet.")
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
                f"  provider: {compact(hint.get('hint'), 260)}",
                f"  latest: target=`{hint.get('latest_target') or 'unknown'}` candidate=`{hint.get('latest_candidate') or 'none'}`",
                f"  summary: {compact(hint.get('latest_summary'), 220) or 'none'}",
            ]
        )
    return "\n".join(lines) + "\n"


def repair_draft_strategy_markdown(strategy, workspace, strategy_json):
    evidence = strategy.get("evidence") if isinstance(strategy.get("evidence"), dict) else {}
    current = strategy.get("current_draft") if isinstance(strategy.get("current_draft"), dict) else {}
    next_need = strategy.get("next_need") if isinstance(strategy.get("next_need"), dict) else {}
    lines = [
        "# Harness Repair Draft Strategy",
        "",
        "This file compresses provider draft status and draft effectiveness into the next repair-draft focus.",
        "It is local tentacle Feed evidence, not clean-brain context.",
        "",
        f"json: `{rel(strategy_json, workspace)}`",
        f"status: `{strategy.get('status')}`",
        f"focus: {compact(strategy.get('focus'), 260)}",
        f"target: `{strategy.get('target_tentacle')}/{strategy.get('target_tool')}`",
        f"next_need: `{next_need.get('kind') or 'verify'} {next_need.get('query') or ''}`",
        "",
        "## Current Draft",
        "",
        f"- status: `{current.get('status') or 'none'}`",
        f"- prefix: `{current.get('prefix') or 'none'}`",
        f"- model: `{current.get('model') or 'none'}`",
        f"- has_content: `{current.get('has_content') or 'false'}`",
        "",
        "## Evidence",
        "",
        f"- draft_effectiveness_used: `{evidence.get('repair_draft_effectiveness_used_count', 0)}`",
        f"- draft_effectiveness_satisfied: `{evidence.get('repair_draft_effectiveness_satisfied_count', 0)}`",
        f"- draft_effectiveness_partial: `{evidence.get('repair_draft_effectiveness_partial_count', 0)}`",
        f"- draft_effectiveness_failed: `{evidence.get('repair_draft_effectiveness_failed_count', 0)}`",
        f"- draft_effectiveness_success: `{evidence.get('repair_draft_effectiveness_success_rate', '0.00')}`",
        f"- learned_reuse: {compact(strategy.get('learned_reuse'), 260) or 'none'}",
        f"- learned_avoid: {compact(strategy.get('learned_avoid'), 260) or 'none'}",
    ]
    return "\n".join(lines) + "\n"


def repair_patch_draft_effectiveness_markdown(effectiveness, workspace, effectiveness_json):
    lines = [
        "# Harness Repair Patch Draft Effectiveness",
        "",
        "This file measures whether provider repair patch drafts helped later reviewed outcomes.",
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
        lines.append("No scored repair outcome has used a provider repair patch draft yet.")
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
                f"  patch: {compact(hint.get('hint'), 260)}",
                f"  latest: target=`{hint.get('latest_target') or 'unknown'}` candidate=`{hint.get('latest_candidate') or 'none'}`",
                f"  summary: {compact(hint.get('latest_summary'), 220) or 'none'}",
            ]
        )
    return "\n".join(lines) + "\n"


def repair_patch_review_effectiveness_markdown(effectiveness, workspace, effectiveness_json):
    lines = [
        "# Harness Repair Patch Review Effectiveness",
        "",
        "This file measures whether local provider patch pre-review helped later reviewed outcomes.",
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
        lines.append("No scored repair outcome has used local provider patch review yet.")
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
                f"  review: {compact(hint.get('hint'), 260)}",
                f"  latest: target=`{hint.get('latest_target') or 'unknown'}` candidate=`{hint.get('latest_candidate') or 'none'}`",
                f"  summary: {compact(hint.get('latest_summary'), 220) or 'none'}",
            ]
        )
    return "\n".join(lines) + "\n"


def repair_patch_learning_markdown(learning, workspace, learning_json):
    next_need = learning.get("next_need") if isinstance(learning.get("next_need"), dict) else {}
    lines = [
        "# Harness Repair Patch Learning",
        "",
        "This file compresses scored apply+verify patch outcomes into reusable harness evidence.",
        "It is local tentacle Feed evidence, not clean-brain context.",
        "",
        f"json: `{rel(learning_json, workspace)}`",
        f"status: `{learning.get('status')}`",
        f"used_count: `{learning.get('used_count', 0)}`",
        f"verified_count: `{learning.get('verified_count', 0)}`",
        f"verified_satisfied_count: `{learning.get('verified_satisfied_count', 0)}`",
        f"verified_failed_count: `{learning.get('verified_failed_count', 0)}`",
        f"success_rate: `{learning.get('success_rate', '0.00')}`",
        f"verified_success_rate: `{learning.get('verified_success_rate', '0.00')}`",
        f"top_reuse: {compact(learning.get('top_reuse') or '', 260) or 'none'}",
        f"top_avoid: {compact(learning.get('top_avoid') or '', 260) or 'none'}",
        f"next_need: `{next_need.get('kind') or 'observe'} {next_need.get('query') or ''}`",
        "",
    ]
    if not learning.get("hints"):
        lines.append("No scored repair patch verification outcome has been recorded yet.")
        return "\n".join(lines) + "\n"
    lines.extend(["## Hints", ""])
    for hint in learning["hints"]:
        lines.extend(
            [
                (
                    f"- `{hint.get('direction')}` used=`{hint.get('used_count')}` "
                    f"satisfied=`{hint.get('satisfied_count')}` partial=`{hint.get('partial_count')}` "
                    f"failed=`{hint.get('failed_count')}` success_rate=`{hint.get('success_rate')}`"
                ),
                f"  patch: {compact(hint.get('hint'), 260)}",
                f"  latest: target=`{hint.get('latest_target') or 'unknown'}` candidate=`{hint.get('latest_candidate') or 'none'}`",
                f"  summary: {compact(hint.get('latest_summary'), 220) or 'none'}",
            ]
        )
    return "\n".join(lines) + "\n"


def repair_patch_learning_effectiveness_markdown(effectiveness, workspace, effectiveness_json):
    lines = [
        "# Harness Repair Patch Learning Effectiveness",
        "",
        "This file measures whether patch-learning evidence helped later reviewed repair outcomes.",
        "It is local tentacle Feed evidence, not clean-brain context.",
        "",
        f"json: `{rel(effectiveness_json, workspace)}`",
        f"used_count: `{effectiveness.get('used_count', 0)}`",
        f"satisfied_count: `{effectiveness.get('satisfied_count', 0)}`",
        f"partial_count: `{effectiveness.get('partial_count', 0)}`",
        f"failed_count: `{effectiveness.get('failed_count', 0)}`",
        f"success_rate: `{effectiveness.get('success_rate', '0.00')}`",
        f"failure_rate: `{effectiveness.get('failure_rate', '0.00')}`",
        f"top_reuse: {compact(effectiveness.get('top_reuse') or '', 260) or 'none'}",
        f"top_avoid: {compact(effectiveness.get('top_avoid') or '', 260) or 'none'}",
        "",
    ]
    if not effectiveness.get("hints"):
        lines.append("No scored repair outcome has used patch-learning evidence yet.")
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
                f"  learning: {compact(hint.get('hint'), 260)}",
                f"  latest: target=`{hint.get('latest_target') or 'unknown'}` candidate=`{hint.get('latest_candidate') or 'none'}`",
                f"  summary: {compact(hint.get('latest_summary'), 220) or 'none'}",
            ]
        )
    return "\n".join(lines) + "\n"


def build_repair_patch_strategy(
    patch_draft,
    patch_review,
    patch_apply,
    patch_verify,
    patch_learning,
    patch_learning_effectiveness,
    patch_strategy_effectiveness,
    target_tentacle,
    target_tool,
):
    draft_target = patch_draft.get("target") if isinstance(patch_draft.get("target"), dict) else {}
    review_target = patch_review.get("target") if isinstance(patch_review.get("target"), dict) else {}
    apply_target = patch_apply.get("target") if isinstance(patch_apply.get("target"), dict) else {}
    verify_target = patch_verify.get("target") if isinstance(patch_verify.get("target"), dict) else {}
    learning_next = patch_learning.get("next_need") if isinstance(patch_learning.get("next_need"), dict) else {}
    used_count = int_count(patch_learning_effectiveness.get("used_count"))
    satisfied_count = int_count(patch_learning_effectiveness.get("satisfied_count"))
    partial_count = int_count(patch_learning_effectiveness.get("partial_count"))
    failed_count = int_count(patch_learning_effectiveness.get("failed_count"))
    strategy_used_count = int_count(patch_strategy_effectiveness.get("used_count"))
    strategy_satisfied_count = int_count(patch_strategy_effectiveness.get("satisfied_count"))
    strategy_partial_count = int_count(patch_strategy_effectiveness.get("partial_count"))
    strategy_failed_count = int_count(patch_strategy_effectiveness.get("failed_count"))
    learned_reuse = compact(
        patch_learning_effectiveness.get("top_reuse") or patch_learning.get("top_reuse") or "",
        260,
    )
    learned_avoid = compact(
        patch_learning_effectiveness.get("top_avoid") or patch_learning.get("top_avoid") or "",
        260,
    )
    strategy_learned_reuse = compact(patch_strategy_effectiveness.get("top_reuse") or "", 260)
    strategy_learned_avoid = compact(patch_strategy_effectiveness.get("top_avoid") or "", 260)
    draft_status = str(patch_draft.get("status") or "")
    review_status = str(patch_review.get("status") or "")
    apply_status = str(patch_apply.get("status") or "")
    verify_status = str(patch_verify.get("status") or "")
    learning_status = str(patch_learning.get("status") or "")
    has_patch = bool(patch_draft.get("has_patch") or patch_review.get("has_patch"))
    applied = bool(patch_apply.get("applied"))
    verified = bool(patch_verify.get("passed"))
    if strategy_used_count > 0 and strategy_failed_count > strategy_satisfied_count:
        status = "inspect_unreliable_patch_strategy"
        focus = strategy_learned_avoid or strategy_learned_reuse or "repair patch strategy needs inspection"
        next_kind = "execute"
        next_query = "octopus repair . after unreliable repair patch strategy"
    elif strategy_used_count > 0 and strategy_partial_count > 0 and strategy_satisfied_count == 0:
        status = "tighten_partial_patch_strategy"
        focus = strategy_learned_avoid or strategy_learned_reuse or "partial repair patch strategy needs tighter evidence"
        next_kind = "execute"
        next_query = "octopus repair . after partial repair patch strategy"
    elif strategy_used_count > 0 and strategy_satisfied_count >= strategy_failed_count and strategy_learned_reuse:
        status = "reuse_effective_patch_strategy"
        focus = strategy_learned_reuse
        next_kind = "verify"
        next_query = "review repair plan with effective patch strategy"
    elif used_count > 0 and failed_count > satisfied_count:
        status = "inspect_unreliable_patch_learning"
        focus = learned_avoid or learned_reuse or "repair patch learning needs inspection"
        next_kind = "execute"
        next_query = "octopus repair . after unreliable repair patch learning"
    elif used_count > 0 and partial_count > 0 and satisfied_count == 0:
        status = "tighten_partial_patch_learning"
        focus = learned_avoid or learned_reuse or "partial repair patch learning needs tighter evidence"
        next_kind = "execute"
        next_query = "octopus repair . after partial repair patch learning"
    elif apply_status in {"failed", "check_failed", "failed_to_start"}:
        status = "inspect_patch_apply_failure"
        focus = compact(patch_apply.get("summary") or "repair patch apply failed", 260)
        next_kind = "execute"
        next_query = "repair failed repair patch apply"
    elif review_status == "check_failed":
        status = "inspect_patch_review_failure"
        focus = compact(patch_review.get("summary") or "repair patch review failed", 260)
        next_kind = "execute"
        next_query = "repair failed repair patch review"
    elif learning_status == "reuse_verified_patch" and learned_reuse:
        status = "reuse_verified_patch_learning"
        focus = learned_reuse
        next_kind = "verify"
        next_query = "review repair plan with verified patch learning"
    elif has_patch and review_status == "check_passed" and not applied:
        status = "review_checked_patch"
        focus = compact(patch_review.get("summary") or patch_draft.get("patch_preview") or "", 260)
        next_kind = "verify"
        next_query = "review checked repair patch before authorized apply"
    elif applied and not verified:
        status = "verify_applied_patch"
        focus = compact(patch_apply.get("summary") or "applied repair patch needs verification", 260)
        next_kind = "verify"
        next_query = "run post-apply repair patch verification"
    elif has_patch:
        status = "review_provider_patch"
        focus = compact(patch_draft.get("patch_preview") or patch_draft.get("proposal_preview") or "", 260)
        next_kind = "verify"
        next_query = "review provider repair patch"
    elif learning_next.get("query") and learning_status != "collect_verified_patch_outcomes":
        status = learning_status or "follow_patch_learning"
        focus = learned_reuse or learned_avoid or compact(learning_next.get("query"), 260)
        next_kind = str(learning_next.get("kind") or "verify")
        next_query = str(learning_next.get("query") or "review repair patch learning")
    else:
        status = "collect_patch_outcomes"
        focus = "score repair outcomes that include patch draft, review, apply, verify, and learning evidence"
        next_kind = "verify"
        next_query = "collect reviewed repair patch outcomes"
    return {
        "schema_version": "octopus-harness-repair-patch-strategy-v1",
        "status": status,
        "focus": compact(focus, 260),
        "target_tentacle": target_tentacle,
        "target_tool": target_tool,
        "current_patch": {
            "draft_status": draft_status,
            "draft_has_patch": "true" if patch_draft.get("has_patch") else "false",
            "draft_target": "/".join(
                item for item in [str(draft_target.get("tentacle") or ""), str(draft_target.get("tool") or "")] if item
            ),
            "review_status": review_status,
            "review_check_status": str(patch_review.get("check_status") or ""),
            "review_target": "/".join(
                item for item in [str(review_target.get("tentacle") or ""), str(review_target.get("tool") or "")] if item
            ),
            "apply_status": apply_status,
            "apply_applied": "true" if applied else "false",
            "apply_target": "/".join(
                item for item in [str(apply_target.get("tentacle") or ""), str(apply_target.get("tool") or "")] if item
            ),
            "verify_status": verify_status,
            "verify_passed": "true" if verified else "false",
            "verify_target": "/".join(
                item for item in [str(verify_target.get("tentacle") or ""), str(verify_target.get("tool") or "")] if item
            ),
            "learning_status": learning_status,
        },
        "learned_reuse": learned_reuse,
        "learned_avoid": learned_avoid,
        "strategy_learned_reuse": strategy_learned_reuse,
        "strategy_learned_avoid": strategy_learned_avoid,
        "next_need": {
            "kind": next_kind,
            "query": next_query,
        },
        "evidence": {
            "patch_learning_effectiveness_used_count": used_count,
            "patch_learning_effectiveness_satisfied_count": satisfied_count,
            "patch_learning_effectiveness_partial_count": partial_count,
            "patch_learning_effectiveness_failed_count": failed_count,
            "patch_learning_effectiveness_success_rate": patch_learning_effectiveness.get("success_rate") or "0.00",
            "patch_learning_effectiveness_failure_rate": patch_learning_effectiveness.get("failure_rate") or "0.00",
            "patch_learning_used_count": int_count(patch_learning.get("used_count")),
            "patch_learning_verified_count": int_count(patch_learning.get("verified_count")),
            "patch_learning_verified_success_rate": patch_learning.get("verified_success_rate") or "0.00",
            "patch_strategy_effectiveness_used_count": strategy_used_count,
            "patch_strategy_effectiveness_satisfied_count": strategy_satisfied_count,
            "patch_strategy_effectiveness_partial_count": strategy_partial_count,
            "patch_strategy_effectiveness_failed_count": strategy_failed_count,
            "patch_strategy_effectiveness_success_rate": patch_strategy_effectiveness.get("success_rate") or "0.00",
            "patch_strategy_effectiveness_failure_rate": patch_strategy_effectiveness.get("failure_rate") or "0.00",
        },
    }


def repair_patch_strategy_markdown(strategy, workspace, strategy_json):
    evidence = strategy.get("evidence") if isinstance(strategy.get("evidence"), dict) else {}
    current = strategy.get("current_patch") if isinstance(strategy.get("current_patch"), dict) else {}
    next_need = strategy.get("next_need") if isinstance(strategy.get("next_need"), dict) else {}
    lines = [
        "# Harness Repair Patch Strategy",
        "",
        "This file compresses patch draft/review/apply/verify/learning evidence into the next patch focus.",
        "It is local tentacle Feed evidence, not clean-brain context.",
        "",
        f"json: `{rel(strategy_json, workspace)}`",
        f"status: `{strategy.get('status')}`",
        f"focus: {compact(strategy.get('focus'), 260)}",
        f"target: `{strategy.get('target_tentacle')}/{strategy.get('target_tool')}`",
        f"next_need: `{next_need.get('kind') or 'verify'} {next_need.get('query') or ''}`",
        "",
        "## Current Patch",
        "",
        f"- draft: `{current.get('draft_status') or 'none'}` has_patch=`{current.get('draft_has_patch') or 'false'}` target=`{current.get('draft_target') or 'unknown'}`",
        f"- review: `{current.get('review_status') or 'none'}` check=`{current.get('review_check_status') or 'none'}` target=`{current.get('review_target') or 'unknown'}`",
        f"- apply: `{current.get('apply_status') or 'none'}` applied=`{current.get('apply_applied') or 'false'}` target=`{current.get('apply_target') or 'unknown'}`",
        f"- verify: `{current.get('verify_status') or 'none'}` passed=`{current.get('verify_passed') or 'false'}` target=`{current.get('verify_target') or 'unknown'}`",
        f"- learning: `{current.get('learning_status') or 'none'}`",
        "",
        "## Evidence",
        "",
        f"- patch_learning_effectiveness_used: `{evidence.get('patch_learning_effectiveness_used_count', 0)}`",
        f"- patch_learning_effectiveness_satisfied: `{evidence.get('patch_learning_effectiveness_satisfied_count', 0)}`",
        f"- patch_learning_effectiveness_partial: `{evidence.get('patch_learning_effectiveness_partial_count', 0)}`",
        f"- patch_learning_effectiveness_failed: `{evidence.get('patch_learning_effectiveness_failed_count', 0)}`",
        f"- patch_learning_effectiveness_success_rate: `{evidence.get('patch_learning_effectiveness_success_rate', '0.00')}`",
        f"- patch_learning_effectiveness_failure_rate: `{evidence.get('patch_learning_effectiveness_failure_rate', '0.00')}`",
        f"- patch_learning_used: `{evidence.get('patch_learning_used_count', 0)}`",
        f"- patch_learning_verified: `{evidence.get('patch_learning_verified_count', 0)}`",
        f"- patch_learning_verified_success_rate: `{evidence.get('patch_learning_verified_success_rate', '0.00')}`",
        f"- patch_strategy_effectiveness_used: `{evidence.get('patch_strategy_effectiveness_used_count', 0)}`",
        f"- patch_strategy_effectiveness_satisfied: `{evidence.get('patch_strategy_effectiveness_satisfied_count', 0)}`",
        f"- patch_strategy_effectiveness_partial: `{evidence.get('patch_strategy_effectiveness_partial_count', 0)}`",
        f"- patch_strategy_effectiveness_failed: `{evidence.get('patch_strategy_effectiveness_failed_count', 0)}`",
        f"- patch_strategy_effectiveness_success_rate: `{evidence.get('patch_strategy_effectiveness_success_rate', '0.00')}`",
        f"- patch_strategy_effectiveness_failure_rate: `{evidence.get('patch_strategy_effectiveness_failure_rate', '0.00')}`",
        f"- learned_reuse: {compact(strategy.get('learned_reuse'), 260) or 'none'}",
        f"- learned_avoid: {compact(strategy.get('learned_avoid'), 260) or 'none'}",
        f"- strategy_learned_reuse: {compact(strategy.get('strategy_learned_reuse'), 260) or 'none'}",
        f"- strategy_learned_avoid: {compact(strategy.get('strategy_learned_avoid'), 260) or 'none'}",
    ]
    return "\n".join(lines) + "\n"


def repair_patch_strategy_effectiveness_markdown(effectiveness, workspace, effectiveness_json):
    lines = [
        "# Harness Repair Patch Strategy Effectiveness",
        "",
        "This file measures whether prior repair patch strategies helped later reviewed outcomes.",
        "It is local tentacle Feed evidence, not clean-brain context.",
        "",
        f"json: `{rel(effectiveness_json, workspace)}`",
        f"used_count: `{effectiveness.get('used_count', 0)}`",
        f"satisfied_count: `{effectiveness.get('satisfied_count', 0)}`",
        f"partial_count: `{effectiveness.get('partial_count', 0)}`",
        f"failed_count: `{effectiveness.get('failed_count', 0)}`",
        f"success_rate: `{effectiveness.get('success_rate', '0.00')}`",
        f"failure_rate: `{effectiveness.get('failure_rate', '0.00')}`",
        f"top_reuse: {compact(effectiveness.get('top_reuse') or '', 260) or 'none'}",
        f"top_avoid: {compact(effectiveness.get('top_avoid') or '', 260) or 'none'}",
        "",
    ]
    if not effectiveness.get("hints"):
        lines.append("No scored repair outcome has used a patch strategy yet.")
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
                f"  strategy: {compact(hint.get('hint'), 260)}",
                f"  latest: target=`{hint.get('latest_target') or 'unknown'}` candidate=`{hint.get('latest_candidate') or 'none'}`",
                f"  summary: {compact(hint.get('latest_summary'), 220) or 'none'}",
            ]
        )
    return "\n".join(lines) + "\n"


def repair_command_effectiveness_markdown(effectiveness, workspace, effectiveness_json):
    lines = [
        "# Harness Repair Command Effectiveness",
        "",
        "This file measures whether reviewable repair command recipes helped later reviewed outcomes.",
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
        lines.append("No scored repair outcome has used a recorded repair command recipe yet.")
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
                f"  command: {compact(hint.get('hint'), 260)}",
                f"  latest: target=`{hint.get('latest_target') or 'unknown'}` candidate=`{hint.get('latest_candidate') or 'none'}`",
                f"  summary: {compact(hint.get('latest_summary'), 220) or 'none'}",
            ]
        )
    return "\n".join(lines) + "\n"


def repair_command_strategy_effectiveness_markdown(effectiveness, workspace, effectiveness_json):
    lines = [
        "# Harness Repair Command Strategy Effectiveness",
        "",
        "This file measures whether prior repair command strategies helped later reviewed outcomes.",
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
        lines.append("No scored repair outcome has used a recorded repair command strategy yet.")
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
                f"  strategy: {compact(hint.get('hint'), 260)}",
                f"  latest: target=`{hint.get('latest_target') or 'unknown'}` candidate=`{hint.get('latest_candidate') or 'none'}`",
                f"  summary: {compact(hint.get('latest_summary'), 220) or 'none'}",
            ]
        )
    return "\n".join(lines) + "\n"


def repair_decision_effectiveness_markdown(effectiveness, workspace, effectiveness_json):
    lines = [
        "# Harness Repair Decision Effectiveness",
        "",
        "This file measures whether prior repair decisions helped later reviewed outcomes.",
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
        lines.append("No scored repair outcome has used a recorded repair decision yet.")
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
                f"  decision: {compact(hint.get('hint'), 260)}",
                f"  latest: target=`{hint.get('latest_target') or 'unknown'}` candidate=`{hint.get('latest_candidate') or 'none'}`",
                f"  summary: {compact(hint.get('latest_summary'), 220) or 'none'}",
            ]
        )
    return "\n".join(lines) + "\n"


def build_repair_command_strategy(effectiveness, strategy_effectiveness, command_recipe, target_tentacle, target_tool):
    used_count = int_count(effectiveness.get("used_count"))
    satisfied_count = int_count(effectiveness.get("satisfied_count"))
    partial_count = int_count(effectiveness.get("partial_count"))
    failed_count = int_count(effectiveness.get("failed_count"))
    strategy_used_count = int_count(strategy_effectiveness.get("used_count"))
    strategy_satisfied_count = int_count(strategy_effectiveness.get("satisfied_count"))
    strategy_partial_count = int_count(strategy_effectiveness.get("partial_count"))
    strategy_failed_count = int_count(strategy_effectiveness.get("failed_count"))
    top_reuse = compact(effectiveness.get("top_reuse") or "", 260)
    top_avoid = compact(effectiveness.get("top_avoid") or "", 260)
    strategy_top_reuse = compact(strategy_effectiveness.get("top_reuse") or "", 260)
    strategy_top_avoid = compact(strategy_effectiveness.get("top_avoid") or "", 260)
    current_recipe = {
        "check": " && ".join(command_recipe.get("checks") or []),
        "grant": command_recipe.get("grant") or "",
        "apply": command_recipe.get("apply") or "",
        "score": command_recipe.get("score") or "",
        "score_options": " || ".join(command_recipe.get("score_options") or []),
    }
    if strategy_used_count > 0 and strategy_failed_count > strategy_satisfied_count:
        status = "inspect_unreliable_command_strategy"
        focus = strategy_top_avoid or top_avoid or "repair command strategy needs inspection"
        next_kind = "execute"
        next_query = "octopus repair . after unreliable repair command strategy"
    elif strategy_used_count > 0 and strategy_partial_count > 0 and strategy_satisfied_count == 0:
        status = "tighten_partial_command_strategy"
        focus = strategy_top_avoid or strategy_top_reuse or "partial command strategy outcomes need tighter evidence"
        next_kind = "execute"
        next_query = "octopus repair . after partial repair command strategy"
    elif used_count <= 0:
        status = "collect_command_outcomes"
        focus = "score reviewed repair outcomes with their command recipes"
        next_kind = "verify"
        next_query = "collect reviewed repair command outcomes"
    elif failed_count > satisfied_count:
        status = "inspect_unreliable_command_recipe"
        focus = top_avoid or top_reuse or "repair command recipe needs inspection"
        next_kind = "execute"
        next_query = "octopus repair . after unreliable repair command recipe"
    elif partial_count > 0 and satisfied_count == 0:
        status = "tighten_partial_command_recipe"
        focus = top_avoid or top_reuse or "partial command recipe outcomes need tighter evidence"
        next_kind = "execute"
        next_query = "octopus repair . after partial repair command recipe"
    else:
        status = "reuse_effective_command_recipe"
        focus = top_reuse or current_recipe["apply"] or "reuse effective repair command recipe"
        next_kind = "verify"
        next_query = "review repair plan with effective command recipe"
    return {
        "schema_version": "octopus-harness-repair-command-strategy-v1",
        "status": status,
        "focus": compact(focus, 260),
        "target_tentacle": target_tentacle,
        "target_tool": target_tool,
        "current_recipe": current_recipe,
        "learned_reuse": top_reuse,
        "learned_avoid": top_avoid,
        "strategy_learned_reuse": strategy_top_reuse,
        "strategy_learned_avoid": strategy_top_avoid,
        "next_need": {
            "kind": next_kind,
            "query": next_query,
        },
        "evidence": {
            "used_count": used_count,
            "satisfied_count": satisfied_count,
            "partial_count": partial_count,
            "failed_count": failed_count,
            "success_rate": effectiveness.get("success_rate") or "0.00",
            "failure_rate": effectiveness.get("failure_rate") or "0.00",
            "strategy_used_count": strategy_used_count,
            "strategy_satisfied_count": strategy_satisfied_count,
            "strategy_partial_count": strategy_partial_count,
            "strategy_failed_count": strategy_failed_count,
            "strategy_success_rate": strategy_effectiveness.get("success_rate") or "0.00",
            "strategy_failure_rate": strategy_effectiveness.get("failure_rate") or "0.00",
        },
    }


def repair_command_strategy_markdown(strategy, workspace, strategy_json):
    evidence = strategy.get("evidence") if isinstance(strategy.get("evidence"), dict) else {}
    current = strategy.get("current_recipe") if isinstance(strategy.get("current_recipe"), dict) else {}
    next_need = strategy.get("next_need") if isinstance(strategy.get("next_need"), dict) else {}
    lines = [
        "# Harness Repair Command Strategy",
        "",
        "This file compresses command effectiveness into the next reviewable repair command focus.",
        "It is local tentacle Feed evidence, not clean-brain context.",
        "",
        f"json: `{rel(strategy_json, workspace)}`",
        f"status: `{strategy.get('status')}`",
        f"focus: {compact(strategy.get('focus'), 260)}",
        f"target: `{strategy.get('target_tentacle')}/{strategy.get('target_tool')}`",
        f"next_need: `{next_need.get('kind') or 'verify'} {next_need.get('query') or ''}`",
        "",
        "## Evidence",
        "",
        f"- used: `{evidence.get('used_count', 0)}`",
        f"- satisfied: `{evidence.get('satisfied_count', 0)}`",
        f"- partial: `{evidence.get('partial_count', 0)}`",
        f"- failed: `{evidence.get('failed_count', 0)}`",
        f"- success_rate: `{evidence.get('success_rate', '0.00')}`",
        f"- failure_rate: `{evidence.get('failure_rate', '0.00')}`",
        f"- learned_reuse: {compact(strategy.get('learned_reuse'), 260) or 'none'}",
        f"- learned_avoid: {compact(strategy.get('learned_avoid'), 260) or 'none'}",
        f"- strategy_used: `{evidence.get('strategy_used_count', 0)}`",
        f"- strategy_satisfied: `{evidence.get('strategy_satisfied_count', 0)}`",
        f"- strategy_partial: `{evidence.get('strategy_partial_count', 0)}`",
        f"- strategy_failed: `{evidence.get('strategy_failed_count', 0)}`",
        f"- strategy_success_rate: `{evidence.get('strategy_success_rate', '0.00')}`",
        f"- strategy_failure_rate: `{evidence.get('strategy_failure_rate', '0.00')}`",
        f"- strategy_learned_reuse: {compact(strategy.get('strategy_learned_reuse'), 260) or 'none'}",
        f"- strategy_learned_avoid: {compact(strategy.get('strategy_learned_avoid'), 260) or 'none'}",
        "",
        "## Current Recipe",
        "",
        f"- check: {compact(current.get('check'), 260) or 'none'}",
        f"- grant: {compact(current.get('grant'), 260) or 'none'}",
        f"- apply: {compact(current.get('apply'), 260) or 'none'}",
        f"- score: {compact(current.get('score'), 260) or 'none'}",
    ]
    return "\n".join(lines) + "\n"


def harness_environment_drift_effectiveness_markdown(effectiveness, workspace, effectiveness_json):
    lines = [
        "# Harness Environment Drift Effectiveness",
        "",
        "This file measures whether responding to prior environment drift helped later reviewed outcomes.",
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
        lines.append("No scored repair outcome has used environment drift evidence yet.")
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
                f"  drift: {compact(hint.get('hint'), 260)}",
                f"  latest: target=`{hint.get('latest_target') or 'unknown'}` candidate=`{hint.get('latest_candidate') or 'none'}`",
                f"  summary: {compact(hint.get('latest_summary'), 220) or 'none'}",
            ]
        )
    return "\n".join(lines) + "\n"


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


def action_trace_effectiveness_markdown(effectiveness, workspace, effectiveness_json):
    lines = [
        "# Harness Action Trace Effectiveness",
        "",
        "This file measures whether tool-side Need/Tool/Action/Feed traces helped later reviewed outcomes.",
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
        lines.append("No scored repair outcome has used a recorded action trace yet.")
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
                f"  trace: {compact(hint.get('hint'), 260)}",
                f"  latest: target=`{hint.get('latest_target') or 'unknown'}` candidate=`{hint.get('latest_candidate') or 'none'}`",
                f"  summary: {compact(hint.get('latest_summary'), 220) or 'none'}",
            ]
        )
    return "\n".join(lines) + "\n"


def build_repair_decision(
    repair_lessons,
    effectiveness,
    decision_effectiveness,
    repair_recall,
    target_tentacle,
    target_tool,
):
    used_count = int_count(effectiveness.get("used_count"))
    satisfied_count = int_count(effectiveness.get("satisfied_count"))
    partial_count = int_count(effectiveness.get("partial_count"))
    failed_count = int_count(effectiveness.get("failed_count"))
    decision_used_count = int_count(decision_effectiveness.get("used_count"))
    decision_satisfied_count = int_count(decision_effectiveness.get("satisfied_count"))
    decision_partial_count = int_count(decision_effectiveness.get("partial_count"))
    decision_failed_count = int_count(decision_effectiveness.get("failed_count"))
    lesson_count = int_count(repair_lessons.get("lesson_count"))
    recall_count = int_count(repair_recall.get("match_count"))
    top_reuse = effectiveness.get("top_reuse") or repair_lessons.get("top_reuse") or ""
    top_avoid = effectiveness.get("top_avoid") or repair_lessons.get("top_avoid") or ""
    decision_top_reuse = decision_effectiveness.get("top_reuse") or ""
    decision_top_avoid = decision_effectiveness.get("top_avoid") or ""
    if decision_used_count > 0 and decision_failed_count > decision_satisfied_count:
        decision = "inspect_unreliable_repair_decisions"
        status = "partial"
        focus = decision_top_avoid or top_avoid or "repair decisions need inspection"
        next_kind = "execute"
        next_query = "octopus repair . after unreliable repair decisions"
    elif decision_used_count > 0 and decision_partial_count > 0 and decision_satisfied_count == 0:
        decision = "tighten_partial_repair_decisions"
        status = "partial"
        focus = decision_top_avoid or decision_top_reuse or "partial repair decision outcomes need tighter evidence"
        next_kind = "execute"
        next_query = "octopus repair . after partial repair decisions"
    elif used_count <= 0:
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
            "decision_effectiveness_used_count": decision_used_count,
            "decision_effectiveness_satisfied_count": decision_satisfied_count,
            "decision_effectiveness_partial_count": decision_partial_count,
            "decision_effectiveness_failed_count": decision_failed_count,
            "decision_effectiveness_success_rate": decision_effectiveness.get("success_rate") or "0.00",
            "decision_effectiveness_failure_rate": decision_effectiveness.get("failure_rate") or "0.00",
            "decision_top_reuse": compact(decision_top_reuse, 260),
            "decision_top_avoid": compact(decision_top_avoid, 260),
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
        f"- decision_effectiveness_used: `{evidence.get('decision_effectiveness_used_count', 0)}`",
        f"- decision_effectiveness_satisfied: `{evidence.get('decision_effectiveness_satisfied_count', 0)}`",
        f"- decision_effectiveness_partial: `{evidence.get('decision_effectiveness_partial_count', 0)}`",
        f"- decision_effectiveness_failed: `{evidence.get('decision_effectiveness_failed_count', 0)}`",
        f"- decision_success_rate: `{evidence.get('decision_effectiveness_success_rate', '0.00')}`",
        f"- decision_failure_rate: `{evidence.get('decision_effectiveness_failure_rate', '0.00')}`",
        f"- decision_top_reuse: {compact(evidence.get('decision_top_reuse'), 260) or 'none'}",
        f"- decision_top_avoid: {compact(evidence.get('decision_top_avoid'), 260) or 'none'}",
    ]
    return "\n".join(lines) + "\n"


def build_harness_adaptation(
    workspace,
    adaptation_json,
    code_context,
    adapter_context,
    harness_environment_profile,
    harness_environment_drift,
    field_trajectory,
    repair_recall,
    repair_lessons,
    repair_lesson_effectiveness,
    harness_adaptation_effectiveness,
    harness_environment_drift_effectiveness,
    repair_decision,
):
    decision_next = repair_decision.get("next_need") if isinstance(repair_decision.get("next_need"), dict) else {}
    adapter_missing = adapter_context.get("missing_core") or ""
    adapter_status = str(adapter_context.get("status") or "")
    profile_status = str(harness_environment_profile.get("status") or "")
    profile_id = str(harness_environment_profile.get("profile_id") or "")
    execution_mode = str(harness_environment_profile.get("execution_mode") or "")
    profile_capabilities = ",".join(harness_environment_profile.get("capabilities") or [])
    profile_constraints = ",".join(harness_environment_profile.get("constraints") or [])
    drift_status = str(harness_environment_drift.get("status") or "")
    drift_detail = str(harness_environment_drift.get("detail") or "")
    drift_next = harness_environment_drift.get("next_need") if isinstance(harness_environment_drift.get("next_need"), dict) else {}
    field_status = str(field_trajectory.get("verifier_status") or "").lower()
    field = field_trajectory.get("field") or ""
    mini_task = field_trajectory.get("mini_task") or ""
    adaptation_used = int_count(harness_adaptation_effectiveness.get("used_count"))
    adaptation_satisfied = int_count(harness_adaptation_effectiveness.get("satisfied_count"))
    adaptation_failed = int_count(harness_adaptation_effectiveness.get("failed_count"))
    adaptation_top_reuse = harness_adaptation_effectiveness.get("top_reuse") or ""
    adaptation_top_avoid = harness_adaptation_effectiveness.get("top_avoid") or ""
    drift_effectiveness_used = int_count(harness_environment_drift_effectiveness.get("used_count"))
    drift_effectiveness_satisfied = int_count(harness_environment_drift_effectiveness.get("satisfied_count"))
    drift_effectiveness_failed = int_count(harness_environment_drift_effectiveness.get("failed_count"))
    drift_effectiveness_top_reuse = harness_environment_drift_effectiveness.get("top_reuse") or ""
    drift_effectiveness_top_avoid = harness_environment_drift_effectiveness.get("top_avoid") or ""
    if adapter_missing or adapter_status in {"failed", "partial"}:
        status = "adapter_substrate"
        focus = f"restore local tool substrate: {adapter_missing or adapter_status}"
        next_need = {
            "kind": "execute",
            "query": f"install missing core adapters: {adapter_missing or adapter_status}",
        }
    elif code_context.get("tentacle") == "unknown":
        status = "target_context"
        focus = "locate editable harness target before repair"
        next_need = {
            "kind": "verify",
            "query": "inspect latest Feed trace and check history for repair target",
        }
    elif drift_status == "degraded" and drift_next.get("query"):
        status = "environment_drift"
        focus = f"repair harness environment drift: {drift_detail}"
        next_need = {
            "kind": drift_next.get("kind") or "execute",
            "query": drift_next.get("query") or "repair harness environment drift",
        }
    elif drift_effectiveness_used > 0 and drift_effectiveness_failed > drift_effectiveness_satisfied:
        status = "environment_drift_effectiveness_review"
        focus = drift_effectiveness_top_avoid or "inspect unreliable harness environment drift response"
        next_need = {
            "kind": "execute",
            "query": "octopus repair . after unreliable harness environment drift response",
        }
    elif field and mini_task and field_status in {"failed", "partial"}:
        status = "field_runtime"
        focus = f"adapt field template for {field}/{mini_task}"
        next_need = {
            "kind": "execute",
            "query": f"evolve recommend {code_context.get('tentacle') or 'field-mini-task'} after {field}/{mini_task}",
        }
    elif adaptation_used > 0 and adaptation_failed > adaptation_satisfied:
        status = "adaptation_effectiveness_review"
        focus = adaptation_top_avoid or "inspect failed harness adaptation focus"
        next_need = {
            "kind": "execute",
            "query": "octopus repair . after unreliable harness adaptation focus",
        }
    elif decision_next.get("query"):
        status = "decision_guided"
        focus = adaptation_top_reuse or repair_decision.get("focus") or "follow repair decision"
        next_need = {
            "kind": decision_next.get("kind") or "verify",
            "query": decision_next.get("query") or "review repair decision",
        }
    else:
        status = "review_required"
        focus = "review repair plan and collect outcome"
        next_need = {"kind": "verify", "query": "review repair plan and score outcome"}
    return {
        "schema_version": "octopus-harness-adaptation-v1",
        "status": status,
        "focus": compact(focus, 320),
        "workspace": str(workspace),
        "json": rel(adaptation_json, workspace),
        "target": {
            "tentacle": code_context.get("tentacle") or "unknown",
            "tool": code_context.get("tool") or "unknown",
            "tool_path": code_context.get("tool_path") or "",
            "manifest": code_context.get("manifest") or "",
        },
        "environment": {
            "adapter_status": adapter_status,
            "adapter_available": adapter_context.get("available") or "",
            "adapter_missing_core": adapter_missing,
            "profile_status": profile_status,
            "profile_id": profile_id,
            "execution_mode": execution_mode,
            "profile_capabilities": profile_capabilities,
            "profile_constraints": profile_constraints,
            "drift_status": drift_status,
            "drift_detail": drift_detail,
            "provider_env": adapter_context.get("provider_env") or "",
            "provider_keys": adapter_context.get("provider_keys") or "",
            "desktop_adapters": adapter_context.get("desktop_adapters") or "",
            "field": field,
            "mini_task": mini_task,
            "verifier_status": field_trajectory.get("verifier_status") or "",
            "verifier_error": field_trajectory.get("verifier_error") or "",
        },
        "memory": {
            "recall_matches": str(repair_recall.get("match_count") or 0),
            "lesson_count": str(repair_lessons.get("lesson_count") or 0),
            "lesson_reuse_count": str(repair_lessons.get("reuse_count") or 0),
            "lesson_avoid_count": str(repair_lessons.get("avoid_count") or 0),
            "effectiveness_used_count": str(repair_lesson_effectiveness.get("used_count") or 0),
            "effectiveness_success_rate": repair_lesson_effectiveness.get("success_rate") or "0.00",
            "adaptation_effectiveness_used_count": str(adaptation_used),
            "adaptation_effectiveness_success_rate": harness_adaptation_effectiveness.get("success_rate") or "0.00",
            "adaptation_top_reuse": adaptation_top_reuse,
            "adaptation_top_avoid": adaptation_top_avoid,
            "environment_drift_effectiveness_used_count": str(drift_effectiveness_used),
            "environment_drift_effectiveness_success_rate": harness_environment_drift_effectiveness.get("success_rate") or "0.00",
            "environment_drift_top_reuse": drift_effectiveness_top_reuse,
            "environment_drift_top_avoid": drift_effectiveness_top_avoid,
        },
        "decision": {
            "status": repair_decision.get("status") or "",
            "kind": repair_decision.get("decision") or "",
            "focus": repair_decision.get("focus") or "",
        },
        "next_need": next_need,
        "review_targets": [
            "HARNESS_ADAPTATION.md",
            "HARNESS_ENVIRONMENT_PROFILE.md",
            "HARNESS_ENVIRONMENT_DRIFT.md",
            "HARNESS_ENVIRONMENT_DRIFT_EFFECTIVENESS.md",
            "ADAPTER_CONTEXT.md",
            "FIELD_TRAJECTORY.md",
            "CODE_CONTEXT.md",
            "ACTION_TRACE.md",
            "REPAIR_PLAN.json",
        ],
        "context_boundary": "Clean brain sees Goal + Mem + Need + Feed. This adaptation plan stays inside the harness tentacle.",
    }


def harness_adaptation_markdown(adaptation, workspace, adaptation_json):
    target = adaptation.get("target") if isinstance(adaptation.get("target"), dict) else {}
    environment = adaptation.get("environment") if isinstance(adaptation.get("environment"), dict) else {}
    memory = adaptation.get("memory") if isinstance(adaptation.get("memory"), dict) else {}
    decision = adaptation.get("decision") if isinstance(adaptation.get("decision"), dict) else {}
    next_need = adaptation.get("next_need") if isinstance(adaptation.get("next_need"), dict) else {}
    lines = [
        "# Harness Adaptation Plan",
        "",
        "This is the repair tentacle's local environment adaptation plan.",
        "It is Feed evidence for heartbeat repair, not clean-brain context.",
        "",
        f"json: `{rel(adaptation_json, workspace)}`",
        f"status: `{adaptation.get('status')}`",
        f"focus: {compact(adaptation.get('focus'), 320)}",
        f"next_need: `{next_need.get('kind') or 'verify'} {next_need.get('query') or ''}`",
        "",
        "## Target",
        "",
        f"- tentacle: `{target.get('tentacle') or 'unknown'}`",
        f"- tool: `{target.get('tool') or 'unknown'}`",
        f"- tool_path: `{target.get('tool_path') or 'missing'}`",
        f"- manifest: `{target.get('manifest') or 'missing'}`",
        "",
        "## Environment",
        "",
        f"- adapter_status: `{environment.get('adapter_status') or 'unknown'}`",
        f"- adapter_available: `{environment.get('adapter_available') or 'none'}`",
        f"- adapter_missing_core: `{environment.get('adapter_missing_core') or 'none'}`",
        f"- profile_status: `{environment.get('profile_status') or 'unknown'}`",
        f"- profile_id: `{environment.get('profile_id') or 'none'}`",
        f"- execution_mode: `{environment.get('execution_mode') or 'none'}`",
        f"- profile_capabilities: `{environment.get('profile_capabilities') or 'none'}`",
        f"- profile_constraints: `{environment.get('profile_constraints') or 'none'}`",
        f"- drift_status: `{environment.get('drift_status') or 'none'}`",
        f"- drift_detail: {compact(environment.get('drift_detail'), 260) or 'none'}",
        f"- provider_env: `{environment.get('provider_env') or 'missing'}`",
        f"- provider_keys: `{environment.get('provider_keys') or 'none'}`",
        f"- desktop_adapters: `{environment.get('desktop_adapters') or 'none'}`",
        f"- field: `{environment.get('field') or 'none'}`",
        f"- mini_task: `{environment.get('mini_task') or 'none'}`",
        f"- verifier_status: `{environment.get('verifier_status') or 'none'}`",
        f"- verifier_error: `{environment.get('verifier_error') or 'none'}`",
        "",
        "## Harness Memory",
        "",
        f"- recall_matches: `{memory.get('recall_matches') or '0'}`",
        f"- lesson_count: `{memory.get('lesson_count') or '0'}`",
        f"- lesson_reuse_count: `{memory.get('lesson_reuse_count') or '0'}`",
        f"- lesson_avoid_count: `{memory.get('lesson_avoid_count') or '0'}`",
        f"- effectiveness_used_count: `{memory.get('effectiveness_used_count') or '0'}`",
        f"- effectiveness_success_rate: `{memory.get('effectiveness_success_rate') or '0.00'}`",
        f"- adaptation_effectiveness_used_count: `{memory.get('adaptation_effectiveness_used_count') or '0'}`",
        f"- adaptation_effectiveness_success_rate: `{memory.get('adaptation_effectiveness_success_rate') or '0.00'}`",
        f"- adaptation_top_reuse: {compact(memory.get('adaptation_top_reuse'), 260) or 'none'}",
        f"- adaptation_top_avoid: {compact(memory.get('adaptation_top_avoid'), 260) or 'none'}",
        f"- environment_drift_effectiveness_used_count: `{memory.get('environment_drift_effectiveness_used_count') or '0'}`",
        f"- environment_drift_effectiveness_success_rate: `{memory.get('environment_drift_effectiveness_success_rate') or '0.00'}`",
        f"- environment_drift_top_reuse: {compact(memory.get('environment_drift_top_reuse'), 260) or 'none'}",
        f"- environment_drift_top_avoid: {compact(memory.get('environment_drift_top_avoid'), 260) or 'none'}",
        "",
        "## Repair Decision",
        "",
        f"- status: `{decision.get('status') or 'unknown'}`",
        f"- decision: `{decision.get('kind') or 'none'}`",
        f"- focus: {compact(decision.get('focus'), 260) or 'none'}",
        "",
        "## Review Targets",
        "",
        *[f"- `{target}`" for target in adaptation.get("review_targets", [])],
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


def split_csv(value):
    return [
        item.strip()
        for item in str(value or "").split(",")
        if item.strip() and item.strip() != "none"
    ]


def installed_profile_ids(state):
    profiles = []
    for item in state.get("installed_profiles") or []:
        if isinstance(item, str):
            profiles.append(item)
        elif isinstance(item, dict):
            profile_id = item.get("id") or item.get("profile") or item.get("tentacle")
            if profile_id:
                profiles.append(str(profile_id))
    for item in state.get("installed_tentacles") or []:
        if isinstance(item, dict):
            tentacle_id = item.get("id") or item.get("profile")
            if tentacle_id:
                profiles.append(str(tentacle_id))
        elif isinstance(item, str):
            profiles.append(item)
    return sorted(set(profiles))


def build_harness_environment_profile(
    workspace,
    profile_json,
    state,
    adapter_context,
    code_context,
    field_trajectory,
    repair_outcomes,
    harness_adaptation_effectiveness,
):
    available = split_csv(adapter_context.get("available"))
    missing_core = split_csv(adapter_context.get("missing_core"))
    provider_keys = split_csv(adapter_context.get("provider_keys"))
    desktop_adapters = split_csv(adapter_context.get("desktop_adapters"))
    available_set = set(available)
    installed = installed_profile_ids(state)
    capabilities = []
    if {"bash", "git"}.issubset(available_set):
        capabilities.append("local_repo_harness")
    if "python3" in available_set:
        capabilities.append("python_runtime")
    if "node" in available_set:
        capabilities.append("node_runtime")
    if "cargo" in available_set:
        capabilities.append("rust_checks")
    if "gh" in available_set:
        capabilities.append("github_cli")
    if desktop_adapters:
        capabilities.append("desktop_observation")
    if provider_keys or adapter_context.get("provider_env") == "present":
        capabilities.append("provider_draft")
    constraints = []
    if missing_core:
        constraints.append("missing_core=" + ",".join(missing_core))
    if not provider_keys and adapter_context.get("provider_env") != "present":
        constraints.append("provider_draft_unconfigured")
    if not desktop_adapters:
        constraints.append("desktop_observation_unavailable")
    if "gh" not in available_set:
        constraints.append("github_cli_unavailable")
    target_tentacle = code_context.get("tentacle") or "unknown"
    target_tool = code_context.get("tool") or "unknown"
    field = field_trajectory.get("field") or ""
    mini_task = field_trajectory.get("mini_task") or ""
    if missing_core:
        execution_mode = "limited-substrate"
    elif target_tentacle == "field-mini-task" or field:
        execution_mode = "field-template-runtime"
    elif "cargo" in available_set and "git" in available_set:
        execution_mode = "repo-check-runtime"
    elif "bash" in available_set:
        execution_mode = "shell-harness-runtime"
    else:
        execution_mode = "manifest-review-runtime"
    provider_mode = (
        "configured"
        if provider_keys or adapter_context.get("provider_env") == "present"
        else "local-only"
    )
    if {"open", "osascript", "screencapture"}.intersection(desktop_adapters):
        desktop_mode = "macos-desktop"
    elif "xdg-open" in desktop_adapters:
        desktop_mode = "linux-desktop"
    else:
        desktop_mode = "headless"
    status = "partial" if missing_core else "satisfied"
    profile_id = "-".join(
        item
        for item in [
            platform.system().lower() or "unknown",
            platform.machine().lower() or "unknown",
            execution_mode,
        ]
        if item
    )
    next_need = (
        {
            "kind": "execute",
            "query": f"install missing core adapters: {','.join(missing_core)}",
        }
        if missing_core
        else {
            "kind": "verify",
            "query": f"reuse harness environment profile for {target_tentacle}/{target_tool}",
        }
    )
    return {
        "schema_version": "octopus-harness-environment-profile-v1",
        "status": status,
        "profile_id": compact(profile_id, 180),
        "workspace": str(workspace),
        "json": rel(profile_json, workspace),
        "system": {
            "os": platform.system(),
            "machine": platform.machine(),
            "python": platform.python_version(),
        },
        "execution_mode": execution_mode,
        "provider_mode": provider_mode,
        "desktop_mode": desktop_mode,
        "available_commands": available,
        "missing_core": missing_core,
        "provider_keys": provider_keys,
        "desktop_adapters": desktop_adapters,
        "installed_profiles": installed,
        "capabilities": capabilities,
        "constraints": constraints,
        "target": {
            "tentacle": target_tentacle,
            "tool": target_tool,
            "field": field,
            "mini_task": mini_task,
        },
        "memory": {
            "repair_outcome_count": str(len(repair_outcomes)),
            "adaptation_effectiveness_used_count": str(harness_adaptation_effectiveness.get("used_count") or 0),
            "adaptation_effectiveness_success_rate": harness_adaptation_effectiveness.get("success_rate") or "0.00",
            "adaptation_top_reuse": harness_adaptation_effectiveness.get("top_reuse") or "",
            "adaptation_top_avoid": harness_adaptation_effectiveness.get("top_avoid") or "",
        },
        "next_need": next_need,
        "context_boundary": "Clean brain sees Goal + Mem + Need + Feed. This profile stays inside the harness tentacle.",
    }


def harness_environment_profile_markdown(profile, workspace, profile_json):
    target = profile.get("target") if isinstance(profile.get("target"), dict) else {}
    memory = profile.get("memory") if isinstance(profile.get("memory"), dict) else {}
    next_need = profile.get("next_need") if isinstance(profile.get("next_need"), dict) else {}
    system = profile.get("system") if isinstance(profile.get("system"), dict) else {}
    lines = [
        "# Harness Environment Profile",
        "",
        "This file compresses local tool, runtime, provider, desktop, and outcome signals for the repair tentacle.",
        "It is Feed evidence for harness adaptation, not clean-brain context.",
        "",
        f"json: `{rel(profile_json, workspace)}`",
        f"status: `{profile.get('status')}`",
        f"profile_id: `{profile.get('profile_id')}`",
        f"execution_mode: `{profile.get('execution_mode')}`",
        f"provider_mode: `{profile.get('provider_mode')}`",
        f"desktop_mode: `{profile.get('desktop_mode')}`",
        f"next_need: `{next_need.get('kind') or 'verify'} {next_need.get('query') or ''}`",
        "",
        "## System",
        "",
        f"- os: `{system.get('os') or 'unknown'}`",
        f"- machine: `{system.get('machine') or 'unknown'}`",
        f"- python: `{system.get('python') or 'unknown'}`",
        "",
        "## Target",
        "",
        f"- tentacle: `{target.get('tentacle') or 'unknown'}`",
        f"- tool: `{target.get('tool') or 'unknown'}`",
        f"- field: `{target.get('field') or 'none'}`",
        f"- mini_task: `{target.get('mini_task') or 'none'}`",
        "",
        "## Capabilities",
        "",
        f"- available_commands: `{','.join(profile.get('available_commands') or []) or 'none'}`",
        f"- missing_core: `{','.join(profile.get('missing_core') or []) or 'none'}`",
        f"- provider_keys: `{','.join(profile.get('provider_keys') or []) or 'none'}`",
        f"- desktop_adapters: `{','.join(profile.get('desktop_adapters') or []) or 'none'}`",
        f"- installed_profiles: `{','.join(profile.get('installed_profiles') or []) or 'none'}`",
        f"- capabilities: `{','.join(profile.get('capabilities') or []) or 'none'}`",
        f"- constraints: `{','.join(profile.get('constraints') or []) or 'none'}`",
        "",
        "## Harness Memory",
        "",
        f"- repair_outcome_count: `{memory.get('repair_outcome_count') or '0'}`",
        f"- adaptation_effectiveness_used_count: `{memory.get('adaptation_effectiveness_used_count') or '0'}`",
        f"- adaptation_effectiveness_success_rate: `{memory.get('adaptation_effectiveness_success_rate') or '0.00'}`",
        f"- adaptation_top_reuse: {compact(memory.get('adaptation_top_reuse'), 260) or 'none'}",
        f"- adaptation_top_avoid: {compact(memory.get('adaptation_top_avoid'), 260) or 'none'}",
    ]
    return "\n".join(lines) + "\n"


def load_profile_history(path, limit=12):
    if not path.exists():
        return []
    rows = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        try:
            item = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(item, dict) and item.get("schema_version") == "octopus-harness-environment-profile-v1":
            rows.append(item)
    return rows[-limit:]


def append_profile_history(path, profile, limit=24):
    path.parent.mkdir(parents=True, exist_ok=True)
    rows = load_profile_history(path, limit - 1)
    rows.append(profile)
    path.write_text(
        "\n".join(json.dumps(row, ensure_ascii=True, sort_keys=True) for row in rows[-limit:]) + "\n",
        encoding="utf-8",
    )


def list_delta(before, after):
    before_set = set(before or [])
    after_set = set(after or [])
    gained = sorted(after_set - before_set)
    lost = sorted(before_set - after_set)
    return gained, lost


def build_harness_environment_drift(workspace, drift_json, profile, history):
    previous = history[-1] if history else {}
    if not previous:
        return {
            "schema_version": "octopus-harness-environment-drift-v1",
            "status": "baseline",
            "summary": "first observed harness environment profile",
            "detail": "no previous profile",
            "workspace": str(workspace),
            "json": rel(drift_json, workspace),
            "history_count": 0,
            "current_profile_id": profile.get("profile_id") or "",
            "previous_profile_id": "",
            "changes": [],
            "gained_count": 0,
            "lost_count": 0,
            "lost_capabilities": [],
            "gained_capabilities": [],
            "gained_missing_core": [],
            "removed_constraints": [],
            "next_need": {
                "kind": "verify",
                "query": "review harness environment baseline",
            },
            "context_boundary": "Clean brain sees Goal + Mem + Need + Feed. Environment drift stays inside the harness tentacle.",
        }
    changes = []
    gains = []
    losses = []
    for key in ["execution_mode", "provider_mode", "desktop_mode", "status"]:
        before = str(previous.get(key) or "")
        after = str(profile.get(key) or "")
        if previous and before != after:
            changes.append({
                "field": key,
                "from": before,
                "to": after,
                "direction": "changed",
            })
    for key in [
        "available_commands",
        "missing_core",
        "provider_keys",
        "desktop_adapters",
        "installed_profiles",
        "capabilities",
        "constraints",
    ]:
        gained, lost = list_delta(previous.get(key) if previous else [], profile.get(key) or [])
        for value in gained:
            change = {"field": key, "value": value, "direction": "gained"}
            changes.append(change)
            gains.append(change)
        for value in lost:
            change = {"field": key, "value": value, "direction": "lost"}
            changes.append(change)
            losses.append(change)
    lost_capabilities = [item["value"] for item in losses if item["field"] == "capabilities"]
    gained_missing_core = [item["value"] for item in gains if item["field"] == "missing_core"]
    lost_provider = [item["value"] for item in losses if item["field"] == "provider_keys"]
    lost_desktop = [item["value"] for item in losses if item["field"] == "desktop_adapters"]
    removed_constraints = [item["value"] for item in losses if item["field"] == "constraints"]
    gained_capabilities = [item["value"] for item in gains if item["field"] == "capabilities"]
    if gained_missing_core or lost_capabilities:
        status = "degraded"
        summary = "environment lost core capability"
    elif lost_provider or lost_desktop:
        status = "changed"
        summary = "provider or desktop adapter availability changed"
    elif removed_constraints or gained_capabilities:
        status = "improved"
        summary = "environment gained capability or removed constraint"
    elif changes:
        status = "changed"
        summary = "environment profile changed"
    else:
        status = "stable"
        summary = "environment profile matches previous run"
    if gained_missing_core:
        detail = f"missing_core gained {','.join(gained_missing_core)}"
    elif lost_capabilities:
        detail = f"capability lost {','.join(lost_capabilities)}"
    elif gained_capabilities:
        detail = f"capability gained {','.join(gained_capabilities)}"
    elif removed_constraints:
        detail = f"constraint removed {','.join(removed_constraints)}"
    elif changes:
        detail = ", ".join(
            compact(
                f"{item.get('field')} {item.get('direction')} {item.get('value') or item.get('from', '')}->{item.get('to', '')}",
                120,
            )
            for item in changes[:3]
        )
    else:
        detail = "no change"
    next_need = (
        {
            "kind": "execute",
            "query": f"repair harness environment drift: {detail}",
        }
        if status == "degraded"
        else {
            "kind": "verify",
            "query": f"review harness environment drift: {detail}",
        }
    )
    return {
        "schema_version": "octopus-harness-environment-drift-v1",
        "status": status,
        "summary": compact(summary, 260),
        "detail": compact(detail, 320),
        "workspace": str(workspace),
        "json": rel(drift_json, workspace),
        "history_count": len(history),
        "current_profile_id": profile.get("profile_id") or "",
        "previous_profile_id": previous.get("profile_id") or "",
        "changes": changes[:24],
        "gained_count": len(gains),
        "lost_count": len(losses),
        "lost_capabilities": lost_capabilities,
        "gained_capabilities": gained_capabilities,
        "gained_missing_core": gained_missing_core,
        "removed_constraints": removed_constraints,
        "next_need": next_need,
        "context_boundary": "Clean brain sees Goal + Mem + Need + Feed. Environment drift stays inside the harness tentacle.",
    }


def harness_environment_drift_markdown(drift, workspace, drift_json, profile_journal):
    next_need = drift.get("next_need") if isinstance(drift.get("next_need"), dict) else {}
    lines = [
        "# Harness Environment Drift",
        "",
        "This file compares the current harness environment profile with prior local profiles.",
        "It is Feed evidence for harness adaptation, not clean-brain context.",
        "",
        f"json: `{rel(drift_json, workspace)}`",
        f"profile_journal: `{rel(profile_journal, workspace)}`",
        f"status: `{drift.get('status')}`",
        f"summary: {compact(drift.get('summary'), 260)}",
        f"detail: {compact(drift.get('detail'), 320)}",
        f"history_count: `{drift.get('history_count')}`",
        f"current_profile_id: `{drift.get('current_profile_id') or 'none'}`",
        f"previous_profile_id: `{drift.get('previous_profile_id') or 'none'}`",
        f"next_need: `{next_need.get('kind') or 'verify'} {next_need.get('query') or ''}`",
        "",
        "## Change Counts",
        "",
        f"- gained: `{drift.get('gained_count', 0)}`",
        f"- lost: `{drift.get('lost_count', 0)}`",
        f"- gained_missing_core: `{','.join(drift.get('gained_missing_core') or []) or 'none'}`",
        f"- lost_capabilities: `{','.join(drift.get('lost_capabilities') or []) or 'none'}`",
        f"- gained_capabilities: `{','.join(drift.get('gained_capabilities') or []) or 'none'}`",
        f"- removed_constraints: `{','.join(drift.get('removed_constraints') or []) or 'none'}`",
        "",
        "## Changes",
        "",
    ]
    if not drift.get("changes"):
        lines.append("- none")
    for item in drift.get("changes", [])[:12]:
        if item.get("direction") == "changed":
            lines.append(
                f"- `{item.get('field')}` changed `{item.get('from') or 'none'}` -> `{item.get('to') or 'none'}`"
            )
        else:
            lines.append(
                f"- `{item.get('field')}` {item.get('direction')} `{item.get('value')}`"
            )
    return "\n".join(lines) + "\n"


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
    repair_commands = build_repair_commands(
        workspace,
        target_tentacle,
        candidate,
        code_context,
        commands,
    )
    target = code_context.get("tentacle") or target_tentacle or "unknown"
    tool = code_context.get("tool") or "unknown"
    tool_path = code_context.get("tool_path") or ""
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
        "commands": repair_commands,
    }


def build_repair_commands(workspace, target_tentacle, candidate, code_context, commands):
    target = code_context.get("tentacle") or target_tentacle or "unknown"
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
        "checks": check_commands,
        "grant": grant_command,
        "apply": apply_command,
        "score": "octopus repair score <trace-index> satisfied \"repair improved Feed\"",
        "score_options": [
            "octopus repair score <trace-index> satisfied \"repair improved Feed\"",
            "octopus repair score <trace-index> partial \"repair partly improved Feed\"",
            "octopus repair score <trace-index> failed \"repair did not improve Feed\"",
        ],
        "suggested": commands,
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
                    "Return a compact review draft with diagnosis, proposed edit, checks, and next Need. "
                    "When the evidence supports a small harness edit, include a compact unified diff."
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


def extract_unified_diff(content):
    text = content or ""
    fence = re.search(r"```(?:diff|patch)?\s*\n(.*?diff --git.*?)(?:\n```|$)", text, re.S)
    if fence:
        return fence.group(1).strip()
    marker = text.find("diff --git")
    if marker < 0:
        return ""
    return text[marker:].strip()


def build_repair_patch_draft(draft, code_context):
    content = draft.get("content") or ""
    diff = extract_unified_diff(content)
    has_patch = bool(diff)
    draft_status = str(draft.get("status") or "unknown")
    target = {
        "tentacle": code_context.get("tentacle") or "unknown",
        "tool": code_context.get("tool") or "",
        "tool_path": code_context.get("tool_path") or "",
    }
    if draft_status != "generated":
        status = "draft_unavailable"
        next_need = {
            "kind": "verify",
            "query": "configure provider repair draft before patch review",
        }
    elif has_patch:
        status = "patch_attached"
        next_need = {
            "kind": "verify",
            "query": "review provider repair patch draft before grant/apply",
        }
    else:
        status = "narrative_only"
        next_need = {
            "kind": "verify",
            "query": "review provider repair narrative before editing harness",
        }
    return {
        "schema_version": "octopus-harness-repair-patch-draft-v1",
        "status": status,
        "draft_status": draft_status,
        "prefix": draft.get("prefix") or "",
        "model": draft.get("model") or "",
        "target": target,
        "has_patch": has_patch,
        "patch": diff,
        "patch_preview": compact(diff, 1800),
        "proposal_preview": compact(content, 1800),
        "next_need": next_need,
    }


def repair_patch_draft_markdown(patch_draft, workspace, patch_draft_json):
    target = patch_draft.get("target") if isinstance(patch_draft.get("target"), dict) else {}
    next_need = patch_draft.get("next_need") if isinstance(patch_draft.get("next_need"), dict) else {}
    lines = [
        "# Harness Repair Patch Draft",
        "",
        "This file structures provider repair draft evidence for review.",
        "It does not apply code. Grant/apply/score stays outside this artifact.",
        "",
        f"json: `{rel(patch_draft_json, workspace)}`",
        f"status: `{patch_draft.get('status')}`",
        f"draft_status: `{patch_draft.get('draft_status')}`",
        f"model: `{patch_draft.get('model') or 'none'}`",
        f"target: `{target.get('tentacle') or 'unknown'}/{target.get('tool') or ''}`",
        f"tool_path: `{rel(Path(target.get('tool_path')), workspace) if target.get('tool_path') else 'missing'}`",
        f"has_patch: `{str(bool(patch_draft.get('has_patch'))).lower()}`",
        f"next_need: `{next_need.get('kind') or 'verify'} {next_need.get('query') or ''}`",
        "",
        "## Patch Preview",
        "",
        "```diff",
        patch_draft.get("patch_preview") or "",
        "```",
        "",
        "## Proposal Preview",
        "",
        patch_draft.get("proposal_preview") or "none",
    ]
    return "\n".join(lines) + "\n"


def build_repair_patch_review(workspace, session_dir, patch_draft):
    patch = patch_draft.get("patch") or patch_draft.get("patch_preview") or ""
    patch = patch.strip()
    has_patch = bool(patch_draft.get("has_patch")) and bool(patch)
    target = patch_draft.get("target") if isinstance(patch_draft.get("target"), dict) else {}
    target_label = "/".join(
        item
        for item in [
            str(target.get("tentacle") or ""),
            str(target.get("tool") or ""),
        ]
        if item
    )
    patch_file = session_dir / "REPAIR_PATCH_REVIEW.patch"
    command = ""
    stdout = ""
    stderr = ""
    returncode = ""
    if not has_patch:
        return {
            "schema_version": "octopus-harness-repair-patch-review-v1",
            "status": "no_patch",
            "check_status": "skipped",
            "has_patch": False,
            "target": target,
            "target_label": target_label,
            "patch_file": "",
            "command": "",
            "returncode": "",
            "stdout": "",
            "stderr": "",
            "summary": "no provider patch to check",
            "next_need": {
                "kind": "verify",
                "query": "review provider repair narrative before editing harness",
            },
        }
    patch_file.write_text(patch.rstrip() + "\n", encoding="utf-8")
    command_parts = ["git", "apply", "--check", "--whitespace=nowarn", str(patch_file)]
    command = " ".join(shlex.quote(part) for part in command_parts)
    if not shutil.which("git"):
        return {
            "schema_version": "octopus-harness-repair-patch-review-v1",
            "status": "unchecked",
            "check_status": "missing_git",
            "has_patch": True,
            "target": target,
            "target_label": target_label,
            "patch_file": rel(patch_file, workspace),
            "command": command,
            "returncode": "",
            "stdout": "",
            "stderr": "",
            "summary": "git is unavailable, provider patch was not checked locally",
            "next_need": {
                "kind": "verify",
                "query": "install git or manually review provider patch before grant/apply",
            },
        }
    try:
        result = subprocess.run(
            command_parts,
            cwd=workspace,
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )
        stdout = compact(result.stdout or "", 1200)
        stderr = compact(result.stderr or "", 1200)
        returncode = str(result.returncode)
    except subprocess.TimeoutExpired as exc:
        stdout = compact(exc.stdout or "", 1200)
        stderr = compact(exc.stderr or "git apply --check timed out", 1200)
        returncode = "timeout"
        return {
            "schema_version": "octopus-harness-repair-patch-review-v1",
            "status": "unchecked",
            "check_status": "timeout",
            "has_patch": True,
            "target": target,
            "target_label": target_label,
            "patch_file": rel(patch_file, workspace),
            "command": command,
            "returncode": returncode,
            "stdout": stdout,
            "stderr": stderr,
            "summary": "provider patch check timed out",
            "next_need": {
                "kind": "verify",
                "query": "review provider patch manually before grant/apply",
            },
        }
    passed = returncode == "0"
    summary = "provider patch applies cleanly" if passed else compact(stderr or stdout or "provider patch check failed", 260)
    return {
        "schema_version": "octopus-harness-repair-patch-review-v1",
        "status": "check_passed" if passed else "check_failed",
        "check_status": "passed" if passed else "failed",
        "has_patch": True,
        "target": target,
        "target_label": target_label,
        "patch_file": rel(patch_file, workspace),
        "command": command,
        "returncode": returncode,
        "stdout": stdout,
        "stderr": stderr,
        "summary": summary,
        "next_need": {
            "kind": "verify" if passed else "execute",
            "query": (
                "review local patch check before grant/apply"
                if passed
                else "repair provider patch draft after failed local check"
            ),
        },
    }


def repair_patch_review_markdown(patch_review, workspace, patch_review_json):
    target = patch_review.get("target") if isinstance(patch_review.get("target"), dict) else {}
    next_need = patch_review.get("next_need") if isinstance(patch_review.get("next_need"), dict) else {}
    lines = [
        "# Harness Repair Patch Review",
        "",
        "This file records a local non-applying review of the provider patch draft.",
        "It runs only `git apply --check`; grant/apply/score stays outside this artifact.",
        "",
        f"json: `{rel(patch_review_json, workspace)}`",
        f"status: `{patch_review.get('status')}`",
        f"check_status: `{patch_review.get('check_status')}`",
        f"target: `{target.get('tentacle') or 'unknown'}/{target.get('tool') or ''}`",
        f"has_patch: `{str(bool(patch_review.get('has_patch'))).lower()}`",
        f"patch_file: `{patch_review.get('patch_file') or 'none'}`",
        f"command: `{patch_review.get('command') or 'none'}`",
        f"returncode: `{patch_review.get('returncode') or 'none'}`",
        f"summary: {compact(patch_review.get('summary') or '', 360) or 'none'}",
        f"next_need: `{next_need.get('kind') or 'verify'} {next_need.get('query') or ''}`",
        "",
        "## Stdout",
        "",
        "```text",
        patch_review.get("stdout") or "",
        "```",
        "",
        "## Stderr",
        "",
        "```text",
        patch_review.get("stderr") or "",
        "```",
    ]
    return "\n".join(lines) + "\n"


def build_repair_patch_apply_plan(workspace, patch_review):
    target = patch_review.get("target") if isinstance(patch_review.get("target"), dict) else {}
    target_tentacle = str(target.get("tentacle") or "unknown")
    target_label = str(patch_review.get("target_label") or "")
    if not target_label:
        target_label = "/".join(
            item
            for item in [
                target_tentacle if target_tentacle != "unknown" else "",
                str(target.get("tool") or ""),
            ]
            if item
        )
    patch_file = str(patch_review.get("patch_file") or "")
    review_status = str(patch_review.get("status") or "")
    check_status = str(patch_review.get("check_status") or "")
    has_patch = bool(patch_review.get("has_patch"))
    if target_tentacle and target_tentacle != "unknown":
        grant_scope = f"evolve:{target_tentacle}"
        required_grant = f"octopus:{grant_scope}"
        grant_command = f"octopus oauth octopus {shell_arg(grant_scope)} harness:write"
    else:
        grant_scope = "harness:write"
        required_grant = "octopus:harness:write"
        grant_command = "octopus oauth octopus harness:write"
    if review_status == "check_passed" and has_patch and patch_file:
        status = "ready_for_authorized_apply"
        next_need = {
            "kind": "execute",
            "query": "grant harness write, then apply reviewed repair patch",
        }
    elif not has_patch:
        status = "blocked_no_patch"
        next_need = {
            "kind": "verify",
            "query": "generate provider patch before repair apply",
        }
    else:
        status = "blocked_patch_review"
        next_need = {
            "kind": "execute",
            "query": "repair provider patch before authorized apply",
        }
    return {
        "schema_version": "octopus-harness-repair-patch-apply-v1",
        "status": status,
        "applied": False,
        "authorized": False,
        "target": target,
        "target_label": target_label,
        "patch_file": patch_file,
        "patch_review_status": review_status,
        "patch_review_check_status": check_status,
        "required_grant": required_grant,
        "grant_command": grant_command,
        "apply_command": f"octopus repair apply {shell_arg(str(workspace))}",
        "check_command": "",
        "command": "",
        "returncode": "",
        "stdout": "",
        "stderr": "",
        "summary": "repair patch is waiting for review and authorization",
        "next_need": next_need,
    }


def repair_patch_apply_markdown(patch_apply, workspace, patch_apply_json):
    target = patch_apply.get("target") if isinstance(patch_apply.get("target"), dict) else {}
    next_need = patch_apply.get("next_need") if isinstance(patch_apply.get("next_need"), dict) else {}
    lines = [
        "# Harness Repair Patch Apply",
        "",
        "This file records the grant-bound local apply status for a reviewed provider repair patch.",
        "",
        f"json: `{rel(patch_apply_json, workspace)}`",
        f"status: `{patch_apply.get('status')}`",
        f"applied: `{str(bool(patch_apply.get('applied'))).lower()}`",
        f"authorized: `{str(bool(patch_apply.get('authorized'))).lower()}`",
        f"target: `{target.get('tentacle') or 'unknown'}/{target.get('tool') or ''}`",
        f"patch_file: `{patch_apply.get('patch_file') or 'none'}`",
        f"patch_review: `{patch_apply.get('patch_review_status') or 'none'} {patch_apply.get('patch_review_check_status') or ''}`",
        f"required_grant: `{patch_apply.get('required_grant') or 'none'}`",
        f"grant_command: `{patch_apply.get('grant_command') or 'none'}`",
        f"apply_command: `{patch_apply.get('apply_command') or 'none'}`",
        f"check_command: `{patch_apply.get('check_command') or 'none'}`",
        f"command: `{patch_apply.get('command') or 'none'}`",
        f"returncode: `{patch_apply.get('returncode') or 'none'}`",
        f"summary: {compact(patch_apply.get('summary') or '', 360) or 'none'}",
        f"next_need: `{next_need.get('kind') or 'verify'} {next_need.get('query') or ''}`",
        "",
        "## Stdout",
        "",
        "```text",
        patch_apply.get("stdout") or "",
        "```",
        "",
        "## Stderr",
        "",
        "```text",
        patch_apply.get("stderr") or "",
        "```",
    ]
    return "\n".join(lines) + "\n"


def build_repair_patch_verify_plan(workspace, patch_apply):
    target = patch_apply.get("target") if isinstance(patch_apply.get("target"), dict) else {}
    target_tentacle = str(target.get("tentacle") or "unknown")
    target_label = str(patch_apply.get("target_label") or "")
    if not target_label:
        target_label = "/".join(
            item
            for item in [
                target_tentacle if target_tentacle != "unknown" else "",
                str(target.get("tool") or ""),
            ]
            if item
        )
    applied = bool(patch_apply.get("applied"))
    if applied:
        status = "ready_for_check"
        next_need = {
            "kind": "verify",
            "query": "run post-apply repair patch verification",
        }
    else:
        status = "waiting_for_patch_apply"
        next_need = {
            "kind": "execute",
            "query": "apply reviewed repair patch before verification",
        }
    check_command = ""
    if target_tentacle and target_tentacle != "unknown":
        check_command = f"octopus check {shell_arg(target_tentacle)}"
    return {
        "schema_version": "octopus-harness-repair-patch-verify-v1",
        "status": status,
        "passed": False,
        "target": target,
        "target_label": target_label,
        "patch_apply": str(patch_apply.get("patch_file") or ""),
        "command": check_command,
        "summary": "repair patch verification waits for an applied patch",
        "check": None,
        "next_need": next_need,
    }


def repair_patch_verify_markdown(patch_verify, workspace, patch_verify_json):
    target = patch_verify.get("target") if isinstance(patch_verify.get("target"), dict) else {}
    next_need = patch_verify.get("next_need") if isinstance(patch_verify.get("next_need"), dict) else {}
    lines = [
        "# Harness Repair Patch Verify",
        "",
        "This file records post-apply verification for a reviewed repair patch.",
        "",
        f"json: `{rel(patch_verify_json, workspace)}`",
        f"status: `{patch_verify.get('status')}`",
        f"passed: `{str(bool(patch_verify.get('passed'))).lower()}`",
        f"target: `{target.get('tentacle') or 'unknown'}/{target.get('tool') or ''}`",
        f"command: `{patch_verify.get('command') or 'none'}`",
        f"summary: {compact(patch_verify.get('summary') or '', 360) or 'none'}",
        f"next_need: `{next_need.get('kind') or 'verify'} {next_need.get('query') or ''}`",
    ]
    return "\n".join(lines) + "\n"


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
    action_trace_effectiveness,
    repair_draft_effectiveness,
    repair_draft_strategy,
    repair_patch_draft,
    repair_patch_draft_effectiveness,
    repair_patch_review,
    repair_patch_review_effectiveness,
    repair_patch_learning,
    repair_patch_learning_effectiveness,
    repair_patch_strategy,
    repair_patch_strategy_effectiveness,
    repair_command_effectiveness,
    repair_command_strategy,
    repair_command_strategy_effectiveness,
    harness_environment_drift,
    harness_environment_drift_effectiveness,
    repair_decision,
    repair_decision_effectiveness,
    harness_adaptation,
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
    action_trace_effectiveness_summary = {
        "used_count": str(action_trace_effectiveness.get("used_count") or 0),
        "satisfied_count": str(action_trace_effectiveness.get("satisfied_count") or 0),
        "failed_count": str(action_trace_effectiveness.get("failed_count") or 0),
        "success_rate": str(action_trace_effectiveness.get("success_rate") or "0.00"),
        "top_reuse": compact(action_trace_effectiveness.get("top_reuse") or "", 240),
        "top_avoid": compact(action_trace_effectiveness.get("top_avoid") or "", 240),
    }
    draft_effectiveness_summary = {
        "used_count": str(repair_draft_effectiveness.get("used_count") or 0),
        "satisfied_count": str(repair_draft_effectiveness.get("satisfied_count") or 0),
        "failed_count": str(repair_draft_effectiveness.get("failed_count") or 0),
        "success_rate": str(repair_draft_effectiveness.get("success_rate") or "0.00"),
        "top_reuse": compact(repair_draft_effectiveness.get("top_reuse") or "", 240),
        "top_avoid": compact(repair_draft_effectiveness.get("top_avoid") or "", 240),
    }
    draft_strategy_next = repair_draft_strategy.get("next_need") if isinstance(repair_draft_strategy.get("next_need"), dict) else {}
    draft_strategy_summary = {
        "status": str(repair_draft_strategy.get("status") or ""),
        "focus": compact(repair_draft_strategy.get("focus") or "", 240),
        "learned_reuse": compact(repair_draft_strategy.get("learned_reuse") or "", 240),
        "learned_avoid": compact(repair_draft_strategy.get("learned_avoid") or "", 240),
        "next_need_kind": str(draft_strategy_next.get("kind") or ""),
        "next_need_query": compact(draft_strategy_next.get("query") or "", 260),
    }
    patch_target = repair_patch_draft.get("target") if isinstance(repair_patch_draft.get("target"), dict) else {}
    patch_next = repair_patch_draft.get("next_need") if isinstance(repair_patch_draft.get("next_need"), dict) else {}
    patch_draft_summary = {
        "status": str(repair_patch_draft.get("status") or ""),
        "draft_status": str(repair_patch_draft.get("draft_status") or ""),
        "has_patch": "true" if repair_patch_draft.get("has_patch") else "false",
        "target": "/".join(
            item
            for item in [
                str(patch_target.get("tentacle") or ""),
                str(patch_target.get("tool") or ""),
            ]
            if item
        ),
        "preview": compact(
            repair_patch_draft.get("patch_preview")
            or repair_patch_draft.get("proposal_preview")
            or "",
            240,
        ),
        "next_need_kind": str(patch_next.get("kind") or ""),
        "next_need_query": compact(patch_next.get("query") or "", 240),
    }
    patch_draft_effectiveness_summary = {
        "used_count": str(repair_patch_draft_effectiveness.get("used_count") or 0),
        "satisfied_count": str(repair_patch_draft_effectiveness.get("satisfied_count") or 0),
        "failed_count": str(repair_patch_draft_effectiveness.get("failed_count") or 0),
        "success_rate": str(repair_patch_draft_effectiveness.get("success_rate") or "0.00"),
        "top_reuse": compact(repair_patch_draft_effectiveness.get("top_reuse") or "", 240),
        "top_avoid": compact(repair_patch_draft_effectiveness.get("top_avoid") or "", 240),
    }
    patch_review_target = repair_patch_review.get("target") if isinstance(repair_patch_review.get("target"), dict) else {}
    patch_review_next = repair_patch_review.get("next_need") if isinstance(repair_patch_review.get("next_need"), dict) else {}
    patch_review_summary = {
        "status": str(repair_patch_review.get("status") or ""),
        "check_status": str(repair_patch_review.get("check_status") or ""),
        "has_patch": "true" if repair_patch_review.get("has_patch") else "false",
        "target": "/".join(
            item
            for item in [
                str(patch_review_target.get("tentacle") or ""),
                str(patch_review_target.get("tool") or ""),
            ]
            if item
        ),
        "patch_file": str(repair_patch_review.get("patch_file") or ""),
        "summary": compact(repair_patch_review.get("summary") or "", 240),
        "next_need_kind": str(patch_review_next.get("kind") or ""),
        "next_need_query": compact(patch_review_next.get("query") or "", 240),
    }
    patch_review_effectiveness_summary = {
        "used_count": str(repair_patch_review_effectiveness.get("used_count") or 0),
        "satisfied_count": str(repair_patch_review_effectiveness.get("satisfied_count") or 0),
        "failed_count": str(repair_patch_review_effectiveness.get("failed_count") or 0),
        "success_rate": str(repair_patch_review_effectiveness.get("success_rate") or "0.00"),
        "top_reuse": compact(repair_patch_review_effectiveness.get("top_reuse") or "", 240),
        "top_avoid": compact(repair_patch_review_effectiveness.get("top_avoid") or "", 240),
    }
    patch_learning_next = repair_patch_learning.get("next_need") if isinstance(repair_patch_learning.get("next_need"), dict) else {}
    patch_learning_summary = {
        "status": str(repair_patch_learning.get("status") or ""),
        "used_count": str(repair_patch_learning.get("used_count") or 0),
        "verified_count": str(repair_patch_learning.get("verified_count") or 0),
        "verified_success_rate": str(repair_patch_learning.get("verified_success_rate") or "0.00"),
        "top_reuse": compact(repair_patch_learning.get("top_reuse") or "", 240),
        "top_avoid": compact(repair_patch_learning.get("top_avoid") or "", 240),
        "next_need_kind": str(patch_learning_next.get("kind") or ""),
        "next_need_query": compact(patch_learning_next.get("query") or "", 260),
    }
    patch_learning_effectiveness_summary = {
        "used_count": str(repair_patch_learning_effectiveness.get("used_count") or 0),
        "satisfied_count": str(repair_patch_learning_effectiveness.get("satisfied_count") or 0),
        "failed_count": str(repair_patch_learning_effectiveness.get("failed_count") or 0),
        "success_rate": str(repair_patch_learning_effectiveness.get("success_rate") or "0.00"),
        "top_reuse": compact(repair_patch_learning_effectiveness.get("top_reuse") or "", 240),
        "top_avoid": compact(repair_patch_learning_effectiveness.get("top_avoid") or "", 240),
    }
    patch_strategy_next = repair_patch_strategy.get("next_need") if isinstance(repair_patch_strategy.get("next_need"), dict) else {}
    patch_strategy_summary = {
        "status": str(repair_patch_strategy.get("status") or ""),
        "focus": compact(repair_patch_strategy.get("focus") or "", 240),
        "learned_reuse": compact(repair_patch_strategy.get("learned_reuse") or "", 240),
        "learned_avoid": compact(repair_patch_strategy.get("learned_avoid") or "", 240),
        "strategy_learned_reuse": compact(repair_patch_strategy.get("strategy_learned_reuse") or "", 240),
        "strategy_learned_avoid": compact(repair_patch_strategy.get("strategy_learned_avoid") or "", 240),
        "next_need_kind": str(patch_strategy_next.get("kind") or ""),
        "next_need_query": compact(patch_strategy_next.get("query") or "", 260),
    }
    patch_strategy_effectiveness_summary = {
        "used_count": str(repair_patch_strategy_effectiveness.get("used_count") or 0),
        "satisfied_count": str(repair_patch_strategy_effectiveness.get("satisfied_count") or 0),
        "failed_count": str(repair_patch_strategy_effectiveness.get("failed_count") or 0),
        "success_rate": str(repair_patch_strategy_effectiveness.get("success_rate") or "0.00"),
        "top_reuse": compact(repair_patch_strategy_effectiveness.get("top_reuse") or "", 240),
        "top_avoid": compact(repair_patch_strategy_effectiveness.get("top_avoid") or "", 240),
    }
    command_effectiveness_summary = {
        "used_count": str(repair_command_effectiveness.get("used_count") or 0),
        "satisfied_count": str(repair_command_effectiveness.get("satisfied_count") or 0),
        "failed_count": str(repair_command_effectiveness.get("failed_count") or 0),
        "success_rate": str(repair_command_effectiveness.get("success_rate") or "0.00"),
        "top_reuse": compact(repair_command_effectiveness.get("top_reuse") or "", 240),
        "top_avoid": compact(repair_command_effectiveness.get("top_avoid") or "", 240),
    }
    command_strategy_next = repair_command_strategy.get("next_need") if isinstance(repair_command_strategy.get("next_need"), dict) else {}
    command_strategy_current = repair_command_strategy.get("current_recipe") if isinstance(repair_command_strategy.get("current_recipe"), dict) else {}
    command_strategy_summary = {
        "status": str(repair_command_strategy.get("status") or ""),
        "focus": compact(repair_command_strategy.get("focus") or "", 240),
        "learned_reuse": compact(repair_command_strategy.get("learned_reuse") or "", 240),
        "learned_avoid": compact(repair_command_strategy.get("learned_avoid") or "", 240),
        "current_apply": compact(command_strategy_current.get("apply") or "", 240),
        "next_need_kind": str(command_strategy_next.get("kind") or ""),
        "next_need_query": compact(command_strategy_next.get("query") or "", 260),
    }
    command_strategy_effectiveness_summary = {
        "used_count": str(repair_command_strategy_effectiveness.get("used_count") or 0),
        "satisfied_count": str(repair_command_strategy_effectiveness.get("satisfied_count") or 0),
        "failed_count": str(repair_command_strategy_effectiveness.get("failed_count") or 0),
        "success_rate": str(repair_command_strategy_effectiveness.get("success_rate") or "0.00"),
        "top_reuse": compact(repair_command_strategy_effectiveness.get("top_reuse") or "", 240),
        "top_avoid": compact(repair_command_strategy_effectiveness.get("top_avoid") or "", 240),
    }
    drift_next = harness_environment_drift.get("next_need") if isinstance(harness_environment_drift.get("next_need"), dict) else {}
    drift_summary = {
        "status": str(harness_environment_drift.get("status") or ""),
        "detail": compact(harness_environment_drift.get("detail") or "", 260),
        "history_count": str(harness_environment_drift.get("history_count") or 0),
        "gained_count": str(harness_environment_drift.get("gained_count") or 0),
        "lost_count": str(harness_environment_drift.get("lost_count") or 0),
        "next_need_kind": str(drift_next.get("kind") or ""),
        "next_need_query": compact(drift_next.get("query") or "", 260),
    }
    drift_effectiveness_summary = {
        "used_count": str(harness_environment_drift_effectiveness.get("used_count") or 0),
        "satisfied_count": str(harness_environment_drift_effectiveness.get("satisfied_count") or 0),
        "failed_count": str(harness_environment_drift_effectiveness.get("failed_count") or 0),
        "success_rate": str(harness_environment_drift_effectiveness.get("success_rate") or "0.00"),
        "top_reuse": compact(harness_environment_drift_effectiveness.get("top_reuse") or "", 240),
        "top_avoid": compact(harness_environment_drift_effectiveness.get("top_avoid") or "", 240),
    }
    decision_next = repair_decision.get("next_need") if isinstance(repair_decision.get("next_need"), dict) else {}
    decision_summary = {
        "status": str(repair_decision.get("status") or ""),
        "decision": str(repair_decision.get("decision") or ""),
        "focus": compact(repair_decision.get("focus") or "", 240),
        "next_need_kind": str(decision_next.get("kind") or ""),
        "next_need_query": compact(decision_next.get("query") or "", 240),
    }
    decision_effectiveness_summary = {
        "used_count": str(repair_decision_effectiveness.get("used_count") or 0),
        "satisfied_count": str(repair_decision_effectiveness.get("satisfied_count") or 0),
        "failed_count": str(repair_decision_effectiveness.get("failed_count") or 0),
        "success_rate": str(repair_decision_effectiveness.get("success_rate") or "0.00"),
        "top_reuse": compact(repair_decision_effectiveness.get("top_reuse") or "", 240),
        "top_avoid": compact(repair_decision_effectiveness.get("top_avoid") or "", 240),
    }
    adaptation_next = harness_adaptation.get("next_need") if isinstance(harness_adaptation.get("next_need"), dict) else {}
    adaptation_summary = {
        "status": str(harness_adaptation.get("status") or ""),
        "focus": compact(harness_adaptation.get("focus") or "", 260),
        "next_need_kind": str(adaptation_next.get("kind") or ""),
        "next_need_query": compact(adaptation_next.get("query") or "", 260),
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
            "result": f"field={field_trajectory['field']} mini_task={field_trajectory['mini_task']} recalled={recall_count} top={recall_top_status} lessons={lessons_summary['lesson_count']} effectiveness_used={effectiveness_summary['used_count']} action_trace_effectiveness_used={action_trace_effectiveness_summary['used_count']} draft_effectiveness_used={draft_effectiveness_summary['used_count']} draft_strategy={draft_strategy_summary['status']} patch_draft={patch_draft_summary['status']} patch_draft_effectiveness_used={patch_draft_effectiveness_summary['used_count']} patch_review={patch_review_summary['status']} patch_review_effectiveness_used={patch_review_effectiveness_summary['used_count']} patch_learning={patch_learning_summary['status']} patch_learning_effectiveness_used={patch_learning_effectiveness_summary['used_count']} patch_strategy={patch_strategy_summary['status']} patch_strategy_effectiveness_used={patch_strategy_effectiveness_summary['used_count']} command_effectiveness_used={command_effectiveness_summary['used_count']} command_strategy={command_strategy_summary['status']} command_strategy_effectiveness_used={command_strategy_effectiveness_summary['used_count']} decision_effectiveness_used={decision_effectiveness_summary['used_count']} drift={drift_summary['status']} drift_effectiveness_used={drift_effectiveness_summary['used_count']} success_rate={effectiveness_summary['success_rate']} decision={decision_summary['decision']}",
            "status": "satisfied",
        },
        {
            "kind": "Action",
            "action": "choose harness adaptation focus",
            "result": f"status={adaptation_summary['status']} focus={adaptation_summary['focus']} next={adaptation_summary['next_need_kind']} {adaptation_summary['next_need_query']}",
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
        "action_trace_effectiveness": action_trace_effectiveness_summary,
        "repair_draft_effectiveness": draft_effectiveness_summary,
        "repair_draft_strategy": draft_strategy_summary,
        "repair_patch_draft": patch_draft_summary,
        "repair_patch_draft_effectiveness": patch_draft_effectiveness_summary,
        "repair_patch_review": patch_review_summary,
        "repair_patch_review_effectiveness": patch_review_effectiveness_summary,
        "repair_patch_learning": patch_learning_summary,
        "repair_patch_learning_effectiveness": patch_learning_effectiveness_summary,
        "repair_patch_strategy": patch_strategy_summary,
        "repair_patch_strategy_effectiveness": patch_strategy_effectiveness_summary,
        "repair_command_effectiveness": command_effectiveness_summary,
        "repair_command_strategy": command_strategy_summary,
        "repair_command_strategy_effectiveness": command_strategy_effectiveness_summary,
        "harness_environment_drift": drift_summary,
        "harness_environment_drift_status": drift_summary["status"],
        "harness_environment_drift_detail": drift_summary["detail"],
        "harness_environment_drift_history_count": drift_summary["history_count"],
        "harness_environment_drift_gained_count": drift_summary["gained_count"],
        "harness_environment_drift_lost_count": drift_summary["lost_count"],
        "harness_environment_drift_next_need_kind": drift_summary["next_need_kind"],
        "harness_environment_drift_next_need_query": drift_summary["next_need_query"],
        "harness_environment_drift_effectiveness": drift_effectiveness_summary,
        "repair_decision": decision_summary,
        "repair_decision_effectiveness": decision_effectiveness_summary,
        "harness_adaptation": adaptation_summary,
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
        f"action_trace_effectiveness_used: `{record.get('action_trace_effectiveness', {}).get('used_count', '0')}`",
        f"action_trace_effectiveness_success_rate: `{record.get('action_trace_effectiveness', {}).get('success_rate', '0.00')}`",
        f"repair_draft_effectiveness_used: `{record.get('repair_draft_effectiveness', {}).get('used_count', '0')}`",
        f"repair_draft_effectiveness_success_rate: `{record.get('repair_draft_effectiveness', {}).get('success_rate', '0.00')}`",
        f"repair_draft_strategy: `{record.get('repair_draft_strategy', {}).get('status', 'none') or 'none'}`",
        f"repair_draft_strategy_focus: {record.get('repair_draft_strategy', {}).get('focus', 'none') or 'none'}",
        f"repair_draft_strategy_next: `{record.get('repair_draft_strategy', {}).get('next_need_kind', 'verify')} {record.get('repair_draft_strategy', {}).get('next_need_query', '')}`",
        f"repair_patch_draft: `{record.get('repair_patch_draft', {}).get('status', 'none') or 'none'}`",
        f"repair_patch_draft_has_patch: `{record.get('repair_patch_draft', {}).get('has_patch', 'false') or 'false'}`",
        f"repair_patch_draft_effectiveness_used: `{record.get('repair_patch_draft_effectiveness', {}).get('used_count', '0')}`",
        f"repair_patch_draft_effectiveness_success_rate: `{record.get('repair_patch_draft_effectiveness', {}).get('success_rate', '0.00')}`",
        f"repair_patch_review: `{record.get('repair_patch_review', {}).get('status', 'none') or 'none'}`",
        f"repair_patch_review_check: `{record.get('repair_patch_review', {}).get('check_status', 'none') or 'none'}`",
        f"repair_patch_review_effectiveness_used: `{record.get('repair_patch_review_effectiveness', {}).get('used_count', '0')}`",
        f"repair_patch_review_effectiveness_success_rate: `{record.get('repair_patch_review_effectiveness', {}).get('success_rate', '0.00')}`",
        f"repair_command_effectiveness_used: `{record.get('repair_command_effectiveness', {}).get('used_count', '0')}`",
        f"repair_command_effectiveness_success_rate: `{record.get('repair_command_effectiveness', {}).get('success_rate', '0.00')}`",
        f"repair_command_strategy: `{record.get('repair_command_strategy', {}).get('status', 'none')}`",
        f"repair_command_strategy_focus: {record.get('repair_command_strategy', {}).get('focus', 'none') or 'none'}",
        f"repair_command_strategy_next: `{record.get('repair_command_strategy', {}).get('next_need_kind', 'verify')} {record.get('repair_command_strategy', {}).get('next_need_query', '')}`",
        f"repair_command_strategy_effectiveness_used: `{record.get('repair_command_strategy_effectiveness', {}).get('used_count', '0')}`",
        f"repair_command_strategy_effectiveness_success_rate: `{record.get('repair_command_strategy_effectiveness', {}).get('success_rate', '0.00')}`",
        f"harness_environment_drift: `{record.get('harness_environment_drift', {}).get('status', 'none') or 'none'}`",
        f"harness_environment_drift_detail: `{record.get('harness_environment_drift', {}).get('detail', 'none') or 'none'}`",
        f"harness_environment_drift_effectiveness_used: `{record.get('harness_environment_drift_effectiveness', {}).get('used_count', '0')}`",
        f"harness_environment_drift_effectiveness_success_rate: `{record.get('harness_environment_drift_effectiveness', {}).get('success_rate', '0.00')}`",
        f"repair_decision: `{record.get('repair_decision', {}).get('decision', 'none') or 'none'}`",
        f"repair_decision_focus: `{record.get('repair_decision', {}).get('focus', 'none') or 'none'}`",
        f"repair_decision_effectiveness_used: `{record.get('repair_decision_effectiveness', {}).get('used_count', '0')}`",
        f"repair_decision_effectiveness_success_rate: `{record.get('repair_decision_effectiveness', {}).get('success_rate', '0.00')}`",
        f"harness_adaptation: `{record.get('harness_adaptation', {}).get('status', 'none') or 'none'}`",
        f"harness_adaptation_focus: `{record.get('harness_adaptation', {}).get('focus', 'none') or 'none'}`",
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
repair_draft_effectiveness_json = session_dir / "REPAIR_DRAFT_EFFECTIVENESS.json"
repair_draft_effectiveness_md = session_dir / "REPAIR_DRAFT_EFFECTIVENESS.md"
repair_draft_strategy_json = session_dir / "REPAIR_DRAFT_STRATEGY.json"
repair_draft_strategy_md = session_dir / "REPAIR_DRAFT_STRATEGY.md"
repair_patch_draft_json = session_dir / "REPAIR_PATCH_DRAFT.json"
repair_patch_draft_md = session_dir / "REPAIR_PATCH_DRAFT.md"
repair_patch_draft_effectiveness_json = session_dir / "REPAIR_PATCH_DRAFT_EFFECTIVENESS.json"
repair_patch_draft_effectiveness_md = session_dir / "REPAIR_PATCH_DRAFT_EFFECTIVENESS.md"
repair_patch_review_json = session_dir / "REPAIR_PATCH_REVIEW.json"
repair_patch_review_md = session_dir / "REPAIR_PATCH_REVIEW.md"
repair_patch_review_effectiveness_json = session_dir / "REPAIR_PATCH_REVIEW_EFFECTIVENESS.json"
repair_patch_review_effectiveness_md = session_dir / "REPAIR_PATCH_REVIEW_EFFECTIVENESS.md"
repair_patch_apply_json = session_dir / "REPAIR_PATCH_APPLY.json"
repair_patch_apply_md = session_dir / "REPAIR_PATCH_APPLY.md"
repair_patch_verify_json = session_dir / "REPAIR_PATCH_VERIFY.json"
repair_patch_verify_md = session_dir / "REPAIR_PATCH_VERIFY.md"
repair_patch_learning_json = session_dir / "REPAIR_PATCH_LEARNING.json"
repair_patch_learning_md = session_dir / "REPAIR_PATCH_LEARNING.md"
repair_patch_learning_effectiveness_json = session_dir / "REPAIR_PATCH_LEARNING_EFFECTIVENESS.json"
repair_patch_learning_effectiveness_md = session_dir / "REPAIR_PATCH_LEARNING_EFFECTIVENESS.md"
repair_patch_strategy_json = session_dir / "REPAIR_PATCH_STRATEGY.json"
repair_patch_strategy_md = session_dir / "REPAIR_PATCH_STRATEGY.md"
repair_patch_strategy_effectiveness_json = session_dir / "REPAIR_PATCH_STRATEGY_EFFECTIVENESS.json"
repair_patch_strategy_effectiveness_md = session_dir / "REPAIR_PATCH_STRATEGY_EFFECTIVENESS.md"
repair_command_effectiveness_json = session_dir / "REPAIR_COMMAND_EFFECTIVENESS.json"
repair_command_effectiveness_md = session_dir / "REPAIR_COMMAND_EFFECTIVENESS.md"
repair_command_strategy_json = session_dir / "REPAIR_COMMAND_STRATEGY.json"
repair_command_strategy_md = session_dir / "REPAIR_COMMAND_STRATEGY.md"
repair_command_strategy_effectiveness_json = session_dir / "REPAIR_COMMAND_STRATEGY_EFFECTIVENESS.json"
repair_command_strategy_effectiveness_md = session_dir / "REPAIR_COMMAND_STRATEGY_EFFECTIVENESS.md"
repair_decision_effectiveness_json = session_dir / "REPAIR_DECISION_EFFECTIVENESS.json"
repair_decision_effectiveness_md = session_dir / "REPAIR_DECISION_EFFECTIVENESS.md"
repair_decision_json = session_dir / "REPAIR_DECISION.json"
repair_decision_md = session_dir / "REPAIR_DECISION.md"
harness_adaptation_effectiveness_json = session_dir / "HARNESS_ADAPTATION_EFFECTIVENESS.json"
harness_adaptation_effectiveness_md = session_dir / "HARNESS_ADAPTATION_EFFECTIVENESS.md"
harness_environment_profile_json = session_dir / "HARNESS_ENVIRONMENT_PROFILE.json"
harness_environment_profile_md = session_dir / "HARNESS_ENVIRONMENT_PROFILE.md"
harness_environment_drift_json = session_dir / "HARNESS_ENVIRONMENT_DRIFT.json"
harness_environment_drift_md = session_dir / "HARNESS_ENVIRONMENT_DRIFT.md"
harness_environment_drift_effectiveness_json = session_dir / "HARNESS_ENVIRONMENT_DRIFT_EFFECTIVENESS.json"
harness_environment_drift_effectiveness_md = session_dir / "HARNESS_ENVIRONMENT_DRIFT_EFFECTIVENESS.md"
harness_adaptation_json = session_dir / "HARNESS_ADAPTATION.json"
harness_adaptation_md = session_dir / "HARNESS_ADAPTATION.md"
adapter_context_md = session_dir / "ADAPTER_CONTEXT.md"
code_context_md = session_dir / "CODE_CONTEXT.md"
field_trajectory_md = session_dir / "FIELD_TRAJECTORY.md"
action_trace_md = session_dir / "ACTION_TRACE.md"
action_trace_json = session_dir / "ACTION_TRACE.json"
action_trace_effectiveness_json = session_dir / "ACTION_TRACE_EFFECTIVENESS.json"
action_trace_effectiveness_md = session_dir / "ACTION_TRACE_EFFECTIVENESS.md"
repair_plan_json = session_dir / "REPAIR_PLAN.json"
environment_profile_journal = repair_root / "environment-profiles.jsonl"
outcome_command = (
    "octopus repair score <trace-index> satisfied \"repair improved Feed\""
)
outcome_commands = [
    outcome_command,
    "octopus repair score <trace-index> partial \"repair partly improved Feed\"",
    "octopus repair score <trace-index> failed \"repair did not improve Feed\"",
]
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
action_trace_effectiveness = build_action_trace_effectiveness(repair_outcomes)
repair_draft_effectiveness = build_repair_draft_effectiveness(repair_outcomes)
repair_patch_draft_effectiveness = build_repair_patch_draft_effectiveness(repair_outcomes)
repair_patch_review_effectiveness = build_repair_patch_review_effectiveness(repair_outcomes)
repair_patch_learning = build_repair_patch_learning(repair_outcomes)
repair_patch_learning_effectiveness = build_repair_patch_learning_effectiveness(repair_outcomes)
repair_patch_strategy_effectiveness = build_repair_patch_strategy_effectiveness(repair_outcomes)
repair_command_effectiveness = build_repair_command_effectiveness(repair_outcomes)
repair_command_strategy_effectiveness = build_repair_command_strategy_effectiveness(repair_outcomes)
repair_decision_effectiveness = build_repair_decision_effectiveness(repair_outcomes)
repair_command_recipe = build_repair_commands(
    workspace,
    target_tentacle,
    candidate,
    code_context,
    commands,
)
repair_command_strategy = build_repair_command_strategy(
    repair_command_effectiveness,
    repair_command_strategy_effectiveness,
    repair_command_recipe,
    code_context["tentacle"],
    code_context["tool"],
)
harness_adaptation_effectiveness = build_harness_adaptation_effectiveness(repair_outcomes)
harness_environment_drift_effectiveness = build_harness_environment_drift_effectiveness(repair_outcomes)
harness_environment_profile = build_harness_environment_profile(
    workspace,
    harness_environment_profile_json,
    state,
    adapter_context,
    code_context,
    field_trajectory,
    repair_outcomes,
    harness_adaptation_effectiveness,
)
profile_history = load_profile_history(environment_profile_journal)
harness_environment_drift = build_harness_environment_drift(
    workspace,
    harness_environment_drift_json,
    harness_environment_profile,
    profile_history,
)
repair_decision = build_repair_decision(
    repair_lessons,
    repair_lesson_effectiveness,
    repair_decision_effectiveness,
    repair_recall,
    target_tentacle,
    target_tool,
)
harness_adaptation = build_harness_adaptation(
    workspace,
    harness_adaptation_json,
    code_context,
    adapter_context,
    harness_environment_profile,
    harness_environment_drift,
    field_trajectory,
    repair_recall,
    repair_lessons,
    repair_lesson_effectiveness,
    harness_adaptation_effectiveness,
    harness_environment_drift_effectiveness,
    repair_decision,
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
    "action_trace_effectiveness": rel(action_trace_effectiveness_md, workspace),
    "action_trace_effectiveness_json": rel(action_trace_effectiveness_json, workspace),
    "repair_draft_effectiveness": rel(repair_draft_effectiveness_md, workspace),
    "repair_draft_effectiveness_json": rel(repair_draft_effectiveness_json, workspace),
    "repair_draft_strategy": rel(repair_draft_strategy_md, workspace),
    "repair_draft_strategy_json": rel(repair_draft_strategy_json, workspace),
    "repair_patch_draft": rel(repair_patch_draft_md, workspace),
    "repair_patch_draft_json": rel(repair_patch_draft_json, workspace),
    "repair_patch_draft_effectiveness": rel(repair_patch_draft_effectiveness_md, workspace),
    "repair_patch_draft_effectiveness_json": rel(repair_patch_draft_effectiveness_json, workspace),
    "repair_patch_review": rel(repair_patch_review_md, workspace),
    "repair_patch_review_json": rel(repair_patch_review_json, workspace),
    "repair_patch_review_effectiveness": rel(repair_patch_review_effectiveness_md, workspace),
    "repair_patch_review_effectiveness_json": rel(repair_patch_review_effectiveness_json, workspace),
    "repair_patch_apply": rel(repair_patch_apply_md, workspace),
    "repair_patch_apply_json": rel(repair_patch_apply_json, workspace),
    "repair_patch_verify": rel(repair_patch_verify_md, workspace),
    "repair_patch_verify_json": rel(repair_patch_verify_json, workspace),
    "repair_patch_learning": rel(repair_patch_learning_md, workspace),
    "repair_patch_learning_json": rel(repair_patch_learning_json, workspace),
    "repair_patch_learning_effectiveness": rel(repair_patch_learning_effectiveness_md, workspace),
    "repair_patch_learning_effectiveness_json": rel(repair_patch_learning_effectiveness_json, workspace),
    "repair_patch_strategy": rel(repair_patch_strategy_md, workspace),
    "repair_patch_strategy_json": rel(repair_patch_strategy_json, workspace),
    "repair_patch_strategy_effectiveness": rel(repair_patch_strategy_effectiveness_md, workspace),
    "repair_patch_strategy_effectiveness_json": rel(repair_patch_strategy_effectiveness_json, workspace),
    "repair_command_effectiveness": rel(repair_command_effectiveness_md, workspace),
    "repair_command_effectiveness_json": rel(repair_command_effectiveness_json, workspace),
    "repair_command_strategy": rel(repair_command_strategy_md, workspace),
    "repair_command_strategy_json": rel(repair_command_strategy_json, workspace),
    "repair_command_strategy_effectiveness": rel(repair_command_strategy_effectiveness_md, workspace),
    "repair_command_strategy_effectiveness_json": rel(repair_command_strategy_effectiveness_json, workspace),
    "repair_decision_effectiveness": rel(repair_decision_effectiveness_md, workspace),
    "repair_decision_effectiveness_json": rel(repair_decision_effectiveness_json, workspace),
    "repair_decision": rel(repair_decision_md, workspace),
    "repair_decision_json": rel(repair_decision_json, workspace),
    "harness_adaptation_effectiveness": rel(harness_adaptation_effectiveness_md, workspace),
    "harness_adaptation_effectiveness_json": rel(harness_adaptation_effectiveness_json, workspace),
    "harness_environment_profile": rel(harness_environment_profile_md, workspace),
    "harness_environment_profile_json": rel(harness_environment_profile_json, workspace),
    "harness_environment_drift": rel(harness_environment_drift_md, workspace),
    "harness_environment_drift_json": rel(harness_environment_drift_json, workspace),
    "harness_environment_drift_effectiveness": rel(harness_environment_drift_effectiveness_md, workspace),
    "harness_environment_drift_effectiveness_json": rel(harness_environment_drift_effectiveness_json, workspace),
    "environment_profile_journal": rel(environment_profile_journal, workspace),
    "harness_adaptation": rel(harness_adaptation_md, workspace),
    "harness_adaptation_json": rel(harness_adaptation_json, workspace),
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
    "action_trace_effectiveness_summary": {
        "used_count": action_trace_effectiveness["used_count"],
        "satisfied_count": action_trace_effectiveness["satisfied_count"],
        "failed_count": action_trace_effectiveness["failed_count"],
        "success_rate": action_trace_effectiveness["success_rate"],
        "top_reuse": action_trace_effectiveness["top_reuse"],
        "top_avoid": action_trace_effectiveness["top_avoid"],
    },
    "repair_draft_effectiveness_summary": {
        "used_count": repair_draft_effectiveness["used_count"],
        "satisfied_count": repair_draft_effectiveness["satisfied_count"],
        "failed_count": repair_draft_effectiveness["failed_count"],
        "success_rate": repair_draft_effectiveness["success_rate"],
        "top_reuse": repair_draft_effectiveness["top_reuse"],
        "top_avoid": repair_draft_effectiveness["top_avoid"],
    },
    "repair_patch_draft_effectiveness_summary": {
        "used_count": repair_patch_draft_effectiveness["used_count"],
        "satisfied_count": repair_patch_draft_effectiveness["satisfied_count"],
        "failed_count": repair_patch_draft_effectiveness["failed_count"],
        "success_rate": repair_patch_draft_effectiveness["success_rate"],
        "top_reuse": repair_patch_draft_effectiveness["top_reuse"],
        "top_avoid": repair_patch_draft_effectiveness["top_avoid"],
    },
    "repair_patch_review_effectiveness_summary": {
        "used_count": repair_patch_review_effectiveness["used_count"],
        "satisfied_count": repair_patch_review_effectiveness["satisfied_count"],
        "failed_count": repair_patch_review_effectiveness["failed_count"],
        "success_rate": repair_patch_review_effectiveness["success_rate"],
        "top_reuse": repair_patch_review_effectiveness["top_reuse"],
        "top_avoid": repair_patch_review_effectiveness["top_avoid"],
    },
    "repair_patch_learning_summary": {
        "status": repair_patch_learning["status"],
        "used_count": repair_patch_learning["used_count"],
        "verified_count": repair_patch_learning["verified_count"],
        "verified_satisfied_count": repair_patch_learning["verified_satisfied_count"],
        "verified_failed_count": repair_patch_learning["verified_failed_count"],
        "success_rate": repair_patch_learning["success_rate"],
        "verified_success_rate": repair_patch_learning["verified_success_rate"],
        "top_reuse": repair_patch_learning["top_reuse"],
        "top_avoid": repair_patch_learning["top_avoid"],
        "next_need": repair_patch_learning["next_need"],
    },
    "repair_patch_learning_effectiveness_summary": {
        "used_count": repair_patch_learning_effectiveness["used_count"],
        "satisfied_count": repair_patch_learning_effectiveness["satisfied_count"],
        "failed_count": repair_patch_learning_effectiveness["failed_count"],
        "success_rate": repair_patch_learning_effectiveness["success_rate"],
        "top_reuse": repair_patch_learning_effectiveness["top_reuse"],
        "top_avoid": repair_patch_learning_effectiveness["top_avoid"],
    },
    "repair_patch_strategy_effectiveness_summary": {
        "used_count": repair_patch_strategy_effectiveness["used_count"],
        "satisfied_count": repair_patch_strategy_effectiveness["satisfied_count"],
        "failed_count": repair_patch_strategy_effectiveness["failed_count"],
        "success_rate": repair_patch_strategy_effectiveness["success_rate"],
        "top_reuse": repair_patch_strategy_effectiveness["top_reuse"],
        "top_avoid": repair_patch_strategy_effectiveness["top_avoid"],
    },
    "repair_command_effectiveness_summary": {
        "used_count": repair_command_effectiveness["used_count"],
        "satisfied_count": repair_command_effectiveness["satisfied_count"],
        "failed_count": repair_command_effectiveness["failed_count"],
        "success_rate": repair_command_effectiveness["success_rate"],
        "top_reuse": repair_command_effectiveness["top_reuse"],
        "top_avoid": repair_command_effectiveness["top_avoid"],
    },
    "repair_command_strategy_summary": {
        "status": repair_command_strategy["status"],
        "focus": repair_command_strategy["focus"],
        "learned_reuse": repair_command_strategy["learned_reuse"],
        "learned_avoid": repair_command_strategy["learned_avoid"],
        "strategy_learned_reuse": repair_command_strategy["strategy_learned_reuse"],
        "strategy_learned_avoid": repair_command_strategy["strategy_learned_avoid"],
        "next_need": repair_command_strategy["next_need"],
    },
    "repair_command_strategy_effectiveness_summary": {
        "used_count": repair_command_strategy_effectiveness["used_count"],
        "satisfied_count": repair_command_strategy_effectiveness["satisfied_count"],
        "failed_count": repair_command_strategy_effectiveness["failed_count"],
        "success_rate": repair_command_strategy_effectiveness["success_rate"],
        "top_reuse": repair_command_strategy_effectiveness["top_reuse"],
        "top_avoid": repair_command_strategy_effectiveness["top_avoid"],
    },
    "repair_decision_effectiveness_summary": {
        "used_count": repair_decision_effectiveness["used_count"],
        "satisfied_count": repair_decision_effectiveness["satisfied_count"],
        "failed_count": repair_decision_effectiveness["failed_count"],
        "success_rate": repair_decision_effectiveness["success_rate"],
        "top_reuse": repair_decision_effectiveness["top_reuse"],
        "top_avoid": repair_decision_effectiveness["top_avoid"],
    },
    "repair_decision_summary": {
        "status": repair_decision["status"],
        "decision": repair_decision["decision"],
        "focus": repair_decision["focus"],
        "next_need": repair_decision["next_need"],
    },
    "harness_adaptation_effectiveness_summary": {
        "used_count": harness_adaptation_effectiveness["used_count"],
        "satisfied_count": harness_adaptation_effectiveness["satisfied_count"],
        "failed_count": harness_adaptation_effectiveness["failed_count"],
        "success_rate": harness_adaptation_effectiveness["success_rate"],
        "top_reuse": harness_adaptation_effectiveness["top_reuse"],
        "top_avoid": harness_adaptation_effectiveness["top_avoid"],
    },
    "harness_environment_profile_summary": {
        "status": harness_environment_profile["status"],
        "profile_id": harness_environment_profile["profile_id"],
        "execution_mode": harness_environment_profile["execution_mode"],
        "provider_mode": harness_environment_profile["provider_mode"],
        "desktop_mode": harness_environment_profile["desktop_mode"],
        "capabilities": harness_environment_profile["capabilities"],
        "constraints": harness_environment_profile["constraints"],
        "next_need": harness_environment_profile["next_need"],
    },
    "harness_environment_drift_summary": {
        "status": harness_environment_drift["status"],
        "summary": harness_environment_drift["summary"],
        "detail": harness_environment_drift["detail"],
        "history_count": harness_environment_drift["history_count"],
        "gained_count": harness_environment_drift["gained_count"],
        "lost_count": harness_environment_drift["lost_count"],
        "next_need": harness_environment_drift["next_need"],
    },
    "harness_environment_drift_effectiveness_summary": {
        "used_count": harness_environment_drift_effectiveness["used_count"],
        "satisfied_count": harness_environment_drift_effectiveness["satisfied_count"],
        "failed_count": harness_environment_drift_effectiveness["failed_count"],
        "success_rate": harness_environment_drift_effectiveness["success_rate"],
        "top_reuse": harness_environment_drift_effectiveness["top_reuse"],
        "top_avoid": harness_environment_drift_effectiveness["top_avoid"],
    },
    "harness_adaptation_summary": {
        "status": harness_adaptation["status"],
        "focus": harness_adaptation["focus"],
        "next_need": harness_adaptation["next_need"],
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
action_trace_effectiveness_json.write_text(
    json.dumps(action_trace_effectiveness, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
action_trace_effectiveness_md.write_text(
    action_trace_effectiveness_markdown(
        action_trace_effectiveness,
        workspace,
        action_trace_effectiveness_json,
    ),
    encoding="utf-8",
)
repair_draft_effectiveness_json.write_text(
    json.dumps(repair_draft_effectiveness, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_draft_effectiveness_md.write_text(
    repair_draft_effectiveness_markdown(
        repair_draft_effectiveness,
        workspace,
        repair_draft_effectiveness_json,
    ),
    encoding="utf-8",
)
repair_patch_draft_effectiveness_json.write_text(
    json.dumps(repair_patch_draft_effectiveness, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_patch_draft_effectiveness_md.write_text(
    repair_patch_draft_effectiveness_markdown(
        repair_patch_draft_effectiveness,
        workspace,
        repair_patch_draft_effectiveness_json,
    ),
    encoding="utf-8",
)
repair_patch_review_effectiveness_json.write_text(
    json.dumps(repair_patch_review_effectiveness, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_patch_review_effectiveness_md.write_text(
    repair_patch_review_effectiveness_markdown(
        repair_patch_review_effectiveness,
        workspace,
        repair_patch_review_effectiveness_json,
    ),
    encoding="utf-8",
)
repair_patch_learning_json.write_text(
    json.dumps(repair_patch_learning, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_patch_learning_md.write_text(
    repair_patch_learning_markdown(repair_patch_learning, workspace, repair_patch_learning_json),
    encoding="utf-8",
)
repair_patch_learning_effectiveness_json.write_text(
    json.dumps(repair_patch_learning_effectiveness, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_patch_learning_effectiveness_md.write_text(
    repair_patch_learning_effectiveness_markdown(
        repair_patch_learning_effectiveness,
        workspace,
        repair_patch_learning_effectiveness_json,
    ),
    encoding="utf-8",
)
repair_patch_strategy_effectiveness_json.write_text(
    json.dumps(repair_patch_strategy_effectiveness, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_patch_strategy_effectiveness_md.write_text(
    repair_patch_strategy_effectiveness_markdown(
        repair_patch_strategy_effectiveness,
        workspace,
        repair_patch_strategy_effectiveness_json,
    ),
    encoding="utf-8",
)
repair_command_effectiveness_json.write_text(
    json.dumps(repair_command_effectiveness, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_command_effectiveness_md.write_text(
    repair_command_effectiveness_markdown(
        repair_command_effectiveness,
        workspace,
        repair_command_effectiveness_json,
    ),
    encoding="utf-8",
)
repair_command_strategy_json.write_text(
    json.dumps(repair_command_strategy, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_command_strategy_md.write_text(
    repair_command_strategy_markdown(
        repair_command_strategy,
        workspace,
        repair_command_strategy_json,
    ),
    encoding="utf-8",
)
repair_command_strategy_effectiveness_json.write_text(
    json.dumps(repair_command_strategy_effectiveness, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_command_strategy_effectiveness_md.write_text(
    repair_command_strategy_effectiveness_markdown(
        repair_command_strategy_effectiveness,
        workspace,
        repair_command_strategy_effectiveness_json,
    ),
    encoding="utf-8",
)
repair_decision_effectiveness_json.write_text(
    json.dumps(repair_decision_effectiveness, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_decision_effectiveness_md.write_text(
    repair_decision_effectiveness_markdown(
        repair_decision_effectiveness,
        workspace,
        repair_decision_effectiveness_json,
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
harness_adaptation_effectiveness_json.write_text(
    json.dumps(harness_adaptation_effectiveness, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
harness_adaptation_effectiveness_md.write_text(
    harness_adaptation_effectiveness_markdown(
        harness_adaptation_effectiveness,
        workspace,
        harness_adaptation_effectiveness_json,
    ),
    encoding="utf-8",
)
harness_environment_profile_json.write_text(
    json.dumps(harness_environment_profile, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
harness_environment_profile_md.write_text(
    harness_environment_profile_markdown(
        harness_environment_profile,
        workspace,
        harness_environment_profile_json,
    ),
    encoding="utf-8",
)
harness_environment_drift_json.write_text(
    json.dumps(harness_environment_drift, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
harness_environment_drift_md.write_text(
    harness_environment_drift_markdown(
        harness_environment_drift,
        workspace,
        harness_environment_drift_json,
        environment_profile_journal,
    ),
    encoding="utf-8",
)
harness_environment_drift_effectiveness_json.write_text(
    json.dumps(harness_environment_drift_effectiveness, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
harness_environment_drift_effectiveness_md.write_text(
    harness_environment_drift_effectiveness_markdown(
        harness_environment_drift_effectiveness,
        workspace,
        harness_environment_drift_effectiveness_json,
    ),
    encoding="utf-8",
)
append_profile_history(environment_profile_journal, harness_environment_profile)
harness_adaptation_json.write_text(
    json.dumps(harness_adaptation, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
harness_adaptation_md.write_text(
    harness_adaptation_markdown(harness_adaptation, workspace, harness_adaptation_json),
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
repair_plan["inputs"]["repair_draft_effectiveness"] = rel(repair_draft_effectiveness_md, workspace)
repair_plan["inputs"]["repair_draft_effectiveness_json"] = rel(repair_draft_effectiveness_json, workspace)
repair_plan["inputs"]["repair_draft_strategy"] = rel(repair_draft_strategy_md, workspace)
repair_plan["inputs"]["repair_draft_strategy_json"] = rel(repair_draft_strategy_json, workspace)
repair_plan["inputs"]["repair_patch_draft"] = rel(repair_patch_draft_md, workspace)
repair_plan["inputs"]["repair_patch_draft_json"] = rel(repair_patch_draft_json, workspace)
repair_plan["inputs"]["repair_patch_draft_effectiveness"] = rel(repair_patch_draft_effectiveness_md, workspace)
repair_plan["inputs"]["repair_patch_draft_effectiveness_json"] = rel(repair_patch_draft_effectiveness_json, workspace)
repair_plan["inputs"]["repair_patch_review"] = rel(repair_patch_review_md, workspace)
repair_plan["inputs"]["repair_patch_review_json"] = rel(repair_patch_review_json, workspace)
repair_plan["inputs"]["repair_patch_review_effectiveness"] = rel(repair_patch_review_effectiveness_md, workspace)
repair_plan["inputs"]["repair_patch_review_effectiveness_json"] = rel(repair_patch_review_effectiveness_json, workspace)
repair_plan["inputs"]["repair_patch_apply"] = rel(repair_patch_apply_md, workspace)
repair_plan["inputs"]["repair_patch_apply_json"] = rel(repair_patch_apply_json, workspace)
repair_plan["inputs"]["repair_patch_verify"] = rel(repair_patch_verify_md, workspace)
repair_plan["inputs"]["repair_patch_verify_json"] = rel(repair_patch_verify_json, workspace)
repair_plan["inputs"]["repair_patch_learning"] = rel(repair_patch_learning_md, workspace)
repair_plan["inputs"]["repair_patch_learning_json"] = rel(repair_patch_learning_json, workspace)
repair_plan["inputs"]["repair_patch_learning_effectiveness"] = rel(repair_patch_learning_effectiveness_md, workspace)
repair_plan["inputs"]["repair_patch_learning_effectiveness_json"] = rel(repair_patch_learning_effectiveness_json, workspace)
repair_plan["inputs"]["repair_patch_strategy"] = rel(repair_patch_strategy_md, workspace)
repair_plan["inputs"]["repair_patch_strategy_json"] = rel(repair_patch_strategy_json, workspace)
repair_plan["inputs"]["repair_patch_strategy_effectiveness"] = rel(repair_patch_strategy_effectiveness_md, workspace)
repair_plan["inputs"]["repair_patch_strategy_effectiveness_json"] = rel(repair_patch_strategy_effectiveness_json, workspace)
repair_plan["inputs"]["repair_command_effectiveness"] = rel(repair_command_effectiveness_md, workspace)
repair_plan["inputs"]["repair_command_effectiveness_json"] = rel(repair_command_effectiveness_json, workspace)
repair_plan["inputs"]["repair_command_strategy"] = rel(repair_command_strategy_md, workspace)
repair_plan["inputs"]["repair_command_strategy_json"] = rel(repair_command_strategy_json, workspace)
repair_plan["inputs"]["repair_command_strategy_effectiveness"] = rel(repair_command_strategy_effectiveness_md, workspace)
repair_plan["inputs"]["repair_command_strategy_effectiveness_json"] = rel(repair_command_strategy_effectiveness_json, workspace)
repair_plan["inputs"]["repair_decision_effectiveness"] = rel(repair_decision_effectiveness_md, workspace)
repair_plan["inputs"]["repair_decision_effectiveness_json"] = rel(repair_decision_effectiveness_json, workspace)
repair_plan["inputs"]["repair_decision"] = rel(repair_decision_md, workspace)
repair_plan["inputs"]["repair_decision_json"] = rel(repair_decision_json, workspace)
repair_plan["inputs"]["harness_adaptation_effectiveness"] = rel(harness_adaptation_effectiveness_md, workspace)
repair_plan["inputs"]["harness_adaptation_effectiveness_json"] = rel(harness_adaptation_effectiveness_json, workspace)
repair_plan["inputs"]["harness_environment_profile"] = rel(harness_environment_profile_md, workspace)
repair_plan["inputs"]["harness_environment_profile_json"] = rel(harness_environment_profile_json, workspace)
repair_plan["inputs"]["harness_environment_drift"] = rel(harness_environment_drift_md, workspace)
repair_plan["inputs"]["harness_environment_drift_json"] = rel(harness_environment_drift_json, workspace)
repair_plan["inputs"]["harness_environment_drift_effectiveness"] = rel(harness_environment_drift_effectiveness_md, workspace)
repair_plan["inputs"]["harness_environment_drift_effectiveness_json"] = rel(harness_environment_drift_effectiveness_json, workspace)
repair_plan["inputs"]["environment_profile_journal"] = rel(environment_profile_journal, workspace)
repair_plan["inputs"]["harness_adaptation"] = rel(harness_adaptation_md, workspace)
repair_plan["inputs"]["harness_adaptation_json"] = rel(harness_adaptation_json, workspace)
repair_plan["inputs"]["action_trace"] = rel(action_trace_md, workspace)
repair_plan["inputs"]["action_trace_json"] = rel(action_trace_json, workspace)
repair_plan["inputs"]["action_trace_effectiveness"] = rel(action_trace_effectiveness_md, workspace)
repair_plan["inputs"]["action_trace_effectiveness_json"] = rel(action_trace_effectiveness_json, workspace)
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
repair_plan["action_trace_effectiveness_summary"] = session["action_trace_effectiveness_summary"]
repair_plan["repair_draft_effectiveness_summary"] = session["repair_draft_effectiveness_summary"]
repair_plan["repair_patch_draft_effectiveness_summary"] = session["repair_patch_draft_effectiveness_summary"]
repair_plan["repair_patch_review_effectiveness_summary"] = session["repair_patch_review_effectiveness_summary"]
repair_plan["repair_patch_learning_summary"] = session["repair_patch_learning_summary"]
repair_plan["repair_patch_learning_effectiveness_summary"] = session["repair_patch_learning_effectiveness_summary"]
repair_plan["repair_patch_strategy_effectiveness_summary"] = session["repair_patch_strategy_effectiveness_summary"]
repair_plan["repair_command_effectiveness_summary"] = session["repair_command_effectiveness_summary"]
repair_plan["repair_command_strategy_summary"] = session["repair_command_strategy_summary"]
repair_plan["repair_command_strategy_effectiveness_summary"] = session["repair_command_strategy_effectiveness_summary"]
repair_plan["commands"]["learned_reuse"] = repair_command_strategy.get("learned_reuse", "")
repair_plan["commands"]["learned_avoid"] = repair_command_strategy.get("learned_avoid", "")
repair_plan["commands"]["strategy_learned_reuse"] = repair_command_strategy.get("strategy_learned_reuse", "")
repair_plan["commands"]["strategy_learned_avoid"] = repair_command_strategy.get("strategy_learned_avoid", "")
repair_plan["repair_decision_effectiveness_summary"] = session["repair_decision_effectiveness_summary"]
repair_plan["repair_decision_summary"] = session["repair_decision_summary"]
repair_plan["harness_adaptation_effectiveness_summary"] = session["harness_adaptation_effectiveness_summary"]
repair_plan["harness_environment_profile_summary"] = session["harness_environment_profile_summary"]
repair_plan["harness_environment_drift_summary"] = session["harness_environment_drift_summary"]
repair_plan["harness_environment_drift_effectiveness_summary"] = session["harness_environment_drift_effectiveness_summary"]
repair_plan["harness_adaptation_summary"] = session["harness_adaptation_summary"]
repair_plan["review_boundary"] = "Review HARNESS_ADAPTATION, HARNESS_ENVIRONMENT_PROFILE, HARNESS_ENVIRONMENT_DRIFT, HARNESS_ENVIRONMENT_DRIFT_EFFECTIVENESS, HARNESS_ADAPTATION_EFFECTIVENESS, ACTION_TRACE.md, ACTION_TRACE.json, ACTION_TRACE_EFFECTIVENESS, REPAIR_RECALL.json, REPAIR_LESSONS, REPAIR_LESSON_EFFECTIVENESS, REPAIR_DRAFT_EFFECTIVENESS, REPAIR_DRAFT_STRATEGY, REPAIR_PATCH_DRAFT, REPAIR_PATCH_DRAFT_EFFECTIVENESS, REPAIR_PATCH_REVIEW, REPAIR_PATCH_REVIEW_EFFECTIVENESS, REPAIR_PATCH_APPLY, REPAIR_PATCH_VERIFY, REPAIR_PATCH_LEARNING, REPAIR_PATCH_LEARNING_EFFECTIVENESS, REPAIR_PATCH_STRATEGY, REPAIR_PATCH_STRATEGY_EFFECTIVENESS, REPAIR_COMMAND_EFFECTIVENESS, REPAIR_COMMAND_STRATEGY, REPAIR_COMMAND_STRATEGY_EFFECTIVENESS, REPAIR_DECISION_EFFECTIVENESS, REPAIR_DECISION, ADAPTER_CONTEXT, FIELD_TRAJECTORY, CODE_CONTEXT, OUTCOME_MEMORY, DRAFT, and this plan before running commands."
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
            "# Choose one:",
            *[f"# {command}" for command in outcome_commands],
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
action_trace_effectiveness_lines = [
    (
        f"- {hint.get('direction')}"
        f" used={hint.get('used_count')}"
        f" satisfied={hint.get('satisfied_count')}"
        f" failed={hint.get('failed_count')}"
        f" success={hint.get('success_rate')}"
        f" trace={compact(hint.get('hint') or '', 180)}"
    )
    for hint in action_trace_effectiveness.get("hints", [])[:4]
] or ["- none"]
draft_effectiveness_lines = [
    (
        f"- {hint.get('direction')}"
        f" used={hint.get('used_count')}"
        f" satisfied={hint.get('satisfied_count')}"
        f" failed={hint.get('failed_count')}"
        f" success={hint.get('success_rate')}"
        f" provider={compact(hint.get('hint') or '', 180)}"
    )
    for hint in repair_draft_effectiveness.get("hints", [])[:4]
] or ["- none"]
patch_draft_effectiveness_lines = [
    (
        f"- {hint.get('direction')}"
        f" used={hint.get('used_count')}"
        f" satisfied={hint.get('satisfied_count')}"
        f" failed={hint.get('failed_count')}"
        f" success={hint.get('success_rate')}"
        f" patch={compact(hint.get('hint') or '', 180)}"
    )
    for hint in repair_patch_draft_effectiveness.get("hints", [])[:4]
] or ["- none"]
patch_review_effectiveness_lines = [
    (
        f"- {hint.get('direction')}"
        f" used={hint.get('used_count')}"
        f" satisfied={hint.get('satisfied_count')}"
        f" failed={hint.get('failed_count')}"
        f" success={hint.get('success_rate')}"
        f" review={compact(hint.get('hint') or '', 180)}"
    )
    for hint in repair_patch_review_effectiveness.get("hints", [])[:4]
] or ["- none"]
patch_learning_lines = [
    (
        f"- {hint.get('direction')}"
        f" used={hint.get('used_count')}"
        f" satisfied={hint.get('satisfied_count')}"
        f" failed={hint.get('failed_count')}"
        f" success={hint.get('success_rate')}"
        f" patch={compact(hint.get('hint') or '', 180)}"
    )
    for hint in repair_patch_learning.get("hints", [])[:4]
] or ["- none"]
patch_learning_effectiveness_lines = [
    (
        f"- {hint.get('direction')}"
        f" used={hint.get('used_count')}"
        f" satisfied={hint.get('satisfied_count')}"
        f" failed={hint.get('failed_count')}"
        f" success={hint.get('success_rate')}"
        f" learning={compact(hint.get('hint') or '', 180)}"
    )
    for hint in repair_patch_learning_effectiveness.get("hints", [])[:4]
] or ["- none"]
command_effectiveness_lines = [
    (
        f"- {hint.get('direction')}"
        f" used={hint.get('used_count')}"
        f" satisfied={hint.get('satisfied_count')}"
        f" failed={hint.get('failed_count')}"
        f" success={hint.get('success_rate')}"
        f" command={compact(hint.get('hint') or '', 180)}"
    )
    for hint in repair_command_effectiveness.get("hints", [])[:4]
] or ["- none"]
command_strategy_next = (
    repair_command_strategy.get("next_need")
    if isinstance(repair_command_strategy.get("next_need"), dict)
    else {}
)
command_strategy_current = (
    repair_command_strategy.get("current_recipe")
    if isinstance(repair_command_strategy.get("current_recipe"), dict)
    else {}
)
command_strategy_effectiveness_lines = [
    (
        f"- {hint.get('direction')}"
        f" used={hint.get('used_count')}"
        f" satisfied={hint.get('satisfied_count')}"
        f" failed={hint.get('failed_count')}"
        f" success={hint.get('success_rate')}"
        f" strategy={compact(hint.get('hint') or '', 180)}"
    )
    for hint in repair_command_strategy_effectiveness.get("hints", [])[:4]
] or ["- none"]
decision_effectiveness_lines = [
    (
        f"- {hint.get('direction')}"
        f" used={hint.get('used_count')}"
        f" satisfied={hint.get('satisfied_count')}"
        f" failed={hint.get('failed_count')}"
        f" success={hint.get('success_rate')}"
        f" decision={compact(hint.get('hint') or '', 180)}"
    )
    for hint in repair_decision_effectiveness.get("hints", [])[:4]
] or ["- none"]
adaptation_effectiveness_lines = [
    (
        f"- {hint.get('direction')}"
        f" used={hint.get('used_count')}"
        f" satisfied={hint.get('satisfied_count')}"
        f" failed={hint.get('failed_count')}"
        f" success={hint.get('success_rate')}"
        f" focus={compact(hint.get('hint') or '', 180)}"
    )
    for hint in harness_adaptation_effectiveness.get("hints", [])[:4]
] or ["- none"]
drift_effectiveness_lines = [
    (
        f"- {hint.get('direction')}"
        f" used={hint.get('used_count')}"
        f" satisfied={hint.get('satisfied_count')}"
        f" failed={hint.get('failed_count')}"
        f" success={hint.get('success_rate')}"
        f" drift={compact(hint.get('hint') or '', 180)}"
    )
    for hint in harness_environment_drift_effectiveness.get("hints", [])[:4]
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
            "action trace effectiveness:",
            f"- effectiveness artifact: `{rel(action_trace_effectiveness_md, workspace)}`",
            f"- effectiveness json: `{rel(action_trace_effectiveness_json, workspace)}`",
            f"- used: `{action_trace_effectiveness.get('used_count', 0)}` satisfied: `{action_trace_effectiveness.get('satisfied_count', 0)}` failed: `{action_trace_effectiveness.get('failed_count', 0)}` success_rate: `{action_trace_effectiveness.get('success_rate', '0.00')}`",
            *action_trace_effectiveness_lines,
            "",
            "repair draft effectiveness:",
            f"- effectiveness artifact: `{rel(repair_draft_effectiveness_md, workspace)}`",
            f"- effectiveness json: `{rel(repair_draft_effectiveness_json, workspace)}`",
            f"- used: `{repair_draft_effectiveness.get('used_count', 0)}` satisfied: `{repair_draft_effectiveness.get('satisfied_count', 0)}` failed: `{repair_draft_effectiveness.get('failed_count', 0)}` success_rate: `{repair_draft_effectiveness.get('success_rate', '0.00')}`",
            *draft_effectiveness_lines,
            "",
            "repair patch draft effectiveness:",
            f"- effectiveness artifact: `{rel(repair_patch_draft_effectiveness_md, workspace)}`",
            f"- effectiveness json: `{rel(repair_patch_draft_effectiveness_json, workspace)}`",
            f"- used: `{repair_patch_draft_effectiveness.get('used_count', 0)}` satisfied: `{repair_patch_draft_effectiveness.get('satisfied_count', 0)}` failed: `{repair_patch_draft_effectiveness.get('failed_count', 0)}` success_rate: `{repair_patch_draft_effectiveness.get('success_rate', '0.00')}`",
            *patch_draft_effectiveness_lines,
            "",
            "repair patch review effectiveness:",
            f"- effectiveness artifact: `{rel(repair_patch_review_effectiveness_md, workspace)}`",
            f"- effectiveness json: `{rel(repair_patch_review_effectiveness_json, workspace)}`",
            f"- used: `{repair_patch_review_effectiveness.get('used_count', 0)}` satisfied: `{repair_patch_review_effectiveness.get('satisfied_count', 0)}` failed: `{repair_patch_review_effectiveness.get('failed_count', 0)}` success_rate: `{repair_patch_review_effectiveness.get('success_rate', '0.00')}`",
            *patch_review_effectiveness_lines,
            "",
            "repair patch learning:",
            f"- learning artifact: `{rel(repair_patch_learning_md, workspace)}`",
            f"- learning json: `{rel(repair_patch_learning_json, workspace)}`",
            f"- status: `{repair_patch_learning.get('status')}` used: `{repair_patch_learning.get('used_count', 0)}` verified: `{repair_patch_learning.get('verified_count', 0)}` verified_success_rate: `{repair_patch_learning.get('verified_success_rate', '0.00')}`",
            f"- top_reuse: {compact(repair_patch_learning.get('top_reuse'), 180) or 'none'}",
            f"- top_avoid: {compact(repair_patch_learning.get('top_avoid'), 180) or 'none'}",
            *patch_learning_lines,
            "",
            "repair patch learning effectiveness:",
            f"- effectiveness artifact: `{rel(repair_patch_learning_effectiveness_md, workspace)}`",
            f"- effectiveness json: `{rel(repair_patch_learning_effectiveness_json, workspace)}`",
            f"- used: `{repair_patch_learning_effectiveness.get('used_count', 0)}` satisfied: `{repair_patch_learning_effectiveness.get('satisfied_count', 0)}` failed: `{repair_patch_learning_effectiveness.get('failed_count', 0)}` success_rate: `{repair_patch_learning_effectiveness.get('success_rate', '0.00')}`",
            *patch_learning_effectiveness_lines,
            "",
            "repair command effectiveness:",
            f"- effectiveness artifact: `{rel(repair_command_effectiveness_md, workspace)}`",
            f"- effectiveness json: `{rel(repair_command_effectiveness_json, workspace)}`",
            f"- used: `{repair_command_effectiveness.get('used_count', 0)}` satisfied: `{repair_command_effectiveness.get('satisfied_count', 0)}` failed: `{repair_command_effectiveness.get('failed_count', 0)}` success_rate: `{repair_command_effectiveness.get('success_rate', '0.00')}`",
            *command_effectiveness_lines,
            "",
            "repair command strategy:",
            f"- strategy artifact: `{rel(repair_command_strategy_md, workspace)}`",
            f"- strategy json: `{rel(repair_command_strategy_json, workspace)}`",
            f"- status: `{repair_command_strategy.get('status')}` focus: {compact(repair_command_strategy.get('focus'), 220)}",
            f"- learned_reuse: {compact(repair_command_strategy.get('learned_reuse'), 180) or 'none'}",
            f"- learned_avoid: {compact(repair_command_strategy.get('learned_avoid'), 180) or 'none'}",
            f"- strategy_learned_reuse: {compact(repair_command_strategy.get('strategy_learned_reuse'), 180) or 'none'}",
            f"- strategy_learned_avoid: {compact(repair_command_strategy.get('strategy_learned_avoid'), 180) or 'none'}",
            f"- current_apply: {compact(command_strategy_current.get('apply'), 180) or 'none'}",
            f"- next_need: `{command_strategy_next.get('kind') or 'verify'} {command_strategy_next.get('query') or ''}`",
            "",
            "repair command strategy effectiveness:",
            f"- effectiveness artifact: `{rel(repair_command_strategy_effectiveness_md, workspace)}`",
            f"- effectiveness json: `{rel(repair_command_strategy_effectiveness_json, workspace)}`",
            f"- used: `{repair_command_strategy_effectiveness.get('used_count', 0)}` satisfied: `{repair_command_strategy_effectiveness.get('satisfied_count', 0)}` failed: `{repair_command_strategy_effectiveness.get('failed_count', 0)}` success_rate: `{repair_command_strategy_effectiveness.get('success_rate', '0.00')}`",
            *command_strategy_effectiveness_lines,
            "",
            "repair decision effectiveness:",
            f"- effectiveness artifact: `{rel(repair_decision_effectiveness_md, workspace)}`",
            f"- effectiveness json: `{rel(repair_decision_effectiveness_json, workspace)}`",
            f"- used: `{repair_decision_effectiveness.get('used_count', 0)}` satisfied: `{repair_decision_effectiveness.get('satisfied_count', 0)}` failed: `{repair_decision_effectiveness.get('failed_count', 0)}` success_rate: `{repair_decision_effectiveness.get('success_rate', '0.00')}`",
            *decision_effectiveness_lines,
            "",
            "harness adaptation effectiveness:",
            f"- effectiveness artifact: `{rel(harness_adaptation_effectiveness_md, workspace)}`",
            f"- effectiveness json: `{rel(harness_adaptation_effectiveness_json, workspace)}`",
            f"- used: `{harness_adaptation_effectiveness.get('used_count', 0)}` satisfied: `{harness_adaptation_effectiveness.get('satisfied_count', 0)}` failed: `{harness_adaptation_effectiveness.get('failed_count', 0)}` success_rate: `{harness_adaptation_effectiveness.get('success_rate', '0.00')}`",
            *adaptation_effectiveness_lines,
            "",
            "harness environment profile:",
            f"- profile artifact: `{rel(harness_environment_profile_md, workspace)}`",
            f"- profile json: `{rel(harness_environment_profile_json, workspace)}`",
            f"- status: `{harness_environment_profile.get('status')}` execution_mode: `{harness_environment_profile.get('execution_mode')}`",
            f"- capabilities: `{','.join(harness_environment_profile.get('capabilities') or []) or 'none'}`",
            f"- constraints: `{','.join(harness_environment_profile.get('constraints') or []) or 'none'}`",
            "",
            "harness environment drift:",
            f"- drift artifact: `{rel(harness_environment_drift_md, workspace)}`",
            f"- drift json: `{rel(harness_environment_drift_json, workspace)}`",
            f"- journal: `{rel(environment_profile_journal, workspace)}`",
            f"- status: `{harness_environment_drift.get('status')}` detail: {compact(harness_environment_drift.get('detail'), 260)}",
            f"- history_count: `{harness_environment_drift.get('history_count')}` gained: `{harness_environment_drift.get('gained_count')}` lost: `{harness_environment_drift.get('lost_count')}`",
            "",
            "harness environment drift effectiveness:",
            f"- effectiveness artifact: `{rel(harness_environment_drift_effectiveness_md, workspace)}`",
            f"- effectiveness json: `{rel(harness_environment_drift_effectiveness_json, workspace)}`",
            f"- used: `{harness_environment_drift_effectiveness.get('used_count', 0)}` satisfied: `{harness_environment_drift_effectiveness.get('satisfied_count', 0)}` failed: `{harness_environment_drift_effectiveness.get('failed_count', 0)}` success_rate: `{harness_environment_drift_effectiveness.get('success_rate', '0.00')}`",
            *drift_effectiveness_lines,
            "",
            "repair decision:",
            f"- decision artifact: `{rel(repair_decision_md, workspace)}`",
            f"- decision json: `{rel(repair_decision_json, workspace)}`",
            f"- status: `{repair_decision.get('status')}` decision: `{repair_decision.get('decision')}`",
            f"- focus: {compact(repair_decision.get('focus'), 260)}",
            f"- decision next Need: `{decision_next.get('kind') or 'verify'} {decision_next.get('query') or ''}`",
            "",
            "harness adaptation:",
            f"- adaptation artifact: `{rel(harness_adaptation_md, workspace)}`",
            f"- adaptation json: `{rel(harness_adaptation_json, workspace)}`",
            f"- status: `{harness_adaptation.get('status')}`",
            f"- focus: {compact(harness_adaptation.get('focus'), 260)}",
            f"- adaptation next Need: `{harness_adaptation.get('next_need', {}).get('kind') or 'verify'} {harness_adaptation.get('next_need', {}).get('query') or ''}`",
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
            f"- repair draft effectiveness: `{rel(repair_draft_effectiveness_md, workspace)}`",
            f"- repair draft effectiveness json: `{rel(repair_draft_effectiveness_json, workspace)}`",
            f"- repair draft strategy: `{rel(repair_draft_strategy_md, workspace)}`",
            f"- repair draft strategy json: `{rel(repair_draft_strategy_json, workspace)}`",
            f"- repair patch draft: `{rel(repair_patch_draft_md, workspace)}`",
            f"- repair patch draft json: `{rel(repair_patch_draft_json, workspace)}`",
            f"- repair patch draft effectiveness: `{rel(repair_patch_draft_effectiveness_md, workspace)}`",
            f"- repair patch draft effectiveness json: `{rel(repair_patch_draft_effectiveness_json, workspace)}`",
            f"- repair patch review: `{rel(repair_patch_review_md, workspace)}`",
            f"- repair patch review json: `{rel(repair_patch_review_json, workspace)}`",
            f"- repair patch review effectiveness: `{rel(repair_patch_review_effectiveness_md, workspace)}`",
            f"- repair patch review effectiveness json: `{rel(repair_patch_review_effectiveness_json, workspace)}`",
            f"- repair patch apply: `{rel(repair_patch_apply_md, workspace)}`",
            f"- repair patch apply json: `{rel(repair_patch_apply_json, workspace)}`",
            f"- repair patch verify: `{rel(repair_patch_verify_md, workspace)}`",
            f"- repair patch verify json: `{rel(repair_patch_verify_json, workspace)}`",
            f"- repair patch learning: `{rel(repair_patch_learning_md, workspace)}`",
            f"- repair patch learning json: `{rel(repair_patch_learning_json, workspace)}`",
            f"- repair patch learning effectiveness: `{rel(repair_patch_learning_effectiveness_md, workspace)}`",
            f"- repair patch learning effectiveness json: `{rel(repair_patch_learning_effectiveness_json, workspace)}`",
            f"- repair patch strategy: `{rel(repair_patch_strategy_md, workspace)}`",
            f"- repair patch strategy json: `{rel(repair_patch_strategy_json, workspace)}`",
            f"- repair patch strategy effectiveness: `{rel(repair_patch_strategy_effectiveness_md, workspace)}`",
            f"- repair patch strategy effectiveness json: `{rel(repair_patch_strategy_effectiveness_json, workspace)}`",
            f"- repair command effectiveness: `{rel(repair_command_effectiveness_md, workspace)}`",
            f"- repair command effectiveness json: `{rel(repair_command_effectiveness_json, workspace)}`",
            f"- repair command strategy: `{rel(repair_command_strategy_md, workspace)}`",
            f"- repair command strategy json: `{rel(repair_command_strategy_json, workspace)}`",
            f"- repair command strategy effectiveness: `{rel(repair_command_strategy_effectiveness_md, workspace)}`",
            f"- repair command strategy effectiveness json: `{rel(repair_command_strategy_effectiveness_json, workspace)}`",
            f"- repair decision effectiveness: `{rel(repair_decision_effectiveness_md, workspace)}`",
            f"- repair decision effectiveness json: `{rel(repair_decision_effectiveness_json, workspace)}`",
            f"- repair decision: `{rel(repair_decision_md, workspace)}`",
            f"- repair decision json: `{rel(repair_decision_json, workspace)}`",
            f"- harness adaptation effectiveness: `{rel(harness_adaptation_effectiveness_md, workspace)}`",
            f"- harness adaptation effectiveness json: `{rel(harness_adaptation_effectiveness_json, workspace)}`",
            f"- harness environment profile: `{rel(harness_environment_profile_md, workspace)}`",
            f"- harness environment profile json: `{rel(harness_environment_profile_json, workspace)}`",
            f"- harness environment drift: `{rel(harness_environment_drift_md, workspace)}`",
            f"- harness environment drift json: `{rel(harness_environment_drift_json, workspace)}`",
            f"- harness environment drift effectiveness: `{rel(harness_environment_drift_effectiveness_md, workspace)}`",
            f"- harness environment drift effectiveness json: `{rel(harness_environment_drift_effectiveness_json, workspace)}`",
            f"- environment profile journal: `{rel(environment_profile_journal, workspace)}`",
            f"- harness adaptation: `{rel(harness_adaptation_md, workspace)}`",
            f"- harness adaptation json: `{rel(harness_adaptation_json, workspace)}`",
            f"- adapter context: `{rel(adapter_context_md, workspace)}`",
            f"- code context: `{rel(code_context_md, workspace)}`",
            f"- field trajectory: `{rel(field_trajectory_md, workspace)}`",
            f"- action trace: `{rel(action_trace_md, workspace)}`",
            f"- action trace json: `{rel(action_trace_json, workspace)}`",
            f"- action trace effectiveness: `{rel(action_trace_effectiveness_md, workspace)}`",
            f"- action trace effectiveness json: `{rel(action_trace_effectiveness_json, workspace)}`",
            f"- repair plan: `{rel(repair_plan_json, workspace)}`",
        ]
    )
    + "\n",
    encoding="utf-8",
)
draft = llm_draft(prompt_md.read_text(encoding="utf-8"), session, workspace)
draft_md.write_text(draft_markdown(draft, prompt_md, session_json, workspace), encoding="utf-8")
repair_draft_strategy = build_repair_draft_strategy(
    draft,
    repair_draft_effectiveness,
    code_context["tentacle"],
    code_context["tool"],
)
repair_draft_strategy_json.write_text(
    json.dumps(repair_draft_strategy, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_draft_strategy_md.write_text(
    repair_draft_strategy_markdown(repair_draft_strategy, workspace, repair_draft_strategy_json),
    encoding="utf-8",
)
repair_patch_draft = build_repair_patch_draft(draft, code_context)
repair_patch_draft_json.write_text(
    json.dumps(repair_patch_draft, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_patch_draft_md.write_text(
    repair_patch_draft_markdown(repair_patch_draft, workspace, repair_patch_draft_json),
    encoding="utf-8",
)
repair_patch_review = build_repair_patch_review(workspace, session_dir, repair_patch_draft)
repair_patch_review_json.write_text(
    json.dumps(repair_patch_review, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_patch_review_md.write_text(
    repair_patch_review_markdown(repair_patch_review, workspace, repair_patch_review_json),
    encoding="utf-8",
)
repair_patch_apply = build_repair_patch_apply_plan(workspace, repair_patch_review)
repair_patch_apply_json.write_text(
    json.dumps(repair_patch_apply, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_patch_apply_md.write_text(
    repair_patch_apply_markdown(repair_patch_apply, workspace, repair_patch_apply_json),
    encoding="utf-8",
)
repair_patch_verify = build_repair_patch_verify_plan(workspace, repair_patch_apply)
repair_patch_verify_json.write_text(
    json.dumps(repair_patch_verify, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_patch_verify_md.write_text(
    repair_patch_verify_markdown(repair_patch_verify, workspace, repair_patch_verify_json),
    encoding="utf-8",
)
repair_patch_strategy = build_repair_patch_strategy(
    repair_patch_draft,
    repair_patch_review,
    repair_patch_apply,
    repair_patch_verify,
    repair_patch_learning,
    repair_patch_learning_effectiveness,
    repair_patch_strategy_effectiveness,
    code_context["tentacle"],
    code_context["tool"],
)
repair_patch_strategy_json.write_text(
    json.dumps(repair_patch_strategy, ensure_ascii=True, indent=2) + "\n",
    encoding="utf-8",
)
repair_patch_strategy_md.write_text(
    repair_patch_strategy_markdown(repair_patch_strategy, workspace, repair_patch_strategy_json),
    encoding="utf-8",
)
draft_strategy_next = repair_draft_strategy.get("next_need") if isinstance(repair_draft_strategy.get("next_need"), dict) else {}
session["repair_draft_strategy_summary"] = {
    "status": repair_draft_strategy["status"],
    "focus": repair_draft_strategy["focus"],
    "learned_reuse": repair_draft_strategy["learned_reuse"],
    "learned_avoid": repair_draft_strategy["learned_avoid"],
    "next_need_kind": draft_strategy_next.get("kind") or "",
    "next_need_query": draft_strategy_next.get("query") or "",
}
session["repair_patch_draft_summary"] = {
    "status": repair_patch_draft["status"],
    "draft_status": repair_patch_draft["draft_status"],
    "has_patch": repair_patch_draft["has_patch"],
    "target": repair_patch_draft["target"],
    "next_need": repair_patch_draft["next_need"],
}
session["repair_patch_review_summary"] = {
    "status": repair_patch_review["status"],
    "check_status": repair_patch_review["check_status"],
    "has_patch": repair_patch_review["has_patch"],
    "target": repair_patch_review["target"],
    "summary": repair_patch_review["summary"],
    "next_need": repair_patch_review["next_need"],
}
session["repair_patch_apply_summary"] = {
    "status": repair_patch_apply["status"],
    "applied": repair_patch_apply["applied"],
    "authorized": repair_patch_apply["authorized"],
    "target": repair_patch_apply["target"],
    "patch_file": repair_patch_apply["patch_file"],
    "required_grant": repair_patch_apply["required_grant"],
    "next_need": repair_patch_apply["next_need"],
}
session["repair_patch_verify_summary"] = {
    "status": repair_patch_verify["status"],
    "passed": repair_patch_verify["passed"],
    "target": repair_patch_verify["target"],
    "command": repair_patch_verify["command"],
    "next_need": repair_patch_verify["next_need"],
}
session["repair_patch_strategy_summary"] = {
    "status": repair_patch_strategy["status"],
    "focus": repair_patch_strategy["focus"],
    "learned_reuse": repair_patch_strategy["learned_reuse"],
    "learned_avoid": repair_patch_strategy["learned_avoid"],
    "strategy_learned_reuse": repair_patch_strategy["strategy_learned_reuse"],
    "strategy_learned_avoid": repair_patch_strategy["strategy_learned_avoid"],
    "next_need": repair_patch_strategy["next_need"],
}
repair_plan["repair_patch_draft_summary"] = session["repair_patch_draft_summary"]
repair_plan["repair_patch_review_summary"] = session["repair_patch_review_summary"]
repair_plan["repair_patch_apply_summary"] = session["repair_patch_apply_summary"]
repair_plan["repair_patch_verify_summary"] = session["repair_patch_verify_summary"]
repair_plan["repair_draft_strategy_summary"] = session["repair_draft_strategy_summary"]
repair_plan["repair_patch_strategy_summary"] = session["repair_patch_strategy_summary"]
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
        action_trace_effectiveness,
        repair_draft_effectiveness,
        repair_draft_strategy,
        repair_patch_draft,
        repair_patch_draft_effectiveness,
        repair_patch_review,
        repair_patch_review_effectiveness,
        repair_patch_learning,
        repair_patch_learning_effectiveness,
        repair_patch_strategy,
        repair_patch_strategy_effectiveness,
        repair_command_effectiveness,
        repair_command_strategy,
        repair_command_strategy_effectiveness,
        harness_environment_drift,
        harness_environment_drift_effectiveness,
        repair_decision,
        repair_decision_effectiveness,
        harness_adaptation,
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
            f"- harness adaptation: `{rel(harness_adaptation_md, workspace)}`",
            f"- harness adaptation json: `{rel(harness_adaptation_json, workspace)}`",
            f"- harness adaptation effectiveness: `{rel(harness_adaptation_effectiveness_md, workspace)}`",
            f"- harness adaptation effectiveness json: `{rel(harness_adaptation_effectiveness_json, workspace)}`",
            f"- harness environment profile: `{rel(harness_environment_profile_md, workspace)}`",
            f"- harness environment profile json: `{rel(harness_environment_profile_json, workspace)}`",
            f"- harness environment drift: `{rel(harness_environment_drift_md, workspace)}`",
            f"- harness environment drift json: `{rel(harness_environment_drift_json, workspace)}`",
            f"- harness environment drift effectiveness: `{rel(harness_environment_drift_effectiveness_md, workspace)}`",
            f"- harness environment drift effectiveness json: `{rel(harness_environment_drift_effectiveness_json, workspace)}`",
            f"- adapter context: `{rel(adapter_context_md, workspace)}`",
            f"- action trace: `{rel(action_trace_md, workspace)}`",
            f"- action trace json: `{rel(action_trace_json, workspace)}`",
            f"- action trace effectiveness: `{rel(action_trace_effectiveness_md, workspace)}`",
            f"- action trace effectiveness json: `{rel(action_trace_effectiveness_json, workspace)}`",
            f"- repair recall: `{rel(repair_recall_json, workspace)}`",
            f"- repair lessons: `{rel(repair_lessons_md, workspace)}`",
            f"- repair lessons json: `{rel(repair_lessons_json, workspace)}`",
            f"- repair lesson effectiveness: `{rel(repair_lesson_effectiveness_md, workspace)}`",
            f"- repair lesson effectiveness json: `{rel(repair_lesson_effectiveness_json, workspace)}`",
            f"- repair draft effectiveness: `{rel(repair_draft_effectiveness_md, workspace)}`",
            f"- repair draft effectiveness json: `{rel(repair_draft_effectiveness_json, workspace)}`",
            f"- repair patch draft: `{rel(repair_patch_draft_md, workspace)}`",
            f"- repair patch draft json: `{rel(repair_patch_draft_json, workspace)}`",
            f"- repair patch draft effectiveness: `{rel(repair_patch_draft_effectiveness_md, workspace)}`",
            f"- repair patch draft effectiveness json: `{rel(repair_patch_draft_effectiveness_json, workspace)}`",
            f"- repair patch review: `{rel(repair_patch_review_md, workspace)}`",
            f"- repair patch review json: `{rel(repair_patch_review_json, workspace)}`",
            f"- repair patch review effectiveness: `{rel(repair_patch_review_effectiveness_md, workspace)}`",
            f"- repair patch review effectiveness json: `{rel(repair_patch_review_effectiveness_json, workspace)}`",
            f"- repair patch apply: `{rel(repair_patch_apply_md, workspace)}`",
            f"- repair patch apply json: `{rel(repair_patch_apply_json, workspace)}`",
            f"- repair patch verify: `{rel(repair_patch_verify_md, workspace)}`",
            f"- repair patch verify json: `{rel(repair_patch_verify_json, workspace)}`",
            f"- repair patch learning: `{rel(repair_patch_learning_md, workspace)}`",
            f"- repair patch learning json: `{rel(repair_patch_learning_json, workspace)}`",
            f"- repair patch learning effectiveness: `{rel(repair_patch_learning_effectiveness_md, workspace)}`",
            f"- repair patch learning effectiveness json: `{rel(repair_patch_learning_effectiveness_json, workspace)}`",
            f"- repair patch strategy: `{rel(repair_patch_strategy_md, workspace)}`",
            f"- repair patch strategy json: `{rel(repair_patch_strategy_json, workspace)}`",
            f"- repair patch strategy effectiveness: `{rel(repair_patch_strategy_effectiveness_md, workspace)}`",
            f"- repair patch strategy effectiveness json: `{rel(repair_patch_strategy_effectiveness_json, workspace)}`",
            f"- repair command effectiveness: `{rel(repair_command_effectiveness_md, workspace)}`",
            f"- repair command effectiveness json: `{rel(repair_command_effectiveness_json, workspace)}`",
            f"- repair command strategy: `{rel(repair_command_strategy_md, workspace)}`",
            f"- repair command strategy json: `{rel(repair_command_strategy_json, workspace)}`",
            f"- repair command strategy effectiveness: `{rel(repair_command_strategy_effectiveness_md, workspace)}`",
            f"- repair command strategy effectiveness json: `{rel(repair_command_strategy_effectiveness_json, workspace)}`",
            f"- repair decision effectiveness: `{rel(repair_decision_effectiveness_md, workspace)}`",
            f"- repair decision effectiveness json: `{rel(repair_decision_effectiveness_json, workspace)}`",
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
            "score options:",
            *[f"- `{command}`" for command in repair_plan["commands"].get("score_options", [])],
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
            f"repair draft effectiveness: `{rel(repair_draft_effectiveness_md, workspace)}`",
            f"repair draft effectiveness json: `{rel(repair_draft_effectiveness_json, workspace)}`",
            f"repair draft strategy: `{rel(repair_draft_strategy_md, workspace)}`",
            f"repair draft strategy json: `{rel(repair_draft_strategy_json, workspace)}`",
            f"repair patch draft: `{rel(repair_patch_draft_md, workspace)}`",
            f"repair patch draft json: `{rel(repair_patch_draft_json, workspace)}`",
            f"repair patch draft effectiveness: `{rel(repair_patch_draft_effectiveness_md, workspace)}`",
            f"repair patch draft effectiveness json: `{rel(repair_patch_draft_effectiveness_json, workspace)}`",
            f"repair patch review: `{rel(repair_patch_review_md, workspace)}`",
            f"repair patch review json: `{rel(repair_patch_review_json, workspace)}`",
            f"repair patch review effectiveness: `{rel(repair_patch_review_effectiveness_md, workspace)}`",
            f"repair patch review effectiveness json: `{rel(repair_patch_review_effectiveness_json, workspace)}`",
            f"repair patch apply: `{rel(repair_patch_apply_md, workspace)}`",
            f"repair patch apply json: `{rel(repair_patch_apply_json, workspace)}`",
            f"repair patch verify: `{rel(repair_patch_verify_md, workspace)}`",
            f"repair patch verify json: `{rel(repair_patch_verify_json, workspace)}`",
            f"repair patch learning: `{rel(repair_patch_learning_md, workspace)}`",
            f"repair patch learning json: `{rel(repair_patch_learning_json, workspace)}`",
            f"repair patch learning effectiveness: `{rel(repair_patch_learning_effectiveness_md, workspace)}`",
            f"repair patch learning effectiveness json: `{rel(repair_patch_learning_effectiveness_json, workspace)}`",
            f"repair patch strategy: `{rel(repair_patch_strategy_md, workspace)}`",
            f"repair patch strategy json: `{rel(repair_patch_strategy_json, workspace)}`",
            f"repair patch strategy effectiveness: `{rel(repair_patch_strategy_effectiveness_md, workspace)}`",
            f"repair patch strategy effectiveness json: `{rel(repair_patch_strategy_effectiveness_json, workspace)}`",
            f"repair command effectiveness: `{rel(repair_command_effectiveness_md, workspace)}`",
            f"repair command effectiveness json: `{rel(repair_command_effectiveness_json, workspace)}`",
            f"repair command strategy: `{rel(repair_command_strategy_md, workspace)}`",
            f"repair command strategy json: `{rel(repair_command_strategy_json, workspace)}`",
            f"repair command strategy effectiveness: `{rel(repair_command_strategy_effectiveness_md, workspace)}`",
            f"repair command strategy effectiveness json: `{rel(repair_command_strategy_effectiveness_json, workspace)}`",
            f"repair decision: `{rel(repair_decision_md, workspace)}`",
            f"repair decision json: `{rel(repair_decision_json, workspace)}`",
            f"harness adaptation effectiveness: `{rel(harness_adaptation_effectiveness_md, workspace)}`",
            f"harness adaptation effectiveness json: `{rel(harness_adaptation_effectiveness_json, workspace)}`",
            f"harness environment profile: `{rel(harness_environment_profile_md, workspace)}`",
            f"harness environment profile json: `{rel(harness_environment_profile_json, workspace)}`",
            f"harness environment drift: `{rel(harness_environment_drift_md, workspace)}`",
            f"harness environment drift json: `{rel(harness_environment_drift_json, workspace)}`",
            f"harness environment drift effectiveness: `{rel(harness_environment_drift_effectiveness_md, workspace)}`",
            f"harness environment drift effectiveness json: `{rel(harness_environment_drift_effectiveness_json, workspace)}`",
            f"environment profile journal: `{rel(environment_profile_journal, workspace)}`",
            f"harness adaptation: `{rel(harness_adaptation_md, workspace)}`",
            f"harness adaptation json: `{rel(harness_adaptation_json, workspace)}`",
            f"adapter context: `{rel(adapter_context_md, workspace)}`",
            f"code context: `{rel(code_context_md, workspace)}`",
            f"field trajectory: `{rel(field_trajectory_md, workspace)}`",
            f"action trace: `{rel(action_trace_md, workspace)}`",
            f"action trace json: `{rel(action_trace_json, workspace)}`",
            f"action trace effectiveness: `{rel(action_trace_effectiveness_md, workspace)}`",
            f"action trace effectiveness json: `{rel(action_trace_effectiveness_json, workspace)}`",
            f"repair plan: `{rel(repair_plan_json, workspace)}`",
            f"outcome command: `{outcome_command}`",
            "outcome options:",
            *[f"- `{command}`" for command in outcome_commands],
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
    "llm_model": draft.get("model", ""),
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
    "action_trace_effectiveness": rel(action_trace_effectiveness_md, workspace),
    "action_trace_effectiveness_json": rel(action_trace_effectiveness_json, workspace),
    "action_trace_effectiveness_used_count": str(action_trace_effectiveness.get("used_count", 0)),
    "action_trace_effectiveness_satisfied_count": str(action_trace_effectiveness.get("satisfied_count", 0)),
    "action_trace_effectiveness_partial_count": str(action_trace_effectiveness.get("partial_count", 0)),
    "action_trace_effectiveness_failed_count": str(action_trace_effectiveness.get("failed_count", 0)),
    "action_trace_effectiveness_success_rate": action_trace_effectiveness.get("success_rate", "0.00"),
    "action_trace_effectiveness_failure_rate": action_trace_effectiveness.get("failure_rate", "0.00"),
    "action_trace_effectiveness_top_reuse": action_trace_effectiveness.get("top_reuse", ""),
    "action_trace_effectiveness_top_avoid": action_trace_effectiveness.get("top_avoid", ""),
    "repair_draft_effectiveness": rel(repair_draft_effectiveness_md, workspace),
    "repair_draft_effectiveness_json": rel(repair_draft_effectiveness_json, workspace),
    "repair_draft_effectiveness_used_count": str(repair_draft_effectiveness.get("used_count", 0)),
    "repair_draft_effectiveness_satisfied_count": str(repair_draft_effectiveness.get("satisfied_count", 0)),
    "repair_draft_effectiveness_partial_count": str(repair_draft_effectiveness.get("partial_count", 0)),
    "repair_draft_effectiveness_failed_count": str(repair_draft_effectiveness.get("failed_count", 0)),
    "repair_draft_effectiveness_success_rate": repair_draft_effectiveness.get("success_rate", "0.00"),
    "repair_draft_effectiveness_failure_rate": repair_draft_effectiveness.get("failure_rate", "0.00"),
    "repair_draft_effectiveness_top_reuse": repair_draft_effectiveness.get("top_reuse", ""),
    "repair_draft_effectiveness_top_avoid": repair_draft_effectiveness.get("top_avoid", ""),
    "repair_draft_strategy": rel(repair_draft_strategy_md, workspace),
    "repair_draft_strategy_json": rel(repair_draft_strategy_json, workspace),
    "repair_draft_strategy_status": repair_draft_strategy.get("status", ""),
    "repair_draft_strategy_focus": repair_draft_strategy.get("focus", ""),
    "repair_draft_strategy_learned_reuse": repair_draft_strategy.get("learned_reuse", ""),
    "repair_draft_strategy_learned_avoid": repair_draft_strategy.get("learned_avoid", ""),
    "repair_draft_strategy_next_need_kind": repair_draft_strategy.get("next_need", {}).get("kind", ""),
    "repair_draft_strategy_next_need_query": repair_draft_strategy.get("next_need", {}).get("query", ""),
    "repair_patch_draft": rel(repair_patch_draft_md, workspace),
    "repair_patch_draft_json": rel(repair_patch_draft_json, workspace),
    "repair_patch_draft_status": repair_patch_draft.get("status", ""),
    "repair_patch_draft_draft_status": repair_patch_draft.get("draft_status", ""),
    "repair_patch_draft_has_patch": "true" if repair_patch_draft.get("has_patch") else "false",
    "repair_patch_draft_target": "/".join(
        item
        for item in [
            repair_patch_draft.get("target", {}).get("tentacle", ""),
            repair_patch_draft.get("target", {}).get("tool", ""),
        ]
        if item
    ),
    "repair_patch_draft_preview": compact(
        repair_patch_draft.get("patch_preview")
        or repair_patch_draft.get("proposal_preview")
        or "",
        700,
    ),
    "repair_patch_draft_effectiveness": rel(repair_patch_draft_effectiveness_md, workspace),
    "repair_patch_draft_effectiveness_json": rel(repair_patch_draft_effectiveness_json, workspace),
    "repair_patch_draft_effectiveness_used_count": str(repair_patch_draft_effectiveness.get("used_count", 0)),
    "repair_patch_draft_effectiveness_satisfied_count": str(repair_patch_draft_effectiveness.get("satisfied_count", 0)),
    "repair_patch_draft_effectiveness_partial_count": str(repair_patch_draft_effectiveness.get("partial_count", 0)),
    "repair_patch_draft_effectiveness_failed_count": str(repair_patch_draft_effectiveness.get("failed_count", 0)),
    "repair_patch_draft_effectiveness_success_rate": repair_patch_draft_effectiveness.get("success_rate", "0.00"),
    "repair_patch_draft_effectiveness_failure_rate": repair_patch_draft_effectiveness.get("failure_rate", "0.00"),
    "repair_patch_draft_effectiveness_top_reuse": repair_patch_draft_effectiveness.get("top_reuse", ""),
    "repair_patch_draft_effectiveness_top_avoid": repair_patch_draft_effectiveness.get("top_avoid", ""),
    "repair_patch_review": rel(repair_patch_review_md, workspace),
    "repair_patch_review_json": rel(repair_patch_review_json, workspace),
    "repair_patch_review_status": repair_patch_review.get("status", ""),
    "repair_patch_review_check_status": repair_patch_review.get("check_status", ""),
    "repair_patch_review_has_patch": "true" if repair_patch_review.get("has_patch") else "false",
    "repair_patch_review_target": repair_patch_review.get("target_label", ""),
    "repair_patch_review_summary": repair_patch_review.get("summary", ""),
    "repair_patch_review_patch_file": repair_patch_review.get("patch_file", ""),
    "repair_patch_review_effectiveness": rel(repair_patch_review_effectiveness_md, workspace),
    "repair_patch_review_effectiveness_json": rel(repair_patch_review_effectiveness_json, workspace),
    "repair_patch_review_effectiveness_used_count": str(repair_patch_review_effectiveness.get("used_count", 0)),
    "repair_patch_review_effectiveness_satisfied_count": str(repair_patch_review_effectiveness.get("satisfied_count", 0)),
    "repair_patch_review_effectiveness_partial_count": str(repair_patch_review_effectiveness.get("partial_count", 0)),
    "repair_patch_review_effectiveness_failed_count": str(repair_patch_review_effectiveness.get("failed_count", 0)),
    "repair_patch_review_effectiveness_success_rate": repair_patch_review_effectiveness.get("success_rate", "0.00"),
    "repair_patch_review_effectiveness_failure_rate": repair_patch_review_effectiveness.get("failure_rate", "0.00"),
    "repair_patch_review_effectiveness_top_reuse": repair_patch_review_effectiveness.get("top_reuse", ""),
    "repair_patch_review_effectiveness_top_avoid": repair_patch_review_effectiveness.get("top_avoid", ""),
    "repair_patch_apply": rel(repair_patch_apply_md, workspace),
    "repair_patch_apply_json": rel(repair_patch_apply_json, workspace),
    "repair_patch_apply_status": repair_patch_apply.get("status", ""),
    "repair_patch_apply_applied": "true" if repair_patch_apply.get("applied") else "false",
    "repair_patch_apply_authorized": "true" if repair_patch_apply.get("authorized") else "false",
    "repair_patch_apply_target": repair_patch_apply.get("target_label", ""),
    "repair_patch_apply_patch_file": repair_patch_apply.get("patch_file", ""),
    "repair_patch_apply_required_grant": repair_patch_apply.get("required_grant", ""),
    "repair_patch_apply_grant_command": repair_patch_apply.get("grant_command", ""),
    "repair_patch_apply_apply_command": repair_patch_apply.get("apply_command", ""),
    "repair_patch_apply_summary": repair_patch_apply.get("summary", ""),
    "repair_patch_verify": rel(repair_patch_verify_md, workspace),
    "repair_patch_verify_json": rel(repair_patch_verify_json, workspace),
    "repair_patch_verify_status": repair_patch_verify.get("status", ""),
    "repair_patch_verify_passed": "true" if repair_patch_verify.get("passed") else "false",
    "repair_patch_verify_target": repair_patch_verify.get("target_label", ""),
    "repair_patch_verify_command": repair_patch_verify.get("command", ""),
    "repair_patch_verify_summary": repair_patch_verify.get("summary", ""),
    "repair_patch_learning": rel(repair_patch_learning_md, workspace),
    "repair_patch_learning_json": rel(repair_patch_learning_json, workspace),
    "repair_patch_learning_status": repair_patch_learning.get("status", ""),
    "repair_patch_learning_used_count": str(repair_patch_learning.get("used_count", 0)),
    "repair_patch_learning_verified_count": str(repair_patch_learning.get("verified_count", 0)),
    "repair_patch_learning_verified_satisfied_count": str(repair_patch_learning.get("verified_satisfied_count", 0)),
    "repair_patch_learning_verified_failed_count": str(repair_patch_learning.get("verified_failed_count", 0)),
    "repair_patch_learning_success_rate": repair_patch_learning.get("success_rate", "0.00"),
    "repair_patch_learning_verified_success_rate": repair_patch_learning.get("verified_success_rate", "0.00"),
    "repair_patch_learning_top_reuse": repair_patch_learning.get("top_reuse", ""),
    "repair_patch_learning_top_avoid": repair_patch_learning.get("top_avoid", ""),
    "repair_patch_learning_next_need_kind": repair_patch_learning.get("next_need", {}).get("kind", ""),
    "repair_patch_learning_next_need_query": repair_patch_learning.get("next_need", {}).get("query", ""),
    "repair_patch_learning_effectiveness": rel(repair_patch_learning_effectiveness_md, workspace),
    "repair_patch_learning_effectiveness_json": rel(repair_patch_learning_effectiveness_json, workspace),
    "repair_patch_learning_effectiveness_used_count": str(repair_patch_learning_effectiveness.get("used_count", 0)),
    "repair_patch_learning_effectiveness_satisfied_count": str(repair_patch_learning_effectiveness.get("satisfied_count", 0)),
    "repair_patch_learning_effectiveness_partial_count": str(repair_patch_learning_effectiveness.get("partial_count", 0)),
    "repair_patch_learning_effectiveness_failed_count": str(repair_patch_learning_effectiveness.get("failed_count", 0)),
    "repair_patch_learning_effectiveness_success_rate": repair_patch_learning_effectiveness.get("success_rate", "0.00"),
    "repair_patch_learning_effectiveness_failure_rate": repair_patch_learning_effectiveness.get("failure_rate", "0.00"),
    "repair_patch_learning_effectiveness_top_reuse": repair_patch_learning_effectiveness.get("top_reuse", ""),
    "repair_patch_learning_effectiveness_top_avoid": repair_patch_learning_effectiveness.get("top_avoid", ""),
    "repair_patch_strategy": rel(repair_patch_strategy_md, workspace),
    "repair_patch_strategy_json": rel(repair_patch_strategy_json, workspace),
    "repair_patch_strategy_status": repair_patch_strategy.get("status", ""),
    "repair_patch_strategy_focus": repair_patch_strategy.get("focus", ""),
    "repair_patch_strategy_learned_reuse": repair_patch_strategy.get("learned_reuse", ""),
    "repair_patch_strategy_learned_avoid": repair_patch_strategy.get("learned_avoid", ""),
    "repair_patch_strategy_next_need_kind": repair_patch_strategy.get("next_need", {}).get("kind", ""),
    "repair_patch_strategy_next_need_query": repair_patch_strategy.get("next_need", {}).get("query", ""),
    "repair_patch_strategy_effectiveness": rel(repair_patch_strategy_effectiveness_md, workspace),
    "repair_patch_strategy_effectiveness_json": rel(repair_patch_strategy_effectiveness_json, workspace),
    "repair_patch_strategy_effectiveness_used_count": str(repair_patch_strategy_effectiveness.get("used_count", 0)),
    "repair_patch_strategy_effectiveness_satisfied_count": str(repair_patch_strategy_effectiveness.get("satisfied_count", 0)),
    "repair_patch_strategy_effectiveness_partial_count": str(repair_patch_strategy_effectiveness.get("partial_count", 0)),
    "repair_patch_strategy_effectiveness_failed_count": str(repair_patch_strategy_effectiveness.get("failed_count", 0)),
    "repair_patch_strategy_effectiveness_success_rate": repair_patch_strategy_effectiveness.get("success_rate", "0.00"),
    "repair_patch_strategy_effectiveness_failure_rate": repair_patch_strategy_effectiveness.get("failure_rate", "0.00"),
    "repair_patch_strategy_effectiveness_top_reuse": repair_patch_strategy_effectiveness.get("top_reuse", ""),
    "repair_patch_strategy_effectiveness_top_avoid": repair_patch_strategy_effectiveness.get("top_avoid", ""),
    "repair_command_effectiveness": rel(repair_command_effectiveness_md, workspace),
    "repair_command_effectiveness_json": rel(repair_command_effectiveness_json, workspace),
    "repair_command_effectiveness_used_count": str(repair_command_effectiveness.get("used_count", 0)),
    "repair_command_effectiveness_satisfied_count": str(repair_command_effectiveness.get("satisfied_count", 0)),
    "repair_command_effectiveness_partial_count": str(repair_command_effectiveness.get("partial_count", 0)),
    "repair_command_effectiveness_failed_count": str(repair_command_effectiveness.get("failed_count", 0)),
    "repair_command_effectiveness_success_rate": repair_command_effectiveness.get("success_rate", "0.00"),
    "repair_command_effectiveness_failure_rate": repair_command_effectiveness.get("failure_rate", "0.00"),
    "repair_command_effectiveness_top_reuse": repair_command_effectiveness.get("top_reuse", ""),
    "repair_command_effectiveness_top_avoid": repair_command_effectiveness.get("top_avoid", ""),
    "repair_command_strategy": rel(repair_command_strategy_md, workspace),
    "repair_command_strategy_json": rel(repair_command_strategy_json, workspace),
    "repair_command_strategy_status": repair_command_strategy.get("status", ""),
    "repair_command_strategy_focus": repair_command_strategy.get("focus", ""),
    "repair_command_strategy_learned_reuse": repair_command_strategy.get("learned_reuse", ""),
    "repair_command_strategy_learned_avoid": repair_command_strategy.get("learned_avoid", ""),
    "repair_command_strategy_current_apply": repair_command_strategy.get("current_recipe", {}).get("apply", ""),
    "repair_command_strategy_next_need_kind": repair_command_strategy.get("next_need", {}).get("kind", ""),
    "repair_command_strategy_next_need_query": repair_command_strategy.get("next_need", {}).get("query", ""),
    "repair_command_strategy_effectiveness": rel(repair_command_strategy_effectiveness_md, workspace),
    "repair_command_strategy_effectiveness_json": rel(repair_command_strategy_effectiveness_json, workspace),
    "repair_command_strategy_effectiveness_used_count": str(repair_command_strategy_effectiveness.get("used_count", 0)),
    "repair_command_strategy_effectiveness_satisfied_count": str(repair_command_strategy_effectiveness.get("satisfied_count", 0)),
    "repair_command_strategy_effectiveness_partial_count": str(repair_command_strategy_effectiveness.get("partial_count", 0)),
    "repair_command_strategy_effectiveness_failed_count": str(repair_command_strategy_effectiveness.get("failed_count", 0)),
    "repair_command_strategy_effectiveness_success_rate": repair_command_strategy_effectiveness.get("success_rate", "0.00"),
    "repair_command_strategy_effectiveness_failure_rate": repair_command_strategy_effectiveness.get("failure_rate", "0.00"),
    "repair_command_strategy_effectiveness_top_reuse": repair_command_strategy_effectiveness.get("top_reuse", ""),
    "repair_command_strategy_effectiveness_top_avoid": repair_command_strategy_effectiveness.get("top_avoid", ""),
    "repair_decision": rel(repair_decision_md, workspace),
    "repair_decision_json": rel(repair_decision_json, workspace),
    "repair_decision_status": repair_decision.get("status", ""),
    "repair_decision_kind": repair_decision.get("decision", ""),
    "repair_decision_focus": repair_decision.get("focus", ""),
    "repair_decision_next_need_kind": repair_decision.get("next_need", {}).get("kind", ""),
    "repair_decision_next_need_query": repair_decision.get("next_need", {}).get("query", ""),
    "harness_adaptation_effectiveness": rel(harness_adaptation_effectiveness_md, workspace),
    "harness_adaptation_effectiveness_json": rel(harness_adaptation_effectiveness_json, workspace),
    "harness_adaptation_effectiveness_used_count": str(harness_adaptation_effectiveness.get("used_count", 0)),
    "harness_adaptation_effectiveness_satisfied_count": str(harness_adaptation_effectiveness.get("satisfied_count", 0)),
    "harness_adaptation_effectiveness_partial_count": str(harness_adaptation_effectiveness.get("partial_count", 0)),
    "harness_adaptation_effectiveness_failed_count": str(harness_adaptation_effectiveness.get("failed_count", 0)),
    "harness_adaptation_effectiveness_success_rate": harness_adaptation_effectiveness.get("success_rate", "0.00"),
    "harness_adaptation_effectiveness_failure_rate": harness_adaptation_effectiveness.get("failure_rate", "0.00"),
    "harness_adaptation_effectiveness_top_reuse": harness_adaptation_effectiveness.get("top_reuse", ""),
    "harness_adaptation_effectiveness_top_avoid": harness_adaptation_effectiveness.get("top_avoid", ""),
    "harness_environment_profile": rel(harness_environment_profile_md, workspace),
    "harness_environment_profile_json": rel(harness_environment_profile_json, workspace),
    "harness_environment_profile_status": harness_environment_profile.get("status", ""),
    "harness_environment_profile_id": harness_environment_profile.get("profile_id", ""),
    "harness_environment_profile_execution_mode": harness_environment_profile.get("execution_mode", ""),
    "harness_environment_profile_provider_mode": harness_environment_profile.get("provider_mode", ""),
    "harness_environment_profile_desktop_mode": harness_environment_profile.get("desktop_mode", ""),
    "harness_environment_profile_capabilities": ",".join(harness_environment_profile.get("capabilities") or []),
    "harness_environment_profile_constraints": ",".join(harness_environment_profile.get("constraints") or []),
    "harness_environment_profile_installed_profiles": ",".join(harness_environment_profile.get("installed_profiles") or []),
    "harness_environment_profile_next_need_kind": harness_environment_profile.get("next_need", {}).get("kind", ""),
    "harness_environment_profile_next_need_query": harness_environment_profile.get("next_need", {}).get("query", ""),
    "harness_environment_drift": rel(harness_environment_drift_md, workspace),
    "harness_environment_drift_json": rel(harness_environment_drift_json, workspace),
    "harness_environment_drift_status": harness_environment_drift.get("status", ""),
    "harness_environment_drift_summary": harness_environment_drift.get("summary", ""),
    "harness_environment_drift_detail": harness_environment_drift.get("detail", ""),
    "harness_environment_drift_history_count": str(harness_environment_drift.get("history_count", 0)),
    "harness_environment_drift_gained_count": str(harness_environment_drift.get("gained_count", 0)),
    "harness_environment_drift_lost_count": str(harness_environment_drift.get("lost_count", 0)),
    "harness_environment_drift_next_need_kind": harness_environment_drift.get("next_need", {}).get("kind", ""),
    "harness_environment_drift_next_need_query": harness_environment_drift.get("next_need", {}).get("query", ""),
    "harness_environment_drift_effectiveness": rel(harness_environment_drift_effectiveness_md, workspace),
    "harness_environment_drift_effectiveness_json": rel(harness_environment_drift_effectiveness_json, workspace),
    "harness_environment_drift_effectiveness_used_count": str(harness_environment_drift_effectiveness.get("used_count", 0)),
    "harness_environment_drift_effectiveness_satisfied_count": str(harness_environment_drift_effectiveness.get("satisfied_count", 0)),
    "harness_environment_drift_effectiveness_partial_count": str(harness_environment_drift_effectiveness.get("partial_count", 0)),
    "harness_environment_drift_effectiveness_failed_count": str(harness_environment_drift_effectiveness.get("failed_count", 0)),
    "harness_environment_drift_effectiveness_success_rate": harness_environment_drift_effectiveness.get("success_rate", "0.00"),
    "harness_environment_drift_effectiveness_failure_rate": harness_environment_drift_effectiveness.get("failure_rate", "0.00"),
    "harness_environment_drift_effectiveness_top_reuse": harness_environment_drift_effectiveness.get("top_reuse", ""),
    "harness_environment_drift_effectiveness_top_avoid": harness_environment_drift_effectiveness.get("top_avoid", ""),
    "environment_profile_journal": rel(environment_profile_journal, workspace),
    "harness_adaptation": rel(harness_adaptation_md, workspace),
    "harness_adaptation_json": rel(harness_adaptation_json, workspace),
    "harness_adaptation_status": harness_adaptation.get("status", ""),
    "harness_adaptation_focus": harness_adaptation.get("focus", ""),
    "harness_adaptation_next_need_kind": harness_adaptation.get("next_need", {}).get("kind", ""),
    "harness_adaptation_next_need_query": harness_adaptation.get("next_need", {}).get("query", ""),
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
    "action_trace_repair_decision_effectiveness_used_count": action_trace["repair_decision_effectiveness"]["used_count"],
    "action_trace_repair_decision_effectiveness_success_rate": action_trace["repair_decision_effectiveness"]["success_rate"],
    "action_trace_repair_patch_draft_status": action_trace["repair_patch_draft"]["status"],
    "action_trace_repair_patch_draft_has_patch": action_trace["repair_patch_draft"]["has_patch"],
    "action_trace_repair_patch_draft_target": action_trace["repair_patch_draft"]["target"],
    "action_trace_repair_patch_draft_preview": action_trace["repair_patch_draft"]["preview"],
    "action_trace_repair_patch_draft_effectiveness_used_count": action_trace["repair_patch_draft_effectiveness"]["used_count"],
    "action_trace_repair_patch_draft_effectiveness_success_rate": action_trace["repair_patch_draft_effectiveness"]["success_rate"],
    "action_trace_repair_patch_review_status": action_trace["repair_patch_review"]["status"],
    "action_trace_repair_patch_review_check_status": action_trace["repair_patch_review"]["check_status"],
    "action_trace_repair_patch_review_has_patch": action_trace["repair_patch_review"]["has_patch"],
    "action_trace_repair_patch_review_target": action_trace["repair_patch_review"]["target"],
    "action_trace_repair_patch_review_summary": action_trace["repair_patch_review"]["summary"],
    "action_trace_repair_patch_review_effectiveness_used_count": action_trace["repair_patch_review_effectiveness"]["used_count"],
    "action_trace_repair_patch_review_effectiveness_success_rate": action_trace["repair_patch_review_effectiveness"]["success_rate"],
    "action_trace_repair_patch_learning_status": action_trace["repair_patch_learning"]["status"],
    "action_trace_repair_patch_learning_used_count": action_trace["repair_patch_learning"]["used_count"],
    "action_trace_repair_patch_learning_verified_count": action_trace["repair_patch_learning"]["verified_count"],
    "action_trace_repair_patch_learning_verified_success_rate": action_trace["repair_patch_learning"]["verified_success_rate"],
    "action_trace_repair_patch_learning_top_reuse": action_trace["repair_patch_learning"]["top_reuse"],
    "action_trace_repair_patch_learning_top_avoid": action_trace["repair_patch_learning"]["top_avoid"],
    "action_trace_repair_patch_learning_next_need_kind": action_trace["repair_patch_learning"]["next_need_kind"],
    "action_trace_repair_patch_learning_next_need_query": action_trace["repair_patch_learning"]["next_need_query"],
    "action_trace_repair_patch_learning_effectiveness_used_count": action_trace["repair_patch_learning_effectiveness"]["used_count"],
    "action_trace_repair_patch_learning_effectiveness_success_rate": action_trace["repair_patch_learning_effectiveness"]["success_rate"],
    "action_trace_repair_draft_strategy_status": action_trace["repair_draft_strategy"]["status"],
    "action_trace_repair_draft_strategy_focus": action_trace["repair_draft_strategy"]["focus"],
    "action_trace_repair_draft_strategy_next_need_kind": action_trace["repair_draft_strategy"]["next_need_kind"],
    "action_trace_repair_draft_strategy_next_need_query": action_trace["repair_draft_strategy"]["next_need_query"],
    "action_trace_repair_patch_strategy_status": action_trace["repair_patch_strategy"]["status"],
    "action_trace_repair_patch_strategy_focus": action_trace["repair_patch_strategy"]["focus"],
    "action_trace_repair_patch_strategy_learned_reuse": action_trace["repair_patch_strategy"]["learned_reuse"],
    "action_trace_repair_patch_strategy_learned_avoid": action_trace["repair_patch_strategy"]["learned_avoid"],
    "action_trace_repair_patch_strategy_next_need_kind": action_trace["repair_patch_strategy"]["next_need_kind"],
    "action_trace_repair_patch_strategy_next_need_query": action_trace["repair_patch_strategy"]["next_need_query"],
    "action_trace_repair_patch_strategy_effectiveness_used_count": action_trace["repair_patch_strategy_effectiveness"]["used_count"],
    "action_trace_repair_patch_strategy_effectiveness_success_rate": action_trace["repair_patch_strategy_effectiveness"]["success_rate"],
    "action_trace_harness_environment_drift_status": action_trace["harness_environment_drift"]["status"],
    "action_trace_harness_environment_drift_detail": action_trace["harness_environment_drift"]["detail"],
    "action_trace_harness_environment_drift_history_count": action_trace["harness_environment_drift"]["history_count"],
    "action_trace_harness_environment_drift_gained_count": action_trace["harness_environment_drift"]["gained_count"],
    "action_trace_harness_environment_drift_lost_count": action_trace["harness_environment_drift"]["lost_count"],
    "action_trace_harness_environment_drift_next_need_kind": action_trace["harness_environment_drift"]["next_need_kind"],
    "action_trace_harness_environment_drift_next_need_query": action_trace["harness_environment_drift"]["next_need_query"],
    "action_trace_harness_adaptation": action_trace["harness_adaptation"]["status"],
    "action_trace_harness_adaptation_focus": action_trace["harness_adaptation"]["focus"],
    "repair_decision_effectiveness": rel(repair_decision_effectiveness_md, workspace),
    "repair_decision_effectiveness_json": rel(repair_decision_effectiveness_json, workspace),
    "repair_decision_effectiveness_used_count": str(repair_decision_effectiveness.get("used_count", 0)),
    "repair_decision_effectiveness_satisfied_count": str(repair_decision_effectiveness.get("satisfied_count", 0)),
    "repair_decision_effectiveness_partial_count": str(repair_decision_effectiveness.get("partial_count", 0)),
    "repair_decision_effectiveness_failed_count": str(repair_decision_effectiveness.get("failed_count", 0)),
    "repair_decision_effectiveness_success_rate": repair_decision_effectiveness.get("success_rate", "0.00"),
    "repair_decision_effectiveness_failure_rate": repair_decision_effectiveness.get("failure_rate", "0.00"),
    "repair_decision_effectiveness_top_reuse": repair_decision_effectiveness.get("top_reuse", ""),
    "repair_decision_effectiveness_top_avoid": repair_decision_effectiveness.get("top_avoid", ""),
    "repair_plan_status": repair_plan["status"],
    "check_command": "; ".join(repair_plan.get("commands", {}).get("checks") or []),
    "grant_command": repair_plan.get("commands", {}).get("grant", ""),
    "apply_command": repair_plan.get("commands", {}).get("apply", ""),
    "score_command": repair_plan.get("commands", {}).get("score", ""),
    "score_commands": " || ".join(repair_plan.get("commands", {}).get("score_options") or []),
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
