# Version Plan

Current version: `0.0.22`.

Cadence from this plan forward:

- 8 product commits: add capability, improve reliability, or polish UX.
- 1 cleanup/version commit: scan for uncomfortable product edges, remove stale wording, update docs, then bump patch version by `0.0.1`.

Release checklist:

- README stays product-page first, with the clean-brain value and install path visible.
- `docs/product-gap.md` records current shape, filled gaps, and next fill.
- CI stays green for Rust, manifests, tentacle runtime checks, pet page, and install path.
- `octopus preflight --live` is the release-readiness summary before `0.1.0` tags.
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

Road to `0.1.0`:

- `0.0.17`: product path cleanup, bridge clarity, readiness diagnostics, and core/harness boundary evidence.
- `0.0.18`: live LLM provider, OAuth, local model, and gateway validation.
- `0.0.19`: stronger harness-evolution feedback loop and repair reuse.
- `0.0.20`: real-machine local app run, release blockers, and current-head evidence.
- `0.0.21`: smallest SWE/Claw/Wild benchmark evidence and stability fixes.
- `0.0.22`: product README, docs, and OpenClaw-style usage tutorial pages.
- `0.0.23`: slim release artifacts and install/update/download path.
- `0.0.24`: full core audit before `0.1.0`; remove redundant code, development residue, and fallback logic that conflicts with the clean-brain and code-as-harness boundary.
- `0.1.0`: first release candidate only after the `0.0.24` audit and recorded real-machine test evidence pass.

Last cleanup/version correction: `0.0.22` on 2026-06-28. Scope: product tutorial pages, Chinese tutorial and recipes, About story page, Goal-first recipes, homepage/mobile guide polish, Quick Install & Use HTML refresh, static app evidence links, and version consistency; no tag cut.

Product commits toward `0.0.23`: 6/8.

Next planned cleanup/version commit: `0.0.23`.
