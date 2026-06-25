# Octopus

> Independent-thinking tools, clean agent brain.

## Why

Agents still carry tool burden in the brain. Octopus moves execution intelligence into tentacles and lets data decide the feed.

## What it does

- Clean brain: emits cognitive needs only.
- Tentacle brain: tools can plan, call sub-LLMs, execute, and return evidence.
- Three hearts: heartbeat, memory evolution, harness route evolution.
- Color change: image interaction stays outside the kernel.
- Kernel: Rust. SDK/prototype: Python.

## Quick start

```bash
cargo test
cargo run -p octopus-core -- need remember "tools stay outside the brain"
pip install -e .
```

## Example output

`remembered m1`

Mechanism: `Need -> Feed -> Feedback`. The model asks for cognition; the harness supplies evidence.

## Proof

Core: `crates/octopus-core`. SDK: `src/octopus`. Docs: `docs`. License: MIT.
