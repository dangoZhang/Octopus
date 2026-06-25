# Architecture

Octopus keeps the non-evolvable base small.

## Kernel

`crates/octopus-core` owns the stable contract:

- `Goal`: active objective that chat can keep refining.
- `Need`: cognitive demand with no tool name.
- `Feed`: supplied evidence from harness or tentacles.
- `Feedback`: compact result for the brain.
- `RouteBook`: data-driven routing that learns from feed quality.
- `HarnessState`: persistent memory and routes.
- `HeartbeatReport`: three-heart pulse for liveness, memory compaction, and route evolution.
- `StatusReport`: read-only doctor view for hearts, installed tentacles, grants, goal, warnings, and next action.
- `ManifestTentacle`: installed manifests become feed providers through runtime adapters and return plan metadata for supported needs.
- `PlanningTentacle`: tool-side planner that selects tools before execution.
- `ChatPlanner`: provider-neutral chat adapter for LLM-backed tentacle brains.
- `SkillManifest` and `TentacleProfile`: installable behavior bundles with brain, tool metadata, implementation pointers, and evolution policy.
- `EnvironmentReport`: local environment detection for adaptive defaults.
- `adapt`: installs recommended profiles and matching tentacle manifests from local environment signals.

Rust is the kernel language because this layer must be fast, typed, portable, and boring. It is the part we do not want an adaptive harness to rewrite casually.

## SDK

`src/octopus` is the Python SDK/prototype layer. It is useful for quick integrations, demos, and LLM-tool experiments.

`OpenAiCompatibleChatClient` in Rust and `OpenAICompatibleLLM` in Python adapt any chat-completions-compatible provider through `OCTOPUS_LLM_MODEL`, `OCTOPUS_LLM_BASE_URL`, and `OCTOPUS_LLM_API_KEY`. The Rust adapter keeps the kernel light by using a `curl` runtime adapter.

## Code-As-Harness

`tentacles/` contains editable harness code. A tentacle is LLM brain prompt, tool metadata, implementation code, and evolution policy. Initial profiles include SWE-style `read/edit`, computer-use `mcp/bash`, and bash-only as one transparent seed runtime.

## Color Pet

Color change is a visual pet/status layer. It can show heartbeat, memory beat, harness beat, blocked state, or success without changing `Need`, `Feed`, or `Feedback`.

## Boundary

Chat refines a `Goal`. Brains emit `Need`. Tentacles may think, plan, and call tools through `Tool`, `Planner`, or `ChatClient`. Harness evolves the feed path from data. The brain never carries tool burden.

See [Research Map](./references.md) for the papers shaping this design and [Self-Iteration Loop](./self-iteration.md) for the future repo-improvement path.
