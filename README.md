# Octopus

> Independent-thinking tools, clean agent brain.

## Insight

Most agents make the brain carry tools, memory, skills, and execution flow. Octopus keeps the brain clean: it only says what cognition it needs.

Tentacles think during execution. A tentacle is an LLM brain prompt, tool metadata, code implementation, and evolution policy. The harness learns which feed works from data.

Initial tentacles are editable code-as-harness: SWE (`read/edit`), computer-use (`mcp/bash`), and bash-only as one transparent seed runtime.

Shell is only a seed runtime. A tentacle can declare `contract: octopus-json-v1` and run Python, Node, MCP, HTTP, native code, or any custom adapter.

Three hearts keep it alive: heartbeat, memory evolution, and harness route evolution. Color change is a visual pet/status layer outside the kernel.

The mechanism is `Need -> Feed -> Feedback`. The outcome is less tool burden and stronger tools.

## Quick Install & Use

```bash
cargo test
tmp=$(mktemp -d)
repo=$PWD
cargo run -q -p octopus-core -- demo dangoZhang/Octopus
cargo run -q -p octopus-core -- --state "$tmp/state.json" adapt
cargo run -q -p octopus-core -- --state "$tmp/state.json" install swe-agent
cargo run -q -p octopus-core -- --state "$tmp/state.json" need observe .
cargo run -q -p octopus-core -- --state "$tmp/state.json" doctor
cargo run -q -p octopus-core -- --state "$tmp/state.json" beat 200
(cd "$tmp" && cargo run -q --manifest-path "$repo/Cargo.toml" -p octopus-core -- scaffold my-feed python)
(cd "$tmp" && cargo run -q --manifest-path "$repo/Cargo.toml" -p octopus-core -- probe my-feed observe README.md)
```

Output includes:

```text
Octopus demo
== project ==
Octopus doctor
heartbeat: alive
memory: compacted 0 memories
harness: evolved 2 routes
status: Satisfied
```

Full walkthrough: [docs/quickstart.md](docs/quickstart.md).
Tentacle contract: [tentacles/README.md](tentacles/README.md).
