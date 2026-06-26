#!/usr/bin/env bash
set -euo pipefail

os_name="$(uname -s 2>/dev/null || printf 'unknown')"
method="none"
content=""
notes=""

if [ "${OCTOPUS_CLIPBOARD_DRY_RUN:-0}" = "1" ]; then
  method="dry_run"
  notes="dry run; clipboard was not read"
elif command -v pbpaste >/dev/null 2>&1; then
  method="pbpaste"
  content="$(pbpaste 2>/dev/null || true)"
elif command -v wl-paste >/dev/null 2>&1; then
  method="wl-paste"
  content="$(wl-paste 2>/dev/null || true)"
elif command -v xclip >/dev/null 2>&1; then
  method="xclip"
  content="$(xclip -selection clipboard -out 2>/dev/null || true)"
elif command -v xsel >/dev/null 2>&1; then
  method="xsel"
  content="$(xsel --clipboard --output 2>/dev/null || true)"
else
  notes="no clipboard read command found"
fi

if command -v python3 >/dev/null 2>&1; then
  python3 - "$os_name" "$method" "$notes" "$content" <<'PY'
import json
import sys

os_name, method, notes, content = sys.argv[1:]
payload = {
    "os": os_name,
    "method": method,
    "available": method not in {"none", "dry_run"},
    "content": content,
    "bytes": len(content.encode()),
    "notes": [notes] if notes else [],
}
print(json.dumps(payload, ensure_ascii=False))
PY
else
  available="true"
  if [ "$method" = none ] || [ "$method" = dry_run ]; then
    available="false"
  fi
  printf '{"os":"%s","method":"%s","available":%s,"content":"","bytes":0,"notes":["python3 unavailable"]}\n' \
    "$os_name" "$method" "$available"
fi
