use super::*;
use crate::field_curriculum::{field_harder_layer_next_action_for_field, select_next_harder_field};
use std::collections::BTreeMap;
use std::path::Path;

impl HarnessState {
    pub fn status_report(&self) -> StatusReport {
        self.status_report_with_state(None)
    }

    pub fn context_report(&self, next_need: Option<Need>, limit: usize) -> ContextReport {
        let limit = limit.max(1);
        let turns = self
            .recent_feed_traces(limit)
            .into_iter()
            .map(|trace| BrainContextTurn {
                need: NeedContext {
                    kind: trace.need_kind,
                    query: trace.need_query,
                },
                feed: FeedContext {
                    status: trace.status,
                    summary: trace.summary,
                },
            })
            .collect::<Vec<_>>();
        let tentacles = self
            .installed_tentacles
            .iter()
            .map(|tentacle| self.tentacle_context_report(tentacle, limit))
            .collect::<Vec<_>>();
        let mut agent_next = vec!["octopus chat \"refine your goal\"".to_string()];
        let mut user_goal_hints =
            vec!["Refine the Goal if the latest Feed changes direction.".to_string()];
        if let Some(need) = &next_need {
            user_goal_hints.push(format!(
                "Review the proposed {} Need: {}",
                kind_key(&need.kind),
                one_line(&need.query)
            ));
            agent_next.push(format!(
                "octopus need {} {}",
                kind_key(&need.kind),
                shell_arg(&need.query)
            ));
            if let Some(tentacle) = tentacles.first() {
                agent_next.push(format!(
                    "octopus think {} {} {}",
                    tentacle.id,
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                ));
            }
        } else {
            user_goal_hints.push("Wait for the next Need or refine the Goal.".to_string());
            agent_next.push("octopus context observe .".to_string());
        }
        agent_next.push("octopus traces".to_string());
        ContextReport {
            brain: BrainContextReport {
                policy: CLEAN_BRAIN_CONTEXT_POLICY.to_string(),
                slots: vec![
                    "Goal".to_string(),
                    "Mem".to_string(),
                    "Need".to_string(),
                    "Feed".to_string(),
                ],
                goal: self.goal.clone(),
                mem: self.memory_context_records(limit),
                turns,
                next_need,
            },
            tentacles,
            hearts: self.status_report().hearts,
            agent_next,
            user_goal_hints: user_goal_hints.clone(),
            next: user_goal_hints,
        }
    }
    pub fn status_report_with_state(&self, state_path: Option<&Path>) -> StatusReport {
        let tentacles = self
            .installed_tentacles
            .iter()
            .map(|tentacle| TentacleStatus {
                id: tentacle.id.clone(),
                name: tentacle.name.clone(),
                brain_kind: tentacle.brain_kind.clone(),
                runtime_kinds: tentacle.runtime_kinds.clone(),
                needs: tentacle.needs.clone(),
                tool_count: tentacle.tool_meta.len().max(tentacle.tools.len()),
                editable: tentacle.editable.clone(),
                evolution_surfaces: tentacle.evolution_surfaces.clone(),
            })
            .collect::<Vec<_>>();
        let active_grants = self
            .grants
            .iter()
            .filter(|grant| grant.status == GrantStatus::Active)
            .map(|grant| grant.id.clone())
            .collect::<Vec<_>>();
        let goal = self.goal.as_ref().map(|goal| GoalSnapshot {
            objective: goal.objective.clone(),
            refinements: goal.constraints.len(),
            status: goal.status.clone(),
            turns: {
                let mut turns = self
                    .goal_turns
                    .iter()
                    .rev()
                    .take(6)
                    .cloned()
                    .collect::<Vec<_>>();
                turns.reverse();
                turns
            },
        });
        let mut warnings = Vec::new();
        if self.installed_tentacles.is_empty() {
            warnings.push("no installed tentacle manifests".to_string());
        }
        if self.goal.is_none() {
            warnings.push("no active goal".to_string());
        }
        if active_grants.is_empty() {
            warnings.push("no active OAuth grants".to_string());
        }
        let pending_need_queue_count = self.pending_need_queue_count();
        let state_args = state_path
            .map(|path| format!(" --state {}", shell_arg(&path.to_string_lossy())))
            .unwrap_or_default();
        let harness_learning = self.harness_learning_summary(&state_args);
        let field_pool = self.field_pool_status_report(state_path);
        let completed_field_pool_next_action = field_pool.as_ref().and_then(|pool| {
            if pool.field_slot_count > 0 && pool.completed_fields == pool.field_slot_count {
                pool.agent_next
                    .iter()
                    .find(|action| action.contains("evolve recommend field-mini-task"))
                    .cloned()
                    .or_else(|| pool.agent_next.last().cloned())
            } else {
                None
            }
        });
        let pending_need = self
            .need_queue
            .iter()
            .find(|item| item.status == NeedQueueStatus::Pending);
        let has_active_parallel_field_goal = self.has_active_parallel_field_goal();
        let agent_next_action = if self.installed_tentacles.is_empty() {
            format!("octopus{state_args} adapt")
        } else if self.goal.is_none() {
            format!("octopus{state_args} chat \"describe your goal\"")
        } else if let Some(item) = pending_need {
            format!("octopus{state_args} needs run {}", item.index)
        } else if let Some(result) = self.latest_unsatisfied_field_verifier_result() {
            let tentacle = self
                .feed_traces
                .iter()
                .rev()
                .find(|trace| trace.index == result.trace_index)
                .and_then(|trace| trace.tentacle.clone())
                .unwrap_or_else(|| format!("field:{}", result.field));
            let reason = result
                .error_category
                .as_deref()
                .unwrap_or("field verifier gap");
            let objective = if field_verifier_error_is_environment_gap(Some(reason)) {
                format!(
                    "adapt {} field runtime to current environment after {reason}; keep missing runtime partial unless fallback evidence is real",
                    result.field
                )
            } else {
                format!("improve {} harness after {reason}", result.field)
            };
            format!(
                "octopus{state_args} evolve recommend {} {}",
                shell_arg(&tentacle),
                shell_arg(&objective)
            )
        } else if let Some(action) = completed_field_pool_next_action {
            action
        } else if self.routes.scores.is_empty() {
            format!("octopus{state_args} need observe .")
        } else if self.has_active_parallel_field_goal() {
            format!(
                "octopus{state_args} evolve parallel --workers 1 {}",
                shell_arg("peer field objectives; open one worker slot from the peer field pool")
            )
        } else if harness_learning.source != "none" {
            harness_learning.next_action.clone()
        } else {
            format!("octopus{state_args} beat 200")
        };
        let user_goal_hint = user_surface::goal_hint(
            !self.installed_tentacles.is_empty(),
            self.goal.is_some(),
            pending_need,
            has_active_parallel_field_goal,
            &harness_learning,
        );
        StatusReport {
            hearts: vec![
                HeartBeat {
                    name: "heartbeat".to_string(),
                    changed: false,
                    summary: "ready".to_string(),
                    data: BTreeMap::new(),
                },
                HeartBeat {
                    name: "memory".to_string(),
                    changed: false,
                    summary: format!("{} memories", self.memory.len()),
                    data: BTreeMap::from([("records".to_string(), self.memory.len().to_string())]),
                },
                HeartBeat {
                    name: "harness".to_string(),
                    changed: false,
                    summary: format!("{} routes", self.routes.scores.len()),
                    data: BTreeMap::from([(
                        "routes".to_string(),
                        self.routes.scores.len().to_string(),
                    )]),
                },
            ],
            memory_count: self.memory.len(),
            route_count: self.routes.scores.len(),
            need_queue_count: pending_need_queue_count,
            feed_trace_count: self.feed_traces.len(),
            field_verifier_result_count: self.field_verifier_results.len(),
            parallel_evolution_run_count: self.parallel_evolution_runs.len(),
            check_history_count: self.check_history.len(),
            repair_outcome_count: self.repair_outcomes.len(),
            evolution_outcome_count: self.evolution_outcomes.len(),
            starter_feedback_count: self.starter_feedback.len(),
            installed_profiles: self.installed_profiles.clone(),
            tentacles,
            goal,
            active_grants,
            last_pet_event: self.last_pet_event.clone(),
            latest_feed_trace: self.feed_traces.last().cloned(),
            latest_field_verifier_result: self.field_verifier_results.last().cloned(),
            latest_parallel_evolution_run: self.parallel_evolution_runs.last().cloned(),
            field_pool,
            latest_need_queue_item: self.need_queue.last().cloned(),
            latest_check: self.check_history.last().cloned(),
            latest_repair_outcome: self.repair_outcomes.last().cloned(),
            latest_evolution_outcome: self.evolution_outcomes.last().cloned(),
            harness_learning,
            latest_starter_feedback: self.starter_feedback.last().cloned(),
            warnings,
            agent_next_action,
            user_goal_hint: user_goal_hint.clone(),
            next_action: user_goal_hint,
        }
    }

