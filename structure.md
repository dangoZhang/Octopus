# Structure

Updated: 2026-06-28, after `v0.1.0`.

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
│   ├── Cargo.toml             `octopus-core`, version 0.1.0, binary `octopus`.
│   ├── examples/
│   │   └── thinking_tentacle.rs
│   └── src/
│       ├── lib.rs             Kernel contracts, state, routes, providers, traces, evolution.
│       ├── main.rs            CLI/product backend aggregation and command dispatch.
│       ├── app_bridge.rs      Local app bridge, HTTP/SSE, static fallback, bridge policy.
│       ├── bundled_harness.rs Installed-binary seed harness materializer.
│       ├── core_boundary.rs   Stable-core vs editable-harness boundary diagnostics.
│       ├── download.rs        Download/install manifest and artifact checks.
│       ├── pet.rs             Pixel Octopus state, SVG export, file URL helpers.
│       ├── profile_registry.rs Seed profile registry loading and status.
│       ├── release_gate.rs    Preflight records, benchmark and real-machine gates.
│       └── shell_words.rs     Shared shell command display quoting.
├── tentacles/
│   ├── profile-registry/      Editable seed profile data.
│   ├── bash-only/             Auditable write-and-run shell harness.
│   ├── computer-use-agent/    Desktop/browser/MCP/shell/clipboard adapters.
│   ├── harness-repair-agent/  Harness diagnosis, repair sessions, adapter probes.
│   ├── json-feed/             `octopus-json-v1` Python runtime seed.
│   ├── repo-maintainer/       Repo inspection, PR draft, GitHub publish path.
│   ├── swe-agent/             Repo read/edit/patch/test tools.
│   ├── visual/                Pixel pet manifest.
│   └── tentacle.schema.json   Manifest schema.
├── field-packs/
│   ├── README.md              `v0.2.0` field adaptation template guide.
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
├── docs/
│   ├── index.html             GitHub Pages landing page.
│   ├── demo.html              Static product demo page with real app screenshots.
│   ├── docs.html              Product docs tutorial page.
│   ├── app.html               One-page local app and browser Try App.
│   ├── assets/demo/           Real Octopus App screenshots for demo page.
│   ├── pet.html               Pixel Octopus page.
│   ├── use.html               Five-minute product use guide.
│   ├── tutorial.html          Product tutorial.
│   ├── recipes.html           Goal-first usage recipes.
│   ├── quickstart.html/md     Install and launch guides.
│   ├── about.html             Product story page.
│   ├── architecture.md        Architecture notes.
│   ├── field-adaptation.md    `v0.2.0` field adaptation harness plan.
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
| Stable kernel | Goal/Need/Feed contracts, state, route scores, memory, provider client, Feed traces, evolution data | `crates/octopus-core/src/lib.rs` | 14,532 |
| CLI and product backend | Command dispatch, Goal/chat/brain, provider setup, doctor/report/preflight aggregation, starter/install/check flows | `crates/octopus-core/src/main.rs` | 20,396 |
| Local app bridge | Local HTTP/SSE server, app policy, command allow-list, static app/docs/demo fallback | `app_bridge.rs`, `docs/app.html` | 2,251 |
| Release and install gates | Release records, benchmark evidence, download/install manifest, real-machine checks | `release_gate.rs`, `download.rs`, `docs/real-machine-test.md`, `docs/download.json`, `docs/install.sh` | 1,021 |
| Pet and visual state | Pixel Octopus state, SVG/export helpers, app pet surface | `pet.rs`, `docs/pet.html`, `tentacles/visual/manifest.json` | 551 |
| Product docs/site | README, landing/demo/tutorial/use/recipes/about/docs pages | `README*`, `docs/*.html`, `docs/*.md`, `docs/zh/*` | 7,330 |
| Editable tentacles | Code-as-harness Feed suppliers: prompts, manifests, tools, repair surfaces | `tentacles/**` | 4,414 |
| Field adaptation packs | `v0.2.0` task-field templates, permission boundaries, verifier contracts, mini tasks | `field-packs/**`, `docs/field-adaptation.md` | 495 |

## Core, Distinctive, Editable

### Core Functions

These should remain stable and hard to accidentally mutate:

- Clean-brain contract: `Goal + Mem + Need + Feed`.
- Need to Feed transport and trace records.
- Route scoring, provider clients, memory/heartbeat state, grants, permissions.
- Product bridge rule: user-facing writes go through Goal; internal actions feed the agent.
- Release gates: preflight, benchmark evidence, real-machine records, local app readiness.

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
- Static product demo showing real app output, not a fake mock.

Where they live:

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
- Field packs for math, search, code, SWE, research, computer-use, IB work, and robotics.

Where they live now:

- `tentacles/`
- `field-packs/`
- `.octopus/` at runtime
- `docs/field-adaptation.md` for the `v0.2.0` plan

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
| `crates/octopus-core/src` | 10 | 37,374 |
| `crates/octopus-core/examples` | 1 | 27 |
| `tentacles` | 38 | 4,414 |
| `field-packs` | 12 | 379 |
| `docs` | 29 | 7,330 |
| `cowork` | 3 | 168 |
| `local/docs` | 12 | 507 |

### Core Rust Files

| File | Lines |
| --- | ---: |
| `main.rs` | 20,396 |
| `lib.rs` | 14,532 |
| `app_bridge.rs` | 1,125 |
| `release_gate.rs` | 513 |
| `pet.rs` | 224 |
| `bundled_harness.rs` | 199 |
| `download.rs` | 166 |
| `core_boundary.rs` | 123 |
| `profile_registry.rs` | 78 |
| `shell_words.rs` | 18 |

### Tentacles

| Tentacle | Files | Lines |
| --- | ---: | ---: |
| `harness-repair-agent` | 6 | 1,774 |
| `repo-maintainer` | 8 | 719 |
| `computer-use-agent` | 10 | 644 |
| `profile-registry` | 1 | 600 |
| `swe-agent` | 6 | 217 |
| `json-feed` | 2 | 162 |
| `bash-only` | 2 | 77 |
| `visual` | 1 | 56 |

## Notes

- The pasted tree was stale: it said version `0.0.19`; current package and release line are `0.1.0`.
- The stable core is still too concentrated in `main.rs` and `lib.rs`. For `v0.2.0`, split by capability before adding more domain infrastructure.
- Product demo is now static screenshot-first. Dynamic Try App remains in `docs/app.html`, not as the demo page's main story.
