use crate::llm_layers::clean_brain_queue_llm_client;
use crate::need_runner::{
    parse_need_run_request, run_queued_need_with_observer_state,
    run_queued_needs_with_observer_state, NeedRunBatchReport, NeedRunReport, NeedRunRequest,
};
use crate::route_surface::need_queue_context_label;
use crate::shell_words::shell_arg;
use crate::{
    clean_brain_session_draft_json, make_executable, need_label, parse_queue_index, Language,
};
use octopus_core::{
    BrainPromptReport, ChatMessage, ChatRole, HarnessState, NeedQueueItem, NeedQueueReport,
    NeedQueueSaveReport, NeedQueueStatus, NeedQueueTakeReport,
};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, serde::Serialize)]
pub(crate) struct NeedQueueScriptReport {
    pub(crate) policy: String,
    pub(crate) script_path: String,
    pub(crate) command_count: usize,
    pub(crate) commands: Vec<String>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct NeedQueueReviewSessionReport {
    pub(crate) policy: String,
    pub(crate) prompt: String,
    pub(crate) pending_count: usize,
    pub(crate) session_dir: String,
    pub(crate) prompt_path: String,
    pub(crate) messages_path: String,
    pub(crate) review_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) draft_path: Option<String>,
    pub(crate) command_path: String,
    pub(crate) next: Vec<String>,
}

pub(crate) fn handle_needs_command(
    rest: &[String],
    state: &Path,
    json: bool,
    language: Language,
) -> Result<(), String> {
    let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
    match rest.get(1).map(String::as_str) {
        Some("run") => {
            let values = rest.get(2..).unwrap_or(&[]);
            match parse_need_run_request(values)? {
                NeedRunRequest::Single(selector) => {
                    let (report, next_state) =
                        run_queued_need_with_observer_state(loaded, selector, Some(&state))?;
                    loaded = next_state;
                    loaded.save(&state).map_err(|error| error.to_string())?;
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .map_err(|error| error.to_string())?
                        );
                    } else {
                        print_need_run_report(&report, language);
                    }
                }
                NeedRunRequest::Batch(limit) => {
                    let (report, next_state) =
                        run_queued_needs_with_observer_state(loaded, limit, Some(&state))?;
                    loaded = next_state;
                    loaded.save(&state).map_err(|error| error.to_string())?;
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .map_err(|error| error.to_string())?
                        );
                    } else {
                        print_need_run_batch_report(&report, language);
                    }
                }
            }
            Ok(())
        }
        Some("take") => {
            let index = rest
                .get(2)
                .ok_or_else(|| "needs take requires a queue index".to_string())
                .and_then(|value| parse_queue_index(value))?;
            let taken = loaded.take_queued_need(index)?;
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&taken).map_err(|error| error.to_string())?
                );
            } else {
                print_need_queue_take(&taken, language);
            }
            Ok(())
        }
        Some("drop") => {
            let index = rest
                .get(2)
                .ok_or_else(|| "needs drop requires a queue index".to_string())
                .and_then(|value| parse_queue_index(value))?;
            let item = loaded.drop_queued_need(index)?;
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&item).map_err(|error| error.to_string())?
                );
            } else {
                print_need_queue_item(&item, language);
            }
            Ok(())
        }
        Some("script") => {
            let path = rest.get(2).map(String::as_str);
            let report = write_need_queue_script(&loaded, &state, path)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_need_queue_script(&report, language);
            }
            Ok(())
        }
        Some("session") => {
            let mut live = false;
            let mut prompt = Vec::new();
            for value in rest.iter().skip(2) {
                if value == "--live" {
                    live = true;
                } else {
                    prompt.push(value.clone());
                }
            }
            let prompt = if prompt.is_empty() {
                "review pending clean-brain Needs".to_string()
            } else {
                prompt.join(" ")
            };
            let report = write_need_queue_review_session(&loaded, &state, &prompt, live)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_need_queue_review_session(&report, language);
            }
            Ok(())
        }
        Some(other) => Err(format!("unknown needs command: {other}")),
        None => {
            let report = loaded.need_queue_report(12);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_need_queue(&report, language);
            }
            Ok(())
        }
    }
}

pub(crate) fn print_need_queue_save(report: &NeedQueueSaveReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus Need queue");
            println!("saved: {}", report.queued.len());
        }
        Language::Zh => {
            println!("章鱼 Need 队列");
            println!("已保存: {}", report.queued.len());
        }
    }
    print_need_queue(&report.queue, language);
}

