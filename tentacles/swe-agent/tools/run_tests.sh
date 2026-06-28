#!/usr/bin/env bash
set -euo pipefail

root="${1:-$(pwd)}"
cd "$root"

unset OCTOPUS_STATE_PATH OCTOPUS_STATE

if test -f Cargo.toml; then
  cargo test
fi

if test -f pyproject.toml && test -d tests; then
  PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -q
fi
