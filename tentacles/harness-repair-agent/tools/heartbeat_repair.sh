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


def repair_recall_metadata(root, value):
    path = resolve_artifact(root, value)
    metadata = {
        "repair_recall_match_count": "",
        "repair_recall_top_score": "",
        "repair_recall_top_status": "",
        "repair_recall_top_target": "",
        "repair_recall_top_candidate": "",
        "repair_recall_top_reasons": "",
        "repair_recall_top_summary": "",
        "repair_recall_top_action_status": "",
        "repair_recall_top_action_hint": "",
        "repair_recall_preview": "",
    }
    if not path or not path.exists():
        return metadata
    data = load_json(path)
    matches = data.get("matches") if isinstance(data.get("matches"), list) else []
    metadata["repair_recall_match_count"] = str(data.get("match_count") or len(matches))
    metadata["repair_recall_preview"] = compact(json.dumps(data, sort_keys=True), 700)
    if not matches:
        return metadata
    top = matches[0] if isinstance(matches[0], dict) else {}
    outcome = top.get("outcome") if isinstance(top.get("outcome"), dict) else {}
    reasons = top.get("reasons") if isinstance(top.get("reasons"), list) else []
    metadata.update({
        "repair_recall_top_score": str(top.get("score") or ""),
        "repair_recall_top_status": str(outcome.get("outcome_status") or ""),
        "repair_recall_top_target": str(outcome.get("target_tentacle") or ""),
        "repair_recall_top_candidate": str(outcome.get("candidate") or ""),
        "repair_recall_top_reasons": compact(", ".join(str(reason) for reason in reasons), 260),
        "repair_recall_top_summary": compact(outcome.get("summary") or "", 320),
        "repair_recall_top_action_status": str(outcome.get("action_trace_status") or ""),
        "repair_recall_top_action_hint": compact(outcome.get("action_trace_repair_hint") or "", 240),
    })
    return metadata


def repair_lessons_metadata(root, value, json_value=""):
    path = resolve_artifact(root, value)
    json_path = resolve_artifact(root, json_value)
    if not json_path and path:
        candidate = path.with_suffix(".json")
        if candidate.exists():
            json_path = candidate
    metadata = {
        "repair_lessons": rel(path, root) if path else "",
        "repair_lessons_json": rel(json_path, root) if json_path else "",
        "repair_lessons_count": "",
        "repair_lessons_reuse_count": "",
        "repair_lessons_avoid_count": "",
        "repair_lessons_top_reuse": "",
        "repair_lessons_top_avoid": "",
        "repair_lessons_preview": "",
    }
    if json_path and json_path.exists():
        data = load_json(json_path)
        metadata.update({
            "repair_lessons_count": str(data.get("lesson_count") or 0),
            "repair_lessons_reuse_count": str(data.get("reuse_count") or 0),
            "repair_lessons_avoid_count": str(data.get("avoid_count") or 0),
            "repair_lessons_top_reuse": compact(data.get("top_reuse") or "", 320),
            "repair_lessons_top_avoid": compact(data.get("top_avoid") or "", 320),
            "repair_lessons_preview": compact(json.dumps(data, sort_keys=True), 700),
        })
    if path and path.exists() and not metadata["repair_lessons_preview"]:
        metadata["repair_lessons_preview"] = compact(path.read_text(encoding="utf-8", errors="replace"), 700)
    return metadata


def repair_lesson_effectiveness_metadata(root, value, json_value=""):
    path = resolve_artifact(root, value)
    json_path = resolve_artifact(root, json_value)
    if not json_path and path:
        candidate = path.with_suffix(".json")
        if candidate.exists():
            json_path = candidate
    metadata = {
        "repair_lesson_effectiveness": rel(path, root) if path else "",
        "repair_lesson_effectiveness_json": rel(json_path, root) if json_path else "",
        "repair_lesson_effectiveness_used_count": "",
        "repair_lesson_effectiveness_satisfied_count": "",
        "repair_lesson_effectiveness_partial_count": "",
        "repair_lesson_effectiveness_failed_count": "",
        "repair_lesson_effectiveness_success_rate": "",
        "repair_lesson_effectiveness_failure_rate": "",
        "repair_lesson_effectiveness_top_reuse": "",
        "repair_lesson_effectiveness_top_avoid": "",
        "repair_lesson_effectiveness_preview": "",
    }
    if json_path and json_path.exists():
        data = load_json(json_path)
        metadata.update({
            "repair_lesson_effectiveness_used_count": str(data.get("used_count") or 0),
            "repair_lesson_effectiveness_satisfied_count": str(data.get("satisfied_count") or 0),
            "repair_lesson_effectiveness_partial_count": str(data.get("partial_count") or 0),
            "repair_lesson_effectiveness_failed_count": str(data.get("failed_count") or 0),
            "repair_lesson_effectiveness_success_rate": str(data.get("success_rate") or "0.00"),
            "repair_lesson_effectiveness_failure_rate": str(data.get("failure_rate") or "0.00"),
            "repair_lesson_effectiveness_top_reuse": compact(data.get("top_reuse") or "", 320),
            "repair_lesson_effectiveness_top_avoid": compact(data.get("top_avoid") or "", 320),
            "repair_lesson_effectiveness_preview": compact(json.dumps(data, sort_keys=True), 700),
        })
    if path and path.exists() and not metadata["repair_lesson_effectiveness_preview"]:
        metadata["repair_lesson_effectiveness_preview"] = compact(path.read_text(encoding="utf-8", errors="replace"), 700)
    return metadata


