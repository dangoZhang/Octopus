# Real-Machine Test Gate

Required before `0.1.0` and every later version tag.

The release gate accepts a current recorded head, or a docs-only record commit whose parent head is recorded here. That keeps the record auditable without requiring a commit to contain its own hash.

For `0.0.18` provider validation, generate `octopus provider matrix` and summarize the completed Codex OAuth, API-key cloud, local model, and gateway results here.

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
