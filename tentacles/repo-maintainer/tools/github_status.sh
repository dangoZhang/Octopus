#!/usr/bin/env bash
set -euo pipefail

repo="${1:?usage: github_status.sh <owner/repo> [limit]}"
limit="${2:-10}"

if command -v gh >/dev/null 2>&1; then
  issues=$(gh issue list --repo "$repo" --limit "$limit" --json number,title,state,labels,updatedAt 2>/dev/null || printf '[]')
  runs=$(gh run list --repo "$repo" --limit "$limit" --json databaseId,status,conclusion,workflowName,headBranch,createdAt 2>/dev/null || printf '[]')
  source="gh"
else
  issues="[]"
  runs="[]"
  source="gh_unavailable"
fi

python3 - "$repo" "$source" "$issues" "$runs" <<'PY'
import json
import sys

repo, source, issues_raw, runs_raw = sys.argv[1:5]
try:
    issues = json.loads(issues_raw)
except json.JSONDecodeError:
    issues = []
try:
    runs = json.loads(runs_raw)
except json.JSONDecodeError:
    runs = []

print(json.dumps({
    "repository": repo,
    "source": source,
    "issues": issues,
    "runs": runs,
}, ensure_ascii=False, indent=2))
PY
