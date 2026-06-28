use super::{provider_profile, valid_env_prefix, DEFAULT_PROVIDER_ENV_PATH};
use std::env;
use std::fs;
use std::process::Command;

#[derive(serde::Deserialize)]
pub(crate) struct RunRequest {
    pub(crate) args: Vec<String>,
}

#[derive(serde::Serialize)]
pub(crate) struct RunResponse {
    pub(crate) ok: bool,
    pub(crate) code: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) policy: Option<String>,
    pub(crate) suggested_args: Vec<Vec<String>>,
}

pub(crate) fn denied_response(args: &[String]) -> RunResponse {
    let command = command_name(args).unwrap_or("unknown");
    let state = state_arg(args).unwrap_or_else(|| ".octopus/state.json".to_string());
    let message = format!(
        "local app input is limited to brain-goal. `{command}` is internal, developer-only, or observation-only from this surface. Use chat, goal set/refine, brain --goal, or first-run."
    );
    RunResponse {
        ok: false,
        code: None,
        stdout: String::new(),
        stderr: message,
        policy: Some("user_writes_brain_goal_only".to_string()),
        suggested_args: vec![
            vec![
                "--state".to_string(),
                state.clone(),
                "--json".to_string(),
                "chat".to_string(),
                "describe or refine the goal".to_string(),
            ],
            vec![
                "--state".to_string(),
                state.clone(),
                "--json".to_string(),
                "goal".to_string(),
                "refine".to_string(),
                "tighten the current objective".to_string(),
            ],
            vec![
                "--state".to_string(),
                state,
                "--json".to_string(),
                "brain".to_string(),
                "--goal".to_string(),
                "--save".to_string(),
                "tighten the current objective".to_string(),
            ],
        ],
    }
}

pub(crate) fn apply_env_overlay(command: &mut Command) {
    for (key, value) in env_overlay() {
        command.env(key, value);
    }
}

fn env_overlay() -> Vec<(String, String)> {
    fs::read_to_string(DEFAULT_PROVIDER_ENV_PATH)
        .map(|content| parse_env_overlay(&content))
        .unwrap_or_default()
}

pub(crate) fn parse_env_overlay(content: &str) -> Vec<(String, String)> {
    content.lines().filter_map(parse_env_line).collect()
}

fn parse_env_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let assignment = line.strip_prefix("export ").unwrap_or(line);
    let (key, value) = assignment.split_once('=')?;
    let key = key.trim();
    if !key.starts_with("OCTOPUS_") || !valid_env_prefix(key) {
        return None;
    }
    Some((key.to_string(), env_value(value.trim())))
}

fn env_value(value: &str) -> String {
    if let Some(inner) = value
        .strip_prefix('"')
        .and_then(|item| item.strip_suffix('"'))
    {
        if let Some(name) = inner
            .strip_prefix("${")
            .and_then(|item| item.strip_suffix(":-}"))
            .filter(|name| valid_env_prefix(name))
        {
            return env::var(name).unwrap_or_default();
        }
        return inner.replace("\\\"", "\"").replace("\\\\", "\\");
    }
    if let Some(inner) = value
        .strip_prefix('\'')
        .and_then(|item| item.strip_suffix('\''))
    {
        return inner.replace("'\\''", "'");
    }
    value.to_string()
}

pub(crate) fn command_allowed(args: &[String]) -> bool {
    let Some(command) = command_name(args) else {
        return false;
    };
    if command == "provider" {
        return provider_observe_allowed(args);
    }
    if command == "preflight" {
        return preflight_observe_allowed(args);
    }
    if command == "first-run" {
        return first_run_allowed(args);
    }
    if command == "update" {
        return update_allowed(args);
    }
    if command == "brain" {
        return brain_goal_allowed(args);
    }
    if command == "goal" {
        return goal_allowed(args);
    }
    if command == "pet" {
        return pet_observe_allowed(args);
    }
    if command == "starter" {
        return starter_observe_allowed(args);
    }
    if command == "needs" || command == "context" {
        return observe_only(args);
    }
    matches!(
        command,
        "chat"
            | "doctor"
            | "report"
            | "status"
            | "installed"
            | "skills"
            | "catalog"
            | "manifests"
            | "providers"
            | "traces"
    )
}

fn observe_only(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    args.len() == index + 1
}

fn goal_allowed(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    matches!(
        args.get(index + 1).map(String::as_str),
        Some("set" | "refine")
    )
}

fn brain_goal_allowed(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    let rest = &args[index + 1..];
    let mut has_goal = false;
    let mut position = 0;
    while position < rest.len() {
        match rest[position].as_str() {
            "--goal" => has_goal = true,
            "--live" | "--save" | "--session" => {}
            "--apply" | "--apply-json" | "--llm-prefix" | "--provider-prefix" => {
                position += 1;
                if position >= rest.len() {
                    return false;
                }
            }
            "--intent" | "--brief" | "--clarify" | "--agenda" | "--scout" | "--deliberate"
            | "--synthesize" | "--council" | "--reflect" | "--align" | "--memory" | "--focus"
            | "--rewrite" | "--models" => return false,
            value if value.starts_with('-') => return false,
            _ => {}
        }
        position += 1;
    }
    has_goal
}

fn first_run_allowed(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    args[index + 1..]
        .iter()
        .all(|arg| !arg.starts_with('-') || arg == "--live")
}

fn update_allowed(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    args.len() == index + 1
        || (args.len() == index + 2
            && args
                .get(index + 1)
                .is_some_and(|value| value == "--dry-run"))
}

fn pet_observe_allowed(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    let rest = &args[index + 1..];
    rest.len() <= 1 && rest.first().is_none_or(|value| value != "image")
}

fn preflight_observe_allowed(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    let rest = &args[index + 1..];
    rest.is_empty() || (rest.len() == 1 && rest[0] == "--live")
}

fn starter_observe_allowed(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    args.get(index + 1)
        .is_none_or(|value| value.as_str() != "feedback")
}

fn provider_observe_allowed(args: &[String]) -> bool {
    let Some(index) = command_index(args) else {
        return false;
    };
    match args.get(index + 1).map(String::as_str) {
        Some("status") => args.len() == index + 2,
        Some("check") => {
            args.get(index + 2)
                .is_none_or(|prefix| valid_env_prefix(prefix))
                && args.len() <= index + 3
        }
        Some(profile) => {
            provider_profile(profile).is_ok()
                && args
                    .get(index + 2)
                    .is_none_or(|prefix| valid_env_prefix(prefix))
                && args.len() <= index + 3
        }
        None => false,
    }
}

pub(crate) fn command_name(args: &[String]) -> Option<&str> {
    command_index(args).and_then(|index| args.get(index).map(String::as_str))
}

fn command_index(args: &[String]) -> Option<usize> {
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--state" | "--lang" => index += 2,
            "--json" => index += 1,
            value if value.starts_with('-') => return None,
            _ => return Some(index),
        }
    }
    None
}

fn state_arg(args: &[String]) -> Option<String> {
    args.windows(2)
        .find(|window| window.first().is_some_and(|arg| arg == "--state"))
        .and_then(|window| window.get(1).cloned())
}
