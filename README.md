# Octopus 🐙

Clean brain. Independent tentacles.

[Try the app](https://dangozhang.github.io/Octopus/app.html) · [5-minute use guide](https://dangozhang.github.io/Octopus/use.html) · [Docs](https://dangozhang.github.io/Octopus/) · [中文](README.zh-CN.md)

Biological octopuses use intent, local arm control, and feedback instead of routing every signal through one central brain.
Octopus brings that shape to agents: the brain expresses the Goal and cognitive Need; tentacles own implementation through their own prompt, tool metadata, code, permissions, traces, and evolution surface.

Tools become local nervous systems.
The main model stays clean.
The harness learns which tentacle can supply the best Feed.

```text
Goal -> Brain -> Need -> Tentacle Intelligence -> Action -> Feed -> Brain
Heartbeat -> Action Data -> Tentacle harness change
```

## Quick Install & Use

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus start --open
```

`start --open` prepares local state, editable seed tentacles, the profile registry, three beats, the pixel pet, and the native app, then opens `http://127.0.0.1:8765/app.html`.

```bash
octopus first-run "make this repo easier to use"
```

`first-run` sets a clean Goal, installs seed tentacles, asks one safe observe Need, records Feed feedback, pulses the hearts, and returns Doctor plus Preflight evidence.

Keep using it by changing Goal:

```bash
octopus chat "make this repo easier to use"
octopus goal refine "prefer small reviewable changes"
octopus brain --goal --save "tighten the current objective"
```

Need, Feed, tool choice, provider routing, repair, and harness evolution stay inside the agent. The product surface changes Goal; the harness handles supply.

## Works Now

- One public input path: Goal from CLI, app, or external chat.
- Seed tentacles for SWE work, computer-use adapters, repo maintenance, harness repair, write-and-run execution, JSON Feed, and the pixel pet.
- Tool-side LLM planning, grants, traces, route scores, and reviewable harness evolution.
- Native app server, provider/readiness observation, preflight gates, and pixel pet state.
- `start --check` writes local app run evidence for release preflight.
- LLM backends through Codex login, API-key clouds, local OpenAI-compatible models, or gateway routers.
- Repo-maintainer can probe local Codex CLI OAuth/API-key readiness and write Codex-backed maintenance reports for granted repos.

Read next: [5-minute use guide](docs/use.html), [Quick Install & Use](docs/quickstart.md), [Architecture](docs/architecture.md), [Research map](docs/references.md).

Pre-release line: `0.0.19`. `0.1.0` waits for recorded real-machine testing.
