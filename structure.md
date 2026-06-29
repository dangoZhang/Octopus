# Structure

Updated: 2026-06-29, `0.1.x` field-adaptation foundation toward `v0.2.0`.

Line counts are `wc -l` over source/text files. Generated state under `.octopus/`, build output under `target/`, and binary PNG asset size are not counted.

## Repository Tree

```text
Octopus/
├── README.md                  English default entry and product story.
├── README.zh-CN.md            Chinese README.
├── Cargo.toml                 Rust workspace.
├── Cargo.lock
├── structure.md               Current structure and module map.
├── crates/octopus-core/
│   ├── Cargo.toml             `octopus-core`, version 0.1.1, binary `octopus`.
│   ├── examples/
│   │   └── thinking_tentacle.rs
│   └── src/
│       ├── lib.rs             Kernel contracts, state, routes, providers, traces, evolution.
│       ├── main.rs            CLI/product backend aggregation and command dispatch.
│       ├── app_bridge.rs      Local app bridge, HTTP/SSE, static fallback, bridge policy.
│       ├── bundled_harness.rs Installed-binary seed harness materializer, including field-mini-task and fixtures.
│       ├── core_boundary.rs   Stable-core vs editable-harness boundary diagnostics.
│       ├── download.rs        Download/install manifest and artifact checks.
│       ├── desktop_pet.rs     macOS read-only desktop pet launcher.
│       ├── field_pack.rs      Field-pack loader, matcher, and Need/trace metadata.
│       ├── pet.rs             Pixel Octopus state, SVG export, file URL helpers.
│       ├── profile_registry.rs Seed profile registry loading and status.
│       ├── release_gate.rs    Preflight records, benchmark and real-machine gates.
│       └── shell_words.rs     Shared shell command display quoting.
├── tentacles/
│   ├── profile-registry/      Editable seed profile data.
│   ├── bash-only/             Auditable write-and-run shell harness.
│   ├── computer-use-agent/    Desktop/browser/MCP/shell/clipboard adapters.
│   ├── field-mini-task/       Generic field mini task trajectory, execution harness, and repair templates.
│   ├── harness-repair-agent/  Harness diagnosis, repair sessions, adapter probes.
│   ├── json-feed/             `octopus-json-v1` Python runtime seed.
│   ├── repo-maintainer/       Repo inspection, PR draft, GitHub publish path.
│   ├── swe-agent/             Repo read/edit/patch/test tools.
│   ├── visual/                Pixel pet manifest.
│   └── tentacle.schema.json   Manifest schema.
├── field-packs/
│   ├── README.md              `0.1.x` field adaptation template guide.
│   ├── index.json             Field pack registry.
│   ├── field-pack.schema.json Field pack schema.
│   ├── _template/             Copyable starter pack.
│   ├── math/
│   ├── search/
│   ├── code/
│   ├── swe/
│   ├── research/
│   ├── computer-use/
│   ├── ib/
│   └── robotics/
├── desktop/
│   └── pet/
│       └── OctopusDesktopPet.swift
│                              Native AppKit read-only state observer.
├── docs/
│   ├── index.html             GitHub Pages landing page.
│   ├── demo.html              Static product demo page with real app screenshots.
│   ├── docs.html              Product docs tutorial page.
│   ├── app.html               One-page local app and browser Try App.
│   ├── assets/demo/           Real Octopus App screenshots for demo page.
│   ├── pet.html               Pixel Octopus read-only HTML preview.
│   ├── use.html               Five-minute product use guide.
│   ├── tutorial.html          Product tutorial.
│   ├── recipes.html           Goal-first usage recipes.
│   ├── quickstart.html/md     Install and launch guides.
│   ├── about.html             Product story page.
│   ├── architecture.md        Architecture notes.
│   ├── field-adaptation.md    Field adaptation TODO and `v0.2.0` gate.
│   ├── product-gap.md         Product gap and change log.
│   ├── real-machine-test.md   `0.1.0+` release gate evidence.
│   ├── version-plan.md        Release cadence and next milestone.
│   └── zh/                    Chinese docs pages.
├── cowork/                    Cross-thread notes; not canonical product docs.
└── local/docs/                Local planning notes; not release docs.
```

