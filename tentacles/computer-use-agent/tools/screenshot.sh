#!/usr/bin/env bash
set -euo pipefail

out="${1:-.octopus/harness/computer-use-agent/screenshot.png}"
mkdir -p "$(dirname "$out")"

if command -v screencapture >/dev/null 2>&1; then
  screencapture -x "$out"
elif command -v gnome-screenshot >/dev/null 2>&1; then
  gnome-screenshot -f "$out"
else
  echo "no supported screenshot command found" >&2
  exit 1
fi

echo "$out"

