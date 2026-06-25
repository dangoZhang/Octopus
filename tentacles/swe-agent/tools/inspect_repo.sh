#!/usr/bin/env bash
set -euo pipefail

root="${1:-$(pwd)}"
cd "$root"

echo "== git =="
git status --short --branch 2>/dev/null || echo "not a git repository"

echo "== files =="
if command -v rg >/dev/null 2>&1; then
  rg --files | sed -n '1,120p'
else
  find . -maxdepth 3 -type f | sed -n '1,120p'
fi

echo "== project =="
if [ -f Cargo.toml ]; then
  echo "rust"
fi
if [ -f pyproject.toml ]; then
  echo "python"
fi
if [ -f package.json ]; then
  echo "javascript"
fi
