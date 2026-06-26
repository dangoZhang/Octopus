# Version Plan

Current version: `0.0.6`.

Cadence from this plan forward:

- 8 product commits: add capability, improve reliability, or polish UX.
- 1 cleanup/version commit: scan for uncomfortable product edges, remove stale wording, update docs, then bump patch version by `0.0.1`.

Release checklist:

- README stays story + Quick Install & Use.
- `docs/product-gap.md` records current shape, filled gaps, and next fill.
- CI stays green for Rust, Python, manifests, tentacle runtime checks, pet page, and install path.
- Pages deploys `index.html`, `app.html`, `pet.html`, quickstart, architecture, research, and self-iteration docs.
- Version changes land together in `crates/octopus-core/Cargo.toml` and `pyproject.toml`.
- `0.1.0` is reserved for the first release-ready build after recorded real-machine testing.
- Every tag from `0.1.0` onward requires real-machine testing before the tag is pushed.

Real-machine test gate for `0.1.0` and later tags:

- Install from GitHub with `cargo install --git ... --force`.
- Run `octopus --version`, `doctor`, `init`, `skills`, `install swe-agent`, `install computer-use-agent`, `install harness-repair-agent`, `install bash-only`, `chat`, `need observe .`, `pet harness`, and `beat 200` on a clean machine.
- Open `docs/app.html` and `docs/pet.html?state=harness` locally.
- Record the result with `docs/real-machine-test.md`; summarize any remaining issue in `docs/product-gap.md`.

Last cleanup/version correction: `0.0.6` on 2026-06-26. Scope: Feed-trace harness evolution, app grant/write apply controls, clean-brain context inspection, product capability report, Feed trace feedback scoring, docs/CI alignment; no tag cut.

Product commits toward `0.0.7`: 5/8.

Next planned cleanup/version commit: `0.0.7`.