## Functional Modules

| Module | Role | Main files | Lines |
| --- | --- | --- | ---: |
| Stable kernel | Goal/Need/Feed contracts, state, route scores, memory, provider client, Feed traces, evolution data | `crates/octopus-core/src/lib.rs` | 17,577 |
| Field adaptation core | Field-pack loading, matching, Need annotation, trace metadata, sampled field execution slots, verifier results, field trajectory summaries, live field mini task loader, editable repair templates, and compile/execute template checks | `field_pack.rs`, `field-packs/**`, `tentacles/field-mini-task/**`, `docs/field-adaptation.md` | 3,606 |
| CLI and product backend | Command dispatch, Goal/chat/brain, provider setup, doctor/report/preflight aggregation, starter/install/check flows | `crates/octopus-core/src/main.rs` | 23,698 |
| Local app bridge | Local HTTP/SSE server, app policy, command allow-list, static app/docs/demo fallback | `app_bridge.rs`, `docs/app.html` | 2,348 |
| Release and install gates | Release records, benchmark evidence, download/install manifest, real-machine checks | `release_gate.rs`, `download.rs`, `docs/real-machine-test.md`, `docs/download.json`, `docs/install.sh` | 1,022 |
| Pet and visual state | Pixel Octopus state, SVG/export helpers, native read-only observer, HTML preview | `pet.rs`, `desktop_pet.rs`, `desktop/pet/OctopusDesktopPet.swift`, `docs/pet.html`, `tentacles/visual/manifest.json` | 1,505 |
| Product docs/site | README, landing/demo/tutorial/use/recipes/about/docs pages | `README*`, `docs/*.html`, `docs/*.md`, `docs/zh/*` | 7,965 |
| Editable tentacles | Code-as-harness Feed suppliers: prompts, manifests, tools, repair templates, repair surfaces | `tentacles/**` | 6,950 |

## Core, Distinctive, Editable

### Core Functions

These should remain stable and hard to accidentally mutate:

- Clean-brain contract: `Goal + Mem + Need + Feed`.
- Need to Feed transport and trace records.
- Field-pack loading and Need/Feed trace annotation.
- Field trajectory summaries for parallel field pool learning.
- Product report field-pool block: eight peer slots, sampled slot, and worker-slot policy.
- Route scoring, provider clients, memory/heartbeat state, grants, permissions.
- Product bridge rule: user-facing writes go through Goal; internal actions feed the agent.
- Release gates: preflight, benchmark evidence, field-pool visibility, real-machine records, local app readiness.

Where they live:

- `crates/octopus-core/src/lib.rs`
- `crates/octopus-core/src/main.rs`
- `crates/octopus-core/src/app_bridge.rs`
- `crates/octopus-core/src/release_gate.rs`
- `crates/octopus-core/src/core_boundary.rs`

### Distinctive Functions

These are the project identity:

- Clean brain: the main model asks for Need, not tool choreography.
- Thinking tentacles: tool-side LLM + tool meta + executable harness code.
- Need -> Feed -> Feedback loop with compact Feed traces.
- Three-heart surface: heartbeat, memory beat, harness beat.
- Pixel Octopus state and color-changing pet.
- Native desktop observer: reads `.octopus/state.json`, shows Goal, Need bubble, action bubbles, Feed/evolution/blocked colors, defaulting to one pet.
- HTML pet page: read-only preview for docs and screenshots.
- Static product demo showing real app output, not a fake mock.

Where they live:

- `desktop/pet/OctopusDesktopPet.swift`
- `docs/app.html`, `docs/demo.html`, `docs/pet.html`
- `crates/octopus-core/src/pet.rs`
- `tentacles/*/manifest.json`
- `tentacles/*/tools/*`
- `docs/assets/demo/*.png`

### Editable / Evolvable Functions

These are intended to be changed by Octopus or by harness iteration:

