# Octopus 🐙

Clean brain. Independent tentacles.

[Open the app](https://dangozhang.github.io/Octopus/app.html) · [Docs](https://dangozhang.github.io/Octopus/) · [中文 README](README.zh-CN.md) · [中文文档](docs/zh/quickstart.md) · [Install guide](docs/quickstart.md)

Biological octopuses do not solve control by pushing every signal through one brain.
Their arms contain local nervous systems.
Their behavior emerges from intent, local control, and feedback.
Their hearts beat for blood and environment adaptation.

Octopus brings that idea to agents.
The brain only owns the goal and the need.
Tentacles with Intelligence own the implementation.
The heartbeat is for self-motivated work and self-evolving harness change.

Octopus Loop:

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

The public writing surface is the Goal:

```bash
octopus chat "make this repo easier to use"
octopus goal refine "prefer small reviewable changes"
octopus brain --goal --save "tighten the current objective"
```

Need, Feed, tool choice, provider routing, repair, and harness evolution are agent-internal. The app and CLI may show them as observable state, but user input only changes brain-goal. Live backends can be Codex login, API keys, local OpenAI-compatible models, or LiteLLM-style gateways through environment config.

## Works Today

- One public input path: Goal from CLI, app, or external chat.
- Seed tentacles for SWE work, computer-use adapters, repo maintenance, harness repair, and write-and-run local execution.
- Tool-side LLM planning, grants, traces, route scores, and reviewable feedback-driven harness evolution.
- Native app server, provider/readiness observation, preflight gates, and pixel pet state.

Pre-release line: `0.0.15`. `0.1.0` waits for recorded real-machine testing.
