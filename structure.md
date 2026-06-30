# Structure

Updated: 2026-06-30, after modular evolution/pet diagnostics refactor and LLM evolution planner/contract/recommend/prompt/candidate/artifact/patch/target plus tentacle scaffold, brain loop, Need queue, state-report, and LLM provider extraction.

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
│       ├── lib.rs             Kernel contracts, state mutation, routes, traces, public kernel API.
│       ├── main.rs            CLI/product backend aggregation and command dispatch.
│       ├── app_bridge.rs      Local app bridge, HTTP/SSE, `/api/config`, embedded assets, bridge policy.
│       ├── brain_loop.rs      Clean-brain Goal/Mem/Need/Feed loop and Need report builders.
│       ├── bundled_harness.rs Installed-binary seed harness materializer, including field-mini-task and fixtures.
│       ├── core_boundary.rs   Stable-core vs editable-harness boundary diagnostics.
│       ├── diagnostics.rs     Strategy diagnostics for clean brain, intelligent tentacles, editable goal, pet supervision.
│       ├── download.rs        Download/install manifest and artifact checks.
│       ├── desktop_pet.rs     macOS read-only desktop pet launcher.
│       ├── evolution.rs       Manifest-owned evolution surface requirement validation.
│       ├── evolution_apply.rs Authorized provider patch apply/check execution for harness evolution.
│       ├── evolution_artifact.rs Evolution proposal/apply artifact writing and markdown rendering.
│       ├── evolution_candidate.rs LLM response parsing, candidate target validation, target-file recovery, and patch draft metadata.
│       ├── evolution_contract.rs Public evolution policy, proposal, candidate, apply, artifact, and harness-beat data contracts.
│       ├── evolution_cycle.rs Stage/error contract for autonomous evolution and pet-visible diagnostics.
│       ├── evolution_drive_surface.rs CLI/report surface for `evolve drive`.
│       ├── evolution_driver.rs One-cycle autonomous harness evolution driver: LLM recommendation, patch apply, check, Need/Feed, optional desktop pet.
│       ├── evolution_feed.rs  Feed-stage runner for field mini-task evolution, worker state, and desktop observer launch.
│       ├── evolution_llm_plan.rs LLM harness-planner chat messages, retry note, and JSON plan parsing.
│       ├── evolution_patch.rs Provider patch normalization, diff path extraction, and apply allow-list filtering.
│       ├── evolution_plan.rs  LLM patch recommendation artifacts and apply-failure retry objective shaping.
│       ├── evolution_prompt.rs LLM evolution prompt, compact trace/history payloads, and target-file context budget.
│       ├── evolution_recommend.rs Proposal assembly, candidate scoring, apply-plan selection, and harness-beat artifact orchestration.
│       ├── evolution_target.rs Manifest/field-pack/repair-template target resolution for harness evolution.
│       ├── field_pack.rs      Field-pack loader, matcher, and Need/trace metadata.
│       ├── llm_provider.rs    Shared ChatClient, OpenAI-compatible, Codex CLI, retry/timeout request layer.
│       ├── need_queue.rs      Queued Need state mutation and queue report boundary.
│       ├── pet.rs             Pixel Octopus state, SVG export, file URL helpers.
│       ├── pet_events.rs      Unified state event write path and JSONL audit log for desktop/pet observers.
│       ├── pet_supervision.rs Desktop pet observation-chain diagnostics: state file, event log, last event, freshness.
│       ├── profile_registry.rs Seed profile registry loading and status.
│       ├── provider_env.rs    Scoped `.octopus/llm.env` loader; fills missing provider variables without overriding shell env.
│       ├── release_gate.rs    Preflight records, benchmark and real-machine gates.
│       ├── shell_words.rs     Shared shell command display quoting.
│       ├── state_report.rs    Read-only status/context/field pool report builder.
│       ├── tentacle_scaffold.rs User-owned code-as-harness tentacle scaffold generation.
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
| Stable kernel | Goal/Need/Feed contracts, state mutation, route scores, memory, Feed traces, manifest tentacle execution, and public kernel API | `crates/octopus-core/src/lib.rs` | 10,433 |
| Clean brain loop boundary | Converts Goal/Mem/recent Feed into clean cognitive Needs and Goal refinement reports. This is the debug entry when Brain output leaks tool choreography or over-specific implementation steps. | `brain_loop.rs`, `BrainExploreReport`, `BrainGoalReport`, `BrainContextReport` | 1,082 |
| LLM provider boundary | Shared model request substrate for brain and tentacle intelligence: ChatClient contract, OpenAI-compatible config/body parsing, Codex CLI prompt-file execution, retries, timeout, and Plan JSON recovery. This is the debug entry when model calls, local model routing, or provider response parsing fail. | `llm_provider.rs`, `ChatClient`, `OpenAiCompatibleChatClient`, `CodexCliChatClient`, `ChatPlanner` | 704 |
| Need queue boundary | Owns queued Need creation, take/drop, queue status, and queue report separation between human Goal hints and agent commands. | `need_queue.rs`, `NeedQueueItem`, `NeedQueueReport` | 234 |
| State report boundary | Builds read-only status, context, field trajectory, field pool, and harness-learning reports from `HarnessState`. This is the debug entry when app output, field pool state, or user-vs-agent next actions look wrong. | `state_report.rs`, `StatusReport`, `ContextReport`, `FieldPoolStatusReport` | 734 |
| User surface boundary | Converts internal state into human Goal hints; keeps agent commands in observer fields so app/CLI users edit Goal only | `user_surface.rs`, `StatusReport`, `ContextReport`, `NeedQueueReport`, `FieldPoolStatusReport`, `ProductReport` | 123 |
| Evolution contract | Manifest-owned surface requirements, required-surface validation, candidate ID normalization, LLM retry guardrails without field-specific Rust branches | `evolution.rs`, `tentacles/*/manifest.json`, `tentacles/tentacle.schema.json` | 471 |
| Evolution apply boundary | Authorized provider patch application, `git apply --check`, reverse already-applied detection, summary helpers, safety tests, and live apply reports shared by CLI apply and autonomous drive | `evolution_apply.rs` | 282 |
| Evolution artifact boundary | Writes proposal/apply markdown, json, and authorized patch artifacts under `.octopus/evolution/**`; keeps review artifacts separate from planner logic and apply authorization. | `evolution_artifact.rs` | 401 |
| Evolution candidate boundary | Parses provider JSON, validates LLM candidate fields and targets, recovers target files from provider diffs, builds patch-draft metadata, and keeps planner-output errors separate from stable kernel state. | `evolution_candidate.rs` | 233 |
| Evolution data contract | Public data shapes for evolution policy, surfaces, requirements, proposals, file targets, patch candidates, feedback, apply plans, artifacts, recommendations, and harness-beat reports. It owns serde defaults for manifest evolution surfaces and keeps contract shape separate from planner/apply execution. | `evolution_contract.rs` | 222 |
| Evolution cycle contract | Stage and error-class event contract for autonomous harness evolution. Pet observers now see whether a block happened in planning, applying, checking, or feeding, and whether it was provider timeout, provider error, patch apply, or check failure. | `evolution_cycle.rs`, `PetEvent.stage`, `PetEvent.error_class` | 153 |
| Evolution LLM planner boundary | Builds the harness-planner chat request, preserves the clean-brain/tentacle context policy prompt, retries once with validator feedback, and parses provider JSON into candidate plans. | `evolution_llm_plan.rs` | 52 |
| Evolution planning boundary | LLM patch recommendation artifact writing and apply-failure retry objective shaping | `evolution_plan.rs` | 84 |
| Evolution prompt boundary | Builds the LLM planner payload from proposal, trace, check-history, file-target, surface, and prompt-budget data. It owns context compaction for harness evolution while `lib.rs` keeps the public proposal/candidate contract. | `evolution_prompt.rs` | 254 |
| Evolution recommendation boundary | Assembles proposal context from manifests and state, scores candidates from outcomes/Feed traces/check history, builds apply plans, and writes harness-beat proposal/apply artifacts. This is where a wrong candidate choice or missing harness-beat artifact should be debugged. | `evolution_recommend.rs` | 679 |
| Evolution drive surface | CLI args, JSON report shape, and human-readable printing for `evolve drive` | `evolution_drive_surface.rs` | 115 |
| Evolution driver | Autonomous harness loop for one cycle: stage events, planning, apply, manifest checks, and Feed delegation. It no longer owns CLI/report shape, patch execution, Feed execution, or retry objective shaping. | `evolution_driver.rs` | 318 |
| Evolution Feed boundary | Field mini-task Feed execution, queued Need worker state, desktop pet launch, Feed event writes, and blocked reporting when no queued Need actually reaches Feed | `evolution_feed.rs` | 130 |
| Evolution patch boundary | Normalizes provider patch text, inserts missing new-file headers, extracts diff paths, renders display paths, and filters authorized patch paths before artifact/apply code can write a patch file. | `evolution_patch.rs` | 245 |
| Evolution target boundary | Resolves manifest editable targets, field-pack targets, repair-template wildcard scopes, candidate target files, and field-name scopes for patch/prompt/candidate modules. | `evolution_target.rs` | 318 |
| Field adaptation core | Field-pack loading, matching, editable aliases, multilingual alias signals, Need annotation, structured peer-field queue context, trace metadata, peer-field worker slots, verifier results, field trajectory summaries, live field mini task loader, editable field-pack task surfaces with concrete pack and registry target files, repair templates, mini task schema guard, LLM template result normalization, and compile/execute template checks | `field_pack.rs`, `field-packs/**`, `tentacles/field-mini-task/**`, `docs/field-adaptation.md` | 5,821 |
| CLI and product backend | Command dispatch, Goal/chat/brain, provider setup, doctor/report/preflight aggregation, starter/install/check flows, field summary, repair/evolve commands, strategy diagnostics entry, pet event log and supervision viewer | `crates/octopus-core/src/main.rs` | 36,317 |
| Provider env | Loads `.octopus/llm.env` for CLI commands with a scoped guard; existing shell variables win, and values are restored after command execution | `provider_env.rs`, `app_bridge.rs::parse_env_overlay` | 107 |
| Local app bridge | Local HTTP/SSE server, `/api/config` state-path injection, app policy, command allow-list, embedded app/docs/showcase assets, read-only pet supervision access, field activity observer with fresh pet-event handling | `app_bridge.rs`, `docs/app.html` | 2,027 |
| Release and install gates | Release records, benchmark evidence, download/install manifest, real-machine checks | `release_gate.rs`, `download.rs`, `docs/real-machine-test.md`, `docs/download.json`, `docs/install.sh` | 1,211 |
| Pet and visual state | Pixel Octopus state, SVG/export helpers, unified event writes, JSONL event audit, latest-event audit coverage, native read-only observer, desktop process/state-path check, stage/error diagnostics, desktop source preflight, HTML preview. Native pet no longer treats stale Feed traces as live state; colors come from fresh pet events, pending Needs, active worker slots, verifier status, goal status, heartbeat, or memory. | `pet.rs`, `pet_events.rs`, `pet_supervision.rs`, `desktop_pet.rs`, `desktop/pet/OctopusDesktopPet.swift`, `docs/pet.html`, `tentacles/visual/manifest.json` | 2,744 |
| Strategy diagnostics | Checks the three core traits and composes pet supervision: clean brain context, LLM tool-side tentacles, editable goal, field surface, observation-chain freshness, native desktop process/state-path evidence, stage/error evidence, JSONL latest-event coverage, Feed/evolution evidence | `diagnostics.rs`, `pet_supervision.rs`, `octopus diagnose strategy --json`, `octopus pet supervise --json` | 872 |
| Tentacle scaffold boundary | Generates user-owned code-as-harness tentacle manifests and optional python/node/shell seed tools. This is the debug entry when `octopus scaffold` creates an invalid manifest, missing executable, or wrong `octopus-json-v1` contract. | `tentacle_scaffold.rs`, `octopus scaffold` | 291 |
| Product docs/site | README, landing/showcase/tutorial/use/recipes/about/docs pages | `README*`, `docs/*.html`, `docs/*.md`, `docs/zh/*` | 8,085 |
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
- Route scoring, memory/heartbeat state, grants, permissions.
- Clean brain loop rule: `brain_loop.rs` may produce Goal reports and Need suggestions, but its Brain context must stay `Goal + Mem + Need + Feed`.
- LLM provider rule: `llm_provider.rs` owns shared model request, retry, timeout, response-content extraction, and Plan JSON recovery. Brain/tentacle modules may call this boundary but should not hand-roll provider requests.
- Need queue rule: `need_queue.rs` owns queued Need state changes; queue reports must keep `next` user-facing and put executable commands in `agent_next`.
- State report rule: `state_report.rs` is read-only; it may derive status/context/field-pool views from `HarnessState` but must not mutate Goal, Need, Feed, routes, grants, or pet events.
- Product bridge rule: user-facing writes go through Goal; internal actions feed the agent.
- User surface rule: `next` on status/context/Need queue/field pool/product report means human Goal hint; internal execution commands stay in `agent_next` or `agent_next_action`.
- App state rule: browser app gets the real bridge state path from `/api/config`; file preview may fall back to `.octopus/state.json`.
- Pet supervision rule: `last_pet_event` is the live state, `.octopus/pet-events.jsonl` is the append-only audit trail, `event_log_contains_last` must pass, and `octopus pet supervise --json` is the first diagnostic for desktop observer issues.
- Evolution observation rule: `evolve drive` writes `PetEvent.stage` and `PetEvent.error_class` through `evolution_cycle.rs`; a stuck pet should identify planning/provider, applying/patch, checking, or feeding instead of only showing `blocked`.
- Desktop observer rule: stale Feed traces cannot drive the native pet's current color or Need bubble. A Feed/action color must come from a fresh `last_pet_event` or an active worker update.
- Provider env rule: CLI reads `.octopus/llm.env` through `provider_env.rs`; explicit shell variables are never overwritten.
- Provider timeout rule: Codex CLI prompts are passed through a temp prompt file inside `llm_provider.rs`, not blocking `stdin.write_all`; `OCTOPUS_LLM_TIMEOUT` and `OCTOPUS_LLM_RETRIES` bound harness evolution calls.
- Evolution apply rule: provider patches are checked and applied only through `evolution_apply.rs`; CLI apply and autonomous drive share the same authorized patch execution path.
- Evolution artifact rule: local proposal/apply review files are rendered and written only through `evolution_artifact.rs`; artifact rendering must not perform planning or mutate harness code directly.
- Evolution contract rule: policy/proposal/candidate/apply/recommendation/artifact structs live in `evolution_contract.rs`; this module has no provider calls, patch application, file mutation, or route scoring.
- Evolution candidate rule: provider JSON parsing, LLM candidate validation, manifest-relative target parsing, provider-diff target recovery, and patch draft metadata live in `evolution_candidate.rs`; `lib.rs` only asks for a parsed plan.
- Evolution LLM planner rule: planner system prompt, retry note, provider chat call, and provider JSON parsing call live in `evolution_llm_plan.rs`; the core only asks for a plan.
- Evolution planning rule: recommendation artifacts and apply-failure retry objectives live in `evolution_plan.rs`; driver only asks for the next plan.
- Evolution prompt rule: LLM planner payload construction, trace/history compaction, target-file context selection, and prompt-budget env knobs live in `evolution_prompt.rs`; proposal/candidate protocol stays in `lib.rs`.
- Evolution recommendation rule: proposal assembly, scored candidate choice, apply-plan construction, and harness-beat proposal/apply artifact orchestration live in `evolution_recommend.rs`; it consumes real state traces and must not invent success.
- Evolution patch rule: provider patch normalization, diff path extraction, display-path collapsing, and authorized patch allow-list checks live in `evolution_patch.rs`; artifact rendering can only ask for an already authorized patch.
- Evolution target rule: manifest editable targets, field-pack paths, repair-template wildcard scopes, and field-name path extraction live in `evolution_target.rs`; planner/prompt/candidate/patch modules ask this boundary for files.
- Evolution drive surface rule: `evolve drive` CLI/report formatting lives in `evolution_drive_surface.rs`; stage execution stays in `evolution_driver.rs`.
- Evolution driver rule: `evolve drive` uses `evolution_cycle.rs` for stage/error events. Apply-check failures are fed back to the LLM once for a regenerated current-file patch.
- Patch apply rule: provider-created new-file patches are normalized before `git apply`; missing `new file mode 100644` is inserted when a diff uses `--- /dev/null`.
- Release gates: preflight, benchmark evidence, field-pool visibility, real-machine records, local app readiness.
- Tentacle scaffold rule: user-owned code-as-harness seed manifests and python/node/shell starter tools live in `tentacle_scaffold.rs`; new scaffold output must declare LLM brain, tool metadata, runtime code, and editable evolution surfaces.
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
| Pet red/blocked during feeding | `stage=feeding`, `error_class=feed_missing_queued_need` | No queued Need actually reached Feed; the system must not report fake success | `evolution_feed.rs` queued-Need boundary |
| Brain suggests tool steps instead of Needs | inspect `brain/context` report and Need audit | Clean-brain loop leaked implementation details into Need | `brain_loop.rs` |
| LLM call hangs or returns empty/invalid content | inspect provider env and provider response error | Request timeout, retry, response extraction, or JSON recovery failed | `llm_provider.rs`, `provider_env.rs` |
| Queued Need cannot be run/taken/dropped | inspect `needs --json` and item status | Queue mutation or user/agent next split is wrong | `need_queue.rs` |
| App status/context shows wrong next step | compare `next` with `agent_next` in `status/context/needs` JSON | Read-only user hints leaked internal commands, or field-pool state was derived incorrectly | `state_report.rs`, `user_surface.rs` |
| Harness beat suggests the wrong patch | inspect `.octopus/evolution/**/proposal.json` and `.octopus/evolution/**/apply/plan.json` | Candidate scoring or apply-plan selection used the wrong trace/check/outcome evidence | `evolution_recommend.rs` |
| `octopus scaffold` creates a bad tentacle | `octopus manifests <root>` and install/feed tests | Scaffold manifest, seed tool contract, or executable bit is wrong | `tentacle_scaffold.rs` |

