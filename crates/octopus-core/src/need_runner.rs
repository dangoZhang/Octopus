use crate::pet_events;
use octopus_core::{
    default_field_pack_catalog, default_field_pack_ids, field_harder_layer_next_action, Feed,
    FieldVerifierResult, HarnessState, Need, NeedQueueItem, NeedQueueStatus, NeedQueueTakeReport,
    ParallelEvolutionRun, PetEvent, Status,
};
use std::path::Path;

use super::{
    ensure_field_mini_task_tentacle, harness_for_need, join_or_none, parse_status, truncate_chars,
    MAX_WORKER_COUNT,
};

#[derive(Debug, serde::Serialize)]
pub(crate) struct NeedRunReport {
    pub(crate) taken: NeedQueueTakeReport,
    pub(crate) feed: Feed,
    pub(crate) feed_trace_index: Option<u64>,
    pub(crate) field: Option<String>,
    pub(crate) verifier_result: Option<FieldVerifierResult>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct NeedRunBatchReport {
    pub(crate) requested: usize,
    pub(crate) ran: usize,
    pub(crate) reports: Vec<NeedRunReport>,
    pub(crate) remaining_pending: usize,
    pub(crate) next: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NeedRunSelector {
    First,
    Latest,
    Index(u64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NeedRunRequest {
    Single(NeedRunSelector),
    Batch(usize),
}

pub(crate) fn parse_parallel_evolution_args(
    values: &[String],
) -> Result<(bool, usize, Option<String>), String> {
    let mut open_pet = false;
    let mut workers = 4usize;
    let mut objective = Vec::new();
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--open" => open_pet = true,
            "--workers" => {
                index += 1;
                let value = values
                    .get(index)
                    .ok_or_else(|| "evolve parallel --workers requires 1..8".to_string())?;
                workers = parse_worker_count_1_to_8(value)?;
            }
            value => objective.push(value.to_string()),
        }
        index += 1;
    }
    let objective = (!objective.is_empty()).then(|| objective.join(" "));
    Ok((open_pet, workers, objective))
}

pub(crate) fn record_parallel_evolution_action_event(
    state: &mut HarnessState,
    run: &ParallelEvolutionRun,
) -> Option<PetEvent> {
    (!run.workers.is_empty()).then(|| {
        let labels = run
            .workers
            .iter()
            .map(|worker| match worker.mini_task.as_deref() {
                Some(task) => format!("{}/{}", worker.field, task),
                None => worker.field.clone(),
            })
            .collect::<Vec<_>>();
        let detail = truncate_chars(&join_or_none(&labels), 180);
        state.record_pet_event(
            "action",
            "parallel evolution",
            format!(
                "running {} peer field worker Need(s) through Feed: {}",
                run.workers.len(),
                detail
            ),
            Status::Partial,
        )
    })
}

pub(crate) fn parallel_evolution_next_actions(auto_feed: &NeedRunBatchReport) -> Vec<String> {
    let mut next = if auto_feed
        .reports
        .iter()
        .any(|report| report.field.is_some())
    {
        vec![
            "octopus fields summary".to_string(),
            "octopus status".to_string(),
        ]
    } else {
        auto_feed.next.clone()
    };
    if !next.iter().any(|action| action == "octopus beat 200") {
        next.push("octopus beat 200".to_string());
    }
    next
}

pub(crate) fn empty_parallel_evolution_batch_report(state: &HarnessState) -> NeedRunBatchReport {
    NeedRunBatchReport {
        requested: 0,
        ran: 0,
        reports: Vec::new(),
        remaining_pending: state.pending_need_queue_count(),
        next: vec![
            "octopus fields summary".to_string(),
            field_harder_layer_next_action(""),
        ],
    }
}

fn need_run_next_actions(
    field: Option<&str>,
    verifier_result: Option<&FieldVerifierResult>,
    feed_trace_index: Option<u64>,
) -> Vec<String> {
    let Some(_) = field else {
        return vec!["octopus status".to_string()];
    };
    if verifier_result.is_some() {
        return vec![
            "octopus fields summary".to_string(),
            "octopus status".to_string(),
        ];
    }
    let Some(trace_index) = feed_trace_index else {
        return vec!["octopus status".to_string()];
    };
    vec![
        format!(
            "octopus fields score {trace_index} satisfied verifier_pass \"record field verifier result\""
        ),
        format!("octopus fields score {trace_index} failed error_category \"record field failure\""),
        "octopus status".to_string(),
    ]
}

pub(crate) fn record_auto_field_verifier_from_feed(
    state: &mut HarnessState,
    feed: &Feed,
) -> (Option<u64>, Option<String>, Option<FieldVerifierResult>) {
    let feed_trace_index = feed
        .metadata
        .get("feed_trace_index")
        .and_then(|index| index.parse::<u64>().ok());
    let field = feed
        .metadata
        .get("field_pack")
        .or_else(|| feed.metadata.get("field"))
        .or_else(|| feed.need.context.get("field_pack"))
        .cloned();
    let verifier_result = field
        .as_ref()
        .and_then(|_| feed_trace_index)
        .and_then(|index| {
            let explicit_verifier_status = feed
                .metadata
                .get("verifier_status")
                .and_then(|value| parse_status(value).ok());
            let verifier_status = match feed.status {
                Status::Failed | Status::Unsupported => feed.status.clone(),
                Status::Satisfied
                    if explicit_verifier_status.as_ref() == Some(&Status::Satisfied) =>
                {
                    Status::Satisfied
                }
                Status::Satisfied | Status::Partial => Status::Partial,
            };
            let mini_task = feed.metadata.get("field_mini_task").cloned();
            let error_category = match verifier_status {
                Status::Satisfied => None,
                Status::Partial if mini_task.is_some() => {
                    Some("field_mini_task_incomplete".to_string())
                }
                Status::Partial => Some("missing_field_mini_task".to_string()),
                Status::Failed | Status::Unsupported => Some("field_feed_failed".to_string()),
            };
            let summary = match (&verifier_status, mini_task.as_deref()) {
                (Status::Satisfied, Some(task)) => format!(
                    "auto verifier: Feed returned {:?}; mini task {task} provided explicit pass evidence",
                    feed.status
                ),
                (Status::Satisfied, None) => format!(
                    "auto verifier: Feed returned {:?}; verifier accepted explicit pass evidence",
                    feed.status
                ),
                (_, Some(task)) => format!(
                    "auto verifier: Feed returned {:?}; mini task {task} still needs explicit pass evidence",
                    feed.status
                ),
                (_, None) => format!(
                    "auto verifier: Feed returned {:?}; field mini task still needs explicit pass evidence",
                    feed.status
                ),
            };
            let artifact = field_verifier_artifact_from_feed(feed);
            state
                .record_field_verifier_result(index, verifier_status, error_category, artifact, summary)
                .ok()
        });
    (feed_trace_index, field, verifier_result)
}

fn field_verifier_artifact_from_feed(feed: &Feed) -> Option<String> {
    [
        "field_pass_evidence",
        "artifact_path",
        "trajectory_artifact",
        "checks_path",
        "artifact",
    ]
    .iter()
    .find_map(|key| {
        feed.metadata
            .get(*key)
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

fn need_run_batch_next_actions(
    reports: &[NeedRunReport],
    remaining_pending: usize,
    current_batch_size: usize,
) -> Vec<String> {
    if remaining_pending > 0 {
        return vec![format!(
            "octopus needs run --workers {}",
            next_need_run_worker_count(remaining_pending, current_batch_size)
        )];
    }
    if reports.iter().any(|report| report.field.is_some()) {
        return vec![
            "octopus fields summary".to_string(),
            "octopus status".to_string(),
        ];
    }
    vec!["octopus status".to_string()]
}

pub(crate) fn parse_worker_cap(values: &[String]) -> Result<usize, String> {
    let mut workers = 1usize;
    let mut index = 0;
    while index < values.len() {
        if values[index] == "--workers" {
            index += 1;
            let value = values
                .get(index)
                .ok_or_else(|| "pet desktop --workers requires 1..8".to_string())?;
            workers = parse_worker_count_1_to_8(value)?;
        }
        index += 1;
    }
    Ok(workers)
}

pub(crate) fn parse_worker_count_1_to_8(value: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .ok()
        .filter(|value| (1..=MAX_WORKER_COUNT).contains(value))
        .ok_or_else(|| format!("invalid workers value: {value}; expected 1..8"))
}

pub(crate) fn next_need_run_worker_count(
    remaining_pending: usize,
    current_batch_size: usize,
) -> usize {
    remaining_pending
        .min(current_batch_size.max(1))
        .min(MAX_WORKER_COUNT)
}

fn parse_need_run_selector(value: Option<&str>) -> Result<NeedRunSelector, String> {
    match value {
        Some("latest") => Ok(NeedRunSelector::Latest),
        Some(value) => value
            .parse::<u64>()
            .ok()
            .filter(|index| *index > 0)
            .map(NeedRunSelector::Index)
            .ok_or_else(|| {
                format!(
                    "invalid Need run selector: {value}; expected latest or positive queue index"
                )
            }),
        None => Ok(NeedRunSelector::First),
    }
}

pub(crate) fn parse_need_run_request(values: &[String]) -> Result<NeedRunRequest, String> {
    match values.first().map(String::as_str) {
        Some("all") | Some("--all") => {
            if values.len() > 1 {
                return Err("needs run all does not accept extra arguments".to_string());
            }
            Ok(NeedRunRequest::Batch(MAX_WORKER_COUNT))
        }
        Some("--workers") => {
            if values.len() > 2 {
                return Err("needs run --workers accepts exactly one value".to_string());
            }
            let value = values
                .get(1)
                .ok_or_else(|| "needs run --workers requires 1..8".to_string())?;
            let workers = parse_worker_count_1_to_8(value)?;
            Ok(NeedRunRequest::Batch(workers))
        }
        Some(value) => {
            if values.len() > 1 {
                return Err("needs run selector accepts one value".to_string());
            }
            Ok(NeedRunRequest::Single(parse_need_run_selector(Some(
                value,
            ))?))
        }
        None => Ok(NeedRunRequest::Single(NeedRunSelector::First)),
    }
}

pub(crate) fn run_queued_need_with_observer_state(
    mut state: HarnessState,
    selector: NeedRunSelector,
    observer_state_path: Option<&Path>,
) -> Result<(NeedRunReport, HarnessState), String> {
    let pending = state
        .need_queue
        .iter()
        .filter(|item| item.status == NeedQueueStatus::Pending)
        .map(|item| item.index)
        .collect::<Vec<_>>();
    let index = match selector {
        NeedRunSelector::First => pending.first().copied(),
        NeedRunSelector::Latest => pending.last().copied(),
        NeedRunSelector::Index(index) => Some(index),
    }
    .ok_or_else(|| "no pending Need to run".to_string())?;
    let taken = state.take_queued_need(index)?;
    let kind = taken.item.need.kind.clone();
    let query = taken.item.need.query.clone();
    state.record_pet_event("need", "need queue", query.clone(), Status::Partial);
    if let Some(path) = observer_state_path {
        state.save(path).map_err(|error| error.to_string())?;
        pet_events::append_latest(path, &state)?;
    }
    let field_hint = field_hint_from_queue_item(&taken.item);
    if field_hint.is_some() {
        ensure_field_mini_task_tentacle(&mut state)?;
    }
    let mut harness = harness_for_need(state, &kind)?;
    let mut need = Need::new(kind, query);
    for (key, value) in &taken.item.context {
        if !value.trim().is_empty() {
            need.context.insert(key.clone(), value.clone());
        }
    }
    if let Some(field) = field_hint {
        need.context
            .insert("field_original_need".to_string(), need.query.clone());
        need.context.insert("field_pack".to_string(), field.clone());
        if let Some((task_id, expected_feed)) = field_mini_task_context(&field, &taken.item) {
            need.context.insert("field_mini_task".to_string(), task_id);
            need.context
                .insert("field_expected_feed".to_string(), expected_feed);
        }
    }
    let feed = harness.feed_one(&need);
    let (feed_trace_index, field, verifier_result) =
        record_auto_field_verifier_from_feed(&mut harness.state, &feed);
    let next = need_run_next_actions(field.as_deref(), verifier_result.as_ref(), feed_trace_index);
    if let Some(path) = observer_state_path {
        harness
            .state
            .save(path)
            .map_err(|error| error.to_string())?;
        pet_events::append_latest(path, &harness.state)?;
    }
    Ok((
        NeedRunReport {
            taken,
            feed,
            feed_trace_index,
            field,
            verifier_result,
            next,
        },
        harness.state,
    ))
}

pub(crate) fn run_queued_needs_with_observer_state(
    mut state: HarnessState,
    limit: usize,
    observer_state_path: Option<&Path>,
) -> Result<(NeedRunBatchReport, HarnessState), String> {
    let pending = state.pending_need_queue_count();
    if pending == 0 {
        return Err("no pending Need to run".to_string());
    }
    let requested = if limit == usize::MAX {
        pending
    } else {
        limit.max(1).min(pending)
    };
    let mut reports = Vec::new();
    for _ in 0..requested {
        let (report, next_state) = run_queued_need_with_observer_state(
            state,
            NeedRunSelector::First,
            observer_state_path,
        )?;
        state = next_state;
        reports.push(report);
    }
    let remaining_pending = state.pending_need_queue_count();
    let next = need_run_batch_next_actions(&reports, remaining_pending, requested);
    Ok((
        NeedRunBatchReport {
            requested,
            ran: reports.len(),
            reports,
            remaining_pending,
            next,
        },
        state,
    ))
}

pub(crate) fn run_queued_need_indices_with_observer_state(
    mut state: HarnessState,
    indices: &[u64],
    observer_state_path: Option<&Path>,
) -> Result<(NeedRunBatchReport, HarnessState), String> {
    if indices.is_empty() {
        return Err("no queued Need indices to run".to_string());
    }
    let mut reports = Vec::new();
    for index in indices {
        let (report, next_state) = run_queued_need_with_observer_state(
            state,
            NeedRunSelector::Index(*index),
            observer_state_path,
        )?;
        state = next_state;
        reports.push(report);
    }
    let remaining_pending = state.pending_need_queue_count();
    let next = need_run_batch_next_actions(&reports, remaining_pending, indices.len());
    Ok((
        NeedRunBatchReport {
            requested: indices.len(),
            ran: reports.len(),
            reports,
            remaining_pending,
            next,
        },
        state,
    ))
}

pub(crate) fn field_hint_from_queue_item(item: &NeedQueueItem) -> Option<String> {
    if item.source != "field evolution" {
        return None;
    }
    if let Some(field) = item.context.get("field_pack") {
        if field_hint_ids().iter().any(|candidate| candidate == field) {
            return Some(field.clone());
        }
    }
    let text = format!(
        "{} {} {}",
        item.need.query.to_ascii_lowercase(),
        item.prompt.to_ascii_lowercase(),
        item.summary.to_ascii_lowercase()
    );
    if let Some(field) = explicit_field_hint_from_text(&text) {
        return Some(field);
    }
    None
}

fn explicit_field_hint_from_text(text: &str) -> Option<String> {
    for field in field_hint_ids() {
        if text.contains(&format!("field: {field}"))
            || text.contains(&format!("run {field} mini task"))
            || text.contains(&format!("run {field} in this peer field slot"))
            || text.contains(&format!("run {field} in this worker slot"))
            || text.contains(&format!("improve {field} harness"))
            || contains_field_mini_marker(text, &field)
        {
            return Some(field.to_string());
        }
    }
    None
}

pub(crate) fn field_hint_ids() -> Vec<String> {
    default_field_pack_ids()
}

pub(crate) fn contains_field_mini_marker(text: &str, field: &str) -> bool {
    let marker = format!("{field}-mini-");
    let mut start = 0;
    while let Some(offset) = text[start..].find(&marker) {
        let index = start + offset;
        let before = text[..index].chars().next_back();
        let has_boundary = before
            .map(|value| !value.is_ascii_alphanumeric() && value != '-')
            .unwrap_or(true);
        if has_boundary {
            return true;
        }
        start = index + marker.len();
    }
    false
}

pub(crate) fn field_mini_task_context(
    field: &str,
    item: &NeedQueueItem,
) -> Option<(String, String)> {
    let catalog = default_field_pack_catalog().ok()?;
    let pack = catalog.packs.iter().find(|pack| pack.id == field)?;
    if let Some(task_id) = item.context.get("field_mini_task") {
        let task = pack.mini_tasks.iter().find(|task| task.id == *task_id)?;
        return Some((
            task.id.clone(),
            item.context
                .get("field_expected_feed")
                .cloned()
                .unwrap_or_else(|| task.expected_feed.clone()),
        ));
    }
    let text = format!(
        "{} {}",
        item.need.query.to_ascii_lowercase(),
        item.summary.to_ascii_lowercase()
    );
    let task = pack
        .mini_tasks
        .iter()
        .find(|task| text.contains(&task.id.to_ascii_lowercase()))?;
    Some((task.id.clone(), task.expected_feed.clone()))
}
