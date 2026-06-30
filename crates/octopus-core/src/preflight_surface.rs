use super::*;

#[derive(Debug, serde::Serialize)]
pub(crate) struct PreflightReport {
    pub(crate) version: String,
    pub(crate) target: String,
    pub(crate) state_path: String,
    pub(crate) live: bool,
    pub(crate) release_ready: bool,
    pub(crate) summary: PreflightSummary,
    pub(crate) current_head: Option<String>,
    pub(crate) checks: Vec<PreflightCheck>,
    pub(crate) live_provider: Option<ProviderCheckReport>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct PreflightSummary {
    pub(crate) required_passed: usize,
    pub(crate) required_total: usize,
    pub(crate) optional_passed: usize,
    pub(crate) optional_total: usize,
    pub(crate) blockers: Vec<PreflightBlocker>,
    pub(crate) next_steps: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct PreflightBlocker {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) evidence: String,
    pub(crate) next: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct PreflightScriptReport {
    pub(crate) path: String,
    pub(crate) state_path: String,
    pub(crate) commands: Vec<String>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct PreflightRecordReport {
    pub(crate) path: String,
    pub(crate) state_path: String,
    pub(crate) version: String,
    pub(crate) current_head: Option<String>,
    pub(crate) commands: Vec<String>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct PreflightRecordCheckReport {
    pub(crate) path: String,
    pub(crate) current_head: Option<String>,
    pub(crate) passed: bool,
    pub(crate) checks: Vec<PreflightCheck>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct PreflightRecordAppendReport {
    pub(crate) record_path: String,
    pub(crate) log_path: String,
    pub(crate) current_head: Option<String>,
    pub(crate) appended: bool,
    pub(crate) already_present: bool,
    pub(crate) bytes: usize,
    pub(crate) check: PreflightRecordCheckReport,
    pub(crate) next: Vec<String>,
}

pub(crate) fn preflight_report(
    state: &HarnessState,
    state_path: &Path,
    live: bool,
) -> Result<PreflightReport, String> {
    let product = product_report(state, state_path)?;
    let field_mini_task_harness = product
        .capabilities
        .iter()
        .find(|capability| capability.id == "field_mini_task_harness");
    let doctor = doctor_report(state, state_path.to_path_buf())?;
    let provider = provider_status_report();
    let provider_matrix = check_provider_matrix_record(Path::new(DEFAULT_PROVIDER_MATRIX_PATH))?;
    let current_head = git_short_head();
    let benchmark_record = check_benchmark_record(
        Path::new(DEFAULT_BENCHMARK_RECORD_PATH),
        current_head.clone(),
    )?;
    let status = &doctor.status;
    let real_machine_doc = repo_root().join("docs/real-machine-test.md");
    let real_machine_log = fs::read_to_string(&real_machine_doc).unwrap_or_default();
    let real_machine_record =
        real_machine_record_status(current_head.as_deref(), &real_machine_log);
    let seed_tentacles = bootstrap_seed_tentacles();
    let missing_seed_tentacles = seed_tentacles
        .iter()
        .filter(|id| {
            !state
                .installed_tentacles
                .iter()
                .any(|tentacle| tentacle.id == **id)
        })
        .map(|id| (*id).to_string())
        .collect::<Vec<_>>();
    let provider_layers_ready = provider_coverage_ready(&provider);
    let github_grant = state.active_grant("github", "dangoZhang/Octopus").is_some();
    let gh_ready = command_ready("gh");
    let docs_ready = repo_root().join("README.md").exists()
        && repo_root().join("docs/index.html").exists()
        && repo_root().join("docs/app.html").exists()
        && repo_root().join("docs/pet.html").exists();
    let scored_feed_traces = state
        .feed_traces
        .iter()
        .filter(|trace| trace.metadata.contains_key("feedback_status"))
        .count();
    let feedback_ready = scored_feed_traces > 0;
    let desktop_pet_source = desktop_pet_source_check();
    let mut live_provider = None;
    let mut live_provider_error = None;
    if live {
        match provider_check_report(
            &doctor.llm.config_prefix,
            "Reply with 'octopus provider live'.",
        ) {
            Ok(report) => live_provider = Some(report),
            Err(error) => live_provider_error = Some(error),
        }
    }

    let mut checks = vec![
        preflight_check(
            "state",
            state_path.exists(),
            true,
            format!(
                "{} ({})",
                state_path.to_string_lossy(),
                if state_path.exists() {
                    "exists"
                } else {
                    "missing"
                }
            ),
            "octopus bootstrap",
        ),
        preflight_check(
            "seed_tentacles",
            missing_seed_tentacles.is_empty(),
            true,
            if missing_seed_tentacles.is_empty() {
                format!("{} seed tentacles installed", seed_tentacles.len())
            } else {
                format!("missing {}", missing_seed_tentacles.join(", "))
            },
            "octopus bootstrap",
        ),
        preflight_check(
            "manifest_integrity",
            doctor.broken_manifests.is_empty() && doctor.manifest_count > 0,
            true,
            if doctor.broken_manifests.is_empty() {
                format!("{} manifests clean", doctor.manifest_count)
            } else {
                doctor.broken_manifests.join("; ")
            },
            "octopus manifests",
        ),
        preflight_check(
            "profile_registry",
            doctor.profile_registry.ok,
            true,
            format!(
                "source={}, path={}, profiles={}, error={}",
                doctor.profile_registry.source,
                doctor.profile_registry.path,
                doctor.profile_registry.profile_count,
                doctor.profile_registry.error.as_deref().unwrap_or("none")
            ),
            "octopus bootstrap",
        ),
        preflight_check(
            "clean_brain_boundary",
            product.context.brain == CLEAN_BRAIN_CONTEXT_POLICY
                && product.context.tentacle == TENTACLE_CONTEXT_POLICY,
            true,
            format!(
                "brain={}, tentacle={}",
                product.context.brain, product.context.tentacle
            ),
            "octopus context observe .",
        ),
        preflight_check(
            "core_harness_boundary",
            product.boundary.ok,
            true,
            format!(
                "stable_rust={}, product_app={}, editable_harness={}, warnings={}",
                product.boundary.stable_rust.len(),
                product.boundary.product_app.len(),
                product.boundary.editable_harness.len(),
                if product.boundary.warnings.is_empty() {
                    "none".to_string()
                } else {
                    product.boundary.warnings.join("; ")
                }
            ),
            "octopus report",
        ),
        preflight_check(
            "field_parallel_pool",
            product_field_pool_ready(&product.field_pool),
            true,
            product_field_pool_line(&product.field_pool, Language::En),
            "octopus fields summary",
        ),
        preflight_check(
            "field_mini_task_harness",
            field_mini_task_harness.is_some_and(|capability| capability.status == "ready"),
            true,
            field_mini_task_harness
                .map(|capability| capability.evidence.clone())
                .unwrap_or_else(|| "missing field-mini-task harness capability".to_string()),
            "octopus check field-mini-task 2",
        ),
        app_bridge::goal_surface_preflight_check(state_path),
        app_bridge::local_app_run_status_check(state_path, current_head.as_deref()),
        preflight_check(
            "docs_and_pet",
            docs_ready,
            true,
            "README, docs homepage, app, and pixel pet files",
            "octopus start",
        ),
        preflight_status_check(
            "desktop_pet_source",
            desktop_pet_source.status,
            cfg!(target_os = "macos"),
            desktop_pet_source.evidence,
            desktop_pet_source.next,
        ),
        download_artifacts_preflight_check(),
        preflight_check(
            "provider_layers",
            provider_layers_ready,
            true,
            provider
                .coverage
                .iter()
                .map(|coverage| {
                    format!(
                        "{} ready={} prefixes={} blockers={}",
                        coverage.id,
                        coverage.ready,
                        join_or_none(&coverage.prefixes),
                        join_or_none(&coverage.blockers)
                    )
                })
                .collect::<Vec<_>>()
                .join("; "),
            "octopus provider save openai; source .octopus/llm.env; octopus provider status",
        ),
        preflight_check(
            "provider_matrix_record",
            provider_matrix.passed,
            true,
            provider_matrix_check_evidence(&provider_matrix),
            "octopus provider matrix; octopus provider matrix run; octopus provider matrix check",
        ),
        preflight_check(
            "benchmark_evidence",
            benchmark_record.passed,
            true,
            benchmark_record_evidence(&benchmark_record),
            "octopus benchmark record; run cases; octopus benchmark check",
        ),
        preflight_status_check(
            "live_provider",
            if live {
                if live_provider.is_some() {
                    "pass"
                } else {
                    "fail"
                }
            } else {
                "skipped"
            },
            true,
            live_provider
                .as_ref()
                .map(|report| report.content.clone())
                .or(live_provider_error)
                .unwrap_or_else(|| "not run; pass --live to call the provider".to_string()),
            format!(
                "octopus --state {} preflight --live",
                shell_arg(&state_path.to_string_lossy())
            ),
        ),
        preflight_check(
            "feedback_data",
            feedback_ready,
            true,
            format!(
                "scored_traces={}, traces={}, checks={}, evolution_scores={}",
                scored_feed_traces,
                status.feed_trace_count,
                status.check_history_count,
                state.evolution_outcomes.len()
            ),
            format!(
                "octopus --state {} first-run \"collect local feedback evidence\"",
                shell_arg(&state_path.to_string_lossy())
            ),
        ),
        preflight_check(
            "github_pr_path",
            github_grant && gh_ready,
            true,
            format!("gh_cli={}, oauth_grant={}", gh_ready, github_grant),
            "gh auth login; octopus oauth github dangoZhang/Octopus repo workflow",
        ),
        preflight_check(
            "real_machine_record",
            real_machine_record.passed,
            true,
            real_machine_record.evidence,
            "octopus preflight record",
        ),
        preflight_check(
            "desktop_adapters",
            state
                .installed_tentacles
                .iter()
                .any(|tentacle| tentacle.id == "computer-use-agent"),
            false,
            "computer-use-agent installed for desktop/browser probes",
            "octopus install computer-use-agent",
        ),
        preflight_check(
            "harness_repair",
            state
                .installed_tentacles
                .iter()
                .any(|tentacle| tentacle.id == "harness-repair-agent"),
            false,
            "harness-repair-agent installed for tool-side diagnosis",
            "octopus install harness-repair-agent",
        ),
    ];
    checks.sort_by(|left, right| left.id.cmp(&right.id));
    let release_ready = checks
        .iter()
        .filter(|check| check.required)
        .all(|check| check.status == "pass");
    let summary = preflight_summary(&checks);
    let mut next = checks
        .iter()
        .filter(|check| check.status != "pass")
        .map(|check| check.next.clone())
        .collect::<Vec<_>>();
    next.push("octopus preflight script".to_string());
    next.push("octopus preflight record".to_string());
    next.push("octopus report".to_string());
    next.sort();
    next.dedup();

    Ok(PreflightReport {
        version: env!("CARGO_PKG_VERSION").to_string(),
        target: "0.2.0".to_string(),
        state_path: state_path.to_string_lossy().to_string(),
        live,
        release_ready,
        summary,
        current_head,
        checks,
        live_provider,
        next,
    })
}

fn preflight_summary(checks: &[PreflightCheck]) -> PreflightSummary {
    let required_total = checks.iter().filter(|check| check.required).count();
    let required_passed = checks
        .iter()
        .filter(|check| check.required && check.status == "pass")
        .count();
    let optional_total = checks.iter().filter(|check| !check.required).count();
    let optional_passed = checks
        .iter()
        .filter(|check| !check.required && check.status == "pass")
        .count();
    let blockers = checks
        .iter()
        .filter(|check| check.required && check.status != "pass")
        .map(|check| PreflightBlocker {
            id: check.id.clone(),
            status: check.status.clone(),
            evidence: check.evidence.clone(),
            next: check.next.clone(),
        })
        .collect::<Vec<_>>();
    let next_steps = blockers
        .iter()
        .map(|blocker| blocker.next.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    PreflightSummary {
        required_passed,
        required_total,
        optional_passed,
        optional_total,
        blockers,
        next_steps,
    }
}

pub(crate) fn write_preflight_script(
    state_path: &Path,
    output_path: &Path,
) -> Result<PreflightScriptReport, String> {
    if let Some(parent) = output_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let commands = preflight_script_commands();
    let state_default = shell_arg(&state_path.to_string_lossy());
    let content = format!(
        r#"#!/bin/sh
set -eu

STATE=${{OCTOPUS_PREFLIGHT_STATE:-{state_default}}}
ROOT=${{OCTOPUS_PREFLIGHT_ROOT:-$(pwd)}}

cd "$ROOT"

{commands}

if [ "${{OCTOPUS_PREFLIGHT_LIVE:-0}}" = "1" ]; then
  octopus provider status
  octopus provider check "${{OCTOPUS_LLM_PREFIX:-OCTOPUS_LLM}}"
  octopus --state "$STATE" preflight --live
else
  octopus --state "$STATE" preflight
fi

if [ "${{OCTOPUS_PREFLIGHT_PR_DRY_RUN:-0}}" = "1" ]; then
  gh auth status
  octopus --state "$STATE" self-iterate dangoZhang/Octopus
  OCTOPUS_PR_DRY_RUN=1 octopus --state "$STATE" self-iterate pr dangoZhang/Octopus "preflight dry run"
fi
"#,
        commands = commands.join("\n")
    );
    fs::write(output_path, content).map_err(|error| error.to_string())?;
    Ok(PreflightScriptReport {
        path: output_path.to_string_lossy().to_string(),
        state_path: state_path.to_string_lossy().to_string(),
        commands,
        next: vec![
            format!("sh {}", shell_arg(&output_path.to_string_lossy())),
            format!(
                "OCTOPUS_PREFLIGHT_LIVE=1 sh {}",
                shell_arg(&output_path.to_string_lossy())
            ),
            format!(
                "OCTOPUS_PREFLIGHT_PR_DRY_RUN=1 sh {}",
                shell_arg(&output_path.to_string_lossy())
            ),
        ],
    })
}

pub(crate) fn write_preflight_record(
    state_path: &Path,
    output_path: &Path,
) -> Result<PreflightRecordReport, String> {
    if let Some(parent) = output_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let current_head = git_short_head();
    let version = env!("CARGO_PKG_VERSION").to_string();
    let commands = preflight_record_commands(state_path, current_head.as_deref(), shell_arg);
    let head_for_record = current_head.as_deref().unwrap_or("unknown");
    let content = format!(
        r#"# Real-Machine Record

Append the completed result to `docs/real-machine-test.md` after the run.

## Machine

- Date:
- Tester:
- Machine:
- OS:
- Shell:
- Rust:
- Python:
- Git commit: `{head}`
- Package version: `{version}`
- State: `{state}`

## Commands

```bash
{commands}
```

## Results

- Install and doctor:
- Download artifacts:
- Core loop:
- Product bridge:

Field pool result should include the required v0.2 peer fields, `missing_required=none`, latest worker slots, `latest_activity` with `worker=`, and `parallel_run` with `requested_worker_slots=`, `active_worker_slots=`, and `candidate_pool=` containing `math` and `search` from `octopus status` or `octopus report`.
- Field pool:
Field mini task harness result should include `checked_count=...`, `executed_count=...`, `satisfied_count=...`, `partial_count=...`, `missing_count=0`, `invalid_count=0`, and `status=ok` from `octopus check field-mini-task 2`.
- Field mini task harness:
- Desktop pet source:
- Start/app:
- Live provider:
- Benchmark evidence:
- PR dry run:

## Decision

- Pass or fail:
- Follow-up:
"#,
        head = head_for_record,
        version = version,
        state = state_path.to_string_lossy(),
        commands = commands.join("\n")
    );
    fs::write(output_path, content).map_err(|error| error.to_string())?;
    Ok(PreflightRecordReport {
        path: output_path.to_string_lossy().to_string(),
        state_path: state_path.to_string_lossy().to_string(),
        version,
        current_head,
        commands,
        next: vec![
            format!("review {}", shell_arg(&output_path.to_string_lossy())),
            "run the commands on a clean real machine".to_string(),
            "append the completed result to docs/real-machine-test.md".to_string(),
            "octopus preflight".to_string(),
        ],
    })
}

pub(crate) fn check_preflight_record(path: &Path) -> Result<PreflightRecordCheckReport, String> {
    let current_head = git_short_head();
    let content = fs::read_to_string(path).unwrap_or_default();
    let file_exists = path.exists();
    let machine_fields = ["Date", "Tester", "Machine", "OS", "Shell", "Rust", "Python"];
    let missing_machine = machine_fields
        .iter()
        .filter(|field| record_field_value(&content, field).is_none())
        .map(|field| (*field).to_string())
        .collect::<Vec<_>>();
    let result_fields: &[&[&str]] = &[
        &["Install and doctor"],
        &["Download artifacts"],
        &["Core loop"],
        &["Product bridge"],
        &["Field pool"],
        &["Field mini task harness"],
        &["Desktop pet source"],
        &["Start/app", "Bridge/app"],
        &["Live provider"],
        &["Benchmark evidence"],
        &["PR dry run"],
    ];
    let missing_results = result_fields
        .iter()
        .filter(|fields| {
            fields
                .iter()
                .all(|field| record_field_value(&content, field).is_none())
        })
        .map(|fields| fields[0].to_string())
        .collect::<Vec<_>>();
    let pass_decision_value = record_field_value(&content, "Pass or fail");
    let pass_decision = pass_decision_value
        .as_deref()
        .is_some_and(record_pass_decision_ready);
    let field_pool_value = record_field_value(&content, "Field pool");
    let field_pool_worker_ready = field_pool_value
        .as_deref()
        .is_some_and(|value| value.contains("latest_activity") && value.contains("worker="));
    let field_pool_named_fields_ready = field_pool_value.as_deref().is_some_and(|value| {
        value.contains("missing_required=none")
            && REQUIRED_PEER_FIELD_IDS
                .iter()
                .all(|field| value.contains(field))
    });
    let field_pool_parallel_run_ready = field_pool_value
        .as_deref()
        .is_some_and(field_pool_parallel_run_record_ready);
    let field_mini_task_value = record_field_value(&content, "Field mini task harness");
    let field_mini_task_record_ready = field_mini_task_value.as_deref().is_some_and(|value| {
        value.contains("checked_count=")
            && value.contains("executed_count=")
            && value.contains("satisfied_count=")
            && value.contains("partial_count=")
            && value.contains("missing_count=0")
            && value.contains("invalid_count=0")
            && value.contains("status=ok")
    });
    let head_ready = current_head
        .as_ref()
        .is_some_and(|head| content.contains(&format!("`{head}`")) || content.contains(head));
    let mut checks = vec![
        preflight_check(
            "record_file",
            file_exists,
            true,
            path.to_string_lossy(),
            "octopus preflight record",
        ),
        preflight_check(
            "current_head",
            head_ready,
            true,
            current_head
                .as_ref()
                .map(|head| format!("record must include current head {head}"))
                .unwrap_or_else(|| "git head unavailable".to_string()),
            "rerun octopus preflight record on the target head",
        ),
        preflight_check(
            "machine_fields",
            missing_machine.is_empty(),
            true,
            if missing_machine.is_empty() {
                "machine fields filled".to_string()
            } else {
                format!("missing {}", missing_machine.join(", "))
            },
            "fill Date, Tester, Machine, OS, Shell, Rust, and Python",
        ),
        preflight_check(
            "result_fields",
            missing_results.is_empty(),
            true,
            if missing_results.is_empty() {
                "result fields filled".to_string()
            } else {
                format!("missing {}", missing_results.join(", "))
            },
            "fill Install, Download artifacts, Core loop, Product bridge, Field pool, Field mini task harness, Desktop pet source, Start/app, Live provider, Benchmark evidence, and PR dry run results",
        ),
        preflight_check(
            "field_pool_worker_identity",
            field_pool_worker_ready,
            true,
            field_pool_value.unwrap_or_else(|| "missing Field pool result".to_string()),
            "fill Field pool with latest_activity=... worker=...",
        ),
        preflight_check(
            "field_pool_named_fields",
            field_pool_named_fields_ready,
            true,
            record_field_value(&content, "Field pool")
                .unwrap_or_else(|| "missing Field pool result".to_string()),
            "fill Field pool with fields=math,...,robotics missing_required=none",
        ),
        preflight_check(
            "field_pool_parallel_run",
            field_pool_parallel_run_ready,
            true,
            record_field_value(&content, "Field pool")
                .unwrap_or_else(|| "missing Field pool result".to_string()),
            "fill Field pool with parallel_run requested_worker_slots=... active_worker_slots=... candidate_pool=math,search",
        ),
        preflight_check(
            "field_mini_task_harness_record",
            field_mini_task_record_ready,
            true,
            field_mini_task_value
                .unwrap_or_else(|| "missing Field mini task harness result".to_string()),
            "fill Field mini task harness with checked_count=... executed_count=... satisfied_count=... partial_count=... missing_count=0 invalid_count=0 status=ok",
        ),
        preflight_check(
            "pass_decision",
            pass_decision,
            true,
            pass_decision_value.unwrap_or_else(|| "missing pass/fail decision".to_string()),
            "write Pass or fail: pass after real-machine checks pass",
        ),
        preflight_check(
            "command_coverage",
            content.contains("first-run")
                && content.contains("download")
                && content.contains("download.json")
                && content.contains("install.sh")
                && content.contains("bridge_goal_surface")
                && content.contains("desktop_pet_source")
                && content.contains("provider matrix")
                && content.contains("provider matrix run")
                && content.contains("provider matrix check")
                && content.contains("fields summary")
                && content.contains("evolve parallel --workers 2")
                && content.contains("preflight math and search field adaptation")
                && content.contains("check field-mini-task 2")
                && content.contains("benchmark record")
                && content.contains("benchmark check")
                && content.contains("preflight --live")
                && content.contains("start --check")
                && content.contains("self-iterate pr"),
            true,
            "record should include first-run, download artifacts, bridge_goal_surface, desktop_pet_source, provider matrix run/check, the math/search field evolution run, field pool summary, field-mini-task harness check, live provider, benchmark record/check, start --check, and PR dry-run commands",
            "regenerate with octopus preflight record",
        ),
    ];
    checks.sort_by(|left, right| left.id.cmp(&right.id));
    let passed = checks
        .iter()
        .filter(|check| check.required)
        .all(|check| check.status == "pass");
    let next = if passed {
        vec![
            format!(
                "octopus preflight record append {}",
                shell_arg(&path.to_string_lossy())
            ),
            "octopus preflight".to_string(),
        ]
    } else {
        checks
            .iter()
            .filter(|check| check.status != "pass")
            .map(|check| check.next.clone())
            .collect::<Vec<_>>()
    };
    Ok(PreflightRecordCheckReport {
        path: path.to_string_lossy().to_string(),
        current_head,
        passed,
        checks,
        next,
    })
}

pub(crate) fn append_preflight_record(
    record_path: &Path,
    log_path: &Path,
) -> Result<PreflightRecordAppendReport, String> {
    let check = check_preflight_record(record_path)?;
    if !check.passed {
        let missing = check
            .checks
            .iter()
            .filter(|item| item.status != "pass")
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "preflight record is not ready: {}; run octopus preflight record check",
            if missing.is_empty() {
                "unknown checks failed".to_string()
            } else {
                missing
            }
        ));
    }
    let record = fs::read_to_string(record_path).map_err(|error| error.to_string())?;
    let record_body = record.trim();
    if record_body.is_empty() {
        return Err("preflight record is empty".to_string());
    }
    if let Some(parent) = log_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut log = fs::read_to_string(log_path).unwrap_or_else(|_| {
        "# Real-Machine Test Gate\n\nRequired before `0.1.0` and every later version tag.\n"
            .to_string()
    });
    let already_present = log.contains(record_body);
    let mut appended = false;
    if !already_present {
        if !log.ends_with('\n') {
            log.push('\n');
        }
        log.push_str("\n---\n\n## Appended Real-Machine Record\n\n");
        log.push_str(record_body);
        log.push('\n');
        fs::write(log_path, &log).map_err(|error| error.to_string())?;
        appended = true;
    }
    Ok(PreflightRecordAppendReport {
        record_path: record_path.to_string_lossy().to_string(),
        log_path: log_path.to_string_lossy().to_string(),
        current_head: check.current_head.clone(),
        appended,
        already_present,
        bytes: record_body.len(),
        check,
        next: vec![
            "octopus preflight".to_string(),
            format!("git add {}", shell_arg(&log_path.to_string_lossy())),
            "git commit -m \"Record real-machine preflight\"".to_string(),
        ],
    })
}

pub(crate) fn field_pool_parallel_run_record_ready(value: &str) -> bool {
    value.contains("parallel_run")
        && value.contains("requested_worker_slots=")
        && value.contains("active_worker_slots=")
        && field_pool_record_candidate_contains(value, "math")
        && field_pool_record_candidate_contains(value, "search")
}

fn field_pool_record_candidate_contains(value: &str, field: &str) -> bool {
    value
        .split("candidate_pool=")
        .skip(1)
        .any(|segment| field_pool_candidate_segment_contains(segment, field))
}

fn field_pool_candidate_segment_contains(segment: &str, field: &str) -> bool {
    let segment = segment
        .split(", workers=")
        .next()
        .unwrap_or(segment)
        .split(" workers=")
        .next()
        .unwrap_or(segment)
        .split(" worker_policy=")
        .next()
        .unwrap_or(segment);
    segment.split([',', ' ', '\t', ';', '[', ']']).any(|token| {
        token.trim_matches(|character: char| character == ':' || character == '"') == field
    })
}

pub(crate) fn git_short_head() -> Option<String> {
    git_short_rev("HEAD")
}

pub(crate) fn real_machine_record_status(
    current_head: Option<&str>,
    log: &str,
) -> RealMachineRecordStatus {
    let parent_head = git_short_rev("HEAD^");
    let changed_files = git_changed_files("HEAD^", "HEAD");
    real_machine_record_status_from_parts(
        current_head,
        parent_head.as_deref(),
        changed_files.as_deref(),
        log,
    )
}

fn git_short_rev(rev: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short", rev])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_changed_files(base: &str, head: &str) -> Option<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--name-only", base, head])
        .output()
        .ok()?;
    output.status.success().then(|| {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect()
    })
}

pub(crate) fn print_preflight_report(report: &PreflightReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus preflight");
            println!("target: {}", report.target);
            println!("version: {}", report.version);
            println!("state: {}", report.state_path);
            println!("live: {}", report.live);
            println!("release_ready: {}", report.release_ready);
            println!(
                "summary: required {}/{}, optional {}/{}",
                report.summary.required_passed,
                report.summary.required_total,
                report.summary.optional_passed,
                report.summary.optional_total
            );
            if !report.summary.blockers.is_empty() {
                println!("blockers:");
                for blocker in &report.summary.blockers {
                    println!(
                        "- {} [{}]: {}; next={}",
                        blocker.id, blocker.status, blocker.evidence, blocker.next
                    );
                }
            }
            if !report.summary.next_steps.is_empty() {
                println!("release next steps:");
                for step in &report.summary.next_steps {
                    println!("- {step}");
                }
            }
            println!(
                "head: {}",
                report.current_head.as_deref().unwrap_or("unknown")
            );
            println!("checks:");
            for check in &report.checks {
                println!(
                    "- {} [{} required={}]: {}; next={}",
                    check.id, check.status, check.required, check.evidence, check.next
                );
            }
            if let Some(provider) = &report.live_provider {
                println!("live_provider_reply: {}", provider.content);
            }
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼发布前检查");
            println!("目标版本: {}", report.target);
            println!("版本: {}", report.version);
            println!("状态文件: {}", report.state_path);
            println!("Live检查: {}", report.live);
            println!("可发布: {}", report.release_ready);
            println!(
                "摘要: 必需 {}/{}, 可选 {}/{}",
                report.summary.required_passed,
                report.summary.required_total,
                report.summary.optional_passed,
                report.summary.optional_total
            );
            if !report.summary.blockers.is_empty() {
                println!("阻塞项:");
                for blocker in &report.summary.blockers {
                    println!(
                        "- {} [{}]: {}; 下一步={}",
                        blocker.id, blocker.status, blocker.evidence, blocker.next
                    );
                }
            }
            if !report.summary.next_steps.is_empty() {
                println!("发布下一步:");
                for step in &report.summary.next_steps {
                    println!("- {step}");
                }
            }
            println!(
                "当前提交: {}",
                report.current_head.as_deref().unwrap_or("未知")
            );
            println!("检查项:");
            for check in &report.checks {
                println!(
                    "- {} [{} 必需={}]: {}; 下一步={}",
                    check.id, check.status, check.required, check.evidence, check.next
                );
            }
            if let Some(provider) = &report.live_provider {
                println!("Live Provider 回复: {}", provider.content);
            }
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

pub(crate) fn print_preflight_script_report(report: &PreflightScriptReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus preflight script");
            println!("path: {}", report.path);
            println!("state: {}", report.state_path);
            println!("commands: {}", report.commands.len());
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼发布前检查脚本");
            println!("路径: {}", report.path);
            println!("状态文件: {}", report.state_path);
            println!("命令数: {}", report.commands.len());
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

pub(crate) fn print_preflight_record_report(report: &PreflightRecordReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus preflight record");
            println!("path: {}", report.path);
            println!("version: {}", report.version);
            println!(
                "head: {}",
                report.current_head.as_deref().unwrap_or("unknown")
            );
            println!("commands: {}", report.commands.len());
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼实机记录模板");
            println!("路径: {}", report.path);
            println!("版本: {}", report.version);
            println!(
                "当前提交: {}",
                report.current_head.as_deref().unwrap_or("未知")
            );
            println!("命令数: {}", report.commands.len());
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

pub(crate) fn print_preflight_record_check(
    report: &PreflightRecordCheckReport,
    language: Language,
) {
    match language {
        Language::En => {
            println!("Octopus preflight record check");
            println!("path: {}", report.path);
            println!(
                "head: {}",
                report.current_head.as_deref().unwrap_or("unknown")
            );
            println!("passed: {}", report.passed);
            for check in &report.checks {
                println!("- {}: {} ({})", check.id, check.status, check.evidence);
            }
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼实机记录检查");
            println!("路径: {}", report.path);
            println!(
                "当前提交: {}",
                report.current_head.as_deref().unwrap_or("未知")
            );
            println!("通过: {}", report.passed);
            for check in &report.checks {
                println!("- {}: {} ({})", check.id, check.status, check.evidence);
            }
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

pub(crate) fn print_preflight_record_append(
    report: &PreflightRecordAppendReport,
    language: Language,
) {
    match language {
        Language::En => {
            println!("Octopus preflight record append");
            println!("record: {}", report.record_path);
            println!("log: {}", report.log_path);
            println!(
                "head: {}",
                report.current_head.as_deref().unwrap_or("unknown")
            );
            println!("appended: {}", report.appended);
            println!("already_present: {}", report.already_present);
            println!("bytes: {}", report.bytes);
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼实机记录追加");
            println!("记录: {}", report.record_path);
            println!("日志: {}", report.log_path);
            println!(
                "当前提交: {}",
                report.current_head.as_deref().unwrap_or("未知")
            );
            println!("已追加: {}", report.appended);
            println!("已存在: {}", report.already_present);
            println!("字节: {}", report.bytes);
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}
