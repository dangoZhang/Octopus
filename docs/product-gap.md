# Product Gap Log

Updated: 2026-06-26

## Current Shape

- Clean brain emits cognitive `Need`; it does not choose tools.
- Tentacles own LLM prompt, tool metadata, runtime code, and evolution policy.
- Harness stores memory, route scores, OAuth grants, installed tentacles, and goal state.
- Three beats exist: heartbeat, memory compaction, and harness route evolution.
- Color pet exposes heartbeat, memory, harness, blocked, and success states through deterministic state mapping.
- Initial agent tentacles are common tool combinations: SWE repo tools, computer-use tools, repo-maintainer, and a write-and-run harness.
- `json-feed` is a runtime seed for the `octopus-json-v1` contract.
- Memory is a heart/beat; visual is the color-changing pet layer.
- LLM adapters support OpenAI-compatible providers for chat goal refinement and manifest tool planning.

## Filled This Iteration

- Clarified the tentacle model as runtime-neutral `LLM + tool meta + code + policy`.
- Moved seed manifests away from `tools/*.sh` as the conceptual evolution surface.
- Authorized `evolve apply` now emits a reviewable `.patch` file and leaves application to the grant-bound outer flow.
- Added stale patch cleanup when authorization is absent.
- Added a built-in non-shell `json-feed` runtime seed that consumes `octopus-json-v1`.
- Added `evolve score` so applied candidates can feed outcome scores back into harness state.
- Split product language into agent tool-combo tentacles, runtime seeds, memory heart, and visual color layer.

## Remaining Gaps

- Self-iteration still prepares local draft data; direct GitHub PR creation needs a stronger OAuth adapter.
- Evolution candidates now carry scored outcomes, but candidate generation still needs LLM-driven rewrite and selection from those scores.
- Computer-use has seed adapters; it needs richer MCP/native/browser bridges.
- LLM provider setup works through env vars; user-facing provider profiles need polish.
- Release packaging, examples, and docs need enough finish for non-Rust users.

## Next Fill

- Turn repo-maintainer from draft artifacts into an OAuth-scoped PR adapter.
- Let the LLM evolution planner use scored outcomes to rewrite harness patches.
- Add richer non-shell adapters beyond the JSON feed seed.
