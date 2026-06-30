use crate::desktop_pet::DesktopPetReport;
use crate::pet::{PetImageReport, PetReport};
use crate::{pet, pet_events, pet_supervision};
use octopus_core::{GoalStatus, HarnessState, NeedQueueStatus, PetEvent, StatusReport};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use super::{join_or_none, repo_root, Language};

#[derive(Debug, serde::Serialize)]
pub(crate) struct PetEventLogReport {
    path: String,
    exists: bool,
    count: usize,
    events: Vec<PetEvent>,
    next: Vec<String>,
}

pub(crate) fn print_pet_report(report: &PetReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus pet");
            println!("pixel: {}", report.chat_badge);
            println!("state: {}", report.state);
            println!("summary: {}", report.summary);
            if let Some(source) = &report.event_source {
                println!("event: {source}");
            }
            if let Some(summary) = &report.event_summary {
                println!("event_summary: {summary}");
            }
            println!("url: {}", report.target);
            println!("exists: {}", report.exists);
        }
        Language::Zh => {
            println!("章鱼桌宠");
            println!("像素: {}", report.chat_badge);
            println!("状态: {}", report.state);
            println!("摘要: {}", report.summary);
            if let Some(source) = &report.event_source {
                println!("事件: {source}");
            }
            if let Some(summary) = &report.event_summary {
                println!("事件摘要: {summary}");
            }
            println!("入口: {}", report.target);
            println!("存在: {}", report.exists);
        }
    }
}

pub(crate) fn print_pet_event_log_report(report: &PetEventLogReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus pet events");
            println!("path: {}", report.path);
            println!("exists: {}", report.exists);
            println!("count: {}", report.count);
            if report.events.is_empty() {
                println!("events: none");
            } else {
                println!("events:");
                for event in &report.events {
                    println!(
                        "- {} {} {} {:?}: {}",
                        event.timestamp_secs,
                        event.state,
                        event.source,
                        event.status,
                        event.summary
                    );
                }
            }
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼桌宠事件");
            println!("路径: {}", report.path);
            println!("存在: {}", report.exists);
            println!("总数: {}", report.count);
            if report.events.is_empty() {
                println!("事件: 无");
            } else {
                println!("事件:");
                for event in &report.events {
                    println!(
                        "- {} {} {} {:?}: {}",
                        event.timestamp_secs,
                        event.state,
                        event.source,
                        event.status,
                        event.summary
                    );
                }
            }
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

pub(crate) fn print_pet_image_report(report: &PetImageReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus pet image");
            println!("state: {}", report.pet.state);
            println!("pixel: {}", report.pet.chat_badge);
            println!("format: {}", report.format);
            println!("path: {}", report.image_path);
            println!("url: {}", report.image_url);
            println!("bytes: {}", report.bytes);
        }
        Language::Zh => {
            println!("章鱼桌宠图片");
            println!("状态: {}", report.pet.state);
            println!("像素: {}", report.pet.chat_badge);
            println!("格式: {}", report.format);
            println!("路径: {}", report.image_path);
            println!("地址: {}", report.image_url);
            println!("字节: {}", report.bytes);
        }
    }
}

pub(crate) fn print_pet_supervision_report(
    report: &pet_supervision::PetSupervisionReport,
    language: Language,
) {
    match language {
        Language::En => {
            println!("Octopus pet supervision");
            println!("status: {}", report.status);
            println!("state: {}", report.state_path);
            println!(
                "events: {} ({})",
                report.event_log_path, report.event_log_count
            );
            for check in &report.checks {
                println!("{}: {} - {}", check.id, check.status, check.evidence);
                if check.status != "pass" {
                    println!("next: {}", check.next);
                }
            }
        }
        Language::Zh => {
            println!("章鱼桌宠监督");
            println!("状态: {}", report.status);
            println!("状态文件: {}", report.state_path);
            println!(
                "事件日志: {} ({})",
                report.event_log_path, report.event_log_count
            );
            for check in &report.checks {
                println!("{}: {} - {}", check.id, check.status, check.evidence);
                if check.status != "pass" {
                    println!("下一步: {}", check.next);
                }
            }
        }
    }
}

pub(crate) fn print_desktop_pet_report(report: &DesktopPetReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus desktop pet");
            println!("mode: read-only observer");
            println!("state: {}", report.state_path);
            println!("worker_cap: {}", report.worker_cap);
            println!("app: {}", report.app_path);
            println!("launched: {}", report.launched);
        }
        Language::Zh => {
            println!("章鱼桌面观察器");
            println!("模式: 只读观察");
            println!("状态文件: {}", report.state_path);
            println!("worker上限: {}", report.worker_cap);
            println!("App: {}", report.app_path);
            println!("已启动: {}", report.launched);
        }
    }
}

pub(crate) fn pet_report_for_state(
    state: &HarnessState,
    state_path: &Path,
    requested: &str,
) -> Result<PetReport, String> {
    let status = if requested == "auto" {
        Some(state.status_report_with_state(Some(state_path)))
    } else {
        None
    };
    let pet_state = if let Some(status) = &status {
        auto_pet_state(status)
    } else {
        requested.to_string()
    };
    let mut report = pet_report(&pet_state)?;
    if let Some(status) = &status {
        if let Some(event) = &status.last_pet_event {
            if event.state == report.state && pet_supervision::pet_event_fresh(event) {
                report.event_source = Some(event.source.clone());
                report.event_summary = Some(event.summary.clone());
            }
        }
        enrich_pet_target(&mut report, status);
    }
    Ok(report)
}

