#!/usr/bin/env bash
set -euo pipefail

root="${1:?usage: publish_pr.sh <root> <repo> <branch> <title> <body_file>}"
repo="${2:?usage: publish_pr.sh <root> <repo> <branch> <title> <body_file>}"
branch="${3:?usage: publish_pr.sh <root> <repo> <branch> <title> <body_file>}"
title="${4:?usage: publish_pr.sh <root> <repo> <branch> <title> <body_file>}"
body_file="${5:?usage: publish_pr.sh <root> <repo> <branch> <title> <body_file>}"
base="${OCTOPUS_PR_BASE:-main}"
dry_run="${OCTOPUS_PR_DRY_RUN:-0}"

cd "$root"
out_dir=".octopus/self-iteration"
mkdir -p "$out_dir"
publish_json="$out_dir/PR_PUBLISH.json"

write_report() {
  local mode="$1"
  local pr_url="$2"
  python3 - "$publish_json" "$repo" "$branch" "$base" "$title" "$body_file" "$mode" "$pr_url" <<'PY'
import json
import sys
from pathlib import Path

path, repo, branch, base, title, body_file, mode, pr_url = sys.argv[1:9]
commands = [
    f"git switch -c {branch}",
    "git add -A",
    f"git commit -m {title!r}",
    f"git push -u origin {branch}",
    f"gh pr create --repo {repo} --head {branch} --base {base} --title {title!r} --body-file {body_file} --draft",
]
Path(path).write_text(json.dumps({
    "mode": mode,
    "repository": repo,
    "branch": branch,
    "base": base,
    "title": title,
    "body_file": body_file,
    "pr_url": pr_url or None,
    "commands": commands,
}, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
}

if [ "$dry_run" = "1" ] || [ "$dry_run" = "true" ]; then
  write_report "dry-run" ""
  echo "mode=dry-run"
  echo "publish=$publish_json"
  echo "branch=$branch"
  exit 0
fi

command -v git >/dev/null 2>&1 || { echo "git command missing" >&2; exit 1; }
command -v gh >/dev/null 2>&1 || { echo "gh command missing" >&2; exit 1; }
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "not inside a git worktree" >&2; exit 1; }
test -s "$body_file" || { echo "PR body file missing or empty: $body_file" >&2; exit 1; }

current_branch="$(git branch --show-current 2>/dev/null || true)"
if [ "$current_branch" != "$branch" ]; then
  if git show-ref --verify --quiet "refs/heads/$branch"; then
    git switch "$branch"
  else
    git switch -c "$branch"
  fi
fi

if ! git diff --quiet || ! git diff --cached --quiet; then
  git add -A
  git commit -m "$title"
fi

git push -u origin "$branch"
pr_url="$(gh pr create --repo "$repo" --head "$branch" --base "$base" --title "$title" --body-file "$body_file" --draft)"
write_report "published" "$pr_url"

echo "mode=published"
echo "publish=$publish_json"
echo "branch=$branch"
echo "pr_url=$pr_url"
