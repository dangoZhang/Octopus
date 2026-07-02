# Version Plan

Current version: `0.2.0`.

Release status: `v0.2.0` is the first usable public release line. The old `v0.1.0` GitHub release and tag were removed on 2026-06-30 because that line did not yet have a non-fake desktop pet observer or real LLM-driven harness evolution.

Completed `0.2.1`:

```text
0.2.1 - align field evolution with the product architecture
├── commit 1: record the todo tree and architecture boundary
│   ├── pet observes real state only
│   ├── brain thinks in Goal/Mem/Need/Feed only
│   ├── tentacles do work through editable harness code
│   ├── memory summarizes traces
│   └── interaction stays snowball-goal driven
├── commit 2: add the default Go worker substrate for field-mini-task
│   ├── add reusable Go Feed helpers
│   ├── add a default Go worker that returns honest partial until domain logic evolves
│   └── missing Go runtime must surface as environment evidence, not a fake pass
├── commit 3: steer evolution toward Go worker patches
│   ├── LLM evolution prompt names .go worker targets first
│   ├── patch allow-list accepts matching .go workers
│   ├── structured apply validates field-pack plus worker pairs
│   └── .pyfrag remains legacy compatibility only
├── commit 4: verify observation and local behavior
│   ├── run field-mini-task in the current no-Go environment
│   ├── confirm status is honest partial with go_runtime_missing or domain gap
│   ├── confirm desktop pet still reads state as observer
│   └── run the short Rust/build checks for touched core
└── pushed to main
```

Active todo tree:

```text
0.2.2 - make Octopus use the Go worker path for one field
├── commit 1: remove field curriculum next-action language that points to repair templates
├── commit 2: let Octopus evolve one Go worker for that field
│   ├── generated `tentacles/field-mini-task/workers/swe/swe-go-default-smoke/main.go`
│   ├── direct run now selects `./workers/swe/swe-go-default-smoke`
│   └── current no-Go environment returns honest `go_runtime_missing`
├── commit 3: record Feed trace and memory summary from the real run
│   ├── trace #143: Go worker partial from missing Go runtime
│   └── trace #144: generic auto-feed lost field_mini_task and returned unsupported
└── commit 4: rerun the same field mini task after post-apply evolution
    ├── target Feed from the source trace when objective names `trace N`
    ├── ignore latest generic field traces that lack `field_mini_task`
    └── keep runtime-created Go workers outside the static field-pack table
```

Next version tree:

```text
0.2.3+ - repeat by field
├── math
├── write
├── translate
├── search
├── code
├── swe
├── research
├── computer-use
├── IB
└── robotics
```

Cadence:

- 8 landed product commits make one patch version.
- Each patch version should include capability work plus a small cleanup/version pass.
- If the cadence drifts, the next cleanup/version commit catches the package version up to the completed 8-commit groups since the last release tag.

Release checklist:

- README stays product-page first, with the clean-brain value and install path visible.
- `docs/product-gap.md` records current shape, filled gaps, and next fill.
- CI stays green for Rust, manifests, tentacle runtime checks, pet page, and install path.
- `octopus preflight --live` is the release-readiness summary before every `0.2.0+` tag.
- Release evidence records the field pool with `fields summary`, the desktop pet compiler gate with `desktop_pet_source`, and filled `Field pool` / `Desktop pet source` results.
- Pages deploys `index.html`, `app.html`, `pet.html`, quickstart, architecture, references, and self-iteration docs.
- Version changes land together in `crates/octopus-core/Cargo.toml` and `Cargo.lock`.
- `0.0.24` must complete `docs/core-audit.md`: full source audit, redundant development residue removal, local substitute logic removal, and core/harness boundary cleanup.
- `0.2.0` requires desktop pet observation and real harness evolution to pass on a real machine.
- Every tag from `0.2.0` onward requires real-machine testing before the tag is pushed.

Real-machine test gate for `0.2.0` and later tags:

- Install from GitHub with `cargo install --git ... --force`.
- Run `octopus --version`, `doctor`, `first-run`, `chat`, `goal refine`, `pet harness`, and `preflight` on a clean machine.
- Run `octopus evolve parallel --workers 2 "preflight math and search field adaptation"`, `octopus fields summary`, and `octopus status`; verify `desktop_pet_source` in preflight; fill the `Field pool` result with the required v0.2 named fields, `missing_required=none`, `latest_activity` including `worker=`, and `parallel_run` including `requested_worker_slots=`, `active_worker_slots=`, and a `candidate_pool` containing `math` and `search`.
- Run `octopus check field-mini-task 2` and fill the `Field mini task harness` result with `checked_count=...`, `executed_count=...`, `satisfied_count=...`, `partial_count=...`, `missing_count=0`, `invalid_count=0`, and `status=ok`. Partial templates are allowed only when they expose the missing evidence.
- Run one live LLM `evolve recommend` while the native desktop pet is visible. Confirm `.octopus/state.json` and the desktop pet change to `evolution/partial` during the long planner call, then to `evolution/satisfied` or `blocked/failed` when the recommendation finishes.
- Open `docs/app.html` and `docs/pet.html?state=harness` locally.
- Record the result with `docs/real-machine-test.md`; summarize any remaining issue in `docs/product-gap.md`.

