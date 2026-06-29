# Version Plan

Current version: `0.1.0`.

Cadence from this plan forward:

- 8 product commits: add capability, improve reliability, or polish UX.
- 1 cleanup/version commit: scan for uncomfortable product edges, remove stale wording, update docs, then bump patch version by `0.0.1`.

Release checklist:

- README stays product-page first, with the clean-brain value and install path visible.
- `docs/product-gap.md` records current shape, filled gaps, and next fill.
- CI stays green for Rust, manifests, tentacle runtime checks, pet page, and install path.
- `octopus preflight --live` is the release-readiness summary before every `0.1.0+` tag.
- Pages deploys `index.html`, `app.html`, `pet.html`, quickstart, architecture, references, and self-iteration docs.
- Version changes land together in `crates/octopus-core/Cargo.toml` and `Cargo.lock`.
- `0.0.24` must complete `docs/core-audit.md`: full source audit, redundant development residue removal, misaligned fallback removal, and core/harness boundary cleanup.
- `0.1.0` is reserved for the first release-ready build after recorded real-machine testing.
- Every tag from `0.1.0` onward requires real-machine testing before the tag is pushed.

Real-machine test gate for `0.1.0` and later tags:

- Install from GitHub with `cargo install --git ... --force`.
- Run `octopus --version`, `doctor`, `first-run`, `chat`, `goal refine`, `brain --goal --save`, `pet harness`, and `preflight` on a clean machine.
- Open `docs/app.html` and `docs/pet.html?state=harness` locally.
- Record the result with `docs/real-machine-test.md`; summarize any remaining issue in `docs/product-gap.md`.

`0.1.0` release history:

- `0.0.17`: product path cleanup, bridge clarity, readiness diagnostics, and core/harness boundary evidence.
- `0.0.18`: live LLM provider, OAuth, local model, and gateway validation.
- `0.0.19`: stronger harness-evolution feedback loop and repair reuse.
- `0.0.20`: real-machine local app run, release blockers, and current-head evidence.
- `0.0.21`: smallest SWE/Claw/Wild benchmark evidence and stability fixes.
- `0.0.22`: product README, docs, and OpenClaw-style usage tutorial pages.
- `0.0.23`: slim release artifacts and install/update/download path.
- `0.0.24`: full core audit before `0.1.0`; remove redundant code, development residue, and fallback logic that conflicts with the clean-brain and code-as-harness boundary.
- `0.1.0`: first release candidate only after the `0.0.24` audit and recorded real-machine test evidence pass.

Last release correction: `0.1.0` on 2026-06-28. Scope: recorded GitHub-installed binary gate, provider matrix, benchmark artifact evidence, GitHub PR dry-run, live preflight, static product demo screenshots, app/docs local startup, and version consistency.

`0.1.0` is the clean local-product release line.

`0.1.x` patch train after release: build the field-adaptation foundation in small usable steps.

- `0.1.x`: field packs, trace field IDs, one native read-only desktop pet, sampled active-field Need -> Feed -> verifier step from parallel field goals.
- `0.1.x`: generic `field-mini-task` code-as-harness surface so sampled field tasks create repairable trajectories before domain behavior is evolved.
- `0.1.x`: field trajectory summaries, failed-run reuse, reviewable harness patches. Current state has `octopus fields summary` reporting all eight first-pass mini tasks as satisfied.
- `0.1.x`: define harder mini tasks per field and keep the same sampled parallel pool. Current state has three satisfied mini-task layers in all eight packs after real trace repair cycles.
- `0.1.x`: keep the eight fields as peer slots; add depth inside each field without turning the field list into a sequence.
- `0.2.0`: Octopus can run, score, repair, and rerun tasks across math, search, code, SWE, research, computer-use, IB work, and robotics with the concrete tentacle code primarily iterated by Octopus.
