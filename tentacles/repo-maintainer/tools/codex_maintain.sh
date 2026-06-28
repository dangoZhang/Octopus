#!/usr/bin/env bash
set -euo pipefail

usage="usage: codex_maintain.sh <root> <repo> <objective...>"

if [ "$#" -ge 2 ] && [ -d "$1" ]; then
  root="$1"
  repo="$2"
  shift 2
  objective="${*:-maintain repository freshness}"
elif [ "$#" -ge 1 ]; then
  root="${OCTOPUS_WORKSPACE:-$(pwd)}"
  repo="${OCTOPUS_REPOSITORY:-unknown}"
  objective="$*"
else
  root="${OCTOPUS_WORKSPACE:-$(pwd)}"
  repo="${OCTOPUS_REPOSITORY:-unknown}"
  objective="maintain repository freshness"
fi

test -d "$root" || { echo "$usage" >&2; echo "missing root: $root" >&2; exit 1; }

codex_cmd="${OCTOPUS_CODEX_COMMAND:-codex}"
run_codex="${OCTOPUS_CODEX_RUN:-0}"
write_mode="${OCTOPUS_CODEX_WRITE:-0}"
sandbox="${OCTOPUS_CODEX_SANDBOX:-read-only}"
if [ "$write_mode" = "1" ] || [ "$write_mode" = "true" ]; then
  sandbox="${OCTOPUS_CODEX_SANDBOX:-workspace-write}"
fi
timeout_secs="${OCTOPUS_CODEX_TIMEOUT_SECS:-180}"
case "$timeout_secs" in
  ''|*[!0-9]*) timeout_secs=180 ;;
esac

out_dir="$root/.octopus/self-iteration"
mkdir -p "$out_dir"
prompt_path="$out_dir/CODEX_MAINTAIN_PROMPT.md"
report_path="$out_dir/CODEX_MAINTAIN_REPORT.md"
stdout_path="$out_dir/CODEX_MAINTAIN_STDOUT.log"
stderr_path="$out_dir/CODEX_MAINTAIN_STDERR.log"
json_path="$out_dir/CODEX_MAINTAIN.json"

branch=""
status_short=""
recent_commit=""
if git -C "$root" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  branch="$(git -C "$root" branch --show-current 2>/dev/null || true)"
  status_short="$(git -C "$root" status --short 2>/dev/null || true)"
  recent_commit="$(git -C "$root" log -1 --oneline 2>/dev/null || true)"
fi

cat > "$prompt_path" <<EOF_PROMPT
# Codex Repository Maintenance Prompt

Repository: $repo
Workspace: $root
Branch: ${branch:-unknown}
Recent commit: ${recent_commit:-unknown}
Objective: $objective

You are running inside Octopus repo-maintainer. Use the local workspace as the
source of truth. Do not print secrets. Do not commit, push, or open PRs.

Check the repository as a maintainer and return a concise report with:

- what is already good
- concrete stale, broken, or confusing items
- the smallest safe patch worth making next
- files that should change, if any
- checks to run before publishing
- whether the work should become a pull request

Runtime guardrails:

- use at most five quick read-only shell checks
- do not audit every external link in one run
- prefer a follow-up recommendation over an exhaustive scan
- write the final report even when evidence is partial

For Awesome-LLM, prefer technical-report-first maintenance. Pay special
attention to README length, docs/wiki placement, GitHub Pages configuration,
latestness notes, and stale standalone-project references.

Current git status:

\`\`\`text
${status_short:-clean}
\`\`\`

Sandbox request: $sandbox
EOF_PROMPT

mode="prepared"
codex_exit=0
if [ "$run_codex" = "1" ] || [ "$run_codex" = "true" ]; then
  command -v "$codex_cmd" >/dev/null 2>&1 || { echo "codex command missing: $codex_cmd" >&2; exit 1; }
  mode="codex-read-only"
  if [ "$sandbox" != "read-only" ]; then
    mode="codex-workspace-write"
  fi
  prompt_text="$(cat "$prompt_path")"
  cmd=(
    "$codex_cmd" exec
    --cd "$root"
    --ephemeral
    --skip-git-repo-check
    --sandbox "$sandbox"
    --output-last-message "$report_path"
  )
  if [ -n "${OCTOPUS_CODEX_MODEL:-}" ]; then
    cmd+=(--model "$OCTOPUS_CODEX_MODEL")
  fi
  cmd+=("$prompt_text")
  set +e
  "${cmd[@]}" >"$stdout_path" 2>"$stderr_path" &
  codex_pid=$!
  elapsed=0
  timed_out="false"
  while kill -0 "$codex_pid" 2>/dev/null; do
    if [ "$timeout_secs" -gt 0 ] && [ "$elapsed" -ge "$timeout_secs" ]; then
      timed_out="true"
      kill "$codex_pid" 2>/dev/null || true
      sleep 2
      kill -9 "$codex_pid" 2>/dev/null || true
      wait "$codex_pid" 2>/dev/null || true
      break
    fi
    sleep 1
    elapsed=$((elapsed + 1))
  done
  if [ "$timed_out" = "true" ]; then
    codex_exit=124
    mode="${mode}-timeout"
    printf '\nOctopus: Codex maintenance timed out after %ss.\n' "$timeout_secs" >> "$stderr_path"
  else
    wait "$codex_pid"
    codex_exit=$?
  fi
  set -e
else
  cat > "$report_path" <<EOF_REPORT
# Codex Maintenance Report

Mode: prepared

Codex was not run. Set \`OCTOPUS_CODEX_RUN=1\` to execute the prompt with the
local Codex CLI login/API-key context.

Prompt: $prompt_path
EOF_REPORT
fi

changed_files=""
if git -C "$root" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  changed_files="$(git -C "$root" status --short 2>/dev/null || true)"
fi

python3 - "$json_path" "$mode" "$repo" "$root" "$objective" "$sandbox" "$timeout_secs" \
  "$prompt_path" "$report_path" "$stdout_path" "$stderr_path" "$codex_exit" "$changed_files" <<'PY'
import json
import sys
from pathlib import Path

(
    json_path,
    mode,
    repo,
    root,
    objective,
    sandbox,
    timeout_secs,
    prompt_path,
    report_path,
    stdout_path,
    stderr_path,
    codex_exit,
    changed_files,
) = sys.argv[1:14]

Path(json_path).write_text(json.dumps({
    "mode": mode,
    "repository": repo,
    "root": root,
    "objective": objective,
    "sandbox": sandbox,
    "timeout_secs": int(timeout_secs),
    "prompt": prompt_path,
    "report": report_path,
    "stdout": stdout_path,
    "stderr": stderr_path,
    "codex_exit": int(codex_exit),
    "changed_files": [line for line in changed_files.splitlines() if line.strip()],
}, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY

echo "mode=$mode"
echo "prompt=$prompt_path"
echo "report=$report_path"
echo "json=$json_path"
echo "codex_exit=$codex_exit"
exit "$codex_exit"