def harness_adaptation_effectiveness_metadata(root, value, json_value=""):
    path = resolve_artifact(root, value)
    json_path = resolve_artifact(root, json_value)
    if not json_path and path:
        candidate = path.with_suffix(".json")
        if candidate.exists():
            json_path = candidate
    metadata = {
        "harness_adaptation_effectiveness": rel(path, root) if path else "",
        "harness_adaptation_effectiveness_json": rel(json_path, root) if json_path else "",
        "harness_adaptation_effectiveness_used_count": "",
        "harness_adaptation_effectiveness_satisfied_count": "",
        "harness_adaptation_effectiveness_partial_count": "",
        "harness_adaptation_effectiveness_failed_count": "",
        "harness_adaptation_effectiveness_success_rate": "",
        "harness_adaptation_effectiveness_failure_rate": "",
        "harness_adaptation_effectiveness_top_reuse": "",
        "harness_adaptation_effectiveness_top_avoid": "",
        "harness_adaptation_effectiveness_preview": "",
    }
    if json_path and json_path.exists():
        data = load_json(json_path)
        metadata.update({
            "harness_adaptation_effectiveness_used_count": str(data.get("used_count") or 0),
            "harness_adaptation_effectiveness_satisfied_count": str(data.get("satisfied_count") or 0),
            "harness_adaptation_effectiveness_partial_count": str(data.get("partial_count") or 0),
            "harness_adaptation_effectiveness_failed_count": str(data.get("failed_count") or 0),
            "harness_adaptation_effectiveness_success_rate": str(data.get("success_rate") or "0.00"),
            "harness_adaptation_effectiveness_failure_rate": str(data.get("failure_rate") or "0.00"),
            "harness_adaptation_effectiveness_top_reuse": compact(data.get("top_reuse") or "", 320),
            "harness_adaptation_effectiveness_top_avoid": compact(data.get("top_avoid") or "", 320),
            "harness_adaptation_effectiveness_preview": compact(json.dumps(data, sort_keys=True), 700),
        })
    if path and path.exists() and not metadata["harness_adaptation_effectiveness_preview"]:
        metadata["harness_adaptation_effectiveness_preview"] = compact(path.read_text(encoding="utf-8", errors="replace"), 700)
    return metadata


def repair_decision_metadata(root, value, json_value=""):
    path = resolve_artifact(root, value)
    json_path = resolve_artifact(root, json_value)
    if not json_path and path:
        candidate = path.with_suffix(".json")
        if candidate.exists():
            json_path = candidate
    metadata = {
        "repair_decision": rel(path, root) if path else "",
        "repair_decision_json": rel(json_path, root) if json_path else "",
        "repair_decision_status": "",
        "repair_decision_kind": "",
        "repair_decision_focus": "",
        "repair_decision_next_need_kind": "",
        "repair_decision_next_need_query": "",
        "repair_decision_preview": "",
    }
    if json_path and json_path.exists():
        data = load_json(json_path)
        next_need = data.get("next_need") if isinstance(data.get("next_need"), dict) else {}
        metadata.update({
            "repair_decision_status": str(data.get("status") or ""),
            "repair_decision_kind": str(data.get("decision") or ""),
            "repair_decision_focus": compact(data.get("focus") or "", 320),
            "repair_decision_next_need_kind": str(next_need.get("kind") or ""),
            "repair_decision_next_need_query": compact(next_need.get("query") or "", 320),
            "repair_decision_preview": compact(json.dumps(data, sort_keys=True), 700),
        })
    if path and path.exists() and not metadata["repair_decision_preview"]:
        metadata["repair_decision_preview"] = compact(path.read_text(encoding="utf-8", errors="replace"), 700)
    return metadata


