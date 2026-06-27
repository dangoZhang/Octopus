# Octopus 🐙

> Independent-thinking tools for agents with a clean brain.

[Open the app](https://dangozhang.github.io/Octopus/app.html) · [Docs](https://dangozhang.github.io/Octopus/) · [Install guide](docs/quickstart.md)

Most agents make one model do everything: keep the goal, remember context, read tool manuals, choose commands, parse logs, and repair the system that just failed. The brain gets crowded before the work gets hard.

Octopus keeps the center quiet. The main model carries only `Goal + Mem + Need + Feed`. It asks for cognition: verify this, reproduce that, compare options, remember a fact, forget stale context, execute a job, repair a broken path. It does not need to know which tool, API, file, shell command, or desktop adapter will be used.

The edge does the work. A tentacle is an LLM prompt, tool metadata, runtime code, local grants, traces, and an editable harness surface. It can think while using tools, return compact Feed, and improve from feedback without dragging implementation burden back into the main brain.

The system has three pulses: one heartbeat for liveness, one memory beat for context, and one harness beat for tool-side evolution. The pixel Octopus changes color from real work states, so the agent has a small visible body instead of only logs.

## Quick Install & Use

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus bootstrap
octopus starter "make this repo easier to use"
octopus bridge
```

Open `http://127.0.0.1:8765/app.html`.

Run the first clean loop:

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

Current line: `0.0.11`. `0.1.0` waits for recorded real-machine testing.
