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

`OpenAiCompatibleChatClient` in Rust and `OpenAICompatibleLLM` in Python adapt any chat-completions-compatible provider through `OCTOPUS_LLM_MODEL`, `OCTOPUS_LLM_BASE_URL`, and `OCTOPUS_LLM_API_KEY`. The Rust adapter keeps the kernel light by using a `curl` runtime adapter. Set `OCTOPUS_CHAT_LLM=1` to let chat refine the active goal and suggest cognitive needs without choosing tools. Set `OCTOPUS_LLM_MANIFEST=1` to let installed manifest tentacles ask the provider to pick tools before execution; failures fall back to rule planning. `OCTOPUS_CHAT_LLM_PREFIX` and `OCTOPUS_MANIFEST_LLM_PREFIX` can route those two layers to different provider env groups.

`demo [repo]` runs a source-local end-to-end loop: adapt, install, chat, feed, probe, heartbeat, self-iteration mode, and pet link.

`doctor` reports local readiness: state, environment, manifests, installed tentacles, optional LLM config, pet page, self-iteration mode, warnings, and next actions.

`skills [root]` lists profile and manifest skills as user-facing capability bundles. It is a catalog view; execution still starts from Need and routes through harness data.

## Code-As-Harness

`tentacles/` contains editable harness code. A tentacle is LLM brain prompt, tool metadata, implementation code, and evolution policy. Each manifest declares evolution surfaces: `brain_prompt`, `tool_meta`, `runtime_code`, and `evolution_policy`. Current seeds cover SWE repo tools, computer-use tools, a Python JSON feed, and one transparent script-writing runner; the contract is runtime-neutral.

`contract: octopus-json-v1` lets any runtime receive the same Need/tool/tentacle JSON envelope through stdin and return compact text or structured JSON feedback. Legacy executable entrypoints still work.

`scaffold <tentacle> [runtime]` creates a user-owned manifest and schema. Python, Node, shell, and HTTP get starter adapters; any other runtime gets a manifest-only `tools/feed` contract so the tentacle can add its own executable, MCP bridge, native binary, or remote adapter.

`probe <tentacle> <kind> <query>` runs one tentacle without saving state, making runtime contracts debuggable.

`evolve <tentacle> <objective>` writes an auditable draft under `.octopus/evolution/` listing the surfaces, prompt, metadata, code files, patch candidates, per-candidate patch drafts, checks, and constraints that can change safely. `evolve apply <tentacle> <candidate>` writes an apply plan; with `harness:write`, it also emits a reviewable `.patch` file without applying it. `evolve score <tentacle> <candidate> <status>` stores the outcome in harness state so later proposals can see what worked.

## Color Pet

Color change is a visual pet/status layer. It can show heartbeat, memory beat, harness beat, blocked state, or success without changing `Need`, `Feed`, or `Feedback`.

`pet` auto-selects a color from current status. `pet [state]` can force `heartbeat`, `memory`, `harness`, `blocked`, or `success`; `--json pet` makes the state usable by other UI shells.

## Boundary

Chat refines a `Goal`. Brains emit `Need`. Tentacles may think, plan, and call tools through `Tool`, `Planner`, or `ChatClient`. Harness evolves the feed path from data. The brain never carries tool burden.

See [Research Map](./references.md) for the papers shaping this design, [Self-Iteration Loop](./self-iteration.md) for repo improvement, and [Product Gap Log](./product-gap.md) for current gaps.
