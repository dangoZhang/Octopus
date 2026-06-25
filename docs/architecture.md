# Architecture

Octopus keeps the non-evolvable base small.

## Kernel

`crates/octopus-core` owns the stable contract:

- `Need`: cognitive demand with no tool name.
- `Feed`: supplied evidence from harness or tentacles.
- `Feedback`: compact result for the brain.
- `RouteBook`: data-driven routing that learns from feed quality.
- `HarnessState`: persistent memory and routes.
- `PlanningTentacle`: tool-side planner that selects tools before execution.

Rust is the kernel language because this layer must be fast, typed, portable, and boring. It is the part we do not want an adaptive harness to rewrite casually.

## SDK

`src/octopus` is the Python SDK/prototype layer. It is useful for quick integrations, demos, and LLM-tool experiments.

## Boundary

Brains emit needs. Tentacles may think, plan, and call tools through `Tool` and `Planner`. Harness evolves the feed path from data. The brain never carries tool burden.

See [Research Map](./references.md) for the papers shaping this design and [Self-Iteration Loop](./self-iteration.md) for the future repo-improvement path.
