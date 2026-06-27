# Octopus 🐙

> A local agent harness for a clean brain and tools that can think.

Most agents ask one model to carry the whole room: goal, memory, tool schemas, shell output, route choices, repair notes, and every failed attempt. Octopus keeps the head light. The main model asks for the evidence it needs; tool-side tentacles plan, run, remember, and return compact Feed.

That gives you a small loop with a real product surface: ask, feed evidence back, score the result, then let the harness learn without dragging tool clutter into the next thought.

## Why It Feels Different

- 🐙 The brain sees only `Goal + Mem + Need + Feed`.
- 🧠 Tentacles carry their own prompt, tool metadata, runtime code, grants, and repair trail.
- 🫀 Three beats keep the system alive: liveness, memory compaction, and harness improvement.
- 🎨 The pixel Octopus changes color from real Need, Feed, beat, repair, and score events.
- 🔐 Local grants gate risky tool actions before execution.
- ⚙️ A Rust kernel keeps the non-evolvable base small, typed, and fast.

## Quick Install & Use

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus --version
octopus bootstrap
octopus bridge
```

Open `http://127.0.0.1:8765/app.html`.

Try the loop:

```bash
octopus goal set "make this repo easier to use"
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

Add a live OpenAI-compatible provider when you want model-backed brain or tentacle planning:

```bash
octopus provider save openai
source .octopus/llm.env
octopus provider check
octopus brain --live --save "what should the brain ask next?"
```

## What Runs Today

- Bootstrap a local state, seed agent-style tentacles, and open the native HTML app.
- Use SWE, computer-use, repo-maintainer, harness-repair, bash-only, and JSON Feed seeds.
- Inspect the clean-brain context before any model call.
- Import external chat replies into Goal or Need Queue without running tools.
- Let tentacles plan from their own manifest, grants, tools, and optional LLM.
- Store Feed traces, score outcomes, update routing, and drive pet color.
- Generate reviewable harness repair sessions and score repair outcomes.
- Gate future releases with `octopus preflight`, generated scripts, and real-machine records.

## Proof

```text
Octopus preflight
target: 0.1.0
version: 0.0.8
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