def harness_adaptation_metadata(root, value, json_value=""):
    path = resolve_artifact(root, value)
    json_path = resolve_artifact(root, json_value)
    if not json_path and path:
        candidate = path.with_suffix(".json")
        if candidate.exists():
            json_path = candidate
    metadata = {
        "harness_adaptation": rel(path, root) if path else "",
        "harness_adaptation_json": rel(json_path, root) if json_path else "",
        "harness_adaptation_status": "",
        "harness_adaptation_focus": "",
        "harness_adaptation_next_need_kind": "",
        "harness_adaptation_next_need_query": "",
        "harness_adaptation_target": "",
        "harness_adaptation_environment": "",
        "harness_adaptation_preview": "",
    }
    if json_path and json_path.exists():
        data = load_json(json_path)
        next_need = data.get("next_need") if isinstance(data.get("next_need"), dict) else {}
        target = data.get("target") if isinstance(data.get("target"), dict) else {}
        environment = data.get("environment") if isinstance(data.get("environment"), dict) else {}
        target_label = "/".join(
            item for item in [
                str(target.get("tentacle") or ""),
                str(target.get("tool") or ""),
            ]
            if item
        )
        environment_label = " ".join(
            item for item in [
                f"adapter={environment.get('adapter_status')}" if environment.get("adapter_status") else "",
                f"missing={environment.get('adapter_missing_core')}" if environment.get("adapter_missing_core") else "",
                f"field={environment.get('field')}/{environment.get('mini_task')}" if environment.get("field") or environment.get("mini_task") else "",
                f"verifier={environment.get('verifier_status')}" if environment.get("verifier_status") else "",
            ]
            if item
        )
        metadata.update({
            "harness_adaptation_status": str(data.get("status") or ""),
            "harness_adaptation_focus": compact(data.get("focus") or "", 320),
            "harness_adaptation_next_need_kind": str(next_need.get("kind") or ""),
            "harness_adaptation_next_need_query": compact(next_need.get("query") or "", 320),
            "harness_adaptation_target": compact(target_label, 220),
            "harness_adaptation_environment": compact(environment_label, 320),
            "harness_adaptation_preview": compact(json.dumps(data, sort_keys=True), 700),
        })
    if path and path.exists() and not metadata["harness_adaptation_preview"]:
        metadata["harness_adaptation_preview"] = compact(path.read_text(encoding="utf-8", errors="replace"), 700)
    return metadata


def harness_environment_profile_metadata(root, value, json_value=""):
    path = resolve_artifact(root, value)
    json_path = resolve_artifact(root, json_value)
    if not json_path and path:
        candidate = path.with_suffix(".json")
        if candidate.exists():
            json_path = candidate
    metadata = {
        "harness_environment_profile": rel(path, root) if path else "",
        "harness_environment_profile_json": rel(json_path, root) if json_path else "",
        "harness_environment_profile_status": "",
        "harness_environment_profile_id": "",
        "harness_environment_profile_execution_mode": "",
        "harness_environment_profile_provider_mode": "",
        "harness_environment_profile_desktop_mode": "",
        "harness_environment_profile_capabilities": "",
        "harness_environment_profile_constraints": "",
        "harness_environment_profile_installed_profiles": "",
        "harness_environment_profile_next_need_kind": "",
        "harness_environment_profile_next_need_query": "",
        "harness_environment_profile_preview": "",
    }
    if json_path and json_path.exists():
        data = load_json(json_path)
        next_need = data.get("next_need") if isinstance(data.get("next_need"), dict) else {}
        metadata.update({
            "harness_environment_profile_status": str(data.get("status") or ""),
            "harness_environment_profile_id": compact(data.get("profile_id") or "", 220),
            "harness_environment_profile_execution_mode": str(data.get("execution_mode") or ""),
            "harness_environment_profile_provider_mode": str(data.get("provider_mode") or ""),
            "harness_environment_profile_desktop_mode": str(data.get("desktop_mode") or ""),
            "harness_environment_profile_capabilities": compact(",".join(data.get("capabilities") or []), 320),
            "harness_environment_profile_constraints": compact(",".join(data.get("constraints") or []), 320),
            "harness_environment_profile_installed_profiles": compact(",".join(data.get("installed_profiles") or []), 320),
            "harness_environment_profile_next_need_kind": str(next_need.get("kind") or ""),
            "harness_environment_profile_next_need_query": compact(next_need.get("query") or "", 320),
            "harness_environment_profile_preview": compact(json.dumps(data, sort_keys=True), 700),
        })
    if path and path.exists() and not metadata["harness_environment_profile_preview"]:
        metadata["harness_environment_profile_preview"] = compact(path.read_text(encoding="utf-8", errors="replace"), 700)
    return metadata


