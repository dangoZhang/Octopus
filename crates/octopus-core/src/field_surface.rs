use super::*;

#[derive(Debug, serde::Serialize)]
pub(crate) struct FieldMatchReport {
    pub(crate) need: Need,
    pub(crate) selection: Option<FieldPackSelection>,
    pub(crate) pack_count: usize,
    pub(crate) next: Vec<String>,
}

pub(crate) fn handle_fields_command(
    rest: &[String],
    state: &Path,
    json: bool,
    language: Language,
) -> Result<(), String> {
    if rest.get(1).map(String::as_str) == Some("score") {
        let mut loaded = HarnessState::load(state).map_err(|error| error.to_string())?;
        let trace_index = rest
            .get(2)
            .ok_or_else(|| "fields score requires a feed trace index or latest".to_string())
            .and_then(|value| resolve_feed_trace_selector(value, &loaded))?;
        let status = rest
            .get(3)
            .ok_or_else(|| "fields score requires a status".to_string())
            .and_then(|value| parse_status(value))?;
        let error_category = rest.get(4).filter(|value| value.as_str() != "-").cloned();
        let summary = rest
            .get(5..)
            .filter(|values| !values.is_empty())
            .map(|values| values.join(" "))
            .unwrap_or_else(|| "recorded field verifier result".to_string());
        let result = loaded.record_field_verifier_result(
            trace_index,
            status,
            error_category,
            None,
            summary,
        )?;
        loaded.save(state).map_err(|error| error.to_string())?;
        pet_events::append_latest(state, &loaded)?;
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&result).map_err(|error| error.to_string())?
            );
        } else {
            print_field_verifier_result(&result, language);
        }
    } else if rest.get(1).map(String::as_str) == Some("summary") {
        let loaded = HarnessState::load(state).map_err(|error| error.to_string())?;
        let report = loaded.field_trajectory_report_with_state(Some(state))?;
        loaded.save(state).map_err(|error| error.to_string())?;
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_field_trajectory_report(&report, language);
        }
    } else if rest.get(1).map(String::as_str) == Some("match") {
        let kind = rest
            .get(2)
            .ok_or_else(|| "fields match requires a need kind".to_string())
            .and_then(|value| parse_kind(value))?;
        let query = rest
            .get(3..)
            .filter(|values| !values.is_empty())
            .map(|values| values.join(" "))
            .ok_or_else(|| "fields match requires a query".to_string())?;
        let catalog = default_field_pack_catalog()?;
        let need = Need::new(kind, query);
        let report = FieldMatchReport {
            selection: select_field_pack(&catalog.packs, None, &need),
            need,
            pack_count: catalog.packs.len(),
            next: vec![
                "octopus need <kind> <query>".to_string(),
                "octopus traces 10".to_string(),
            ],
        };
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_field_match_report(&report, language);
        }
    } else {
        let root = rest.get(1).map(PathBuf::from);
        let report = field_pack_report(root.as_deref());
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_field_pack_report(&report, language);
        }
    }
    Ok(())
}

