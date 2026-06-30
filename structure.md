# Structure

Updated: 2026-06-30, after stage-aware evolution cycle events, pet observer process/stage/error diagnostics, provider env loader, Codex provider timeout fix, and patch apply normalization.

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
│       ├── lib.rs             Kernel contracts, state, routes, providers, traces, patch normalization.
│       ├── main.rs            CLI/product backend aggregation and command dispatch.
│       ├── app_bridge.rs      Local app bridge, HTTP/SSE, `/api/config`, embedded assets, bridge policy.
│       ├── bundled_harness.rs Installed-binary seed harness materializer, including field-mini-task and fixtures.
│       ├── core_boundary.rs   Stable-core vs editable-harness boundary diagnostics.
│       ├── diagnostics.rs     Strategy diagnostics for clean brain, intelligent tentacles, editable goal, pet supervision.
│       ├── download.rs        Download/install manifest and artifact checks.
│       ├── desktop_pet.rs     macOS read-only desktop pet launcher.
│       ├── evolution.rs       Manifest-owned evolution surface requirement validation.
│       ├── evolution_apply.rs Authorized provider patch apply/check execution for harness evolution.
│       ├── evolution_cycle.rs Stage/error contract for autonomous evolution and pet-visible diagnostics.
│       ├── evolution_driver.rs One-cycle autonomous harness evolution driver: LLM recommendation, patch apply, check, Need/Feed, optional desktop pet.
│       ├── field_pack.rs      Field-pack loader, matcher, and Need/trace metadata.
│       ├── pet.rs             Pixel Octopus state, SVG export, file URL helpers.
│       ├── pet_events.rs      Unified state event write path and JSONL audit log for desktop/pet observers.
│       ├── pet_supervision.rs Desktop pet observation-chain diagnostics: state file, event log, last event, freshness.
│       ├── profile_registry.rs Seed profile registry loading and status.
│       ├── provider_env.rs    Scoped `.octopus/llm.env` loader; fills missing provider variables without overriding shell env.
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
| Stable kernel | Goal/Need/Feed contracts, state, route scores, memory, provider client, Feed traces, evolution data, field-pool status snapshots, resolved field-pack evolution targets, provider patch target checks, LLM patch normalization, missing `new file mode` normalization for provider-created files, Codex CLI prompt-file stdin with timeout enforcement, and LLM-native repair/evolve contracts | `crates/octopus-core/src/lib.rs` | 15,758 |
| User surface boundary | Converts internal state into human Goal hints; keeps agent commands in observer fields so app/CLI users edit Goal only | `user_surface.rs`, `StatusReport`, `ContextReport`, `NeedQueueReport`, `FieldPoolStatusReport`, `ProductReport` | 123 |
| Evolution contract | Manifest-owned surface requirements, required-surface validation, candidate ID normalization, LLM retry guardrails without field-specific Rust branches | `evolution.rs`, `tentacles/*/manifest.json`, `tentacles/tentacle.schema.json` | 471 |
| Evolution apply boundary | Authorized provider patch application, `git apply --check`, reverse already-applied detection, safety tests, and live apply reports shared by CLI apply and autonomous drive | `evolution_apply.rs` | 221 |
| Evolution cycle contract | Stage and error-class event contract for autonomous harness evolution. Pet observers now see whether a block happened in planning, applying, checking, or feeding, and whether it was provider timeout, provider error, patch apply, or check failure. | `evolution_cycle.rs`, `PetEvent.stage`, `PetEvent.error_class` | 153 |
| Evolution driver | Autonomous harness loop for one cycle: mark pet state, ask LLM for candidate, write artifacts, apply authorized patch, retry once with `git apply` failure feedback, run manifest-declared checks, optionally run field Need -> Feed and launch desktop pet observer. This still owns too many surfaces and is the next split target. | `evolution_driver.rs` | 557 |
| Field adaptation core | Field-pack loading, matching, editable aliases, multilingual alias signals, Need annotation, structured peer-field queue context, trace metadata, peer-field worker slots, verifier results, field trajectory summaries, live field mini task loader, editable field-pack task surfaces with concrete pack and registry target files, repair templates, mini task schema guard, LLM template result normalization, and compile/execute template checks | `field_pack.rs`, `field-packs/**`, `tentacles/field-mini-task/**`, `docs/field-adaptation.md` | 5,821 |
| CLI and product backend | Command dispatch, Goal/chat/brain, provider setup, doctor/report/preflight aggregation, starter/install/check flows, field summary, repair/evolve commands, strategy diagnostics entry, pet event log and supervision viewer | `crates/octopus-core/src/main.rs` | 36,327 |
| Provider env | Loads `.octopus/llm.env` for CLI commands with a scoped guard; existing shell variables win, and values are restored after command execution | `provider_env.rs`, `app_bridge.rs::parse_env_overlay` | 107 |
| Local app bridge | Local HTTP/SSE server, `/api/config` state-path injection, app policy, command allow-list, embedded app/docs/showcase assets, read-only pet supervision access, field activity observer with fresh pet-event handling | `app_bridge.rs`, `docs/app.html` | 2,027 |
| Release and install gates | Release records, benchmark evidence, download/install manifest, real-machine checks | `release_gate.rs`, `download.rs`, `docs/real-machine-test.md`, `docs/download.json`, `docs/install.sh` | 1,211 |
| Pet and visual state | Pixel Octopus state, SVG/export helpers, unified event writes, JSONL event audit, latest-event audit coverage, native read-only observer, desktop process/state-path check, stage/error diagnostics, desktop source preflight, HTML preview. Native pet no longer treats stale Feed traces as live state; colors come from fresh pet events, pending Needs, active worker slots, verifier status, goal status, heartbeat, or memory. | `pet.rs`, `pet_events.rs`, `pet_supervision.rs`, `desktop_pet.rs`, `desktop/pet/OctopusDesktopPet.swift`, `docs/pet.html`, `tentacles/visual/manifest.json` | 2,744 |
| Strategy diagnostics | Checks the three core traits and composes pet supervision: clean brain context, LLM tool-side tentacles, editable goal, field surface, observation-chain freshness, native desktop process/state-path evidence, stage/error evidence, JSONL latest-event coverage, Feed/evolution evidence | `diagnostics.rs`, `pet_supervision.rs`, `octopus diagnose strategy --json`, `octopus pet supervise --json` | 872 |
| Product docs/site | README, landing/showcase/tutorial/use/recipes/about/docs pages | `README*`, `docs/*.html`, `docs/*.md`, `docs/zh/*` | 8,178 |
| Editable tentacles | Code-as-harness Feed suppliers: prompts, manifests, tools, declared evolution requirements, field-pack task targets, repair templates, repair surfaces | `tentacles/**` | 18,974 |

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
- Evolution observation rule: `evolve drive` writes `PetEvent.stage` and `PetEvent.error_class` through `evolution_cycle.rs`; a stuck pet should identify planning/provider, applying/patch, checking, or feeding instead of only showing `blocked`.
- Desktop observer rule: stale Feed traces cannot drive the native pet's current color or Need bubble. A Feed/action color must come from a fresh `last_pet_event` or an active worker update.
- Provider env rule: CLI reads `.octopus/llm.env` through `provider_env.rs`; explicit shell variables are never overwritten.
- Evolution apply rule: provider patches are checked and applied only through `evolution_apply.rs`; CLI apply and autonomous drive share the same authorized patch execution path.
- Evolution driver rule: `evolve drive` uses `evolution_cycle.rs` for stage/error events; `main.rs` only parses CLI and prints the report. Apply-check failures are fed back to the LLM once for a regenerated current-file patch.
- Patch apply rule: provider-created new-file patches are normalized before `git apply`; missing `new file mode 100644` is inserted when a diff uses `--- /dev/null`.
- Provider timeout rule: Codex CLI prompts are passed through a temp prompt file, not blocking `stdin.write_all`; `OCTOPUS_LLM_TIMEOUT` and `OCTOPUS_LLM_RETRIES` can bound harness evolution calls.
- Release gates: preflight, benchmark evidence, field-pool visibility, real-machine records, local app readiness.
- Strategy diagnostics: `octopus diagnose strategy --json` checks clean-brain context, user-surface command leakage, tool-side LLM tentacles, editable Goal, field-pack evolution surface, fresh pet events, desktop process/state-path evidence, stage/error evidence, and Feed/evolution evidence.

