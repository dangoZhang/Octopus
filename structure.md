# Structure

Updated: 2026-06-30, after modular user-surface/evolution/pet diagnostics refactor.

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
│   ├── Cargo.toml             `octopus-core`, version 0.2.0, binary `octopus`.
│   └── src/
│       ├── lib.rs             Kernel contracts, state, routes, providers, traces, evolution orchestration.
│       ├── main.rs            CLI/product backend aggregation and command dispatch.
│       ├── app_bridge.rs      Local app bridge, HTTP/SSE, embedded app assets, bridge policy.
│       ├── bundled_harness.rs Installed-binary seed harness materializer, including field-mini-task and fixtures.
│       ├── core_boundary.rs   Stable-core vs editable-harness boundary diagnostics.
│       ├── diagnostics.rs     Strategy diagnostics for clean brain, intelligent tentacles, editable goal, pet freshness.
│       ├── download.rs        Download/install manifest and artifact checks.
│       ├── desktop_pet.rs     macOS read-only desktop pet launcher.
│       ├── evolution.rs       Manifest-owned evolution surface requirement validation.
│       ├── field_pack.rs      Field-pack loader, matcher, and Need/trace metadata.
│       ├── pet.rs             Pixel Octopus state, SVG export, file URL helpers.
│       ├── pet_events.rs      Unified state event write path for desktop/pet observers.
│       ├── profile_registry.rs Seed profile registry loading and status.
│       ├── release_gate.rs    Preflight records, benchmark and real-machine gates.
│       ├── shell_words.rs     Shared shell command display quoting.
│       └── user_surface.rs    User-facing Goal hints separated from agent/observer commands.
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
│   ├── robotics/
│   ├── write/
│   └── translate/
├── desktop/
│   └── pet/
│       └── OctopusDesktopPet.swift
│                              Native AppKit read-only state observer.
├── docs/
│   ├── index.html             GitHub Pages landing page.
│   ├── demo.html              Release showcase page with real app screenshots.
│   ├── docs.html              Product docs tutorial page.
│   ├── app.html               One-page local Goal/Need/Feed app surface.
│   ├── assets/demo/           Real Octopus App screenshots for showcase pages.
│   ├── pet.html               Pixel Octopus read-only HTML preview.
│   ├── use.html               Five-minute product use guide.
│   ├── tutorial.html          Product tutorial.
│   ├── recipes.html           Goal-first usage recipes.
│   ├── quickstart.html/md     Install and launch guides.
│   ├── about.html             Product story page.
│   ├── architecture.md        Architecture notes.
│   ├── field-adaptation.md    Field adaptation TODO and `0.2.x -> 0.3.0` path.
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
| Stable kernel | Goal/Need/Feed contracts, state, route scores, memory, provider client, Feed traces, evolution data, field-pool status snapshots, resolved field-pack evolution targets, provider patch target checks, and LLM-native repair/evolve contracts | `crates/octopus-core/src/lib.rs` | 15,645 |
| User surface boundary | Converts internal state into human Goal hints; keeps agent commands in observer fields so app/CLI users edit Goal only | `user_surface.rs`, `StatusReport`, `ContextReport`, `NeedQueueReport`, `FieldPoolStatusReport`, `ProductReport` | 123 |
| Evolution contract | Manifest-owned surface requirements, required-surface validation, candidate ID normalization, LLM retry guardrails without field-specific Rust branches | `evolution.rs`, `tentacles/*/manifest.json`, `tentacles/tentacle.schema.json` | 471 |
| Field adaptation core | Field-pack loading, matching, editable aliases, multilingual alias signals, Need annotation, structured peer-field queue context, trace metadata, peer-field worker slots, verifier results, field trajectory summaries, live field mini task loader, editable field-pack task surfaces with concrete pack and registry target files, repair templates, and compile/execute template checks | `field_pack.rs`, `field-packs/**`, `tentacles/field-mini-task/**`, `docs/field-adaptation.md` | 4,012 |
| CLI and product backend | Command dispatch, Goal/chat/brain, provider setup, doctor/report/preflight aggregation, starter/install/check flows, field summary, repair/evolve commands, strategy diagnostics entry | `crates/octopus-core/src/main.rs` | 36,153 |
| Local app bridge | Local HTTP/SSE server, app policy, command allow-list, embedded app/docs/showcase assets, field activity observer with fresh pet-event handling | `app_bridge.rs`, `docs/app.html` | 1,988 |
| Release and install gates | Release records, benchmark evidence, download/install manifest, real-machine checks | `release_gate.rs`, `download.rs`, `docs/real-machine-test.md`, `docs/download.json`, `docs/install.sh` | 1,211 |
| Pet and visual state | Pixel Octopus state, SVG/export helpers, unified event writes, native read-only observer, desktop source preflight, HTML preview | `pet.rs`, `pet_events.rs`, `desktop_pet.rs`, `desktop/pet/OctopusDesktopPet.swift`, `docs/pet.html`, `tentacles/visual/manifest.json` | 2,158 |
| Strategy diagnostics | Checks the three core traits, user-surface policy, and desktop observer chain: clean brain context, LLM tool-side tentacles, editable goal, field surface, pet freshness, Feed/evolution evidence | `diagnostics.rs`, `octopus diagnose strategy --json` | 388 |
| Product docs/site | README, landing/showcase/tutorial/use/recipes/about/docs pages | `README*`, `docs/*.html`, `docs/*.md`, `docs/zh/*` | 8,178 |
| Editable tentacles | Code-as-harness Feed suppliers: prompts, manifests, tools, declared evolution requirements, field-pack task targets, repair templates, repair surfaces | `tentacles/**` | 18,268 |

