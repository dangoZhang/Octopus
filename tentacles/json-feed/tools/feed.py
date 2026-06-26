#!/usr/bin/env python3
import json
import sys
from pathlib import Path


MAX_BYTES = 4000
MAX_ENTRIES = 20


def compact(value):
    return " ".join(str(value).split())


def safe_target(query):
    token = query.strip().split()[0] if query.strip() else "."
    path = Path(token).expanduser()
    if not path.is_absolute():
        path = Path.cwd() / path
    try:
        resolved = path.resolve()
        cwd = Path.cwd().resolve()
        resolved.relative_to(cwd)
    except Exception:
        return token, None, "outside workspace"
    return token, resolved, None


def read_observation(query):
    token, path, error = safe_target(query)
    if error:
        return "partial", f"target {error}: {token}", 0.35, {"target": token, "target_state": error}
    if path is None or not path.exists():
        return "partial", f"target missing: {token}", 0.45, {"target": token, "target_state": "missing"}
    rel = str(path.relative_to(Path.cwd().resolve()))
    if path.is_dir():
        entries = sorted(item.name for item in path.iterdir())[:MAX_ENTRIES]
        content = f"directory {rel}: {', '.join(entries) if entries else 'empty'}"
        return "satisfied", content, 0.82, {"target": rel, "target_state": "directory"}
    try:
        text = path.read_text(encoding="utf-8", errors="replace")[:MAX_BYTES]
    except OSError as exc:
        return "partial", f"read failed: {rel}: {exc}", 0.4, {"target": rel, "target_state": "read_failed"}
    lines = text.splitlines()[:20]
    content = f"file {rel}: " + compact(" | ".join(lines))
    return "satisfied", content, 0.86, {"target": rel, "target_state": "file"}


def main():
    try:
        payload = json.load(sys.stdin)
    except json.JSONDecodeError as exc:
        print(json.dumps({
            "status": "failed",
            "output": f"invalid octopus-json-v1 payload: {exc}",
            "metadata": {"runtime": "python", "contract": "octopus-json-v1"}
        }))
        return 1

    need = payload.get("need", {})
    tool = payload.get("tool", {})
    tentacle = payload.get("tentacle", {})
    kind = need.get("kind", "observe")
    query = need.get("query", "")

    if kind == "observe":
        status, evidence_text, confidence, extra = read_observation(query)
    else:
        status = "satisfied"
        evidence_text = f"{kind} need received by Python JSON runtime: {compact(query)}"
        confidence = 0.72
        extra = {"target": compact(query), "target_state": "need_envelope"}

    output = f"json-feed {kind}: {evidence_text}"
    metadata = {
        "runtime": "python",
        "contract": "octopus-json-v1",
        "schema": payload.get("schema_version", ""),
        "tool": tool.get("id", "feed"),
        "tentacle": tentacle.get("id", "json-feed"),
        "need_kind": kind,
    }
    metadata.update(extra)
    print(json.dumps({
        "status": status,
        "output": output,
        "evidence": [{
            "source": "json-feed",
            "content": evidence_text,
            "confidence": confidence,
            "metadata": metadata
        }],
        "metadata": metadata
    }, ensure_ascii=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