pub(crate) fn print_need_queue(report: &NeedQueueReport, language: Language) {
    match language {
        Language::En => {
            println!("Need queue");
            println!("brain: {}", report.policy);
            if report.pending.is_empty() {
                println!("pending: none");
            } else {
                println!("pending:");
            }
        }
        Language::Zh => {
            println!("Need 队列");
            println!("主脑: {}", report.policy);
            if report.pending.is_empty() {
                println!("待处理: 无");
            } else {
                println!("待处理:");
            }
        }
    }
    for item in &report.pending {
        print_need_queue_item(item, language);
    }
    if !report.history.is_empty() {
        match language {
            Language::En => println!("history:"),
            Language::Zh => println!("历史:"),
        }
        for item in &report.history {
            print_need_queue_item(item, language);
        }
    }
    for next in &report.next {
        match language {
            Language::En => println!("next: {next}"),
            Language::Zh => println!("下一步: {next}"),
        }
    }
}

pub(crate) fn print_need_queue_take(report: &NeedQueueTakeReport, language: Language) {
    print_need_queue_item(&report.item, language);
    match language {
        Language::En => println!("command: {}", report.command),
        Language::Zh => println!("命令: {}", report.command),
    }
    for next in &report.next {
        match language {
            Language::En => println!("next: {next}"),
            Language::Zh => println!("下一步: {next}"),
        }
    }
}

pub(crate) fn print_need_run_report(report: &NeedRunReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus ran queued Need");
            println!("need: #{}", report.taken.item.index);
            println!("status: {:?}", report.feed.status);
            if let Some(field) = &report.field {
                println!("field: {field}");
            }
            if let Some(index) = report.feed_trace_index {
                println!("trace: #{index}");
            }
            if let Some(result) = &report.verifier_result {
                println!("verifier: #{} {:?}", result.index, result.status);
            }
            println!("summary: {}", report.feed.summary);
        }
        Language::Zh => {
            println!("章鱼已自动运行队列 Need");
            println!("Need: #{}", report.taken.item.index);
            println!("状态: {:?}", report.feed.status);
            if let Some(field) = &report.field {
                println!("领域: {field}");
            }
            if let Some(index) = report.feed_trace_index {
                println!("轨迹: #{index}");
            }
            if let Some(result) = &report.verifier_result {
                println!("验证: #{} {:?}", result.index, result.status);
            }
            println!("摘要: {}", report.feed.summary);
        }
    }
    for next in &report.next {
        match language {
            Language::En => println!("next: {next}"),
            Language::Zh => println!("下一步: {next}"),
        }
    }
}

pub(crate) fn print_need_run_batch_report(report: &NeedRunBatchReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus ran queued Needs");
            println!("requested: {}", report.requested);
            println!("ran: {}", report.ran);
            println!("remaining_pending: {}", report.remaining_pending);
            for item in &report.reports {
                let field = item.field.as_deref().unwrap_or("none");
                let trace = item
                    .feed_trace_index
                    .map(|index| format!("#{index}"))
                    .unwrap_or_else(|| "none".to_string());
                println!(
                    "- need #{} status={:?} field={} trace={} :: {}",
                    item.taken.item.index, item.feed.status, field, trace, item.feed.summary
                );
            }
        }
        Language::Zh => {
            println!("章鱼已批量运行队列 Need");
            println!("请求数: {}", report.requested);
            println!("已运行: {}", report.ran);
            println!("剩余 pending: {}", report.remaining_pending);
            for item in &report.reports {
                let field = item.field.as_deref().unwrap_or("none");
                let trace = item
                    .feed_trace_index
                    .map(|index| format!("#{index}"))
                    .unwrap_or_else(|| "none".to_string());
                println!(
                    "- Need #{} 状态={:?} 领域={} 轨迹={} :: {}",
                    item.taken.item.index, item.feed.status, field, trace, item.feed.summary
                );
            }
        }
    }
    for next in &report.next {
        match language {
            Language::En => println!("next: {next}"),
            Language::Zh => println!("下一步: {next}"),
        }
    }
}

pub(crate) fn print_need_queue_script(report: &NeedQueueScriptReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus Need script");
            println!("brain: {}", report.policy);
            println!("path: {}", report.script_path);
            println!("commands: {}", report.command_count);
        }
        Language::Zh => {
            println!("章鱼 Need 脚本");
            println!("主脑: {}", report.policy);
            println!("路径: {}", report.script_path);
            println!("命令数: {}", report.command_count);
        }
    }
    for next in &report.next {
        match language {
            Language::En => println!("next: {next}"),
            Language::Zh => println!("下一步: {next}"),
        }
    }
}

