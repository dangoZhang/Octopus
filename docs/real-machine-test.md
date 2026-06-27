# Real-Machine Test Gate

Required before `0.1.0` and every later version tag.

The release gate accepts a current recorded head, or a docs-only record commit whose parent head is recorded here. That keeps the record auditable without requiring a commit to contain its own hash.

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
"$tmp/root/bin/octopus" bridge 127.0.0.1:18765
curl -fsS "http://127.0.0.1:18765/app.html"
curl -fsS "http://127.0.0.1:18765/pet.html?state=harness"
curl -fsS -X POST "http://127.0.0.1:18765/api/run" \
  -H "content-type: application/json" \
  --data-binary "{\"args\":[\"--state\",\"$tmp/state.json\",\"--json\",\"doctor\"]}"
```

Result: pass. App and pet pages were served locally. `/api/run` returned a JSON bridge envelope with doctor output.

## Known Issues

- LLM provider was intentionally not configured during this preflight; `doctor` reported provider setup needed.
- OAuth PR publishing and live provider-assisted harness evolution still need real-machine coverage.
- Rich desktop control beyond diagnostics still needs real desktop feedback.

## Tag Decision

- Pass or fail: pass for pre-`0.1.0` local MVP preflight.
- Follow-up: do not cut `0.1.0` yet; repeat this gate with live LLM provider, OAuth PR publishing, and richer desktop-control coverage.