### Desktop Pet / Evolution Failure Map

Use this before changing UI or harness code:

| Symptom | First check | Meaning | Fix location |
| --- | --- | --- | --- |
| Pet not visible | `octopus pet supervise --json`, check `desktop_process` | Native observer is not running or is reading another state path | `desktop_pet.rs`, `desktop/pet/OctopusDesktopPet.swift` |
| Pet visible but stale | `event_freshness` | Active Octopus loop is not writing fresh events | caller that should use `pet_events` or `evolution_cycle.rs` |
| Pet red/blocked during planning | `last_event.stage=planning`, `error_class=provider_timeout/provider_error` | LLM planner did not return a harness candidate | provider config/client and evolution planner boundary |
| Pet red/blocked during applying | `stage=applying`, `error_class=patch_*` | LLM produced an invalid, missing, or unauthorized patch | apply/patch normalization boundary |
| Pet red/blocked during checking | `stage=checking`, `error_class=check_failed` | Harness patch applied but its declared check failed | tentacle check policy or editable harness code |
| Pet says fed with no real work | compare `auto_feed.ran`, queued Need count, and field summary | The driver reached Feed surface without useful queued work | `evolution_driver.rs` feed boundary; remove fake-empty success paths |

Where they live:

- `crates/octopus-core/src/lib.rs`
- `crates/octopus-core/src/main.rs`
- `crates/octopus-core/src/app_bridge.rs`
- `crates/octopus-core/src/release_gate.rs`
- `crates/octopus-core/src/core_boundary.rs`
- `crates/octopus-core/src/evolution.rs`
- `crates/octopus-core/src/evolution_apply.rs`
- `crates/octopus-core/src/evolution_cycle.rs`
- `crates/octopus-core/src/user_surface.rs`
- `crates/octopus-core/src/diagnostics.rs`
- `crates/octopus-core/src/pet_events.rs`
- `crates/octopus-core/src/pet_supervision.rs`

