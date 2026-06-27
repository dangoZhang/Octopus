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