    pub fn field_trajectory_report(&self) -> Result<FieldTrajectoryReport, String> {
        self.field_trajectory_report_with_state(None)
    }

    pub fn field_trajectory_report_with_state(
        &self,
        state_path: Option<&Path>,
    ) -> Result<FieldTrajectoryReport, String> {
        let state_args = state_path
            .map(|path| format!(" --state {}", shell_arg(&path.to_string_lossy())))
            .unwrap_or_default();
        let catalog = default_field_pack_catalog()?;
        let mut fields = catalog
            .packs
            .iter()
            .into_iter()
            .map(|pack| pack.id.clone())
            .collect::<Vec<_>>();
        for field in self
            .feed_traces
            .iter()
            .filter_map(|trace| trace_field(trace))
            .chain(
                self.field_verifier_results
                    .iter()
                    .map(|result| result.field.clone()),
            )
            .chain(
                self.parallel_evolution_runs
                    .iter()
                    .flat_map(|run| run.workers.iter().map(|worker| worker.field.clone())),
            )
        {
            push_unique_limited(&mut fields, field, usize::MAX);
        }

        let mut summaries = Vec::new();
        for field in fields {
            let traces = self
                .feed_traces
                .iter()
                .filter(|trace| trace_field(trace).as_deref() == Some(field.as_str()))
                .collect::<Vec<_>>();
            let verifiers = self
                .field_verifier_results
                .iter()
                .filter(|result| result.field == field)
                .collect::<Vec<_>>();
            let latest_verifier = verifiers.last().copied();
            let latest_trace = latest_verifier
                .and_then(|result| {
                    self.feed_traces
                        .iter()
                        .find(|trace| trace.index == result.trace_index)
                })
                .or_else(|| traces.last().copied());
            let latest_parallel_run = self.parallel_evolution_runs.iter().rev().find(|run| {
                run.workers
                    .iter()
                    .any(|worker| worker.field.as_str() == field.as_str())
            });
            let satisfied_verifier_count = verifiers
                .iter()
                .filter(|result| result.status == Status::Satisfied)
                .count();
            let unsatisfied_verifier_count =
                verifiers.len().saturating_sub(satisfied_verifier_count);
            let pack = catalog.packs.iter().find(|pack| pack.id == field);
            let mini_task_count = pack.map(|pack| pack.mini_tasks.len()).unwrap_or_default();
            let satisfied_mini_task_count = pack
                .map(|pack| {
                    pack.mini_tasks
                        .iter()
                        .filter(|task| {
                            self.field_mini_task_status(&field, &task.id) == Some(Status::Satisfied)
                        })
                        .count()
                })
                .unwrap_or_default();
            let selected_mini_task = pack.and_then(|pack| self.next_field_mini_task(pack));
            let next_mini_task = selected_mini_task.map(|task| task.id.clone());
            let next_mini_task_goal = selected_mini_task.map(|task| task.goal.clone());
            let latest_mini_task = latest_trace.and_then(|trace| {
                trace
                    .metadata
                    .get("field_mini_task")
                    .filter(|value| !value.trim().is_empty())
                    .cloned()
            });
            let latest_pass_evidence = latest_trace.and_then(|trace| {
                trace
                    .metadata
                    .get("field_pass_evidence")
                    .filter(|value| !value.trim().is_empty())
                    .cloned()
            });
            let latest_error_category = latest_verifier
                .and_then(|result| result.error_category.clone())
                .or_else(|| {
                    latest_trace.and_then(|trace| {
                        trace
                            .metadata
                            .get("error_category")
                            .filter(|value| !value.trim().is_empty())
                            .cloned()
                    })
                });
            let latest_summary = latest_verifier
                .map(|result| result.summary.clone())
                .or_else(|| latest_trace.map(|trace| trace.summary.clone()));
            let latest_verifier_status = latest_verifier.map(|result| result.status.clone());
            let has_real_verifier_gap = latest_verifier.is_some_and(|result| {
                result.status != Status::Satisfied
                    && self.field_verifier_result_has_real_mini_task(result)
            });
            let needs_environment = has_real_verifier_gap
                && field_verifier_error_is_environment_gap(latest_error_category.as_deref());
            let needs_repair = has_real_verifier_gap && !needs_environment;
            let ready_for_harder_task = latest_verifier_status == Some(Status::Satisfied);
            let has_pending_mini_task = next_mini_task.as_ref().is_some_and(|task| {
                self.field_mini_task_status(&field, task) != Some(Status::Satisfied)
            });
            let field_pack_layer_complete =
                mini_task_count > 0 && satisfied_mini_task_count == mini_task_count;
            let next_action = if needs_environment {
                let tentacle = latest_trace
                    .and_then(|trace| trace.tentacle.clone())
                    .unwrap_or_else(|| "field-mini-task".to_string());
                let reason = latest_error_category
                    .as_deref()
                    .unwrap_or("environment gap");
                format!(
                    "octopus{state_args} evolve recommend {} {}",
                    shell_arg(&tentacle),
                    shell_arg(&format!(
                        "adapt {field} field runtime to current environment after {reason}; keep missing runtime partial unless fallback evidence is real"
                    ))
                )
            } else if needs_repair {
                let tentacle = latest_trace
                    .and_then(|trace| trace.tentacle.clone())
                    .unwrap_or_else(|| "field-mini-task".to_string());
                let reason = latest_error_category
                    .as_deref()
                    .unwrap_or("field verifier gap");
                format!(
                    "octopus{state_args} evolve recommend {} {}",
                    shell_arg(&tentacle),
                    shell_arg(&format!("improve {field} harness after {reason}"))
                )
            } else if field_pack_layer_complete {
                field_harder_layer_next_action_for_field(&state_args, &field)
            } else if ready_for_harder_task || has_pending_mini_task {
                format!(
                    "octopus{state_args} evolve parallel --workers 1 {}",
                    shell_arg(
                        "peer field objectives; open one harder-task worker slot from the peer field pool"
                    )
                )
            } else {
                format!(
                    "octopus{state_args} evolve parallel --workers 1 {}",
                    shell_arg(
                        "peer field objectives; open one worker slot from the peer field pool"
                    )
                )
            };
            summaries.push(FieldTrajectorySummary {
                field,
                mini_task_count,
                satisfied_mini_task_count,
                next_mini_task,
                next_mini_task_goal,
                trace_count: traces.len(),
                verifier_result_count: verifiers.len(),
                satisfied_verifier_count,
                unsatisfied_verifier_count,
                latest_trace_index: latest_trace.map(|trace| trace.index),
                latest_trace_status: latest_trace.map(|trace| trace.status.clone()),
                latest_verifier_result_index: latest_verifier.map(|result| result.index),
                latest_verifier_status,
                latest_parallel_run_index: latest_parallel_run.map(|run| run.index),
                latest_mini_task,
                latest_error_category,
                latest_pass_evidence,
                latest_summary,
                needs_environment,
                needs_repair,
                ready_for_harder_task,
                next_action,
            });
        }

        let active_slot_field = summaries
            .iter()
            .find(|summary| summary.needs_repair)
            .map(|summary| summary.field.clone())
            .or_else(|| {
                summaries
                    .iter()
                    .find(|summary| summary.needs_environment)
                    .map(|summary| summary.field.clone())
            })
            .or_else(|| {
                self.fair_parallel_field_pool(
                    &catalog,
                    self.next_parallel_evolution_run_index,
                    None,
                )
                .into_iter()
                .find(|field| {
                    summaries
                        .iter()
                        .find(|summary| summary.field == *field)
                        .is_some_and(|summary| {
                            let has_pending_mini_task =
                                summary.next_mini_task.as_ref().is_some_and(|task| {
                                    self.field_mini_task_status(&summary.field, task)
                                        != Some(Status::Satisfied)
                                });
                            has_pending_mini_task
                                || summary.latest_verifier_status != Some(Status::Satisfied)
                        })
                })
            })
            .or_else(|| {
                summaries
                    .iter()
                    .find(|summary| !summary.ready_for_harder_task)
                    .map(|summary| summary.field.clone())
            });
        let all_first_pass_satisfied = !catalog.packs.is_empty()
            && catalog.packs.iter().all(|pack| {
                pack.mini_tasks.first().is_some_and(|task| {
                    self.field_mini_task_status(&pack.id, &task.id) == Some(Status::Satisfied)
                })
            });
        let all_pack_tasks_satisfied = !catalog.packs.is_empty()
            && catalog.packs.iter().all(|pack| {
                !pack.mini_tasks.is_empty()
                    && pack.mini_tasks.iter().all(|task| {
                        self.field_mini_task_status(&pack.id, &task.id) == Some(Status::Satisfied)
                    })
            })
            && summaries
                .iter()
                .all(|summary| !summary.needs_repair && !summary.needs_environment);
        let curriculum_step = if all_pack_tasks_satisfied {
            select_next_harder_field(&summaries, &state_args)
        } else {
            None
        };
        let active_slot_field = if all_pack_tasks_satisfied {
            curriculum_step.as_ref().map(|step| step.field.clone())
        } else {
            active_slot_field
        };
        let active_slot_reason = if all_pack_tasks_satisfied {
            curriculum_step
                .as_ref()
                .map(|step| step.reason.clone())
                .unwrap_or_else(|| {
                    "all peer field tasks satisfied; no curriculum field selected".to_string()
                })
        } else if let Some(field) = &active_slot_field {
            summaries
                .iter()
                .find(|summary| summary.field == *field)
                .map(|summary| {
                    if summary.needs_repair {
                        format!("{field} selected by latest verifier failure")
                    } else if summary.needs_environment {
                        format!("{field} selected by latest environment gap")
                    } else {
                        format!("{field} selected by field status and recent-run fairness")
                    }
                })
                .unwrap_or_else(|| format!("{field} selected from the peer field pool"))
        } else {
            "no runnable peer field slot selected".to_string()
        };
        let latest_worker_slot_count = self
            .parallel_evolution_runs
            .last()
            .map(|run| run.worker_count)
            .unwrap_or_default();
        let next = if all_pack_tasks_satisfied {
            let mut actions = vec![format!("octopus{state_args} fields summary")];
            if let Some(step) = &curriculum_step {
                actions.push(step.next_action.clone());
            } else {
                actions.push(field_harder_layer_next_action(&state_args));
            }
            actions
        } else if let Some(field) = &active_slot_field {
            summaries
                .iter()
                .find(|summary| summary.field == *field)
                .map(|summary| vec![summary.next_action.clone()])
                .unwrap_or_default()
        } else {
            vec![format!("octopus{state_args} evolve parallel --workers 1")]
        };
        Ok(FieldTrajectoryReport {
            field_count: summaries.len(),
            latest_worker_slot_count,
            trace_count: self.feed_traces.len(),
            verifier_result_count: self.field_verifier_results.len(),
            parallel_evolution_run_count: self.parallel_evolution_runs.len(),
            all_first_pass_satisfied,
            all_pack_tasks_satisfied,
            active_slot_field,
            active_slot_reason,
            fields: summaries,
            next,
        })
    }

