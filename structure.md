# Structure

Updated: 2026-07-01, after translate-mini-6 self-repair, verifier artifact propagation, FEED.md status sync, bundled harness coverage sync, and desktop-pet Need/Action/Feed verification.

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
│       ├── brain_goal_surface.rs
│       │                      Brain/Goal command handling, Goal editing, Brain session/report printing, Need audit display.
│       ├── brain_loop.rs      Clean-brain Goal/Mem/Need/Feed loop and Need report builders.
│       ├── bundled_harness.rs Installed-binary seed harness materializer, including field-mini-task and fixtures.
│       ├── core_boundary.rs   Stable-core vs editable-harness boundary diagnostics.
│       ├── diagnostics.rs     Strategy diagnostics for clean brain, intelligent tentacles, editable goal, pet supervision.
│       ├── doctor_surface.rs  Doctor report: environment, manifests, provider readiness, profile registry, and pet page.
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
│       ├── evolution_patch.rs Provider patch normalization, hunk recounting, diff path extraction, and apply allow-list filtering.
│       ├── evolution_plan.rs  LLM patch recommendation artifacts and apply-failure retry objective shaping.
│       ├── evolution_prompt.rs LLM evolution prompt, compact trace/history payloads, and target-file context budget.
│       ├── evolution_recommend.rs Proposal assembly, candidate scoring, apply-plan selection, and harness-beat artifact orchestration.
│       ├── evolution_surface.rs `evolve` command surface, pet event writes, recommend/apply/score/parallel dispatch.
│       ├── evolution_target.rs Manifest/field-pack/repair-template target resolution for harness evolution.
│       ├── feedback_surface.rs Feed feedback command surface, route score outcome rendering, pet event append.
│       ├── field_curriculum.rs Field curriculum selection for the next concrete harder layer.
│       ├── field_pack.rs      Field-pack loader, matcher, and Need/trace metadata.
│       ├── field_surface.rs   Field adaptation command/report surface, verifier scoring, parallel field status lines.
│       ├── heartbeat_surface.rs
│       │                      Three-heart beat command surface, early pet heartbeat event, harness-beat recommendation output.
│       ├── llm_layers.rs      Chat/brain/manifest/evolve LLM layer prefix and client routing.
│       ├── llm_provider.rs    Shared ChatClient, OpenAI-compatible, Codex CLI, retry/timeout request layer.
│       ├── manifest_catalog.rs Manifest/profile data types, loading, inspection, and install conversion.
│       ├── manifest_runtime.rs Manifest-owned tool-side thinking, authorization, execution, and Feed parsing.
│       ├── need_queue.rs      Queued Need state mutation and queue report boundary.
│       ├── need_surface.rs    Need command/report surface: queue display, run output, script/session artifacts.
│       ├── need_runner.rs     Queued Need execution, Feed handoff, field mini-task context, worker events.
│       ├── observation_contract.rs
│       │                      Shared pet-visible evolution stage/error contract and error classifiers.
│       ├── pet.rs             Pixel Octopus state, SVG export, file URL helpers.
│       ├── pet_events.rs      Unified state event write path and JSONL audit log for desktop/pet observers.
│       ├── pet_surface.rs     Pet CLI/report surface: auto state, event log, image/desktop/supervision printing.
│       ├── pet_supervision.rs Desktop pet observation-chain diagnostics: state file, event log, last event, freshness, active work.
│       ├── profile_registry.rs Seed profile registry loading and status.
│       ├── preflight_surface.rs Release preflight report, script/record templates, record audit, and preflight printing.
│       ├── provider_env.rs    Scoped `.octopus/llm.env` loader; fills missing provider variables without overriding shell env.
│       ├── provider_surface.rs Provider profiles, env save/check/status, matrix evidence, and provider client selection.
│       ├── repair_surface.rs  Harness repair command surface, repair plan reports, patch apply/verify, scoring, and repair learning journal.
│       ├── product_surface.rs Product report: capability/gap/next split for user Goal hints and agent actions.
│       ├── release_gate.rs    Preflight records, benchmark and real-machine gates.
│       ├── route_surface.rs   Route/Trace command surface, Feed trace, and Need queue line rendering.
│       ├── shell_words.rs     Shared shell command display quoting.
│       ├── state_report.rs    Read-only status/context/field pool report builder.
│       ├── status_surface.rs  Status, context, goal, tentacle, and harness-learning rendering.
│       ├── strategy_surface.rs Strategy diagnostics report assembly and printing.
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
| Stable kernel | Goal/Need/Feed contracts, state mutation, route scores, memory, Feed traces, and public kernel API | `crates/octopus-core/src/lib.rs` | 9,047 |
| Clean brain loop boundary | Converts Goal/Mem/recent Feed into clean cognitive Needs and Goal refinement reports. This is the debug entry when Brain output leaks tool choreography or over-specific implementation steps. | `brain_loop.rs`, `BrainExploreReport`, `BrainGoalReport`, `BrainContextReport` | 1,082 |
| Brain/Goal command surface | Owns user Goal editing, Brain command validation, Brain session/report dispatch, clean-brain Need audit rendering, and Goal text output. This is the debug entry when the user can change more than Goal, Brain modes leak into the product surface, or Goal/Need audit text is confusing. | `brain_goal_surface.rs`, `handle_goal_command`, `handle_brain_command`, `print_brain_*`, `print_goal` | 1,610 |
| LLM provider boundary | Shared model request substrate for brain and tentacle intelligence: ChatClient contract, OpenAI-compatible config/body parsing, Codex CLI prompt-file execution, retries, timeout, and Plan JSON recovery. This is the debug entry when model calls, local model routing, or provider response parsing fail. | `llm_provider.rs`, `ChatClient`, `OpenAiCompatibleChatClient`, `CodexCliChatClient`, `ChatPlanner` | 704 |
| LLM layer routing | Maps configured provider prefixes into chat, clean-brain, tentacle-planning, and harness-evolution clients. This is the debug entry when the provider works but one Octopus layer uses the wrong prefix or disabled layer. | `llm_layers.rs`, `BrainLlmPrefixOverride`, `manifest_llm_factory`, `clean_brain_*_llm_client` | 300 |
| Provider surface boundary | User-facing provider profiles, `.octopus/llm.env` generation, secret-safe key save, provider check/status, provider matrix evidence, and client factory selection for brain, tentacle, and harness evolution layers. | `provider_surface.rs`, `ProviderProfile`, `ProviderStatusReport`, `ProviderMatrixReport`, `provider_client_factory` | 1,887 |
| Manifest catalog boundary | Manifest/profile structs, registry loading, manifest inspection, missing-entrypoint checks, and conversion into installed tentacles. This is the debug entry when manifests install incorrectly or editable harness metadata is missing. | `manifest_catalog.rs`, `TentacleManifest`, `TentacleProfile`, `InstalledTentacle`, `TentacleManifestReport` | 373 |
| Manifest tentacle runtime | Tool-side LLM planning, structured field mini-task selection, manifest tool authorization, command/http execution, `octopus-json-v1` parsing, tool-output metadata flattening, and compact Feed assembly. This is the debug entry when a Need reaches a tentacle but no correct Action/Feed returns. | `manifest_runtime.rs`, `ManifestTentacle`, `TentacleThinkingPlan` | 1,150 |
| Need queue boundary | Owns queued Need creation, take/drop, queue status, and queue report separation between human Goal hints and agent commands. | `need_queue.rs`, `NeedQueueItem`, `NeedQueueReport` | 234 |
| Need command surface | Owns `octopus needs`: queue display, run/take/drop output, executable queue script generation, and review-session artifacts. This is the debug entry when queue JSON is right but CLI/app/script/session output hides Need, Feed, field, or trace evidence. | `need_surface.rs`, `handle_needs_command`, `print_need_queue*`, `write_need_queue_*` | 663 |
| Need runner boundary | Takes queued Needs into the real Feed path, writes the live Need event before Feed and Feed state after execution, attaches field mini-task context, records automatic verifier hints, and emits parallel worker action events. This is the debug entry when the pet shows a Need but no Feed follows. | `need_runner.rs`, `run_queued_need_with_observer_state`, `NeedRunReport`, `record_auto_field_verifier_from_feed` | 544 |
| Feedback command surface | Owns `octopus feedback`: trace selector/status parsing, feedback record mutation, route score outcome rendering, and pet-event append after feedback. This is the debug entry when Feed exists but Feedback, route score, or pet state after feedback looks wrong. | `feedback_surface.rs`, `handle_feedback_command`, `print_feed_feedback_outcome` | 59 |
| Route and trace surface | Owns `octopus routes` and `octopus traces`, plus route reports, Feed traces, and Need queue rows. This is the debug entry when selected tentacle, field context, or latest trace is hidden in CLI/app output. | `route_surface.rs`, `handle_routes_command`, `handle_traces_command`, `print_route_report`, `print_feed_traces`, `trace_line`, `need_queue_line` | 205 |
| State report boundary | Builds read-only status, context, field trajectory, field pool, and harness-learning reports from `HarnessState`. It consumes the field curriculum decision instead of inventing the next harder field inline. This is the debug entry when app output, field pool state, or user-vs-agent next actions look wrong. | `state_report.rs`, `StatusReport`, `ContextReport`, `FieldPoolStatusReport` | 753 |
| Status and context surface | CLI/report rendering for status, context, tentacle summaries, Goal summary, recent Need/Feed turns, and harness-learning lines. This is the debug entry when the state JSON is right but app/CLI text confuses Goal, tentacles, or learning state. | `status_surface.rs`, `print_status_report`, `print_context_report`, `status_tentacles`, `harness_learning_line` | 257 |
| User surface boundary | Converts internal state into human Goal hints; keeps agent commands in observer fields so app/CLI users edit Goal only | `user_surface.rs`, `StatusReport`, `ContextReport`, `NeedQueueReport`, `FieldPoolStatusReport`, `ProductReport` | 123 |
| Product report surface | Builds `octopus report` and app-facing capability/gap summaries. It keeps `next` as human Goal hints and `agent_next` as internal executable actions, so the product surface does not become a hidden control panel. | `product_surface.rs`, `ProductReport`, `ProductCapability`, `ProductGap`, `product_report` | 791 |
| Preflight surface boundary | Builds `octopus preflight`, preflight scripts, real-machine record templates/audits/appends, release summary, and preflight printing. This is the debug entry when product readiness, real-machine evidence, or desktop/app release checks look wrong. | `preflight_surface.rs`, `PreflightReport`, `PreflightSummary`, `PreflightRecordCheckReport`, `preflight_report` | 1,101 |
| Doctor surface boundary | Builds `octopus doctor`: state existence, detected environment, bundled/installed manifest health, profile registry, provider setup, pet page, warnings, and next Goal hints. This is the debug entry when a local install looks unhealthy before strategy-level checks. | `doctor_surface.rs`, `DoctorReport`, `DoctorLlmReport`, `doctor_report`, `print_doctor_report` | 302 |
| Harness repair surface | Owns `octopus repair`: repair plan extraction, repair continuation, patch apply/verify, repair scoring, repair outcome journal, and repair report rendering. This is the debug entry when a harness change, repair plan, score, or post-apply check cannot be diagnosed from Need/Feed traces alone. | `repair_surface.rs`, `RepairReport`, `RepairPlanReport`, `RepairScoreReport`, `handle_repair_command` | 6,670 |
| Heartbeat command surface | Owns `octopus beat`: memory keep parsing, early heartbeat pet event, seed-source repair before beat, three-heart report rendering, and harness-beat evolution artifact recommendation. This is the debug entry when the desktop pet looks stale during beat, memory/harness beat output is wrong, or harness beat chooses the wrong repair target. | `heartbeat_surface.rs`, `handle_beat_command`, `print_heartbeat_report`, `write_harness_beat_evolution_artifacts_with_bundled_manifest` | 246 |
| Evolution contract | Manifest-owned surface requirements, required-surface validation, candidate ID normalization, LLM retry guardrails without field-specific Rust branches | `evolution.rs`, `tentacles/*/manifest.json`, `tentacles/tentacle.schema.json` | 471 |
| Evolution apply boundary | Authorized provider patch application, `git apply --check`, exact-content hunk relocation when provider line numbers drift, field-pack post-apply schema validation with automatic reverse on failure, reverse already-applied detection, summary helpers, safety tests, and live apply reports shared by CLI apply and autonomous drive | `evolution_apply.rs` | 696 |
| Evolution artifact boundary | Writes proposal/apply markdown, json, and authorized patch artifacts under `.octopus/evolution/**`; keeps review artifacts separate from planner logic and apply authorization. | `evolution_artifact.rs` | 401 |
| Evolution candidate boundary | Parses provider JSON, validates LLM candidate fields, targets, field-pack schema shape, and harder-layer field-pack/template patch pairs, recovers target files from provider diffs, builds patch-draft metadata, and keeps planner-output errors separate from stable kernel state. Wrong `task_layers`, object-shaped `mini_tasks`, or orphan harder-layer templates are rejected here so the LLM planner can retry. | `evolution_candidate.rs` | 411 |
| Evolution data contract | Public data shapes for evolution policy, surfaces, requirements, proposals, file targets, patch candidates, feedback, apply plans, artifacts, recommendations, and harness-beat reports. It owns serde defaults for manifest evolution surfaces and keeps contract shape separate from planner/apply execution. | `evolution_contract.rs` | 222 |
| Observation contract | Shared pet-visible event contract for harness evolution: stage names, provider/apply/schema error classes, and structured event writes. This is the first code boundary when the desktop pet turns red but the failing phase is unclear. | `observation_contract.rs`, `PetEvent.stage`, `PetEvent.error_class` | 155 |
| Evolution cycle compatibility | Thin compatibility layer for autonomous `evolve drive` stage/error writes; it delegates the actual observation contract to `observation_contract.rs`. | `evolution_cycle.rs` | 99 |
| Evolution LLM planner boundary | Builds the harness-planner chat request, preserves the clean-brain/tentacle context policy prompt, retries once with validator feedback, and parses provider JSON into candidate plans. | `evolution_llm_plan.rs` | 52 |
| Evolution planning boundary | LLM patch recommendation artifact writing and apply-failure retry objective shaping | `evolution_plan.rs` | 83 |
| Evolution prompt boundary | Builds the LLM planner payload from proposal, trace, check-history, file-target, surface, prompt-budget data, and supporting schema contracts. It owns context compaction, objective path hints, line-numbered current target content, and field-pack schema context for harness evolution while `lib.rs` keeps the public proposal/candidate contract. | `evolution_prompt.rs` | 345 |
| Evolution recommendation boundary | Assembles proposal context from manifests and state, scores candidates from outcomes/Feed traces/check history, builds apply plans, and writes harness-beat proposal/apply artifacts. This is where a wrong candidate choice or missing harness-beat artifact should be debugged. | `evolution_recommend.rs` | 679 |
| Evolution command surface | Owns `octopus evolve`: parallel worker start, drive dispatch, recommend/apply/score command flow, proposal/apply artifact loading, structured pet-event writes around evolution actions, and user/JSON output selection. This is the debug entry when self-evolution commands do not light the desktop pet or return misleading command output. | `evolution_surface.rs`, `handle_evolve_command`, `propose_evolution_for_cli`, `evolution_score_report` | 849 |
| Evolution drive surface | CLI args, JSON report shape, and human-readable printing for `evolve drive` | `evolution_drive_surface.rs` | 116 |
| Evolution driver | Autonomous harness loop for one cycle: stage events, planning, apply, manifest checks, and Feed delegation. It no longer owns CLI/report shape, patch execution, Feed execution, or retry objective shaping. | `evolution_driver.rs` | 318 |
| Evolution Feed boundary | Field mini-task Feed execution, queued Need worker state, desktop pet launch, Feed event writes, partial/failed batch status folding, blocked reporting when opened workers fail to reach Feed, and non-blocked success reporting for harness-only repairs with zero workers | `evolution_feed.rs` | 259 |
| Evolution patch boundary | Normalizes provider patch text, strips apply-patch/markdown wrappers, splits embedded diff headers, strips provider-prefixed patch control lines, inserts missing existing-file and new-file headers, removes diff-boundary blank separators, recounts hunk headers from actual patch body, extracts diff paths, renders display paths, and filters authorized patch paths before artifact/apply code can write a patch file. | `evolution_patch.rs` | 648 |
| Evolution target boundary | Resolves manifest editable targets, field-pack targets, repair-template wildcard scopes, candidate target files, and field-name scopes for patch/prompt/candidate modules. | `evolution_target.rs` | 318 |
| Field curriculum boundary | Selects the next concrete field for a harder mini-task layer from real trajectory summaries. It prefers unfinished/least-layer fields and returns a field-specific objective and agent action. This is the debug entry when the field pool says every field is complete but app/desktop cannot name the next field. | `field_curriculum.rs`, `state_report.rs` | 123 |
| Field adaptation core | Field-pack loading, matching, editable aliases, multilingual alias signals, Need annotation, structured peer-field queue context, trace metadata, peer-field worker slots, verifier results, live field mini task loader, editable field-pack task surfaces with concrete pack and registry target files, repair templates, mini task schema guard, LLM template result normalization, verifier-status normalization, pyfrag prevalidation, compile/execute template checks, verifier artifact propagation, FEED.md status sync, evolved math-mini-6 stationary-point evidence, evolved write-mini-6 claim-map evidence, evolved translate-mini-6 term-preservation evidence, evolved search-mini-5 source-audit evidence, evolved code-mini-5 deterministic patch-verification evidence, evolved swe-mini-5 multi-file regression evidence, evolved research-mini-5 claim-to-source evidence, evolved computer-use-mini-5 adapter-readiness evidence, evolved ib-mini-5 formula/compliance evidence, and evolved robotics-mini-5 dynamic-obstacle replanning evidence | `field_pack.rs`, `field-packs/**`, `tentacles/field-mini-task/**`, `docs/field-adaptation.md` | 8,308 |
| Field adaptation surface | Owns `octopus fields`: field-pack reports, field matching, field trajectory summaries, verifier score recording, pet-event append after verifier scoring, parallel field worker slots, field-pool latest activity, and user-visible field status lines. This is the debug entry when field state exists but the app/CLI explains the wrong active domain, score, worker slot, or verifier pet state. | `field_surface.rs`, `handle_fields_command`, `FieldMatchReport`, `print_field_trajectory_report`, `field_pool_status_line`, `parallel_run_status_line` | 558 |
| CLI and product backend | Command dispatch, structured direct Need context parsing, chat, starter/install/check flows, and surface entry dispatch. Goal/Brain command logic lives in `brain_goal_surface.rs`; Need command logic lives in `need_surface.rs`; Evolution command logic lives in `evolution_surface.rs`; Route/Trace command logic lives in `route_surface.rs`; Feedback command logic lives in `feedback_surface.rs`; Field command logic lives in `field_surface.rs`; beat command logic lives in `heartbeat_surface.rs`; repair command logic lives in `repair_surface.rs`. | `crates/octopus-core/src/main.rs` | 20,203 |
| Provider env | Loads `.octopus/llm.env` for CLI commands with a scoped guard; existing shell variables win, and values are restored after command execution | `provider_env.rs`, `app_bridge.rs::parse_env_overlay` | 107 |
| Local app bridge | Local HTTP/SSE server, `/api/config` state-path injection, app policy, command allow-list, embedded app/docs/showcase assets, read-only pet supervision access, field activity observer with fresh pet-event handling | `app_bridge.rs`, `docs/app.html` | 2,027 |
| Release and install gates | Low-level release records, benchmark evidence, download/install manifest, and real-machine record parsing used by the preflight surface | `release_gate.rs`, `preflight_surface.rs`, `download.rs`, `docs/real-machine-test.md`, `docs/download.json`, `docs/install.sh` | 2,312 |
| Pet observation surface | Builds the user-visible pet report, auto state choice, event-log view, pixel image report, desktop launch report, and supervision printout. It keeps fresh event text, pending Need text, and latest Feed trace text from being merged into a fake Goal/Need/Feed chain. This is the first code boundary after `octopus pet supervise --json` finds an observation mismatch. | `pet_surface.rs`, `pet_report_for_state`, `pet_event_log_report`, `print_pet_supervision_report` | 385 |
| Pet and visual state | Pixel Octopus state, SVG/export helpers, unified event writes, JSONL event audit, latest-event audit coverage, native read-only observer, desktop process/state-path check, runtime-state detection, active-work detection, stage/error diagnostics, desktop source preflight, HTML preview. Native pet no longer treats stale Feed traces, stale historical workers, or fresh blocked events as live active work; colors come from fresh pet events, pending Needs, fresh active worker slots, verifier status, goal status, heartbeat, or memory. | `pet.rs`, `pet_surface.rs`, `pet_events.rs`, `pet_supervision.rs`, `desktop_pet.rs`, `desktop/pet/OctopusDesktopPet.swift`, `docs/pet.html`, `tentacles/visual/manifest.json` | 3,467 |
| Strategy diagnostics | Checks the three core traits and composes pet supervision: clean brain context, LLM tool-side tentacles, editable goal, field surface, observation-chain freshness, active work, native desktop process/state-path evidence, stage/error evidence, JSONL latest-event coverage, Feed/evolution evidence | `diagnostics.rs`, `strategy_surface.rs`, `pet_supervision.rs`, `octopus diagnose strategy --json`, `octopus pet supervise --json` | 1,086 |
| Tentacle scaffold boundary | Generates user-owned code-as-harness tentacle manifests and optional python/node/shell seed tools. This is the debug entry when `octopus scaffold` creates an invalid manifest, missing executable, or wrong `octopus-json-v1` contract. | `tentacle_scaffold.rs`, `octopus scaffold` | 291 |
| Product docs/site | README, landing/showcase/tutorial/use/recipes/about/docs pages | `README*`, `docs/*.html`, `docs/*.md`, `docs/zh/*` | 8,085 |
| Editable tentacles | Code-as-harness Feed suppliers: prompts, manifests, tools, declared evolution requirements, field-pack task targets, repair templates, repair surfaces | `tentacles/**` | 20,240 |

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
- Brain/Goal surface rule: `brain_goal_surface.rs` owns Goal editing, Brain command mode validation, Brain session/report dispatch, and clean-brain Need audit rendering. It must not execute tools, mutate Feed traces directly, or become a user control panel beyond Goal changes.
- LLM provider rule: `llm_provider.rs` owns shared model request, retry, timeout, response-content extraction, and Plan JSON recovery. Brain/tentacle modules may call this boundary but should not hand-roll provider requests.
- LLM layer routing rule: `llm_layers.rs` owns chat/brain/manifest/evolve prefix selection, enable flags, and client factories. Provider setup lives in `provider_surface.rs`; request execution lives in `llm_provider.rs`.
- Provider surface rule: `provider_surface.rs` owns provider profiles, env-file generation, secret-safe key save, provider status/check/matrix reports, and client factory selection. CLI/app surfaces may present this data but should not duplicate provider setup logic.
- Manifest catalog rule: `manifest_catalog.rs` owns manifest/profile structs, registry loading, manifest inspection, missing-entrypoint checks, and installed-tentacle conversion. Runtime/state modules consume installed data and should not parse manifest files directly.
- Manifest runtime rule: `manifest_runtime.rs` owns tool-side LLM plan selection, permission grants, command/http execution, multi-action summaries, and `octopus-json-v1` Feed parsing. Tool JSON may return evolving metadata shapes; this boundary flattens them into stable Feed metadata so verifier/pet/state code can stay small. Brain/state/report modules must not run tools directly.
- Need queue rule: `need_queue.rs` owns queued Need state changes; queue reports must keep `next` user-facing and put executable commands in `agent_next`.
- Direct Need rule: `octopus need <kind> [--context key=value] <query>` may carry structured debug/automation context. Context flags must never be appended to natural-language query text; product users still change Goal, while agent internals may pass structured Need context.
- Need surface rule: `need_surface.rs` owns CLI/report/script/session surfaces for queued Needs. Queue mutation stays in `need_queue.rs`; Feed execution stays in `need_runner.rs`; tool execution stays in `manifest_runtime.rs`. It must not invent Feed or hide field/trace evidence.
- Need runner rule: `need_runner.rs` owns the handoff from queued Need to Feed. It must save the Need pet event before `feed_one`, save the Feed state after `feed_one`, and keep field mini-task/parallel worker evidence attached to the Feed path.
- Feedback surface rule: `feedback_surface.rs` owns CLI Feedback mutation and printing. It may record feedback on an existing Feed trace and append the resulting pet event, but it must not execute tools, synthesize Feed, or change route scoring outside `HarnessState::record_feed_feedback`.
- Route surface rule: `route_surface.rs` owns route/trace command handling, route reports, Feed traces, and Need queue line rendering. It may explain selected tentacles, field context, and latest traces but must not mutate route scores, queues, or Feed records.
- State report rule: `state_report.rs` is read-only; it may derive status/context/field-pool views from `HarnessState` but must not mutate Goal, Need, Feed, routes, grants, or pet events.
- Status surface rule: `status_surface.rs` owns status/context/Goal/tentacle/harness-learning rendering. It consumes `StatusReport` and `ContextReport`; data derivation stays in `state_report.rs`.
- Product bridge rule: user-facing writes go through Goal; internal actions feed the agent.
- User surface rule: `next` on status/context/Need queue/field pool/product report means human Goal hint; internal execution commands stay in `agent_next` or `agent_next_action`.
- Product surface rule: `product_surface.rs` owns capability/gap report construction. Any user-visible `next` must remain a Goal hint; executable commands belong in `agent_next`.
- Preflight surface rule: `preflight_surface.rs` owns product readiness aggregation, release summary, preflight scripts, real-machine record templates/audits/appends, and preflight printing. `release_gate.rs` stays the lower-level record/check parser.
- Doctor surface rule: `doctor_surface.rs` owns install health aggregation, provider setup summary, manifest health, profile registry health, pet page health, warnings, and doctor printing. Strategy-specific policy checks stay in `diagnostics.rs`.
- Harness repair surface rule: `repair_surface.rs` owns repair command handling, repair plan extraction from Feed metadata, patch apply/verify reports, repair scoring, and repair outcome journals. It must not bypass manifest runtime for new Feed, must not invent successful harness changes, and must keep write access behind grants/checks.
- Heartbeat surface rule: `heartbeat_surface.rs` owns the command surface for the three hearts. It may call the stable state `beat()` and the LLM harness-beat recommender, but it must not create fake Feed, bypass provider routing, or hide stale-pet evidence.
- Strategy surface rule: `strategy_surface.rs` owns `diagnose strategy` report assembly and printing. `diagnostics.rs` owns policy checks; command dispatch must not duplicate strategy or pet-observer logic.
- App state rule: browser app gets the real bridge state path from `/api/config`; file preview may fall back to `.octopus/state.json`.
- Pet supervision rule: `last_pet_event` is the live state, `.octopus/pet-events.jsonl` is the append-only audit trail, `event_log_contains_last` must pass, `runtime_state` reports fresh blocked/failed events as runtime warnings, `active_work` may only count pending Needs, fresh partial workers, fresh running events, or fresh successful terminal events, and `octopus pet supervise --json` is the first diagnostic for desktop observer issues.
- Heartbeat observation rule: `octopus beat` writes a fresh heartbeat pet event before long harness-beat work starts, then writes the final memory/harness event after the beat finishes. A long LLM/provider call must not leave the desktop observer looking dead.
- Pet observation surface rule: `pet_surface.rs` owns pet auto-state selection, event-log report reading, pixel image report printing, desktop launch report printing, and supervision printing. It must not merge a stale queue item, stale Feed trace, and fresh event into one fake chain; fresh event text wins, pending Need text is only used when pending, and Feed text must come from the same latest Feed trace. Command dispatch and harness execution should not duplicate pet state logic.
- Evolution observation rule: every `octopus evolve` path that writes a pet event must attach `PetEvent.stage` and blocked events must attach `PetEvent.error_class` through `observation_contract.rs`; a stuck pet should identify planning/provider, applying/patch, checking, or feeding instead of only showing `blocked`.
- Desktop observer rule: stale Feed traces cannot drive the native pet's current color or Need bubble. A Feed/action color must come from a fresh `last_pet_event` or an active worker update.
- Provider env rule: CLI reads `.octopus/llm.env` through `provider_env.rs`; explicit shell variables are never overwritten.
- Provider timeout rule: Codex CLI prompts are passed through a temp prompt file inside `llm_provider.rs`, not blocking `stdin.write_all`; `OCTOPUS_LLM_TIMEOUT` and `OCTOPUS_LLM_RETRIES` bound harness evolution calls.
- Evolution apply rule: provider patches are checked and applied only through `evolution_apply.rs`; CLI apply and autonomous drive share the same authorized patch execution path. Field-pack patches must parse after apply and are automatically reversed when schema validation fails.
- Evolution artifact rule: local proposal/apply review files are rendered and written only through `evolution_artifact.rs`; artifact rendering must not perform planning or mutate harness code directly.
- Evolution contract rule: policy/proposal/candidate/apply/recommendation/artifact structs live in `evolution_contract.rs`; this module has no provider calls, patch application, file mutation, or route scoring.
- Evolution candidate rule: provider JSON parsing, LLM candidate validation, harder-layer field-pack/template pair validation, manifest-relative target parsing, provider-diff target recovery, and patch draft metadata live in `evolution_candidate.rs`; `lib.rs` only asks for a parsed plan.
- Evolution LLM planner rule: planner system prompt, retry note, provider chat call, and provider JSON parsing call live in `evolution_llm_plan.rs`; the core only asks for a plan.
- Evolution planning rule: recommendation artifacts and apply-failure retry objectives live in `evolution_plan.rs`; driver only asks for the next plan.
- Evolution prompt rule: LLM planner payload construction, trace/history compaction, target-file context selection, and prompt-budget env knobs live in `evolution_prompt.rs`; proposal/candidate protocol stays in `lib.rs`.
- Evolution recommendation rule: proposal assembly, scored candidate choice, apply-plan construction, and harness-beat proposal/apply artifact orchestration live in `evolution_recommend.rs`; it consumes real state traces and must not invent success.
- Evolution patch rule: provider patch normalization, diff-boundary cleanup, hunk-header recounting, diff path extraction, display-path collapsing, and authorized patch allow-list checks live in `evolution_patch.rs`; artifact rendering can only ask for an already authorized patch.
- Evolution target rule: manifest editable targets, field-pack paths, repair-template wildcard scopes, and field-name path extraction live in `evolution_target.rs`; planner/prompt/candidate/patch modules ask this boundary for files.
- Evolution command surface rule: `evolution_surface.rs` owns `octopus evolve` command handling, proposal/apply artifact loading, evolution score reports reused by repair scoring, and pet-event writes around recommend/apply/score/parallel actions. Driver logic stays in `evolution_driver.rs`; Feed execution stays in `evolution_feed.rs`; patch application stays in `evolution_apply.rs`.
- Evolution drive surface rule: `evolve drive` CLI/report formatting lives in `evolution_drive_surface.rs`; stage execution stays in `evolution_driver.rs`.
- Evolution driver rule: `evolve drive` uses `evolution_cycle.rs` for stage/error events. Apply-check failures are fed back to the LLM once for a regenerated current-file patch.
- Evolution Feed rule: `evolution_feed.rs` folds verifier/Feed statuses across the actual batch. A batch with any partial verifier result must show `harness/partial`, not a fake `feed/satisfied` event. A harness-only repair with zero workers must not become `feed_missing_queued_need`; opened workers with no Feed still block.
- Field surface rule: `field_surface.rs` owns field-adaptation command handling, display, matching reports, field-pool lines, verifier score recording, verifier result printing, and parallel field run rendering. It may append the verifier pet event created by `record_field_verifier_result`; it must not mutate harness code or bypass field-pack/manifest runtime execution.
- Patch apply rule: provider-created new-file patches are normalized before `git apply`; missing `new file mode 100644` is inserted when a diff uses `--- /dev/null`.
- Release gates: preflight, benchmark evidence, field-pool visibility, real-machine records, local app readiness.
- Tentacle scaffold rule: user-owned code-as-harness seed manifests and python/node/shell starter tools live in `tentacle_scaffold.rs`; new scaffold output must declare LLM brain, tool metadata, runtime code, and editable evolution surfaces.
- Strategy diagnostics: `octopus diagnose strategy --json` checks clean-brain context, user-surface command leakage, tool-side LLM tentacles, editable Goal, field-pack evolution surface, fresh pet events, desktop process/state-path evidence, stage/error evidence, and Feed/evolution evidence.

