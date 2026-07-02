use super::*;

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProductReport {
    pub(crate) version: String,
    pub(crate) state_path: String,
    pub(crate) state_exists: bool,
    pub(crate) context: ProductContextPolicy,
    pub(crate) field_pool: ProductFieldPoolReport,
    pub(crate) boundary: core_boundary::CoreBoundaryReport,
    pub(crate) harness_learning: HarnessLearningSummary,
    pub(crate) capabilities: Vec<ProductCapability>,
    pub(crate) gaps: Vec<ProductGap>,
    pub(crate) agent_next: Vec<String>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProductContextPolicy {
    pub(crate) brain: String,
    pub(crate) tentacle: String,
    pub(crate) feedback_loop: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProductFieldPoolReport {
    pub(crate) policy: String,
    pub(crate) field_count: usize,
    pub(crate) field_slot_count: usize,
    pub(crate) latest_worker_slot_count: usize,
    pub(crate) latest_activity: String,
    pub(crate) fields: Vec<String>,
    pub(crate) completed_fields: usize,
    pub(crate) active_slot_field: Option<String>,
    pub(crate) active_slot_reason: String,
    pub(crate) worker_slots: String,
    pub(crate) next: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProductCapability {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) evidence: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) command: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProductGap {
    pub(crate) id: String,
    pub(crate) impact: String,
    pub(crate) user_goal_hint: String,
    pub(crate) agent_next: String,
    pub(crate) next: String,
}

pub(crate) fn product_report(
    state: &HarnessState,
    state_path: &Path,
) -> Result<ProductReport, String> {
    let status = state.status_report_with_state(Some(state_path));
    let cwd = env::current_dir().map_err(|error| error.to_string())?;
    let environment = EnvironmentReport::detect(&cwd);
    let manifests =
        inspect_tentacle_manifests_with_bundled_seed_overlay(&default_tentacles_root())?;
    let provider = provider_status_report();
    let provider_matrix = check_provider_matrix_record(Path::new(DEFAULT_PROVIDER_MATRIX_PATH))?;
    let profile_registry = profile_registry_report(state_path);
    let boundary = core_boundary::report(state_path);
    let field_trajectory = state.field_trajectory_report_with_state(Some(state_path))?;
    let field_pool = product_field_pool_report(&field_trajectory, status.field_pool.as_ref());
    let pet_exists = repo_root().join("docs/pet.html").exists();
    let app_exists = repo_root().join("docs/app.html").exists();
    let docs_exists = repo_root().join("docs/index.html").exists();
    let swiftc_ready = command_ready("swiftc");
    let broken_manifests = manifests
        .iter()
        .filter(|manifest| !manifest.missing_entrypoints.is_empty())
        .map(|manifest| manifest.id.clone())
        .collect::<Vec<_>>();
    let broken_installed_seed_sources = collect_broken_installed_seed_sources(state);
    let runtime_kinds = manifests
        .iter()
        .flat_map(|manifest| manifest.runtime_kinds.iter().cloned())
        .collect::<BTreeSet<_>>();
    let required_surfaces = [
        "brain_prompt",
        "tool_meta",
        "runtime_code",
        "evolution_policy",
    ];
    let harness_surfaces_ready = manifests.iter().any(|manifest| {
        required_surfaces.iter().all(|surface| {
            manifest
                .evolution_surfaces
                .iter()
                .any(|item| item == surface)
        })
    });
    let provider_configured = provider.layers.iter().any(|layer| layer.configured);
    let provider_ready = provider_coverage_ready(&provider);
    let github_ready = state.active_grant("github", "dangoZhang/Octopus").is_some();

    let mut capabilities = vec![
        product_capability(
            "core_harness_boundary",
            if boundary.ok {
                "ready"
            } else {
                "needs_attention"
            },
            format!(
                "stable_rust={}, editable_harness={}, warnings={}",
                boundary.stable_rust.len(),
                boundary.editable_harness.len(),
                boundary.warnings.len()
            ),
            Some("octopus report"),
        ),
        product_capability(
            "clean_brain",
            "ready",
            "brain context is Goal + Mem + Need + Feed",
            Some("octopus context observe ."),
        ),
        product_capability(
            "context_inspection",
            "ready",
            "CLI, JSON, and native app expose clean-brain and tentacle context boundaries",
            Some("octopus context observe ."),
        ),
        product_capability(
            "clean_brain_explore",
            "ready",
            "Goal/Mem/Need/Feed exploration returns Need suggestions without tool execution",
            Some("octopus explore"),
        ),
        product_capability(
            "clean_brain_intent",
            "ready",
            "Goal/Mem/Need/Feed intent selection returns cognitive Needs without tool execution",
            Some("octopus brain --intent"),
        ),
        product_capability(
            "clean_brain_brief",
            "ready",
            "Goal/Mem/Need/Feed brief compacts clean context for the next model turn without tool execution",
            Some("octopus brain --brief"),
        ),
        product_capability(
            "clean_brain_align",
            "ready",
            "Goal/Mem/Need/Feed alignment checks Goal constraints and clean next Needs without Feed execution",
            Some("octopus brain --align"),
        ),
        product_capability(
            "clean_brain_agenda",
            "ready",
            "Goal/Mem/Need/Feed agenda returns cognitive priorities and clean Needs without Feed execution",
            Some("octopus brain --agenda"),
        ),
        product_capability(
            "clean_brain_scout",
            "ready",
            "Goal/Mem/Need/Feed scout maps assumptions, unknowns, directions, risks, and clean Needs without Feed execution",
            Some("octopus brain --scout"),
        ),
        product_capability(
            "clean_brain_deliberation",
            "ready",
            "Goal/Mem/Need/Feed deliberation returns observations, questions, options, risks, and clean Needs without Feed execution",
            Some("octopus brain --deliberate"),
        ),
        product_capability(
            "clean_brain_audit",
            "ready",
            "clean-brain Goal, agenda, and exploration reports flag implementation burden and expose only clean Needs for queueing",
            Some("octopus brain --live \"what should the brain ask next?\""),
        ),
        product_capability(
            "clean_brain_rewrite",
            "ready",
            "provider-backed or external-chat rewrite turns polluted model Needs into cognitive Needs before queueing",
            Some("octopus brain --rewrite --live --apply reply.json --save"),
        ),
        product_capability(
            "clean_brain_model_slots",
            "ready",
            "Goal, Intent, Brief, Align, Clarify, Agenda, Scout, Deliberate, Explore, Rewrite, and Need Queue review can use separate provider prefixes without changing brain context",
            Some("octopus provider status"),
        ),
        product_capability(
            "need_queue",
            "ready",
            format!(
                "{} pending clean-brain Need suggestions",
                status.need_queue_count
            ),
            Some("octopus explore --save"),
        ),
        product_capability(
            "need_queue_script",
            "ready",
            "writes pending clean-brain Needs into a reviewable Feed script without execution",
            Some("octopus needs script"),
        ),
        product_capability(
            "need_queue_review",
            "ready",
            "writes prompt, messages, review template, and optional live draft for pending Needs",
            Some("octopus needs session"),
        ),
        product_capability(
            "harness_repair_queue",
            if state
                .installed_tentacles
                .iter()
                .any(|tentacle| tentacle.id == "harness-repair-agent")
            {
                "ready"
            } else {
                "available"
            },
            "harness-repair Feed can queue the next clean-brain Need",
            Some("octopus repair ."),
        ),
        product_capability(
            "clean_brain_prompt",
            "ready",
            "exports pasteable Goal/Mem/Need/Feed messages for any chat model",
            Some("octopus brain"),
        ),
        product_capability(
            "external_brain_reply",
            "ready",
            "imports chat-model JSON into Goal or Need Queue without Feed execution",
            Some("octopus brain --apply reply.json --save"),
        ),
        product_capability(
            "brain_session",
            "ready",
            "writes prompt, messages, reply template, optional live draft, and apply command for external chat",
            Some("octopus brain --session"),
        ),
        product_capability(
            "tentacle_brains",
            if status.tentacles.is_empty() {
                "available"
            } else {
                "ready"
            },
            format!(
                "{} installed, {} manifests",
                status.tentacles.len(),
                manifests.len()
            ),
            Some("octopus install swe-agent"),
        ),
        product_capability(
            "runtime_neutral_harness",
            if harness_surfaces_ready {
                "ready"
            } else {
                "needs_manifest"
            },
            format!(
                "runtimes: {}; surfaces: {}",
                join_or_none(&runtime_kinds.iter().cloned().collect::<Vec<_>>()),
                required_surfaces.join(", ")
            ),
            Some("octopus manifests"),
        ),
        product_capability(
            "field_parallel_pool",
            if product_field_pool_ready(&field_pool) {
                "ready"
            } else {
                "needs_field_packs"
            },
            product_field_pool_line(&field_pool, Language::En),
            Some("octopus fields summary"),
        ),
        field_mini_task_harness_product_capability(),
        product_capability(
            "profile_registry",
            if profile_registry.ok {
                "ready"
            } else {
                "invalid"
            },
            format!(
                "source={}, path={}, profiles={}",
                profile_registry.source, profile_registry.path, profile_registry.profile_count
            ),
            Some("octopus bootstrap"),
        ),
        product_capability(
            "three_hearts",
            if status.hearts.len() >= 3 {
                "ready"
            } else {
                "missing"
            },
            status
                .hearts
                .iter()
                .map(|beat| format!("{}={}", beat.name, beat.summary))
                .collect::<Vec<_>>()
                .join(", "),
            Some("octopus beat 200"),
        ),
        product_capability(
            "color_pet",
            if pet_exists { "ready" } else { "missing" },
            repo_root()
                .join("docs/pet.html")
                .to_string_lossy()
                .to_string(),
            Some("octopus pet"),
        ),
        product_capability(
            "native_desktop_pet",
            if cfg!(target_os = "macos") && swiftc_ready {
                "ready"
            } else if cfg!(target_os = "macos") {
                "needs_swiftc"
            } else {
                "macos_only"
            },
            format!(
                "observer=read-only, macos={}, swiftc={}, preflight_gate=desktop_pet_source",
                cfg!(target_os = "macos"),
                swiftc_ready
            ),
            Some("octopus preflight"),
        ),
        product_capability(
            "native_html_app",
            if app_exists { "ready" } else { "missing" },
            repo_root()
                .join("docs/app.html")
                .to_string_lossy()
                .to_string(),
            Some("octopus start --open"),
        ),
        product_capability(
            "local_project_start",
            if app_exists { "ready" } else { "missing" },
            "start prepares local state, seed tentacles, heartbeat state, serves the native app, and can open it",
            Some("octopus start --open"),
        ),
        product_capability(
            "local_update",
            "ready",
            "dry-run update report plus explicit cargo install runner",
            Some("octopus update"),
        ),
        product_capability(
            "starter_panel",
            if app_exists { "ready" } else { "missing" },
            "native HTML app renders starter recommendations with evidence signals, choice feedback, install, check, and first-Need actions",
            Some("octopus starter \"build a clean-brain agent\""),
        ),
        product_capability(
            "local_bootstrap",
            "ready",
            "init, environment adaptation, seed tentacle install, heartbeat, and report in one command",
            Some("octopus bootstrap"),
        ),
        product_capability(
            "harness_apply_review",
            "ready",
            "harness beat recommendations can be granted and written as reviewable apply artifacts",
            Some("octopus evolve apply swe-agent 03-runtime-code"),
        ),
        product_capability(
            "harness_self_repair",
            if status
                .tentacles
                .iter()
                .any(|tentacle| tentacle.id == "harness-repair-agent")
            {
                "ready"
            } else {
                "available"
            },
            format!(
                "harness-repair-agent can turn signals into repair Feed; {} scored repair outcomes",
                status.repair_outcome_count
            ),
            Some("octopus install harness-repair-agent"),
        ),
        product_capability(
            "harness_learning",
            if status.harness_learning.source == "none" {
                "waiting_for_feedback"
            } else {
                "ready"
            },
            harness_learning_line(&status.harness_learning),
            Some(status.harness_learning.next_action.as_str()),
        ),
        product_capability(
            "goal_setting",
            "ready",
            "human goals and constraints can be set or refined without Feed execution",
            Some("octopus goal set --constraint \"keep tools outside the brain\" \"build a clean-brain agent\""),
        ),
        product_capability(
            "route_report",
            "ready",
            format!(
                "{} route scores, {} Feed traces",
                status.route_count, status.feed_trace_count
            ),
            Some("octopus routes observe ."),
        ),
        product_capability(
            "provider_layers",
            if provider_ready {
                "ready"
            } else if provider_configured {
                "configured"
            } else {
                "setup_needed"
            },
            provider
                .coverage
                .iter()
                .map(|coverage| {
                    format!(
                        "{} ready={} prefixes={}",
                        coverage.id,
                        coverage.ready,
                        join_or_none(&coverage.prefixes)
                    )
                })
                .collect::<Vec<_>>()
                .join("; "),
            Some("octopus provider status"),
        ),
        product_capability(
            "provider_matrix_record",
            if provider_matrix.passed {
                "ready"
            } else {
                "needs_record"
            },
            provider_matrix_check_evidence(&provider_matrix),
            Some("octopus provider matrix check"),
        ),
        product_capability(
            "self_iteration",
            if github_ready {
                "pr_ready"
            } else {
                "oauth_required"
            },
            state.self_iteration_plan("dangoZhang/Octopus", None).mode,
            Some("octopus self-iterate dangoZhang/Octopus"),
        ),
        product_capability(
            "docs_home",
            if docs_exists { "ready" } else { "missing" },
            "README story plus native docs homepage".to_string(),
            Some("open docs/index.html"),
        ),
    ];
    capabilities.sort_by(|left, right| left.id.cmp(&right.id));

    let mut gaps = Vec::new();
    let state_args = format!(" --state {}", shell_arg(&state_path.to_string_lossy()));
    if status.goal.is_none() {
        gaps.push(product_gap(
            "goal_loop_empty",
            "human goal has not been set in this state",
            format!("octopus{state_args} goal set \"describe your goal\""),
        ));
    }
    if status.tentacles.is_empty() {
        gaps.push(product_gap(
            "no_installed_tentacles",
            "Needs cannot reach tool-side brains yet",
            format!("octopus{state_args} bootstrap"),
        ));
    }
    if !broken_manifests.is_empty() {
        gaps.push(product_gap(
            "broken_manifests",
            format!("missing entrypoints: {}", broken_manifests.join(", ")),
            "octopus manifests",
        ));
    }
    if !broken_installed_seed_sources.is_empty() {
        gaps.push(product_gap(
            "broken_installed_seed_sources",
            format!(
                "installed seed sources need repair: {}",
                broken_installed_seed_sources.join(", ")
            ),
            format!("octopus{state_args} beat 200"),
        ));
    }
    if !provider_ready {
        let next = if provider_configured {
            "octopus provider check"
        } else {
            "octopus provider save openai"
        };
        gaps.push(product_gap(
            "provider_feedback",
            "LLM-backed Goal chat, clean brain, tentacle planning, or evolution coverage is incomplete",
            next,
        ));
    }
    if !provider_matrix.passed {
        gaps.push(product_gap(
            "provider_matrix_record",
            "current-head OAuth, API-key, local model, and gateway provider evidence is incomplete",
            "octopus provider matrix; octopus provider matrix run; octopus provider matrix check",
        ));
    }
    if status.feed_trace_count == 0 {
        gaps.push(product_gap(
            "feed_trace_empty",
            "harness evolution has no Feed data to learn from",
            "octopus need observe .",
        ));
    }
    if status.check_history_count == 0 {
        gaps.push(product_gap(
            "check_history_empty",
            "harness beat cannot target failing runtime checks yet",
            "octopus check swe-agent",
        ));
    }
    if !profile_registry.ok {
        gaps.push(product_gap(
            "profile_registry_invalid",
            profile_registry
                .error
                .clone()
                .unwrap_or_else(|| "profile registry is not readable".to_string()),
            "octopus bootstrap",
        ));
    }
    if !boundary.ok {
        gaps.push(product_gap(
            "core_harness_boundary",
            boundary.warnings.join("; "),
            "octopus report",
        ));
    }
    if state.evolution_outcomes.is_empty() {
        gaps.push(product_gap(
            "evolution_scores_empty",
            "recommendations have not received outcome feedback",
            "octopus evolve score swe-agent 03-runtime-code partial \"reviewed\"",
        ));
    }
    if status.repair_outcome_count == 0 {
        gaps.push(product_gap(
            "repair_outcomes_empty",
            "harness repair sessions have no scored outcome memory yet",
            "octopus repair score <trace-index> satisfied \"repair improved harness\"",
        ));
    }
    if environment.recommended_profiles.is_empty() {
        gaps.push(product_gap(
            "environment_signal_sparse",
            "adapter selection has little local project evidence",
            "octopus env",
        ));
    }
    gaps.push(product_gap(
        "real_machine_gate",
        "0.1.0 and later tags need recorded real-machine testing",
        "update docs/real-machine-test.md before a 0.1.0 tag",
    ));
    gaps.sort_by(|left, right| left.id.cmp(&right.id));

    let mut agent_next = gaps
        .iter()
        .map(|gap| gap.agent_next.clone())
        .collect::<Vec<_>>();
    agent_next.push(status.agent_next_action.clone());
    agent_next.extend(provider.next);
    agent_next.sort();
    agent_next.dedup();

    let mut next = gaps
        .iter()
        .map(|gap| gap.user_goal_hint.clone())
        .collect::<Vec<_>>();
    next.push(status.user_goal_hint.clone());
    next.sort();
    next.dedup();

    Ok(ProductReport {
        version: env!("CARGO_PKG_VERSION").to_string(),
        state_path: state_path.to_string_lossy().to_string(),
        state_exists: state_path.exists(),
        context: ProductContextPolicy {
            brain: "Goal + Mem + Need + Feed".to_string(),
            tentacle: "Need + Tool + Action + Tool + Action -> Feed".to_string(),
            feedback_loop: "Need -> Feed -> Feedback".to_string(),
        },
        field_pool,
        boundary,
        harness_learning: status.harness_learning,
        capabilities,
        gaps,
        agent_next,
        next,
    })
}

fn product_field_pool_report(
    report: &FieldTrajectoryReport,
    pool: Option<&FieldPoolStatusReport>,
) -> ProductFieldPoolReport {
    let fields = report
        .fields
        .iter()
        .map(|field| field.field.clone())
        .collect::<Vec<_>>();
    let completed_fields = report
        .fields
        .iter()
        .filter(|field| {
            field.mini_task_count > 0
                && field.satisfied_mini_task_count == field.mini_task_count
                && !field.needs_environment
                && !field.needs_repair
        })
        .count();
    ProductFieldPoolReport {
        policy: parallel_field_pool_policy().to_string(),
        field_count: report.field_count,
        field_slot_count: report.fields.len(),
        latest_worker_slot_count: report.latest_worker_slot_count,
        latest_activity: pool
            .map(field_pool_latest_activity_line)
            .unwrap_or_else(|| "none".to_string()),
        fields,
        completed_fields,
        active_slot_field: report.active_slot_field.clone(),
        active_slot_reason: report.active_slot_reason.clone(),
        worker_slots: parallel_worker_policy().to_string(),
        next: report.next.first().cloned().unwrap_or_default(),
    }
}

pub(crate) fn product_field_pool_line(
    report: &ProductFieldPoolReport,
    language: Language,
) -> String {
    let fields = join_or_none(&report.fields);
    let missing_required = join_or_none(&product_field_pool_missing_required_fields(report));
    match language {
        Language::En => format!(
            "{} peer slots, fields={}, missing_required={}, latest_worker_slots={}, latest_activity={}, {} completed, active_slot={}, reason={}, workers={}",
            report.field_slot_count,
            fields,
            missing_required,
            report.latest_worker_slot_count,
            report.latest_activity,
            report.completed_fields,
            option_or_none(&report.active_slot_field),
            report.active_slot_reason,
            report.worker_slots
        ),
        Language::Zh => format!(
            "{} 个并列领域槽，领域={}，缺失必需领域={}，最新 worker 执行槽数={}，最新活动={}，{} 个完成，当前活动槽={}，原因={}，worker策略={}",
            report.field_slot_count,
            fields,
            missing_required,
            report.latest_worker_slot_count,
            report.latest_activity,
            report.completed_fields,
            option_or_none(&report.active_slot_field),
            report.active_slot_reason,
            report.worker_slots
        ),
    }
}

pub(crate) fn product_field_pool_missing_required_fields(
    report: &ProductFieldPoolReport,
) -> Vec<String> {
    REQUIRED_PEER_FIELD_IDS
        .iter()
        .filter(|id| !report.fields.iter().any(|field| field == **id))
        .map(|id| (*id).to_string())
        .collect()
}

pub(crate) fn product_field_pool_ready(report: &ProductFieldPoolReport) -> bool {
    report.field_slot_count >= REQUIRED_PEER_FIELD_IDS.len()
        && product_field_pool_missing_required_fields(report).is_empty()
}

fn field_mini_task_harness_preflight_check() -> PreflightCheck {
    match check_report("field-mini-task", Some(2)) {
        Ok(report) => {
            let result = report.results.first();
            let template_ready = result.is_some_and(|result| {
                field_mini_task_template_result_ready(
                    result.status == Status::Satisfied,
                    &result.stdout,
                )
            });
            preflight_check(
                "field_mini_task_harness",
                report.passed && template_ready,
                true,
                field_mini_task_harness_evidence(&report),
                "octopus check field-mini-task 2",
            )
        }
        Err(error) => preflight_check(
            "field_mini_task_harness",
            false,
            true,
            error,
            "octopus check field-mini-task 2",
        ),
    }
}

fn field_mini_task_harness_evidence(report: &CheckReport) -> String {
    let Some(result) = report.results.first() else {
        return format!(
            "source={}, cwd={}, checks=0, template_check=missing",
            report.source_kind, report.cwd
        );
    };
    format!(
        "source={}, cwd={}, command={}, {}",
        report.source_kind,
        report.cwd,
        result.command,
        field_mini_task_template_evidence(result.status == Status::Satisfied, &result.stdout)
    )
}

fn field_mini_task_harness_product_capability() -> ProductCapability {
    let check = field_mini_task_harness_preflight_check();
    product_capability(
        "field_mini_task_harness",
        if check.status == "pass" {
            "ready"
        } else {
            "needs_harness_check"
        },
        check.evidence,
        Some("octopus check field-mini-task 2"),
    )
}

fn product_capability(
    id: impl Into<String>,
    status: impl Into<String>,
    evidence: impl Into<String>,
    command: Option<&str>,
) -> ProductCapability {
    ProductCapability {
        id: id.into(),
        status: status.into(),
        evidence: evidence.into(),
        command: command.map(str::to_string),
    }
}

fn product_gap(
    id: impl Into<String>,
    impact: impl Into<String>,
    agent_next: impl Into<String>,
) -> ProductGap {
    let id = id.into();
    let impact = impact.into();
    let agent_next = agent_next.into();
    let user_goal_hint = user_surface::product_gap_hint(&id, &impact);
    ProductGap {
        id,
        impact,
        user_goal_hint: user_goal_hint.clone(),
        agent_next,
        next: user_goal_hint,
    }
}
