# Version Plan

Current version: `0.0.13`.

Cadence from this plan forward:

- 8 product commits: add capability, improve reliability, or polish UX.
- 1 cleanup/version commit: scan for uncomfortable product edges, remove stale wording, update docs, then bump patch version by `0.0.1`.

Release checklist:

- README stays product-page first, with the clean-brain value and install path visible.
- `docs/product-gap.md` records current shape, filled gaps, and next fill.
- CI stays green for Rust, Python, manifests, tentacle runtime checks, pet page, and install path.
- `octopus preflight --live` is the release-readiness summary before `0.1.0` tags.
- Pages deploys `index.html`, `app.html`, `pet.html`, quickstart, architecture, references, and self-iteration docs.
- Version changes land together in `crates/octopus-core/Cargo.toml`, `Cargo.lock`, and `pyproject.toml`.
- `0.1.0` is reserved for the first release-ready build after recorded real-machine testing.
- Every tag from `0.1.0` onward requires real-machine testing before the tag is pushed.

Real-machine test gate for `0.1.0` and later tags:

- Install from GitHub with `cargo install --git ... --force`.
- Run `octopus --version`, `doctor`, `init`, `skills`, `install swe-agent`, `install computer-use-agent`, `install harness-repair-agent`, `install bash-only`, `chat`, `need observe .`, `pet harness`, and `beat 200` on a clean machine.
- Open `docs/app.html` and `docs/pet.html?state=harness` locally.
- Record the result with `docs/real-machine-test.md`; summarize any remaining issue in `docs/product-gap.md`.

Last cleanup/version correction: `0.0.13` on 2026-06-27. Scope: launch-and-open startup, bundled seed startup, dry-run update, native update report, human Goal constraints, clean-brain agenda/model routing, removed stale demo CI checks, removed secondary startup route, README/docs route cleanup, and version consistency; no tag cut.

Product commits toward `0.0.14`: 2/8.

Next planned cleanup/version commit: `0.0.14`.