def harness_environment_drift_metadata(root, value, json_value=""):
    path = resolve_artifact(root, value)
    json_path = resolve_artifact(root, json_value)
    if not json_path and path:
        candidate = path.with_suffix(".json")
        if candidate.exists():
            json_path = candidate
    metadata = {
        "harness_environment_drift": rel(path, root) if path else "",
        "harness_environment_drift_json": rel(json_path, root) if json_path else "",
        "harness_environment_drift_status": "",
        "harness_environment_drift_summary": "",
        "harness_environment_drift_detail": "",
        "harness_environment_drift_history_count": "",
        "harness_environment_drift_gained_count": "",
        "harness_environment_drift_lost_count": "",
        "harness_environment_drift_next_need_kind": "",
        "harness_environment_drift_next_need_query": "",
        "harness_environment_drift_preview": "",
    }
    if json_path and json_path.exists():
        data = load_json(json_path)
        next_need = data.get("next_need") if isinstance(data.get("next_need"), dict) else {}
        metadata.update({
            "harness_environment_drift_status": str(data.get("status") or ""),
            "harness_environment_drift_summary": compact(data.get("summary") or "", 260),
            "harness_environment_drift_detail": compact(data.get("detail") or "", 320),
            "harness_environment_drift_history_count": str(data.get("history_count") or "0"),
            "harness_environment_drift_gained_count": str(data.get("gained_count") or "0"),
            "harness_environment_drift_lost_count": str(data.get("lost_count") or "0"),
            "harness_environment_drift_next_need_kind": str(next_need.get("kind") or ""),
            "harness_environment_drift_next_need_query": compact(next_need.get("query") or "", 320),
            "harness_environment_drift_preview": compact(json.dumps(data, sort_keys=True), 700),
        })
    if path and path.exists() and not metadata["harness_environment_drift_preview"]:
        metadata["harness_environment_drift_preview"] = compact(path.read_text(encoding="utf-8", errors="replace"), 700)
    return metadata


def harness_environment_drift_effectiveness_metadata(root, value, json_value=""):
    path = resolve_artifact(root, value)
    json_path = resolve_artifact(root, json_value)
    if not json_path and path:
        candidate = path.with_suffix(".json")
        if candidate.exists():
            json_path = candidate
    metadata = {
        "harness_environment_drift_effectiveness": rel(path, root) if path else "",
        "harness_environment_drift_effectiveness_json": rel(json_path, root) if json_path else "",
        "harness_environment_drift_effectiveness_used_count": "",
        "harness_environment_drift_effectiveness_satisfied_count": "",
        "harness_environment_drift_effectiveness_partial_count": "",
        "harness_environment_drift_effectiveness_failed_count": "",
        "harness_environment_drift_effectiveness_success_rate": "",
        "harness_environment_drift_effectiveness_failure_rate": "",
        "harness_environment_drift_effectiveness_top_reuse": "",
        "harness_environment_drift_effectiveness_top_avoid": "",
        "harness_environment_drift_effectiveness_preview": "",
    }
    if json_path and json_path.exists():
        data = load_json(json_path)
        metadata.update({
            "harness_environment_drift_effectiveness_used_count": str(data.get("used_count") or 0),
            "harness_environment_drift_effectiveness_satisfied_count": str(data.get("satisfied_count") or 0),
            "harness_environment_drift_effectiveness_partial_count": str(data.get("partial_count") or 0),
            "harness_environment_drift_effectiveness_failed_count": str(data.get("failed_count") or 0),
            "harness_environment_drift_effectiveness_success_rate": str(data.get("success_rate") or "0.00"),
            "harness_environment_drift_effectiveness_failure_rate": str(data.get("failure_rate") or "0.00"),
            "harness_environment_drift_effectiveness_top_reuse": compact(data.get("top_reuse") or "", 320),
            "harness_environment_drift_effectiveness_top_avoid": compact(data.get("top_avoid") or "", 320),
            "harness_environment_drift_effectiveness_preview": compact(json.dumps(data, sort_keys=True), 700),
        })
    if path and path.exists() and not metadata["harness_environment_drift_effectiveness_preview"]:
        metadata["harness_environment_drift_effectiveness_preview"] = compact(path.read_text(encoding="utf-8", errors="replace"), 700)
    return metadata