    pub fn field_pool_status_report(
        &self,
        state_path: Option<&Path>,
    ) -> Option<FieldPoolStatusReport> {
        let report = self.field_trajectory_report_with_state(state_path).ok()?;
        let slots = report
            .fields
            .iter()
            .map(|summary| {
                let latest_worker = self
                    .parallel_evolution_runs
                    .iter()
                    .rev()
                    .flat_map(|run| run.workers.iter())
                    .find(|worker| worker.field == summary.field);
                let completed = summary.mini_task_count > 0
                    && summary.satisfied_mini_task_count == summary.mini_task_count
                    && !summary.needs_environment
                    && !summary.needs_repair;
                let user_goal_hint = user_surface::field_slot_hint(
                    &summary.field,
                    completed,
                    summary.next_mini_task.as_deref(),
                    summary.needs_environment,
                    summary.needs_repair,
                );
                FieldPoolSlotReport {
                    field: summary.field.clone(),
                    completed,
                    mini_task_count: summary.mini_task_count,
                    satisfied_mini_task_count: summary.satisfied_mini_task_count,
                    next_mini_task: summary.next_mini_task.clone(),
                    latest_worker_id: latest_worker.map(|worker| worker.id.clone()),
                    latest_mini_task: latest_worker
                        .and_then(|worker| worker.mini_task.clone())
                        .or_else(|| summary.latest_mini_task.clone()),
                    latest_worker_status: latest_worker.map(|worker| worker.status.clone()),
                    latest_status: summary
                        .latest_verifier_status
                        .clone()
                        .or_else(|| summary.latest_trace_status.clone())
                        .or_else(|| latest_worker.map(|worker| worker.status.clone())),
                    latest_parallel_run_index: summary.latest_parallel_run_index,
                    latest_updated_at_secs: latest_worker
                        .map(|worker| worker.updated_at_secs)
                        .unwrap_or_default(),
                    needs_environment: summary.needs_environment,
                    needs_repair: summary.needs_repair,
                    agent_next_action: summary.next_action.clone(),
                    user_goal_hint: user_goal_hint.clone(),
                    next_action: user_goal_hint,
                }
            })
            .collect::<Vec<_>>();
        let completed_fields = slots.iter().filter(|slot| slot.completed).count();
        let user_goal_hints = user_surface::field_pool_hints(
            report.field_count,
            completed_fields,
            report.active_slot_field.as_deref(),
        );
        Some(FieldPoolStatusReport {
            policy: parallel_field_pool_policy().to_string(),
            field_count: report.field_count,
            field_slot_count: report.field_count,
            latest_worker_slot_count: report.latest_worker_slot_count,
            completed_fields,
            active_slot_field: report.active_slot_field,
            active_slot_reason: report.active_slot_reason,
            worker_slots: parallel_worker_policy().to_string(),
            slots,
            agent_next: report.next,
            user_goal_hints: user_goal_hints.clone(),
            next: user_goal_hints,
        })
    }

