# Tentacles

Each tentacle is code-as-harness: LLM brain prompt, tool metadata, code implementation, and evolution policy.

A tentacle is a runtime-neutral execution unit. Prompt, meta, code, and policy can evolve together while the clean brain keeps emitting only cognitive Needs.

`profile-registry/default.json` is the seed profile registry. It keeps starter prompts, tool metadata, checks, permissions, and evolution policy outside the Rust kernel source. `octopus start` and `octopus bootstrap` materialize an editable copy at `.octopus/profile-registry/default.json`, and `OCTOPUS_PROFILE_REGISTRY=/path/to/default.json` can point the kernel at another registry. This is a tentacle/harness authoring surface: users express goals, while registry edits change supply paths, permissions, checks, and evolution policy.

Use `octopus doctor`, `octopus report`, or `octopus preflight` to see which registry is active and whether it parses cleanly.

Every manifest declares four evolution surfaces:

- `brain_prompt`: tool-side LLM prompt and feedback contract.
- `tool_meta`: descriptions, inputs, outputs, permissions, and call contracts.
- `runtime_code`: executable adapter code in any runtime.
- `evolution_policy`: checks, constraints, and safe edit boundaries.

The core treats runtime as metadata plus an entrypoint contract. A product tentacle usually starts as an agent tool combination, then evolves prompt, meta, code, and policy together.

Developer/harness commands for inspecting installable manifests:

```bash
octopus manifests
octopus install computer-use-agent
octopus --json install computer-use-agent
tmp=$(mktemp -d)
(cd "$tmp" && octopus scaffold my-feed python)
(cd "$tmp" && octopus probe my-feed observe README.md)
(cd "$tmp" && octopus scaffold native-feed rust)
```

`install` reads manifest/profile metadata and prints needs, runtimes, grant commands, checks, and next commands. If a tentacle evolves its tool metadata or policy, this report changes with it.

Agent tool-combo tentacles:

- `swe-agent`: repo work through `read`, `edit`, inspection, patch, and test tools.
- `computer-use-agent`: local UI work through configurable MCP calls, local commands, screenshots, URLs, browser status, front-window status, clipboard adapters, and desktop probes.
- `repo-maintainer`: OAuth-bounded self-iteration with repo inspection, PR drafts, and explicit PR publishing.
- `harness-repair-agent`: heartbeat, evolution, state, adapter diagnostics, optional LLM repair drafts, and local repair outcome memory that return the next reviewable repair Feed.
- `bash-only`: transparent write-and-run harness for audit and replay.

Runtime seeds:

- `json-feed`: Python runtime that consumes `octopus-json-v1` and returns structured feedback.

Visual layer:

- `visual`: color-changing pixel pet for heartbeat, memory, harness, blocked, and success states.

Memory lives in `HarnessState` as a heart/beat. It is exposed in catalogs for installation and status, but it is not an agent tool-combo tentacle.

Shell is only one seed runtime. A tentacle can evolve prompt, metadata, runtime code, or policy into Python, TypeScript, Rust, MCP, HTTP, native tools, or mixed runtimes while the kernel contract stays stable. Unknown runtimes scaffold as manifest-only and become installable after the tentacle adds its executable adapter.

Use `contract: octopus-json-v1` when a tool wants the full Need/tool/tentacle JSON envelope on stdin. Use `stdio-argv-v1` for argv/stdin adapters, `adapter-v1` for abstract adapters, `static-html-v1` for pet/page artifacts, and `native-harness-v1` for built-in harness surfaces.

Use `permission` when a tool needs an explicit grant before execution. Example: `octopus oauth octopus tool:bash-only tool:execute`.

Computer-use MCP calls are adapter-neutral. Set `OCTOPUS_MCP_<SERVER>_COMMAND` or `OCTOPUS_MCP_COMMAND` to a client command that reads JSON-RPC on stdin, then grant `octopus oauth octopus tool:computer-use-agent tool:mcp`.

Computer-use clipboard tools are grant-bound. Use `tool:observe` for `clipboard_read` and `tool:ui` for `clipboard_write`; combine permissions for the same scope in one `octopus oauth` command. Set `OCTOPUS_CLIPBOARD_DRY_RUN=1` when testing without touching the real clipboard.