Where they live:

- `crates/octopus-core/src/lib.rs`
- `crates/octopus-core/src/main.rs`
- `crates/octopus-core/src/app_bridge.rs`
- `crates/octopus-core/src/brain_loop.rs`
- `crates/octopus-core/src/llm_provider.rs`
- `crates/octopus-core/src/release_gate.rs`
- `crates/octopus-core/src/core_boundary.rs`
- `crates/octopus-core/src/evolution.rs`
- `crates/octopus-core/src/evolution_apply.rs`
- `crates/octopus-core/src/evolution_artifact.rs`
- `crates/octopus-core/src/evolution_candidate.rs`
- `crates/octopus-core/src/evolution_contract.rs`
- `crates/octopus-core/src/evolution_cycle.rs`
- `crates/octopus-core/src/evolution_drive_surface.rs`
- `crates/octopus-core/src/evolution_feed.rs`
- `crates/octopus-core/src/evolution_llm_plan.rs`
- `crates/octopus-core/src/evolution_patch.rs`
- `crates/octopus-core/src/evolution_plan.rs`
- `crates/octopus-core/src/evolution_prompt.rs`
- `crates/octopus-core/src/evolution_recommend.rs`
- `crates/octopus-core/src/evolution_target.rs`
- `crates/octopus-core/src/need_queue.rs`
- `crates/octopus-core/src/state_report.rs`
- `crates/octopus-core/src/tentacle_scaffold.rs`
- `crates/octopus-core/src/user_surface.rs`
- `crates/octopus-core/src/diagnostics.rs`
- `crates/octopus-core/src/pet_events.rs`
- `crates/octopus-core/src/pet_supervision.rs`

