#!/usr/bin/env bash
set -euo pipefail

url="${1:?usage: open_url.sh <url>}"

if command -v open >/dev/null 2>&1; then
  open "$url"
elif command -v xdg-open >/dev/null 2>&1; then
  xdg-open "$url"
else
  echo "no supported opener found" >&2
  exit 1
fi