    fn harness_learning_summary(&self, state_args: &str) -> HarnessLearningSummary {
        let prefer_evolution = self
            .last_pet_event
            .as_ref()
            .is_some_and(|event| event.source == "evolve score");
        if prefer_evolution {
            if let Some(summary) = self.evolution_learning_summary(state_args) {
                return summary;
            }
        }
        if let Some(outcome) = self.repair_outcomes.last() {
            let target = outcome
                .target_tentacle
                .clone()
                .unwrap_or_else(|| outcome.tentacle_id.clone());
            let candidate = outcome.candidate.clone();
            let next_action = if !target.trim().is_empty() {
                format!(
                    "octopus{state_args} evolve recommend {}",
                    shell_arg(&target)
                )
            } else {
                format!("octopus{state_args} beat 200")
            };
            return HarnessLearningSummary {
                source: "repair_outcome".to_string(),
                repair_outcomes: self.repair_outcomes.len(),
                evolution_outcomes: self.evolution_outcomes.len(),
                target_tentacle: Some(target),
                candidate,
                status: Some(outcome.status.clone()),
                score: Some(outcome.score),
                summary: Some(outcome.summary.clone()),
                next_action,
            };
        }
        if let Some(summary) = self.evolution_learning_summary(state_args) {
            return summary;
        }
        HarnessLearningSummary {
            source: "none".to_string(),
            repair_outcomes: 0,
            evolution_outcomes: 0,
            target_tentacle: None,
            candidate: None,
            status: None,
            score: None,
            summary: None,
            next_action: format!("octopus{state_args} beat 200"),
        }
    }

