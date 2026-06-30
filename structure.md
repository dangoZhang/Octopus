# Structure

Updated: 2026-07-01, after Feedback command surface, Heartbeat command surface, Need command surface, harness repair surface, and Brain/Goal command surface extraction plus modular evolution/pet diagnostics refactor, LLM evolution planner/contract/recommend/prompt/candidate/artifact/patch/target, tentacle scaffold, brain loop, Need queue, Need runner, route surface, state-report, status surface, field surface, product surface, preflight surface, doctor surface, strategy surface, LLM provider, LLM layer routing, manifest catalog, manifest tentacle runtime, pet observation surface, and provider surface extraction.

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
│       ├── evolution_patch.rs Provider patch normalization, diff path extraction, and apply allow-list filtering.
│       ├── evolution_plan.rs  LLM patch recommendation artifacts and apply-failure retry objective shaping.
│       ├── evolution_prompt.rs LLM evolution prompt, compact trace/history payloads, and target-file context budget.
│       ├── evolution_recommend.rs Proposal assembly, candidate scoring, apply-plan selection, and harness-beat artifact orchestration.
│       ├── evolution_target.rs Manifest/field-pack/repair-template target resolution for harness evolution.
│       ├── feedback_surface.rs Feed feedback command surface, route score outcome rendering, pet event append.
│       ├── field_pack.rs      Field-pack loader, matcher, and Need/trace metadata.
│       ├── field_surface.rs   Field adaptation CLI/report rendering and parallel field status lines.
│       ├── heartbeat_surface.rs
│       │                      Three-heart beat command surface, early pet heartbeat event, harness-beat recommendation output.
│       ├── llm_layers.rs      Chat/brain/manifest/evolve LLM layer prefix and client routing.
│       ├── llm_provider.rs    Shared ChatClient, OpenAI-compatible, Codex CLI, retry/timeout request layer.
│       ├── manifest_catalog.rs Manifest/profile data types, loading, inspection, and install conversion.
│       ├── manifest_runtime.rs Manifest-owned tool-side thinking, authorization, execution, and Feed parsing.
│       ├── need_queue.rs      Queued Need state mutation and queue report boundary.
│       ├── need_surface.rs    Need command/report surface: queue display, run output, script/session artifacts.
│       ├── need_runner.rs     Queued Need execution, Feed handoff, field mini-task context, worker events.
│       ├── pet.rs             Pixel Octopus state, SVG export, file URL helpers.
│       ├── pet_events.rs      Unified state event write path and JSONL audit log for desktop/pet observers.
│       ├── pet_surface.rs     Pet CLI/report surface: auto state, event log, image/desktop/supervision printing.
│       ├── pet_supervision.rs Desktop pet observation-chain diagnostics: state file, event log, last event, freshness.
│       ├── profile_registry.rs Seed profile registry loading and status.
│       ├── preflight_surface.rs Release preflight report, script/record templates, record audit, and preflight printing.
│       ├── provider_env.rs    Scoped `.octopus/llm.env` loader; fills missing provider variables without overriding shell env.
│       ├── provider_surface.rs Provider profiles, env save/check/status, matrix evidence, and provider client selection.
│       ├── repair_surface.rs  Harness repair command surface, repair plan reports, patch apply/verify, scoring, and repair learning journal.
│       ├── product_surface.rs Product report: capability/gap/next split for user Goal hints and agent actions.
│       ├── release_gate.rs    Preflight records, benchmark and real-machine gates.
│       ├── route_surface.rs   Route, Feed trace, and Need queue line rendering.
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
| Stable kernel | Goal/Need/Feed contracts, state mutation, route scores, memory, Feed traces, and public kernel API | `crates/octopus-core/src/lib.rs` | 8,999 |
| Clean brain loop boundary | Converts Goal/Mem/recent Feed into clean cognitive Needs and Goal refinement reports. This is the debug entry when Brain output leaks tool choreography or over-specific implementation steps. | `brain_loop.rs`, `BrainExploreReport`, `BrainGoalReport`, `BrainContextReport` | 1,082 |
| Brain/Goal command surface | Owns user Goal editing, Brain command validation, Brain session/report dispatch, clean-brain Need audit rendering, and Goal text output. This is the debug entry when the user can change more than Goal, Brain modes leak into the product surface, or Goal/Need audit text is confusing. | `brain_goal_surface.rs`, `handle_goal_command`, `handle_brain_command`, `print_brain_*`, `print_goal` | 1,610 |
| LLM provider boundary | Shared model request substrate for brain and tentacle intelligence: ChatClient contract, OpenAI-compatible config/body parsing, Codex CLI prompt-file execution, retries, timeout, and Plan JSON recovery. This is the debug entry when model calls, local model routing, or provider response parsing fail. | `llm_provider.rs`, `ChatClient`, `OpenAiCompatibleChatClient`, `CodexCliChatClient`, `ChatPlanner` | 704 |
| LLM layer routing | Maps configured provider prefixes into chat, clean-brain, tentacle-planning, and harness-evolution clients. This is the debug entry when the provider works but one Octopus layer uses the wrong prefix or disabled layer. | `llm_layers.rs`, `BrainLlmPrefixOverride`, `manifest_llm_factory`, `clean_brain_*_llm_client` | 300 |
| Provider surface boundary | User-facing provider profiles, `.octopus/llm.env` generation, secret-safe key save, provider check/status, provider matrix evidence, and client factory selection for brain, tentacle, and harness evolution layers. | `provider_surface.rs`, `ProviderProfile`, `ProviderStatusReport`, `ProviderMatrixReport`, `provider_client_factory` | 1,887 |
| Manifest catalog boundary | Manifest/profile structs, registry loading, manifest inspection, missing-entrypoint checks, and conversion into installed tentacles. This is the debug entry when manifests install incorrectly or editable harness metadata is missing. | `manifest_catalog.rs`, `TentacleManifest`, `TentacleProfile`, `InstalledTentacle`, `TentacleManifestReport` | 373 |
| Manifest tentacle runtime | Tool-side LLM planning, structured field mini-task selection, manifest tool authorization, command/http execution, `octopus-json-v1` parsing, and compact Feed assembly. This is the debug entry when a Need reaches a tentacle but no correct Action/Feed returns. | `manifest_runtime.rs`, `ManifestTentacle`, `TentacleThinkingPlan` | 1,090 |
| Need queue boundary | Owns queued Need creation, take/drop, queue status, and queue report separation between human Goal hints and agent commands. | `need_queue.rs`, `NeedQueueItem`, `NeedQueueReport` | 234 |
| Need command surface | Owns `octopus needs`: queue display, run/take/drop output, executable queue script generation, and review-session artifacts. This is the debug entry when queue JSON is right but CLI/app/script/session output hides Need, Feed, field, or trace evidence. | `need_surface.rs`, `handle_needs_command`, `print_need_queue*`, `write_need_queue_*` | 663 |
| Need runner boundary | Takes queued Needs into the real Feed path, writes the live Need event before Feed and Feed state after execution, attaches field mini-task context, records automatic verifier hints, and emits parallel worker action events. This is the debug entry when the pet shows a Need but no Feed follows. | `need_runner.rs`, `run_queued_need_with_observer_state`, `NeedRunReport`, `record_auto_field_verifier_from_feed` | 544 |
| Feedback command surface | Owns `octopus feedback`: trace selector/status parsing, feedback record mutation, route score outcome rendering, and pet-event append after feedback. This is the debug entry when Feed exists but Feedback, route score, or pet state after feedback looks wrong. | `feedback_surface.rs`, `handle_feedback_command`, `print_feed_feedback_outcome` | 59 |
| Route and trace surface | CLI/report rendering for route reports, Feed traces, and Need queue rows. This is the debug entry when a Need routes correctly in JSON but the CLI/app explanation hides the selected tentacle, field context, or latest trace. | `route_surface.rs`, `print_route_report`, `print_feed_traces`, `trace_line`, `need_queue_line` | 143 |
| State report boundary | Builds read-only status, context, field trajectory, field pool, and harness-learning reports from `HarnessState`. This is the debug entry when app output, field pool state, or user-vs-agent next actions look wrong. | `state_report.rs`, `StatusReport`, `ContextReport`, `FieldPoolStatusReport` | 734 |
| Status and context surface | CLI/report rendering for status, context, tentacle summaries, Goal summary, recent Need/Feed turns, and harness-learning lines. This is the debug entry when the state JSON is right but app/CLI text confuses Goal, tentacles, or learning state. | `status_surface.rs`, `print_status_report`, `print_context_report`, `status_tentacles`, `harness_learning_line` | 257 |
| User surface boundary | Converts internal state into human Goal hints; keeps agent commands in observer fields so app/CLI users edit Goal only | `user_surface.rs`, `StatusReport`, `ContextReport`, `NeedQueueReport`, `FieldPoolStatusReport`, `ProductReport` | 123 |
| Product report surface | Builds `octopus report` and app-facing capability/gap summaries. It keeps `next` as human Goal hints and `agent_next` as internal executable actions, so the product surface does not become a hidden control panel. | `product_surface.rs`, `ProductReport`, `ProductCapability`, `ProductGap`, `product_report` | 791 |
| Preflight surface boundary | Builds `octopus preflight`, preflight scripts, real-machine record templates/audits/appends, release summary, and preflight printing. This is the debug entry when product readiness, real-machine evidence, or desktop/app release checks look wrong. | `preflight_surface.rs`, `PreflightReport`, `PreflightSummary`, `PreflightRecordCheckReport`, `preflight_report` | 1,101 |
| Doctor surface boundary | Builds `octopus doctor`: state existence, detected environment, bundled/installed manifest health, profile registry, provider setup, pet page, warnings, and next Goal hints. This is the debug entry when a local install looks unhealthy before strategy-level checks. | `doctor_surface.rs`, `DoctorReport`, `DoctorLlmReport`, `doctor_report`, `print_doctor_report` | 302 |
| Harness repair surface | Owns `octopus repair`: repair plan extraction, repair continuation, patch apply/verify, repair scoring, repair outcome journal, and repair report rendering. This is the debug entry when a harness change, repair plan, score, or post-apply check cannot be diagnosed from Need/Feed traces alone. | `repair_surface.rs`, `RepairReport`, `RepairPlanReport`, `RepairScoreReport`, `handle_repair_command` | 6,669 |
| Heartbeat command surface | Owns `octopus beat`: memory keep parsing, early heartbeat pet event, seed-source repair before beat, three-heart report rendering, and harness-beat evolution artifact recommendation. This is the debug entry when the desktop pet looks stale during beat, memory/harness beat output is wrong, or harness beat chooses the wrong repair target. | `heartbeat_surface.rs`, `handle_beat_command`, `print_heartbeat_report`, `write_harness_beat_evolution_artifacts_with_bundled_manifest` | 246 |
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
| Field adaptation core | Field-pack loading, matching, editable aliases, multilingual alias signals, Need annotation, structured peer-field queue context, trace metadata, peer-field worker slots, verifier results, live field mini task loader, editable field-pack task surfaces with concrete pack and registry target files, repair templates, mini task schema guard, LLM template result normalization, and compile/execute template checks | `field_pack.rs`, `field-packs/**`, `tentacles/field-mini-task/**`, `docs/field-adaptation.md` | 5,357 |
| Field adaptation surface | CLI/report rendering for field packs, field matching, field trajectory summaries, verifier results, parallel field worker slots, field-pool latest activity, and user-visible field status lines. This is the debug entry when field state exists but the app/CLI explains the wrong active domain or worker slot. | `field_surface.rs`, `FieldMatchReport`, `print_field_trajectory_report`, `field_pool_status_line`, `parallel_run_status_line` | 464 |
| CLI and product backend | Command dispatch, chat, starter/install/check flows, evolve commands, and surface entry dispatch. Goal/Brain command logic lives in `brain_goal_surface.rs`; Need command logic lives in `need_surface.rs`; Feedback command logic lives in `feedback_surface.rs`; beat command logic lives in `heartbeat_surface.rs`; repair command logic lives in `repair_surface.rs`. | `crates/octopus-core/src/main.rs` | 20,994 |
| Provider env | Loads `.octopus/llm.env` for CLI commands with a scoped guard; existing shell variables win, and values are restored after command execution | `provider_env.rs`, `app_bridge.rs::parse_env_overlay` | 107 |
| Local app bridge | Local HTTP/SSE server, `/api/config` state-path injection, app policy, command allow-list, embedded app/docs/showcase assets, read-only pet supervision access, field activity observer with fresh pet-event handling | `app_bridge.rs`, `docs/app.html` | 2,027 |
| Release and install gates | Low-level release records, benchmark evidence, download/install manifest, and real-machine record parsing used by the preflight surface | `release_gate.rs`, `preflight_surface.rs`, `download.rs`, `docs/real-machine-test.md`, `docs/download.json`, `docs/install.sh` | 2,312 |
| Pet observation surface | Builds the user-visible pet report, auto state choice, event-log view, pixel image report, desktop launch report, and supervision printout. This is the first code boundary after `octopus pet supervise --json` finds an observation mismatch. | `pet_surface.rs`, `pet_report_for_state`, `pet_event_log_report`, `print_pet_supervision_report` | 357 |
| Pet and visual state | Pixel Octopus state, SVG/export helpers, unified event writes, JSONL event audit, latest-event audit coverage, native read-only observer, desktop process/state-path check, stage/error diagnostics, desktop source preflight, HTML preview. Native pet no longer treats stale Feed traces as live state; colors come from fresh pet events, pending Needs, active worker slots, verifier status, goal status, heartbeat, or memory. | `pet.rs`, `pet_surface.rs`, `pet_events.rs`, `pet_supervision.rs`, `desktop_pet.rs`, `desktop/pet/OctopusDesktopPet.swift`, `docs/pet.html`, `tentacles/visual/manifest.json` | 3,101 |
| Strategy diagnostics | Checks the three core traits and composes pet supervision: clean brain context, LLM tool-side tentacles, editable goal, field surface, observation-chain freshness, native desktop process/state-path evidence, stage/error evidence, JSONL latest-event coverage, Feed/evolution evidence | `diagnostics.rs`, `strategy_surface.rs`, `pet_supervision.rs`, `octopus diagnose strategy --json`, `octopus pet supervise --json` | 912 |
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
- Brain/Goal surface rule: `brain_goal_surface.rs` owns Goal editing, Brain command mode validation, Brain session/report dispatch, and clean-brain Need audit rendering. It must not execute tools, mutate Feed traces directly, or become a user control panel beyond Goal changes.
- LLM provider rule: `llm_provider.rs` owns shared model request, retry, timeout, response-content extraction, and Plan JSON recovery. Brain/tentacle modules may call this boundary but should not hand-roll provider requests.
- LLM layer routing rule: `llm_layers.rs` owns chat/brain/manifest/evolve prefix selection, enable flags, and client factories. Provider setup lives in `provider_surface.rs`; request execution lives in `llm_provider.rs`.
- Provider surface rule: `provider_surface.rs` owns provider profiles, env-file generation, secret-safe key save, provider status/check/matrix reports, and client factory selection. CLI/app surfaces may present this data but should not duplicate provider setup logic.
- Manifest catalog rule: `manifest_catalog.rs` owns manifest/profile structs, registry loading, manifest inspection, missing-entrypoint checks, and installed-tentacle conversion. Runtime/state modules consume installed data and should not parse manifest files directly.
- Manifest runtime rule: `manifest_runtime.rs` owns tool-side LLM plan selection, permission grants, command/http execution, multi-action summaries, and `octopus-json-v1` Feed parsing. Brain/state/report modules must not run tools directly.
- Need queue rule: `need_queue.rs` owns queued Need state changes; queue reports must keep `next` user-facing and put executable commands in `agent_next`.
- Need surface rule: `need_surface.rs` owns CLI/report/script/session surfaces for queued Needs. Queue mutation stays in `need_queue.rs`; Feed execution stays in `need_runner.rs`; tool execution stays in `manifest_runtime.rs`. It must not invent Feed or hide field/trace evidence.
- Need runner rule: `need_runner.rs` owns the handoff from queued Need to Feed. It must save the Need pet event before `feed_one`, save the Feed state after `feed_one`, and keep field mini-task/parallel worker evidence attached to the Feed path.
- Feedback surface rule: `feedback_surface.rs` owns CLI Feedback mutation and printing. It may record feedback on an existing Feed trace and append the resulting pet event, but it must not execute tools, synthesize Feed, or change route scoring outside `HarnessState::record_feed_feedback`.
- Route surface rule: `route_surface.rs` owns route report, Feed trace, and Need queue line rendering. It may explain selected tentacles, field context, and latest traces but must not mutate route scores, queues, or Feed records.
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
- Pet supervision rule: `last_pet_event` is the live state, `.octopus/pet-events.jsonl` is the append-only audit trail, `event_log_contains_last` must pass, and `octopus pet supervise --json` is the first diagnostic for desktop observer issues.
- Heartbeat observation rule: `octopus beat` writes a fresh heartbeat pet event before long harness-beat work starts, then writes the final memory/harness event after the beat finishes. A long LLM/provider call must not leave the desktop observer looking dead.
- Pet observation surface rule: `pet_surface.rs` owns pet auto-state selection, event-log report reading, pixel image report printing, desktop launch report printing, and supervision printing. Command dispatch and harness execution should not duplicate pet state logic.
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
- Field surface rule: `field_surface.rs` owns field-adaptation display, matching reports, field-pool lines, verifier result printing, and parallel field run rendering. It must not mutate harness code or bypass field-pack/manifest runtime execution.
- Patch apply rule: provider-created new-file patches are normalized before `git apply`; missing `new file mode 100644` is inserted when a diff uses `--- /dev/null`.
- Release gates: preflight, benchmark evidence, field-pool visibility, real-machine records, local app readiness.
- Tentacle scaffold rule: user-owned code-as-harness seed manifests and python/node/shell starter tools live in `tentacle_scaffold.rs`; new scaffold output must declare LLM brain, tool metadata, runtime code, and editable evolution surfaces.
- Strategy diagnostics: `octopus diagnose strategy --json` checks clean-brain context, user-surface command leakage, tool-side LLM tentacles, editable Goal, field-pack evolution surface, fresh pet events, desktop process/state-path evidence, stage/error evidence, and Feed/evolution evidence.