## Core, Distinctive, Editable

### Core Functions

These should remain stable and hard to accidentally mutate:

- Clean-brain contract: `Goal + Mem + Need + Feed`.
- Need to Feed transport and trace records.
- Field-pack loading and Need/Feed trace annotation.
- Structured queued-Need context for peer-field worker slots, with compact field/task labels visible in CLI and the local app.
- Feed trace metadata preserves field context from Need context when a tentacle omits it.
- Parallel worker records expose the active mini task in CLI, JSON, and the native pet.
- Parallel worker next actions follow queued, satisfied, and repair-needed outcomes.
- `status` JSON exposes a read-only `field_pool` snapshot for peer slots, completion, active slot, mini-task progress, latest status, latest worker update time, and next action.
- Field trajectory summaries for parallel field pool learning.
- Product report field-pool block: required v0.2 peer fields plus expansion packs such as write and translate, latest worker execution-slot count, latest field activity, active slot, and worker-slot policy.
- Route scoring, provider clients, memory/heartbeat state, grants, permissions.
- Product bridge rule: user-facing writes go through Goal; internal actions feed the agent.
- User surface rule: `next` on status/context/Need queue/field pool/product report means human Goal hint; internal execution commands stay in `agent_next` or `agent_next_action`.
- Release gates: preflight, benchmark evidence, field-pool visibility, real-machine records, local app readiness.
- Strategy diagnostics: `octopus diagnose strategy --json` checks clean-brain context, user-surface command leakage, tool-side LLM tentacles, editable Goal, field-pack evolution surface, fresh pet events, and Feed/evolution evidence.

Where they live:

- `crates/octopus-core/src/lib.rs`
- `crates/octopus-core/src/main.rs`
- `crates/octopus-core/src/app_bridge.rs`
- `crates/octopus-core/src/release_gate.rs`
- `crates/octopus-core/src/core_boundary.rs`
- `crates/octopus-core/src/evolution.rs`
- `crates/octopus-core/src/user_surface.rs`
- `crates/octopus-core/src/diagnostics.rs`
- `crates/octopus-core/src/pet_events.rs`

### Distinctive Functions

These are the project identity:

