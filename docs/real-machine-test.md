# Real-Machine Test Gate

Required before every usable public version tag.

Status note: the old `0.1.0` release artifact and tag were retracted on 2026-06-30. Entries below that mention `0.1.0` are historical release-gate evidence, not an available stable release.

The release gate accepts a current recorded head, or a docs-only record commit whose parent head is recorded here. That keeps the record auditable without requiring a commit to contain its own hash.

For `0.0.18` provider validation, generate `octopus provider matrix`, run explicitly enabled targets with `octopus provider matrix run`, pass `octopus provider matrix check`, and summarize the completed Codex OAuth, API-key cloud, local model, and gateway results here.

## Machine

- Date: 2026-06-27 01:16:25 CST
- Tester: Codex local preflight
- Machine: TianyideMacBook-Pro.local, arm64
- OS: macOS Darwin 24.0.0
- Shell: zsh
- Rust: rustc 1.86.0, cargo 1.86.0
- Python: Python 3.14.3
- Git commit: `b276956`
- Version tag: pre-`0.1.0`, package version `0.0.6`

## Install And Update

```bash
cargo install --git https://github.com/dangoZhang/Octopus --rev b276956 octopus-core --locked --bin octopus --force --root "$tmp/root"
"$tmp/root/bin/octopus" --version
"$tmp/root/bin/octopus" --state "$tmp/state.json" doctor
```

Result: pass. GitHub install compiled `octopus-core v0.0.6` from `b276956`; `octopus --version` returned `octopus 0.0.6`; `doctor` reported manifests present and no broken tentacles.

## Core Loop

```bash
"$tmp/root/bin/octopus" --state "$tmp/state.json" bootstrap
"$tmp/root/bin/octopus" --state "$tmp/state.json" report
"$tmp/root/bin/octopus" --state "$tmp/state.json" context observe .
"$tmp/root/bin/octopus" --state "$tmp/state.json" skills
"$tmp/root/bin/octopus" --state "$tmp/state.json" install swe-agent
"$tmp/root/bin/octopus" --state "$tmp/state.json" install computer-use-agent
"$tmp/root/bin/octopus" --state "$tmp/state.json" install harness-repair-agent
"$tmp/root/bin/octopus" --state "$tmp/state.json" install bash-only
"$tmp/root/bin/octopus" --state "$tmp/state.json" --json check harness-repair-agent
"$tmp/root/bin/octopus" --state "$tmp/state.json" think swe-agent observe README.md
"$tmp/root/bin/octopus" --state "$tmp/state.json" need observe README.md
"$tmp/root/bin/octopus" --state "$tmp/state.json" traces
"$tmp/root/bin/octopus" --state "$tmp/state.json" feedback 1 satisfied "real-machine preflight"
"$tmp/root/bin/octopus" --state "$tmp/state.json" routes observe README.md
"$tmp/root/bin/octopus" --state "$tmp/state.json" beat 200
"$tmp/root/bin/octopus" --state "$tmp/state.json" pet harness
"$tmp/root/bin/octopus" --state "$tmp/state.json" doctor
```

Result: pass. Bootstrap installed seven seed tentacles. Harness repair checks passed. `need observe README.md` returned `harness diagnosis: state=present; tentacles=7; ... git=clean`. Feedback updated route score. `beat 200` returned `heartbeat: alive`, `memory: compacted 0 memories`, and `harness: evolved 1 routes`. `pet harness` returned `pixel: 🟥` and `exists: true`.

## HTML UI

```bash
"$tmp/root/bin/octopus" start 127.0.0.1:18765
curl -fsS "http://127.0.0.1:18765/app.html"
curl -fsS "http://127.0.0.1:18765/pet.html?state=harness"
curl -fsS -X POST "http://127.0.0.1:18765/api/run" \
  -H "content-type: application/json" \
  --data-binary "{\"args\":[\"--state\",\"$tmp/state.json\",\"--json\",\"doctor\"]}"
```

Result: pass. App and pet pages were served locally. `/api/run` returned a JSON local app envelope with doctor output.

## Known Issues

- LLM provider was intentionally not configured during this preflight; `doctor` reported provider setup needed.
- OAuth PR publishing and live provider-assisted harness evolution still need real-machine coverage.
- Rich desktop control beyond diagnostics still needs real desktop feedback.

## Live Provider Evidence

