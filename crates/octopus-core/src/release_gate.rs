use std::path::Path;

#[derive(Debug, serde::Serialize)]
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
        "\"$OCTOPUS\" --state \"$STATE\" preflight".to_string(),
        "\"$OCTOPUS\" --state \"$STATE\" preflight | grep bridge_goal_surface".to_string(),
        "\"$OCTOPUS\" provider status".to_string(),
        "\"$OCTOPUS\" provider matrix \"$tmp/provider-matrix.md\"".to_string(),
        "\"$OCTOPUS\" provider check \"${OCTOPUS_LLM_PREFIX:-OCTOPUS_LLM}\"".to_string(),
        "\"$OCTOPUS\" --state \"$STATE\" preflight --live".to_string(),
        "\"$OCTOPUS\" start 127.0.0.1:18765 &".to_string(),
        "START_PID=$!".to_string(),
        "trap 'kill \"$START_PID\" 2>/dev/null || true' EXIT".to_string(),
        "sleep 1".to_string(),
        "curl -fsS http://127.0.0.1:18765/app.html >/dev/null".to_string(),
        "curl -fsS 'http://127.0.0.1:18765/pet.html?state=harness' >/dev/null".to_string(),
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