Retracted `0.1.0` history:

- `0.0.17`: product path cleanup, bridge clarity, readiness diagnostics, and core/harness boundary evidence.
- `0.0.18`: live LLM provider, OAuth, local model, and gateway validation.
- `0.0.19`: stronger harness-evolution feedback loop and repair reuse.
- `0.0.20`: real-machine local app run, release blockers, and current-head evidence.
- `0.0.21`: smallest SWE/Claw/Wild benchmark evidence and stability fixes.
- `0.0.22`: product README, docs, and OpenClaw-style usage tutorial pages.
- `0.0.23`: slim release artifacts and install/update/download path.
- `0.0.24`: full core audit before `0.1.0`; remove redundant code, development residue, and local substitute logic that conflicts with the clean-brain and code-as-harness boundary.
- `0.1.0`: first release candidate only after the `0.0.24` audit and recorded real-machine test evidence pass.

Release correction: `0.1.0` on 2026-06-28 was retracted on 2026-06-30. Scope was recorded GitHub-installed binary gate, provider matrix, benchmark artifact evidence, GitHub PR dry-run, live preflight, static product demo screenshots, app/docs local startup, and version consistency, but the line is no longer treated as a usable public release.

`0.2.0` is the first usable public release line after the desktop pet reflects real state and harness evolution is driven by LLM/tool traces rather than scripted steering.

`0.2.x` toward `v0.3.0`: pre-evolve installable field packs beyond the required v0.2 pool.

- `0.2.x`: add `translate` as a first-class field pack beside `write`, with `translate-mini-1` in editable `field-mini-task` repair templates.
- `0.2.x`: run `translate-mini-1` through Octopus Need -> Feed -> verifier; current trace `#82` and verifier `#66` are satisfied in local state.
- `0.2.x`: field matching now supports multilingual non-ASCII aliases from field-pack data, so Chinese goals such as `翻译一段中文到英文并保留术语` route to `translate`.

Completed `0.1.x` patch train: field-adaptation foundation built in small usable steps.

- `0.1.x`: field packs, trace field IDs, one native read-only desktop pet, peer-field worker slots that run Need -> Feed -> verifier from the parallel Goal pool.
- `0.1.x`: generic `field-mini-task` code-as-harness surface so peer-field worker tasks create repairable trajectories before domain behavior is evolved.
- `0.1.x`: field trajectory summaries, failed-run reuse, reviewable harness patches. Current state has `octopus fields summary` reporting all eight first-pass mini tasks as satisfied.
- `0.1.x`: define harder mini tasks per field and keep the same parallel field pool. Current state has three satisfied mini-task layers in all required v0.2 packs after real trace repair cycles.
- `0.1.1`-`0.1.8`: repair self-evolution cleanup: scored patch draft/review/apply/verify/learning/strategy evidence is reviewable, and repair effectiveness labels share one helper.
- `0.1.1`-`0.1.8`: repair effectiveness rollup gives heartbeat one reviewable Feed across repair sources, with failure-dominated rollups able to queue the next self-repair Need.
- `0.1.1`-`0.1.8`: repair outcomes now remember repair-effectiveness-rollup usage, so later rollups can score whether the rollup guidance helped.
- `0.1.1`-`0.1.8`: rollup guidance effectiveness is now a first-class repair artifact and heartbeat Feed field.
- `0.1.1`-`0.1.8`: rollup guidance adaptation is now a reviewable Feed that can repair unreliable rollup guidance before it keeps driving heartbeat.
- `0.1.1`-`0.1.8`: repair queue brief is now a reviewable heartbeat artifact that explains which harness evidence queued the next self-repair Need.
- `0.1.1`-`0.1.8`: repair queue brief effectiveness is now a scored repair Feed artifact, so heartbeat can reuse or avoid queue sources from reviewed outcomes.
- `0.1.1`-`0.1.8`: repair queue brief adaptation is now a heartbeat Feed that can repair unreliable queue source selection.
- `0.1.8`: field-pack task evolution guards now include the declared pack files plus `field-packs/index.json`, and LLM-generated field-pack patches are prompted and tested as declared multi-file target patches.
- `0.1.8`: completed peer-field layers now point to executable `evolve recommend field-mini-task` next actions with the active state path, so Octopus can recommend the next harder layer instead of stopping at prose.
- `0.1.8`: native desktop pet observation and self-iteration next actions keep worker/Need state scoped, state-path aware, and read-only.
- `0.1.x`: keep the required v0.2 fields as peer slots; add depth inside each field without turning the field list into a sequence.
- `0.2.0`: Octopus can run, score, repair, and rerun tasks across math, search, code, SWE, research, computer-use, IB work, and robotics with the concrete tentacle code primarily iterated by Octopus.
