# Version Plan

Current version: `0.1.1`.

Cadence from this plan forward:

- 8 product commits: add capability, improve reliability, or polish UX.
- 1 cleanup/version commit: scan for uncomfortable product edges, remove stale wording, update docs, then bump patch version by `0.0.1`.

Release checklist:

- README stays story + Quick Install & Use.
- `docs/product-gap.md` records current shape, filled gaps, and next fill.
- CI stays green for Rust, Python, manifests, tentacle runtime checks, pet page, and install path.
- Pages deploys `index.html`, `app.html`, `pet.html`, quickstart, architecture, research, and self-iteration docs.
- Version changes land together in `crates/octopus-core/Cargo.toml` and `pyproject.toml`.
- Every tag after `0.1.0` requires real-machine testing before the tag is pushed.

Real-machine test gate for tags after `0.1.0`:

- Install from GitHub with `cargo install --git ... --force`.
- Run `octopus --version`, `doctor`, `init`, `skills`, `install swe-agent`, `install computer-use-agent`, `install bash-only`, `chat`, `need observe .`, `pet harness`, and `beat 200` on a clean machine.
- Open `docs/app.html` and `docs/pet.html?state=harness` locally.
- Record the result with `docs/real-machine-test.md`; summarize any remaining issue in `docs/product-gap.md`.

Last cleanup/version commit: `0.1.1` on 2026-06-26. Scope: pet events, LLM harness evolution, docs/CI alignment.

Next planned cleanup/version commit: `0.1.2`.
