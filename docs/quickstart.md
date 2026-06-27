# Quick Install & Use

## Install

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus start
```

Open `http://127.0.0.1:8765/app.html`.

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
octopus start
```

## First Loop

Run these from the project you want Octopus to inhabit:

```bash
octopus goal set "make this repo easier to use"
octopus brain --session "what should the brain ask next?"
octopus need observe README.md
octopus feedback 1 satisfied "useful evidence"
octopus beat 200
octopus pet
```

The app can continue the same loop without opening a shell: starter tentacles, provider setup, clean-brain sessions, Need Queue, Feed tests, repair plans, route feedback, preflight, and harness-beat review all run through `octopus start`.

## Optional LLM

```bash
octopus provider save openai
source .octopus/llm.env
octopus provider check
octopus brain --agenda --save "what matters next?"
octopus brain --live --save "what should the brain ask next?"
```

OpenAI-compatible profiles include OpenAI, local servers, routers, DeepSeek, Groq, Gemini, DashScope, Moonshot, LM Studio, and custom endpoints.

## Harness Evolution

```bash
octopus install swe-agent
octopus need observe README.md
octopus feedback 1 partial "feed needs sharper file evidence"
octopus beat 200
octopus repair .
```

Harness changes stay reviewable: plans, patches, repair bundles, and scores are written under `.octopus/` before any high-risk action.
