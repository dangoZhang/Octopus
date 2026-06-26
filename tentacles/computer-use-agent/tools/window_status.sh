#!/usr/bin/env bash
set -euo pipefail

os_name="$(uname -s 2>/dev/null || printf 'unknown')"
front_app=""
window_title=""
source="none"
notes=""

if [ "$os_name" = "Darwin" ] && command -v osascript >/dev/null 2>&1; then
  result="$(osascript <<'OSA' 2>/dev/null || true
tell application "System Events"
  set frontApp to name of first application process whose frontmost is true
  set frontTitle to ""
  try
    tell process frontApp
      if (count of windows) > 0 then set frontTitle to name of front window
    end tell
  end try
  return frontApp & "\t" & frontTitle
end tell
OSA
)"
  if [ -n "$result" ]; then
    front_app="$(printf '%s' "$result" | awk -F '\t' '{print $1}')"
    window_title="$(printf '%s' "$result" | awk -F '\t' '{print $2}')"
    source="macos_osascript"
  else
    notes="macOS automation permission denied or no front window"
  fi
elif command -v xdotool >/dev/null 2>&1; then
  window_title="$(xdotool getactivewindow getwindowname 2>/dev/null || true)"
  front_app="$(xdotool getactivewindow getwindowpid 2>/dev/null | xargs -r ps -p 2>/dev/null | awk 'NR==2 {print $4}' || true)"
  source="xdotool"
elif [ -n "${WAYLAND_DISPLAY:-}" ] || [ -n "${DISPLAY:-}" ]; then
  source="desktop_env"
  notes="active window inspection needs xdotool, wmctrl, or a compositor-specific adapter"
else
  notes="no desktop session detected"
fi

ui_process_hints="$(
  ps -axo comm 2>/dev/null \
    | awk -F/ '{print $NF}' \
    | sort \
    | uniq \
    | grep -Ei 'Arc|Brave|Chrome|Codex|Code|Edge|Finder|Firefox|Safari|Terminal|iTerm' \
    | sed -n '1,20p' \
    | tr '\n' '\t' || true
)"

if command -v python3 >/dev/null 2>&1; then
  python3 - "$os_name" "$source" "$front_app" "$window_title" "$notes" "$ui_process_hints" <<'PY'
import json
import sys

os_name, source, front_app, window_title, notes, ui_process_hints = sys.argv[1:]
payload = {
    "os": os_name,
    "source": source,
    "front_app": front_app,
    "window_title": window_title,
    "display": {
        "wayland": bool(__import__("os").environ.get("WAYLAND_DISPLAY")),
        "x11": bool(__import__("os").environ.get("DISPLAY")),
    },
    "ui_process_hints": [item for item in ui_process_hints.split("\t") if item],
    "notes": [notes] if notes else [],
}
print(json.dumps(payload, ensure_ascii=False))
PY
else
  printf '{"os":"%s","source":"%s","front_app":"","window_title":"","notes":["python3 unavailable"]}\n' "$os_name" "$source"
fi
