# Octopus 🐙

> A clean-brain agent harness for tools that can think.

Agents usually make one model carry the whole room: goal, memory, tool schemas, shell output, route choices, repair notes, and every failed attempt. Octopus keeps the main model light. It asks for the evidence it needs; independent tentacles decide how to gather it, run the tools, remember what happened, and return compact Feed.

The loop stays small enough to inspect and strong enough to grow: ask, feed evidence back, score the result, then let the harness improve without dragging tool clutter into the next thought.

## Why It Lands

- 🐙 The brain sees only `Goal + Mem + Need + Feed`.
- 🧠 Each tentacle owns its prompt, tool metadata, runtime code, grants, and repair trail.
- 🫀 Three beats keep liveness, memory compaction, and harness improvement moving.
- 🎨 The pixel Octopus changes color from real Need, Feed, beat, repair, and score events.
- 🔐 Local grants stop risky tool actions before execution.
- ⚙️ The Rust base stays small, typed, fast, and hard to accidentally mutate.

## Quick Install & Use

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus --version
octopus bootstrap
octopus starter "make this repo easier to use"
octopus bridge
```

Open `http://127.0.0.1:8765/app.html`.

Run the first local loop:

```bash
octopus goal set "make this repo easier to use"
octopus starter "make this repo easier to use"
octopus brain "what should the brain ask next?"
octopus explore --save "what should the brain ask next?"
octopus needs
octopus context observe .
octopus need observe README.md
octopus repair .
octopus repair score 1 satisfied "repair improved harness"
octopus feedback 1 satisfied "useful evidence"
octopus routes observe .
octopus beat 200
octopus pet
```

Add an OpenAI-compatible provider when you want live clean-brain or tentacle planning:

```bash
octopus provider save openai
source .octopus/llm.env
octopus provider check
octopus brain --live --save "what should the brain ask next?"
```

## Use It Today

- Bootstrap local state and open the native HTML app.
- Start from common agent tool sets: SWE, computer-use, repo-maintainer, harness-repair, bash-only, and JSON Feed.
- Inspect exactly what the clean brain will see before any model call.
- Import external chat replies into Goal or Need Queue without executing tools.
- Let tentacles plan with their own manifest, grants, tools, and optional LLM.
- Score Feed and repair outcomes so routing and harness beats learn from use.
- Generate reviewable release gates with `octopus preflight`, scripts, and real-machine records.

## Current Line

```text
Octopus preflight
target: 0.1.0
version: 0.0.10
brain: Goal + Mem + Need + Feed
tentacle: Need + Tool + Action + Tool + Action -> Feed
next: octopus preflight record, octopus preflight script, octopus report
```

Status: pre-`0.1`; `0.1.0` is reserved for the first release-ready build with recorded real-machine testing.

## Links

- App: [docs/app.html](docs/app.html)
- Pixel Octopus: [docs/pet.html](docs/pet.html)
- Install guide: [docs/quickstart.md](docs/quickstart.md)
- Architecture: [docs/architecture.md](docs/architecture.md)
- Real-machine gate: [docs/real-machine-test.md](docs/real-machine-test.md)
- Tentacle contract: [tentacles/README.md](tentacles/README.md)
- License: MIT
