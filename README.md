# Octopus 🐙

> Independent-thinking tools, clean agent brain.

🐙 clean brain · 🟢🐙 heartbeat · 🟣🐙 memory beat · 🟠🐙 harness beat

## Insight

Most agents make the brain carry tools, memory, skills, and execution flow. Octopus keeps the brain clean: chat refines the `Goal`, then the model only says what cognition it needs.

Tentacle brains give tools LLM intelligence. A tentacle is `LLM prompt + tool meta + code implementation + evolution policy`, and those surfaces can evolve as code-as-harness.

Initial agent tentacles are common tool combinations: SWE repo tools, computer-use tools with browser diagnostics, repo-maintainer tools, and a transparent write-and-run harness. Runtime seeds such as `json-feed` prove the same contract can run beyond shell.

Shell is only one runtime. A tentacle can declare `contract: octopus-json-v1` and run Python, Node, MCP, HTTP, native code, or any custom adapter.

Run `octopus providers` to generate OpenAI-compatible profiles, then `octopus provider check` to validate the live endpoint for chat, tentacle planning, and harness evolution.

Three hearts keep it alive: heartbeat, memory evolution, and harness route evolution. Color change is a pixel pet layer.

The mechanism is `Need -> Feed -> Feedback`. The outcome is less tool burden and stronger tools.

## Quick Install & Use

```bash
cargo install --git https://github.com/dangoZhang/Octopus --locked --package octopus-core --bin octopus
octopus --version
octopus providers
octopus doctor
octopus demo dangoZhang/Octopus
octopus init
octopus provider openai > .octopus/llm.env
# set OPENAI_API_KEY, source .octopus/llm.env, then run: octopus provider check
octopus skills
octopus install swe-agent
octopus install computer-use-agent
octopus install bash-only
octopus chat "build a clean-brain agent"
octopus need observe .
octopus oauth octopus tool:bash-only tool:execute
octopus need execute "echo octopus"
octopus pet
octopus doctor
octopus beat 200
octopus pet
tmp=$(mktemp -d)
(cd "$tmp" && octopus scaffold my-feed python)
(cd "$tmp" && octopus probe my-feed observe README.md)
```

Local app:

```bash
octopus bridge
```

Update:

```bash
cargo install --git https://github.com/dangoZhang/Octopus --locked --package octopus-core --bin octopus --force
```

Output includes:

```text
Octopus demo
Octopus init
json-feed observe
Octopus doctor
pixel: 🟥
event: harness beat
heartbeat: alive
memory: compacted 0 memories
harness: evolved 2 routes
status: Satisfied
```

Full walkthrough: [docs/quickstart.md](docs/quickstart.md).
Native HTML app: [docs/app.html](docs/app.html). Local bridge: `octopus bridge`.
Tentacle contract: [tentacles/README.md](tentacles/README.md).