pub(crate) fn print_need_queue_review_session(
    report: &NeedQueueReviewSessionReport,
    language: Language,
) {
    match language {
        Language::En => {
            println!("Octopus Need review session");
            println!("brain: {}", report.policy);
            println!("pending: {}", report.pending_count);
            println!("session: {}", report.session_dir);
            println!("prompt: {}", report.prompt_path);
            println!("messages: {}", report.messages_path);
            println!("review: {}", report.review_path);
            if let Some(draft_path) = &report.draft_path {
                println!("draft: {draft_path}");
            }
            println!("commands: {}", report.command_path);
        }
        Language::Zh => {
            println!("章鱼 Need 审阅会话");
            println!("主脑: {}", report.policy);
            println!("待处理: {}", report.pending_count);
            println!("会话: {}", report.session_dir);
            println!("提示词: {}", report.prompt_path);
            println!("消息: {}", report.messages_path);
            println!("审阅: {}", report.review_path);
            if let Some(draft_path) = &report.draft_path {
                println!("草稿: {draft_path}");
            }
            println!("命令: {}", report.command_path);
        }
    }
    for next in &report.next {
        match language {
            Language::En => println!("next: {next}"),
            Language::Zh => println!("下一步: {next}"),
        }
    }
}

pub(crate) fn print_need_queue_item(item: &NeedQueueItem, language: Language) {
    let status = match &item.status {
        NeedQueueStatus::Pending => match language {
            Language::En => "pending",
            Language::Zh => "待处理",
        },
        NeedQueueStatus::Taken => match language {
            Language::En => "taken",
            Language::Zh => "已取用",
        },
        NeedQueueStatus::Dropped => match language {
            Language::En => "dropped",
            Language::Zh => "已丢弃",
        },
    };
    let context = need_queue_context_label(item);
    match language {
        Language::En => println!(
            "#{} [{}] {} {}{}",
            item.index,
            status,
            need_label(&item.need.kind),
            item.need.query,
            context
        ),
        Language::Zh => println!(
            "#{} [{}] {} {}{}",
            item.index,
            status,
            need_label(&item.need.kind),
            item.need.query,
            context
        ),
    }
}

pub(crate) fn write_need_queue_script(
    state: &HarnessState,
    state_path: &Path,
    path: Option<&str>,
) -> Result<NeedQueueScriptReport, String> {
    let queue = state.need_queue_report(1000);
    let script_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| default_need_queue_script_path(state_path));
    if let Some(parent) = script_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let state_arg = shell_arg(state_path.to_string_lossy().as_ref());
    let mut commands = Vec::new();
    let mut lines = vec![
        "#!/usr/bin/env sh".to_string(),
        "set -eu".to_string(),
        String::new(),
        "# Review before running. The clean brain emitted Needs only; this script turns accepted Needs into Feed.".to_string(),
        String::new(),
    ];
    if queue.pending.is_empty() {
        lines.push("# No pending Needs.".to_string());
    }
    for item in &queue.pending {
        let command = format!(
            "octopus --state {state_arg} need {} {}",
            need_label(&item.need.kind),
            shell_arg(&item.need.query)
        );
        let take = format!("octopus --state {state_arg} needs take {}", item.index);
        lines.push(format!("# Need {} from {}", item.index, item.source));
        lines.push(format!("# Summary: {}", item.summary));
        lines.push(command.clone());
        lines.push(take.clone());
        lines.push(String::new());
        commands.push(command);
        commands.push(take);
    }
    fs::write(&script_path, format!("{}\n", lines.join("\n")))
        .map_err(|error| error.to_string())?;
    make_executable(&script_path)?;
    let script = script_path.to_string_lossy().to_string();
    let next = if queue.pending.is_empty() {
        vec!["octopus explore --save \"what should the brain ask next?\"".to_string()]
    } else {
        vec![
            format!("review {}", shell_arg(&script)),
            format!("sh {}", shell_arg(&script)),
            "octopus needs".to_string(),
        ]
    };
    Ok(NeedQueueScriptReport {
        policy: queue.policy,
        script_path: script,
        command_count: commands.len(),
        commands,
        next,
    })
}

pub(crate) fn default_need_queue_script_path(state_path: &Path) -> PathBuf {
    state_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new(".octopus"))
        .join("needs")
        .join("RUN.sh")
}

