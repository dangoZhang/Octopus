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
octopus bootstrap
octopus starter "make this repo easier to use"
octopus bridge
```

Open `http://127.0.0.1:8765/app.html`.

Run a first loop:

```bash
octopus goal set "make this repo easier to use"
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
octopus brain --deliberate --save "think before the next Need"
octopus brain --synthesize --apply-json '{"summary":"model jury","drafts":[{"source":"model-a","needs":[{"kind":"verify","query":"whether the goal is satisfied"}]}]}' --save "merge model drafts"
octopus brain --live --save "what should the brain ask next?"
```

Set `OCTOPUS_BRAIN_DELIBERATE_LLM_PREFIX`, `OCTOPUS_BRAIN_SYNTHESIZE_LLM_PREFIX`, `OCTOPUS_BRAIN_GOAL_LLM_PREFIX`, `OCTOPUS_BRAIN_EXPLORE_LLM_PREFIX`, `OCTOPUS_BRAIN_REWRITE_LLM_PREFIX`, or `OCTOPUS_BRAIN_QUEUE_LLM_PREFIX` to route clean-brain jobs to different models.

## Works Today

- Clean-brain Goal, Need, Feed, queue, context, deliberation, synthesis, and external-chat sessions.
- Seed tentacles for SWE work, computer-use adapters, repo maintenance, harness repair, and write-and-run local execution.
- Tool-side LLM planning with grants, traces, route scores, feedback scoring, and reviewable harness evolution drafts.
- Native HTML app, local bridge, install reports, provider setup, preflight gates, and pixel pet SVG export.

Pre-release line: `0.0.11`. `0.1.0` waits for recorded real-machine testing.
