#!/usr/bin/env bash
set -euo pipefail

root="${1:-$(pwd)}"
cd "$root"

echo "repo_root=$(pwd)"
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "branch=$(git branch --show-current 2>/dev/null || true)"
  echo "status:"
  git status --short
else
  echo "git=unavailable"
fi

echo "manifests:"
for manifest in Cargo.toml pyproject.toml package.json README.md docs; do
  if [ -e "$manifest" ]; then
    echo "- $manifest"
  fi
done
