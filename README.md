# Octopus 🐙

> A clean-brain agent runtime where tools think at the edge.

[Open the app](https://dangozhang.github.io/Octopus/app.html) · [Docs](https://dangozhang.github.io/Octopus/) · [Install guide](docs/quickstart.md)

Most agents turn one model into a crowded operator: hold the goal, remember context, read tool manuals, choose commands, parse logs, and repair the adapters that just failed. The hard part starts after the brain is already full.

Octopus gives the center a smaller job. The main model carries only `Goal + Mem + Need + Feed`. It asks for cognition: verify this, reproduce that, compare options, remember a fact, forget stale context, execute a job, repair a broken path. Tool choice and implementation stay out of its context.

The edge does the work. A tentacle is an LLM prompt, tool metadata, runtime code, local grants, traces, and an editable harness. It can think while using SWE tools, computer-use tools, repo maintenance flows, or local scripts; then it returns compact Feed and improves from scored outcomes.

The runtime keeps three pulses alive: liveness, memory, and harness evolution. A pixel Octopus changes color from real work states, giving the agent a visible body instead of only logs.

## Install & Launch

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus start --open
```

`start --open` prepares local state, seed tentacles, heartbeats, and the native app surface, then opens `http://127.0.0.1:8765/app.html`. Use `octopus start` on headless machines. If no source checkout is present, it creates editable seed tentacles under `.octopus/bundled-tentacles`.

Update later with `octopus update --run`.

Run a first loop:

```bash
octopus goal set --constraint "keep tools outside the brain" "make this repo easier to use"
octopus brain --session "what should the brain ask next?"
octopus context observe .
octopus need observe README.md
octopus feedback 1 satisfied "useful evidence"
octopus beat 200
octopus pet
```

Use an OpenAI-compatible provider for live clean-brain calls and tool-side planning:

```bash
octopus provider save openai
source .octopus/llm.env
octopus provider check
octopus brain --llm-prefix OCTOPUS_LLM --agenda --save "what matters next?"
octopus brain --council --models OCTOPUS_LLM --save "ask clean brains"
octopus brain --agenda --save "what matters next?"
octopus brain --clarify --save "what should the user clarify?"
octopus brain --deliberate --save "think before the next Need"
octopus brain --reflect --save "what evidence is missing?"
octopus brain --memory --save "what should be remembered?"
octopus brain --live --save "what should the brain ask next?"
```

## Works Today

- Clean-brain Goal, Need, Feed, queue, context, agenda, clarification, deliberation, reflection, memory, council, synthesis, and external-chat sessions.
- Seed tentacles for SWE work, computer-use adapters, repo maintenance, harness repair, and write-and-run local execution.
- Tool-side LLM planning with grants, traces, route scores, feedback scoring, and reviewable harness evolution drafts.
- Native HTML app server, install reports, provider setup, preflight gates, and pixel pet SVG export.

Pre-release line: `0.0.13`. `0.1.0` waits for recorded real-machine testing.
