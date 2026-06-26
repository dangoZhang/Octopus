# Product Gap Log

Updated: 2026-06-26

## Current Shape

- Clean brain emits cognitive `Need`; it does not choose tools.
- Tentacles own LLM prompt, tool metadata, runtime code, and evolution policy.
- Harness stores memory, route scores, OAuth grants, installed tentacles, and goal state.
- Three beats exist: heartbeat, memory compaction, and harness route evolution.
- Color pet exposes heartbeat, memory, harness, blocked, and success states.
- Seeds include SWE repo tools, computer-use tools, repo-maintainer, visual, memory, a Python JSON feed, and a script-writing runner.
- LLM adapters support OpenAI-compatible providers for chat goal refinement and manifest tool planning.

## Filled This Iteration

- Clarified the tentacle model as runtime-neutral `LLM + tool meta + code + policy`.
- Moved seed manifests away from `tools/*.sh` as the conceptual evolution surface.
- Authorized `evolve apply` now emits a reviewable `.patch` file and leaves application to the grant-bound outer flow.
- Added stale patch cleanup when authorization is absent.
- Added a built-in non-shell `json-feed` tentacle that consumes `octopus-json-v1`.

## Remaining Gaps

- Self-iteration still prepares local draft data; direct GitHub PR creation needs a stronger OAuth adapter.
- Evolution candidates are still template-driven; they need scoring from previous feed outcomes.
- Computer-use has seed adapters; it needs richer MCP/native/browser bridges.
- LLM provider setup works through env vars; user-facing provider profiles need polish.
- Release packaging, examples, and docs need enough finish for non-Rust users.

## Next Fill

- Turn repo-maintainer from draft artifacts into an OAuth-scoped PR adapter.
- Persist evolution outcomes so harness changes can be ranked by real feedback.
- Add richer non-shell adapters beyond the JSON feed seed.