    fn evolution_learning_summary(&self, state_args: &str) -> Option<HarnessLearningSummary> {
        let outcome = self.evolution_outcomes.last()?;
        Some(HarnessLearningSummary {
            source: "evolution_outcome".to_string(),
            repair_outcomes: self.repair_outcomes.len(),
            evolution_outcomes: self.evolution_outcomes.len(),
            target_tentacle: Some(outcome.tentacle_id.clone()),
            candidate: Some(outcome.candidate_id.clone()),
            status: Some(outcome.status.clone()),
            score: Some(outcome.score),
            summary: Some(outcome.summary.clone()),
            next_action: format!(
                "octopus{state_args} evolve recommend {}",
                shell_arg(&outcome.tentacle_id)
            ),
        })
    }

    fn memory_context_records(&self, limit: usize) -> Vec<MemoryContextRecord> {
        let mut records = self.memory.records.values().cloned().collect::<Vec<_>>();
        records.sort_by(|left, right| {
            right
                .weight
                .total_cmp(&left.weight)
                .then_with(|| memory_id_number(&right.id).cmp(&memory_id_number(&left.id)))
        });
        records
            .into_iter()
            .take(limit)
            .map(|record| MemoryContextRecord {
                id: record.id,
                text: short_text(&record.text, FEED_TRACE_SUMMARY_BYTES),
                weight: record.weight,
            })
            .collect()
    }

