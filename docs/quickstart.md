# Quick Install & Use

## Install

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus start --open
```

`start --open` opens `http://127.0.0.1:8765/app.html`. Use `octopus start` on headless machines.

`start` is the whole-project entry: it prepares `.octopus/` state, installs editable seed tentacles, pulses the three hearts, serves the native app and pixel pet, loads `.octopus/llm.env`, and falls back to bundled app/tentacle files when no source checkout is present.

Update later:

```bash
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

Manual equivalent:

```bash
octopus bootstrap
octopus goal set --constraint "keep tools outside the brain" "make this repo easier to use"
octopus goal refine "prefer cognitive Needs over tool instructions"
octopus brain --session "what should the brain ask next?"
octopus brain --align --save "does this still follow the goal?"
octopus brain --scout --save "map what the brain should understand next"
octopus brain --focus verify --save "what proof matters?"
octopus need observe README.md
octopus feedback latest satisfied "useful evidence"
octopus beat 200
octopus pet
```

The app can continue the same loop without opening a shell: starter tentacles, provider setup, clean-brain sessions, Need Queue, Feed tests, repair plans, route feedback, preflight, and harness-beat review all run through `octopus start`.

## Optional LLM

Use whichever credential shape you already have.

Codex login/OAuth:

```bash
codex login
octopus provider save codex
source .octopus/llm.env
octopus provider check
```

Direct API key:

```bash
export OPENAI_API_KEY=...
octopus provider save openai
source .octopus/llm.env
# Optional when your model/provider supports explicit thinking controls:
# export OCTOPUS_LLM_REASONING_EFFORT=medium
# export OCTOPUS_LLM_MAX_TOKENS=2048
octopus provider check
```

Direct API key saved locally:

```bash
export ZAI_API_KEY=...
octopus provider save-key zai
source .octopus/llm.env
octopus provider check
```

Local OpenAI-compatible model:

```bash
octopus provider save lmstudio
source .octopus/llm.env
# edit .octopus/llm.env if your server/model id differs
octopus provider check
```

Any provider through a LiteLLM/OpenAI-compatible gateway:

```bash
# Start LiteLLM separately with your provider key and model routing.
octopus provider save litellm
source .octopus/llm.env
octopus provider check
```

Then run live Octopus flows:

```bash
octopus first-run --live "make this repo easier to use"
octopus brain --agenda --save "what matters next?"
octopus brain --focus compare --save "which path should the brain compare?"
octopus brain --llm-prefix OCTOPUS_LLM --agenda --save "what matters next?"
octopus brain --council --models OCTOPUS_LLM --save "ask clean brains"
octopus brain --live --save "what should the brain ask next?"
```

OpenAI-compatible profiles include OpenAI, local servers, LiteLLM, Codex CLI OAuth, Z.AI/BigModel, routers, DeepSeek, Groq, Gemini, DashScope, Moonshot, LM Studio, and custom endpoints.
The same provider env powers clean-brain calls, tool-side tentacle planning, and harness evolution.

## Harness Evolution

```bash
octopus install swe-agent
octopus need observe README.md
octopus feedback latest partial "feed needs sharper file evidence"
octopus beat 200
octopus repair .
```

Harness changes stay reviewable: plans, patches, repair bundles, and scores are written under `.octopus/` before any high-risk action.

## Release Gate

```bash
tmp=$(mktemp -d)
octopus --state "$tmp/state.json" first-run "preflight local evidence"
octopus --state "$tmp/state.json" preflight
octopus --state "$tmp/state.json" preflight record "$tmp/real-machine-record.md"
octopus --state "$tmp/state.json" preflight record check "$tmp/real-machine-record.md"
octopus --state "$tmp/state.json" preflight record append "$tmp/real-machine-record.md" docs/real-machine-test.md
```
