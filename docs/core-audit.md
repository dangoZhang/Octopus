# Core Audit Gate

Required for `0.0.24` before `0.1.0`.

## Goal

Ship the smallest honest Octopus core:

- clean brain: `Goal + Mem + Need + Feed`
- tentacle intelligence: `Need + Tool + Action -> Feed`
- three beats: heartbeat, memory beat, harness-evolution beat
- editable code-as-harness outside the stable kernel
- product start/app/preflight path that really runs locally

## Keep

- Rust kernel contracts for brain, Need, Feed, Feedback, route learning, memory, heartbeat, harness evolution, provider access, app bridge, release gates, and product startup.
- Product app code that is stable infrastructure.
- Tentacle manifests, prompts, tool metadata, runtime code, and evolution policy as editable harness units.
- Regression tests that prove product invariants, release gates, context boundaries, provider routing, repair feedback, and local startup.
- Current-head local app evidence from `octopus start --check`, stored as `.octopus/local-app-run.json` and checked by preflight.

## Remove Or Move

- Dead prototype code, old demo commands, duplicate product paths, stale generated artifacts, and hidden temporary flows.
- Development-only tests or fixtures that only preserve past implementation accidents.
- Fallback logic that chooses implementation for the clean brain or leaks tool/API/file burden into brain context.
- Hard-coded seed harness data in Rust when it can live in manifest/profile registry files.
- Internal controls exposed as first-use product actions outside Goal/chat/first-run.

## Evidence

The `0.0.24` audit must update this file with:

- reviewed file tree snapshot
- kept core modules
- removed code/tests/docs
- moved harness units
- remaining deliberate exceptions
- commands run
- current commit hash

`0.1.0` cannot be cut until this evidence exists and the real-machine gate is recorded.

## Audit Pass 1: Hidden Tool-Thinking Fallback

- Commit scanned: `53b3ea4`.
- Reviewed tree snapshot: `README*`, `Cargo.*`, `structure.md`, `crates/octopus-core/src/{lib.rs,main.rs,app_bridge.rs,core_boundary.rs,release_gate.rs}`, `crates/octopus-core/examples/thinking_tentacle.rs`, `tentacles/{profile-registry,swe-agent,computer-use-agent,repo-maintainer,harness-repair-agent,bash-only,json-feed,visual}`, and `docs/{app,tutorial,recipes,download,install,pet,architecture,product-gap,version-plan,real-machine-test,zh}`.
- Kept core modules: Rust kernel contracts, planner/tentacle traits, app bridge, release gate, core boundary diagnostics, product startup, provider adapters, memory/heartbeat/harness evolution state.
- Removed code: `ChatPlanner` no longer stores or invokes a hidden `RulePlanner` fallback after LLM failure or invalid Plan JSON.
- Kept deliberate exception: `RulePlanner` remains as an explicit planner for deterministic tests and the small `examples/thinking_tentacle.rs` example; it is not hidden inside tool-side LLM planning.
- Moved harness units: none in this pass.
- Remaining candidates: shrink `main.rs` product/backend aggregation, audit legacy executable tool contracts, review historical docs/log residue, and keep hard-coded seed data out of Rust.
- Commands: `rg --files`, `find . -maxdepth 3 -type d`, fallback/residue `rg`, `git status --ignored=matching`, `wc -l crates/octopus-core/src/*.rs`, `cargo fmt --all --check`, `cargo check -q -p octopus-core`, `cargo test -q -p octopus-core chat_planner_uses_chat_client_plan`, `cargo test -q -p octopus-core chat_planner_does_not_fallback_to_rule_execution`, `cargo test -q -p octopus-core planning_tentacle_selects_tool_before_execution`.

## Audit Pass 2: Explicit Tool Call Contracts

