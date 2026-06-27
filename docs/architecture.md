# Architecture

Octopus keeps the non-evolvable base small.

## Kernel

`crates/octopus-core` owns the stable contract:

- `Goal`: active objective that chat can keep refining.
- `Need`: cognitive demand with no tool name.
- `Feed`: supplied evidence from harness or tentacles.
- `Feedback`: compact result for the brain.
- `RouteBook`: data-driven routing that learns from feed quality and scored Feed traces.
- `HarnessState`: persistent memory and routes.
- `HeartbeatReport`: three-heart pulse for liveness, memory compaction, and route evolution.
- `StatusReport`: read-only doctor view for hearts, installed tentacles, grants, goal, warnings, and next action.
- `ManifestTentacle`: installed manifests become feed providers through runtime adapters and return plan metadata for supported needs.
- `ToolPermission`: optional tool metadata that blocks execution until a matching grant exists.
- `PlanningTentacle`: tool-side planner that selects tools before execution.
- `ChatPlanner`: provider-neutral chat adapter for LLM-backed tentacle brains.
- `SkillManifest` and `TentacleProfile`: installable behavior bundles with brain, tool metadata, implementation pointers, and evolution policy.
- `EnvironmentReport`: local environment detection for adaptive defaults.
- `adapt`: installs recommended profiles and matching tentacle manifests from local environment signals.

Rust is the kernel language because this layer must be fast, typed, portable, and boring. It is the part we do not want an adaptive harness to rewrite casually.

## SDK

`src/octopus` is the Python SDK/prototype layer. It is useful for quick integrations, demos, and LLM-tool experiments.

`OpenAiCompatibleChatClient` in Rust and `OpenAICompatibleLLM` in Python adapt any chat-completions-compatible provider. `providers` lists built-in OpenAI-compatible profiles for OpenAI, local servers, routers, DeepSeek, Groq, Gemini, DashScope, Moonshot, LM Studio, and custom endpoints. `provider <name> [prefix]` emits shell env for chat goal refinement, clean-brain exploration, manifest tool planning, and harness evolution; `provider save <name> [prefix] [path]` writes the same env to `.octopus/llm.env` by default; `provider status` shows all four LLM layers without a network call; `provider check [prefix]` validates the live endpoint with the same adapter. The Rust adapter keeps the kernel light by using a `curl` runtime adapter. `OCTOPUS_CHAT_LLM_PREFIX`, `OCTOPUS_BRAIN_LLM_PREFIX`, `OCTOPUS_MANIFEST_LLM_PREFIX`, and `OCTOPUS_EVOLVE_LLM_PREFIX` can route those layers to different provider env groups.

Context boundary: clean-brain LLM context is `Goal + Mem + Need + Feed`; tentacle LLM context is `Need + Tool + Action + Tool + Action -> Feed`; harness evolution sees manifest surfaces, outcomes, checks, and constraints so it can modify prompt, metadata, runtime code, or policy without moving tool burden into the clean brain.

`demo [repo]` runs a source-local end-to-end loop: adapt, install, chat, feed, probe, heartbeat, self-iteration mode, and pet link.

`bootstrap [tentacles-root]` creates the local state files, adapts to the current project, installs the built-in seed tentacles, pulses the heartbeat, and returns a product report plus next commands. It does not grant high-risk tool scopes.

`doctor` reports local readiness: state, environment, manifests, installed tentacles, optional LLM config, pet page, self-iteration mode, warnings, and next actions.

`think <tentacle> <kind> <query>` asks one tentacle brain to plan from its prompt, tool metadata, grants, and optional LLM provider, then returns the selected tool and planned actions without executing them.

`context [kind query]` prints the clean-brain context slots, memory summary, recent Need/Feed turns, and installed tentacle tool/action context so the separation is inspectable without running tools. `brain [prompt]` exports pasteable chat messages built only from `Goal + Mem + Need + Feed`, so any model UI can act as the clean brain without seeing tools. `brain --session [--live] [--goal] [prompt]` writes a local external-chat session with prompt, messages, reply template, optional provider draft, and apply command. `brain --apply <file|->` and `brain --apply-json <json>` import that external chat reply into Goal or Need Queue without Feed execution. `brain --goal [--live] [--save] [prompt]` lets the clean brain refine Goal and optional queued Needs without Feed execution. `brain --live [prompt]` calls the clean-brain provider and returns suggested Needs without Feed execution; `brain --live --save [prompt]` writes those Needs into the reviewable queue. `explore [prompt]` reads only the clean-brain side and returns suggested Needs without writing Feed traces or route scores. `explore --save [prompt]` writes those suggested Needs into a reviewable Need Queue; `needs session [--live] [prompt]` writes a clean-brain review session for pending Needs, `needs take <index>` returns one `octopus need ...` command, and `needs script [path]` writes a reviewable script for all pending Needs without executing it.

Every executed Feed is also written to a compact harness trace journal and returned with `feed_trace_index`. Manifest Feed carries a `tentacle_plan` evidence item, action metadata, and a short `feed_trace`, so tool-side thinking is visible after execution without adding tool burden to clean-brain context. LLM tentacle plans may execute up to two tool actions for one Need. `traces [limit]` reads that journal. `feedback <trace-index> <status> [summary]` scores a trace, updates route data, and changes pet state.

