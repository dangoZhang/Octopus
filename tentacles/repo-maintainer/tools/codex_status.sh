#!/usr/bin/env bash
set -euo pipefail

root="${1:-$(pwd)}"
repo="${2:-}"
codex_cmd="${OCTOPUS_CODEX_COMMAND:-codex}"
live="${OCTOPUS_CODEX_STATUS_LIVE:-0}"
auth_file="${CODEX_HOME:-$HOME/.codex}/auth.json"

codex_path=""
codex_version=""
codex_available="false"
if codex_path="$(command -v "$codex_cmd" 2>/dev/null)"; then
  codex_available="true"
  codex_version="$("$codex_path" --version 2>&1 || true)"
fi

gh_available="false"
gh_authenticated="false"
if command -v gh >/dev/null 2>&1; then
  gh_available="true"
  if gh auth status >/dev/null 2>&1; then
    gh_authenticated="true"
  fi
fi

git_branch=""
git_dirty="unknown"
if [ -d "$root" ]; then
  if git -C "$root" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git_branch="$(git -C "$root" branch --show-current 2>/dev/null || true)"
    if [ -n "$(git -C "$root" status --short 2>/dev/null || true)" ]; then
      git_dirty="true"
    else
      git_dirty="false"
    fi
  fi
fi

token_env="false"
if [ -n "${CODEX_ACCESS_TOKEN:-}" ]; then
  token_env="true"
fi

auth_file_present="false"
if [ -f "$auth_file" ]; then
  auth_file_present="true"
fi

live_status="skipped"
live_reply=""
live_error=""
if { [ "$live" = "1" ] || [ "$live" = "true" ]; } && [ "$codex_available" = "true" ]; then
  tmp_dir="$(mktemp -d)"
  live_out="$tmp_dir/last-message.txt"
  live_stdout="$tmp_dir/stdout.txt"
  live_stderr="$tmp_dir/stderr.txt"
  if "$codex_path" exec \
    --ephemeral \
    --skip-git-repo-check \
    --sandbox read-only \
    --output-last-message "$live_out" \
    "Reply exactly with OCTOPUS_CODEX_OK." \
    >"$live_stdout" 2>"$live_stderr"; then
    live_status="ok"
    live_reply="$(tr -d '\r' < "$live_out" | head -n 1)"
  else
    live_status="failed"
    live_error="$(tail -c 2000 "$live_stderr" | tr '\n' ' ')"
  fi
fi

python3 - "$root" "$repo" "$codex_cmd" "$codex_path" "$codex_version" \
  "$codex_available" "$token_env" "$auth_file_present" "$gh_available" \
  "$gh_authenticated" "$git_branch" "$git_dirty" "$live_status" "$live_reply" \
  "$live_error" <<'PY'
import json
import sys

(
    root,
    repo,
    codex_command,
    codex_path,
    codex_version,
    codex_available,
    token_env,
    auth_file_present,
    gh_available,
    gh_authenticated,
    git_branch,
    git_dirty,
    live_status,
    live_reply,
    live_error,
) = sys.argv[1:16]

print(json.dumps({
    "root": root,
    "repository": repo or None,
    "codex": {
        "command": codex_command,
        "path": codex_path or None,
        "version": codex_version or None,
        "available": codex_available == "true",
        "access_token_env_present": token_env == "true",
        "auth_cache_present": auth_file_present == "true",
        "live_status": live_status,
        "live_reply": live_reply or None,
        "live_error": live_error or None,
    },
    "github": {
        "gh_available": gh_available == "true",
        "gh_authenticated": gh_authenticated == "true",
    },
    "workspace": {
        "branch": git_branch or None,
        "dirty": None if git_dirty == "unknown" else git_dirty == "true",
    },
}, ensure_ascii=False, indent=2))
PY
