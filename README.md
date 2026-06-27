# Octopus 🐙

> Build agents that keep the main brain clear without making tools dumb.

Most agents make one model carry everything: the goal, memory, tool manuals, shell noise, repair notes, and every failed attempt. Octopus gives the main brain a clean desk. It says what evidence it needs; independent tentacles decide how to get it, run the right tools, remember the result, and send back compact Feed.

The trick is small but sharp: the core thought loop stays `Goal + Mem + Need + Feed`; tool work lives inside tentacles with their own prompt, tool metadata, runtime code, permissions, and repair trail.

## Why

The clean brain should not memorize every tool manual before it can think. Octopus keeps that burden in the harness, where it can be tested, scored, repaired, and evolved as code.

## What it does

- 🐙 Keeps the brain on `Goal + Mem + Need + Feed`, so thinking stays inspectable.
- 🧠 Lets tentacles plan with their own LLM prompt, tool metadata, runtime code, permissions, and repair trail.
- 🛠️ Ships seed tentacles for SWE, computer-use, repo-maintainer, harness-repair, bash-only, and JSON Feed.
- 🫀 Runs three beats: heartbeat, memory compaction, and harness improvement from scored feedback.
- 🎨 Shows a pixel Octopus that changes color from real Need, Feed, repair, beat, blocked, and success events.
- 🔐 Stops risky tools with `needs_authorization` until a local grant exists.
- ⚙️ Uses a small Rust base while leaving code-as-harness easy for Octopus to inspect and evolve.

## Quick Install & Use

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus bootstrap
octopus starter "make this repo easier to use"
octopus bridge
```

Open `http://127.0.0.1:8765/app.html`.

Run the first local loop:

```bash
octopus goal set "make this repo easier to use"
octopus brain --session "what should the brain ask next?"
octopus context observe .
octopus need observe README.md
octopus feedback 1 satisfied "useful evidence"
octopus routes observe .
octopus beat 200
octopus pet
```

Add an OpenAI-compatible provider only when you want live clean-brain or tentacle planning:

```bash
octopus provider save openai
source .octopus/llm.env
octopus provider check
octopus brain --live --save "what should the brain ask next?"
```

## Example output

```text
Octopus preflight
target: 0.1.0
version: 0.0.10
brain: Goal + Mem + Need + Feed
tentacle: Need + Tool + Action + Tool + Action -> Feed
feed_trace_index: 1
pixel: 🟥
status: Satisfied
```

## Proof

Octopus can bootstrap a local state, recommend starter tentacles, run Feed through manifest tools, inspect the exact clean-brain context, score Feed and repair outcomes, keep route evidence, write reviewable harness-evolution patches, and expose the whole loop through a local HTML app.

- Runtime: Rust CLI plus local HTML app and bridge.
- Checks: CI covers install path, manifests, app pages, provider surfaces, Feed traces, and seed tentacle flows.
- Current line: `0.0.10`; `0.1.0` waits for recorded real-machine release testing.
- License: MIT.

## Links

- App: [docs/app.html](docs/app.html)
- Pixel Octopus: [docs/pet.html](docs/pet.html)
- Install guide: [docs/quickstart.md](docs/quickstart.md)
- Architecture: [docs/architecture.md](docs/architecture.md)
- Real-machine gate: [docs/real-machine-test.md](docs/real-machine-test.md)
- Tentacle contract: [tentacles/README.md](tentacles/README.md)