def action_trace_metadata(root, value, json_value=""):
    path = resolve_artifact(root, value)
    json_path = resolve_artifact(root, json_value)
    if not json_path and path:
        candidate = path.with_suffix(".json")
        if candidate.exists():
            json_path = candidate
    metadata = {
        "action_trace_json": rel(json_path, root) if json_path else "",
        "action_trace_status": "",
        "action_trace_stage_count": "",
        "action_trace_last_action": "",
        "action_trace_repair_hint": "",
        "action_trace_next_need_kind": "",
        "action_trace_next_need_query": "",
        "action_trace_failed_stage": "",
        "action_trace_preview": "",
    }
    if json_path and json_path.exists():
        data = load_json(json_path)
        next_need = data.get("next_need") if isinstance(data.get("next_need"), dict) else {}
        metadata.update({
            "action_trace_status": clean_markdown_value(data.get("status")),
            "action_trace_stage_count": clean_markdown_value(data.get("stage_count")),
            "action_trace_last_action": clean_markdown_value(data.get("last_action")),
            "action_trace_repair_hint": clean_markdown_value(data.get("repair_hint")),
            "action_trace_next_need_kind": clean_markdown_value(next_need.get("kind")),
            "action_trace_next_need_query": clean_markdown_value(next_need.get("query")),
        })
        for stage in data.get("stages") if isinstance(data.get("stages"), list) else []:
            if not isinstance(stage, dict):
                continue
            stage_status = clean_markdown_value(stage.get("status"))
            if stage_status in {"failed", "missing_context"}:
                metadata["action_trace_failed_stage"] = clean_markdown_value(stage.get("action"))
                break
        metadata["action_trace_preview"] = compact(json.dumps(data, sort_keys=True), 600)
    if not path or not path.exists():
        return metadata
    text = path.read_text(encoding="utf-8", errors="replace")
    if not metadata["action_trace_preview"]:
        metadata["action_trace_preview"] = compact(text, 600)
    key_map = {
        "status": "action_trace_status",
        "stage_count": "action_trace_stage_count",
        "last_action": "action_trace_last_action",
        "repair_hint": "action_trace_repair_hint",
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


def command_lines(commands, key):
    return "\n".join(as_list(commands.get(key)))


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
next_need_source = "heartbeat_repair"

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
    repair_recall = str(inputs.get("repair_recall") or "")
    repair_lessons = str(inputs.get("repair_lessons") or "")
    repair_lessons_json = str(inputs.get("repair_lessons_json") or "")
    repair_lesson_effectiveness = str(inputs.get("repair_lesson_effectiveness") or "")
    repair_lesson_effectiveness_json = str(inputs.get("repair_lesson_effectiveness_json") or "")
    repair_decision = str(inputs.get("repair_decision") or "")
    repair_decision_json = str(inputs.get("repair_decision_json") or "")
    harness_adaptation_effectiveness = str(inputs.get("harness_adaptation_effectiveness") or "")
    harness_adaptation_effectiveness_json = str(inputs.get("harness_adaptation_effectiveness_json") or "")
    harness_environment_profile = str(inputs.get("harness_environment_profile") or "")
    harness_environment_profile_json = str(inputs.get("harness_environment_profile_json") or "")
    harness_environment_drift = str(inputs.get("harness_environment_drift") or "")
    harness_environment_drift_json = str(inputs.get("harness_environment_drift_json") or "")
    harness_environment_drift_effectiveness = str(inputs.get("harness_environment_drift_effectiveness") or "")
    harness_environment_drift_effectiveness_json = str(inputs.get("harness_environment_drift_effectiveness_json") or "")
    environment_profile_journal = str(inputs.get("environment_profile_journal") or "")
    harness_adaptation = str(inputs.get("harness_adaptation") or "")
    harness_adaptation_json = str(inputs.get("harness_adaptation_json") or "")
    action_trace = str(inputs.get("action_trace") or "")
    action_trace_json = str(inputs.get("action_trace_json") or "")
    draft_metadata = repair_draft_metadata(root, draft)
    recall_metadata = repair_recall_metadata(root, repair_recall)
    lessons_metadata = repair_lessons_metadata(root, repair_lessons, repair_lessons_json)
    effectiveness_metadata = repair_lesson_effectiveness_metadata(
        root,
        repair_lesson_effectiveness,
        repair_lesson_effectiveness_json,
    )
    adaptation_effectiveness_metadata = harness_adaptation_effectiveness_metadata(
        root,
        harness_adaptation_effectiveness,
        harness_adaptation_effectiveness_json,
    )
    environment_profile_metadata = harness_environment_profile_metadata(
        root,
        harness_environment_profile,
        harness_environment_profile_json,
    )
    environment_drift_metadata = harness_environment_drift_metadata(
        root,
        harness_environment_drift,
        harness_environment_drift_json,
    )
    environment_drift_effectiveness_metadata = harness_environment_drift_effectiveness_metadata(
        root,
        harness_environment_drift_effectiveness,
        harness_environment_drift_effectiveness_json,
    )
    decision_metadata = repair_decision_metadata(
        root,
        repair_decision,
        repair_decision_json,
    )
    adaptation_metadata = harness_adaptation_metadata(
        root,
        harness_adaptation,
        harness_adaptation_json,
    )
    action_metadata = action_trace_metadata(root, action_trace, action_trace_json)
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
    draft_status = draft_metadata.get("draft_status", "")
    draft_prefix = draft_metadata.get("draft_prefix", "")
    draft_blocked = (
        not has_outcome
        and not adapter_blocked
        and draft_status in {"missing_config", "failed"}
    )
    action_trace_status = action_metadata.get("action_trace_status", "")
    recall_count = recall_metadata.get("repair_recall_match_count", "")
    recall_top_status = recall_metadata.get("repair_recall_top_status", "")
    recall_top_reason = recall_metadata.get("repair_recall_top_reasons", "")
    lesson_count = lessons_metadata.get("repair_lessons_count", "")
    lesson_reuse = lessons_metadata.get("repair_lessons_reuse_count", "")
    lesson_avoid = lessons_metadata.get("repair_lessons_avoid_count", "")
    effectiveness_used = effectiveness_metadata.get("repair_lesson_effectiveness_used_count", "")
    effectiveness_success = effectiveness_metadata.get("repair_lesson_effectiveness_success_rate", "")
    decision_kind = decision_metadata.get("repair_decision_kind", "")
    decision_focus = decision_metadata.get("repair_decision_focus", "")
    decision_next_kind = decision_metadata.get("repair_decision_next_need_kind", "")
    decision_next_query = decision_metadata.get("repair_decision_next_need_query", "")
    adaptation_status = adaptation_metadata.get("harness_adaptation_status", "")
    adaptation_focus = adaptation_metadata.get("harness_adaptation_focus", "")
    adaptation_effectiveness_used = adaptation_effectiveness_metadata.get("harness_adaptation_effectiveness_used_count", "")
    adaptation_effectiveness_success = adaptation_effectiveness_metadata.get("harness_adaptation_effectiveness_success_rate", "")
    environment_profile_status = environment_profile_metadata.get("harness_environment_profile_status", "")
    environment_profile_mode = environment_profile_metadata.get("harness_environment_profile_execution_mode", "")
    environment_profile_capabilities = environment_profile_metadata.get("harness_environment_profile_capabilities", "")
    environment_profile_constraints = environment_profile_metadata.get("harness_environment_profile_constraints", "")
    environment_drift_status = environment_drift_metadata.get("harness_environment_drift_status", "")
    environment_drift_detail = environment_drift_metadata.get("harness_environment_drift_detail", "")
    environment_drift_history = environment_drift_metadata.get("harness_environment_drift_history_count", "")
    environment_drift_gained = environment_drift_metadata.get("harness_environment_drift_gained_count", "")
    environment_drift_lost = environment_drift_metadata.get("harness_environment_drift_lost_count", "")
    environment_drift_effectiveness_used = environment_drift_effectiveness_metadata.get("harness_environment_drift_effectiveness_used_count", "")
    environment_drift_effectiveness_success = environment_drift_effectiveness_metadata.get("harness_environment_drift_effectiveness_success_rate", "")
    adaptation_next_kind = adaptation_metadata.get("harness_adaptation_next_need_kind", "")
    adaptation_next_query = adaptation_metadata.get("harness_adaptation_next_need_query", "")
    next_need_source = "repair_plan"
    if adaptation_next_query:
        next_need_kind = adaptation_next_kind or "verify"
        next_need_query = adaptation_next_query
        next_need_source = "harness_adaptation"
    elif decision_next_query:
        next_need_kind = decision_next_kind or "verify"
        next_need_query = decision_next_query
        next_need_source = "repair_decision"
    action_trace_blocked = (
        not has_outcome
        and not adapter_blocked
        and not draft_blocked
        and action_trace_status in {"failed", "missing_context"}
    )
    if has_outcome:
        outcome_status = outcome_metadata.get("outcome_status", "reviewed")
        plan_status = f"outcome_{outcome_status}"
        if outcome_status in {"failed", "partial"}:
            status = "partial"
            next_need = f"continue repair after {outcome_status} outcome"
            next_need_kind = "execute"
            next_need_query = f"octopus repair . after {outcome_status} repair outcome"
            next_need_source = "repair_outcome"
        else:
            status = "satisfied"
            next_need = "retain reviewed repair outcome"
            next_need_kind = "verify"
            next_need_query = "harness repair outcome retained"
            next_need_source = "repair_outcome"
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
        next_need_source = "adapter_context"
        output = (
            f"heartbeat repair: repair_plan={rel(latest_repair_plan, root)}; "
            f"adapter_status={adapter_status or 'unknown'}; missing_core={missing}; "
            f"target={target_tentacle}/{target_tool}; next={next_need_kind} {next_need_query}"
        )
        checks = []
        suggested = []
    elif draft_blocked:
        plan_status = "draft_blocked"
        status = "partial"
        prefix = draft_prefix or "repair provider"
        next_need = f"repair provider draft: {draft_status}"
        next_need_kind = "execute"
        next_need_query = f"configure repair provider draft: {prefix}"
        next_need_source = "provider_draft"
        output = (
            f"heartbeat repair: repair_plan={rel(latest_repair_plan, root)}; "
            f"draft_status={draft_status}; prefix={prefix}; "
            f"target={target_tentacle}/{target_tool}; next={next_need_kind} {next_need_query}"
        )
        checks = []
        suggested = []
    elif action_trace_blocked:
        plan_status = "action_trace_blocked"
        status = "partial"
        blocker = action_trace_status or "failed"
        next_need = action_metadata.get("action_trace_repair_hint") or f"repair tool-side action trace: {blocker}"
        next_need_kind = action_metadata.get("action_trace_next_need_kind") or "execute"
        next_need_query = action_metadata.get("action_trace_next_need_query") or f"repair harness action trace: {blocker}"
        next_need_source = "action_trace"
        output = (
            f"heartbeat repair: repair_plan={rel(latest_repair_plan, root)}; "
            f"action_trace_status={blocker}; "
            f"failed_stage={action_metadata.get('action_trace_failed_stage') or 'unknown'}; "
            f"target={target_tentacle}/{target_tool}; next={next_need_kind} {next_need_query}"
        )
        checks = []
        suggested = []
    else:
        status = "satisfied"
        next_need = (
            f"follow harness adaptation: {adaptation_status}"
            if adaptation_status and adaptation_next_query
            else f"follow repair decision: {decision_kind}"
            if decision_kind and decision_next_query
            else "review latest repair action plan"
        )
        output = (
            f"heartbeat repair: repair_plan={rel(latest_repair_plan, root)}; "
            f"status={plan_status}; target={target_tentacle}/{target_tool}; "
            f"next={next_need_kind} {next_need_query}; source={next_need_source}; "
            f"field={field_label or 'none'}; action_trace={action_trace_status or 'none'}; "
            f"recall={recall_count or '0'} top={recall_top_status or 'none'} "
            f"reason={recall_top_reason or 'none'}; "
            f"lessons={lesson_count or '0'} reuse={lesson_reuse or '0'} avoid={lesson_avoid or '0'}; "
            f"effectiveness={effectiveness_used or '0'} success_rate={effectiveness_success or '0.00'}; "
            f"decision={decision_kind or 'none'} focus={decision_focus or 'none'}; "
            f"environment_profile={environment_profile_status or 'none'} mode={environment_profile_mode or 'none'} "
            f"capabilities={environment_profile_capabilities or 'none'} constraints={environment_profile_constraints or 'none'}; "
            f"environment_drift={environment_drift_status or 'none'} history={environment_drift_history or '0'} "
            f"gained={environment_drift_gained or '0'} lost={environment_drift_lost or '0'} detail={environment_drift_detail or 'none'}; "
            f"environment_drift_effectiveness={environment_drift_effectiveness_used or '0'} success_rate={environment_drift_effectiveness_success or '0.00'}; "
            f"adaptation={adaptation_status or 'none'} focus={adaptation_focus or 'none'} "
            f"adaptation_effectiveness={adaptation_effectiveness_used or '0'} success_rate={adaptation_effectiveness_success or '0.00'}; "
            "review before grant/apply/score"
        )
    repair_metadata = {
        "repair_plan": rel(latest_repair_plan, root),
        "repair_plan_status": plan_status,
        "repair_plan_schema": str(repair_plan.get("schema_version") or ""),
        "repair_plan_session": str(repair_plan.get("session") or ""),
        "review": str(inputs.get("review") or ""),
        "draft": draft,
        "code_context": code_context,
        "repair_recall": repair_recall,
        "repair_lessons": repair_lessons,
        "repair_lessons_json": repair_lessons_json,
        "repair_lesson_effectiveness": repair_lesson_effectiveness,
        "repair_lesson_effectiveness_json": repair_lesson_effectiveness_json,
        "repair_decision": repair_decision,
        "repair_decision_json": repair_decision_json,
        "harness_adaptation_effectiveness": harness_adaptation_effectiveness,
        "harness_adaptation_effectiveness_json": harness_adaptation_effectiveness_json,
        "harness_environment_profile": harness_environment_profile,
        "harness_environment_profile_json": harness_environment_profile_json,
        "harness_environment_drift": harness_environment_drift,
        "harness_environment_drift_json": harness_environment_drift_json,
        "harness_environment_drift_effectiveness": harness_environment_drift_effectiveness,
        "harness_environment_drift_effectiveness_json": harness_environment_drift_effectiveness_json,
        "environment_profile_journal": environment_profile_journal,
        "harness_adaptation": harness_adaptation,
        "harness_adaptation_json": harness_adaptation_json,
        "action_trace": action_trace,
        "action_trace_json": action_trace_json or action_metadata.get("action_trace_json", ""),
        "adapter_context": adapter_context,
        "field_trajectory": field_trajectory,
        "target_tentacle": target_tentacle,
        "target_tool": target_tool,
        "candidate": candidate,
        "review_boundary": str(repair_plan.get("review_boundary") or ""),
        "check_command": " && ".join(checks),
        "adapter_blocker": adapter_missing_core if adapter_blocked else "",
        "draft_blocker": draft_status if draft_blocked else "",
        "action_trace_blocker": action_trace_status if action_trace_blocked else "",
        "grant_command": "" if has_outcome or adapter_blocked or draft_blocked or action_trace_blocked else command_value(commands, "grant"),
        "apply_command": "" if has_outcome or adapter_blocked or draft_blocked or action_trace_blocked else command_value(commands, "apply"),
        "score_command": "" if has_outcome or adapter_blocked or draft_blocked or action_trace_blocked else command_value(commands, "score"),
        "score_commands": "" if has_outcome or adapter_blocked or draft_blocked or action_trace_blocked else command_lines(commands, "score_options"),
        "suggested_commands": " && ".join(suggested),
        "next_need_source": next_need_source,
    }
    repair_metadata.update(draft_metadata)
    repair_metadata.update(recall_metadata)
    repair_metadata.update(lessons_metadata)
    repair_metadata.update(effectiveness_metadata)
    repair_metadata.update(adaptation_effectiveness_metadata)
    repair_metadata.update(environment_profile_metadata)
    repair_metadata.update(environment_drift_metadata)
    repair_metadata.update(environment_drift_effectiveness_metadata)
    repair_metadata.update(decision_metadata)
    repair_metadata.update(adaptation_metadata)
    repair_metadata.update(action_metadata)
    repair_metadata.update(adapter_metadata)
    repair_metadata.update(field_metadata)
    repair_metadata.update(outcome_metadata)
elif not tentacles:
    status = "partial"
    next_need = "run beat after failed check or scored Feed"
    next_need_kind = "execute"
    next_need_query = "octopus beat 200 after failed check or scored Feed"
    next_need_source = "heartbeat_repair"
    output = "heartbeat repair: no evolution artifacts yet; collect failed or partial feedback, then run octopus beat 200"
else:
    status = "satisfied"
    next_need = "review harness apply plan"
    next_need_kind = "verify"
    next_need_query = "review harness apply plan"
    next_need_source = "harness_apply_plan"
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
    "next_need_source": next_need_source,
    "grant_boundary": "octopus oauth octopus evolve:<tentacle> harness:write",
}
metadata.update(repair_metadata)
for key in list(metadata.keys()):
    if key.endswith("_preview"):
        metadata[key] = compact(metadata[key], 320)

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
