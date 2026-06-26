#!/usr/bin/env bash
set -euo pipefail

text="${1:-}"
if [ -z "$text" ] && [ ! -t 0 ]; then
  text="$(cat)"
fi

os_name="$(uname -s 2>/dev/null || printf 'unknown')"
method="none"
written="false"
notes=""

write_with() {
  method="$1"
  shift
  if printf '%s' "$text" | "$@" >/dev/null 2>&1; then
    written="true"
  else
    notes="$method failed"
  fi
}

if [ "${OCTOPUS_CLIPBOARD_DRY_RUN:-0}" = "1" ]; then
  method="dry_run"
  notes="dry run; clipboard was not modified"
elif command -v pbcopy >/dev/null 2>&1; then
  write_with "pbcopy" pbcopy
elif command -v wl-copy >/dev/null 2>&1; then
  write_with "wl-copy" wl-copy
elif command -v xclip >/dev/null 2>&1; then
  write_with "xclip" xclip -selection clipboard
elif command -v xsel >/dev/null 2>&1; then
  write_with "xsel" xsel --clipboard --input
else
  notes="no clipboard write command found"
fi

if command -v python3 >/dev/null 2>&1; then
  python3 - "$os_name" "$method" "$written" "$notes" "$text" <<'PY'
import json
import sys

os_name, method, written, notes, text = sys.argv[1:]
payload = {
    "os": os_name,
    "method": method,
    "written": written == "true",
    "bytes": len(text.encode()),
    "notes": [notes] if notes else [],
}
print(json.dumps(payload, ensure_ascii=False))
PY
else
  printf '{"os":"%s","method":"%s","written":%s,"bytes":0,"notes":["python3 unavailable"]}\n' \
    "$os_name" "$method" "$written"
fi
