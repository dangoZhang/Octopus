# Structure

Updated: 2026-06-30, after write/translate harder-layer harness and pet audit sync landed.

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
│       ├── app_bridge.rs      Local app bridge, HTTP/SSE, `/api/config`, embedded assets, bridge policy.
│       ├── bundled_harness.rs Installed-binary seed harness materializer, including field-mini-task and fixtures.
│       ├── core_boundary.rs   Stable-core vs editable-harness boundary diagnostics.
│       ├── diagnostics.rs     Strategy diagnostics for clean brain, intelligent tentacles, editable goal, pet supervision.
│       ├── download.rs        Download/install manifest and artifact checks.
│       ├── desktop_pet.rs     macOS read-only desktop pet launcher.
│       ├── evolution.rs       Manifest-owned evolution surface requirement validation.
│       ├── field_pack.rs      Field-pack loader, matcher, and Need/trace metadata.
│       ├── pet.rs             Pixel Octopus state, SVG export, file URL helpers.
│       ├── pet_events.rs      Unified state event write path and JSONL audit log for desktop/pet observers.
│       ├── pet_supervision.rs Desktop pet observation-chain diagnostics: state file, event log, last event, freshness.
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
| Field adaptation core | Field-pack loading, matching, editable aliases, multilingual alias signals, Need annotation, structured peer-field queue context, trace metadata, peer-field worker slots, verifier results, field trajectory summaries, live field mini task loader, editable field-pack task surfaces with concrete pack and registry target files, repair templates, mini task schema guard, and compile/execute template checks | `field_pack.rs`, `field-packs/**`, `tentacles/field-mini-task/**`, `docs/field-adaptation.md` | 5,368 |
| CLI and product backend | Command dispatch, Goal/chat/brain, provider setup, doctor/report/preflight aggregation, starter/install/check flows, field summary, repair/evolve commands, strategy diagnostics entry, pet event log and supervision viewer | `crates/octopus-core/src/main.rs` | 36,358 |
| Local app bridge | Local HTTP/SSE server, `/api/config` state-path injection, app policy, command allow-list, embedded app/docs/showcase assets, read-only pet supervision access, field activity observer with fresh pet-event handling | `app_bridge.rs`, `docs/app.html` | 2,027 |
| Release and install gates | Release records, benchmark evidence, download/install manifest, real-machine checks | `release_gate.rs`, `download.rs`, `docs/real-machine-test.md`, `docs/download.json`, `docs/install.sh` | 1,211 |
| Pet and visual state | Pixel Octopus state, SVG/export helpers, unified event writes, JSONL event audit, latest-event audit coverage, native read-only observer, desktop source preflight, HTML preview | `pet.rs`, `pet_events.rs`, `pet_supervision.rs`, `desktop_pet.rs`, `desktop/pet/OctopusDesktopPet.swift`, `docs/pet.html`, `tentacles/visual/manifest.json` | 2,552 |
| Strategy diagnostics | Checks the three core traits and composes pet supervision: clean brain context, LLM tool-side tentacles, editable goal, field surface, observation-chain freshness, JSONL latest-event coverage, Feed/evolution evidence | `diagnostics.rs`, `pet_supervision.rs`, `octopus diagnose strategy --json`, `octopus pet supervise --json` | 703 |
| Product docs/site | README, landing/showcase/tutorial/use/recipes/about/docs pages | `README*`, `docs/*.html`, `docs/*.md`, `docs/zh/*` | 8,178 |
| Editable tentacles | Code-as-harness Feed suppliers: prompts, manifests, tools, declared evolution requirements, field-pack task targets, repair templates, repair surfaces | `tentacles/**` | 18,526 |

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
- App state rule: browser app gets the real bridge state path from `/api/config`; file preview may fall back to `.octopus/state.json`.
- Pet supervision rule: `last_pet_event` is the live state, `.octopus/pet-events.jsonl` is the append-only audit trail, `event_log_contains_last` must pass, and `octopus pet supervise --json` is the first diagnostic for desktop observer issues.
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
- `crates/octopus-core/src/pet_supervision.rs`

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
- Field mini task trajectory adapters, live task-template loading, editable `field_pack_tasks` targets that resolve to concrete pack files plus `field-packs/index.json`, bundled seed templates, task-specific repair templates, canonical mini task schema guards, and compile/execute template checks.

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
  write/      mini-1..3
  translate/  mini-1..3
```

## Code Size

### Top-Level Areas

| Area | Files | Lines |
| --- | ---: | ---: |
| `crates/octopus-core/src` | 17 | 57,180 |
| `crates/octopus-core/examples` | 1 | 27 |
| `tentacles` | 81 | 18,526 |
| `field-packs` | 14 | 593 |
| `desktop/pet` | 1 | 846 |
| `docs` | 29 md/html/json/sh files | 7,937 |
| `cowork` | 3 | 101 |
| `local/docs` | 12 | 507 |

### Core Rust Files

| File | Lines |
| --- | ---: |
| `main.rs` | 36,358 |
| `lib.rs` | 15,645 |
| `app_bridge.rs` | 1,167 |
| `release_gate.rs` | 703 |
| `field_pack.rs` | 868 |
| `bundled_harness.rs` | 419 |
| `desktop_pet.rs` | 368 |
| `diagnostics.rs` | 354 |
| `pet_supervision.rs` | 349 |
| `pet.rs` | 280 |
| `download.rs` | 175 |
| `core_boundary.rs` | 123 |
| `user_surface.rs` | 123 |
| `evolution.rs` | 78 |
| `profile_registry.rs` | 78 |
| `pet_events.rs` | 74 |
| `shell_words.rs` | 18 |

### Tentacles

| Tentacle | Files | Lines |
| --- | ---: | ---: |
| `harness-repair-agent` | 6 | 12,202 |
| `repo-maintainer` | 8 | 719 |
| `computer-use-agent` | 10 | 644 |
| `profile-registry` | 1 | 600 |
| `field-mini-task` | 43 | 3,668 |
| `swe-agent` | 6 | 217 |
| `json-feed` | 2 | 162 |
| `bash-only` | 2 | 77 |
| `visual` | 1 | 56 |
| shared README/schema | 2 | 166 |

## Notes

- The stable core is still too concentrated in `main.rs` and `lib.rs`; `0.2.x` cleanup should split product backend aggregation without moving field behavior back into Rust.
- If desktop Octopus looks wrong, start with `octopus pet supervise --json`. It separates missing state file, missing/corrupt event log, absent last event, invalid event state, and stale event freshness. Then use `octopus diagnose strategy --json` for full project-policy status.
- Evolution surface requirements now belong to manifests. Rust validates declared missing surfaces and must not grow domain-specific trigger rules.
- Current field-mini-task template coverage is 32/32 satisfied in source and bundled seed. The next `0.2.x` track should let Octopus add harder mini task layers one field at a time: math, search, code, SWE, research, computer-use, IB, robotics, then continue write and translate.
- The release showcase is screenshot-first. The local app surface in `docs/app.html` observes and updates the real local Octopus loop.
