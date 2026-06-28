# Core Audit Gate

Required for `0.0.24` before `0.1.0`.

## Goal

Ship the smallest honest Octopus core:

- clean brain: `Goal + Mem + Need + Feed`
- tentacle intelligence: `Need + Tool + Action -> Feed`
- three beats: heartbeat, memory beat, harness-evolution beat
- editable code-as-harness outside the stable kernel
- product start/app/preflight path that really runs locally

## Keep

- Rust kernel contracts for brain, Need, Feed, Feedback, route learning, memory, heartbeat, harness evolution, provider access, app bridge, release gates, and product startup.
- Product app code that is stable infrastructure.
- Tentacle manifests, prompts, tool metadata, runtime code, and evolution policy as editable harness units.
- Regression tests that prove product invariants, release gates, context boundaries, provider routing, repair feedback, and local startup.
- Current-head local app evidence from `octopus start --check`, stored as `.octopus/local-app-run.json` and checked by preflight.

## Remove Or Move

- Dead prototype code, old demo commands, duplicate product paths, stale generated artifacts, and hidden temporary flows.
- Development-only tests or fixtures that only preserve past implementation accidents.
- Fallback logic that chooses implementation for the clean brain or leaks tool/API/file burden into brain context.
- Hard-coded seed harness data in Rust when it can live in manifest/profile registry files.
- Internal controls exposed as first-use product actions outside Goal/chat/first-run.

## Evidence

The `0.0.24` audit must update this file with:

- reviewed file tree snapshot
- kept core modules
- removed code/tests/docs
- moved harness units
- remaining deliberate exceptions
- commands run
- current commit hash

`0.1.0` cannot be cut until this evidence exists and the real-machine gate is recorded.

## Audit Pass 1: Hidden Tool-Thinking Fallback

- Commit scanned: `53b3ea4`.
- Reviewed tree snapshot: `README*`, `Cargo.*`, `structure.md`, `crates/octopus-core/src/{lib.rs,main.rs,app_bridge.rs,core_boundary.rs,release_gate.rs}`, `crates/octopus-core/examples/thinking_tentacle.rs`, `tentacles/{profile-registry,swe-agent,computer-use-agent,repo-maintainer,harness-repair-agent,bash-only,json-feed,visual}`, and `docs/{app,tutorial,recipes,download,install,pet,architecture,product-gap,version-plan,real-machine-test,zh}`.
- Kept core modules: Rust kernel contracts, planner/tentacle traits, app bridge, release gate, core boundary diagnostics, product startup, provider adapters, memory/heartbeat/harness evolution state.
- Removed code: `ChatPlanner` no longer stores or invokes a hidden `RulePlanner` fallback after LLM failure or invalid Plan JSON.
- Kept deliberate exception: `RulePlanner` remains as an explicit planner for deterministic tests and the small `examples/thinking_tentacle.rs` example; it is not hidden inside tool-side LLM planning.
- Moved harness units: none in this pass.
- Remaining candidates: shrink `main.rs` product/backend aggregation, audit legacy executable tool contracts, review historical docs/log residue, and keep hard-coded seed data out of Rust.
- Commands: `rg --files`, `find . -maxdepth 3 -type d`, fallback/residue `rg`, `git status --ignored=matching`, `wc -l crates/octopus-core/src/*.rs`, `cargo fmt --all --check`, `cargo check -q -p octopus-core`, `cargo test -q -p octopus-core chat_planner_uses_chat_client_plan`, `cargo test -q -p octopus-core chat_planner_does_not_fallback_to_rule_execution`, `cargo test -q -p octopus-core planning_tentacle_selects_tool_before_execution`.