pub(crate) fn print_field_pack_report(report: &FieldPackReport, language: Language) {
    match language {
        Language::En => {
            println!("Field packs");
            println!("source: {}", report.source);
            if let Some(root) = &report.root {
                println!("root: {root}");
            }
            println!("packs: {}", report.pack_count);
            for pack in &report.packs {
                println!("- {} {} :: {}", pack.id, pack.version, pack.description);
                println!(
                    "  aliases={} hints={} verifier={} mini_tasks={}",
                    join_or_none(&pack.aliases),
                    join_or_none(&pack.capability_hints),
                    pack.verifier_method,
                    pack.mini_tasks
                );
            }
            for error in &report.errors {
                println!("error: {error}");
            }
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("领域包");
            println!("来源: {}", report.source);
            if let Some(root) = &report.root {
                println!("路径: {root}");
            }
            println!("数量: {}", report.pack_count);
            for pack in &report.packs {
                println!("- {} {} :: {}", pack.id, pack.version, pack.description);
                println!(
                    "  别名={} 能力={} 验证={} mini_tasks={}",
                    join_or_none(&pack.aliases),
                    join_or_none(&pack.capability_hints),
                    pack.verifier_method,
                    pack.mini_tasks
                );
            }
            for error in &report.errors {
                println!("错误: {error}");
            }
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

pub(crate) fn print_field_match_report(report: &FieldMatchReport, language: Language) {
    match language {
        Language::En => {
            println!(
                "Field match: {} {}",
                need_label(&report.need.kind),
                report.need.query
            );
            if let Some(selection) = &report.selection {
                println!(
                    "selected: {} score={:.2} reason={}",
                    selection.field, selection.score, selection.reason
                );
                println!(
                    "verifier: {} | pass={}",
                    selection.verifier_method, selection.pass_signal
                );
            } else {
                println!("selected: none");
            }
            println!("packs: {}", report.pack_count);
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!(
                "领域匹配: {} {}",
                need_label(&report.need.kind),
                report.need.query
            );
            if let Some(selection) = &report.selection {
                println!(
                    "已选择: {} 分数={:.2} 原因={}",
                    selection.field, selection.score, selection.reason
                );
                println!(
                    "验证: {} | 通过信号={}",
                    selection.verifier_method, selection.pass_signal
                );
            } else {
                println!("已选择: 无");
            }
            println!("领域包: {}", report.pack_count);
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

pub(crate) fn print_field_trajectory_report(report: &FieldTrajectoryReport, language: Language) {
    match language {
        Language::En => {
            println!("Field trajectory summary");
            println!("{}", field_trajectory_policy_line(language));
            println!("fields: {}", report.field_count);
            println!("{}", field_trajectory_worker_slot_line(report, language));
            println!("traces: {}", report.trace_count);
            println!("verifier_results: {}", report.verifier_result_count);
            println!(
                "first_pass: {}",
                if report.all_first_pass_satisfied {
                    "satisfied"
                } else {
                    "incomplete"
                }
            );
            println!(
                "all_pack_tasks: {}",
                if report.all_pack_tasks_satisfied {
                    "satisfied"
                } else {
                    "incomplete"
                }
            );
            for field in &report.fields {
                println!("{}", render_field_trajectory_summary_line(field, language));
            }
            println!(
                "active_slot_field: {}",
                option_or_none(&report.active_slot_field)
            );
            println!("active_slot_reason: {}", report.active_slot_reason);
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("领域轨迹汇总");
            println!("{}", field_trajectory_policy_line(language));
            println!("领域数: {}", report.field_count);
            println!("{}", field_trajectory_worker_slot_line(report, language));
            println!("轨迹数: {}", report.trace_count);
            println!("验证结果数: {}", report.verifier_result_count);
            println!(
                "首轮状态: {}",
                if report.all_first_pass_satisfied {
                    "已通过"
                } else {
                    "未完成"
                }
            );
            println!(
                "全部任务: {}",
                if report.all_pack_tasks_satisfied {
                    "已通过"
                } else {
                    "未完成"
                }
            );
            for field in &report.fields {
                println!("{}", render_field_trajectory_summary_line(field, language));
            }
            println!(
                "当前活动槽领域: {}",
                option_or_none(&report.active_slot_field)
            );
            println!("当前活动槽原因: {}", report.active_slot_reason);
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

pub(crate) fn field_trajectory_policy_line(language: Language) -> &'static str {
    match language {
        Language::En => "pool_policy: peer field slots; workers are execution capacity only",
        Language::Zh => "领域池策略: 并列领域槽；workers 只表示执行容量",
    }
}

pub(crate) fn field_trajectory_worker_slot_line(
    report: &FieldTrajectoryReport,
    language: Language,
) -> String {
    match language {
        Language::En => format!("latest_worker_slots: {}", report.latest_worker_slot_count),
        Language::Zh => format!("最新 worker 执行槽数: {}", report.latest_worker_slot_count),
    }
}

pub(crate) fn render_field_trajectory_summary_line(
    field: &FieldTrajectorySummary,
    language: Language,
) -> String {
    let status = field
        .latest_verifier_status
        .as_ref()
        .or(field.latest_trace_status.as_ref())
        .map(|status| format!("{status:?}"))
        .unwrap_or_else(|| "None".to_string());
    let latest_trace = field
        .latest_trace_index
        .map(|index| format!("#{index}"))
        .unwrap_or_else(|| match language {
            Language::En => "none".to_string(),
            Language::Zh => "无".to_string(),
        });
    let latest_verifier = field
        .latest_verifier_result_index
        .map(|index| format!("#{index}"))
        .unwrap_or_else(|| match language {
            Language::En => "none".to_string(),
            Language::Zh => "无".to_string(),
        });
    match language {
        Language::En => format!(
            "- {} status={} tasks={}/{} next_task={} goal={} traces={} verifiers={} latest_trace={} latest_verifier={} latest_task={} next={}",
            field.field,
            status,
            field.satisfied_mini_task_count,
            field.mini_task_count,
            field.next_mini_task.as_deref().unwrap_or("none"),
            field.next_mini_task_goal.as_deref().unwrap_or("none"),
            field.trace_count,
            field.verifier_result_count,
            latest_trace,
            latest_verifier,
            field.latest_mini_task.as_deref().unwrap_or("none"),
            field.next_action
        ),
        Language::Zh => format!(
            "- {} 状态={} 任务={}/{} 下一题={} 目标={} 轨迹={} 验证={} 最新轨迹={} 最新验证={} 最新题={} 下一步={}",
            field.field,
            status,
            field.satisfied_mini_task_count,
            field.mini_task_count,
            field.next_mini_task.as_deref().unwrap_or("无"),
            field.next_mini_task_goal.as_deref().unwrap_or("无"),
            field.trace_count,
            field.verifier_result_count,
            latest_trace,
            latest_verifier,
            field.latest_mini_task.as_deref().unwrap_or("无"),
            field.next_action
        ),
    }
}

pub(crate) fn print_field_verifier_result(result: &FieldVerifierResult, language: Language) {
    match language {
        Language::En => {
            println!("Field verifier result");
            println!("field: {}", result.field);
            println!("trace: #{}", result.trace_index);
            println!("status: {:?}", result.status);
            if let Some(category) = &result.error_category {
                println!("error_category: {category}");
            }
            if let Some(artifact) = &result.artifact {
                println!("artifact: {artifact}");
            }
            println!("summary: {}", result.summary);
        }
        Language::Zh => {
            println!("领域验证结果");
            println!("领域: {}", result.field);
            println!("轨迹: #{}", result.trace_index);
            println!("状态: {:?}", result.status);
            if let Some(category) = &result.error_category {
                println!("错误类别: {category}");
            }
            if let Some(artifact) = &result.artifact {
                println!("证据文件: {artifact}");
            }
            println!("摘要: {}", result.summary);
        }
    }
}

pub(crate) fn print_parallel_evolution_run(
    run: &ParallelEvolutionRun,
    desktop_pet: Option<&DesktopPetReport>,
    language: Language,
) {
    match language {
        Language::En => {
            println!("Octopus automatic field evolution");
            println!("run: #{}", run.index);
            println!("{}", parallel_worker_slot_line(run.worker_count, language));
            println!(
                "requested_worker_slots: {}",
                run.requested_worker_count.max(run.worker_count)
            );
            println!("candidate_pool: {}", join_or_none(&run.candidate_fields));
            println!("summary: {}", run.summary);
            for worker in &run.workers {
                println!(
                    "- {} field={} task={} need={} trace={} verifier={} :: {}",
                    worker.id,
                    worker.field,
                    worker.mini_task.as_deref().unwrap_or("none"),
                    worker
                        .queued_need_index
                        .map(|index| format!("#{index}"))
                        .unwrap_or_else(|| "none".to_string()),
                    worker
                        .source_trace_index
                        .map(|index| format!("#{index}"))
                        .unwrap_or_else(|| "none".to_string()),
                    worker
                        .verifier_result_index
                        .map(|index| format!("#{index}"))
                        .unwrap_or_else(|| "none".to_string()),
                    worker.next_action
                );
            }
            if let Some(report) = desktop_pet {
                println!("desktop_pet: {}", report.app_path);
                println!("observing: {}", report.state_path);
            } else {
                println!("desktop_pet: octopus pet desktop");
            }
        }
        Language::Zh => {
            println!("章鱼自动领域进化");
            println!("run: #{}", run.index);
            println!("{}", parallel_worker_slot_line(run.worker_count, language));
            println!(
                "请求 worker 执行槽数: {}",
                run.requested_worker_count.max(run.worker_count)
            );
            println!("候选领域池: {}", join_or_none(&run.candidate_fields));
            println!("摘要: {}", run.summary);
            for worker in &run.workers {
                println!(
                    "- {} 领域={} 任务={} Need={} 轨迹={} 验证={} :: {}",
                    worker.id,
                    worker.field,
                    worker.mini_task.as_deref().unwrap_or("无"),
                    worker
                        .queued_need_index
                        .map(|index| format!("#{index}"))
                        .unwrap_or_else(|| "无".to_string()),
                    worker
                        .source_trace_index
                        .map(|index| format!("#{index}"))
                        .unwrap_or_else(|| "无".to_string()),
                    worker
                        .verifier_result_index
                        .map(|index| format!("#{index}"))
                        .unwrap_or_else(|| "无".to_string()),
                    worker.next_action
                );
            }
            if let Some(report) = desktop_pet {
                println!("桌宠: {}", report.app_path);
                println!("观察状态: {}", report.state_path);
            } else {
                println!("桌宠: octopus pet desktop");
            }
        }
    }
}

pub(crate) fn parallel_worker_slot_line(worker_count: usize, language: Language) -> String {
    match language {
        Language::En => {
            format!("worker_slots: {worker_count} execution slot(s) from the peer field pool")
        }
        Language::Zh => format!("worker 执行槽数: {worker_count}，来自并列领域池"),
    }
}

pub(crate) fn field_pool_status_line(report: &FieldPoolStatusReport) -> String {
    let active = report.active_slot_field.as_deref().unwrap_or("none");
    let slots = report
        .slots
        .iter()
        .map(|slot| {
            let active_marker = if report.active_slot_field.as_deref() == Some(slot.field.as_str())
            {
                "*"
            } else {
                ""
            };
            let next = slot.next_mini_task.as_deref().unwrap_or("done");
            format!(
                "{active_marker}{}:{}/{}:{}",
                slot.field, slot.satisfied_mini_task_count, slot.mini_task_count, next
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let latest_activity = field_pool_latest_activity_line(report);
    format!(
        "{} field slots ({} peer slots), latest_worker_slots={}, latest_activity={}, {} complete, active={}, reason={}, workers={}, [{}]",
        report.field_slot_count,
        report.field_count,
        report.latest_worker_slot_count,
        latest_activity,
        report.completed_fields,
        active,
        report.active_slot_reason,
        report.worker_slots,
        slots
    )
}

pub(crate) fn parallel_run_status_line(run: &ParallelEvolutionRun, language: Language) -> String {
    let candidate_pool = join_or_none(&run.candidate_fields);
    match language {
        Language::En => format!(
            "#{} requested_worker_slots={}, active_worker_slots={}, candidate_pool={}, workers={}",
            run.index,
            run.requested_worker_count,
            run.worker_count,
            candidate_pool,
            run.worker_policy
        ),
        Language::Zh => format!(
            "#{} 请求worker执行槽={}, 活动worker执行槽={}, 候选领域池={}, worker策略={}",
            run.index,
            run.requested_worker_count,
            run.worker_count,
            candidate_pool,
            run.worker_policy
        ),
    }
}

pub(crate) fn field_pool_latest_activity_line(report: &FieldPoolStatusReport) -> String {
    let Some(slot) = report
        .slots
        .iter()
        .filter(|slot| slot.latest_updated_at_secs > 0)
        .max_by_key(|slot| slot.latest_updated_at_secs)
    else {
        return "none".to_string();
    };
    let task = slot
        .latest_mini_task
        .as_deref()
        .or(slot.next_mini_task.as_deref())
        .unwrap_or("done");
    let status = slot
        .latest_status
        .as_ref()
        .or(slot.latest_worker_status.as_ref())
        .map(|status| format!("{status:?}"))
        .unwrap_or_else(|| "unknown".to_string());
    let worker = slot
        .latest_worker_id
        .as_deref()
        .map(|worker| format!(" worker={worker}"))
        .unwrap_or_default();
    format!(
        "{}:{}:{}@{}{}",
        slot.field, task, status, slot.latest_updated_at_secs, worker
    )
}