### Desktop Pet / Evolution Failure Map

Use this before changing UI or harness code:

| Symptom | First check | Meaning | Fix location |
| --- | --- | --- | --- |
| Pet not visible | `octopus pet supervise --json`, check `desktop_process` | Native observer is not running or is reading another state path | `desktop_pet.rs`, `desktop/pet/OctopusDesktopPet.swift` |
| Pet visible but stale | `event_freshness` plus `active_work` | If `active_work=pass`, a real loop exists but is not writing fresh events. If `active_work=warn`, no fresh Need/evolve loop is running. Historical partial workers do not count. | caller that should use `pet_events` or `evolution_cycle.rs`; heartbeat entry in `heartbeat_surface.rs`; active-loop start path in `evolution_surface.rs`/`need_runner.rs`; report mismatch in `pet_surface.rs` |
| Evolution command runs but pet does not change | compare `octopus evolve ... --json`, `.octopus/state.json.last_pet_event`, and `.octopus/pet-events.jsonl` | The command surface skipped the unified pet-event write before/after an evolution action | `evolution_surface.rs`; stage-aware `drive` events stay in `evolution_driver.rs` and `evolution_cycle.rs` |
| Doctor reports unhealthy install | `octopus doctor --json`, then warnings and `llm/profile_registry/pet` fields | Local install, provider config, profile registry, manifest health, or pet page is broken before strategy-level execution | `doctor_surface.rs`, then the boundary named by the warning |
| Preflight says desktop/app is not ready | `octopus preflight --json`, then the failing check id | Product readiness aggregation failed; inspect the check evidence before editing app or harness code | `preflight_surface.rs`, then the boundary named by the check |
| Pet red/blocked during planning | `last_event.stage=planning`, `error_class=provider_timeout/provider_error` | LLM planner did not return a harness candidate | provider config/client and evolution planner boundary |
| Pet red/blocked during applying | `stage=applying`, `error_class=patch_*` | LLM produced an invalid, missing, or unauthorized patch | apply/patch normalization boundary |
| Pet red/blocked with `field_pack_schema_failed` | `stage=applying`, `error_class=field_pack_schema_failed`, then inspect the patch artifact | A field-pack patch applied cleanly as text but broke the editable JSON contract; the apply boundary should reverse it before checks run | `evolution_apply.rs`, affected `field-packs/<field>/field-pack.json`, affected repair template |
| Pet red/blocked during checking | `stage=checking`, `error_class=check_failed` | Harness patch applied but its declared check failed | tentacle check policy or editable harness code |
| Pet red/blocked during feeding | `stage=feeding`, `error_class=feed_missing_queued_need` | Worker slots were opened but no queued Need actually reached Feed; the system must not report fake success | `evolution_feed.rs` queued-Need boundary |
| Pet shows `harness/partial` after a Feed | `last_event.state=harness`, `status=partial`, then compare latest verifier result | A real Feed ran, but verifier still found partial evidence. The pet is observing the unresolved harness result, not controlling it. | `evolution_feed.rs` status fold; then `need_runner.rs` verifier and affected repair template |
| Pet shows Need but no Feed follows | inspect `NeedRunReport`, latest pet event, and `.octopus/state.json` around the queue item | Queued Need was taken but did not reach `feed_one`, or observer state was not saved after Feed | `need_runner.rs`, affected manifest runtime |
| User can change too many things, or Brain/Goal text is confusing | `octopus goal --json`, `octopus brain --session --json`, then compare printed output | Goal/Brain surface leaked internal modes into the user path, or audit text hid polluted Needs | `brain_goal_surface.rs`; Need generation stays in `brain_loop.rs` |
| Brain suggests tool steps instead of Needs | inspect `brain/context` report and Need audit | Clean-brain loop leaked implementation details into Need | `brain_loop.rs` |
| LLM call hangs or returns empty/invalid content | inspect provider env and provider response error | Request timeout, retry, response extraction, or JSON recovery failed | `llm_provider.rs`, `provider_env.rs` |
| Provider appears configured but brain/tentacle/evolution layer does not use it | `octopus provider status --json`, `octopus provider check <PREFIX>`, provider matrix record | Profile/env/status or layer prefix selection is wrong | `provider_surface.rs`, `llm_layers.rs`, `.octopus/llm.env`, `.octopus/providers/*.env` |
| Tentacle thinks but no action or wrong Feed returns | `octopus think ... --json`, Feed metadata `plan_source`, `tools`, `contract`, `authorization_required` | Need reached a local tool brain, but planning, authorization, execution, or Feed parsing failed | `manifest_runtime.rs`, affected `tentacles/*/manifest.json`, affected tool code |
| Direct `octopus need` routes to the wrong field | Inspect returned `need.context` and ensure `--context key=value` did not land in `need.query` | Structured Need context was missing or parsed as text, so field matching guessed from prose | `main.rs::parse_need_command_args`, then `field_pack.rs` matcher |
| Manifest install or inspect reports wrong metadata | `octopus manifests <root> --json`, missing entrypoints, installed tentacle metadata | Manifest/profile loading, report derivation, or installed conversion is wrong | `manifest_catalog.rs`, affected `tentacles/*/manifest.json` |
| Queued Need cannot be run/taken/dropped | inspect `needs --json` and item status | Queue mutation or user/agent next split is wrong | `need_queue.rs` |
| Need queue text/script/session is wrong | compare `octopus needs --json`, `octopus needs script --json`, `octopus needs session --json`, and generated files | Queue data exists, but the command surface hid evidence or generated the wrong observer artifact | `need_surface.rs`; mutation stays in `need_queue.rs`, execution stays in `need_runner.rs` |
| Route or trace text hides the selected tentacle/field | compare `routes --json`, `traces --json`, and CLI/app text | Routing or Feed data is present, but the observation surface is misleading | `route_surface.rs`; score data stays in `lib.rs` route reports |
| Feedback records wrong status, route score, or pet state | compare `octopus feedback latest <status> --json`, latest trace, and `.octopus/state.json.last_pet_event` | Feed trace exists, but feedback mutation, route score update, or pet event append is wrong | `feedback_surface.rs`; feedback state mutation stays in `HarnessState::record_feed_feedback` |
| App status/context shows wrong next step | compare `next` with `agent_next` in `status/context/needs` JSON | Read-only user hints leaked internal commands, or field-pool state was derived incorrectly | `state_report.rs`, `user_surface.rs` |
| Status/context text misstates Goal, tentacles, or learning | compare `status --json`, `context --json`, and CLI/app text | Derived state is right, but rendering is misleading | `status_surface.rs`; derivation stays in `state_report.rs` |
| Report/app suggests commands as user actions | compare product `next` and `agent_next` | Product surface leaked internal agent controls into the user Goal path | `product_surface.rs`, `user_surface.rs` |
| Field summary, score, active field, or worker slot is wrong | `octopus fields summary --json`, `octopus fields score latest <status> --json`, then compare app/CLI text and latest pet event | Field data exists but the command surface, verifier score path, latest-activity line, or pet event append is misleading | `field_surface.rs`; data derivation stays in `state_report.rs` and `field_pack.rs` |
| Harness beat suggests the wrong patch | inspect `.octopus/evolution/**/proposal.json` and `.octopus/evolution/**/apply/plan.json` | Candidate scoring or apply-plan selection used the wrong trace/check/outcome evidence, or the beat surface selected the wrong latest failing tentacle | `heartbeat_surface.rs`, then `evolution_recommend.rs` |
| Repair plan, apply, verify, or score output is inconsistent | `octopus repair . --json`, `octopus repair apply . --json`, `octopus repair verify . --json`, then inspect the returned plan/status fields | The repair command surface is misreading Feed metadata, skipping authorization/checks, or recording the wrong outcome journal | `repair_surface.rs`; Feed creation stays in `manifest_runtime.rs` and write checks stay in `evolution_apply.rs`/declared harness policy |
| `octopus scaffold` creates a bad tentacle | `octopus manifests <root>` and install/feed tests | Scaffold manifest, seed tool contract, or executable bit is wrong | `tentacle_scaffold.rs` |

