# Quick Install & Use

For the shortest product walkthrough, open [Octopus Tutorial](tutorial.html), then keep [Recipes](recipes.html) and [Use Octopus in five minutes](use.html) nearby for CLI details.

Machine-readable install/update links are in [download.json](download.json) and from `octopus download`.

One-line install:

```bash
curl -fsSL https://dangozhang.github.io/Octopus/install.sh | sh
```

## Install

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus download
octopus start --open
```

`start --open` opens `http://127.0.0.1:8765/app.html`. Use `octopus start` on headless machines.

`start` is the whole-project entry: it prepares `.octopus/` state, installs editable seed tentacles, materializes the editable profile registry, pulses the three hearts, serves the native app and pixel pet, loads `.octopus/llm.env`, and falls back to bundled app/tentacle files when no source checkout is present.

For release or machine evidence, run:

```bash
octopus start --check
```

That writes `.octopus/local-app-run.json` with state, page, app URL, and bridge-policy evidence for `preflight`.

Product users change Goal. The profile registry is a developer/harness surface for changing tentacle supply paths, permissions, checks, and evolution policy.

Update later:

```bash
octopus download
octopus update
octopus update --run
```

## From Source

```bash
git clone https://github.com/dangoZhang/Octopus
cd Octopus
cargo run -p octopus-core --bin octopus -- start
```

Or install the local binary:

```bash
cargo install --path crates/octopus-core --bin octopus --force
octopus start --open
```

## First Loop

Run these from the project you want Octopus to inhabit:

```bash
octopus first-run "make this repo easier to use"
```

That command sets a clean Goal, bootstraps seed tentacles, runs a safe observe Feed, scores the latest trace, pulses the hearts, and returns Doctor plus Preflight evidence.

After launch, the user-writable surface stays limited to brain-goal:

```bash
octopus chat "make this repo easier to use"
octopus goal refine "prefer small reviewable changes"
octopus brain --goal --save "tighten the current objective"
```

Need, Feed, route choice, provider routing, repair, and harness evolution are internal agent work. The app can show those states for review, but user input should only change Goal.

## Optional Model Backends

Octopus can use Codex login, API keys, local OpenAI-compatible servers, or LiteLLM-style gateways. Treat this as runtime plumbing; it should not leak into Need text.

Codex login/OAuth:

```bash
codex login
export OCTOPUS_LLM_BACKEND=codex
export OCTOPUS_LLM_CODEX_COMMAND=codex
```

OpenAI-compatible API, router, or local server:

```bash
export OCTOPUS_LLM_BACKEND=openai-compatible
export OCTOPUS_LLM_MODEL=gpt-4.1-mini
export OCTOPUS_LLM_BASE_URL=https://api.openai.com/v1
export OCTOPUS_LLM_API_KEY="$OPENAI_API_KEY"
```

Local example:

```bash
export OCTOPUS_LLM_BACKEND=openai-compatible
export OCTOPUS_LLM_MODEL=local-model
export OCTOPUS_LLM_BASE_URL=http://localhost:1234/v1
export OCTOPUS_LLM_API_KEY=
```

If you launch through `octopus start`, place the same exports in `.octopus/llm.env`. Check readiness without changing state:

```bash
octopus providers
octopus provider status
```

`provider status` shows four coverage rows before the detailed slots: Goal chat, clean brain, tentacle planning, and harness evolution. A local model or Codex OAuth path can be ready without an API key; use `provider check` or `preflight --live` for live proof.

Generate a provider matrix record before release testing. This also prepares missing `.octopus/providers/*.env` files:

```bash
octopus provider matrix
OCTOPUS_LOCAL_OK=1 octopus provider matrix run
octopus provider matrix check
```

`matrix run` only calls provider targets that you explicitly enable, such as `OCTOPUS_LOCAL_OK=1`; skipped or failed targets keep the release gate closed.

Then run the same Goal path live:

```bash
octopus first-run --live "make this repo easier to use"
octopus brain --goal --live --save "tighten the current objective"
```

## Observe

```bash
octopus doctor
octopus report
octopus preflight
octopus pet
```

These are observation surfaces. They should explain what the agent did without asking the user to drive tools directly.

## 0.1.0 Release Gate

```bash
tmp=$(mktemp -d)
octopus --state "$tmp/state.json" first-run "preflight local evidence"
octopus --state "$tmp/state.json" preflight
octopus benchmark record
# Fill .octopus/benchmark-evidence.md with SWE/Claw/Wild case ids, commands, pass results, summaries, and artifacts.
octopus benchmark check
octopus --state "$tmp/state.json" preflight record "$tmp/real-machine-record.md"
octopus --state "$tmp/state.json" preflight record check "$tmp/real-machine-record.md"
octopus --state "$tmp/state.json" preflight record append "$tmp/real-machine-record.md" docs/real-machine-test.md
```

`0.1.0` needs the local loop, live provider, benchmark evidence, GitHub OAuth/PR path, and real-machine record to pass.