- Clean brain: the main model asks for Need, not tool choreography.
- Thinking tentacles: tool-side LLM + tool meta + executable harness code.
- Need -> Feed -> Feedback loop with compact Feed traces.
- Three-heart surface: heartbeat, memory beat, harness beat. Harness beat can write evolution artifacts only through a configured LLM planner; local rule candidates are removed.
- Pixel Octopus state and color-changing pet.
- Native desktop observer: reads `.octopus/state.json`, shows Goal, transient Need bubble, independent action/Feed bubbles, Feed/evolution/blocked colors, and maps each active worker window to its own queued Need, field/task label, worker status color, and work bubbles.
- Native desktop observer can also read the serialized peer field pool from `.octopus/state.json`, show the active field first, use worker update timestamps for transient work bubbles, expand to the latest run worker count only while that run is active, accept only fresh pet events for color/state, include worker policy, requested worker capacity, active slots, and candidate field pool in the click-open Goal sheet, fall back to legacy traces and latest run policy for old state files, and keep this as observation rather than a control surface.
- HTML pet page: read-only preview for docs and screenshots.
- Release showcase showing real app output, not a fake mock.

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
- Manifest-owned evolution requirements: domain/harness data declares which objective signals require which editable surfaces.
- Seed profile registry.
- Harness repair sessions and adapter probes.
- Field mini task trajectory adapters, live task-template loading, editable `field_pack_tasks` targets that resolve to concrete pack files plus `field-packs/index.json`, task-specific repair templates, and compile/execute template checks.

Where they live now:

- `tentacles/`
- `field-packs/`
- `.octopus/` at runtime
- `docs/field-adaptation.md` for the `0.2.x` field expansion TODO and `v0.3.0` path

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
  write/
  translate/
```

## Code Size

### Top-Level Areas

| Area | Files | Lines |
| --- | ---: | ---: |
| `crates/octopus-core/src` | 16 | 56,579 |
| `crates/octopus-core/examples` | 1 | 27 |
| `tentacles` | 70 | 18,268 |
| `field-packs` | 14 | 583 |
| `desktop/pet` | 1 | 844 |
| `docs` | 29 md/html/json/sh files | 7,923 |
| `cowork` | 3 | 101 |
| `local/docs` | 12 | 507 |

### Core Rust Files

| File | Lines |
| --- | ---: |
| `main.rs` | 36,153 |
| `lib.rs` | 15,645 |
| `app_bridge.rs` | 1,142 |
| `release_gate.rs` | 703 |
| `field_pack.rs` | 868 |
| `diagnostics.rs` | 388 |
| `pet.rs` | 280 |
| `bundled_harness.rs` | 411 |
| `download.rs` | 175 |
| `desktop_pet.rs` | 368 |
| `core_boundary.rs` | 123 |
| `user_surface.rs` | 123 |
| `evolution.rs` | 78 |
| `profile_registry.rs` | 78 |
| `pet_events.rs` | 26 |
| `shell_words.rs` | 18 |

### Tentacles

| Tentacle | Files | Lines |
| --- | ---: | ---: |
| `harness-repair-agent` | 6 | 12,202 |
| `repo-maintainer` | 8 | 719 |
| `computer-use-agent` | 10 | 644 |
| `profile-registry` | 1 | 600 |
| `field-mini-task` | 34 | 3,410 |
| `swe-agent` | 6 | 217 |
| `json-feed` | 2 | 162 |
| `bash-only` | 2 | 77 |
| `visual` | 1 | 56 |
| shared README/schema | 2 | 166 |

## Notes

- The stable core is still too concentrated in `main.rs` and `lib.rs`; `0.2.x` cleanup should split product backend aggregation without moving field behavior back into Rust.
- If desktop Octopus looks wrong, start with `octopus diagnose strategy --json`, then inspect `.octopus/state.json:last_pet_event`, pending Needs, latest Feed trace, latest parallel worker update, and provider/evolution artifact status.
- Evolution surface requirements now belong to manifests. Rust validates declared missing surfaces and must not grow domain-specific trigger rules.
- The next `0.2.x` track should let Octopus add harder mini task layers one field at a time: math, write, translate, search, code, SWE, research, computer-use, IB, robotics.
- The release showcase is screenshot-first. The local app surface in `docs/app.html` observes and updates the real local Octopus loop.
