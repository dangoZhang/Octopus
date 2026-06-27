# Octopus 🐙

> Independent-thinking tools for agents with a clean brain.

[Open the app](https://dangozhang.github.io/Octopus/app.html) · [Read the docs](https://dangozhang.github.io/Octopus/) · [Install guide](docs/quickstart.md)

Agents get noisy when one model has to carry tool manuals, command syntax, logs, memory, permissions, and repair steps in the same context. Octopus moves that load to intelligent tentacles. The center asks for cognition. The edge gathers evidence, runs tools, learns from feedback, and returns compact Feed.

## The Agent Brain Got Too Loud

Octopus keeps the main context small: `Goal + Mem + Need + Feed`.

A Need is cognitive: verify, reproduce, compare, remember, forget, execute, or repair. It does not need to name tools, APIs, commands, files, or implementation steps. Tentacles own that burden through their own prompt, tool metadata, runtime code, grants, traces, and evolution policy.

## What Feels Different

- 🧠 Clean center: export, review, rewrite, and import Needs without dragging tool manuals into the main brain.
- 🐙 Thinking tentacles: SWE, computer-use, repo maintenance, harness repair, bash-only, and JSON Feed ship as editable agent tool suites.
- 🫀 Three pulses: heartbeat tracks liveness, memory beat compacts state, and harness beat turns failures into reviewable repairs.
- 🎨 Pixel state: the Octopus changes color from real Need, Feed, repair, beat, blocked, and success events.
- 🔐 Local grants: risky tools stop at `needs_authorization` until the local workspace allows them.
- ⚙️ Rust kernel, code-as-harness: the base stays small while the tool surface stays readable, testable, and replaceable.

## Download, Open, Run

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus bootstrap
octopus starter "make this repo easier to use"
octopus bridge
```

Open `http://127.0.0.1:8765/app.html`.

Run a first clean loop:

```bash
octopus goal set "make this repo easier to use"
octopus brain --session "what should the brain ask next?"
octopus context observe .
octopus need observe README.md
octopus feedback 1 satisfied "useful evidence"
octopus beat 200
octopus pet
```

Add an OpenAI-compatible provider when you want live clean-brain calls or tool-side planning:

```bash
octopus provider save openai
source .octopus/llm.env
octopus provider check
octopus brain --live --save "what should the brain ask next?"
```

## Usable Today

- Bootstrap local state and install seed tentacles.
- Recommend first-run tentacles and learn from accepted, ignored, or failed choices.
- Export clean-brain prompts, run external chat sessions, and rewrite polluted Needs before queueing them.
- Run manifest Feed through permissioned tools, record traces, score outcomes, and update route evidence.
- Inspect and operate the loop through the local HTML app and pixel Octopus.
- Generate harness evolution proposals and reviewable apply plans from failed Feed, checks, or repair outcomes.

## Pre-0.1 Line

Octopus is on the `0.0.x` line. The kernel, app, starter tentacles, provider bridge, Feed traces, feedback scoring, repair plans, and Pages docs are working. `0.1.0` waits for recorded real-machine release testing with live providers, desktop adapters, and PR publishing.

## Docs

- [Product app](docs/app.html)
- [Pixel Octopus](docs/pet.html)
- [Install guide](docs/quickstart.md)
- [Architecture](docs/architecture.md)
- [Real-machine gate](docs/real-machine-test.md)
- [Tentacle contract](tentacles/README.md)