Where they live:

- `crates/octopus-core/src/lib.rs`
- `crates/octopus-core/src/main.rs`
- `crates/octopus-core/src/app_bridge.rs`
- `crates/octopus-core/src/brain_goal_surface.rs`
- `crates/octopus-core/src/brain_loop.rs`
- `crates/octopus-core/src/doctor_surface.rs`
- `crates/octopus-core/src/llm_layers.rs`
- `crates/octopus-core/src/llm_provider.rs`
- `crates/octopus-core/src/manifest_catalog.rs`
- `crates/octopus-core/src/manifest_runtime.rs`
- `crates/octopus-core/src/provider_surface.rs`
- `crates/octopus-core/src/product_surface.rs`
- `crates/octopus-core/src/preflight_surface.rs`
- `crates/octopus-core/src/repair_surface.rs`
- `crates/octopus-core/src/release_gate.rs`
- `crates/octopus-core/src/route_surface.rs`
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
- `crates/octopus-core/src/feedback_surface.rs`
- `crates/octopus-core/src/field_surface.rs`
- `crates/octopus-core/src/heartbeat_surface.rs`
- `crates/octopus-core/src/need_queue.rs`
- `crates/octopus-core/src/need_surface.rs`
- `crates/octopus-core/src/need_runner.rs`
- `crates/octopus-core/src/observation_contract.rs`
- `crates/octopus-core/src/state_report.rs`
- `crates/octopus-core/src/status_surface.rs`
- `crates/octopus-core/src/strategy_surface.rs`
- `crates/octopus-core/src/tentacle_scaffold.rs`
- `crates/octopus-core/src/user_surface.rs`
- `crates/octopus-core/src/diagnostics.rs`
- `crates/octopus-core/src/pet_events.rs`
- `crates/octopus-core/src/pet_surface.rs`
- `crates/octopus-core/src/pet_supervision.rs`