### Desktop Pet / Evolution Failure Map

Use this before changing UI or harness code:

| Symptom | First check | Meaning | Fix location |
| --- | --- | --- | --- |
| Pet not visible | `octopus pet supervise --json`, check `desktop_process` | Native observer is not running or is reading another state path | `desktop_pet.rs`, `desktop/pet/OctopusDesktopPet.swift` |
| Pet visible but stale | `event_freshness` | Active Octopus loop is not writing fresh events, heartbeat is blocked before its early event, or the report surface is reading the wrong event/state source | caller that should use `pet_events` or `evolution_cycle.rs`; heartbeat entry in `heartbeat_surface.rs`; report mismatch in `pet_surface.rs` |
| Doctor reports unhealthy install | `octopus doctor --json`, then warnings and `llm/profile_registry/pet` fields | Local install, provider config, profile registry, manifest health, or pet page is broken before strategy-level execution | `doctor_surface.rs`, then the boundary named by the warning |
| Preflight says desktop/app is not ready | `octopus preflight --json`, then the failing check id | Product readiness aggregation failed; inspect the check evidence before editing app or harness code | `preflight_surface.rs`, then the boundary named by the check |
| Pet red/blocked during planning | `last_event.stage=planning`, `error_class=provider_timeout/provider_error` | LLM planner did not return a harness candidate | provider config/client and evolution planner boundary |
| Pet red/blocked during applying | `stage=applying`, `error_class=patch_*` | LLM produced an invalid, missing, or unauthorized patch | apply/patch normalization boundary |
| Pet red/blocked during checking | `stage=checking`, `error_class=check_failed` | Harness patch applied but its declared check failed | tentacle check policy or editable harness code |
| Pet red/blocked during feeding | `stage=feeding`, `error_class=feed_missing_queued_need` | No queued Need actually reached Feed; the system must not report fake success | `evolution_feed.rs` queued-Need boundary |
| Pet shows Need but no Feed follows | inspect `NeedRunReport`, latest pet event, and `.octopus/state.json` around the queue item | Queued Need was taken but did not reach `feed_one`, or observer state was not saved after Feed | `need_runner.rs`, affected manifest runtime |
| User can change too many things, or Brain/Goal text is confusing | `octopus goal --json`, `octopus brain --session --json`, then compare printed output | Goal/Brain surface leaked internal modes into the user path, or audit text hid polluted Needs | `brain_goal_surface.rs`; Need generation stays in `brain_loop.rs` |
| Brain suggests tool steps instead of Needs | inspect `brain/context` report and Need audit | Clean-brain loop leaked implementation details into Need | `brain_loop.rs` |
| LLM call hangs or returns empty/invalid content | inspect provider env and provider response error | Request timeout, retry, response extraction, or JSON recovery failed | `llm_provider.rs`, `provider_env.rs` |
| Provider appears configured but brain/tentacle/evolution layer does not use it | `octopus provider status --json`, `octopus provider check <PREFIX>`, provider matrix record | Profile/env/status or layer prefix selection is wrong | `provider_surface.rs`, `llm_layers.rs`, `.octopus/llm.env`, `.octopus/providers/*.env` |
| Tentacle thinks but no action or wrong Feed returns | `octopus think ... --json`, Feed metadata `plan_source`, `tools`, `contract`, `authorization_required` | Need reached a local tool brain, but planning, authorization, execution, or Feed parsing failed | `manifest_runtime.rs`, affected `tentacles/*/manifest.json`, affected tool code |
| Manifest install or inspect reports wrong metadata | `octopus manifests <root> --json`, missing entrypoints, installed tentacle metadata | Manifest/profile loading, report derivation, or installed conversion is wrong | `manifest_catalog.rs`, affected `tentacles/*/manifest.json` |
| Queued Need cannot be run/taken/dropped | inspect `needs --json` and item status | Queue mutation or user/agent next split is wrong | `need_queue.rs` |
| Need queue text/script/session is wrong | compare `octopus needs --json`, `octopus needs script --json`, `octopus needs session --json`, and generated files | Queue data exists, but the command surface hid evidence or generated the wrong observer artifact | `need_surface.rs`; mutation stays in `need_queue.rs`, execution stays in `need_runner.rs` |
| Route or trace text hides the selected tentacle/field | compare `routes --json`, `traces --json`, and CLI/app text | Routing or Feed data is present, but the observation surface is misleading | `route_surface.rs`; score data stays in `lib.rs` route reports |
| Feedback records wrong status, route score, or pet state | compare `octopus feedback latest <status> --json`, latest trace, and `.octopus/state.json.last_pet_event` | Feed trace exists, but feedback mutation, route score update, or pet event append is wrong | `feedback_surface.rs`; feedback state mutation stays in `HarnessState::record_feed_feedback` |
| App status/context shows wrong next step | compare `next` with `agent_next` in `status/context/needs` JSON | Read-only user hints leaked internal commands, or field-pool state was derived incorrectly | `state_report.rs`, `user_surface.rs` |
| Status/context text misstates Goal, tentacles, or learning | compare `status --json`, `context --json`, and CLI/app text | Derived state is right, but rendering is misleading | `status_surface.rs`; derivation stays in `state_report.rs` |
| Report/app suggests commands as user actions | compare product `next` and `agent_next` | Product surface leaked internal agent controls into the user Goal path | `product_surface.rs`, `user_surface.rs` |
| Field summary shows wrong active field or worker slot | `octopus fields summary --json`, then compare app/CLI text | Field data exists but the rendering or latest-activity line is misleading | `field_surface.rs`; data derivation stays in `state_report.rs` and `field_pack.rs` |
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
| `crates/octopus-core/src` | 55 | 58,980 |
| `crates/octopus-core/examples` | 0 | 0 |
| `tentacles` | 83 | 18,974 |
| `field-packs` | 14 | 598 |
| `desktop/pet` | 1 | 841 |
| `docs` | 29 md/html/json/sh files | 7,937 |
| `cowork` | 3 | 101 |
| `local/docs` | 12 | 507 |

