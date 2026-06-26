# Quick Install & Use

## Source Run

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus --version
octopus doctor
```

## Source Build

```bash
cargo test
cargo install --path crates/octopus-core --bin octopus --force
tmp=$(mktemp -d)
octopus demo dangoZhang/Octopus
octopus catalog
octopus skills
octopus manifests
octopus providers
octopus provider status
octopus --lang zh env
octopus --state "$tmp/state.json" init
octopus --state "$tmp/state.json" bootstrap
octopus --state "$tmp/state.json" install research
octopus --state "$tmp/state.json" install json-feed
octopus --state "$tmp/state.json" install swe-agent
octopus --state "$tmp/state.json" install computer-use-agent
octopus --state "$tmp/state.json" install harness-repair-agent
octopus --state "$tmp/state.json" --json install computer-use-agent
octopus --state "$tmp/state.json" --json check computer-use-agent
octopus --state "$tmp/state.json" --json check computer-use-agent 1
octopus --state "$tmp/state.json" --json check harness-repair-agent
octopus --state "$tmp/state.json" install bash-only
octopus --state "$tmp/state.json" installed
octopus --state "$tmp/state.json" goal set "build a clean-brain agent"
octopus --state "$tmp/state.json" chat "build a clean-brain agent"
octopus --state "$tmp/state.json" explore "what should the brain ask next?"
octopus --state "$tmp/state.json" context observe .
octopus --state "$tmp/state.json" think swe-agent observe README.md
octopus --state "$tmp/state.json" need observe .
octopus --state "$tmp/state.json" traces
octopus --state "$tmp/state.json" routes observe .
octopus --state "$tmp/state.json" report
octopus --state "$tmp/state.json" preflight
octopus --state "$tmp/state.json" preflight script "$tmp/preflight.sh"
octopus --state "$tmp/state.json" feedback 1 satisfied "feed worked"
octopus --state "$tmp/state.json" oauth octopus tool:bash-only tool:execute
octopus --state "$tmp/state.json" need execute "echo octopus"
octopus --state "$tmp/state.json" pet
octopus --state "$tmp/state.json" doctor
octopus --state "$tmp/state.json" chat "make tools think"
octopus --state "$tmp/state.json" beat 200
octopus --state "$tmp/state.json" pet
octopus --state "$tmp/state.json" goal
```

## Local App

```bash
octopus bridge
```

Open `http://127.0.0.1:8765/app.html`. The app can bootstrap a local state, install tentacles, inspect context, run a structured Feed test, show grant/check/next reports, and grant local Octopus tool scopes.
The Feed panel can score the latest trace as satisfied, partial, or failed; that feedback updates harness route data and pet color.
It can also save provider env to `.octopus/llm.env`; bridge reads that file when it runs child Octopus commands.

## Tentacle Run

```bash
(cd "$tmp" && octopus scaffold my-feed python)
(cd "$tmp" && octopus probe my-feed observe README.md)
(cd "$tmp" && octopus scaffold native-feed rust)
(cd "$tmp" && octopus manifests tentacles)
```

Known runtimes get starter code. Any other runtime gets a manifest and `tools/feed` contract so the tentacle can add its own adapter, MCP bridge, native binary, or remote executor.

`json-feed` is the built-in non-shell seed: Python receives the full `octopus-json-v1` envelope and returns structured feedback.

## Tool Grants

```bash
octopus oauth octopus tool:bash-only tool:execute
octopus oauth octopus tool:computer-use-agent tool:observe tool:ui
```

Tools can declare `permission` in their manifest. Without a matching grant, Octopus returns `needs_authorization` as Feed and does not run that tool.

`install` prints the tentacle's needs, runtimes, required grant commands, manifest checks, and next commands. `--json install <tentacle>` returns the same report for the HTML app or another shell.

## Optional LLM

```bash
octopus providers
octopus provider status
octopus provider save openai OCTOPUS_LLM "$tmp/llm.env"
# set OPENAI_API_KEY first, then:
. "$tmp/llm.env"
octopus provider check

octopus --state "$tmp/state.json" chat "make the harness self-evolve"

octopus --state "$tmp/state.json" think swe-agent observe README.md

octopus --state "$tmp/state.json" --json need observe README.md 1 1

octopus --state "$tmp/state.json" evolve swe-agent "improve observe feed"

octopus provider save local OCTOPUS_LOCAL "$tmp/local-llm.env"
octopus doctor
```

The native app can generate or save provider env, show the same provider layers, run an explicit provider check, render `octopus report`, render `octopus preflight`, and write a local preflight script.

## Self-Iteration

```bash
octopus --state "$tmp/state.json" self-iterate dangoZhang/Octopus
octopus --state "$tmp/state.json" oauth github dangoZhang/Octopus
octopus --state "$tmp/state.json" self-iterate dangoZhang/Octopus
OCTOPUS_PR_DRY_RUN=1 octopus --state "$tmp/state.json" self-iterate pr dangoZhang/Octopus "build a clean-brain agent"
octopus evolve swe-agent "improve repository observation feed quality"
octopus evolve apply swe-agent runtime_code
octopus evolve score swe-agent 03-runtime-code satisfied "patch improved feed"
octopus evolve recommend swe-agent "improve repository observation feed quality"
tentacles/repo-maintainer/tools/patch_queue.sh "$tmp" dangoZhang/Octopus "build a clean-brain agent"
```

`evolve` writes `PROPOSAL.md`, `PATCH_CANDIDATES.md`, `PATCH_DRAFTS.md`, `patches/`, `apply/`, optional `.patch` files, and `proposal.json` under `.octopus/evolution/<tentacle>/`. With `OCTOPUS_LLM_EVOLVE=1`, LLM candidates may include a provider-assisted patch draft for review. `evolve recommend` uses previous `evolve score` outcomes to pick the next apply plan.

`beat 200` can now do the same from check history: failed or partial checks for a known tentacle become a harness-beat recommendation with an apply plan. In `docs/app.html`, the Harness Beat panel shows the candidate, plan path, apply-plan preview, next action, Harness Grant, Write Apply, and review buttons that write `evolve score` feedback.

## Expected Signals

```text
Octopus demo
Octopus init
Provider status
Octopus bootstrap
installed swe-agent
grants:
checks:
next:
json-feed observe
Octopus context
Octopus think
plan_source: rule
feed_trace: json-feed/feed via rule
Feed traces
trace_index: 1
Octopus doctor
Octopus report
Octopus preflight
Octopus preflight script
Octopus bridge: http://127.0.0.1:8765
pixel: 🟥
event: harness beat
heartbeat: alive
memory: compacted 0 memories
harness: evolved 2 routes
status: Satisfied
```
