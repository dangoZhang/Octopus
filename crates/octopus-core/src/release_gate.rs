use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct PreflightCheck {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) required: bool,
    pub(crate) evidence: String,
    pub(crate) next: String,
}

#[derive(Debug)]
pub(crate) struct RealMachineRecordStatus {
    pub(crate) passed: bool,
    pub(crate) evidence: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct BenchmarkCase {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) suite: String,
    pub(crate) target: String,
    pub(crate) command_hint: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct BenchmarkRecordReport {
    pub(crate) path: String,
    pub(crate) version: String,
    pub(crate) current_head: Option<String>,
    pub(crate) cases: Vec<BenchmarkCase>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct BenchmarkRecordCheckReport {
    pub(crate) path: String,
    pub(crate) current_head: Option<String>,
    pub(crate) passed: bool,
    pub(crate) checks: Vec<PreflightCheck>,
    pub(crate) next: Vec<String>,
}

pub(crate) fn preflight_check(
    id: impl Into<String>,
    passed: bool,
    required: bool,
    evidence: impl Into<String>,
    next: impl Into<String>,
) -> PreflightCheck {
    preflight_status_check(
        id,
        if passed {
            "pass"
        } else if required {
            "fail"
        } else {
            "warn"
        },
        required,
        evidence,
        next,
    )
}

pub(crate) fn preflight_status_check(
    id: impl Into<String>,
    status: impl Into<String>,
    required: bool,
    evidence: impl Into<String>,
    next: impl Into<String>,
) -> PreflightCheck {
    PreflightCheck {
        id: id.into(),
        status: status.into(),
        required,
        evidence: evidence.into(),
        next: next.into(),
    }
}

pub(crate) fn record_field_value(content: &str, label: &str) -> Option<String> {
    let prefix = format!("- {label}:");
    content.lines().find_map(|line| {
        let value = line.trim().strip_prefix(&prefix)?.trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

pub(crate) fn record_pass_decision_ready(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    normalized == "pass"
        || normalized == "passed"
        || normalized == "ok"
        || normalized.starts_with("pass ")
        || normalized.starts_with("pass:")
}

pub(crate) fn preflight_script_commands() -> Vec<String> {
    [
        "octopus --version",
        "octopus --state \"$STATE\" first-run \"preflight local evidence\"",
        "octopus --state \"$STATE\" doctor",
        "octopus --state \"$STATE\" context observe .",
        "octopus --state \"$STATE\" think swe-agent observe README.md",
        "octopus --state \"$STATE\" traces",
        "octopus --state \"$STATE\" repair .",
        "octopus --state \"$STATE\" needs",
        "octopus --state \"$STATE\" check swe-agent",
        "octopus --state \"$STATE\" routes observe README.md",
        "octopus --state \"$STATE\" beat 200",
        "octopus --state \"$STATE\" pet harness",
        "octopus --state \"$STATE\" report",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

pub(crate) fn real_machine_record_status_from_parts(
    current_head: Option<&str>,
    parent_head: Option<&str>,
    changed_files: Option<&[String]>,
    log: &str,
) -> RealMachineRecordStatus {
    let Some(head) = current_head else {
        return RealMachineRecordStatus {
            passed: false,
            evidence: "git head unavailable".to_string(),
        };
    };

    if log.contains(head) {
        return RealMachineRecordStatus {
            passed: true,
            evidence: format!("head {head} recorded=true"),
        };
    }

    let parent_recorded = parent_head.is_some_and(|parent| log.contains(parent));
    let docs_only = changed_files.is_some_and(|files| {
        !files.is_empty()
            && files
                .iter()
                .all(|path| real_machine_record_doc_path(path.as_str()))
    });

    if parent_recorded && docs_only {
        return RealMachineRecordStatus {
            passed: true,
            evidence: format!(
                "head {head} recorded=false; parent {} recorded=true; docs_only=true",
                parent_head.unwrap_or("unknown")
            ),
        };
    }

    RealMachineRecordStatus {
        passed: false,
        evidence: format!(
            "head {head} recorded=false; parent {} recorded={parent_recorded}; docs_only={docs_only}",
            parent_head.unwrap_or("unknown")
        ),
    }
}

pub(crate) fn preflight_record_commands(
    state_path: &Path,
    current_head: Option<&str>,
    shell_arg: impl Fn(&str) -> String,
) -> Vec<String> {
    let rev = current_head.unwrap_or("HEAD");
    let state_default = shell_arg(&state_path.to_string_lossy());
    let mut commands = vec![
        "tmp=$(mktemp -d)".to_string(),
        format!(
            "cargo install --git https://github.com/dangoZhang/Octopus --rev {} octopus-core --locked --bin octopus --force --root \"$tmp/root\"",
            shell_arg(rev)
        ),
        "OCTOPUS=\"$tmp/root/bin/octopus\"".to_string(),
        format!("STATE=${{OCTOPUS_PREFLIGHT_STATE:-{state_default}}}"),
    ];
    commands.extend(preflight_script_commands().into_iter().map(|command| {
        command
            .replace("octopus --version", "\"$OCTOPUS\" --version")
            .replace(
                "octopus --state \"$STATE\"",
                "\"$OCTOPUS\" --state \"$STATE\"",
            )
    }));
    commands.extend([
        "\"$OCTOPUS\" download".to_string(),
        "\"$OCTOPUS\" --json download > \"$tmp/download.json\"".to_string(),
        "grep '\"install_script_url\"' \"$tmp/download.json\"".to_string(),
        "\"$OCTOPUS\" --state \"$STATE\" preflight".to_string(),
        "\"$OCTOPUS\" --state \"$STATE\" preflight | grep bridge_goal_surface".to_string(),
        "\"$OCTOPUS\" provider status".to_string(),
        "\"$OCTOPUS\" provider matrix \"$tmp/provider-matrix.md\"".to_string(),
        "\"$OCTOPUS\" --state \"$STATE\" provider matrix run \"$tmp/provider-matrix.md\""
            .to_string(),
        "\"$OCTOPUS\" provider matrix check \"$tmp/provider-matrix.md\"".to_string(),
        "\"$OCTOPUS\" provider check \"${OCTOPUS_LLM_PREFIX:-OCTOPUS_LLM}\"".to_string(),
        "\"$OCTOPUS\" --state \"$STATE\" preflight --live".to_string(),
        "\"$OCTOPUS\" benchmark record".to_string(),
        "# Fill .octopus/benchmark-evidence.md with real SWE/Claw/Wild case ids, commands, pass results, output summaries, and artifact paths.".to_string(),
        "\"$OCTOPUS\" benchmark check".to_string(),
        "\"$OCTOPUS\" --state \"$STATE\" start --check 127.0.0.1:18765".to_string(),
        "\"$OCTOPUS\" start 127.0.0.1:18765 &".to_string(),
        "START_PID=$!".to_string(),
        "trap 'kill \"$START_PID\" 2>/dev/null || true' EXIT".to_string(),
        "sleep 1".to_string(),
        "curl -fsS http://127.0.0.1:18765/app.html >/dev/null".to_string(),
        "curl -fsS 'http://127.0.0.1:18765/pet.html?state=harness' >/dev/null".to_string(),
        "curl -fsS http://127.0.0.1:18765/download.json | grep '\"cargo_package\"'"
            .to_string(),
        "curl -fsS http://127.0.0.1:18765/install.sh | grep 'cargo install'"
            .to_string(),
        "curl -fsS -X POST http://127.0.0.1:18765/api/run -H 'content-type: application/json' --data-binary \"{\\\"args\\\":[\\\"--state\\\",\\\"$STATE\\\",\\\"--json\\\",\\\"doctor\\\"]}\"".to_string(),
        "kill \"$START_PID\"".to_string(),
        "\"$OCTOPUS\" --state \"$STATE\" self-iterate dangoZhang/Octopus".to_string(),
        "OCTOPUS_PR_DRY_RUN=1 \"$OCTOPUS\" --state \"$STATE\" self-iterate pr dangoZhang/Octopus \"preflight dry run\"".to_string(),
    ]);
    commands
}

fn real_machine_record_doc_path(path: &str) -> bool {
    matches!(
        path,
        "docs/real-machine-test.md" | "docs/product-gap.md" | "docs/version-plan.md"
    )
}

pub(crate) fn benchmark_cases() -> Vec<BenchmarkCase> {
    [
        (
            "swe_bench_verify_mini_1",
            "SWE-Bench verify mini 1",
            "SWE-Bench verify mini",
            "first task",
            "run the first SWE-Bench verify mini case through Octopus or the chosen harness",
        ),
        (
            "swe_bench_verify_mini_2",
            "SWE-Bench verify mini 2",
            "SWE-Bench verify mini",
            "second task",
            "run the second SWE-Bench verify mini case through Octopus or the chosen harness",
        ),
        (
            "swe_bench_verify_mini_3",
            "SWE-Bench verify mini 3",
            "SWE-Bench verify mini",
            "third task",
            "run the third SWE-Bench verify mini case through Octopus or the chosen harness",
        ),
        (
            "claw_swe_bench_simplest",
            "Claw-SWE-Bench simplest",
            "Claw-SWE-Bench",
            "simplest task",
            "run the simplest Claw-SWE-Bench case through Octopus or the chosen harness",
        ),
        (
            "wild_claw_bench_simplest",
            "Wild-Claw-Bench simplest",
            "Wild-Claw-Bench",
            "simplest task",
            "run the simplest Wild-Claw-Bench case through Octopus or the chosen harness",
        ),
    ]
    .into_iter()
    .map(|(id, title, suite, target, command_hint)| BenchmarkCase {
        id: id.to_string(),
        title: title.to_string(),
        suite: suite.to_string(),
        target: target.to_string(),
        command_hint: command_hint.to_string(),
    })
    .collect()
}

pub(crate) fn write_benchmark_record(
    output_path: &Path,
    version: impl Into<String>,
    current_head: Option<String>,
    shell_arg: impl Fn(&str) -> String,
) -> Result<BenchmarkRecordReport, String> {
    if let Some(parent) = output_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let version = version.into();
    let cases = benchmark_cases();
    let head_for_record = current_head.as_deref().unwrap_or("unknown");
    let mut content = format!(
        "# Benchmark Evidence\n\n\
Record the smallest SWE/Claw/Wild evidence before `0.1.0`.\n\n\
## Machine\n\n\
- Date:\n\
- Tester:\n\
- Machine:\n\
- OS:\n\
- Git commit: `{head}`\n\
- Package version: `{version}`\n\n\
## Cases\n\n",
        head = head_for_record,
        version = version
    );
    for case in &cases {
        content.push_str(&format!(
            "### {title}\n\n\
- Id: {id}\n\
- Suite: {suite}\n\
- Target: {target}\n\
- Case id:\n\
- Command:\n\
- Pass or fail:\n\
- Output summary:\n\
- Artifact path:\n\
- Notes: {command_hint}\n\n",
            title = case.title,
            id = case.id,
            suite = case.suite,
            target = case.target,
            command_hint = case.command_hint
        ));
    }
    fs::write(output_path, content).map_err(|error| error.to_string())?;
    Ok(BenchmarkRecordReport {
        path: output_path.to_string_lossy().to_string(),
        version,
        current_head,
        cases,
        next: vec![
            "run each listed benchmark case".to_string(),
            format!(
                "octopus benchmark check {}",
                shell_arg(&output_path.to_string_lossy())
            ),
            "octopus preflight".to_string(),
        ],
    })
}

pub(crate) fn check_benchmark_record(
    path: &Path,
    current_head: Option<String>,
) -> Result<BenchmarkRecordCheckReport, String> {
    let content = fs::read_to_string(path).unwrap_or_default();
    let file_exists = path.exists();
    let head_ready = current_head
        .as_ref()
        .is_some_and(|head| content.contains(&format!("`{head}`")) || content.contains(head));
    let cases = benchmark_cases();
    let missing_sections = cases
        .iter()
        .filter(|case| benchmark_case_section(&content, case).is_none())
        .map(|case| case.id.clone())
        .collect::<Vec<_>>();
    let mut checks = vec![
        preflight_check(
            "benchmark_record_file",
            file_exists,
            true,
            path.to_string_lossy(),
            "octopus benchmark record",
        ),
        preflight_check(
            "benchmark_current_head",
            head_ready,
            true,
            current_head
                .as_ref()
                .map(|head| format!("record must include current head {head}"))
                .unwrap_or_else(|| "git head unavailable".to_string()),
            "rerun octopus benchmark record on the target head",
        ),
        preflight_check(
            "benchmark_case_sections",
            missing_sections.is_empty(),
            true,
            if missing_sections.is_empty() {
                format!("{} benchmark case sections present", cases.len())
            } else {
                format!("missing {}", missing_sections.join(", "))
            },
            "regenerate with octopus benchmark record",
        ),
    ];
    for case in &cases {
        let Some(section) = benchmark_case_section(&content, case) else {
            checks.push(preflight_check(
                format!("benchmark_{}", case.id),
                false,
                true,
                "section missing",
                "regenerate with octopus benchmark record",
            ));
            continue;
        };
        let missing = ["Case id", "Command", "Output summary", "Artifact path"]
            .iter()
            .filter(|label| record_field_value(section, label).is_none())
            .map(|label| (*label).to_string())
            .collect::<Vec<_>>();
        let pass_decision_value = record_field_value(section, "Pass or fail");
        let pass_decision = pass_decision_value
            .as_deref()
            .is_some_and(record_pass_decision_ready);
        let ok = missing.is_empty() && pass_decision;
        let evidence = if ok {
            format!("{} pass evidence recorded", case.title)
        } else {
            let mut blockers = missing;
            if !pass_decision {
                blockers.push("Pass or fail".to_string());
            }
            format!("missing {}", blockers.join(", "))
        };
        checks.push(preflight_check(
            format!("benchmark_{}", case.id),
            ok,
            true,
            evidence,
            format!("fill {} in {}", case.title, path.to_string_lossy()),
        ));
    }
    checks.sort_by(|left, right| left.id.cmp(&right.id));
    let passed = checks
        .iter()
        .filter(|check| check.required)
        .all(|check| check.status == "pass");
    let next = if passed {
        vec!["octopus preflight".to_string()]
    } else {
        checks
            .iter()
            .filter(|check| check.status != "pass")
            .map(|check| check.next.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    };
    Ok(BenchmarkRecordCheckReport {
        path: path.to_string_lossy().to_string(),
        current_head,
        passed,
        checks,
        next,
    })
}

pub(crate) fn benchmark_record_evidence(report: &BenchmarkRecordCheckReport) -> String {
    let failed = report
        .checks
        .iter()
        .filter(|check| check.status != "pass")
        .map(|check| format!("{}={}", check.id, check.status))
        .collect::<Vec<_>>();
    if failed.is_empty() {
        format!("passed=true checks={}", report.checks.len())
    } else {
        format!(
            "passed=false checks={} blockers={}",
            report.checks.len(),
            failed.join("; ")
        )
    }
}

fn benchmark_case_section<'a>(content: &'a str, case: &BenchmarkCase) -> Option<&'a str> {
    let marker = format!("### {}", case.title);
    let start = content.find(&marker)?;
    let rest = &content[start..];
    let next_case = rest
        .find("\n### ")
        .filter(|index| *index > 0)
        .unwrap_or(rest.len());
    let next_section = rest
        .find("\n## ")
        .filter(|index| *index > 0)
        .unwrap_or(rest.len());
    Some(&rest[..next_case.min(next_section)])
}
