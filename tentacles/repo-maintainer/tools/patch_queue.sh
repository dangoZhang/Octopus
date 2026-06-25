#!/usr/bin/env bash
set -euo pipefail

root="${1:?usage: patch_queue.sh <root> <repo> <objective...>}"
repo="${2:?usage: patch_queue.sh <root> <repo> <objective...>}"
shift 2
objective="${*:-improve Octopus usability}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$root"
out_dir=".octopus/self-iteration"
mkdir -p "$out_dir"

status_json="$out_dir/GITHUB_STATUS.json"
queue_md="$out_dir/PATCH_QUEUE.md"

"$script_dir/github_status.sh" "$repo" > "$status_json"

python3 - "$status_json" "$queue_md" "$objective" <<'PY'
import json
import re
import sys
from pathlib import Path

status_path, queue_path, objective = sys.argv[1:4]
status = json.loads(Path(status_path).read_text())

def slug(value: str) -> str:
    value = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    return (value or "update")[:48]

candidates = []
for issue in status.get("issues", []):
    title = issue.get("title") or f"Issue {issue.get('number')}"
    candidates.append({
        "source": f"issue#{issue.get('number')}",
        "title": title,
        "branch": f"octopus/issue-{issue.get('number')}-{slug(title)}",
        "reason": "open issue",
    })

for run in status.get("runs", []):
    if run.get("conclusion") not in (None, "", "success"):
        name = run.get("workflowName") or "workflow"
        title = f"Fix {name} failure"
        candidates.append({
            "source": f"run:{run.get('databaseId')}",
            "title": title,
            "branch": f"octopus/fix-{slug(name)}",
            "reason": f"CI conclusion: {run.get('conclusion')}",
        })

if not candidates:
    candidates.append({
        "source": "goal",
        "title": objective[:1].upper() + objective[1:],
        "branch": f"octopus/{slug(objective)}",
        "reason": "no open issue or failing CI; use current goal",
    })

lines = ["# Patch Queue", "", f"Repository: {status.get('repository')}", ""]
for index, item in enumerate(candidates, 1):
    lines.extend([
        f"## {index}. {item['title']}",
        f"- source: {item['source']}",
        f"- branch: {item['branch']}",
        f"- reason: {item['reason']}",
        "",
    ])

Path(queue_path).write_text("\n".join(lines), encoding="utf-8")
print(f"queue={queue_path}")
print(f"candidates={len(candidates)}")
print(f"top_branch={candidates[0]['branch']}")
PY