pub(crate) fn pet_event_log_report(
    state_path: &Path,
    limit: usize,
) -> Result<PetEventLogReport, String> {
    let path = pet_events::event_log_path(state_path);
    if !path.exists() {
        return Ok(PetEventLogReport {
            path: path.to_string_lossy().to_string(),
            exists: false,
            count: 0,
            events: Vec::new(),
            next: vec![
                "Run the current Goal until the next Need or Feed writes a pet event.".to_string(),
            ],
        });
    }
    let file = fs::File::open(&path).map_err(|error| error.to_string())?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let event = serde_json::from_str::<PetEvent>(&line).map_err(|error| {
            format!(
                "invalid pet event log entry in {}: {error}",
                path.to_string_lossy()
            )
        })?;
        events.push(event);
    }
    let count = events.len();
    if count > limit {
        events = events.split_off(count.saturating_sub(limit));
    }
    Ok(PetEventLogReport {
        path: path.to_string_lossy().to_string(),
        exists: true,
        count,
        events,
        next: vec!["Use strategy diagnostics if the latest pet event is stale.".to_string()],
    })
}

fn enrich_pet_target(report: &mut PetReport, status: &StatusReport) {
    let mut params = Vec::new();
    if let Some(goal) = &status.goal {
        params.push(("goal", goal.objective.as_str()));
    }
    let fresh_event = status
        .last_pet_event
        .as_ref()
        .filter(|event| pet_supervision::pet_event_fresh(event));
    let event_state = fresh_event.map(|event| event.state.as_str());
    if matches!(event_state, Some("need")) {
        if let Some(event) = fresh_event {
            params.push(("need", event.summary.as_str()));
        }
    } else if event_state.is_none() {
        if let Some(item) = status
            .latest_need_queue_item
            .as_ref()
            .filter(|item| item.status == NeedQueueStatus::Pending)
        {
            params.push(("need", item.need.query.as_str()));
        } else if let Some(trace) = &status.latest_feed_trace {
            params.push(("need", trace.need_query.as_str()));
        }
    }
    if matches!(event_state, Some("feed" | "success")) {
        if let Some(event) = fresh_event {
            params.push(("feed", event.summary.as_str()));
        }
    } else if event_state.is_none() {
        if let Some(trace) = &status.latest_feed_trace {
            params.push(("feed", trace.summary.as_str()));
        }
    }
    if let Some(event) = fresh_event {
        params.push(("source", event.source.as_str()));
    } else if let Some(trace) = &status.latest_feed_trace {
        if let Some(tentacle) = trace.tentacle.as_deref() {
            params.push(("source", tentacle));
        }
    }
    let worker_count = status
        .latest_parallel_evolution_run
        .as_ref()
        .map(|run| run.worker_count)
        .unwrap_or(status.tentacles.len())
        .clamp(1, 8);
    let workers = worker_count.to_string();
    params.push(("workers", workers.as_str()));
    for (key, value) in params {
        if !value.trim().is_empty() {
            report.target.push('&');
            report.target.push_str(key);
            report.target.push('=');
            report
                .target
                .push_str(&percent_encode_query_component(&truncate_query_param(
                    value, 180,
                )));
        }
    }
}

fn truncate_query_param(value: &str, max_chars: usize) -> String {
    let mut output = String::new();
    for character in value.chars().take(max_chars) {
        output.push(character);
    }
    output
}

fn percent_encode_query_component(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*byte as char)
            }
            value => encoded.push_str(&format!("%{value:02X}")),
        }
    }
    encoded
}

fn auto_pet_state(report: &StatusReport) -> String {
    if report
        .goal
        .as_ref()
        .is_some_and(|goal| goal.status == GoalStatus::Blocked)
        || report.tentacles.is_empty()
    {
        return "blocked".to_string();
    }
    if report
        .goal
        .as_ref()
        .is_some_and(|goal| goal.status == GoalStatus::Satisfied)
    {
        return "success".to_string();
    }
    if let Some(event) = &report.last_pet_event {
        if pet_supervision::pet_event_fresh(event)
            && pet::state_known(&event.state)
            && event.state != "heartbeat"
        {
            return event.state.clone();
        }
    }
    if report.need_queue_count > 0 {
        return "need".to_string();
    }
    if report.latest_feed_trace.is_some() {
        return "feed".to_string();
    }
    if report.evolution_outcome_count > 0 || report.repair_outcome_count > 0 {
        return "harness".to_string();
    }
    if report.route_count > 0 {
        return "harness".to_string();
    }
    if report.memory_count > 0 {
        return "memory".to_string();
    }
    "heartbeat".to_string()
}

pub(crate) fn pet_report(state: &str) -> Result<PetReport, String> {
    pet::pet_report(state, &repo_root().join("docs/pet.html"))
}