### Known Refactor Debt

These remain intentionally visible:

- `evolution_feed.rs` is still field-mini-task specific. The next version should make Feed-stage runners manifest/routing owned so future field packs can choose their own Feed executor.
- `lib.rs` still owns broad state mutation. The next cleanup should split durable state mutation before touching product backend behavior.
- Automatic `cargo test` next step is still suspicious for LLM-native self-evolution and should be removed or pushed into editable manifest/harness policy.

### Distinctive Functions

These are the project identity:

- Clean brain: the main model asks for Need, not tool choreography.
- Thinking tentacles: tool-side LLM + tool meta + executable harness code.
- Need -> Feed -> Feedback loop with compact Feed traces.
- Three-heart surface: heartbeat, memory beat, harness beat. Harness beat can write evolution artifacts only through a configured LLM planner; local rule candidates are removed.
- Field evolution proof point: math-mini-5 was generated through `evolve recommend`, failed through apply/check/Feed, repaired in editable harness, then passed Need -> Tentacle -> Feed with trace evidence. Math then evolved to `math-mini-6`, exposed hunk-only existing-file patch output from the LLM planner, added generic patch header normalization, applied the same candidate, repaired the editable template result shape, and passed Need -> Feed with trace #122 and stationary-point evidence under `.octopus/field-mini-task/math/20260701T051444Z-math-mini-6/`. Write then evolved to `write-mini-6`, exposed provider-prefixed new-file control lines that dropped the template, added generic control-line normalization, restored the editable template, and passed Need -> Feed with trace #123 and claim-map evidence under `.octopus/field-mini-task/write/write-mini-6/20260701T052609Z/`. Translate then evolved to `translate-mini-6`, exposed a partial-result metadata failure, self-repaired its runtime template, cleaned stale unreachable code, synced FEED.md to satisfied, propagated verifier artifact metadata, and passed official Need -> Feed with trace #126 and evidence under `.octopus/field-mini-task/translate/translate-mini-6/20260701T054408Z/evidence.json`.
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
  math/      mini-1..6
  search/    mini-1..5
  code/       mini-1..5
  swe/       mini-1..5
  research/  mini-1..5
  computer-use/ mini-1..5
  ib/        mini-1..5
  robotics/ mini-1..5
  write/      mini-1..6
  translate/  mini-1..6
