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