- Commit scanned: `f72322e`.
- Reviewed files: `tentacles/*/manifest.json`, `tentacles/profile-registry/default.json`, `tentacles/tentacle.schema.json`, `tentacles/README.md`, `docs/architecture.md`, `docs/zh/architecture.md`, and manifest execution in `crates/octopus-core/src/lib.rs`.
- Removed residue: seed tools no longer rely on omitted contract fields to mean a legacy executable entrypoint.
- Kept core behavior: `octopus-json-v1` still receives the full JSON envelope; `stdio-argv-v1`, `adapter-v1`, `static-html-v1`, and `native-harness-v1` document non-JSON contracts without changing their existing runtime path.
- Moved harness units: none; the change keeps contract metadata in editable seed/profile files.
- Remaining candidates: shrink `main.rs`, review historical docs/log residue, and keep hard-coded seed data out of Rust.
- Commands: manifest/profile JSON parse and missing-contract scan, `cargo fmt --all --check`, `cargo check -q -p octopus-core`, `cargo test -q -p octopus-core default_catalog_contains_installable_profiles`, `cargo test -q -p octopus-core seed_manifests_declare_explicit_tool_contracts`, `cargo test -q -p octopus-core harness_state_installs_manifest_tentacle_package`.

## Audit Pass 3: Bundled Harness Materializer Boundary

- Commit scanned: `e8b5715`.
- Reviewed files: bundled seed tentacle startup path in `crates/octopus-core/src/main.rs`, new `crates/octopus-core/src/bundled_harness.rs`, startup materialization tests, `structure.md`, and this audit log.
- Removed residue: the large embedded seed file table and install-binary harness materialization helpers no longer live in the CLI aggregation file.
- Kept deliberate exception: installed binaries still embed seed harness files only so `start` and `bootstrap` can materialize editable `.octopus/bundled-tentacles` when source `tentacles/` files are unavailable.
- Moved harness units: the bundle fallback materializer moved to `bundled_harness.rs`; editable source-of-truth manifests, tools, and profile data remain under `tentacles/`.
- Remaining candidates: continue shrinking `main.rs`, review historical docs/log residue, and remove any stale fallback that chooses Feed implementation for the clean brain.
- Commands: `rg` for bundled seed symbols, `cargo fmt --all --check`, `cargo check -q -p octopus-core`, `cargo test -q -p octopus-core bundled_tentacles_materialize_as_editable_startup_surface`, `cargo test -q -p octopus-core bundled_tentacles_materialize_from_current_directory`, `cargo test -q -p octopus-core cli_bootstrap_installs_seed_tentacles`.

## Audit Pass 4: Download Manifest Boundary

- Commit scanned: `0c21dae`.
- Reviewed files: download/install report structs, `download_report`, `download_artifacts_preflight_check`, `update` install command sharing, download manifest tests, `structure.md`, and this audit log.
- Removed residue: download/install manifest generation and artifact release checks no longer live in the CLI aggregation file.
- Kept core behavior: `octopus download`, `octopus update`, `docs/download.json`, `docs/install.sh`, local static serving, and the required `download_artifacts` preflight gate share the same command and manifest contract.
- Moved harness units: none; this pass separates product release plumbing from the CLI dispatcher.
- Remaining candidates: continue shrinking `main.rs`, review historical docs/log residue, and remove any stale fallback that chooses Feed implementation for the clean brain.
- Commands: `rg` for download symbols, `cargo fmt --all --check`, `cargo check -q -p octopus-core`, `cargo test -q -p octopus-core static_download_manifest_matches_cli_report`, `cargo test -q -p octopus-core download_artifacts_preflight_check_passes_for_current_docs`, `cargo test -q -p octopus-core download_report_exposes_install_update_and_pages_paths`.

## Audit Pass 5: Shared Shell Display Quoting

- Commit scanned: `37d0d92`.
- Reviewed files: shell command display helpers in `crates/octopus-core/src/main.rs`, `crates/octopus-core/src/download.rs`, new `crates/octopus-core/src/shell_words.rs`, download/update tests, `structure.md`, and this audit log.
- Removed residue: duplicate shell command quoting helpers no longer exist separately in the CLI aggregation file and download module.
- Kept core behavior: command strings used by update, install, download, preflight records, provider matrix, and brain session files keep the same quoting rules.
- Moved harness units: none; this pass removes duplicated product plumbing.
- Remaining candidates: continue shrinking `main.rs`, review historical docs/log residue, and remove any stale fallback that chooses Feed implementation for the clean brain.
- Commands: `rg` for shell helper symbols, `cargo fmt --all --check`, `cargo check -q -p octopus-core`, `cargo test -q -p octopus-core download_report_exposes_install_update_and_pages_paths`, `cargo test -q -p octopus-core update_report_defaults_to_dry_run_command`, `cargo test -q -p octopus-core static_download_manifest_matches_cli_report`.