- Tentacle manifests, prompts, tool metadata, runtime scripts, repair policies.
- Seed profile registry.
- Harness repair sessions and adapter probes.
- Field mini task trajectory adapters, live task-template loading, task-specific repair templates, and compile/execute template checks.
- Field packs for math, search, code, SWE, research, computer-use, IB work, and robotics. The eight packs are peer slots in the same parallel Goal pool; multi-field objectives constrain the candidate pool while field status and recent-run fairness choose sampled slots, then each worker slot writes its own Need Queue item. `evolve parallel`, `check field-mini-task`, `install`, `probe`, `think`, `repair`, `beat` harness evolution, `evolve recommend/apply/score`, provider matrix tentacle checks, `starter`, `skills`, `init`, `bootstrap`, `adapt`, default `manifests`, `report`, `doctor`, and `preflight` preserve local manifests while falling back to bundled seeds when a project-local `tentacles/` lacks or shadows a seed. The default merged view replaces broken same-ID seed manifests with healthy bundled seeds, and the direct resolver skips local same-ID seed manifests with missing entrypoints, so `chat`/`need`/`beat`, `check`/`think`/`probe`/`evolve`, `repair`, `start --check`, and direct `install` retry bundled seeds when the local seed is incomplete. `report` exposes the field pool as eight peer slots and broken installed seed sources as a gap pointing to `beat 200` without mutating state; `preflight` now requires that field-pool visibility before release readiness can pass. Each worker records its queued Need index, automatically runs sampled Needs through Feed, and backfills worker trace/verifier/status. Installed binaries also materialize the `field-mini-task` runner, checker, 24 repair templates, field-packs, and minimal docs fixtures into editable bundled seeds. Each pack now has first, second, and third-layer mini tasks, all eight third-layer tasks passed real failure -> repair -> rerun cycles, all 24 task-specific repair templates live outside Rust core, live runtime loads standalone templates without duplicate field-specific branches, and the harness check verifies and executes 24/24 templates.

Where they live now:

- `tentacles/`
- `field-packs/`
- `.octopus/` at runtime
- `docs/field-adaptation.md` for the `0.1.x` TODO and `v0.2.0` gate

Current field pack layout:

```text
field-packs/
  _template/
  math/
  search/
  code/
  swe/
  research/
  computer-use/
  ib/
  robotics/
```

## Code Size

### Top-Level Areas

| Area | Files | Lines |
| --- | ---: | ---: |
| `crates/octopus-core/src` | 12 | 44,617 |
| `crates/octopus-core/examples` | 1 | 27 |
| `tentacles` | 66 | 6,950 |
| `field-packs` | 12 | 487 |
| `desktop/pet` | 1 | 468 |
| `docs` | 31 | 8,833 |
| `cowork` | 3 | 101 |
| `local/docs` | 12 | 507 |

### Core Rust Files

| File | Lines |
| --- | ---: |
| `main.rs` | 23,698 |
| `lib.rs` | 17,577 |
| `app_bridge.rs` | 1,125 |
| `release_gate.rs` | 514 |
| `field_pack.rs` | 537 |
| `pet.rs` | 271 |
| `bundled_harness.rs` | 379 |
| `download.rs` | 166 |
| `desktop_pet.rs` | 131 |
| `core_boundary.rs` | 123 |
| `profile_registry.rs` | 78 |
| `shell_words.rs` | 18 |

### Tentacles

| Tentacle | Files | Lines |
| --- | ---: | ---: |
| `harness-repair-agent` | 6 | 1,918 |
| `repo-maintainer` | 8 | 719 |
| `computer-use-agent` | 10 | 644 |
| `profile-registry` | 1 | 600 |
| `field-mini-task` | 28 | 2,391 |
| `swe-agent` | 6 | 217 |
| `json-feed` | 2 | 162 |
| `bash-only` | 2 | 77 |
| `visual` | 1 | 56 |

## Notes

- The pasted tree was stale: it said version `0.0.19`; current package line is `0.1.1`.
- The stable core is still too concentrated in `main.rs` and `lib.rs`. During `0.1.x`, split by capability before the `v0.2.0` field gate.
- Product demo is now static screenshot-first. Dynamic Try App remains in `docs/app.html`, not as the demo page's main story.
