# Octopus

> Independent-thinking tools, clean agent brain.

## Story

Most agents make the brain carry tools, memory, skills, and execution flow. Octopus keeps the brain clean: it only says what cognition it needs.

Tentacles think during execution. They can plan, call sub-LLMs, use tools, and return compact evidence. The harness learns which feed works from data.

Three hearts keep it alive: heartbeat, memory evolution, and harness route evolution. Color change can become an image UI layer, outside the kernel.

The mechanism is `Need -> Feed -> Feedback`; the product promise is less tool burden, stronger tools.

## Quick Start

```bash
cargo test
tmp=$(mktemp -d)
cargo run -p octopus-core -- --state "$tmp/state.json" --lang zh need remember "工具不进大脑"
cargo run -p octopus-core -- --state "$tmp/state.json" need recall 工具
cargo run -q -p octopus-core --example thinking_tentacle
```

Output:

```text
已记住 m1
工具不进大脑
verified: the brain does not name tools
plan: selected first matching tool
```