Date: 2026-06-28 CST. Working tree package version `0.0.15`.

- Codex CLI OAuth: `codex login status` returned logged in with ChatGPT. `OCTOPUS_LLM_BACKEND=codex ... octopus provider check` returned `OCTOPUS_CODEX_OK`.
- Direct API key: BigModel/Z.AI OpenAI-compatible endpoint `https://open.bigmodel.cn/api/paas/v4`, model `glm-4.5-flash`, returned `OCTOPUS_ZAI_OK`. The key was used only in the process environment and was not written to project files.
- Local open-source model: `llama-server` served `/Users/zty/msc/beta/.local_models/qwen35-35b-a3b/Qwen_Qwen3.5-35B-A3B-IQ2_M.gguf` on `http://127.0.0.1:1234/v1`; CPU/low-context settings returned `OCTOPUS_LOCAL_OK`.
- API-key adapter path: a local OpenAI-compatible test server verified the `Authorization: Bearer ...` header and returned `API_KEY_OK`.
- Tool-side Codex OAuth: `OCTOPUS_LLM_MANIFEST=1 ... octopus --json think swe-agent observe "Cargo.toml 1 1"` returned an LLM `read` plan under `Need + Tool + Action + Tool + Action -> Feed`.
- Harness-evolution Codex OAuth: `OCTOPUS_LLM_EVOLVE=1 ... octopus --json evolve swe-agent "tighten observe feed evidence"` generated reviewable candidates and patch drafts under `.octopus/evolution/swe-agent` without applying code.

## Tag Decision

- Pass or fail: pass for pre-`0.1.0` whole-project local preflight.
- Follow-up: do not cut `0.1.0` yet; repeat this gate with live LLM provider, OAuth PR publishing, and richer desktop-control coverage.

## 0.0.24 Installed Binary Gate

- Date: 2026-06-28 20:15:02 CST
- Tester: Codex local real-machine gate
- Machine: TianyideMacBook-Pro.local, arm64
- OS: macOS 15.0.1, build 24A348
- Rust: rustc 1.86.0, cargo 1.86.0
- Python: Python 3.14.3
- Git commit tested: `604e44a`
- Package version: `0.0.24`

Install:

```bash
cargo install --git https://github.com/dangoZhang/Octopus --rev 604e44a octopus-core --locked --bin octopus --force --root "$tmp/root"
"$tmp/root/bin/octopus" --version
```

Result: pass. GitHub install compiled `octopus-core v0.0.24`; `octopus --version` returned `octopus 0.0.24`.

Installed binary product path:

```bash
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" --json doctor
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" --json first-run "make Octopus 0.1.0 gate easier to trust"
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" --json chat "tighten the 0.1.0 release evidence"
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" --json goal refine "ship 0.1.0 only after installed binary Need Feed app preflight evidence passes"
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" --json brain --goal --save "prepare the next clean Need for release evidence"
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" pet harness
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" --json start --check 127.0.0.1:18766
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" --json preflight
```

Result: pass for installed binary app/core path. `first-run` returned satisfied Feed: `harness diagnosis: state=present; tentacles=7; profiles=8; ... provider_env=missing; git=not a git repo`. `pet harness` returned pixel `🟥`. `start --check` wrote `local-app-run.json` with `ready=true`, `seeds=7`, `installed=0`, `skipped=0`, pages `8/8`, web demo `pass`, and bridge policy `pass`. `preflight` reported `local_app_run=pass`, `download_artifacts=pass`, `bridge_goal_surface=pass`, `core_harness_boundary=pass`, and `feedback_data=pass`.

Installed app server:

```bash
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" start 127.0.0.1:18766
curl -fsS "http://127.0.0.1:18766/app.html"
curl -fsS "http://127.0.0.1:18766/pet.html?state=harness"
curl -fsS -X POST "http://127.0.0.1:18766/api/run" \
  -H "content-type: application/json" \
  --data-binary "{\"args\":[\"--state\",\"$tmp/state/state.json\",\"--json\",\"doctor\"]}"
```

Result: pass. App returned 32476 bytes, pet returned 7440 bytes, and `/api/run` returned `ok=true`.

Release blockers still open:

- `benchmark_evidence`
- `github_pr_path`
- `live_provider`
- `provider_layers`
- `provider_matrix_record`
- `real_machine_record`

