use super::*;

pub(crate) fn handle_routes_command(
    rest: &[String],
    state: &Path,
    json: bool,
    language: Language,
) -> Result<(), String> {
    let loaded = HarnessState::load(state).map_err(|error| error.to_string())?;
    if let Some(kind_value) = rest.get(1) {
        let kind = parse_kind(kind_value)?;
        let query = if rest.len() > 2 {
            rest[2..].join(" ")
        } else {
            ".".to_string()
        };
        let harness = harness_for_need(loaded, &kind)?;
        let report = harness.route_report(&Need::new(kind, query));
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_route_report(&report, language);
        }
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&loaded.routes.scores)
                .map_err(|error| error.to_string())?
        );
    }
    Ok(())
}

pub(crate) fn handle_traces_command(
    rest: &[String],
    state: &Path,
    json: bool,
    language: Language,
) -> Result<(), String> {
    let limit = rest
        .get(1)
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| format!("invalid trace limit: {value}"))
        })
        .transpose()?
        .unwrap_or(10);
    let loaded = HarnessState::load(state).map_err(|error| error.to_string())?;
    let traces = loaded.recent_feed_traces(limit);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&traces).map_err(|error| error.to_string())?
        );
    } else {
        print_feed_traces(&traces, language);
    }
    Ok(())
}

pub(crate) fn print_feed_traces(traces: &[FeedTraceRecord], language: Language) {
    match language {
        Language::En => println!("Feed traces"),
        Language::Zh => println!("Feed轨迹"),
    }
    if traces.is_empty() {
        match language {
            Language::En => println!("none"),
            Language::Zh => println!("暂无"),
        }
        return;
    }
    for trace in traces {
        println!("{}", trace_line(trace));
    }
}

pub(crate) fn print_route_report(report: &RouteReport, language: Language) {
    match language {
        Language::En => {
            println!(
                "Route report: {} {}",
                need_label(&report.need.kind),
                report.need.query
            );
            if let Some(selected) = &report.selected {
                println!(
                    "selected: {} score={:.2} reason={}",
                    selected.tentacle, selected.score, selected.reason
                );
            } else {
                println!("selected: none");
            }
            for option in &report.candidates {
                let marker = if option.selected { "*" } else { "-" };
                println!(
                    "{marker} {} score={:.2} supported={} traces={} :: {}",
                    option.tentacle,
                    option.score,
                    option.supported,
                    option.trace_count,
                    option.reason
                );
                if let Some(trace) = &option.last_trace {
                    println!("  last_trace: {}", trace_line(trace));
                }
            }
            for next in &report.next {
                println!("next: {next}");
            }
        }
        Language::Zh => {
            println!(
                "路由报告: {} {}",
                need_label(&report.need.kind),
                report.need.query
            );
            if let Some(selected) = &report.selected {
                println!(
                    "已选择: {} 分数={:.2} 原因={}",
                    selected.tentacle, selected.score, selected.reason
                );
            } else {
                println!("已选择: 无");
            }
            for option in &report.candidates {
                let marker = if option.selected { "*" } else { "-" };
                println!(
                    "{marker} {} 分数={:.2} 支持={} 轨迹={} :: {}",
                    option.tentacle,
                    option.score,
                    option.supported,
                    option.trace_count,
                    option.reason
                );
                if let Some(trace) = &option.last_trace {
                    println!("  最近轨迹: {}", trace_line(trace));
                }
            }
            for next in &report.next {
                println!("下一步: {next}");
            }
        }
    }
}

pub(crate) fn trace_line(trace: &FeedTraceRecord) -> String {
    let field = trace.field.as_deref().unwrap_or("none");
    let tentacle = trace.tentacle.as_deref().unwrap_or("none");
    let tool = trace.tool.as_deref().unwrap_or("none");
    let plan = trace.plan_source.as_deref().unwrap_or("none");
    format!(
        "#{} {}:{} -> {:?} field={} via {}/{} plan={} evidence={} :: {}",
        trace.index,
        need_label(&trace.need_kind),
        trace.need_query,
        trace.status,
        field,
        tentacle,
        tool,
        plan,
        trace.evidence_count,
        trace.summary
    )
}

pub(crate) fn need_queue_line(item: &NeedQueueItem) -> String {
    format!(
        "#{} {:?} {}:{}{}",
        item.index,
        item.status,
        need_label(&item.need.kind),
        item.need.query,
        need_queue_context_label(item)
    )
}

pub(crate) fn need_queue_context_label(item: &NeedQueueItem) -> String {
    let mut parts = Vec::new();
    if let Some(field) = item
        .context
        .get("field_pack")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        parts.push(format!("field={field}"));
    }
    if let Some(task) = item
        .context
        .get("field_mini_task")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        parts.push(format!("task={task}"));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" [{}]", parts.join(" "))
    }
}
