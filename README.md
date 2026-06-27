# Octopus 🐙

> Clean brain. Independent tentacles.

[Open the app](https://dangozhang.github.io/Octopus/app.html) · [Docs](https://dangozhang.github.io/Octopus/) · [Install guide](docs/quickstart.md)

Biological octopuses do not push every signal through one brain. Their arms contain local nervous systems. Behavior emerges from intent, local control, and feedback.

That biology is the product bet: high-level intent, distributed local control, and layered feedback can make agents faster to think with. See the [research map](docs/references.md).

Octopus brings that idea to agents. The main model carries only `Goal + Mem + Need + Feed`. It asks for cognition: verify this, reproduce that, compare options, remember a fact, forget stale context, execute a job, repair a broken path.

The edge does the work. Octopus turns tools into local nervous systems: an LLM prompt, tool metadata, runtime code, local grants, traces, and an editable harness. A tentacle chooses how to observe, execute, verify, and compress Feed without polluting the clean brain.

The runtime keeps three pulses alive: liveness, memory, and harness evolution. A pixel Octopus changes color from real work states, giving the agent a visible body instead of only logs.

```text
Goal -> Brain -> Need -> Tentacle Intelligence -> Action -> Feed -> Brain
Heartbeat -> Action Data -> Tentacle harness change
```

## Install & Launch

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus start --open
```

`start --open` prepares local state, seed tentacles, heartbeats, and the native app surface, then opens `http://127.0.0.1:8765/app.html`. Use `octopus start` on headless machines. If no source checkout is present, it creates editable seed tentacles under `.octopus/bundled-tentacles`.

Update later with `octopus update --run`.

Run a first loop:

```bash
octopus first-run "make this repo easier to use"
```

`first-run` sets a clean Goal, installs seed tentacles, asks a safe observe Need, records Feed feedback, pulses the hearts, and returns Doctor plus Preflight evidence. Add `--live` when provider env is ready and you want the same loop to include the live provider gate.

Use an OpenAI-compatible provider for live clean-brain calls and tool-side planning:

```bash
octopus provider save openai
source .octopus/llm.env
octopus provider check
octopus brain --brief --save "compress the clean brain state"
octopus brain --intent --save "what should the brain ask next?"
octopus brain --align --save "does this still follow the goal?"
octopus brain --scout --save "map what the brain should understand next"
octopus need verify .
```

Strong-model knobs such as `OCTOPUS_LLM_REASONING_EFFORT`, `OCTOPUS_LLM_MAX_TOKENS`, and `OCTOPUS_LLM_EXTRA_BODY` stay in provider env, outside Need text.

## Works Today

- Clean-brain Goal, Need, Feed, queue, context, intent, brief, focused Need kinds, agenda, scout, clarification, deliberation, reflection, alignment, memory, council, synthesis, and external-chat sessions.
- Seed tentacles for SWE work, computer-use adapters, repo maintenance, harness repair, and write-and-run local execution.
- Tool-side LLM planning with grants, traces, route scores, feedback scoring, and reviewable feedback-driven harness evolution.
- Native HTML app server, install reports, provider setup, preflight gates, and pixel pet SVG export.

Pre-release line: `0.0.15`. `0.1.0` waits for recorded real-machine testing.
