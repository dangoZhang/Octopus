# Octopus

> Independent-thinking tools, clean agent brain.

## Insight

Most agents make the brain carry tools, memory, skills, and execution flow. Octopus keeps the brain clean: it only says what cognition it needs.

Tentacles think during execution. A tentacle is an LLM brain prompt, tool metadata, code implementation, and evolution policy. The harness learns which feed works from data.

Initial tentacles are editable code-as-harness: SWE (`read/edit`), computer-use (`mcp/bash`), and one transparent bash-only seed.

Shell is only a seed runtime. A tentacle can declare `contract: octopus-json-v1` and run Python, Node, MCP, HTTP, native code, or any custom adapter.

Set `OCTOPUS_CHAT_LLM=1` for LLM goal refinement; set `OCTOPUS_LLM_MANIFEST=1` for LLM tool planning inside tentacles. Use `OCTOPUS_CHAT_LLM_PREFIX` or `OCTOPUS_MANIFEST_LLM_PREFIX` when each layer should use a different provider env.

Three hearts keep it alive: heartbeat, memory evolution, and harness route evolution. Color change is a visual pet/status layer outside the kernel.

The mechanism is `Need -> Feed -> Feedback`. The outcome is less tool burden and stronger tools.

## Quick Install & Use

```bash
cargo install --git https://github.com/dangoZhang/Octopus --locked --package octopus-core --bin octopus
octopus demo dangoZhang/Octopus
octopus init
octopus install swe-agent
octopus chat "build a clean-brain agent"
octopus need observe .
octopus doctor
octopus beat 200
tmp=$(mktemp -d)
(cd "$tmp" && octopus scaffold my-feed python)
(cd "$tmp" && octopus probe my-feed observe README.md)
```

Output includes:

```text
Octopus demo
Octopus init
== project ==
Octopus doctor
heartbeat: alive
memory: compacted 0 memories
harness: evolved 2 routes
status: Satisfied
```

Full walkthrough: [docs/quickstart.md](docs/quickstart.md).
Tentacle contract: [tentacles/README.md](tentacles/README.md).