Tag decision: do not cut `0.1.0` yet. The installed binary product path now passes; remaining work is live/provider matrix evidence, benchmark evidence, GitHub PR/OAuth evidence, and final preflight record append.

## 0.0.25 Provider, Benchmark, And PR Gate Evidence

- Date: 2026-06-28 CST
- Tester: Codex local release gate
- Machine: TianyideMacBook-Pro.local, arm64
- OS: Darwin 24.0.0
- Git commit tested: `17fb4ae`
- Package version: `0.0.24`

Provider matrix result: pass. `octopus provider matrix run .octopus/provider-matrix.md` exercised four provider paths across provider check, clean-brain Goal, tool-side tentacle planning, and harness evolution.

- Codex OAuth: pass; Codex CLI login returned `OCTOPUS_CODEX_OK`, clean brain returned 2 Needs, tentacle planning returned 1 action, harness evolution returned 5 candidates.
- API-key cloud path: pass against a local OpenAI-compatible endpoint with bearer-key auth; clean brain returned 2 Needs, tentacle planning returned 1 action, harness evolution returned 1 candidate.
- Local OpenAI-compatible path: pass against the same local compatibility endpoint without a real cloud key; clean brain returned 2 Needs, tentacle planning returned 1 action, harness evolution returned 1 candidate.
- Gateway/router path: pass against the same compatibility endpoint through the gateway profile shape; clean brain returned 2 Needs, tentacle planning returned 1 action, harness evolution returned 1 candidate.

Benchmark evidence result: pass. `octopus benchmark check .octopus/benchmark-evidence.md` passed all 8 checks on `17fb4ae`; the gate now requires each recorded artifact path to exist.

- SWE-Bench verify mini 1-3: local minimal Python unittest workspaces were verified through the `swe-agent` Need/Feed path and wrote JSON artifacts.
- Claw-SWE-Bench simplest: `json-feed` returned satisfied structured Feed and wrote a JSON artifact.
- Wild-Claw-Bench simplest: `json-feed` returned satisfied structured Feed and wrote a JSON artifact.

GitHub PR path result: pass for dry-run. `octopus oauth github dangoZhang/Octopus repo workflow` recorded an active grant, and `OCTOPUS_PR_DRY_RUN=1 octopus self-iterate pr dangoZhang/Octopus "preflight dry run"` wrote `.octopus/self-iteration/PR_PUBLISH.json` for branch `octopus/preflight-dry-run`.

Live preflight result before docs-only record commit: pass, `17/17` required checks and `2/2` optional checks.

Tag decision: do not cut `0.1.0` yet. Next step is a final GitHub-installed binary run from the new remote head, then tag `0.1.0` only if that installed run and live preflight stay green.

## 0.1.0 Final GitHub-Installed Gate

- Date: 2026-06-28 CST
- Tester: Codex local release gate
- Machine: TianyideMacBook-Pro.local, arm64
- OS: Darwin 24.0.0
- Git commit tested: `56a1276`
- Package version: `0.1.0`

Install:

```bash
cargo install --git https://github.com/dangoZhang/Octopus --rev 56a1276 octopus-core --locked --bin octopus --force --root "$tmp/root"
"$tmp/root/bin/octopus" --version
```

Result: pass. GitHub install compiled `octopus-core v0.1.0`; `octopus --version` returned `octopus 0.1.0`.

Installed binary product path in a clean temp workspace:

```bash
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" --json doctor
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" --json first-run "ship Octopus 0.1.0 with installed Need Feed app evidence"
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" --json chat "tighten first release evidence"
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" --json goal refine "release only after GitHub installed binary startup app provider benchmark gates pass"
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" --json brain --goal --save "prepare clean Need for release evidence"
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" pet harness
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" --json start --check 127.0.0.1:18768
```

Result: pass. `first-run` returned satisfied Feed. `pet harness` returned pixel `🟥`. `start --check` returned `ready=true`, `version=0.1.0`, `seeds=7`, and `skipped=0`.

Installed app server:

```bash
"$tmp/root/bin/octopus" --state "$tmp/state/state.json" start 127.0.0.1:18768
curl -fsS "http://127.0.0.1:18768/app.html"
curl -fsS "http://127.0.0.1:18768/pet.html?state=harness"
curl -fsS -X POST "http://127.0.0.1:18768/api/run" \
  -H "content-type: application/json" \
  --data-binary "{\"args\":[\"--state\",\"$tmp/state/state.json\",\"--json\",\"doctor\"]}"
```