### Core Rust Files

| File | Lines |
| --- | ---: |
| `main.rs` | 20,994 |
| `lib.rs` | 8,999 |
| `repair_surface.rs` | 6,669 |
| `provider_surface.rs` | 1,887 |
| `brain_goal_surface.rs` | 1,610 |
| `app_bridge.rs` | 1,167 |
| `preflight_surface.rs` | 1,101 |
| `manifest_runtime.rs` | 1,090 |
| `brain_loop.rs` | 1,082 |
| `field_pack.rs` | 868 |
| `product_surface.rs` | 791 |
| `state_report.rs` | 734 |
| `llm_provider.rs` | 704 |
| `release_gate.rs` | 703 |
| `evolution_recommend.rs` | 679 |
| `need_surface.rs` | 663 |
| `need_runner.rs` | 544 |
| `pet_supervision.rs` | 518 |
| `field_surface.rs` | 464 |
| `bundled_harness.rs` | 423 |
| `evolution_artifact.rs` | 401 |
| `desktop_pet.rs` | 377 |
| `manifest_catalog.rs` | 373 |
| `pet_surface.rs` | 357 |
| `diagnostics.rs` | 354 |
| `evolution_driver.rs` | 318 |
| `evolution_target.rs` | 318 |
| `doctor_surface.rs` | 302 |
| `llm_layers.rs` | 300 |
| `tentacle_scaffold.rs` | 291 |
| `evolution_apply.rs` | 282 |
| `pet.rs` | 280 |
| `status_surface.rs` | 257 |
| `evolution_prompt.rs` | 254 |
| `heartbeat_surface.rs` | 246 |
| `evolution_patch.rs` | 245 |
| `need_queue.rs` | 234 |
| `evolution_candidate.rs` | 233 |
| `evolution_contract.rs` | 222 |
| `core_boundary.rs` | 215 |
| `download.rs` | 175 |
| `evolution_cycle.rs` | 153 |
| `route_surface.rs` | 143 |
| `evolution_feed.rs` | 131 |
| `user_surface.rs` | 123 |
| `evolution_drive_surface.rs` | 116 |
| `provider_env.rs` | 107 |
| `evolution_plan.rs` | 84 |
| `evolution.rs` | 78 |
| `profile_registry.rs` | 78 |
| `pet_events.rs` | 74 |
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
- Cleanup audit after the evolution-boundary commits moved prompt, candidate, artifact, patch, target, contract, recommendation, LLM planner, scaffold, clean-brain loop, Need queue, Need runner, Feedback surface, heartbeat surface, route surface, state-report, status surface, field surface, LLM provider, LLM layer routing, manifest catalog, manifest runtime, provider surface, preflight surface, doctor surface, and strategy surface responsibilities out of the largest files. Remaining discomfort: `main.rs` is still product-backend aggregation, and `lib.rs` still owns broad state mutation.
- Evolution surface requirements now belong to manifests. Rust validates declared missing surfaces and must not grow domain-specific trigger rules.
- Current field-mini-task template coverage is 33/33 satisfied in source and bundled seed. The apply path now normalizes LLM patch file headers before `git apply`, runs an apply precheck first, and field-mini-task normalizes common LLM template result shapes into `octopus-json-v1` Feed. The next `0.2.x` track should let Octopus add harder mini task layers one field at a time: math, search, SWE, research, computer-use, IB, robotics, then continue write and translate.
- The release showcase is screenshot-first. The local app surface in `docs/app.html` observes and updates the real local Octopus loop.