`bridge [addr]` serves the native HTML app and exposes `/api/run` plus `/api/stream` for local Octopus subcommands so the GUI can execute init, bootstrap, provider env/status/check, install, check, chat, brain prompt export, external brain session creation, external brain reply apply, explore, Need Queue review, repair, repair outcome scoring, think, context, Need, Feed feedback, pet, doctor, report, preflight, preflight artifact, and beat flows without a shell. Bridge overlays `.octopus/llm.env` onto child Octopus commands. The app can generate or save provider env, read `--json provider status`, run live provider checks, bootstrap a local state, inspect clean-brain prompts and exploration, write external Chat session files with optional provider drafts, paste external chat replies into Goal or Need Queue, save suggested Needs, run harness repair into a queued Need, score the repair Feed, read `--json install` reports, run seed-tentacle checks, inspect clean-brain context, run a structured `--json need` Feed test, show plan source/tool/action/evidence for the resulting Feed, score that Feed trace, show harness-beat evolution recommendations with an apply-plan preview, grant `octopus:evolve:<seed>` `harness:write`, write reviewable apply artifacts, score the recommendation after review, write local preflight script/record artifacts, and grant local `octopus tool:*` scopes; bridge does not allow GitHub OAuth, direct source patch application, provider env writes outside the default path, custom preflight artifact paths, or PR publishing.

`skills [root]` lists profile and manifest skills as user-facing capability bundles. It is a catalog view; execution still starts from Need and routes through harness data.

`install <tentacle>` installs a profile or manifest, then reports the tentacle's needs, runtimes, required grants, evolution checks, recent check history, and next commands from its own metadata. `--json install <tentacle>` exposes the same report to the native HTML app or any outer shell.

`check <tentacle> [index]` runs all manifest/profile evolution checks or one 1-based check, returns per-command status, records compact check history in harness state, and updates pet state from the result. Bridge exposes it only for built-in seed tentacles.

`beat [memory_keep]` pulses the three hearts. The harness beat now watches recent failed or partial check history, Feed traces, and repair outcomes; when a matching tentacle manifest exists, it writes the next evolution proposal, recommendation, and apply plan under `.octopus/evolution/`, then exposes the source signal, candidate, preview, next action, and review target in heartbeat data.

`harness-repair-agent` is the seed tentacle for that loop. Its tool-side LLM can inspect heartbeat/evolution/state diagnostics, probe adapter readiness, write a reviewable `.octopus/harness-repair/` session with `PROMPT.md`, `DRAFT.md`, `NEXT_NEED.json`, and `COMMANDS.sh`, then return the next review/grant/apply/score Feed without adding tool logs to clean-brain context. `OCTOPUS_REPAIR_LLM=1` lets the configured provider fill `DRAFT.md`; the default path stays offline. Reviewed session outcomes are recorded as `OUTCOME.md` plus `.octopus/harness-repair/outcomes.jsonl`, and `repair score` mirrors app/CLI review into that journal when the scored Feed trace came from a repair session.

## Code-As-Harness

`tentacles/` contains editable harness code. A tentacle is LLM brain prompt, tool metadata, implementation code, and evolution policy. Each manifest declares evolution surfaces: `brain_prompt`, `tool_meta`, `runtime_code`, and `evolution_policy`. Agent tool-combo seeds cover SWE repo tools, computer-use tools with configurable MCP calls plus browser/window diagnostics and clipboard adapters, repo-maintainer tools, harness-repair diagnostics, and one transparent write-and-run harness. Runtime seeds such as `json-feed` prove the same contract can run beyond shell.

`contract: octopus-json-v1` lets any runtime receive the same Need/tool/tentacle JSON envelope through stdin and return compact text or structured JSON feedback. Legacy executable entrypoints still work. Tools can declare `permission`; missing grants return `needs_authorization` before the entrypoint starts.

`scaffold <tentacle> [runtime]` creates a user-owned manifest and schema. Python, Node, shell, and HTTP get starter adapters; any other runtime gets a manifest-only `tools/feed` contract so the tentacle can add its own executable, MCP bridge, native binary, or remote adapter.

`probe <tentacle> <kind> <query>` runs one tentacle without saving state, making runtime contracts debuggable.

`evolve <tentacle> <objective>` writes an auditable draft under `.octopus/evolution/` listing the surfaces, prompt, metadata, code files, patch candidates, per-candidate patch drafts, checks, constraints, scored outcomes, recent Feed traces, and recent check history that can guide safe change. Runtime-code candidates use recent traces and failed checks to target the concrete tool entrypoint that produced or broke Feed. Patch candidates, drafts, and apply plans carry a `feedback focus` so the repair scope stays narrow. With `OCTOPUS_LLM_EVOLVE=1`, candidates are generated by the provider from harness surfaces, outcomes, traces, and check history, and may include a provider-assisted unified diff for the declared target. `evolve recommend <tentacle>` scores candidates from previous outcomes plus matching check history and writes the next apply plan. `evolve apply <tentacle> <candidate>` writes an apply plan; with `harness:write`, it also emits a reviewable `.patch` file without applying it. `evolve score <tentacle> <candidate> <status>` stores the outcome in harness state so later proposals can see what worked.

## Pixel Pet

Color change is a pixel pet layer. It can show heartbeat, memory beat, harness beat, blocked state, or success without changing `Need`, `Feed`, or `Feedback`.

`need`, `chat`, `feedback`, `beat`, and `evolve score` write the latest pet event into harness state. `pet` auto-selects a color from that event plus goal status. `pet [state]` can force `heartbeat`, `memory`, `harness`, `blocked`, or `success`; `--json pet` returns the chat fallback square, event source, event summary, and local `pet.html` URL for other UI shells.

## Boundary

Chat refines a `Goal`. Brains emit `Need`. Tentacles may think, plan, and call tools through `Tool`, `Planner`, or `ChatClient`. Harness evolves the feed path from data. The brain never carries tool burden.

See [Research Map](./references.md) for the papers shaping this design, [Self-Iteration Loop](./self-iteration.md) for repo improvement, and [Product Gap Log](./product-gap.md) for current gaps.