```

## Code Size

### Top-Level Areas

| Area | Files | Lines |
| --- | ---: | ---: |
| `crates/octopus-core/src` | 58 | 61,247 |
| `crates/octopus-core/examples` | 0 | 0 |
| `tentacles` | 95 | 21,361 |
| `field-packs` | 14 | 698 |
| `desktop/pet` | 1 | 841 |
| `docs` | 29 md/html/json/sh files | 7,937 |
| `cowork` | 3 | 101 |
| `local/docs` | 12 | 507 |

### Core Rust Files

| File | Lines |
| --- | ---: |
| `main.rs` | 20,203 |
| `lib.rs` | 9,047 |
| `repair_surface.rs` | 6,670 |
| `provider_surface.rs` | 1,887 |
| `brain_goal_surface.rs` | 1,610 |
| `app_bridge.rs` | 1,167 |
| `preflight_surface.rs` | 1,101 |
| `manifest_runtime.rs` | 1,150 |
| `brain_loop.rs` | 1,082 |
| `field_pack.rs` | 868 |
| `evolution_surface.rs` | 849 |
| `product_surface.rs` | 791 |
| `state_report.rs` | 753 |
| `llm_provider.rs` | 704 |
| `release_gate.rs` | 703 |
| `pet_supervision.rs` | 831 |
| `evolution_recommend.rs` | 679 |
| `need_surface.rs` | 663 |
| `field_surface.rs` | 558 |
| `need_runner.rs` | 563 |
| `bundled_harness.rs` | 503 |
| `evolution_artifact.rs` | 401 |
| `desktop_pet.rs` | 377 |
| `manifest_catalog.rs` | 373 |
| `pet_surface.rs` | 385 |
| `diagnostics.rs` | 354 |
| `evolution_prompt.rs` | 345 |
| `evolution_patch.rs` | 648 |
| `evolution_driver.rs` | 318 |
| `evolution_target.rs` | 318 |
| `doctor_surface.rs` | 302 |
| `llm_layers.rs` | 300 |
| `evolution_candidate.rs` | 411 |
| `tentacle_scaffold.rs` | 291 |
| `evolution_apply.rs` | 696 |
| `pet.rs` | 280 |
| `status_surface.rs` | 257 |
| `heartbeat_surface.rs` | 246 |
| `need_queue.rs` | 234 |
| `evolution_contract.rs` | 222 |
| `core_boundary.rs` | 219 |
| `route_surface.rs` | 205 |
| `download.rs` | 175 |
| `observation_contract.rs` | 155 |
| `evolution_feed.rs` | 259 |
| `field_curriculum.rs` | 123 |
| `user_surface.rs` | 123 |
| `pet_events.rs` | 118 |
| `evolution_drive_surface.rs` | 116 |
| `provider_env.rs` | 107 |
| `evolution_cycle.rs` | 99 |
| `evolution_plan.rs` | 83 |
| `evolution.rs` | 78 |
| `profile_registry.rs` | 78 |
| `feedback_surface.rs` | 59 |
| `evolution_llm_plan.rs` | 52 |
| `strategy_surface.rs` | 40 |
| `shell_words.rs` | 18 |

### Tentacles

| Tentacle | Files | Lines |
| --- | ---: | ---: |
| `harness-repair-agent` | 6 | 12,202 |
| `repo-maintainer` | 8 | 719 |
| `computer-use-agent` | 10 | 644 |
| `profile-registry` | 1 | 600 |
| `field-mini-task` | 57 | 6,503 |
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
- If LLM evolution apply fails on new files, inspect `.octopus/evolution/<tentacle>/apply/*.patch`; the normalizer in `evolution_patch.rs` must preserve `/dev/null`, insert `new file mode 100644`, and strip provider-prefixed control lines such as `+--- /dev/null`, `++++ b/...`, and `+@@ ...`.
- If LLM evolution apply fails with `*** End Patch`, markdown fences, stale hunk context, bad hunk counts, blank lines between diff file sections, or valid old-context with wrong hunk line numbers, inspect `evolution_patch.rs` wrapper stripping, diff-boundary cleanup, hunk recounting, `evolution_apply.rs` exact-content hunk relocation, and `evolution_prompt.rs` line-numbered target content. Retry prompts should include the current target path so the planner sees exact current lines.
- If field-pack evolution proposes `task_layers`, object-shaped `mini_tasks`, patch text with a nested `diff --git`, or a syntactically applied but invalid field-pack JSON, inspect `evolution_prompt.rs`, `evolution_candidate.rs`, `evolution_patch.rs`, and `evolution_apply.rs`. The planner should see `field-packs/field-pack.schema.json`, bad schema shapes should be rejected before apply, embedded diff headers and bad hunk counts should be normalized before `git apply`, and post-apply schema failures should be reversed with `field_pack_schema_failed`.
- If `octopus evolve` stays in planning, run with explicit bounds such as `OCTOPUS_LLM_TIMEOUT=60 OCTOPUS_LLM_RETRIES=0`. A timeout must become a fresh `blocked` pet event with `stage=planning` and `error_class=provider_timeout`; if it does not, inspect `observation_contract.rs`, `evolution_surface.rs`, `evolution_llm_plan.rs`, and `llm_provider.rs`.
- Construction verification: `OCTOPUS_LLM_TIMEOUT=1 OCTOPUS_LLM_RETRIES=0 octopus --json evolve recommend field-mini-task ...` now records `blocked`, `stage=planning`, `error_class=provider_timeout`; `octopus pet supervise --json` reports `last_stage=pass`, `error_category=pass`, `desktop_process=pass`, `runtime_state=warn`, and `active_work=warn` so a fresh visible failure is not counted as active work.
- Pet URL chain verification: `pet auto` must not combine a non-pending queued Need, stale Feed trace, and fresh blocked event. The current rule only shows pending Need text, or a Need/Feed pair from the same latest Feed trace, or text from the fresh event itself.
- Field evolution proof point: SWE evolved to `swe-mini-4` through real `evolve drive`, exposed syntax and patch-context failures, fed those failures back into `evolve score`, added patch-wrapper normalization, line-numbered retry context, pyfrag runtime prevalidation, and passed Need -> Feed with trace #97 and `before_rc=1 after_rc=0`. SWE then evolved to `swe-mini-5` through real `evolve drive`, passed manifest checks, reached trace #116, and recorded before=1/after=0 evidence under `.octopus/field-mini-task/swe/20260701T041613Z-swe-mini-5/`.
- Field evolution proof points: write evolved to `write-mini-4` through concrete curriculum selection and passed Need -> Feed with trace #99; write evolved again to `write-mini-5`, exposed invalid field-pack JSON, top-level pyfrag import, and timestamp warning failures, repaired the editable template, then passed Need -> Feed with trace #111 and evidence under `.octopus/field-mini-task/write/write-mini-5/20260701T032204Z/`; write then evolved to `write-mini-6`, exposed provider-prefixed new-file control lines, added generic normalization, restored the editable template, and passed Need -> Feed with trace #123 and claim-map evidence under `.octopus/field-mini-task/write/write-mini-6/20260701T052609Z/`; translate evolved to `translate-mini-5`, exposed patch-context failure, invalid trailing field-pack JSON, and pyfrag boundary failure, then passed Need -> Feed with trace #112 and evidence under `.octopus/field-mini-task/translate/20260701T034324Z-translate-mini-5/evidence.json`; translate then evolved to `translate-mini-6`, exposed partial-result metadata failure, self-repaired the runtime template, cleaned unreachable stale code, synced FEED.md to satisfied, propagated verifier artifact metadata, and passed Need -> Feed with trace #126 and evidence under `.octopus/field-mini-task/translate/translate-mini-6/20260701T054408Z/evidence.json`; search evolved to `search-mini-5`, exposed partial Feed from weak local-source checks, a bad runtime patch that left duplicated tail code, and a failed repair patch, then passed Need -> Feed with trace #114 and evidence under `.octopus/field-mini-task/search/20260701T035440Z-search-mini-5/evidence.json`; code evolved to `code-mini-5`, exposed a malformed hunk, invalid field-pack JSON, and a template variable-name error, then passed Need -> Feed with trace #115 and evidence under `.octopus/field-mini-task/code/20260701T040943Z-code-mini-5/`; research evolved to `research-mini-4`, exposed orphan-template, invalid JSON, stale patch-context, partial Feed, and missing metadata propagation failures, then passed structured Need -> Feed with trace #105; research then evolved to `research-mini-5`, exposed a metadata-shape check failure and an ineffective self-repair, received a narrow editable-template repair, then passed Need -> Feed with trace #117 and claim-to-source evidence under `.octopus/field-mini-task/research/20260701T042903Z-research-mini-5/`; computer-use evolved to `computer-use-mini-4`, exposed patch-header, JSON-placement, and bad-hunk-count failures, then passed `planning -> recommended -> applied -> checking -> need -> feed` with trace #106 and evidence under `.octopus/field-mini-task/computer-use/computer-use-mini-4/`; computer-use then evolved to `computer-use-mini-5`, exposed one provider timeout, a Feed failure from a template/helper collision, and an adapter-path mismatch, then passed Need -> Feed with trace #119 and adapter-readiness evidence under `.octopus/field-mini-task/computer-use/computer-use-mini-5/20260701T044430Z/`; ib evolved to `ib-mini-4`, exposed new-file header, missing context guard, restricted-language, and bad patch-context failures, then passed Need -> Feed with trace #109; ib then evolved to `ib-mini-5`, exposed a top-level import/standalone-template check failure and artifact-scope mismatch, received a narrow editable-template repair, then passed Need -> Feed with trace #120 and formula/compliance evidence under `.octopus/field-mini-task/ib/20260701T045216Z-ib-mini-5/`; robotics evolved to `robotics-mini-4`, exposed a field-pack patch-context failure, regenerated the patch, then passed Need -> Feed with trace #110; robotics then evolved to `robotics-mini-5`, passed configured harness checks on the LLM-generated patch, emitted `evolution -> action -> need -> feed` pet events, and recorded dynamic-obstacle replanning evidence under `.octopus/field-mini-task/robotics/20260701T045901Z-robotics-mini-5/`; math evolved to `math-mini-6`, exposed repeated hunk-only patch failures, then passed Need -> Feed with trace #122 after generic patch normalization and editable-template shape repair.
- Cleanup audit after the evolution-boundary commits moved prompt, candidate, artifact, patch, target, contract, recommendation, LLM planner, scaffold, clean-brain loop, Need queue, Need runner, Feedback surface, heartbeat surface, route surface, state-report, status surface, field surface, LLM provider, LLM layer routing, manifest catalog, manifest runtime, provider surface, preflight surface, doctor surface, and strategy surface responsibilities out of the largest files. Remaining discomfort: `main.rs` is still product-backend aggregation, and `lib.rs` still owns broad state mutation.
- Evolution surface requirements now belong to manifests. Rust validates declared missing surfaces and must not grow domain-specific trigger rules.
- Current field-mini-task template coverage is 53/53 satisfied after translate-mini-6. The apply path normalizes LLM patch file headers, strips provider-prefixed new-file control lines, strips patch wrappers before `git apply`, inserts missing existing-file headers, removes diff-boundary blank lines, recounts hunk headers, relocates exact-content hunks when provider line numbers drift, rejects field-pack schema-shape and orphan harder-layer mistakes before apply, runs an apply precheck first, reverses post-apply invalid field-pack JSON, retries with current-file context after patch failure, and field-mini-task normalizes common LLM template result shapes into `octopus-json-v1` Feed. Math, write, and translate are at mini-6; the next concrete curriculum field is `search`.
- The release showcase is screenshot-first. The local app surface in `docs/app.html` observes and updates the real local Octopus loop.