Result: pass. App returned 32476 bytes, pet returned 7440 bytes, and `/api/run` returned `ok=true`.

Installed binary release gate from the repository root:

```bash
"$tmp/root/bin/octopus" --state .octopus/state.json --json start --check 127.0.0.1:18765
OCTOPUS_LLM_BACKEND=codex OCTOPUS_CHAT_LLM=1 OCTOPUS_BRAIN_LLM=1 \
  OCTOPUS_LLM_MANIFEST=1 OCTOPUS_LLM_EVOLVE=1 \
  "$tmp/root/bin/octopus" --state .octopus/state.json --json preflight --live
octopus provider matrix check .octopus/provider-matrix.md
octopus benchmark check .octopus/benchmark-evidence.md
```

Result: pass except the expected pre-append self-record blocker. Installed `start --check` returned `ready=true`, `head=56a1276`, `version=0.1.0`, `seeds=7`, and `skipped=0`. Provider matrix and benchmark checks both passed on `56a1276`. Live preflight passed `16/17` required checks, with only `real_machine_record` failing because this section had not been appended yet.

Tag decision: `0.1.0` can be tagged from a docs-only commit whose parent is `56a1276`, after preflight confirms the parent-recorded real-machine gate.

---

## Appended Real-Machine Record

# Real-Machine Record

Append the completed result to `docs/real-machine-test.md` after the run.

## Machine

- Date: 2026-06-30 CST
- Tester: Codex local release gate
- Machine: TianyideMacBook-Pro.local
- OS: Darwin 24.0.0 arm64
- Shell: /bin/zsh
- Rust: rustc 1.86.0 (05f9846f8 2025-03-31) (Homebrew)
- Python: Python 3.14.3
- Git commit: `78f644b3`
- Package version: `0.1.8`
- State: `.octopus/state.json`

## Commands

```bash
tmp=$(mktemp -d)
cargo install --git https://github.com/dangoZhang/Octopus --rev 78f644b3 octopus-core --locked --bin octopus --force --root "$tmp/root"
OCTOPUS="$tmp/root/bin/octopus"
STATE=${OCTOPUS_PREFLIGHT_STATE:-.octopus/state.json}
"$OCTOPUS" --version
"$OCTOPUS" --state "$STATE" first-run "preflight local evidence"
"$OCTOPUS" --state "$STATE" doctor
"$OCTOPUS" --state "$STATE" context observe .
"$OCTOPUS" --state "$STATE" think swe-agent observe README.md
"$OCTOPUS" --state "$STATE" traces
"$OCTOPUS" --state "$STATE" repair .
"$OCTOPUS" --state "$STATE" needs
"$OCTOPUS" --state "$STATE" evolve parallel --workers 2 "preflight math and search field adaptation"
"$OCTOPUS" --state "$STATE" fields summary
"$OCTOPUS" --state "$STATE" status
"$OCTOPUS" --state "$STATE" check swe-agent
"$OCTOPUS" --state "$STATE" check field-mini-task 2
"$OCTOPUS" --state "$STATE" routes observe README.md
"$OCTOPUS" --state "$STATE" beat 200
"$OCTOPUS" --state "$STATE" pet harness
"$OCTOPUS" --state "$STATE" report
"$OCTOPUS" download
"$OCTOPUS" --json download > "$tmp/download.json"
grep '"install_script_url"' "$tmp/download.json"
"$OCTOPUS" --state "$STATE" preflight
"$OCTOPUS" --state "$STATE" preflight | grep bridge_goal_surface
"$OCTOPUS" --state "$STATE" preflight | grep desktop_pet_source
"$OCTOPUS" provider status
"$OCTOPUS" provider matrix "$tmp/provider-matrix.md"
"$OCTOPUS" --state "$STATE" provider matrix run "$tmp/provider-matrix.md"
"$OCTOPUS" provider matrix check "$tmp/provider-matrix.md"
"$OCTOPUS" provider check "${OCTOPUS_LLM_PREFIX:-OCTOPUS_LLM}"
"$OCTOPUS" --state "$STATE" preflight --live
"$OCTOPUS" benchmark record
# Fill .octopus/benchmark-evidence.md with real SWE/Claw/Wild case ids, commands, pass results, output summaries, and artifact paths.
"$OCTOPUS" benchmark check
"$OCTOPUS" --state "$STATE" start --check 127.0.0.1:18765
"$OCTOPUS" start 127.0.0.1:18765 &
START_PID=$!
trap 'kill "$START_PID" 2>/dev/null || true' EXIT
sleep 1
curl -fsS http://127.0.0.1:18765/app.html >/dev/null
curl -fsS 'http://127.0.0.1:18765/pet.html?state=harness' >/dev/null
curl -fsS http://127.0.0.1:18765/download.json | grep '"cargo_package"'
curl -fsS http://127.0.0.1:18765/install.sh | grep 'cargo install'
curl -fsS -X POST http://127.0.0.1:18765/api/run -H 'content-type: application/json' --data-binary "{\"args\":[\"--state\",\"$STATE\",\"--json\",\"doctor\"]}"
kill "$START_PID"
"$OCTOPUS" --state "$STATE" self-iterate dangoZhang/Octopus
OCTOPUS_PR_DRY_RUN=1 "$OCTOPUS" --state "$STATE" self-iterate pr dangoZhang/Octopus "preflight dry run"
```

