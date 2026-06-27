# Octopus 🐙

> Ship agents whose main brain stays readable while tools plan, act, and repair at the edge.

Most agent loops ask one model to think, remember tool manuals, choose commands, read noisy logs, and repair itself inside the same crowded context. Octopus splits that load. The main brain stays small: `Goal + Mem + Need + Feed`. Each tentacle carries its own LLM prompt, tool metadata, runtime code, grants, traces, and evolution policy, then returns compact Feed the brain can use.

## Why

Tool intelligence should live close to the work. Octopus lets the center ask for evidence, lets the arms decide how to get it, and lets feedback improve the harness without polluting the brain.

## What It Does

- 🐙 The brain asks for evidence, validation, comparison, memory, execution, or repair; it does not carry tool manuals.
- 🧠 Tentacles can think while using tools: SWE repo work, computer-use, repo maintenance, harness repair, bash-only scripts, and JSON Feed.
- 🫀 Heartbeat, memory beat, and harness beat keep runtime state, compact memory, and turn scored failures into reviewable improvements.
- 🎨 The pixel Octopus changes color from real Need, Feed, repair, beat, blocked, and success events: 🟥 🟨 🟩 🟦.
- 🔐 Risky tools stop at `needs_authorization` until a local grant exists.
- ⚙️ The base is small Rust; the evolving surface stays in readable manifests and code-as-harness.

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

Add an OpenAI-compatible provider when you want live clean-brain calls or tool-side planning:

```bash
octopus provider save openai
source .octopus/llm.env
octopus provider check
octopus brain --live --save "what should the brain ask next?"
```

Set `OCTOPUS_BRAIN_GOAL_LLM_PREFIX`, `OCTOPUS_BRAIN_EXPLORE_LLM_PREFIX`, `OCTOPUS_BRAIN_REWRITE_LLM_PREFIX`, or `OCTOPUS_BRAIN_QUEUE_LLM_PREFIX` to route clean-brain jobs to different models.

## Example Output

```text
brain: Goal + Mem + Need + Feed
tentacle: Need + Tool + Action + Tool + Action -> Feed
feed_trace_index: 1
pixel: 🟥
status: Satisfied
```

## What Works Today

Octopus can bootstrap local state, recommend starter tentacles with evidence, run Feed through manifest tools, inspect the exact clean-brain context, score Feed and repair outcomes, keep route evidence, write reviewable harness-evolution patches, and expose the loop through a local HTML app.

## Proof

- Runtime: Rust CLI plus local HTML app and bridge.
- Checks: CI covers install path, manifests, app pages, provider surfaces, Feed traces, and seed tentacle flows.
- Current line: `0.0.10`. `0.1.0` waits for recorded real-machine release testing.
- License: MIT.

## Links

- App: [docs/app.html](docs/app.html)
- Pixel Octopus: [docs/pet.html](docs/pet.html)
- Install guide: [docs/quickstart.md](docs/quickstart.md)
- Architecture: [docs/architecture.md](docs/architecture.md)
- Real-machine gate: [docs/real-machine-test.md](docs/real-machine-test.md)
- Tentacle contract: [tentacles/README.md](tentacles/README.md)
