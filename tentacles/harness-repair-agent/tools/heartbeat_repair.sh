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


payload = read_payload()
root = resolve_workspace(sys.argv[1] if len(sys.argv) > 1 else ".", payload)
evolution_root = root / ".octopus/evolution"
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

if not tentacles:
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
