# Octopus 🐙

> Build agents whose tools plan, execute, and improve outside the main model's context.

Octopus is a local-first agent harness for builders who want powerful tools without flooding model context. The main brain keeps the objective, memory, a clean cognitive request, and the compact evidence that comes back. Tentacles carry prompt, metadata, code, permissions, and runtime adapters, so they can plan, execute, ask for grants, and improve from scored traces.

The bet is simple: give tools their own intelligence, and the main model gets room to think.

## Why

- Tool plumbing should not crowd out judgment.
- Tool behavior should be inspectable code, not hidden prompt drift.
- Self-improvement should leave reviewable artifacts.
- Local power needs explicit permission boundaries.

## What It Does

- Keeps the brain context to goal, memory, request, evidence.
- Runs editable tentacles with prompt, metadata, runtime code, and policy.
- Learns from Feed traces, check history, route outcomes, and feedback scores.
- Gates risky local actions through explicit grants.
- Shows runtime state through a pixel Octopus that changes color.
- Uses a Rust kernel with shell, Python, MCP, and native tentacle runtimes.

## Quick Install & Use

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus
octopus --version
octopus bootstrap
octopus need execute "repair harness session"
octopus doctor
octopus preflight
octopus preflight script
octopus preflight record
```

Open the local control surface:

```bash
octopus bridge
```

Then visit `http://127.0.0.1:8765/app.html` for brain prompts, exploration, Need Queue, preflight, and the pixel pet.

Try the core flow:

```bash
octopus goal set "make this repo easier to use"
octopus brain "what should the brain ask next?"
octopus brain --live "what should the brain ask next?"
octopus brain --live --save "what should the brain ask next?"
octopus explore "what should the brain ask next?"
octopus explore --save "what should the brain ask next?"
octopus needs
octopus needs take 1
octopus context observe .
octopus need observe README.md
octopus traces
octopus repair .
octopus needs
octopus feedback 1 satisfied "useful evidence"
octopus beat 200
octopus pet
```

Add an OpenAI-compatible provider when you want live LLM planning:

```bash
octopus provider save openai
source .octopus/llm.env
octopus provider check
```

## Example Output

```text
Octopus preflight
target: 0.1.0
version: 0.0.7
live: false
release_ready: false
checks:
- clean_brain_boundary [pass required=true]: brain=Goal + Mem + Need + Feed
next: octopus preflight record, octopus preflight script, octopus report
```

## What Works Today

- Local bootstrap, doctor, report, preflight, release-gate script, and real-machine record template.
- Goal refinement, clean-brain exploration, context inspection, Feed traces, and feedback scoring.
- Harness repair can turn tool-side diagnosis into a queued Need.
- Seed tentacles for SWE work, computer-use diagnostics, repo maintenance, harness repair, bash-only execution, and structured JSON Feed.
- OpenAI-compatible chat, live clean-brain exploration, tool-side planning, and harness evolution candidates.
- Native HTML app for setup, providers, traces, checks, grants, pet state, and harness review.
- Reviewable evolution artifacts instead of silent self-patching.

## Seed Tentacles

- `swe-agent`: read, edit, patch, inspect, test.
- `computer-use-agent`: browser/window diagnostics, clipboard, MCP, local desktop probes.
- `repo-maintainer`: repo inspection, PR drafts, grant-bound publishing.
- `harness-repair-agent`: state, traces, checks, adapters, and evolution diagnosis.
- `bash-only`: transparent write-and-run execution.
- `json-feed`: `octopus-json-v1` runtime seed.

## Proof

- CI covers Rust tests, Clippy, Python tests, manifests, install path, seed tool checks, app strings, and preflight artifact generation.
- Release readiness is exposed by `octopus preflight`, a reviewable script, and a real-machine record template.
- License: MIT.
- Status: pre-`0.1`; `0.1.0` is reserved for recorded real-machine release testing.

## Links

- Product app: [docs/app.html](docs/app.html)
- Pixel pet: [docs/pet.html](docs/pet.html)
- Quick install guide: [docs/quickstart.md](docs/quickstart.md)
- Architecture: [docs/architecture.md](docs/architecture.md)
- Real-machine gate: [docs/real-machine-test.md](docs/real-machine-test.md)
- Tentacle contract: [tentacles/README.md](tentacles/README.md)
