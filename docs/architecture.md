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

`src/octopus` is the Python SDK/prototype layer. It is useful for quick integrations, SDK tests, and LLM-tool experiments.

`OpenAiCompatibleChatClient`, `CodexCliChatClient`, and `OpenAICompatibleLLM` form the provider base. Octopus can use OpenAI-compatible clouds, Codex CLI OAuth, Z.AI/BigModel, local servers, LiteLLM gateways, routers, DeepSeek, Groq, Gemini, DashScope, Moonshot, LM Studio, and custom endpoints. Provider env can carry `{PREFIX}_REASONING_EFFORT`, `{PREFIX}_MAX_TOKENS`, `{PREFIX}_TEMPERATURE`, `{PREFIX}_TOP_P`, `{PREFIX}_RETRIES`, `{PREFIX}_RETRY_MS`, and `{PREFIX}_EXTRA_BODY`, so model controls stay outside Need text and clean-brain context. The request layer forces non-streaming chat completions, parses provider error JSON, adapts text/content-array/reasoning-only responses, retries transient failures, and supports local `curl` or Codex CLI runtime adapters. Provider setup commands remain runtime/developer plumbing; the product bridge exposes provider readiness as observation and does not let provider writes become the main user path.

Context boundary: clean-brain LLM context is `Goal + Mem + Need + Feed`; tentacle LLM context is `Need + Tool + Action + Tool + Action -> Feed`; harness evolution sees manifest surfaces, outcomes, checks, and constraints so it can modify prompt, metadata, runtime code, or policy without moving tool burden into the clean brain.

Biological analogy: octopus behavior combines high-level intent, distributed local control, and layered feedback. Octopus follows that pressure: the brain owns intent, attention, and evaluation; tentacles own local implementation. A stable Need such as `verify repo health` can route through bash-only, SWE, repo-maintainer, or another tentacle. Need describes what, tentacles choose how, and RouteBook learns who is good at what.

`bootstrap [tentacles-root]` creates the local state files, adapts to the current project, installs the built-in seed tentacles, pulses the heartbeat, and returns a product report plus next commands. It does not grant high-risk tool scopes.

`starter [objective]` reads profile, manifest metadata, and scored first-run choice feedback, then recommends grouped starter tentacles and evidence signals for that objective without installing or executing tools. Starter feedback is harness data, not a primary user control.

`doctor` reports local readiness: state, environment, manifests, installed tentacles, optional LLM config, pet page, self-iteration mode, warnings, and next actions.

`think <tentacle> <kind> <query>` asks one tentacle brain to plan from its prompt, tool metadata, grants, and optional LLM provider, then returns the selected tool and planned actions without executing them.

`context` prints clean-brain slots, memory summary, and recent Need/Feed turns so the separation is inspectable without running tools. `chat`, `goal set`, `goal refine`, and `brain --goal` are the user-writable brain-goal surface. Other clean-brain modes, Need Queue review, synthesis, reflection, and exploration exist as internal or developer flows; the product bridge blocks them as direct user writes. The stable rule is simple: external users may change Goal, while Need/Feed execution and harness changes are driven by the agent loop.

Every executed Feed is also written to a compact harness trace journal and returned with `feed_trace_index`. Manifest Feed carries a `tentacle_plan` evidence item, action metadata, and a short `feed_trace`, so tool-side thinking is visible after execution without adding tool burden to clean-brain context. LLM tentacle plans may execute up to two tool actions for one Need. `traces [limit]` reads that journal. Feed scoring updates route data and pet state as harness feedback, not as a required user-facing step.

`start [--open] [addr]` prepares local state, starts the native HTML app, overlays `.octopus/llm.env`, and exposes `/api/run` plus `/api/stream` through a narrow bridge. The bridge allows Goal writes through `chat`, `goal set/refine`, `brain --goal`, and `first-run [--live] [objective]`. Doctor, report, preflight, provider status/check, starter recommendations, traces, and pet state are observable. Need execution, Feed scoring, repair, evolve, OAuth grants, installs, checks, provider env writes, preflight record writes, and pet image writes stay internal or developer-only. Blocked bridge calls return `user_writes_brain_goal_only` with suggested Goal commands instead of a generic server error.

`update [--run]` reports the GitHub reinstall command by default and only executes it with `--run`. The local app API allows the dry-run report and blocks `--run`.

`skills [root]` lists profile and manifest skills as user-facing capability bundles. It is a catalog view; execution still starts from Need and routes through harness data.

`install <tentacle>` installs a profile or manifest, then reports the tentacle's needs, runtimes, required grants, evolution checks, recent check history, and next commands from its own metadata. It is an internal/developer operation, not the public brain-goal path.

`check <tentacle> [index]` runs all manifest/profile evolution checks or one 1-based check, returns per-command status, records compact check history in harness state, and updates pet state from the result. The product bridge keeps this behind the harness.