pub(crate) fn write_need_queue_review_session(
    state: &HarnessState,
    state_path: &Path,
    prompt: &str,
    live: bool,
) -> Result<NeedQueueReviewSessionReport, String> {
    let queue = state.need_queue_report(1000);
    let brain = state.clean_brain_prompt(prompt.to_string(), 6);
    let messages = need_queue_review_messages(&brain, &queue);
    let session_dir = next_need_queue_session_dir(state_path)?;
    fs::create_dir_all(&session_dir).map_err(|error| error.to_string())?;

    let prompt_path = session_dir.join("PROMPT.md");
    let messages_path = session_dir.join("messages.json");
    let review_path = session_dir.join("REVIEW.json");
    let draft_path = session_dir.join("DRAFT.json");
    let command_path = session_dir.join("COMMANDS.sh");
    let state_arg = shell_arg(state_path.to_string_lossy().as_ref());
    let review_arg = shell_arg(review_path.to_string_lossy().as_ref());
    let prompt_arg = shell_arg(prompt);
    let apply_command =
        format!("octopus --state {state_arg} brain --apply {review_arg} --save {prompt_arg}");
    let script_command = format!("octopus --state {state_arg} needs script");
    let review_template = serde_json::json!({
        "summary": "short Need queue review",
        "keep": [1],
        "drop": [],
        "needs": [
            {"kind": "verify", "query": "short cognitive request"}
        ]
    });

    fs::write(
        &prompt_path,
        need_queue_review_prompt_markdown(
            &brain,
            &queue,
            &messages,
            &apply_command,
            &script_command,
        ),
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        &messages_path,
        serde_json::to_string_pretty(&messages).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        &review_path,
        serde_json::to_string_pretty(&review_template).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let draft_path = if live {
        let mut client = clean_brain_queue_llm_client()?;
        let response = client.chat(&messages)?;
        let draft = clean_brain_session_draft_json(&response.content)?;
        fs::write(&draft_path, draft).map_err(|error| error.to_string())?;
        Some(draft_path)
    } else {
        None
    };
    fs::write(
        &command_path,
        format!(
            "#!/usr/bin/env sh\nset -eu\n# Review REVIEW.json before running these commands.\n{apply_command}\n{script_command}\n"
        ),
    )
    .map_err(|error| error.to_string())?;
    make_executable(&command_path)?;
    let draft_path_string = draft_path
        .as_ref()
        .map(|path| path.to_string_lossy().to_string());
    let mut next = Vec::new();
    if draft_path_string.is_some() {
        next.push("review DRAFT.json, then copy accepted JSON into REVIEW.json".to_string());
    } else {
        next.push("paste PROMPT.md or messages.json into a chat model".to_string());
        next.push("replace REVIEW.json with the model JSON reply".to_string());
    }
    next.push(apply_command.clone());
    next.push(script_command.clone());

    Ok(NeedQueueReviewSessionReport {
        policy: queue.policy,
        prompt: prompt.to_string(),
        pending_count: queue.pending.len(),
        session_dir: session_dir.to_string_lossy().to_string(),
        prompt_path: prompt_path.to_string_lossy().to_string(),
        messages_path: messages_path.to_string_lossy().to_string(),
        review_path: review_path.to_string_lossy().to_string(),
        draft_path: draft_path_string,
        command_path: command_path.to_string_lossy().to_string(),
        next,
    })
}

fn need_queue_review_messages(
    brain: &BrainPromptReport,
    queue: &NeedQueueReport,
) -> Vec<ChatMessage> {
    let context = serde_json::json!({
        "policy": brain.policy,
        "slots": ["Goal", "Mem", "Need", "Feed"],
        "goal": brain.goal,
        "mem": brain.mem,
        "recent_need_feed": brain.recent,
        "pending_needs": queue.pending,
    });
    vec![
        ChatMessage::new(
            ChatRole::System,
            "You are the Octopus clean-brain Need queue reviewer. You see only Goal, Mem, Need, Feed, and pending Needs. Keep, drop, or suggest cognitive Needs only. Do not choose tools, APIs, files, commands, routes, tentacles, or implementation. Return only JSON: {\"summary\":\"short queue review\",\"keep\":[1],\"drop\":[2],\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}",
        ),
        ChatMessage::new(
            ChatRole::User,
            format!(
                "Clean brain Need queue review prompt: {}\nClean brain context JSON: {context}",
                brain.prompt
            ),
        ),
    ]
}

fn need_queue_review_prompt_markdown(
    brain: &BrainPromptReport,
    queue: &NeedQueueReport,
    messages: &[ChatMessage],
    apply_command: &str,
    script_command: &str,
) -> String {
    let message_text = messages
        .iter()
        .map(|message| format!("{:?}:\n{}", message.role, message.content))
        .collect::<Vec<_>>()
        .join("\n\n");
    format!(
        "# Octopus Need Queue Review\n\npolicy: {}\nprompt: {}\npending: {}\n\n## Messages\n\n{}\n\n## Reply Schema\n\nPaste accepted model JSON into `REVIEW.json`.\n\n## Apply\n\n```sh\n{}\n{}\n```\n",
        brain.policy,
        brain.prompt,
        queue.pending.len(),
        message_text,
        apply_command,
        script_command
    )
}

fn next_need_queue_session_dir(state_path: &Path) -> Result<PathBuf, String> {
    let root = state_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new(".octopus"))
        .join("needs");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for index in 1..10000 {
        let candidate = root.join(format!("session-{index}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("no available Need queue session slot".to_string())
}
