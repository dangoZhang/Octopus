# Quick Install & Use

## Source Run

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus --version
octopus bootstrap
octopus starter "make this repo easier to use"
octopus doctor
```

## Source Build

```bash
cargo test
cargo install --path crates/octopus-core --bin octopus --force
tmp=$(mktemp -d)
octopus demo dangoZhang/Octopus
octopus catalog
octopus starter "fix repo tests"
octopus --json starter "use desktop and browser"
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
octopus --state "$tmp/state.json" brain "what should the brain ask next?"
octopus --state "$tmp/state.json" brain --session "what should the brain ask next?"
octopus --state "$tmp/state.json" brain --goal --session "make the goal sharper"
octopus --state "$tmp/state.json" brain --apply-json '{"summary":"external chat explored","needs":[{"kind":"verify","query":"goal evidence stays clean"}]}' --save "what should the brain ask next?"
octopus --state "$tmp/state.json" brain --goal --apply-json '{"objective":"build a clean-brain agent","constraints":["Need only"],"summary":"external goal refined","needs":[{"kind":"observe","query":"goal history"}]}' --save "make the goal sharper"
octopus --state "$tmp/state.json" brain --goal --live --save "make the goal sharper"
octopus --state "$tmp/state.json" brain --live "what should the brain ask next?"
octopus --state "$tmp/state.json" brain --live --save "what should the brain ask next?"
octopus --state "$tmp/state.json" brain --session --live "what should the brain ask next?"
octopus --state "$tmp/state.json" explore "what should the brain ask next?"
octopus --state "$tmp/state.json" explore --save "what should the brain ask next?"
octopus --state "$tmp/state.json" needs
octopus --state "$tmp/state.json" needs session "review pending clean-brain Needs"
octopus --state "$tmp/state.json" needs script "$tmp/run-needs.sh"
octopus --state "$tmp/state.json" needs take 1
octopus --state "$tmp/state.json" context observe .
octopus --state "$tmp/state.json" think swe-agent observe README.md
octopus --state "$tmp/state.json" think harness-repair-agent observe .
octopus --state "$tmp/state.json" think harness-repair-agent execute .
octopus --state "$tmp/state.json" need execute "repair harness session"
octopus --state "$tmp/state.json" need observe .
octopus --state "$tmp/state.json" traces
octopus --state "$tmp/state.json" repair .
octopus --state "$tmp/state.json" needs
octopus --state "$tmp/state.json" repair score 1 satisfied "repair improved harness"
octopus --state "$tmp/state.json" routes observe .
octopus --state "$tmp/state.json" report
octopus --state "$tmp/state.json" preflight
octopus --state "$tmp/state.json" preflight script "$tmp/preflight.sh"
octopus --state "$tmp/state.json" preflight record "$tmp/real-machine-record.md"
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

Open `http://127.0.0.1:8765/app.html`. The app can bootstrap a local state, render and filter starter tentacle cards, install tentacles, inspect context, apply external brain replies, take/drop queued Needs, write pending Needs as a script, run a structured Feed test, score Feed or repair outcomes, show grant/check/next reports, and grant local Octopus tool scopes.
The Feed panel can score the latest trace as satisfied, partial, or failed; the Repair panel can score repair outcomes through the same harness journal. Both feedback paths update route data and pet color.
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
octopus --state "$tmp/state.json" brain --goal --live --save "make the goal sharper"
octopus --state "$tmp/state.json" brain --live "what should the brain ask next?"
octopus --state "$tmp/state.json" brain --session --live "what should the brain ask next?"
octopus --state "$tmp/state.json" brain --apply-json '{"summary":"external chat explored","needs":[{"kind":"compare","query":"provider and external chat paths"}]}' --save "what should the brain ask next?"

octopus --state "$tmp/state.json" chat "make the harness self-evolve"

octopus --state "$tmp/state.json" think swe-agent observe README.md

octopus --state "$tmp/state.json" --json need observe README.md 1 1

octopus --state "$tmp/state.json" evolve swe-agent "improve observe feed"

octopus provider save local OCTOPUS_LOCAL "$tmp/local-llm.env"
octopus provider save deepseek OCTOPUS_DEEPSEEK "$tmp/deepseek.env"
octopus provider save gemini OCTOPUS_GEMINI "$tmp/gemini.env"
octopus doctor
```

The native app can generate or save provider env, show the same provider layers, run an explicit provider check, render `octopus report`, render `octopus preflight`, run harness repair into Need Queue, take/drop queued Needs, score repair outcomes, and write local preflight script/record artifacts.

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

`beat 200` can now do the same from check history, Feed traces, or repair outcomes: failed or partial signals for a known tentacle become a harness-beat recommendation with an apply plan. In `docs/app.html`, the Harness Beat panel shows the candidate, plan path, apply-plan preview, next action, Harness Grant, Write Apply, and review buttons that write `evolve score` feedback.

`harness-repair-agent` reads those heartbeat/evolution artifacts as a normal tentacle Feed. Its `repair_session` tool writes `.octopus/harness-repair/SESSION.*`, `PROMPT.md`, `DRAFT.md`, `NEXT_NEED.json`, `COMMANDS.sh`, `OUTCOME_MEMORY.md`, `CODE_CONTEXT.md`, and `REPAIR_PLAN.json`, then returns the next grant, apply, or score step. Set `OCTOPUS_REPAIR_LLM=1` to let the configured provider fill `DRAFT.md`; otherwise it stays an offline review artifact. After review, `repair_outcome` records `OUTCOME.md` and `.octopus/harness-repair/outcomes.jsonl`, so later sessions can learn from satisfied, partial, or failed repairs. `repair score` mirrors app/CLI review into that journal when the scored trace came from a repair session. In `docs/app.html`, the Repair panel can score the repair Feed and render recent repair outcome memory.

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
audit: all Needs stay cognitive
feed_trace: json-feed/feed via rule
Feed traces
Octopus repair
Need queue
trace_index: 1
Octopus doctor
Octopus report
Octopus preflight
Octopus preflight script
Octopus preflight record
Octopus bridge: http://127.0.0.1:8765
pixel: 🟥
event: harness beat
heartbeat: alive
memory: compacted 0 memories
harness: evolved 2 routes
status: Satisfied
```