### Known Refactor Debt

These remain intentionally visible:

- `evolution_feed.rs` is still field-mini-task specific. The next version should make Feed-stage runners manifest/routing owned so future field packs can choose their own Feed executor.
- `lib.rs` still owns broad state mutation and manifest runtime APIs. The next cleanup should split durable state mutation or manifest tentacle runtime before touching product backend behavior.
- Automatic `cargo test` next step is still suspicious for LLM-native self-evolution and should be removed or pushed into editable manifest/harness policy.

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
| `crates/octopus-core/src` | 37 | 58,763 |
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
| `main.rs` | 36,317 |
| `lib.rs` | 10,433 |
| `app_bridge.rs` | 1,167 |
| `brain_loop.rs` | 1,082 |
| `field_pack.rs` | 868 |
| `state_report.rs` | 734 |
| `llm_provider.rs` | 704 |
| `release_gate.rs` | 703 |
| `evolution_recommend.rs` | 679 |
| `pet_supervision.rs` | 518 |
| `bundled_harness.rs` | 423 |
| `evolution_artifact.rs` | 401 |
| `desktop_pet.rs` | 377 |
| `diagnostics.rs` | 354 |
| `evolution_driver.rs` | 318 |
| `evolution_target.rs` | 318 |
| `tentacle_scaffold.rs` | 291 |
| `evolution_apply.rs` | 282 |
| `pet.rs` | 280 |
| `evolution_prompt.rs` | 254 |
| `evolution_patch.rs` | 245 |
| `need_queue.rs` | 234 |
| `evolution_candidate.rs` | 233 |
| `evolution_contract.rs` | 222 |
| `download.rs` | 175 |
| `evolution_cycle.rs` | 153 |
| `evolution_feed.rs` | 130 |
| `user_surface.rs` | 123 |
| `core_boundary.rs` | 139 |
| `evolution_drive_surface.rs` | 115 |
| `provider_env.rs` | 107 |
| `evolution_plan.rs` | 84 |
| `evolution.rs` | 78 |
| `profile_registry.rs` | 78 |
| `pet_events.rs` | 74 |
| `evolution_llm_plan.rs` | 52 |
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
- If LLM evolution apply fails on new files, inspect `.octopus/evolution/<tentacle>/apply/*.patch`; the normalizer in `evolution_patch.rs` must preserve `/dev/null` and insert `new file mode 100644`.
- If `evolve drive` stays in planning, run with explicit bounds such as `OCTOPUS_LLM_TIMEOUT=60 OCTOPUS_LLM_RETRIES=0`. A timeout must become a fresh `blocked` pet event; if it does not, inspect `evolution_llm_plan.rs`, `llm_provider.rs`, and stage writes in `evolution_cycle.rs`.
- Cleanup audit after the evolution-boundary commits moved prompt, candidate, artifact, patch, target, contract, recommendation, LLM planner, scaffold, clean-brain loop, Need queue, state-report, and LLM provider responsibilities out of `lib.rs`. Remaining discomfort: `main.rs` is still product-backend aggregation, and `lib.rs` still owns broad state mutation/manifest runtime APIs.
- Evolution surface requirements now belong to manifests. Rust validates declared missing surfaces and must not grow domain-specific trigger rules.
- Current field-mini-task template coverage is 33/33 satisfied in source and bundled seed. The apply path now normalizes LLM patch file headers before `git apply`, runs an apply precheck first, and field-mini-task normalizes common LLM template result shapes into `octopus-json-v1` Feed. The next `0.2.x` track should let Octopus add harder mini task layers one field at a time: math, search, SWE, research, computer-use, IB, robotics, then continue write and translate.
- The release showcase is screenshot-first. The local app surface in `docs/app.html` observes and updates the real local Octopus loop.