`beat [memory_keep]` pulses the three hearts. The harness beat now watches recent failed or partial check history, Feed traces, and repair outcomes; when a matching tentacle manifest exists, it writes the next evolution proposal, recommendation, and apply plan under `.octopus/evolution/`, then exposes the source signal, candidate, preview, next action, and review target in heartbeat data.

`harness-repair-agent` is the seed tentacle for that loop. Its tool-side LLM can inspect heartbeat/evolution/state diagnostics, probe adapter readiness, write a reviewable `.octopus/harness-repair/` session with `PROMPT.md`, `DRAFT.md`, `REVIEW.md`, `NEXT_NEED.json`, `COMMANDS.sh`, `OUTCOME_MEMORY.md`, `CODE_CONTEXT.md`, and `REPAIR_PLAN.json`, then return the next review/grant/apply/score Feed without adding tool logs to clean-brain context. `repair continue [query]` runs that repair Feed, takes the queued clean Need, routes it back through the harness, and returns the continued Feed plus trace-scoring commands. `heartbeat_repair` reads the newest `REPAIR_PLAN.json` first, so the beat can continue a repair session before falling back to older evolution artifacts. `OCTOPUS_REPAIR_LLM=1` lets the configured provider fill `DRAFT.md`; the default path stays offline. Reviewed session outcomes are recorded as `OUTCOME.md` plus `.octopus/harness-repair/outcomes.jsonl`, `repair score` mirrors app/CLI review into that journal when the scored Feed trace came from a repair session, and later sessions feed merged outcome memory plus target manifest/tool code into reviewable repair planning.

## Code-As-Harness

`tentacles/` contains editable harness code. A tentacle is LLM brain prompt, tool metadata, implementation code, and evolution policy. Each manifest declares evolution surfaces: `brain_prompt`, `tool_meta`, `runtime_code`, and `evolution_policy`. Agent tool-combo seeds cover SWE repo tools, computer-use tools with configurable MCP calls plus browser/window diagnostics and clipboard adapters, repo-maintainer tools, harness-repair diagnostics, and one transparent write-and-run harness. Runtime seeds such as `json-feed` prove the same contract can run beyond shell.

Tentacles can be segmented internally without exposing every local step to the main brain. A SWE tentacle can observe, hypothesize, patch, test, and explain inside its local harness, then return one compact Feed. Like segmented arm cords with local sucker maps, local modules may exchange feedback while the clean brain still receives only Goal, Mem, Need, and Feed.

`contract: octopus-json-v1` lets any runtime receive the same Need/tool/tentacle JSON envelope through stdin and return compact text or structured JSON feedback. Legacy executable entrypoints still work. Tools can declare `permission`; missing grants return `needs_authorization` before the entrypoint starts.

`scaffold <tentacle> [runtime]` creates a user-owned manifest and schema. Python, Node, shell, and HTTP get starter adapters; any other runtime gets a manifest-only `tools/feed` contract so the tentacle can add its own executable, MCP bridge, native binary, or remote adapter.

`probe <tentacle> <kind> <query>` runs one tentacle without saving state, making runtime contracts debuggable.

`evolve <tentacle> <objective>` writes an auditable draft under `.octopus/evolution/` listing the surfaces, prompt, metadata, code files, patch candidates, per-candidate patch drafts, checks, constraints, scored outcomes, recent Feed traces, and recent check history that can guide safe change. Runtime-code candidates use recent traces and failed checks to target the concrete tool entrypoint that produced or broke Feed. Patch candidates, drafts, and apply plans carry a `feedback focus` so the repair scope stays narrow. With `OCTOPUS_LLM_EVOLVE=1`, candidates are generated by the provider from harness surfaces, outcomes, traces, and check history, and may include a provider-assisted unified diff for the declared target. `evolve recommend <tentacle>` scores candidates from previous outcomes plus matching check history and writes the next apply plan. `evolve apply <tentacle> <candidate>` writes an apply plan; with `harness:write`, it also emits a reviewable `.patch` file without applying it. `evolve score <tentacle> <candidate> <status>` stores the outcome in harness state, updates route learning, and writes the next recommendation/apply plan from the updated feedback.

## Pixel Pet

Color change is a pixel pet layer. It can show heartbeat, memory beat, harness beat, blocked state, or success without changing `Need`, `Feed`, or `Feedback`. `pet image [state] [path]` exports the same pixel body as an SVG for chat, docs, or app surfaces.

`need`, `chat`, `feedback`, `beat`, and `evolve score` write the latest pet event into harness state. `pet` auto-selects a color from that event plus goal status. `pet [state]` can force `heartbeat`, `memory`, `harness`, `blocked`, or `success`; `--json pet` returns the chat fallback square, event source, event summary, and local `pet.html` URL for other UI shells.

## Boundary

Chat refines a `Goal`. Brains emit `Need`. Tentacles may think, plan, and call tools through `Tool`, `Planner`, or `ChatClient`. Harness evolves the feed path from data. The brain never carries tool burden.

See [Research Map](./references.md) for the papers shaping this design, [Self-Iteration Loop](./self-iteration.md) for repo improvement, and [Product Gap Log](./product-gap.md) for current gaps.
