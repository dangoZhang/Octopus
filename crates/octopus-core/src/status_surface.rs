use super::*;

pub(crate) fn print_status_report(report: &StatusReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus status");
            println!(
                "hearts: {}",
                report
                    .hearts
                    .iter()
                    .map(|beat| format!("{}={}", beat.name, beat.summary))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!("profiles: {}", join_or_none(&report.installed_profiles));
            println!("tentacles: {}", status_tentacles(report));
            println!("goal: {}", status_goal(report));
            println!("grants: {}", join_or_none(&report.active_grants));
            println!("need_queue: {}", report.need_queue_count);
            if let Some(item) = &report.latest_need_queue_item {
                println!("latest_need: {}", need_queue_line(item));
            }
            println!("feed_traces: {}", report.feed_trace_count);
            if let Some(trace) = &report.latest_feed_trace {
                println!("latest_trace: {}", trace_line(trace));
            }
            if let Some(field_pool) = &report.field_pool {
                println!("field_pool: {}", field_pool_status_line(field_pool));
            }
            if let Some(run) = &report.latest_parallel_evolution_run {
                println!("parallel_run: {}", parallel_run_status_line(run, language));
            }
            println!("check_history: {}", report.check_history_count);
            if let Some(record) = &report.latest_check {
                println!("latest_check: {}", check_history_line(record));
            }
            println!("repair_outcomes: {}", report.repair_outcome_count);
            if let Some(outcome) = &report.latest_repair_outcome {
                println!(
                    "latest_repair: #{} {:?} {}",
                    outcome.index, outcome.status, outcome.summary
                );
            }
            println!(
                "harness_learning: {}",
                harness_learning_line(&report.harness_learning)
            );
            println!("warnings: {}", join_or_none(&report.warnings));
            println!("goal_hint: {}", report.user_goal_hint);
        }
        Language::Zh => {
            println!("章鱼状态");
            println!(
                "心脏: {}",
                report
                    .hearts
                    .iter()
                    .map(|beat| format!("{}={}", localize_beat(&beat.name), beat.summary))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!("配置: {}", join_or_none(&report.installed_profiles));
            println!("触手: {}", status_tentacles(report));
            println!("目标: {}", status_goal(report));
            println!("授权: {}", join_or_none(&report.active_grants));
            println!("Need队列: {}", report.need_queue_count);
            if let Some(item) = &report.latest_need_queue_item {
                println!("最近Need: {}", need_queue_line(item));
            }
            println!("Feed轨迹数: {}", report.feed_trace_count);
            if let Some(trace) = &report.latest_feed_trace {
                println!("最近轨迹: {}", trace_line(trace));
            }
            if let Some(field_pool) = &report.field_pool {
                println!("领域池: {}", field_pool_status_line(field_pool));
            }
            if let Some(run) = &report.latest_parallel_evolution_run {
                println!("并行领域运行: {}", parallel_run_status_line(run, language));
            }
            println!("检查历史数: {}", report.check_history_count);
            if let Some(record) = &report.latest_check {
                println!("最近检查: {}", check_history_line(record));
            }
            println!("repair结果数: {}", report.repair_outcome_count);
            if let Some(outcome) = &report.latest_repair_outcome {
                println!(
                    "最近repair: #{} {:?} {}",
                    outcome.index, outcome.status, outcome.summary
                );
            }
            println!(
                "Harness学习: {}",
                harness_learning_line(&report.harness_learning)
            );
            println!("警告: {}", join_or_none(&report.warnings));
            println!("目标提示: {}", report.user_goal_hint);
        }
    }
}

pub(crate) fn harness_learning_line(summary: &HarnessLearningSummary) -> String {
    let target = summary.target_tentacle.as_deref().unwrap_or("none");
    let candidate = summary.candidate.as_deref().unwrap_or("none");
    let status = summary
        .status
        .as_ref()
        .map(|status| format!("{status:?}"))
        .unwrap_or_else(|| "none".to_string());
    let score = summary
        .score
        .map(|score| format!("{score:.2}"))
        .unwrap_or_else(|| "none".to_string());
    let text = summary.summary.as_deref().unwrap_or("no feedback yet");
    format!(
        "{} repair={} evolve={} target={} candidate={} status={} score={} summary={}",
        summary.source,
        summary.repair_outcomes,
        summary.evolution_outcomes,
        target,
        candidate,
        status,
        score,
        truncate_chars(&compact_status_line(text), 120)
    )
}

pub(crate) fn compact_status_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn print_context_report(report: &ContextReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus context");
            println!("brain: {}", report.brain.policy);
            println!(
                "goal: {}",
                report
                    .brain
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("none")
            );
            println!("mem: {}", report.brain.mem.len());
            if let Some(need) = &report.brain.next_need {
                println!("next_need: {} {}", need_label(&need.kind), need.query);
            }
            print_context_turns(report, language);
            print_context_tentacles(report, language);
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼上下文");
            println!("主脑: {}", report.brain.policy);
            println!(
                "目标: {}",
                report
                    .brain
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("无")
            );
            println!("记忆: {}", report.brain.mem.len());
            if let Some(need) = &report.brain.next_need {
                println!("下个Need: {} {}", need_label(&need.kind), need.query);
            }
            print_context_turns(report, language);
            print_context_tentacles(report, language);
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

pub(crate) fn print_context_turns(report: &ContextReport, language: Language) {
    match language {
        Language::En => println!("recent Need/Feed:"),
        Language::Zh => println!("最近Need/Feed:"),
    }
    if report.brain.turns.is_empty() {
        match language {
            Language::En => println!("- none"),
            Language::Zh => println!("- 暂无"),
        }
        return;
    }
    for turn in &report.brain.turns {
        println!(
            "- {}:{} -> {:?} :: {}",
            need_label(&turn.need.kind),
            turn.need.query,
            turn.feed.status,
            turn.feed.summary
        );
    }
}

pub(crate) fn print_context_tentacles(report: &ContextReport, language: Language) {
    match language {
        Language::En => println!("tentacles:"),
        Language::Zh => println!("触手:"),
    }
    if report.tentacles.is_empty() {
        match language {
            Language::En => println!("- none installed"),
            Language::Zh => println!("- 暂未安装"),
        }
        return;
    }
    for tentacle in &report.tentacles {
        let tools = tentacle
            .tools
            .iter()
            .map(|tool| tool.id.clone())
            .collect::<Vec<_>>();
        println!(
            "- {}: {} | tools={}",
            tentacle.id,
            tentacle.policy,
            join_or_none(&tools)
        );
        if let Some(action) = tentacle.recent_actions.last() {
            println!(
                "  latest: #{} {} -> {:?} {}",
                action.index,
                action.tool.as_deref().unwrap_or("no tool"),
                action.status,
                action.summary
            );
        }
    }
}

pub(crate) fn status_tentacles(report: &StatusReport) -> String {
    let items = report
        .tentacles
        .iter()
        .map(|tentacle| {
            format!(
                "{}({}; {} tools)",
                tentacle.id,
                tentacle.runtime_kinds.join(","),
                tentacle.tool_count
            )
        })
        .collect::<Vec<_>>();
    join_or_none(&items)
}

pub(crate) fn status_goal(report: &StatusReport) -> String {
    report.goal.as_ref().map_or_else(
        || "none".to_string(),
        |goal| format!("{} ({} refinements)", goal.objective, goal.refinements),
    )
}
