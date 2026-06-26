#!/usr/bin/env bash
set -euo pipefail

os_name="$(uname -s 2>/dev/null || printf 'unknown')"
opener="none"
if command -v open >/dev/null 2>&1; then
  opener="open"
elif command -v xdg-open >/dev/null 2>&1; then
  opener="xdg-open"
fi

browser_file="$(mktemp)"
active_file="$(mktemp)"
trap 'rm -f "$browser_file" "$active_file"' EXIT

add_browser() {
  printf '%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$4" >> "$browser_file"
}

add_command_browser() {
  id="$1"
  name="$2"
  command_name="$3"
  path="$(command -v "$command_name" 2>/dev/null || true)"
  if [ -n "$path" ]; then
    add_browser "$id" "$name" "command" "$path"
  fi
}

add_app_bundle() {
  id="$1"
  name="$2"
  path="$3"
  if [ -d "$path" ]; then
    add_browser "$id" "$name" "app" "$path"
  fi
}

capture_safari_tab() {
  command -v osascript >/dev/null 2>&1 || return 0
  command -v pgrep >/dev/null 2>&1 || return 0
  pgrep -x "Safari" >/dev/null 2>&1 || return 0
  result="$(osascript <<'OSA' 2>/dev/null || true
tell application "Safari"
  if (count of windows) is 0 then return ""
  set activeTab to current tab of front window
  return (URL of activeTab) & "\t" & (name of activeTab)
end tell
OSA
)"
  if [ -n "$result" ]; then
    printf 'safari\tSafari\t%s\n' "$result" >> "$active_file"
  fi
}

capture_chromium_tab() {
  process_name="$1"
  app_name="$2"
  id="$3"
  name="$4"
  command -v osascript >/dev/null 2>&1 || return 0
  command -v pgrep >/dev/null 2>&1 || return 0
  pgrep -x "$process_name" >/dev/null 2>&1 || return 0
  result="$(osascript <<OSA 2>/dev/null || true
tell application "$app_name"
  if (count of windows) is 0 then return ""
  set activeTab to active tab of front window
  return (URL of activeTab) & "\t" & (title of activeTab)
end tell
OSA
)"
  if [ -n "$result" ]; then
    printf '%s\t%s\t%s\n' "$id" "$name" "$result" >> "$active_file"
  fi
}

add_command_browser "chrome" "Google Chrome" "google-chrome"
add_command_browser "chromium" "Chromium" "chromium"
add_command_browser "chromium" "Chromium" "chromium-browser"
add_command_browser "firefox" "Firefox" "firefox"
add_command_browser "brave" "Brave" "brave-browser"
add_command_browser "edge" "Microsoft Edge" "microsoft-edge"

if [ "$os_name" = "Darwin" ]; then
  add_app_bundle "safari" "Safari" "/Applications/Safari.app"
  add_app_bundle "chrome" "Google Chrome" "/Applications/Google Chrome.app"
  add_app_bundle "chrome" "Google Chrome" "$HOME/Applications/Google Chrome.app"
  add_app_bundle "firefox" "Firefox" "/Applications/Firefox.app"
  add_app_bundle "brave" "Brave Browser" "/Applications/Brave Browser.app"
  add_app_bundle "edge" "Microsoft Edge" "/Applications/Microsoft Edge.app"

  capture_safari_tab
  capture_chromium_tab "Google Chrome" "Google Chrome" "chrome" "Google Chrome"
  capture_chromium_tab "Brave Browser" "Brave Browser" "brave" "Brave Browser"
  capture_chromium_tab "Microsoft Edge" "Microsoft Edge" "edge" "Microsoft Edge"
fi

py_bin=""
if command -v python3 >/dev/null 2>&1; then
  py_bin="python3"
fi

if [ -z "$py_bin" ]; then
  printf '{"os":"%s","opener":"%s","browsers":[],"active_tabs":[],"notes":["python not found; browser_status could not serialize diagnostics"]}\n' "$os_name" "$opener"
  exit 0
fi

"$py_bin" - "$os_name" "$opener" "$browser_file" "$active_file" <<'PY'
import json
import sys

os_name, opener, browser_path, active_path = sys.argv[1:]


def read_rows(path, size):
    rows = []
    with open(path, "r", encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            parts = raw.rstrip("\n").split("\t", size - 1)
            if len(parts) == size:
                rows.append(parts)
    return rows


seen = set()
browsers = []
for browser_id, name, source, path in read_rows(browser_path, 4):
    key = (browser_id, source, path)
    if key in seen:
        continue
    seen.add(key)
    browsers.append(
        {
            "id": browser_id,
            "name": name,
            "source": source,
            "path": path,
        }
    )

active_tabs = []
for browser_id, name, url, title in read_rows(active_path, 4):
    active_tabs.append(
        {
            "browser": browser_id,
            "name": name,
            "url": url,
            "title": title,
        }
    )

notes = []
if not browsers:
    notes.append("no browser command or app bundle detected")
if os_name != "Darwin":
    notes.append("active tab inspection is only implemented for macOS")
elif not active_tabs:
    notes.append("active tabs unavailable or OS automation permission denied")

print(
    json.dumps(
        {
            "os": os_name,
            "opener": opener,
            "browsers": browsers,
            "active_tabs": active_tabs,
            "notes": notes,
        },
        ensure_ascii=False,
        indent=2,
    )
)
PY