## Audit Pass 6: Pixel Pet Boundary

- Commit scanned: `e404705`.
- Reviewed files: pet reports, state table, SVG renderer, file URL encoding, `pet` CLI/image tests, `structure.md`, and this audit log.
- Removed residue: pixel pet state/rendering helpers no longer live in the CLI aggregation file.
- Kept core behavior: harness status still chooses the automatic pet state in the product flow, while `pet.rs` owns deterministic color state, SVG export, and browser file URLs.
- Moved harness units: none; this pass separates the non-evolving product display layer from the CLI dispatcher.
- Remaining candidates: continue shrinking `main.rs`, review historical docs/log residue, and remove any stale fallback that chooses Feed implementation for the clean brain.
- Commands: `rg` for pet symbols, `cargo fmt --all --check`, `cargo check -q -p octopus-core`, `cargo test -q -p octopus-core pet_report_maps_state_to_local_target`, `cargo test -q -p octopus-core pet_image_writes_pixel_svg`, `cargo test -q -p octopus-core pet_auto_state_prefers_recent_action_event`.

## Audit Pass 7: Profile Registry Boundary

- Commit scanned: `c1f6d59`.
- Reviewed files: profile registry report struct, source/path selection, parse diagnostics, doctor/report/preflight call sites, `structure.md`, and this audit log.
- Removed residue: editable seed profile registry observation no longer lives in the CLI aggregation file.
- Kept core behavior: `OCTOPUS_PROFILE_REGISTRY`, state-local registry, cwd-local registry, and embedded default registry keep the same load order and diagnostics.
- Moved harness units: none; this pass separates editable harness registry observation from CLI dispatch.
- Remaining candidates: one more product cleanup pass, review historical docs/log residue, then perform the cleanup/version commit for `0.0.24`.
- Commands: `rg` for profile registry symbols, `cargo fmt --all --check`, `cargo check -q -p octopus-core`, `cargo test -q -p octopus-core cli_status_and_doctor_commands_run`, `cargo test -q -p octopus-core start_check_writes_local_app_evidence_for_preflight`, `cargo test -q -p octopus-core first_run_report_writes_scored_feed_evidence`.

## Audit Pass 8: Boundary Report Matches Split Modules

- Commit scanned: `af1b19b`.
- Reviewed files: `core_boundary.rs`, split stable Rust modules, product report/preflight boundary tests, `docs/product-gap.md`, `docs/version-plan.md`, and this audit log.
- Removed residue: the boundary report no longer describes only the old four-file Rust surface after the CLI split.
- Kept core behavior: stable Rust remains the kernel/product/release/startup shell, while editable Feed supply remains in `tentacles/` and profile registry data.
- Moved harness units: none; this pass updates release evidence to match the current source layout.
- Remaining candidates: cleanup/version commit for `0.0.24`, then real-machine gate evidence before `0.1.0`.
- Commands: `rg` for core boundary symbols, `cargo fmt --all --check`, `cargo check -q -p octopus-core`, `cargo test -q -p octopus-core start_check_writes_local_app_evidence_for_preflight`, `cargo test -q -p octopus-core cli_status_and_doctor_commands_run`, `cargo test -q -p octopus-core first_run_report_writes_scored_feed_evidence`.

## 0.0.24 Closeout

- Commit scanned: `e5cbf2c`.
- Completed passes: hidden planner fallback removal, explicit seed tool contracts, bundled harness boundary, download manifest boundary, shared shell display quoting, pixel pet boundary, profile registry boundary, and boundary report alignment.
- Current source shape: stable Rust owns kernel/product/release/startup support; editable Feed supply remains in `tentacles/` and profile registry data.
- Remaining before `0.1.0`: recorded real-machine gate evidence on the `0.0.24` line.
- Commands: `rg` version scan, `cargo fmt --all --check`, `cargo check -q -p octopus-core`, `cargo test -q -p octopus-core static_download_manifest_matches_cli_report`, `cargo test -q -p octopus-core download_artifacts_preflight_check_passes_for_current_docs`, `cargo test -q -p octopus-core start_check_writes_local_app_evidence_for_preflight`.
