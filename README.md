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
cargo run -p octopus-core -- need remember "tools stay outside the brain"
cargo run -p octopus-core -- need recall tools
```

Output:

```text
remembered m1
tools stay outside the brain
```