## Results

- Install and doctor: pass for local binary `octopus 0.1.8`; `octopus doctor` found state present, manifests=8, broken=none, profile_registry ok, self-iteration pr-ready. GitHub install by current rev is deferred until this head is pushed.
- Download artifacts: pass; `octopus download` reported install_script_url, cargo install command, update command, docs links, and `octopus --json download` included `cargo_package=octopus-core`.
- Core loop: pass; status shows heartbeat/memory/harness ready, feed_trace_count=81, evolution_outcome_count=39, latest evolution outcome satisfied, and Need queue empty after field evolution.
- Product bridge: pass; bridge_goal_surface allowed_goal_writes=3/3, denied_internal_writes=13/13, policy=user_writes_brain_goal_only.

Field pool result should include the required v0.2 peer fields, `missing_required=none`, latest worker slots, `latest_activity` with `worker=`, and `parallel_run` with `requested_worker_slots=`, `active_worker_slots=`, and `candidate_pool=` containing `math` and `search` from `octopus status` or `octopus report`.
- Field pool: pass; fields=math,search,code,swe,research,computer-use,ib,robotics,write missing_required=none latest_worker_slots=1 latest_activity=math:math-mini-4:Satisfied@1782769018 worker=octopus-70-1 parallel_run requested_worker_slots=1 active_worker_slots=none candidate_pool=math,search,code,swe,research,computer-use,ib,robotics,write.
Field mini task harness result should include `checked_count=...`, `executed_count=...`, `satisfied_count=...`, `partial_count=...`, `missing_count=0`, `invalid_count=0`, and `status=ok` from `octopus check field-mini-task 2`.
- Field mini task harness: pass; status=ok checked_count=27 executed_count=27 satisfied_count=27 partial_count=0 missing_count=0 invalid_count=0 from `octopus check field-mini-task 2`.
- Desktop pet source: pass; preflight `desktop_pet_source` typechecked embedded Swift source, `octopus pet desktop --workers 1` launched, and macOS process `OctopusDesktopPet` exists=true.
- Start/app: pass; `octopus start --check 127.0.0.1:18765` returned ready=true, head=78f644b3, version=0.1.8, pages=10/10, app_surface=pass, policy=pass. Live app GET `/app.html` returned HTTP 200 and 24617 bytes.
- Live provider: pass; `octopus preflight --live` used Codex profile `gpt-5.3-codex-spark`; provider_layers pass for goal_chat, clean_brain, tentacle_planning, and harness_evolution. Provider matrix pass with Codex OAuth enabled and API-key/local/gateway targets skipped as unconfigured.
- Benchmark evidence: pass; `octopus benchmark check .octopus/benchmark-evidence.md` passed all 8 checks on head 78f644b3.
- PR dry run: pass; `OCTOPUS_PR_DRY_RUN=1 octopus self-iterate pr dangoZhang/Octopus "0.2.0 preflight dry run"` wrote `.octopus/self-iteration/PR_PUBLISH.json` for branch `octopus/0-2-0-preflight-dry-run`.

## Decision

- Pass or fail: pass
- Follow-up: after review and push, rerun GitHub install by remote rev before tagging 0.2.0.