    fn tentacle_context_report(
        &self,
        tentacle: &InstalledTentacle,
        limit: usize,
    ) -> TentacleContextReport {
        let tools = installed_tentacle_tools(tentacle)
            .into_iter()
            .map(|tool| self.tool_context(&tool))
            .collect::<Vec<_>>();
        let recent_actions = self
            .recent_feed_traces_for_tentacle(&tentacle.id, limit)
            .into_iter()
            .map(|trace| TentacleActionContext {
                index: trace.index,
                need: NeedContext {
                    kind: trace.need_kind,
                    query: trace.need_query,
                },
                tool: trace.tool,
                plan_source: trace.plan_source,
                status: trace.status,
                summary: trace.summary,
            })
            .collect::<Vec<_>>();
        TentacleContextReport {
            id: tentacle.id.clone(),
            brain_kind: tentacle.brain_kind.clone(),
            policy: TENTACLE_CONTEXT_POLICY.to_string(),
            slots: vec![
                "Need".to_string(),
                "Tool".to_string(),
                "Action".to_string(),
                "Feed".to_string(),
            ],
            tools,
            recent_actions,
        }
    }

    fn tool_context(&self, tool: &InstalledToolRef) -> ToolContext {
        let active_grant = active_grant_for_tool(tool, &self.grants).map(|grant| grant.id.clone());
        ToolContext {
            id: tool.id.clone(),
            description: tool.description.clone(),
            runtime: tool.kind.clone(),
            entrypoint: tool.entrypoint.clone(),
            contract: tool.contract.clone(),
            permission: tool.permission.as_ref().map(tool_permission_text),
            authorization_required: tool.permission.is_some(),
            active_grant,
        }
    }
}

fn field_verifier_error_is_environment_gap(error_category: Option<&str>) -> bool {
    matches!(error_category, Some("go_runtime_missing"))
}