### Known Refactor Debt

These remain intentionally visible:

- `evolution_driver.rs` still owns CLI-ish report formatting, retry policy, check selection, desktop launch, and field-mini-task feeding. Next split should move retry, check selection, and field feeding into separate modules.
- `lib.rs` still owns LLM evolution prompt construction, target-file context budget, candidate target parsing, artifact writing, patch authorization, and diff normalization. These belong outside the stable kernel once public API boundaries are safe.
- Automatic `cargo test` next step and empty Feed batch reporting are still suspicious for LLM-native self-evolution and should be removed or pushed into editable manifest/harness policy.

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
  code/       mini-1..4
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
| `crates/octopus-core/src` | 21 | 58,482 |
| `crates/octopus-core/examples` | 1 | 27 |
| `tentacles` | 83 | 18,974 |
| `field-packs` | 14 | 598 |
| `desktop/pet` | 1 | 841 |
| `docs` | 29 md/html/json/sh files | 7,937 |
| `cowork` | 3 | 101 |
| `local/docs` | 12 | 507 |

### Core Rust Files

| File | Lines |
| --- | ---: |
| `main.rs` | 36,327 |
| `lib.rs` | 15,758 |
| `app_bridge.rs` | 1,167 |
| `field_pack.rs` | 868 |
| `release_gate.rs` | 703 |
| `evolution_driver.rs` | 557 |
| `pet_supervision.rs` | 518 |
| `bundled_harness.rs` | 423 |
| `desktop_pet.rs` | 377 |
| `diagnostics.rs` | 354 |
| `pet.rs` | 280 |
| `download.rs` | 175 |
| `evolution_apply.rs` | 221 |
| `evolution_cycle.rs` | 153 |
| `core_boundary.rs` | 123 |
| `user_surface.rs` | 123 |
| `provider_env.rs` | 107 |
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
| `field-mini-task` | 45 | 4,116 |
| `swe-agent` | 6 | 217 |
| `json-feed` | 2 | 162 |
| `bash-only` | 2 | 77 |
| `visual` | 1 | 56 |
| shared README/schema | 2 | 166 |

## Notes

- The stable core is still too concentrated in `main.rs` and `lib.rs`; `0.2.x` cleanup should split product backend aggregation without moving field behavior back into Rust.
- If desktop Octopus looks wrong, start with `octopus pet supervise --json`. It separates missing state file, missing/corrupt event log, absent last event, invalid event state, and stale event freshness. Then use `octopus diagnose strategy --json` for full project-policy status.
- If the pet stays red with no Need bubble, inspect `.octopus/state.json.last_pet_event`, `.octopus/pet-events.jsonl`, `.octopus/state.json.need_queue`, and `.octopus/state.json.parallel_evolution_runs`. A red idle pet means the latest real event is blocked or stale, not a desktop control failure.
- If CLI LLM works in the app but not terminal, inspect `.octopus/llm.env` and `provider_env.rs`. The CLI now auto-loads missing OCTOPUS provider variables from that file.
- If LLM evolution apply fails on new files, inspect `.octopus/evolution/<tentacle>/apply/*.patch`; the normalizer in `lib.rs` must preserve `/dev/null` and insert `new file mode 100644`.
- If `evolve drive` stays in planning, run with explicit bounds such as `OCTOPUS_LLM_TIMEOUT=60 OCTOPUS_LLM_RETRIES=0`. A timeout must become a fresh `blocked` pet event; if it does not, inspect `CodexCliChatClient` in `lib.rs`.
- Evolution surface requirements now belong to manifests. Rust validates declared missing surfaces and must not grow domain-specific trigger rules.
- Current field-mini-task template coverage is 33/33 satisfied in source and bundled seed. The apply path now normalizes LLM patch file headers before `git apply`, runs an apply precheck first, and field-mini-task normalizes common LLM template result shapes into `octopus-json-v1` Feed. The next `0.2.x` track should let Octopus add harder mini task layers one field at a time: math, search, SWE, research, computer-use, IB, robotics, then continue write and translate.
- The release showcase is screenshot-first. The local app surface in `docs/app.html` observes and updates the real local Octopus loop.
