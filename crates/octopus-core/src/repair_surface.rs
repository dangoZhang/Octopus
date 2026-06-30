use super::*;

#[derive(Debug, serde::Serialize)]
pub(crate) struct RepairReport {
    pub(crate) state_path: String,
    pub(crate) tentacle: String,
    pub(crate) feed: Feed,
    pub(crate) repair_plan: Option<RepairPlanReport>,
    pub(crate) queued: Vec<NeedQueueItem>,
    pub(crate) queue: NeedQueueReport,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct RepairContinueReport {
    pub(crate) repair: RepairReport,
    pub(crate) taken: Option<NeedQueueTakeReport>,
    pub(crate) feedback: Option<Feedback>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) score: Option<RepairScoreReport>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct RepairPatchApplyReport {
    pub(crate) workspace: String,
    pub(crate) plan: String,
    pub(crate) patch_review: String,
    pub(crate) apply: String,
    pub(crate) status: String,
    pub(crate) applied: bool,
    pub(crate) authorized: bool,
    pub(crate) required_grant: String,
    pub(crate) grant_command: String,
    pub(crate) check_command: String,
    pub(crate) command: String,
    pub(crate) patch_file: String,
    pub(crate) target: String,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct RepairPatchVerifyReport {
    pub(crate) workspace: String,
    pub(crate) plan: String,
    pub(crate) apply: String,
    pub(crate) verify: String,
    pub(crate) status: String,
    pub(crate) passed: bool,
    pub(crate) target: String,
    pub(crate) command: String,
    pub(crate) summary: String,
    pub(crate) check: Option<CheckReport>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct RepairContinueArgs {
    pub(crate) query: String,
    pub(crate) score: Option<RepairContinueScoreArgs>,
}

#[derive(Debug)]
pub(crate) struct RepairContinueScoreArgs {
    pub(crate) status: Status,
    pub(crate) summary: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct RepairPlanReport {
    pub(crate) path: String,
    pub(crate) repair_queue_brief: String,
    pub(crate) repair_queue_brief_json: String,
    pub(crate) repair_queue_brief_status: String,
    pub(crate) repair_queue_brief_plan_status: String,
    pub(crate) repair_queue_brief_source: String,
    pub(crate) repair_queue_brief_next_need_kind: String,
    pub(crate) repair_queue_brief_next_need_query: String,
    pub(crate) repair_queue_brief_blockers: String,
    pub(crate) repair_queue_brief_review_first: String,
    pub(crate) review: String,
    pub(crate) outcome: String,
    pub(crate) outcome_status: String,
    pub(crate) outcome_summary: String,
    pub(crate) draft: String,
    pub(crate) draft_status: String,
    pub(crate) draft_prefix: String,
    pub(crate) draft_model: String,
    pub(crate) repair_draft_effectiveness: String,
    pub(crate) repair_draft_effectiveness_json: String,
    pub(crate) repair_draft_effectiveness_used_count: String,
    pub(crate) repair_draft_effectiveness_satisfied_count: String,
    pub(crate) repair_draft_effectiveness_partial_count: String,
    pub(crate) repair_draft_effectiveness_failed_count: String,
    pub(crate) repair_draft_effectiveness_success_rate: String,
    pub(crate) repair_draft_effectiveness_failure_rate: String,
    pub(crate) repair_draft_effectiveness_top_reuse: String,
    pub(crate) repair_draft_effectiveness_top_avoid: String,
    pub(crate) repair_draft_strategy: String,
    pub(crate) repair_draft_strategy_json: String,
    pub(crate) repair_draft_strategy_status: String,
    pub(crate) repair_draft_strategy_focus: String,
    pub(crate) repair_draft_strategy_learned_reuse: String,
    pub(crate) repair_draft_strategy_learned_avoid: String,
    pub(crate) repair_draft_strategy_strategy_learned_reuse: String,
    pub(crate) repair_draft_strategy_strategy_learned_avoid: String,
    pub(crate) repair_draft_strategy_next_need_kind: String,
    pub(crate) repair_draft_strategy_next_need_query: String,
    pub(crate) repair_draft_strategy_effectiveness: String,
    pub(crate) repair_draft_strategy_effectiveness_json: String,
    pub(crate) repair_draft_strategy_effectiveness_used_count: String,
    pub(crate) repair_draft_strategy_effectiveness_satisfied_count: String,
    pub(crate) repair_draft_strategy_effectiveness_partial_count: String,
    pub(crate) repair_draft_strategy_effectiveness_failed_count: String,
    pub(crate) repair_draft_strategy_effectiveness_success_rate: String,
    pub(crate) repair_draft_strategy_effectiveness_failure_rate: String,
    pub(crate) repair_draft_strategy_effectiveness_top_reuse: String,
    pub(crate) repair_draft_strategy_effectiveness_top_avoid: String,
    pub(crate) repair_patch_draft: String,
    pub(crate) repair_patch_draft_json: String,
    pub(crate) repair_patch_draft_status: String,
    pub(crate) repair_patch_draft_draft_status: String,
    pub(crate) repair_patch_draft_has_patch: String,
    pub(crate) repair_patch_draft_target: String,
    pub(crate) repair_patch_draft_next_need_kind: String,
    pub(crate) repair_patch_draft_next_need_query: String,
    pub(crate) repair_patch_draft_effectiveness: String,
    pub(crate) repair_patch_draft_effectiveness_json: String,
    pub(crate) repair_patch_draft_effectiveness_used_count: String,
    pub(crate) repair_patch_draft_effectiveness_satisfied_count: String,
    pub(crate) repair_patch_draft_effectiveness_partial_count: String,
    pub(crate) repair_patch_draft_effectiveness_failed_count: String,
    pub(crate) repair_patch_draft_effectiveness_success_rate: String,
    pub(crate) repair_patch_draft_effectiveness_failure_rate: String,
    pub(crate) repair_patch_draft_effectiveness_top_reuse: String,
    pub(crate) repair_patch_draft_effectiveness_top_avoid: String,
    pub(crate) repair_patch_review: String,
    pub(crate) repair_patch_review_json: String,
    pub(crate) repair_patch_review_status: String,
    pub(crate) repair_patch_review_check_status: String,
    pub(crate) repair_patch_review_has_patch: String,
    pub(crate) repair_patch_review_target: String,
    pub(crate) repair_patch_review_summary: String,
    pub(crate) repair_patch_review_patch_file: String,
    pub(crate) repair_patch_review_next_need_kind: String,
    pub(crate) repair_patch_review_next_need_query: String,
    pub(crate) repair_patch_review_effectiveness: String,
    pub(crate) repair_patch_review_effectiveness_json: String,
    pub(crate) repair_patch_review_effectiveness_used_count: String,
    pub(crate) repair_patch_review_effectiveness_satisfied_count: String,
    pub(crate) repair_patch_review_effectiveness_partial_count: String,
    pub(crate) repair_patch_review_effectiveness_failed_count: String,
    pub(crate) repair_patch_review_effectiveness_success_rate: String,
    pub(crate) repair_patch_review_effectiveness_failure_rate: String,
    pub(crate) repair_patch_review_effectiveness_top_reuse: String,
    pub(crate) repair_patch_review_effectiveness_top_avoid: String,
    pub(crate) repair_patch_apply: String,
    pub(crate) repair_patch_apply_json: String,
    pub(crate) repair_patch_apply_status: String,
    pub(crate) repair_patch_apply_applied: String,
    pub(crate) repair_patch_apply_authorized: String,
    pub(crate) repair_patch_apply_target: String,
    pub(crate) repair_patch_apply_patch_file: String,
    pub(crate) repair_patch_apply_required_grant: String,
    pub(crate) repair_patch_apply_grant_command: String,
    pub(crate) repair_patch_apply_apply_command: String,
    pub(crate) repair_patch_apply_summary: String,
    pub(crate) repair_patch_apply_effectiveness: String,
    pub(crate) repair_patch_apply_effectiveness_json: String,
    pub(crate) repair_patch_apply_effectiveness_used_count: String,
    pub(crate) repair_patch_apply_effectiveness_satisfied_count: String,
    pub(crate) repair_patch_apply_effectiveness_partial_count: String,
    pub(crate) repair_patch_apply_effectiveness_failed_count: String,
    pub(crate) repair_patch_apply_effectiveness_success_rate: String,
    pub(crate) repair_patch_apply_effectiveness_failure_rate: String,
    pub(crate) repair_patch_apply_effectiveness_top_reuse: String,
    pub(crate) repair_patch_apply_effectiveness_top_avoid: String,
    pub(crate) repair_patch_verify: String,
    pub(crate) repair_patch_verify_json: String,
    pub(crate) repair_patch_verify_status: String,
    pub(crate) repair_patch_verify_passed: String,
    pub(crate) repair_patch_verify_target: String,
    pub(crate) repair_patch_verify_command: String,
    pub(crate) repair_patch_verify_summary: String,
    pub(crate) repair_patch_verify_effectiveness: String,
    pub(crate) repair_patch_verify_effectiveness_json: String,
    pub(crate) repair_patch_verify_effectiveness_used_count: String,
    pub(crate) repair_patch_verify_effectiveness_satisfied_count: String,
    pub(crate) repair_patch_verify_effectiveness_partial_count: String,
    pub(crate) repair_patch_verify_effectiveness_failed_count: String,
    pub(crate) repair_patch_verify_effectiveness_success_rate: String,
    pub(crate) repair_patch_verify_effectiveness_failure_rate: String,
    pub(crate) repair_patch_verify_effectiveness_top_reuse: String,
    pub(crate) repair_patch_verify_effectiveness_top_avoid: String,
    pub(crate) repair_patch_learning: String,
    pub(crate) repair_patch_learning_json: String,
    pub(crate) repair_patch_learning_status: String,
    pub(crate) repair_patch_learning_used_count: String,
    pub(crate) repair_patch_learning_verified_count: String,
    pub(crate) repair_patch_learning_verified_satisfied_count: String,
    pub(crate) repair_patch_learning_verified_failed_count: String,
    pub(crate) repair_patch_learning_success_rate: String,
    pub(crate) repair_patch_learning_verified_success_rate: String,
    pub(crate) repair_patch_learning_top_reuse: String,
    pub(crate) repair_patch_learning_top_avoid: String,
    pub(crate) repair_patch_learning_next_need_kind: String,
    pub(crate) repair_patch_learning_next_need_query: String,
    pub(crate) repair_patch_learning_effectiveness: String,
    pub(crate) repair_patch_learning_effectiveness_json: String,
    pub(crate) repair_patch_learning_effectiveness_used_count: String,
    pub(crate) repair_patch_learning_effectiveness_satisfied_count: String,
    pub(crate) repair_patch_learning_effectiveness_partial_count: String,
    pub(crate) repair_patch_learning_effectiveness_failed_count: String,
    pub(crate) repair_patch_learning_effectiveness_success_rate: String,
    pub(crate) repair_patch_learning_effectiveness_failure_rate: String,
    pub(crate) repair_patch_learning_effectiveness_top_reuse: String,
    pub(crate) repair_patch_learning_effectiveness_top_avoid: String,
    pub(crate) repair_patch_strategy: String,
    pub(crate) repair_patch_strategy_json: String,
    pub(crate) repair_patch_strategy_status: String,
    pub(crate) repair_patch_strategy_focus: String,
    pub(crate) repair_patch_strategy_learned_reuse: String,
    pub(crate) repair_patch_strategy_learned_avoid: String,
    pub(crate) repair_patch_strategy_next_need_kind: String,
    pub(crate) repair_patch_strategy_next_need_query: String,
    pub(crate) repair_patch_strategy_effectiveness: String,
    pub(crate) repair_patch_strategy_effectiveness_json: String,
    pub(crate) repair_patch_strategy_effectiveness_used_count: String,
    pub(crate) repair_patch_strategy_effectiveness_satisfied_count: String,
    pub(crate) repair_patch_strategy_effectiveness_partial_count: String,
    pub(crate) repair_patch_strategy_effectiveness_failed_count: String,
    pub(crate) repair_patch_strategy_effectiveness_success_rate: String,
    pub(crate) repair_patch_strategy_effectiveness_failure_rate: String,
    pub(crate) repair_patch_strategy_effectiveness_top_reuse: String,
    pub(crate) repair_patch_strategy_effectiveness_top_avoid: String,
    pub(crate) repair_command_effectiveness: String,
    pub(crate) repair_command_effectiveness_json: String,
    pub(crate) repair_command_effectiveness_used_count: String,
    pub(crate) repair_command_effectiveness_satisfied_count: String,
    pub(crate) repair_command_effectiveness_partial_count: String,
    pub(crate) repair_command_effectiveness_failed_count: String,
    pub(crate) repair_command_effectiveness_success_rate: String,
    pub(crate) repair_command_effectiveness_failure_rate: String,
    pub(crate) repair_command_effectiveness_top_reuse: String,
    pub(crate) repair_command_effectiveness_top_avoid: String,
    pub(crate) repair_command_strategy: String,
    pub(crate) repair_command_strategy_json: String,
    pub(crate) repair_command_strategy_status: String,
    pub(crate) repair_command_strategy_focus: String,
    pub(crate) repair_command_strategy_learned_reuse: String,
    pub(crate) repair_command_strategy_learned_avoid: String,
    pub(crate) repair_command_strategy_current_apply: String,
    pub(crate) repair_command_strategy_next_need_kind: String,
    pub(crate) repair_command_strategy_next_need_query: String,
    pub(crate) repair_command_strategy_effectiveness: String,
    pub(crate) repair_command_strategy_effectiveness_json: String,
    pub(crate) repair_command_strategy_effectiveness_used_count: String,
    pub(crate) repair_command_strategy_effectiveness_satisfied_count: String,
    pub(crate) repair_command_strategy_effectiveness_partial_count: String,
    pub(crate) repair_command_strategy_effectiveness_failed_count: String,
    pub(crate) repair_command_strategy_effectiveness_success_rate: String,
    pub(crate) repair_command_strategy_effectiveness_failure_rate: String,
    pub(crate) repair_command_strategy_effectiveness_top_reuse: String,
    pub(crate) repair_command_strategy_effectiveness_top_avoid: String,
    pub(crate) action_trace: String,
    pub(crate) action_trace_json: String,
    pub(crate) action_trace_status: String,
    pub(crate) action_trace_stage_count: String,
    pub(crate) action_trace_last_action: String,
    pub(crate) action_trace_effectiveness: String,
    pub(crate) action_trace_effectiveness_json: String,
    pub(crate) action_trace_effectiveness_used_count: String,
    pub(crate) action_trace_effectiveness_satisfied_count: String,
    pub(crate) action_trace_effectiveness_partial_count: String,
    pub(crate) action_trace_effectiveness_failed_count: String,
    pub(crate) action_trace_effectiveness_success_rate: String,
    pub(crate) action_trace_effectiveness_failure_rate: String,
    pub(crate) action_trace_effectiveness_top_reuse: String,
    pub(crate) action_trace_effectiveness_top_avoid: String,
    pub(crate) repair_recall: String,
    pub(crate) repair_recall_match_count: String,
    pub(crate) repair_recall_top_status: String,
    pub(crate) repair_recall_top_score: String,
    pub(crate) repair_recall_top_reasons: String,
    pub(crate) repair_recall_top_summary: String,
    pub(crate) repair_lessons: String,
    pub(crate) repair_lessons_json: String,
    pub(crate) repair_lessons_count: String,
    pub(crate) repair_lessons_reuse_count: String,
    pub(crate) repair_lessons_avoid_count: String,
    pub(crate) repair_lessons_top_reuse: String,
    pub(crate) repair_lessons_top_avoid: String,
    pub(crate) repair_lesson_effectiveness: String,
    pub(crate) repair_lesson_effectiveness_json: String,
    pub(crate) repair_lesson_effectiveness_used_count: String,
    pub(crate) repair_lesson_effectiveness_satisfied_count: String,
    pub(crate) repair_lesson_effectiveness_partial_count: String,
    pub(crate) repair_lesson_effectiveness_failed_count: String,
    pub(crate) repair_lesson_effectiveness_success_rate: String,
    pub(crate) repair_lesson_effectiveness_failure_rate: String,
    pub(crate) repair_lesson_effectiveness_top_reuse: String,
    pub(crate) repair_lesson_effectiveness_top_avoid: String,
    pub(crate) repair_decision: String,
    pub(crate) repair_decision_json: String,
    pub(crate) repair_decision_status: String,
    pub(crate) repair_decision_kind: String,
    pub(crate) repair_decision_focus: String,
    pub(crate) repair_decision_next_need_kind: String,
    pub(crate) repair_decision_next_need_query: String,
    pub(crate) repair_decision_effectiveness: String,
    pub(crate) repair_decision_effectiveness_json: String,
    pub(crate) repair_decision_effectiveness_used_count: String,
    pub(crate) repair_decision_effectiveness_satisfied_count: String,
    pub(crate) repair_decision_effectiveness_partial_count: String,
    pub(crate) repair_decision_effectiveness_failed_count: String,
    pub(crate) repair_decision_effectiveness_success_rate: String,
    pub(crate) repair_decision_effectiveness_failure_rate: String,
    pub(crate) repair_decision_effectiveness_top_reuse: String,
    pub(crate) repair_decision_effectiveness_top_avoid: String,
    pub(crate) harness_adaptation_effectiveness: String,
    pub(crate) harness_adaptation_effectiveness_json: String,
    pub(crate) harness_adaptation_effectiveness_used_count: String,
    pub(crate) harness_adaptation_effectiveness_satisfied_count: String,
    pub(crate) harness_adaptation_effectiveness_partial_count: String,
    pub(crate) harness_adaptation_effectiveness_failed_count: String,
    pub(crate) harness_adaptation_effectiveness_success_rate: String,
    pub(crate) harness_adaptation_effectiveness_failure_rate: String,
    pub(crate) harness_adaptation_effectiveness_top_reuse: String,
    pub(crate) harness_adaptation_effectiveness_top_avoid: String,
    pub(crate) harness_environment_profile: String,
    pub(crate) harness_environment_profile_json: String,
    pub(crate) harness_environment_profile_status: String,
    pub(crate) harness_environment_profile_id: String,
    pub(crate) harness_environment_profile_execution_mode: String,
    pub(crate) harness_environment_profile_provider_mode: String,
    pub(crate) harness_environment_profile_desktop_mode: String,
    pub(crate) harness_environment_profile_capabilities: String,
    pub(crate) harness_environment_profile_constraints: String,
    pub(crate) harness_environment_profile_installed_profiles: String,
    pub(crate) harness_environment_profile_next_need_kind: String,
    pub(crate) harness_environment_profile_next_need_query: String,
    pub(crate) harness_environment_drift: String,
    pub(crate) harness_environment_drift_json: String,
    pub(crate) harness_environment_drift_status: String,
    pub(crate) harness_environment_drift_summary: String,
    pub(crate) harness_environment_drift_detail: String,
    pub(crate) harness_environment_drift_history_count: String,
    pub(crate) harness_environment_drift_gained_count: String,
    pub(crate) harness_environment_drift_lost_count: String,
    pub(crate) harness_environment_drift_next_need_kind: String,
    pub(crate) harness_environment_drift_next_need_query: String,
    pub(crate) harness_environment_drift_effectiveness: String,
    pub(crate) harness_environment_drift_effectiveness_json: String,
    pub(crate) harness_environment_drift_effectiveness_used_count: String,
    pub(crate) harness_environment_drift_effectiveness_satisfied_count: String,
    pub(crate) harness_environment_drift_effectiveness_partial_count: String,
    pub(crate) harness_environment_drift_effectiveness_failed_count: String,
    pub(crate) harness_environment_drift_effectiveness_success_rate: String,
    pub(crate) harness_environment_drift_effectiveness_failure_rate: String,
    pub(crate) harness_environment_drift_effectiveness_top_reuse: String,
    pub(crate) harness_environment_drift_effectiveness_top_avoid: String,
    pub(crate) repair_effectiveness_rollup: String,
    pub(crate) repair_effectiveness_rollup_json: String,
    pub(crate) repair_effectiveness_rollup_status: String,
    pub(crate) repair_effectiveness_rollup_source_count: String,
    pub(crate) repair_effectiveness_rollup_active_source_count: String,
    pub(crate) repair_effectiveness_rollup_used_count: String,
    pub(crate) repair_effectiveness_rollup_satisfied_count: String,
    pub(crate) repair_effectiveness_rollup_partial_count: String,
    pub(crate) repair_effectiveness_rollup_failed_count: String,
    pub(crate) repair_effectiveness_rollup_success_rate: String,
    pub(crate) repair_effectiveness_rollup_failure_rate: String,
    pub(crate) repair_effectiveness_rollup_strongest_source: String,
    pub(crate) repair_effectiveness_rollup_weakest_source: String,
    pub(crate) repair_effectiveness_rollup_top_reuse: String,
    pub(crate) repair_effectiveness_rollup_top_avoid: String,
    pub(crate) repair_effectiveness_rollup_next_need_kind: String,
    pub(crate) repair_effectiveness_rollup_next_need_query: String,
    pub(crate) repair_effectiveness_rollup_effectiveness: String,
    pub(crate) repair_effectiveness_rollup_effectiveness_json: String,
    pub(crate) repair_effectiveness_rollup_effectiveness_used_count: String,
    pub(crate) repair_effectiveness_rollup_effectiveness_satisfied_count: String,
    pub(crate) repair_effectiveness_rollup_effectiveness_partial_count: String,
    pub(crate) repair_effectiveness_rollup_effectiveness_failed_count: String,
    pub(crate) repair_effectiveness_rollup_effectiveness_success_rate: String,
    pub(crate) repair_effectiveness_rollup_effectiveness_failure_rate: String,
    pub(crate) repair_effectiveness_rollup_effectiveness_top_reuse: String,
    pub(crate) repair_effectiveness_rollup_effectiveness_top_avoid: String,
    pub(crate) repair_effectiveness_rollup_adaptation: String,
    pub(crate) repair_effectiveness_rollup_adaptation_json: String,
    pub(crate) repair_effectiveness_rollup_adaptation_status: String,
    pub(crate) repair_effectiveness_rollup_adaptation_focus: String,
    pub(crate) repair_effectiveness_rollup_adaptation_rollup_status: String,
    pub(crate) repair_effectiveness_rollup_adaptation_guidance_used_count: String,
    pub(crate) repair_effectiveness_rollup_adaptation_guidance_satisfied_count: String,
    pub(crate) repair_effectiveness_rollup_adaptation_guidance_partial_count: String,
    pub(crate) repair_effectiveness_rollup_adaptation_guidance_failed_count: String,
    pub(crate) repair_effectiveness_rollup_adaptation_guidance_success_rate: String,
    pub(crate) repair_effectiveness_rollup_adaptation_guidance_failure_rate: String,
    pub(crate) repair_effectiveness_rollup_adaptation_next_need_kind: String,
    pub(crate) repair_effectiveness_rollup_adaptation_next_need_query: String,
    pub(crate) repair_queue_brief_effectiveness: String,
    pub(crate) repair_queue_brief_effectiveness_json: String,
    pub(crate) repair_queue_brief_effectiveness_used_count: String,
    pub(crate) repair_queue_brief_effectiveness_satisfied_count: String,
    pub(crate) repair_queue_brief_effectiveness_partial_count: String,
    pub(crate) repair_queue_brief_effectiveness_failed_count: String,
    pub(crate) repair_queue_brief_effectiveness_success_rate: String,
    pub(crate) repair_queue_brief_effectiveness_failure_rate: String,
    pub(crate) repair_queue_brief_effectiveness_top_reuse: String,
    pub(crate) repair_queue_brief_effectiveness_top_avoid: String,
    pub(crate) repair_queue_brief_adaptation: String,
    pub(crate) repair_queue_brief_adaptation_json: String,
    pub(crate) repair_queue_brief_adaptation_status: String,
    pub(crate) repair_queue_brief_adaptation_focus: String,
    pub(crate) repair_queue_brief_adaptation_queue_used_count: String,
    pub(crate) repair_queue_brief_adaptation_queue_satisfied_count: String,
    pub(crate) repair_queue_brief_adaptation_queue_partial_count: String,
    pub(crate) repair_queue_brief_adaptation_queue_failed_count: String,
    pub(crate) repair_queue_brief_adaptation_queue_success_rate: String,
    pub(crate) repair_queue_brief_adaptation_queue_failure_rate: String,
    pub(crate) repair_queue_brief_adaptation_next_need_kind: String,
    pub(crate) repair_queue_brief_adaptation_next_need_query: String,
    pub(crate) environment_profile_journal: String,
    pub(crate) harness_adaptation: String,
    pub(crate) harness_adaptation_json: String,
    pub(crate) harness_adaptation_status: String,
    pub(crate) harness_adaptation_focus: String,
    pub(crate) harness_adaptation_next_need_kind: String,
    pub(crate) harness_adaptation_next_need_query: String,
    pub(crate) harness_adaptation_target: String,
    pub(crate) harness_adaptation_environment: String,
    pub(crate) code_context: String,
    pub(crate) adapter_context: String,
    pub(crate) adapter_context_status: String,
    pub(crate) adapter_context_missing_core: String,
    pub(crate) adapter_context_provider_env: String,
    pub(crate) adapter_context_provider_keys: String,
    pub(crate) adapter_context_desktop: String,
    pub(crate) field_trajectory: String,
    pub(crate) field_trajectory_field: String,
    pub(crate) field_trajectory_mini_task: String,
    pub(crate) field_trajectory_verifier_status: String,
    pub(crate) field_trajectory_verifier_error: String,
    pub(crate) status: String,
    pub(crate) schema: String,
    pub(crate) session: String,
    pub(crate) target_tentacle: String,
    pub(crate) target_tool: String,
    pub(crate) candidate: String,
    pub(crate) review_boundary: String,
    pub(crate) check_command: String,
    pub(crate) grant_command: String,
    pub(crate) apply_command: String,
    pub(crate) score_command: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) score_commands: Vec<String>,
    pub(crate) suggested_commands: String,
    pub(crate) next_need_kind: String,
    pub(crate) next_need_query: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct RepairScoreReport {
    pub(crate) outcome: RepairOutcome,
    pub(crate) journal: Option<RepairOutcomeJournalReport>,
    pub(crate) recent: Vec<RepairOutcome>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) evolution: Option<EvolutionOutcome>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) recommendation: Option<EvolutionRecommendation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) apply_artifact: Option<EvolutionApplyArtifact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) evolution_artifact: Option<EvolutionArtifact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) recommendation_error: Option<String>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct EvolutionScoreReport {
    pub(crate) outcome: EvolutionOutcome,
    pub(crate) recent: Vec<EvolutionOutcome>,
    pub(crate) recommendation: Option<EvolutionRecommendation>,
    pub(crate) apply_artifact: Option<EvolutionApplyArtifact>,
    pub(crate) evolution_artifact: Option<EvolutionArtifact>,
    pub(crate) recommendation_error: Option<String>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct RepairOutcomeJournalReport {
    pub(crate) session_path: String,
    pub(crate) outcome_path: String,
    pub(crate) outcomes_file: String,
}

pub(crate) fn handle_repair_command(
    rest: &[String],
    state: &Path,
    json: bool,
    language: Language,
) -> Result<(), String> {
    if rest.get(1).map(String::as_str) == Some("apply") {
        let query = rest
            .get(2..)
            .filter(|values| !values.is_empty())
            .map(|values| values.join(" "))
            .unwrap_or_else(|| ".".to_string());
        let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
        let report = repair_patch_apply_report(&state, &loaded, query)?;
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_repair_patch_apply_report(&report, language);
        }
        return Ok(());
    }
    if rest.get(1).map(String::as_str) == Some("verify") {
        let query = rest
            .get(2..)
            .filter(|values| !values.is_empty())
            .map(|values| values.join(" "))
            .unwrap_or_else(|| ".".to_string());
        let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
        let report = repair_patch_verify_report(&state, &mut loaded, query)?;
        loaded.save(&state).map_err(|error| error.to_string())?;
        pet_events::append_latest(&state, &loaded)?;
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_repair_patch_verify_report(&report, language);
        }
        return Ok(());
    }
    if rest.get(1).map(String::as_str) == Some("score") {
        let status = rest
            .get(3)
            .ok_or_else(|| "repair score requires a status".to_string())
            .and_then(|value| parse_status(value))?;
        let summary = rest
            .get(4..)
            .filter(|values| !values.is_empty())
            .map(|values| values.join(" "))
            .unwrap_or_else(|| "recorded repair outcome".to_string());
        let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
        let cwd = env::current_dir().map_err(|error| error.to_string())?;
        let trace_selector = rest
            .get(2)
            .ok_or_else(|| "repair score requires a feed trace index".to_string())?;
        let report = repair_score_report(&mut loaded, trace_selector, status, summary, &cwd)?;
        loaded.save(&state).map_err(|error| error.to_string())?;
        pet_events::append_latest(&state, &loaded)?;
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_repair_score_report(&report, language);
        }
        return Ok(());
    }
    if rest.get(1).map(String::as_str) == Some("continue") {
        let parsed = parse_repair_continue_args(rest.get(2..).unwrap_or(&[]))?;
        let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
        let mut report = repair_continue_report(&state, &mut loaded, parsed.query)?;
        if let Some(score_args) = parsed.score {
            let trace_index = report
                .feedback
                .as_ref()
                .and_then(feedback_trace_index)
                .ok_or_else(|| {
                    "repair continue --score requires a continued Feed trace".to_string()
                })?;
            let cwd = env::current_dir().map_err(|error| error.to_string())?;
            let score_report = repair_score_report(
                &mut loaded,
                &trace_index,
                score_args.status,
                score_args.summary,
                &cwd,
            )?;
            report.next = score_report.next.clone();
            report.score = Some(score_report);
        }
        loaded.save(&state).map_err(|error| error.to_string())?;
        pet_events::append_latest(&state, &loaded)?;
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_repair_continue_report(&report, language);
        }
        return Ok(());
    }
    let query = rest
        .get(1..)
        .filter(|values| !values.is_empty())
        .map(|values| values.join(" "))
        .unwrap_or_else(|| ".".to_string());
    let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
    let report = repair_report(&state, &mut loaded, query)?;
    loaded.save(&state).map_err(|error| error.to_string())?;
    pet_events::append_latest(&state, &loaded)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
        );
    } else {
        print_repair_report(&report, language);
    }
    Ok(())
}

pub(crate) fn print_repair_report(report: &RepairReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus repair");
            println!("tentacle: {}", report.tentacle);
            println!("state: {}", report.state_path);
            println!("feed: {:?} {}", report.feed.status, report.feed.summary);
            if let Some(index) = report.feed.metadata.get("feed_trace_index") {
                println!("trace_index: {index}");
            }
            if let Some(plan) = &report.repair_plan {
                println!("repair_plan: {}", plan.path);
                print_optional_line("repair_queue_brief", &plan.repair_queue_brief);
                print_optional_line(
                    "repair_queue_brief_status",
                    &repair_plan_queue_brief_label(plan),
                );
                print_optional_line("review", &plan.review);
                print_optional_line("outcome", &plan.outcome);
                print_optional_line("outcome_status", &plan.outcome_status);
                print_optional_line("draft", &plan.draft);
                print_optional_line("draft_status", &repair_plan_draft_label(plan));
                print_optional_line(
                    "repair_draft_effectiveness",
                    &plan.repair_draft_effectiveness,
                );
                print_optional_line(
                    "repair_draft_effectiveness_status",
                    &repair_plan_draft_effectiveness_label(plan),
                );
                print_optional_line("repair_draft_strategy", &plan.repair_draft_strategy);
                print_optional_line(
                    "repair_draft_strategy_status",
                    &repair_plan_draft_strategy_label(plan),
                );
                print_optional_line(
                    "repair_draft_strategy_effectiveness",
                    &plan.repair_draft_strategy_effectiveness,
                );
                print_optional_line(
                    "repair_draft_strategy_effectiveness_status",
                    &repair_plan_draft_strategy_effectiveness_label(plan),
                );
                print_optional_line("repair_patch_draft", &plan.repair_patch_draft);
                print_optional_line(
                    "repair_patch_draft_status",
                    &repair_plan_patch_draft_label(plan),
                );
                print_optional_line(
                    "repair_patch_draft_effectiveness",
                    &plan.repair_patch_draft_effectiveness,
                );
                print_optional_line(
                    "repair_patch_draft_effectiveness_status",
                    &repair_plan_patch_draft_effectiveness_label(plan),
                );
                print_optional_line("repair_patch_review", &plan.repair_patch_review);
                print_optional_line(
                    "repair_patch_review_status",
                    &repair_plan_patch_review_label(plan),
                );
                print_optional_line(
                    "repair_patch_review_effectiveness",
                    &plan.repair_patch_review_effectiveness,
                );
                print_optional_line(
                    "repair_patch_review_effectiveness_status",
                    &repair_plan_patch_review_effectiveness_label(plan),
                );
                print_optional_line("repair_patch_apply", &plan.repair_patch_apply);
                print_optional_line(
                    "repair_patch_apply_status",
                    &repair_plan_patch_apply_label(plan),
                );
                print_optional_line(
                    "repair_patch_apply_effectiveness",
                    &plan.repair_patch_apply_effectiveness,
                );
                print_optional_line(
                    "repair_patch_apply_effectiveness_status",
                    &repair_plan_patch_apply_effectiveness_label(plan),
                );
                print_optional_line("repair_patch_verify", &plan.repair_patch_verify);
                print_optional_line(
                    "repair_patch_verify_status",
                    &repair_plan_patch_verify_label(plan),
                );
                print_optional_line(
                    "repair_patch_verify_effectiveness",
                    &plan.repair_patch_verify_effectiveness,
                );
                print_optional_line(
                    "repair_patch_verify_effectiveness_status",
                    &repair_plan_patch_verify_effectiveness_label(plan),
                );
                print_optional_line("repair_patch_learning", &plan.repair_patch_learning);
                print_optional_line(
                    "repair_patch_learning_status",
                    &repair_plan_patch_learning_label(plan),
                );
                print_optional_line(
                    "repair_patch_learning_effectiveness",
                    &plan.repair_patch_learning_effectiveness,
                );
                print_optional_line(
                    "repair_patch_learning_effectiveness_status",
                    &repair_plan_patch_learning_effectiveness_label(plan),
                );
                print_optional_line("repair_patch_strategy", &plan.repair_patch_strategy);
                print_optional_line(
                    "repair_patch_strategy_status",
                    &repair_plan_patch_strategy_label(plan),
                );
                print_optional_line(
                    "repair_patch_strategy_effectiveness",
                    &plan.repair_patch_strategy_effectiveness,
                );
                print_optional_line(
                    "repair_patch_strategy_effectiveness_status",
                    &repair_plan_patch_strategy_effectiveness_label(plan),
                );
                print_optional_line(
                    "repair_command_effectiveness",
                    &plan.repair_command_effectiveness,
                );
                print_optional_line(
                    "repair_command_effectiveness_status",
                    &repair_plan_command_effectiveness_label(plan),
                );
                print_optional_line("repair_command_strategy", &plan.repair_command_strategy);
                print_optional_line(
                    "repair_command_strategy_status",
                    &repair_plan_command_strategy_label(plan),
                );
                print_optional_line(
                    "repair_command_strategy_effectiveness",
                    &plan.repair_command_strategy_effectiveness,
                );
                print_optional_line(
                    "repair_command_strategy_effectiveness_status",
                    &repair_plan_command_strategy_effectiveness_label(plan),
                );
                print_optional_line("action_trace", &plan.action_trace);
                print_optional_line("action_trace_json", &plan.action_trace_json);
                print_optional_line("action_trace_status", &repair_plan_action_trace_label(plan));
                print_optional_line(
                    "action_trace_effectiveness",
                    &plan.action_trace_effectiveness,
                );
                print_optional_line(
                    "action_trace_effectiveness_status",
                    &repair_plan_action_trace_effectiveness_label(plan),
                );
                print_optional_line("repair_recall", &plan.repair_recall);
                print_optional_line("repair_recall_status", &repair_plan_recall_label(plan));
                print_optional_line("repair_lessons", &plan.repair_lessons);
                print_optional_line("repair_lessons_status", &repair_plan_lessons_label(plan));
                print_optional_line(
                    "repair_lesson_effectiveness",
                    &plan.repair_lesson_effectiveness,
                );
                print_optional_line(
                    "repair_lesson_effectiveness_status",
                    &repair_plan_lesson_effectiveness_label(plan),
                );
                print_optional_line("repair_decision", &plan.repair_decision);
                print_optional_line("repair_decision_status", &repair_plan_decision_label(plan));
                print_optional_line(
                    "repair_decision_effectiveness",
                    &plan.repair_decision_effectiveness,
                );
                print_optional_line(
                    "repair_decision_effectiveness_status",
                    &repair_plan_decision_effectiveness_label(plan),
                );
                print_optional_line(
                    "harness_adaptation_effectiveness",
                    &plan.harness_adaptation_effectiveness,
                );
                print_optional_line(
                    "harness_adaptation_effectiveness_status",
                    &repair_plan_adaptation_effectiveness_label(plan),
                );
                print_optional_line(
                    "harness_environment_profile",
                    &plan.harness_environment_profile,
                );
                print_optional_line(
                    "harness_environment_profile_status",
                    &repair_plan_environment_profile_label(plan),
                );
                print_optional_line("harness_environment_drift", &plan.harness_environment_drift);
                print_optional_line(
                    "harness_environment_drift_status",
                    &repair_plan_environment_drift_label(plan),
                );
                print_optional_line(
                    "harness_environment_drift_effectiveness",
                    &plan.harness_environment_drift_effectiveness,
                );
                print_optional_line(
                    "harness_environment_drift_effectiveness_status",
                    &repair_plan_environment_drift_effectiveness_label(plan),
                );
                print_optional_line(
                    "repair_effectiveness_rollup",
                    &plan.repair_effectiveness_rollup,
                );
                print_optional_line(
                    "repair_effectiveness_rollup_status",
                    &repair_plan_effectiveness_rollup_label(plan),
                );
                print_optional_line(
                    "repair_effectiveness_rollup_effectiveness",
                    &plan.repair_effectiveness_rollup_effectiveness,
                );
                print_optional_line(
                    "repair_effectiveness_rollup_effectiveness_status",
                    &repair_plan_effectiveness_rollup_effectiveness_label(plan),
                );
                print_optional_line(
                    "repair_effectiveness_rollup_adaptation",
                    &plan.repair_effectiveness_rollup_adaptation,
                );
                print_optional_line(
                    "repair_effectiveness_rollup_adaptation_status",
                    &repair_plan_effectiveness_rollup_adaptation_label(plan),
                );
                print_optional_line(
                    "repair_queue_brief_effectiveness",
                    &plan.repair_queue_brief_effectiveness,
                );
                print_optional_line(
                    "repair_queue_brief_effectiveness_status",
                    &repair_plan_queue_brief_effectiveness_label(plan),
                );
                print_optional_line(
                    "repair_queue_brief_adaptation",
                    &plan.repair_queue_brief_adaptation,
                );
                print_optional_line(
                    "repair_queue_brief_adaptation_status",
                    &repair_plan_queue_brief_adaptation_label(plan),
                );
                print_optional_line(
                    "environment_profile_journal",
                    &plan.environment_profile_journal,
                );
                print_optional_line("harness_adaptation", &plan.harness_adaptation);
                print_optional_line(
                    "harness_adaptation_status",
                    &repair_plan_adaptation_label(plan),
                );
                print_optional_line("code_context", &plan.code_context);
                print_optional_line("adapter_context", &plan.adapter_context);
                print_optional_line("adapter_status", &repair_plan_adapter_label(plan));
                print_optional_line("field_trajectory", &plan.field_trajectory);
                print_optional_line("field_target", &repair_plan_field_label(plan));
                print_optional_line("field_verifier", &repair_plan_field_verifier(plan));
                println!("plan_status: {}", plan.status);
                println!("target: {}/{}", plan.target_tentacle, plan.target_tool);
                print_optional_line("check", &plan.check_command);
                print_optional_line("grant", &plan.grant_command);
                print_optional_line("apply", &plan.apply_command);
                print_optional_line("score", &plan.score_command);
                if !plan.score_commands.is_empty() {
                    println!("score_options: {}", join_or_none(&plan.score_commands));
                }
            }
            println!("queued: {}", report.queued.len());
            for item in &report.queued {
                println!(
                    "- #{} {}:{}",
                    item.index,
                    need_label(&item.need.kind),
                    item.need.query
                );
            }
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼修复");
            println!("触手: {}", report.tentacle);
            println!("状态文件: {}", report.state_path);
            println!("Feed: {:?} {}", report.feed.status, report.feed.summary);
            if let Some(index) = report.feed.metadata.get("feed_trace_index") {
                println!("Feed轨迹编号: {index}");
            }
            if let Some(plan) = &report.repair_plan {
                println!("修复计划: {}", plan.path);
                print_optional_line("修复队列摘要", &plan.repair_queue_brief);
                print_optional_line("修复队列摘要状态", &repair_plan_queue_brief_label(plan));
                print_optional_line("审阅", &plan.review);
                print_optional_line("结果", &plan.outcome);
                print_optional_line("结果状态", &plan.outcome_status);
                print_optional_line("草稿", &plan.draft);
                print_optional_line("草稿状态", &repair_plan_draft_label(plan));
                print_optional_line("草稿有效性", &plan.repair_draft_effectiveness);
                print_optional_line(
                    "草稿有效性状态",
                    &repair_plan_draft_effectiveness_label(plan),
                );
                print_optional_line("草稿策略", &plan.repair_draft_strategy);
                print_optional_line("草稿策略状态", &repair_plan_draft_strategy_label(plan));
                print_optional_line("草稿策略有效性", &plan.repair_draft_strategy_effectiveness);
                print_optional_line(
                    "草稿策略有效性状态",
                    &repair_plan_draft_strategy_effectiveness_label(plan),
                );
                print_optional_line("补丁草稿", &plan.repair_patch_draft);
                print_optional_line("补丁草稿状态", &repair_plan_patch_draft_label(plan));
                print_optional_line("补丁草稿有效性", &plan.repair_patch_draft_effectiveness);
                print_optional_line(
                    "补丁草稿有效性状态",
                    &repair_plan_patch_draft_effectiveness_label(plan),
                );
                print_optional_line("补丁预审", &plan.repair_patch_review);
                print_optional_line("补丁预审状态", &repair_plan_patch_review_label(plan));
                print_optional_line("补丁预审有效性", &plan.repair_patch_review_effectiveness);
                print_optional_line(
                    "补丁预审有效性状态",
                    &repair_plan_patch_review_effectiveness_label(plan),
                );
                print_optional_line("补丁应用", &plan.repair_patch_apply);
                print_optional_line("补丁应用状态", &repair_plan_patch_apply_label(plan));
                print_optional_line("补丁应用有效性", &plan.repair_patch_apply_effectiveness);
                print_optional_line(
                    "补丁应用有效性状态",
                    &repair_plan_patch_apply_effectiveness_label(plan),
                );
                print_optional_line("补丁验证", &plan.repair_patch_verify);
                print_optional_line("补丁验证状态", &repair_plan_patch_verify_label(plan));
                print_optional_line("补丁验证有效性", &plan.repair_patch_verify_effectiveness);
                print_optional_line(
                    "补丁验证有效性状态",
                    &repair_plan_patch_verify_effectiveness_label(plan),
                );
                print_optional_line("补丁学习", &plan.repair_patch_learning);
                print_optional_line("补丁学习状态", &repair_plan_patch_learning_label(plan));
                print_optional_line("补丁学习有效性", &plan.repair_patch_learning_effectiveness);
                print_optional_line(
                    "补丁学习有效性状态",
                    &repair_plan_patch_learning_effectiveness_label(plan),
                );
                print_optional_line("补丁策略", &plan.repair_patch_strategy);
                print_optional_line("补丁策略状态", &repair_plan_patch_strategy_label(plan));
                print_optional_line("补丁策略有效性", &plan.repair_patch_strategy_effectiveness);
                print_optional_line(
                    "补丁策略有效性状态",
                    &repair_plan_patch_strategy_effectiveness_label(plan),
                );
                print_optional_line("命令有效性", &plan.repair_command_effectiveness);
                print_optional_line(
                    "命令有效性状态",
                    &repair_plan_command_effectiveness_label(plan),
                );
                print_optional_line("命令策略", &plan.repair_command_strategy);
                print_optional_line("命令策略状态", &repair_plan_command_strategy_label(plan));
                print_optional_line(
                    "命令策略有效性",
                    &plan.repair_command_strategy_effectiveness,
                );
                print_optional_line(
                    "命令策略有效性状态",
                    &repair_plan_command_strategy_effectiveness_label(plan),
                );
                print_optional_line("行动轨迹", &plan.action_trace);
                print_optional_line("行动轨迹JSON", &plan.action_trace_json);
                print_optional_line("行动轨迹状态", &repair_plan_action_trace_label(plan));
                print_optional_line("行动轨迹有效性", &plan.action_trace_effectiveness);
                print_optional_line(
                    "行动轨迹有效性状态",
                    &repair_plan_action_trace_effectiveness_label(plan),
                );
                print_optional_line("修复召回", &plan.repair_recall);
                print_optional_line("修复召回状态", &repair_plan_recall_label(plan));
                print_optional_line("修复教训", &plan.repair_lessons);
                print_optional_line("修复教训状态", &repair_plan_lessons_label(plan));
                print_optional_line("教训有效性", &plan.repair_lesson_effectiveness);
                print_optional_line(
                    "教训有效性状态",
                    &repair_plan_lesson_effectiveness_label(plan),
                );
                print_optional_line("修复决策", &plan.repair_decision);
                print_optional_line("修复决策状态", &repair_plan_decision_label(plan));
                print_optional_line("修复决策有效性", &plan.repair_decision_effectiveness);
                print_optional_line(
                    "修复决策有效性状态",
                    &repair_plan_decision_effectiveness_label(plan),
                );
                print_optional_line("Harness适应有效性", &plan.harness_adaptation_effectiveness);
                print_optional_line(
                    "Harness适应有效性状态",
                    &repair_plan_adaptation_effectiveness_label(plan),
                );
                print_optional_line("Harness环境Profile", &plan.harness_environment_profile);
                print_optional_line(
                    "Harness环境Profile状态",
                    &repair_plan_environment_profile_label(plan),
                );
                print_optional_line("Harness环境漂移", &plan.harness_environment_drift);
                print_optional_line(
                    "Harness环境漂移状态",
                    &repair_plan_environment_drift_label(plan),
                );
                print_optional_line(
                    "Harness环境漂移有效性",
                    &plan.harness_environment_drift_effectiveness,
                );
                print_optional_line(
                    "Harness环境漂移有效性状态",
                    &repair_plan_environment_drift_effectiveness_label(plan),
                );
                print_optional_line("修复有效性总览", &plan.repair_effectiveness_rollup);
                print_optional_line(
                    "修复有效性总览状态",
                    &repair_plan_effectiveness_rollup_label(plan),
                );
                print_optional_line(
                    "修复有效性总览有效性",
                    &plan.repair_effectiveness_rollup_effectiveness,
                );
                print_optional_line(
                    "修复有效性总览有效性状态",
                    &repair_plan_effectiveness_rollup_effectiveness_label(plan),
                );
                print_optional_line(
                    "修复有效性总览适应",
                    &plan.repair_effectiveness_rollup_adaptation,
                );
                print_optional_line(
                    "修复有效性总览适应状态",
                    &repair_plan_effectiveness_rollup_adaptation_label(plan),
                );
                print_optional_line("修复队列摘要有效性", &plan.repair_queue_brief_effectiveness);
                print_optional_line(
                    "修复队列摘要有效性状态",
                    &repair_plan_queue_brief_effectiveness_label(plan),
                );
                print_optional_line("修复队列摘要适应", &plan.repair_queue_brief_adaptation);
                print_optional_line(
                    "修复队列摘要适应状态",
                    &repair_plan_queue_brief_adaptation_label(plan),
                );
                print_optional_line("环境Profile日志", &plan.environment_profile_journal);
                print_optional_line("Harness适应", &plan.harness_adaptation);
                print_optional_line("Harness适应状态", &repair_plan_adaptation_label(plan));
                print_optional_line("代码上下文", &plan.code_context);
                print_optional_line("适配器上下文", &plan.adapter_context);
                print_optional_line("适配器状态", &repair_plan_adapter_label(plan));
                print_optional_line("领域轨迹", &plan.field_trajectory);
                print_optional_line("领域目标", &repair_plan_field_label(plan));
                print_optional_line("领域验证", &repair_plan_field_verifier(plan));
                println!("计划状态: {}", plan.status);
                println!("目标: {}/{}", plan.target_tentacle, plan.target_tool);
                print_optional_line("检查", &plan.check_command);
                print_optional_line("授权", &plan.grant_command);
                print_optional_line("应用", &plan.apply_command);
                print_optional_line("评分", &plan.score_command);
                if !plan.score_commands.is_empty() {
                    println!("评分选项: {}", join_or_none(&plan.score_commands));
                }
            }
            println!("排队Need: {}", report.queued.len());
            for item in &report.queued {
                println!(
                    "- #{} {}:{}",
                    item.index,
                    need_label(&item.need.kind),
                    item.need.query
                );
            }
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

fn print_optional_line(label: &str, value: &str) {
    if !value.trim().is_empty() {
        println!("{label}: {value}");
    }
}

fn repair_plan_queue_brief_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.repair_queue_brief_status.trim().is_empty() {
        parts.push(plan.repair_queue_brief_status.trim().to_string());
    }
    if !plan.repair_queue_brief_source.trim().is_empty() {
        parts.push(format!("source={}", plan.repair_queue_brief_source));
    }
    if !plan.repair_queue_brief_next_need_query.trim().is_empty() {
        let kind = if plan.repair_queue_brief_next_need_kind.trim().is_empty() {
            "verify"
        } else {
            plan.repair_queue_brief_next_need_kind.trim()
        };
        parts.push(format!(
            "next={kind} {}",
            plan.repair_queue_brief_next_need_query
        ));
    }
    if !plan.repair_queue_brief_blockers.trim().is_empty() {
        parts.push(format!("blockers={}", plan.repair_queue_brief_blockers));
    }
    if !plan.repair_queue_brief_review_first.trim().is_empty() {
        parts.push(format!(
            "review_first={}",
            plan.repair_queue_brief_review_first
        ));
    }
    parts.join(" ")
}

fn repair_plan_field_label(plan: &RepairPlanReport) -> String {
    match (
        plan.field_trajectory_field.trim(),
        plan.field_trajectory_mini_task.trim(),
    ) {
        ("", "") => String::new(),
        (field, "") => field.to_string(),
        ("", mini_task) => mini_task.to_string(),
        (field, mini_task) => format!("{field}/{mini_task}"),
    }
}

fn repair_plan_adapter_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.adapter_context_status.trim().is_empty() {
        parts.push(plan.adapter_context_status.trim().to_string());
    }
    if !plan.adapter_context_missing_core.trim().is_empty() {
        parts.push(format!(
            "missing_core={}",
            plan.adapter_context_missing_core
        ));
    }
    if !plan.adapter_context_provider_env.trim().is_empty() {
        parts.push(format!(
            "provider_env={}",
            plan.adapter_context_provider_env
        ));
    }
    if !plan.adapter_context_provider_keys.trim().is_empty() {
        parts.push(format!(
            "provider_keys={}",
            plan.adapter_context_provider_keys
        ));
    }
    if !plan.adapter_context_desktop.trim().is_empty() {
        parts.push(format!("desktop={}", plan.adapter_context_desktop));
    }
    parts.join(" ")
}

fn repair_plan_draft_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.draft_status.trim().is_empty() {
        parts.push(plan.draft_status.trim().to_string());
    }
    if !plan.draft_prefix.trim().is_empty() {
        parts.push(format!("prefix={}", plan.draft_prefix));
    }
    if !plan.draft_model.trim().is_empty() {
        parts.push(format!("model={}", plan.draft_model));
    }
    parts.join(" ")
}

fn repair_effectiveness_label(fields: &[(&str, &str)]) -> String {
    let mut parts = Vec::new();
    for (name, value) in fields {
        if !value.trim().is_empty() {
            parts.push(format!("{name}={value}"));
        }
    }
    parts.join(" ")
}

fn repair_plan_draft_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        ("used", plan.repair_draft_effectiveness_used_count.as_str()),
        (
            "satisfied",
            plan.repair_draft_effectiveness_satisfied_count.as_str(),
        ),
        (
            "failed",
            plan.repair_draft_effectiveness_failed_count.as_str(),
        ),
        (
            "success_rate",
            plan.repair_draft_effectiveness_success_rate.as_str(),
        ),
        (
            "top_reuse",
            plan.repair_draft_effectiveness_top_reuse.as_str(),
        ),
        (
            "top_avoid",
            plan.repair_draft_effectiveness_top_avoid.as_str(),
        ),
    ])
}

fn repair_plan_draft_strategy_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.repair_draft_strategy_status.trim().is_empty() {
        parts.push(plan.repair_draft_strategy_status.trim().to_string());
    }
    if !plan.repair_draft_strategy_focus.trim().is_empty() {
        parts.push(format!("focus={}", plan.repair_draft_strategy_focus));
    }
    if !plan.repair_draft_strategy_learned_reuse.trim().is_empty() {
        parts.push(format!(
            "learned_reuse={}",
            plan.repair_draft_strategy_learned_reuse
        ));
    }
    if !plan.repair_draft_strategy_learned_avoid.trim().is_empty() {
        parts.push(format!(
            "learned_avoid={}",
            plan.repair_draft_strategy_learned_avoid
        ));
    }
    if !plan
        .repair_draft_strategy_strategy_learned_reuse
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "strategy_reuse={}",
            plan.repair_draft_strategy_strategy_learned_reuse
        ));
    }
    if !plan
        .repair_draft_strategy_strategy_learned_avoid
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "strategy_avoid={}",
            plan.repair_draft_strategy_strategy_learned_avoid
        ));
    }
    if !plan.repair_draft_strategy_next_need_query.trim().is_empty() {
        let kind = if plan.repair_draft_strategy_next_need_kind.trim().is_empty() {
            "verify"
        } else {
            plan.repair_draft_strategy_next_need_kind.trim()
        };
        parts.push(format!(
            "next={} {}",
            kind, plan.repair_draft_strategy_next_need_query
        ));
    }
    parts.join(" ")
}

fn repair_plan_draft_strategy_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        (
            "used",
            plan.repair_draft_strategy_effectiveness_used_count.as_str(),
        ),
        (
            "satisfied",
            plan.repair_draft_strategy_effectiveness_satisfied_count
                .as_str(),
        ),
        (
            "failed",
            plan.repair_draft_strategy_effectiveness_failed_count
                .as_str(),
        ),
        (
            "success_rate",
            plan.repair_draft_strategy_effectiveness_success_rate
                .as_str(),
        ),
        (
            "top_reuse",
            plan.repair_draft_strategy_effectiveness_top_reuse.as_str(),
        ),
        (
            "top_avoid",
            plan.repair_draft_strategy_effectiveness_top_avoid.as_str(),
        ),
    ])
}

fn repair_plan_patch_draft_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.repair_patch_draft_status.trim().is_empty() {
        parts.push(plan.repair_patch_draft_status.trim().to_string());
    }
    if !plan.repair_patch_draft_has_patch.trim().is_empty() {
        parts.push(format!("has_patch={}", plan.repair_patch_draft_has_patch));
    }
    if !plan.repair_patch_draft_target.trim().is_empty() {
        parts.push(format!("target={}", plan.repair_patch_draft_target));
    }
    if !plan.repair_patch_draft_next_need_query.trim().is_empty() {
        let kind = if plan.repair_patch_draft_next_need_kind.trim().is_empty() {
            "verify"
        } else {
            plan.repair_patch_draft_next_need_kind.trim()
        };
        parts.push(format!(
            "next={kind} {}",
            plan.repair_patch_draft_next_need_query
        ));
    }
    parts.join(" ")
}

fn repair_plan_patch_draft_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        (
            "used",
            plan.repair_patch_draft_effectiveness_used_count.as_str(),
        ),
        (
            "satisfied",
            plan.repair_patch_draft_effectiveness_satisfied_count
                .as_str(),
        ),
        (
            "failed",
            plan.repair_patch_draft_effectiveness_failed_count.as_str(),
        ),
        (
            "success_rate",
            plan.repair_patch_draft_effectiveness_success_rate.as_str(),
        ),
        (
            "top_reuse",
            plan.repair_patch_draft_effectiveness_top_reuse.as_str(),
        ),
        (
            "top_avoid",
            plan.repair_patch_draft_effectiveness_top_avoid.as_str(),
        ),
    ])
}

fn repair_plan_patch_review_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.repair_patch_review_status.trim().is_empty() {
        parts.push(plan.repair_patch_review_status.trim().to_string());
    }
    if !plan.repair_patch_review_check_status.trim().is_empty() {
        parts.push(format!("check={}", plan.repair_patch_review_check_status));
    }
    if !plan.repair_patch_review_has_patch.trim().is_empty() {
        parts.push(format!("has_patch={}", plan.repair_patch_review_has_patch));
    }
    if !plan.repair_patch_review_target.trim().is_empty() {
        parts.push(format!("target={}", plan.repair_patch_review_target));
    }
    if !plan.repair_patch_review_summary.trim().is_empty() {
        parts.push(format!("summary={}", plan.repair_patch_review_summary));
    }
    if !plan.repair_patch_review_next_need_query.trim().is_empty() {
        let kind = if plan.repair_patch_review_next_need_kind.trim().is_empty() {
            "verify"
        } else {
            plan.repair_patch_review_next_need_kind.trim()
        };
        parts.push(format!(
            "next={kind} {}",
            plan.repair_patch_review_next_need_query
        ));
    }
    parts.join(" ")
}

fn repair_plan_patch_review_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        (
            "used",
            plan.repair_patch_review_effectiveness_used_count.as_str(),
        ),
        (
            "satisfied",
            plan.repair_patch_review_effectiveness_satisfied_count
                .as_str(),
        ),
        (
            "failed",
            plan.repair_patch_review_effectiveness_failed_count.as_str(),
        ),
        (
            "success_rate",
            plan.repair_patch_review_effectiveness_success_rate.as_str(),
        ),
        (
            "top_reuse",
            plan.repair_patch_review_effectiveness_top_reuse.as_str(),
        ),
        (
            "top_avoid",
            plan.repair_patch_review_effectiveness_top_avoid.as_str(),
        ),
    ])
}

fn repair_plan_patch_apply_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.repair_patch_apply_status.trim().is_empty() {
        parts.push(plan.repair_patch_apply_status.trim().to_string());
    }
    if !plan.repair_patch_apply_applied.trim().is_empty() {
        parts.push(format!("applied={}", plan.repair_patch_apply_applied));
    }
    if !plan.repair_patch_apply_authorized.trim().is_empty() {
        parts.push(format!("authorized={}", plan.repair_patch_apply_authorized));
    }
    if !plan.repair_patch_apply_target.trim().is_empty() {
        parts.push(format!("target={}", plan.repair_patch_apply_target));
    }
    if !plan.repair_patch_apply_required_grant.trim().is_empty() {
        parts.push(format!(
            "required_grant={}",
            plan.repair_patch_apply_required_grant
        ));
    }
    if !plan.repair_patch_apply_summary.trim().is_empty() {
        parts.push(format!("summary={}", plan.repair_patch_apply_summary));
    }
    parts.join(" ")
}

fn repair_plan_patch_apply_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        (
            "used",
            plan.repair_patch_apply_effectiveness_used_count.as_str(),
        ),
        (
            "satisfied",
            plan.repair_patch_apply_effectiveness_satisfied_count
                .as_str(),
        ),
        (
            "failed",
            plan.repair_patch_apply_effectiveness_failed_count.as_str(),
        ),
        (
            "success_rate",
            plan.repair_patch_apply_effectiveness_success_rate.as_str(),
        ),
        (
            "top_reuse",
            plan.repair_patch_apply_effectiveness_top_reuse.as_str(),
        ),
        (
            "top_avoid",
            plan.repair_patch_apply_effectiveness_top_avoid.as_str(),
        ),
    ])
}

fn repair_plan_patch_verify_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.repair_patch_verify_status.trim().is_empty() {
        parts.push(plan.repair_patch_verify_status.trim().to_string());
    }
    if !plan.repair_patch_verify_passed.trim().is_empty() {
        parts.push(format!("passed={}", plan.repair_patch_verify_passed));
    }
    if !plan.repair_patch_verify_target.trim().is_empty() {
        parts.push(format!("target={}", plan.repair_patch_verify_target));
    }
    if !plan.repair_patch_verify_command.trim().is_empty() {
        parts.push(format!("command={}", plan.repair_patch_verify_command));
    }
    if !plan.repair_patch_verify_summary.trim().is_empty() {
        parts.push(format!("summary={}", plan.repair_patch_verify_summary));
    }
    parts.join(" ")
}

fn repair_plan_patch_verify_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        (
            "used",
            plan.repair_patch_verify_effectiveness_used_count.as_str(),
        ),
        (
            "satisfied",
            plan.repair_patch_verify_effectiveness_satisfied_count
                .as_str(),
        ),
        (
            "failed",
            plan.repair_patch_verify_effectiveness_failed_count.as_str(),
        ),
        (
            "success_rate",
            plan.repair_patch_verify_effectiveness_success_rate.as_str(),
        ),
        (
            "top_reuse",
            plan.repair_patch_verify_effectiveness_top_reuse.as_str(),
        ),
        (
            "top_avoid",
            plan.repair_patch_verify_effectiveness_top_avoid.as_str(),
        ),
    ])
}

fn repair_plan_patch_learning_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.repair_patch_learning_status.trim().is_empty() {
        parts.push(plan.repair_patch_learning_status.trim().to_string());
    }
    if !plan.repair_patch_learning_used_count.trim().is_empty() {
        parts.push(format!("used={}", plan.repair_patch_learning_used_count));
    }
    if !plan.repair_patch_learning_verified_count.trim().is_empty() {
        parts.push(format!(
            "verified={}",
            plan.repair_patch_learning_verified_count
        ));
    }
    if !plan
        .repair_patch_learning_verified_success_rate
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "verified_success_rate={}",
            plan.repair_patch_learning_verified_success_rate
        ));
    }
    if !plan.repair_patch_learning_top_reuse.trim().is_empty() {
        parts.push(format!(
            "top_reuse={}",
            plan.repair_patch_learning_top_reuse
        ));
    }
    if !plan.repair_patch_learning_top_avoid.trim().is_empty() {
        parts.push(format!(
            "top_avoid={}",
            plan.repair_patch_learning_top_avoid
        ));
    }
    if !plan.repair_patch_learning_next_need_query.trim().is_empty() {
        parts.push(format!(
            "next={} {}",
            if plan.repair_patch_learning_next_need_kind.trim().is_empty() {
                "remember"
            } else {
                plan.repair_patch_learning_next_need_kind.trim()
            },
            plan.repair_patch_learning_next_need_query.trim()
        ));
    }
    parts.join(" ")
}

fn repair_plan_patch_learning_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        (
            "used",
            plan.repair_patch_learning_effectiveness_used_count.as_str(),
        ),
        (
            "satisfied",
            plan.repair_patch_learning_effectiveness_satisfied_count
                .as_str(),
        ),
        (
            "failed",
            plan.repair_patch_learning_effectiveness_failed_count
                .as_str(),
        ),
        (
            "success_rate",
            plan.repair_patch_learning_effectiveness_success_rate
                .as_str(),
        ),
        (
            "top_reuse",
            plan.repair_patch_learning_effectiveness_top_reuse.as_str(),
        ),
        (
            "top_avoid",
            plan.repair_patch_learning_effectiveness_top_avoid.as_str(),
        ),
    ])
}

fn repair_plan_patch_strategy_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.repair_patch_strategy_status.trim().is_empty() {
        parts.push(plan.repair_patch_strategy_status.trim().to_string());
    }
    if !plan.repair_patch_strategy_focus.trim().is_empty() {
        parts.push(format!("focus={}", plan.repair_patch_strategy_focus));
    }
    if !plan.repair_patch_strategy_learned_reuse.trim().is_empty() {
        parts.push(format!(
            "learned_reuse={}",
            plan.repair_patch_strategy_learned_reuse
        ));
    }
    if !plan.repair_patch_strategy_learned_avoid.trim().is_empty() {
        parts.push(format!(
            "learned_avoid={}",
            plan.repair_patch_strategy_learned_avoid
        ));
    }
    if !plan.repair_patch_strategy_next_need_query.trim().is_empty() {
        let kind = if plan.repair_patch_strategy_next_need_kind.trim().is_empty() {
            "verify"
        } else {
            plan.repair_patch_strategy_next_need_kind.trim()
        };
        parts.push(format!(
            "next={} {}",
            kind, plan.repair_patch_strategy_next_need_query
        ));
    }
    parts.join(" ")
}

fn repair_plan_patch_strategy_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        (
            "used",
            plan.repair_patch_strategy_effectiveness_used_count.as_str(),
        ),
        (
            "satisfied",
            plan.repair_patch_strategy_effectiveness_satisfied_count
                .as_str(),
        ),
        (
            "failed",
            plan.repair_patch_strategy_effectiveness_failed_count
                .as_str(),
        ),
        (
            "success_rate",
            plan.repair_patch_strategy_effectiveness_success_rate
                .as_str(),
        ),
        (
            "top_reuse",
            plan.repair_patch_strategy_effectiveness_top_reuse.as_str(),
        ),
        (
            "top_avoid",
            plan.repair_patch_strategy_effectiveness_top_avoid.as_str(),
        ),
    ])
}

fn repair_plan_command_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        (
            "used",
            plan.repair_command_effectiveness_used_count.as_str(),
        ),
        (
            "satisfied",
            plan.repair_command_effectiveness_satisfied_count.as_str(),
        ),
        (
            "failed",
            plan.repair_command_effectiveness_failed_count.as_str(),
        ),
        (
            "success_rate",
            plan.repair_command_effectiveness_success_rate.as_str(),
        ),
        (
            "top_reuse",
            plan.repair_command_effectiveness_top_reuse.as_str(),
        ),
        (
            "top_avoid",
            plan.repair_command_effectiveness_top_avoid.as_str(),
        ),
    ])
}

fn repair_plan_command_strategy_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.repair_command_strategy_status.trim().is_empty() {
        parts.push(plan.repair_command_strategy_status.trim().to_string());
    }
    if !plan.repair_command_strategy_focus.trim().is_empty() {
        parts.push(format!("focus={}", plan.repair_command_strategy_focus));
    }
    if !plan.repair_command_strategy_learned_reuse.trim().is_empty() {
        parts.push(format!(
            "reuse={}",
            plan.repair_command_strategy_learned_reuse
        ));
    }
    if !plan.repair_command_strategy_learned_avoid.trim().is_empty() {
        parts.push(format!(
            "avoid={}",
            plan.repair_command_strategy_learned_avoid
        ));
    }
    if !plan
        .repair_command_strategy_next_need_query
        .trim()
        .is_empty()
    {
        let kind = if plan
            .repair_command_strategy_next_need_kind
            .trim()
            .is_empty()
        {
            "verify"
        } else {
            plan.repair_command_strategy_next_need_kind.trim()
        };
        parts.push(format!(
            "next={} {}",
            kind, plan.repair_command_strategy_next_need_query
        ));
    }
    parts.join(" ")
}

fn repair_plan_command_strategy_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        (
            "used",
            plan.repair_command_strategy_effectiveness_used_count
                .as_str(),
        ),
        (
            "satisfied",
            plan.repair_command_strategy_effectiveness_satisfied_count
                .as_str(),
        ),
        (
            "failed",
            plan.repair_command_strategy_effectiveness_failed_count
                .as_str(),
        ),
        (
            "success_rate",
            plan.repair_command_strategy_effectiveness_success_rate
                .as_str(),
        ),
        (
            "top_reuse",
            plan.repair_command_strategy_effectiveness_top_reuse
                .as_str(),
        ),
        (
            "top_avoid",
            plan.repair_command_strategy_effectiveness_top_avoid
                .as_str(),
        ),
    ])
}

fn repair_plan_action_trace_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.action_trace_status.trim().is_empty() {
        parts.push(plan.action_trace_status.trim().to_string());
    }
    if !plan.action_trace_stage_count.trim().is_empty() {
        parts.push(format!("stages={}", plan.action_trace_stage_count));
    }
    if !plan.action_trace_last_action.trim().is_empty() {
        parts.push(format!("last={}", plan.action_trace_last_action));
    }
    parts.join(" ")
}

fn repair_plan_recall_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.repair_recall_match_count.trim().is_empty() {
        parts.push(format!("matches={}", plan.repair_recall_match_count));
    }
    if !plan.repair_recall_top_status.trim().is_empty() {
        parts.push(format!("top={}", plan.repair_recall_top_status));
    }
    if !plan.repair_recall_top_score.trim().is_empty() {
        parts.push(format!("score={}", plan.repair_recall_top_score));
    }
    if !plan.repair_recall_top_reasons.trim().is_empty() {
        parts.push(format!("reason={}", plan.repair_recall_top_reasons));
    }
    parts.join(" ")
}

fn repair_plan_lessons_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.repair_lessons_count.trim().is_empty() {
        parts.push(format!("lessons={}", plan.repair_lessons_count));
    }
    if !plan.repair_lessons_reuse_count.trim().is_empty() {
        parts.push(format!("reuse={}", plan.repair_lessons_reuse_count));
    }
    if !plan.repair_lessons_avoid_count.trim().is_empty() {
        parts.push(format!("avoid={}", plan.repair_lessons_avoid_count));
    }
    if !plan.repair_lessons_top_reuse.trim().is_empty() {
        parts.push(format!("top_reuse={}", plan.repair_lessons_top_reuse));
    }
    if !plan.repair_lessons_top_avoid.trim().is_empty() {
        parts.push(format!("top_avoid={}", plan.repair_lessons_top_avoid));
    }
    parts.join(" ")
}

fn repair_plan_action_trace_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        ("used", plan.action_trace_effectiveness_used_count.as_str()),
        (
            "satisfied",
            plan.action_trace_effectiveness_satisfied_count.as_str(),
        ),
        (
            "failed",
            plan.action_trace_effectiveness_failed_count.as_str(),
        ),
        (
            "success_rate",
            plan.action_trace_effectiveness_success_rate.as_str(),
        ),
        (
            "top_reuse",
            plan.action_trace_effectiveness_top_reuse.as_str(),
        ),
        (
            "top_avoid",
            plan.action_trace_effectiveness_top_avoid.as_str(),
        ),
    ])
}

fn repair_plan_lesson_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        ("used", plan.repair_lesson_effectiveness_used_count.as_str()),
        (
            "satisfied",
            plan.repair_lesson_effectiveness_satisfied_count.as_str(),
        ),
        (
            "failed",
            plan.repair_lesson_effectiveness_failed_count.as_str(),
        ),
        (
            "success_rate",
            plan.repair_lesson_effectiveness_success_rate.as_str(),
        ),
        (
            "top_reuse",
            plan.repair_lesson_effectiveness_top_reuse.as_str(),
        ),
        (
            "top_avoid",
            plan.repair_lesson_effectiveness_top_avoid.as_str(),
        ),
    ])
}

fn repair_plan_decision_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.repair_decision_status.trim().is_empty() {
        parts.push(plan.repair_decision_status.trim().to_string());
    }
    if !plan.repair_decision_kind.trim().is_empty() {
        parts.push(format!("decision={}", plan.repair_decision_kind));
    }
    if !plan.repair_decision_focus.trim().is_empty() {
        parts.push(format!("focus={}", plan.repair_decision_focus));
    }
    if !plan.repair_decision_next_need_query.trim().is_empty() {
        let kind = if plan.repair_decision_next_need_kind.trim().is_empty() {
            "verify"
        } else {
            plan.repair_decision_next_need_kind.trim()
        };
        parts.push(format!(
            "next={kind} {}",
            plan.repair_decision_next_need_query
        ));
    }
    parts.join(" ")
}

fn repair_plan_decision_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        (
            "used",
            plan.repair_decision_effectiveness_used_count.as_str(),
        ),
        (
            "satisfied",
            plan.repair_decision_effectiveness_satisfied_count.as_str(),
        ),
        (
            "failed",
            plan.repair_decision_effectiveness_failed_count.as_str(),
        ),
        (
            "success_rate",
            plan.repair_decision_effectiveness_success_rate.as_str(),
        ),
        (
            "top_reuse",
            plan.repair_decision_effectiveness_top_reuse.as_str(),
        ),
        (
            "top_avoid",
            plan.repair_decision_effectiveness_top_avoid.as_str(),
        ),
    ])
}

fn repair_plan_adaptation_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        (
            "used",
            plan.harness_adaptation_effectiveness_used_count.as_str(),
        ),
        (
            "satisfied",
            plan.harness_adaptation_effectiveness_satisfied_count
                .as_str(),
        ),
        (
            "failed",
            plan.harness_adaptation_effectiveness_failed_count.as_str(),
        ),
        (
            "success_rate",
            plan.harness_adaptation_effectiveness_success_rate.as_str(),
        ),
        (
            "top_reuse",
            plan.harness_adaptation_effectiveness_top_reuse.as_str(),
        ),
        (
            "top_avoid",
            plan.harness_adaptation_effectiveness_top_avoid.as_str(),
        ),
    ])
}

fn repair_plan_environment_profile_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.harness_environment_profile_status.trim().is_empty() {
        parts.push(plan.harness_environment_profile_status.trim().to_string());
    }
    if !plan
        .harness_environment_profile_execution_mode
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "mode={}",
            plan.harness_environment_profile_execution_mode
        ));
    }
    if !plan
        .harness_environment_profile_provider_mode
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "provider={}",
            plan.harness_environment_profile_provider_mode
        ));
    }
    if !plan
        .harness_environment_profile_desktop_mode
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "desktop={}",
            plan.harness_environment_profile_desktop_mode
        ));
    }
    if !plan
        .harness_environment_profile_capabilities
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "capabilities={}",
            plan.harness_environment_profile_capabilities
        ));
    }
    if !plan
        .harness_environment_profile_constraints
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "constraints={}",
            plan.harness_environment_profile_constraints
        ));
    }
    if !plan
        .harness_environment_profile_next_need_query
        .trim()
        .is_empty()
    {
        let kind = if plan
            .harness_environment_profile_next_need_kind
            .trim()
            .is_empty()
        {
            "verify"
        } else {
            plan.harness_environment_profile_next_need_kind.trim()
        };
        parts.push(format!(
            "next={kind} {}",
            plan.harness_environment_profile_next_need_query
        ));
    }
    parts.join(" ")
}

fn repair_plan_environment_drift_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.harness_environment_drift_status.trim().is_empty() {
        parts.push(plan.harness_environment_drift_status.trim().to_string());
    }
    if !plan
        .harness_environment_drift_history_count
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "history={}",
            plan.harness_environment_drift_history_count
        ));
    }
    if !plan
        .harness_environment_drift_gained_count
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "gained={}",
            plan.harness_environment_drift_gained_count
        ));
    }
    if !plan.harness_environment_drift_lost_count.trim().is_empty() {
        parts.push(format!(
            "lost={}",
            plan.harness_environment_drift_lost_count
        ));
    }
    if !plan.harness_environment_drift_detail.trim().is_empty() {
        parts.push(format!("detail={}", plan.harness_environment_drift_detail));
    }
    if !plan
        .harness_environment_drift_next_need_query
        .trim()
        .is_empty()
    {
        let kind = if plan
            .harness_environment_drift_next_need_kind
            .trim()
            .is_empty()
        {
            "verify"
        } else {
            plan.harness_environment_drift_next_need_kind.trim()
        };
        parts.push(format!(
            "next={kind} {}",
            plan.harness_environment_drift_next_need_query
        ));
    }
    parts.join(" ")
}

fn repair_plan_environment_drift_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        (
            "used",
            plan.harness_environment_drift_effectiveness_used_count
                .as_str(),
        ),
        (
            "satisfied",
            plan.harness_environment_drift_effectiveness_satisfied_count
                .as_str(),
        ),
        (
            "failed",
            plan.harness_environment_drift_effectiveness_failed_count
                .as_str(),
        ),
        (
            "success_rate",
            plan.harness_environment_drift_effectiveness_success_rate
                .as_str(),
        ),
        (
            "top_reuse",
            plan.harness_environment_drift_effectiveness_top_reuse
                .as_str(),
        ),
        (
            "top_avoid",
            plan.harness_environment_drift_effectiveness_top_avoid
                .as_str(),
        ),
    ])
}

fn repair_plan_effectiveness_rollup_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.repair_effectiveness_rollup_status.trim().is_empty() {
        parts.push(plan.repair_effectiveness_rollup_status.trim().to_string());
    }
    if !plan
        .repair_effectiveness_rollup_active_source_count
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "active_sources={}",
            plan.repair_effectiveness_rollup_active_source_count
        ));
    }
    if !plan
        .repair_effectiveness_rollup_used_count
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "used={}",
            plan.repair_effectiveness_rollup_used_count
        ));
    }
    if !plan
        .repair_effectiveness_rollup_failed_count
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "failed={}",
            plan.repair_effectiveness_rollup_failed_count
        ));
    }
    if !plan
        .repair_effectiveness_rollup_success_rate
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "success_rate={}",
            plan.repair_effectiveness_rollup_success_rate
        ));
    }
    if !plan
        .repair_effectiveness_rollup_weakest_source
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "weakest={}",
            plan.repair_effectiveness_rollup_weakest_source
        ));
    }
    if !plan
        .repair_effectiveness_rollup_next_need_query
        .trim()
        .is_empty()
    {
        let kind = if plan
            .repair_effectiveness_rollup_next_need_kind
            .trim()
            .is_empty()
        {
            "verify"
        } else {
            plan.repair_effectiveness_rollup_next_need_kind.trim()
        };
        parts.push(format!(
            "next={kind} {}",
            plan.repair_effectiveness_rollup_next_need_query
        ));
    }
    parts.join(" ")
}

fn repair_plan_effectiveness_rollup_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        (
            "used",
            plan.repair_effectiveness_rollup_effectiveness_used_count
                .as_str(),
        ),
        (
            "satisfied",
            plan.repair_effectiveness_rollup_effectiveness_satisfied_count
                .as_str(),
        ),
        (
            "failed",
            plan.repair_effectiveness_rollup_effectiveness_failed_count
                .as_str(),
        ),
        (
            "success_rate",
            plan.repair_effectiveness_rollup_effectiveness_success_rate
                .as_str(),
        ),
        (
            "top_reuse",
            plan.repair_effectiveness_rollup_effectiveness_top_reuse
                .as_str(),
        ),
        (
            "top_avoid",
            plan.repair_effectiveness_rollup_effectiveness_top_avoid
                .as_str(),
        ),
    ])
}

fn repair_plan_effectiveness_rollup_adaptation_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan
        .repair_effectiveness_rollup_adaptation_status
        .trim()
        .is_empty()
    {
        parts.push(
            plan.repair_effectiveness_rollup_adaptation_status
                .trim()
                .to_string(),
        );
    }
    if !plan
        .repair_effectiveness_rollup_adaptation_guidance_used_count
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "guidance_used={}",
            plan.repair_effectiveness_rollup_adaptation_guidance_used_count
        ));
    }
    if !plan
        .repair_effectiveness_rollup_adaptation_guidance_failed_count
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "failed={}",
            plan.repair_effectiveness_rollup_adaptation_guidance_failed_count
        ));
    }
    if !plan
        .repair_effectiveness_rollup_adaptation_guidance_success_rate
        .trim()
        .is_empty()
    {
        parts.push(format!(
            "success_rate={}",
            plan.repair_effectiveness_rollup_adaptation_guidance_success_rate
        ));
    }
    if !plan
        .repair_effectiveness_rollup_adaptation_next_need_query
        .trim()
        .is_empty()
    {
        let kind = if plan
            .repair_effectiveness_rollup_adaptation_next_need_kind
            .trim()
            .is_empty()
        {
            "verify"
        } else {
            plan.repair_effectiveness_rollup_adaptation_next_need_kind
                .trim()
        };
        parts.push(format!(
            "next={kind} {}",
            plan.repair_effectiveness_rollup_adaptation_next_need_query
        ));
    }
    parts.join(" ")
}

fn repair_plan_queue_brief_effectiveness_label(plan: &RepairPlanReport) -> String {
    repair_effectiveness_label(&[
        (
            "used",
            plan.repair_queue_brief_effectiveness_used_count.as_str(),
        ),
        (
            "satisfied",
            plan.repair_queue_brief_effectiveness_satisfied_count
                .as_str(),
        ),
        (
            "failed",
            plan.repair_queue_brief_effectiveness_failed_count.as_str(),
        ),
        (
            "success_rate",
            plan.repair_queue_brief_effectiveness_success_rate.as_str(),
        ),
        (
            "reuse",
            plan.repair_queue_brief_effectiveness_top_reuse.as_str(),
        ),
        (
            "avoid",
            plan.repair_queue_brief_effectiveness_top_avoid.as_str(),
        ),
    ])
}

fn repair_plan_queue_brief_adaptation_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.repair_queue_brief_adaptation_status.trim().is_empty() {
        parts.push(plan.repair_queue_brief_adaptation_status.trim().to_string());
    }
    let queue_used = plan.repair_queue_brief_adaptation_queue_used_count.trim();
    let queue_used = if queue_used.is_empty() {
        "0"
    } else {
        queue_used
    };
    let failed = plan.repair_queue_brief_adaptation_queue_failed_count.trim();
    let failed = if failed.is_empty() { "0" } else { failed };
    let success_rate = plan.repair_queue_brief_adaptation_queue_success_rate.trim();
    let success_rate = if success_rate.is_empty() {
        "0"
    } else {
        success_rate
    };
    parts.push(format!("queue_used={queue_used}"));
    parts.push(format!("failed={failed}"));
    parts.push(format!("success_rate={success_rate}"));
    if !plan
        .repair_queue_brief_adaptation_next_need_query
        .trim()
        .is_empty()
    {
        let next_kind = if plan
            .repair_queue_brief_adaptation_next_need_kind
            .trim()
            .is_empty()
        {
            "verify"
        } else {
            plan.repair_queue_brief_adaptation_next_need_kind.trim()
        };
        parts.push(format!(
            "next={} {}",
            next_kind,
            plan.repair_queue_brief_adaptation_next_need_query.trim()
        ));
    }
    parts.join(" ")
}

fn repair_plan_adaptation_label(plan: &RepairPlanReport) -> String {
    let mut parts = Vec::new();
    if !plan.harness_adaptation_status.trim().is_empty() {
        parts.push(plan.harness_adaptation_status.trim().to_string());
    }
    if !plan.harness_adaptation_focus.trim().is_empty() {
        parts.push(format!("focus={}", plan.harness_adaptation_focus));
    }
    if !plan.harness_adaptation_target.trim().is_empty() {
        parts.push(format!("target={}", plan.harness_adaptation_target));
    }
    if !plan.harness_adaptation_environment.trim().is_empty() {
        parts.push(format!("env={}", plan.harness_adaptation_environment));
    }
    if !plan.harness_adaptation_next_need_query.trim().is_empty() {
        let kind = if plan.harness_adaptation_next_need_kind.trim().is_empty() {
            "verify"
        } else {
            plan.harness_adaptation_next_need_kind.trim()
        };
        parts.push(format!(
            "next={kind} {}",
            plan.harness_adaptation_next_need_query
        ));
    }
    parts.join(" ")
}

fn repair_plan_field_verifier(plan: &RepairPlanReport) -> String {
    match (
        plan.field_trajectory_verifier_status.trim(),
        plan.field_trajectory_verifier_error.trim(),
    ) {
        ("", "") => String::new(),
        (status, "") => status.to_string(),
        ("", error) => error.to_string(),
        (status, error) => format!("{status} {error}"),
    }
}

pub(crate) fn print_repair_continue_report(report: &RepairContinueReport, language: Language) {
    print_repair_report(&report.repair, language);
    match language {
        Language::En => {
            if let Some(taken) = &report.taken {
                println!("continued_need: {}", need_queue_line(&taken.item));
            } else {
                println!("continued_need: none");
            }
            if let Some(feedback) = &report.feedback {
                println!("continued_feed: {}", feedback.summary);
                for feed in &feedback.feeds {
                    if let Some(index) = feed.metadata.get("feed_trace_index") {
                        println!("continued_trace_index: {index}");
                    }
                    println!("continued_status: {:?} {}", feed.status, feed.summary);
                }
            }
            if let Some(score) = &report.score {
                println!("continued_score: #{}", score.outcome.index);
                println!("continued_score_status: {:?}", score.outcome.status);
                println!("continued_score_summary: {}", score.outcome.summary);
            }
            println!("continue_next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            if let Some(taken) = &report.taken {
                println!("已继续Need: {}", need_queue_line(&taken.item));
            } else {
                println!("已继续Need: 无");
            }
            if let Some(feedback) = &report.feedback {
                println!("继续Feed: {}", feedback.summary);
                for feed in &feedback.feeds {
                    if let Some(index) = feed.metadata.get("feed_trace_index") {
                        println!("继续轨迹编号: {index}");
                    }
                    println!("继续状态: {:?} {}", feed.status, feed.summary);
                }
            }
            if let Some(score) = &report.score {
                println!("继续评分: #{}", score.outcome.index);
                println!("继续评分状态: {:?}", score.outcome.status);
                println!("继续评分摘要: {}", score.outcome.summary);
            }
            println!("继续下一步: {}", join_or_none(&report.next));
        }
    }
}

pub(crate) fn print_repair_patch_apply_report(report: &RepairPatchApplyReport, language: Language) {
    match language {
        Language::En => {
            println!("repair_patch_apply: {}", report.status);
            println!("applied: {}", report.applied);
            println!("authorized: {}", report.authorized);
            println!("target: {}", report.target);
            println!("patch: {}", report.patch_file);
            println!("required_grant: {}", report.required_grant);
            if !report.grant_command.is_empty() {
                println!("grant: {}", report.grant_command);
            }
            if !report.check_command.is_empty() {
                println!("check: {}", report.check_command);
            }
            if !report.command.is_empty() {
                println!("command: {}", report.command);
            }
            println!("apply_artifact: {}", report.apply);
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("修复补丁应用: {}", report.status);
            println!("已应用: {}", report.applied);
            println!("已授权: {}", report.authorized);
            println!("目标: {}", report.target);
            println!("补丁: {}", report.patch_file);
            println!("所需授权: {}", report.required_grant);
            if !report.grant_command.is_empty() {
                println!("授权: {}", report.grant_command);
            }
            if !report.check_command.is_empty() {
                println!("检查: {}", report.check_command);
            }
            if !report.command.is_empty() {
                println!("命令: {}", report.command);
            }
            println!("应用记录: {}", report.apply);
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

pub(crate) fn print_repair_patch_verify_report(
    report: &RepairPatchVerifyReport,
    language: Language,
) {
    match language {
        Language::En => {
            println!("repair_patch_verify: {}", report.status);
            println!("passed: {}", report.passed);
            println!("target: {}", report.target);
            if !report.command.is_empty() {
                println!("command: {}", report.command);
            }
            println!("summary: {}", report.summary);
            println!("verify_artifact: {}", report.verify);
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("修复补丁验证: {}", report.status);
            println!("已通过: {}", report.passed);
            println!("目标: {}", report.target);
            if !report.command.is_empty() {
                println!("命令: {}", report.command);
            }
            println!("摘要: {}", report.summary);
            println!("验证记录: {}", report.verify);
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

pub(crate) fn print_repair_score_report(report: &RepairScoreReport, language: Language) {
    match language {
        Language::En => {
            println!("recorded repair outcome #{}", report.outcome.index);
            if let Some(trace_index) = report.outcome.trace_index {
                println!("feed_trace: {trace_index}");
            }
            println!("status: {:?}", report.outcome.status);
            println!("score: {:.2}", report.outcome.score);
            println!("summary: {}", report.outcome.summary);
            if let Some(journal) = &report.journal {
                println!("outcome: {}", journal.outcome_path);
                println!("journal: {}", journal.outcomes_file);
            }
            if let Some(evolution) = &report.evolution {
                println!(
                    "evolution_outcome: #{} {}/{}",
                    evolution.index, evolution.tentacle_id, evolution.candidate_id
                );
            }
            if let Some(recommendation) = &report.recommendation {
                println!("next recommended: {}", recommendation.candidate_id);
                println!("next score: {:.2}", recommendation.recommendation_score);
                println!("next reason: {}", recommendation.reason);
            }
            if let Some(artifact) = &report.apply_artifact {
                println!("next plan: {}", artifact.plan_path);
                if let Some(path) = &artifact.patch_path {
                    println!("next patch: {path}");
                }
            }
            if let Some(error) = &report.recommendation_error {
                println!("recommendation error: {error}");
            }
            println!("recent_repair_outcomes: {}", report.recent.len());
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("已记录 repair 结果 #{}", report.outcome.index);
            if let Some(trace_index) = report.outcome.trace_index {
                println!("Feed trace: {trace_index}");
            }
            println!("状态: {:?}", report.outcome.status);
            println!("分数: {:.2}", report.outcome.score);
            println!("摘要: {}", report.outcome.summary);
            if let Some(journal) = &report.journal {
                println!("结果文件: {}", journal.outcome_path);
                println!("结果日志: {}", journal.outcomes_file);
            }
            if let Some(evolution) = &report.evolution {
                println!(
                    "Evolution 结果: #{} {}/{}",
                    evolution.index, evolution.tentacle_id, evolution.candidate_id
                );
            }
            if let Some(recommendation) = &report.recommendation {
                println!("下一推荐: {}", recommendation.candidate_id);
                println!("下一分数: {:.2}", recommendation.recommendation_score);
                println!("下一原因: {}", recommendation.reason);
            }
            if let Some(artifact) = &report.apply_artifact {
                println!("下一计划: {}", artifact.plan_path);
                if let Some(path) = &artifact.patch_path {
                    println!("下一补丁: {path}");
                }
            }
            if let Some(error) = &report.recommendation_error {
                println!("推荐错误: {error}");
            }
            println!("近期 repair 结果: {}", report.recent.len());
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

fn mirror_repair_score_to_evolution_outcome(
    state: &mut HarnessState,
    trace: Option<&FeedTraceRecord>,
    outcome: &RepairOutcome,
) -> Option<EvolutionOutcome> {
    let target = repair_score_outcome_or_trace_value(
        outcome.target_tentacle.as_deref(),
        trace,
        "target_tentacle",
    )?;
    let candidate =
        repair_score_outcome_or_trace_value(outcome.candidate.as_deref(), trace, "candidate")?;
    if !repair_score_evolution_value_ready(target) || !repair_score_evolution_value_ready(candidate)
    {
        return None;
    }
    let summary = format!("repair outcome #{}: {}", outcome.index, outcome.summary);
    Some(state.record_repair_linked_evolution_outcome(
        target.to_string(),
        candidate.to_string(),
        outcome.status.clone(),
        summary,
    ))
}

fn repair_score_outcome_or_trace_value<'a>(
    outcome_value: Option<&'a str>,
    trace: Option<&'a FeedTraceRecord>,
    key: &str,
) -> Option<&'a str> {
    outcome_value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| trace?.metadata.get(key).map(String::as_str).map(str::trim))
}

fn repair_json_feed_label(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.trim().to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Array(values) => values
            .iter()
            .map(repair_json_feed_label)
            .filter(|value| !value.trim().is_empty())
            .collect::<Vec<_>>()
            .join(","),
        _ => String::new(),
    }
}

fn repair_sidecar_value(sidecar: Option<&serde_json::Value>, key: &str) -> Option<String> {
    let value = repair_json_feed_label(sidecar?.get(key)?);
    (!value.trim().is_empty()).then_some(value)
}

fn repair_sidecar_nested_value(
    sidecar: Option<&serde_json::Value>,
    parent: &str,
    key: &str,
) -> Option<String> {
    let value = repair_json_feed_label(sidecar?.get(parent)?.get(key)?);
    (!value.trim().is_empty()).then_some(value)
}

fn repair_metadata_or_sidecar(
    trace: &FeedTraceRecord,
    sidecar: Option<&serde_json::Value>,
    metadata_key: &str,
    sidecar_key: &str,
) -> String {
    trace
        .metadata
        .get(metadata_key)
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| repair_sidecar_value(sidecar, sidecar_key))
        .unwrap_or_default()
}

fn repair_score_evolution_value_ready(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() {
        return false;
    }
    !matches!(
        value.to_ascii_lowercase().as_str(),
        "none" | "unknown" | "recommended"
    )
}

fn write_repair_score_journal(
    cwd: &Path,
    trace: Option<&FeedTraceRecord>,
    outcome: &RepairOutcome,
) -> Result<Option<RepairOutcomeJournalReport>, String> {
    let Some(trace) = trace else {
        return Ok(None);
    };
    let Some(session) = trace.metadata.get("session") else {
        return Ok(None);
    };
    let workspace = trace
        .metadata
        .get("workspace")
        .map(PathBuf::from)
        .unwrap_or_else(|| cwd.to_path_buf());
    let session_path = metadata_path(&workspace, session);
    let Some(session_dir) = session_path.parent() else {
        return Ok(None);
    };
    if !session_path.exists() {
        return Ok(None);
    }
    let repair_root = workspace.join(".octopus").join("harness-repair");
    fs::create_dir_all(&repair_root).map_err(|error| error.to_string())?;
    let outcomes_file = repair_root.join("outcomes.jsonl");
    let outcome_path = session_dir.join("OUTCOME.md");
    let repair_queue_brief_path = session_dir.join("REPAIR_QUEUE_BRIEF.json");
    let repair_queue_brief_sidecar = fs::read_to_string(&repair_queue_brief_path)
        .ok()
        .and_then(|payload| serde_json::from_str::<serde_json::Value>(&payload).ok());
    let status = cli_status_key(&outcome.status);
    let target = outcome
        .target_tentacle
        .clone()
        .or_else(|| trace.metadata.get("target_tentacle").cloned())
        .unwrap_or_else(|| outcome.tentacle_id.clone());
    let candidate = outcome
        .candidate
        .clone()
        .or_else(|| trace.metadata.get("candidate").cloned())
        .unwrap_or_else(|| "none".to_string());
    let draft_status = trace
        .metadata
        .get("llm_draft_status")
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());
    let draft_prefix = trace
        .metadata
        .get("llm_prefix")
        .or_else(|| trace.metadata.get("draft_prefix"))
        .cloned()
        .unwrap_or_default();
    let draft_model = trace
        .metadata
        .get("llm_model")
        .or_else(|| trace.metadata.get("draft_model"))
        .cloned()
        .unwrap_or_default();
    let repair_command_check = trace
        .metadata
        .get("check_command")
        .cloned()
        .unwrap_or_default();
    let repair_command_grant = trace
        .metadata
        .get("grant_command")
        .cloned()
        .unwrap_or_default();
    let repair_command_apply = trace
        .metadata
        .get("apply_command")
        .cloned()
        .unwrap_or_default();
    let repair_command_score = trace
        .metadata
        .get("score_command")
        .cloned()
        .unwrap_or_default();
    let repair_command_score_options = trace
        .metadata
        .get("score_commands")
        .cloned()
        .unwrap_or_default();
    let repair_effectiveness_rollup_json = trace
        .metadata
        .get("repair_effectiveness_rollup_json")
        .cloned()
        .unwrap_or_default();
    let repair_effectiveness_rollup_status = trace
        .metadata
        .get("repair_effectiveness_rollup_status")
        .cloned()
        .unwrap_or_default();
    let repair_effectiveness_rollup_source_count = trace
        .metadata
        .get("repair_effectiveness_rollup_source_count")
        .cloned()
        .unwrap_or_default();
    let repair_effectiveness_rollup_active_source_count = trace
        .metadata
        .get("repair_effectiveness_rollup_active_source_count")
        .cloned()
        .unwrap_or_default();
    let repair_effectiveness_rollup_used_count = trace
        .metadata
        .get("repair_effectiveness_rollup_used_count")
        .cloned()
        .unwrap_or_default();
    let repair_effectiveness_rollup_satisfied_count = trace
        .metadata
        .get("repair_effectiveness_rollup_satisfied_count")
        .cloned()
        .unwrap_or_default();
    let repair_effectiveness_rollup_partial_count = trace
        .metadata
        .get("repair_effectiveness_rollup_partial_count")
        .cloned()
        .unwrap_or_default();
    let repair_effectiveness_rollup_failed_count = trace
        .metadata
        .get("repair_effectiveness_rollup_failed_count")
        .cloned()
        .unwrap_or_default();
    let repair_effectiveness_rollup_success_rate = trace
        .metadata
        .get("repair_effectiveness_rollup_success_rate")
        .cloned()
        .unwrap_or_default();
    let repair_effectiveness_rollup_failure_rate = trace
        .metadata
        .get("repair_effectiveness_rollup_failure_rate")
        .cloned()
        .unwrap_or_default();
    let repair_effectiveness_rollup_strongest_source = trace
        .metadata
        .get("repair_effectiveness_rollup_strongest_source")
        .cloned()
        .unwrap_or_default();
    let repair_effectiveness_rollup_weakest_source = trace
        .metadata
        .get("repair_effectiveness_rollup_weakest_source")
        .cloned()
        .unwrap_or_default();
    let repair_effectiveness_rollup_top_reuse = trace
        .metadata
        .get("repair_effectiveness_rollup_top_reuse")
        .cloned()
        .unwrap_or_default();
    let repair_effectiveness_rollup_top_avoid = trace
        .metadata
        .get("repair_effectiveness_rollup_top_avoid")
        .cloned()
        .unwrap_or_default();
    let repair_effectiveness_rollup_next_need_kind = trace
        .metadata
        .get("repair_effectiveness_rollup_next_need_kind")
        .cloned()
        .unwrap_or_default();
    let repair_effectiveness_rollup_next_need_query = trace
        .metadata
        .get("repair_effectiveness_rollup_next_need_query")
        .cloned()
        .unwrap_or_default();
    let repair_queue_brief_json = trace
        .metadata
        .get("repair_queue_brief_json")
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            repair_queue_brief_path
                .exists()
                .then(|| display_path(&workspace, &repair_queue_brief_path))
        })
        .unwrap_or_default();
    let repair_queue_brief_sidecar = repair_queue_brief_sidecar.as_ref();
    let repair_queue_brief_status = repair_metadata_or_sidecar(
        trace,
        repair_queue_brief_sidecar,
        "repair_queue_brief_status",
        "status",
    );
    let repair_queue_brief_plan_status = repair_metadata_or_sidecar(
        trace,
        repair_queue_brief_sidecar,
        "repair_queue_brief_plan_status",
        "plan_status",
    );
    let repair_queue_brief_source = repair_metadata_or_sidecar(
        trace,
        repair_queue_brief_sidecar,
        "repair_queue_brief_source",
        "source",
    );
    let repair_queue_brief_blockers = repair_metadata_or_sidecar(
        trace,
        repair_queue_brief_sidecar,
        "repair_queue_brief_blockers",
        "blockers",
    );
    let repair_queue_brief_review_first = repair_metadata_or_sidecar(
        trace,
        repair_queue_brief_sidecar,
        "repair_queue_brief_review_first",
        "review_first",
    );
    let repair_queue_brief_next_need_kind = trace
        .metadata
        .get("repair_queue_brief_next_need_kind")
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| repair_sidecar_nested_value(repair_queue_brief_sidecar, "next_need", "kind"))
        .unwrap_or_default();
    let repair_queue_brief_next_need_query = trace
        .metadata
        .get("repair_queue_brief_next_need_query")
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| repair_sidecar_nested_value(repair_queue_brief_sidecar, "next_need", "query"))
        .unwrap_or_default();
    let action_trace_json = trace
        .metadata
        .get("action_trace_json")
        .cloned()
        .unwrap_or_default();
    let action_trace_status = trace
        .metadata
        .get("action_trace_status")
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());
    let action_trace_stage_count = trace
        .metadata
        .get("action_trace_stage_count")
        .cloned()
        .unwrap_or_default();
    let action_trace_last_action = trace
        .metadata
        .get("action_trace_last_action")
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_hint = trace
        .metadata
        .get("action_trace_repair_hint")
        .cloned()
        .unwrap_or_default();
    let action_trace_recall_count = trace
        .metadata
        .get("action_trace_recall_count")
        .cloned()
        .unwrap_or_default();
    let action_trace_recall_top_status = trace
        .metadata
        .get("action_trace_recall_top_status")
        .cloned()
        .unwrap_or_default();
    let action_trace_recall_top_score = trace
        .metadata
        .get("action_trace_recall_top_score")
        .cloned()
        .unwrap_or_default();
    let action_trace_recall_top_reasons = trace
        .metadata
        .get("action_trace_recall_top_reasons")
        .cloned()
        .unwrap_or_default();
    let action_trace_recall_top_summary = trace
        .metadata
        .get("action_trace_recall_top_summary")
        .cloned()
        .unwrap_or_default();
    let action_trace_lesson_count = trace
        .metadata
        .get("action_trace_lesson_count")
        .cloned()
        .unwrap_or_default();
    let action_trace_lesson_reuse_count = trace
        .metadata
        .get("action_trace_lesson_reuse_count")
        .cloned()
        .unwrap_or_default();
    let action_trace_lesson_avoid_count = trace
        .metadata
        .get("action_trace_lesson_avoid_count")
        .cloned()
        .unwrap_or_default();
    let action_trace_lesson_top_reuse = trace
        .metadata
        .get("action_trace_lesson_top_reuse")
        .cloned()
        .unwrap_or_default();
    let action_trace_lesson_top_avoid = trace
        .metadata
        .get("action_trace_lesson_top_avoid")
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_draft_strategy_status = trace
        .metadata
        .get("action_trace_repair_draft_strategy_status")
        .or_else(|| trace.metadata.get("repair_draft_strategy_status"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_draft_strategy_focus = trace
        .metadata
        .get("action_trace_repair_draft_strategy_focus")
        .or_else(|| trace.metadata.get("repair_draft_strategy_focus"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_draft_strategy_learned_reuse = trace
        .metadata
        .get("action_trace_repair_draft_strategy_learned_reuse")
        .or_else(|| trace.metadata.get("repair_draft_strategy_learned_reuse"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_draft_strategy_learned_avoid = trace
        .metadata
        .get("action_trace_repair_draft_strategy_learned_avoid")
        .or_else(|| trace.metadata.get("repair_draft_strategy_learned_avoid"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_draft_strategy_next_need_kind = trace
        .metadata
        .get("action_trace_repair_draft_strategy_next_need_kind")
        .or_else(|| trace.metadata.get("repair_draft_strategy_next_need_kind"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_draft_strategy_next_need_query = trace
        .metadata
        .get("action_trace_repair_draft_strategy_next_need_query")
        .or_else(|| trace.metadata.get("repair_draft_strategy_next_need_query"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_draft_strategy_effectiveness_used_count = trace
        .metadata
        .get("action_trace_repair_draft_strategy_effectiveness_used_count")
        .or_else(|| {
            trace
                .metadata
                .get("repair_draft_strategy_effectiveness_used_count")
        })
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_draft_strategy_effectiveness_success_rate = trace
        .metadata
        .get("action_trace_repair_draft_strategy_effectiveness_success_rate")
        .or_else(|| {
            trace
                .metadata
                .get("repair_draft_strategy_effectiveness_success_rate")
        })
        .cloned()
        .unwrap_or_default();
    let action_trace_command_strategy_status = trace
        .metadata
        .get("action_trace_command_strategy_status")
        .or_else(|| trace.metadata.get("repair_command_strategy_status"))
        .cloned()
        .unwrap_or_default();
    let action_trace_command_strategy_focus = trace
        .metadata
        .get("action_trace_command_strategy_focus")
        .or_else(|| trace.metadata.get("repair_command_strategy_focus"))
        .cloned()
        .unwrap_or_default();
    let action_trace_command_strategy_learned_reuse = trace
        .metadata
        .get("action_trace_command_strategy_learned_reuse")
        .or_else(|| trace.metadata.get("repair_command_strategy_learned_reuse"))
        .cloned()
        .unwrap_or_default();
    let action_trace_command_strategy_learned_avoid = trace
        .metadata
        .get("action_trace_command_strategy_learned_avoid")
        .or_else(|| trace.metadata.get("repair_command_strategy_learned_avoid"))
        .cloned()
        .unwrap_or_default();
    let action_trace_command_strategy_next_need_kind = trace
        .metadata
        .get("action_trace_command_strategy_next_need_kind")
        .or_else(|| trace.metadata.get("repair_command_strategy_next_need_kind"))
        .cloned()
        .unwrap_or_default();
    let action_trace_command_strategy_next_need_query = trace
        .metadata
        .get("action_trace_command_strategy_next_need_query")
        .or_else(|| {
            trace
                .metadata
                .get("repair_command_strategy_next_need_query")
        })
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_draft_status = trace
        .metadata
        .get("action_trace_repair_patch_draft_status")
        .or_else(|| trace.metadata.get("repair_patch_draft_status"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_draft_has_patch = trace
        .metadata
        .get("action_trace_repair_patch_draft_has_patch")
        .or_else(|| trace.metadata.get("repair_patch_draft_has_patch"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_draft_target = trace
        .metadata
        .get("action_trace_repair_patch_draft_target")
        .or_else(|| trace.metadata.get("repair_patch_draft_target"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_draft_preview = trace
        .metadata
        .get("action_trace_repair_patch_draft_preview")
        .or_else(|| trace.metadata.get("repair_patch_draft_preview"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_review_status = trace
        .metadata
        .get("action_trace_repair_patch_review_status")
        .or_else(|| trace.metadata.get("repair_patch_review_status"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_review_check_status = trace
        .metadata
        .get("action_trace_repair_patch_review_check_status")
        .or_else(|| trace.metadata.get("repair_patch_review_check_status"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_review_has_patch = trace
        .metadata
        .get("action_trace_repair_patch_review_has_patch")
        .or_else(|| trace.metadata.get("repair_patch_review_has_patch"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_review_target = trace
        .metadata
        .get("action_trace_repair_patch_review_target")
        .or_else(|| trace.metadata.get("repair_patch_review_target"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_review_summary = trace
        .metadata
        .get("action_trace_repair_patch_review_summary")
        .or_else(|| trace.metadata.get("repair_patch_review_summary"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_apply_status = trace
        .metadata
        .get("action_trace_repair_patch_apply_status")
        .or_else(|| trace.metadata.get("repair_patch_apply_status"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_apply_applied = trace
        .metadata
        .get("action_trace_repair_patch_apply_applied")
        .or_else(|| trace.metadata.get("repair_patch_apply_applied"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_apply_target = trace
        .metadata
        .get("action_trace_repair_patch_apply_target")
        .or_else(|| trace.metadata.get("repair_patch_apply_target"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_apply_summary = trace
        .metadata
        .get("action_trace_repair_patch_apply_summary")
        .or_else(|| trace.metadata.get("repair_patch_apply_summary"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_apply_effectiveness_used_count = trace
        .metadata
        .get("action_trace_repair_patch_apply_effectiveness_used_count")
        .or_else(|| {
            trace
                .metadata
                .get("repair_patch_apply_effectiveness_used_count")
        })
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_apply_effectiveness_success_rate = trace
        .metadata
        .get("action_trace_repair_patch_apply_effectiveness_success_rate")
        .or_else(|| {
            trace
                .metadata
                .get("repair_patch_apply_effectiveness_success_rate")
        })
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_verify_status = trace
        .metadata
        .get("action_trace_repair_patch_verify_status")
        .or_else(|| trace.metadata.get("repair_patch_verify_status"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_verify_passed = trace
        .metadata
        .get("action_trace_repair_patch_verify_passed")
        .or_else(|| trace.metadata.get("repair_patch_verify_passed"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_verify_target = trace
        .metadata
        .get("action_trace_repair_patch_verify_target")
        .or_else(|| trace.metadata.get("repair_patch_verify_target"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_verify_command = trace
        .metadata
        .get("action_trace_repair_patch_verify_command")
        .or_else(|| trace.metadata.get("repair_patch_verify_command"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_verify_summary = trace
        .metadata
        .get("action_trace_repair_patch_verify_summary")
        .or_else(|| trace.metadata.get("repair_patch_verify_summary"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_verify_effectiveness_used_count = trace
        .metadata
        .get("action_trace_repair_patch_verify_effectiveness_used_count")
        .or_else(|| {
            trace
                .metadata
                .get("repair_patch_verify_effectiveness_used_count")
        })
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_verify_effectiveness_success_rate = trace
        .metadata
        .get("action_trace_repair_patch_verify_effectiveness_success_rate")
        .or_else(|| {
            trace
                .metadata
                .get("repair_patch_verify_effectiveness_success_rate")
        })
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_learning_status = trace
        .metadata
        .get("action_trace_repair_patch_learning_status")
        .or_else(|| trace.metadata.get("repair_patch_learning_status"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_learning_used_count = trace
        .metadata
        .get("action_trace_repair_patch_learning_used_count")
        .or_else(|| trace.metadata.get("repair_patch_learning_used_count"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_learning_verified_count = trace
        .metadata
        .get("action_trace_repair_patch_learning_verified_count")
        .or_else(|| trace.metadata.get("repair_patch_learning_verified_count"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_learning_verified_success_rate = trace
        .metadata
        .get("action_trace_repair_patch_learning_verified_success_rate")
        .or_else(|| {
            trace
                .metadata
                .get("repair_patch_learning_verified_success_rate")
        })
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_learning_top_reuse = trace
        .metadata
        .get("action_trace_repair_patch_learning_top_reuse")
        .or_else(|| trace.metadata.get("repair_patch_learning_top_reuse"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_learning_top_avoid = trace
        .metadata
        .get("action_trace_repair_patch_learning_top_avoid")
        .or_else(|| trace.metadata.get("repair_patch_learning_top_avoid"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_learning_next_need_kind = trace
        .metadata
        .get("action_trace_repair_patch_learning_next_need_kind")
        .or_else(|| trace.metadata.get("repair_patch_learning_next_need_kind"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_learning_next_need_query = trace
        .metadata
        .get("action_trace_repair_patch_learning_next_need_query")
        .or_else(|| trace.metadata.get("repair_patch_learning_next_need_query"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_strategy_status = trace
        .metadata
        .get("action_trace_repair_patch_strategy_status")
        .or_else(|| trace.metadata.get("repair_patch_strategy_status"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_strategy_focus = trace
        .metadata
        .get("action_trace_repair_patch_strategy_focus")
        .or_else(|| trace.metadata.get("repair_patch_strategy_focus"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_strategy_learned_reuse = trace
        .metadata
        .get("action_trace_repair_patch_strategy_learned_reuse")
        .or_else(|| trace.metadata.get("repair_patch_strategy_learned_reuse"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_strategy_learned_avoid = trace
        .metadata
        .get("action_trace_repair_patch_strategy_learned_avoid")
        .or_else(|| trace.metadata.get("repair_patch_strategy_learned_avoid"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_strategy_next_need_kind = trace
        .metadata
        .get("action_trace_repair_patch_strategy_next_need_kind")
        .or_else(|| trace.metadata.get("repair_patch_strategy_next_need_kind"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_patch_strategy_next_need_query = trace
        .metadata
        .get("action_trace_repair_patch_strategy_next_need_query")
        .or_else(|| trace.metadata.get("repair_patch_strategy_next_need_query"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_decision = trace
        .metadata
        .get("action_trace_repair_decision")
        .or_else(|| trace.metadata.get("repair_decision_kind"))
        .cloned()
        .unwrap_or_default();
    let action_trace_repair_decision_focus = trace
        .metadata
        .get("action_trace_repair_decision_focus")
        .or_else(|| trace.metadata.get("repair_decision_focus"))
        .cloned()
        .unwrap_or_default();
    let action_trace_harness_environment_drift_status = trace
        .metadata
        .get("action_trace_harness_environment_drift_status")
        .or_else(|| trace.metadata.get("harness_environment_drift_status"))
        .cloned()
        .unwrap_or_default();
    let action_trace_harness_environment_drift_detail = trace
        .metadata
        .get("action_trace_harness_environment_drift_detail")
        .or_else(|| trace.metadata.get("harness_environment_drift_detail"))
        .cloned()
        .unwrap_or_default();
    let action_trace_harness_environment_drift_history_count = trace
        .metadata
        .get("action_trace_harness_environment_drift_history_count")
        .or_else(|| {
            trace
                .metadata
                .get("harness_environment_drift_history_count")
        })
        .cloned()
        .unwrap_or_default();
    let action_trace_harness_environment_drift_gained_count = trace
        .metadata
        .get("action_trace_harness_environment_drift_gained_count")
        .or_else(|| trace.metadata.get("harness_environment_drift_gained_count"))
        .cloned()
        .unwrap_or_default();
    let action_trace_harness_environment_drift_lost_count = trace
        .metadata
        .get("action_trace_harness_environment_drift_lost_count")
        .or_else(|| trace.metadata.get("harness_environment_drift_lost_count"))
        .cloned()
        .unwrap_or_default();
    let action_trace_harness_environment_drift_next_need_kind = trace
        .metadata
        .get("action_trace_harness_environment_drift_next_need_kind")
        .or_else(|| {
            trace
                .metadata
                .get("harness_environment_drift_next_need_kind")
        })
        .cloned()
        .unwrap_or_default();
    let action_trace_harness_environment_drift_next_need_query = trace
        .metadata
        .get("action_trace_harness_environment_drift_next_need_query")
        .or_else(|| {
            trace
                .metadata
                .get("harness_environment_drift_next_need_query")
        })
        .cloned()
        .unwrap_or_default();
    let action_trace_harness_adaptation_status = trace
        .metadata
        .get("action_trace_harness_adaptation")
        .or_else(|| trace.metadata.get("action_trace_harness_adaptation_status"))
        .cloned()
        .unwrap_or_default();
    let action_trace_harness_adaptation_focus = trace
        .metadata
        .get("action_trace_harness_adaptation_focus")
        .cloned()
        .unwrap_or_default();
    let timestamp_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_secs();
    let mut record = serde_json::json!({
        "schema_version": "octopus-harness-repair-outcome-v1",
        "timestamp_secs": timestamp_secs,
        "workspace": workspace.to_string_lossy(),
        "session": display_path(&workspace, &session_path),
        "trace_index": outcome.trace_index,
        "target_tentacle": target,
        "candidate": candidate,
        "source": trace.metadata.get("source").cloned().unwrap_or_else(|| "repair_score".to_string()),
        "next_need": trace.metadata.get("next_need").cloned().unwrap_or_default(),
        "draft_status": draft_status,
        "draft_prefix": draft_prefix,
        "draft_model": draft_model,
        "action_trace_json": action_trace_json,
        "action_trace_status": action_trace_status,
        "action_trace_stage_count": action_trace_stage_count,
        "action_trace_last_action": action_trace_last_action,
        "action_trace_repair_hint": action_trace_repair_hint,
        "action_trace_recall_count": action_trace_recall_count,
        "action_trace_recall_top_status": action_trace_recall_top_status,
        "action_trace_recall_top_score": action_trace_recall_top_score,
        "action_trace_recall_top_reasons": action_trace_recall_top_reasons,
        "action_trace_recall_top_summary": action_trace_recall_top_summary,
        "action_trace_lesson_count": action_trace_lesson_count,
        "action_trace_lesson_reuse_count": action_trace_lesson_reuse_count,
        "action_trace_lesson_avoid_count": action_trace_lesson_avoid_count,
        "action_trace_lesson_top_reuse": action_trace_lesson_top_reuse,
        "action_trace_lesson_top_avoid": action_trace_lesson_top_avoid,
        "action_trace_harness_environment_drift_status": action_trace_harness_environment_drift_status,
        "action_trace_harness_environment_drift_detail": action_trace_harness_environment_drift_detail,
        "action_trace_harness_environment_drift_history_count": action_trace_harness_environment_drift_history_count,
        "action_trace_harness_environment_drift_gained_count": action_trace_harness_environment_drift_gained_count,
        "action_trace_harness_environment_drift_lost_count": action_trace_harness_environment_drift_lost_count,
        "action_trace_harness_environment_drift_next_need_kind": action_trace_harness_environment_drift_next_need_kind,
        "action_trace_harness_environment_drift_next_need_query": action_trace_harness_environment_drift_next_need_query,
        "action_trace_harness_adaptation_status": action_trace_harness_adaptation_status,
        "action_trace_harness_adaptation_focus": action_trace_harness_adaptation_focus,
        "outcome_status": status,
        "summary": truncate_chars(&outcome.summary, 500),
    });
    if let Some(record) = record.as_object_mut() {
        record.insert(
            "repair_command_check".to_string(),
            serde_json::Value::String(repair_command_check),
        );
        record.insert(
            "repair_command_grant".to_string(),
            serde_json::Value::String(repair_command_grant),
        );
        record.insert(
            "repair_command_apply".to_string(),
            serde_json::Value::String(repair_command_apply),
        );
        record.insert(
            "repair_command_score".to_string(),
            serde_json::Value::String(repair_command_score),
        );
        record.insert(
            "repair_command_score_options".to_string(),
            serde_json::Value::String(repair_command_score_options),
        );
        record.insert(
            "repair_effectiveness_rollup_json".to_string(),
            serde_json::Value::String(repair_effectiveness_rollup_json),
        );
        record.insert(
            "repair_effectiveness_rollup_status".to_string(),
            serde_json::Value::String(repair_effectiveness_rollup_status),
        );
        record.insert(
            "repair_effectiveness_rollup_source_count".to_string(),
            serde_json::Value::String(repair_effectiveness_rollup_source_count),
        );
        record.insert(
            "repair_effectiveness_rollup_active_source_count".to_string(),
            serde_json::Value::String(repair_effectiveness_rollup_active_source_count),
        );
        record.insert(
            "repair_effectiveness_rollup_used_count".to_string(),
            serde_json::Value::String(repair_effectiveness_rollup_used_count),
        );
        record.insert(
            "repair_effectiveness_rollup_satisfied_count".to_string(),
            serde_json::Value::String(repair_effectiveness_rollup_satisfied_count),
        );
        record.insert(
            "repair_effectiveness_rollup_partial_count".to_string(),
            serde_json::Value::String(repair_effectiveness_rollup_partial_count),
        );
        record.insert(
            "repair_effectiveness_rollup_failed_count".to_string(),
            serde_json::Value::String(repair_effectiveness_rollup_failed_count),
        );
        record.insert(
            "repair_effectiveness_rollup_success_rate".to_string(),
            serde_json::Value::String(repair_effectiveness_rollup_success_rate),
        );
        record.insert(
            "repair_effectiveness_rollup_failure_rate".to_string(),
            serde_json::Value::String(repair_effectiveness_rollup_failure_rate),
        );
        record.insert(
            "repair_effectiveness_rollup_strongest_source".to_string(),
            serde_json::Value::String(repair_effectiveness_rollup_strongest_source),
        );
        record.insert(
            "repair_effectiveness_rollup_weakest_source".to_string(),
            serde_json::Value::String(repair_effectiveness_rollup_weakest_source),
        );
        record.insert(
            "repair_effectiveness_rollup_top_reuse".to_string(),
            serde_json::Value::String(repair_effectiveness_rollup_top_reuse),
        );
        record.insert(
            "repair_effectiveness_rollup_top_avoid".to_string(),
            serde_json::Value::String(repair_effectiveness_rollup_top_avoid),
        );
        record.insert(
            "repair_effectiveness_rollup_next_need_kind".to_string(),
            serde_json::Value::String(repair_effectiveness_rollup_next_need_kind),
        );
        record.insert(
            "repair_effectiveness_rollup_next_need_query".to_string(),
            serde_json::Value::String(repair_effectiveness_rollup_next_need_query),
        );
        record.insert(
            "repair_queue_brief_json".to_string(),
            serde_json::Value::String(repair_queue_brief_json),
        );
        record.insert(
            "repair_queue_brief_status".to_string(),
            serde_json::Value::String(repair_queue_brief_status),
        );
        record.insert(
            "repair_queue_brief_plan_status".to_string(),
            serde_json::Value::String(repair_queue_brief_plan_status),
        );
        record.insert(
            "repair_queue_brief_source".to_string(),
            serde_json::Value::String(repair_queue_brief_source),
        );
        record.insert(
            "repair_queue_brief_blockers".to_string(),
            serde_json::Value::String(repair_queue_brief_blockers),
        );
        record.insert(
            "repair_queue_brief_review_first".to_string(),
            serde_json::Value::String(repair_queue_brief_review_first),
        );
        record.insert(
            "repair_queue_brief_next_need_kind".to_string(),
            serde_json::Value::String(repair_queue_brief_next_need_kind),
        );
        record.insert(
            "repair_queue_brief_next_need_query".to_string(),
            serde_json::Value::String(repair_queue_brief_next_need_query),
        );
        record.insert(
            "action_trace_repair_draft_strategy_status".to_string(),
            serde_json::Value::String(action_trace_repair_draft_strategy_status),
        );
        record.insert(
            "action_trace_repair_draft_strategy_focus".to_string(),
            serde_json::Value::String(action_trace_repair_draft_strategy_focus),
        );
        record.insert(
            "action_trace_repair_draft_strategy_learned_reuse".to_string(),
            serde_json::Value::String(action_trace_repair_draft_strategy_learned_reuse),
        );
        record.insert(
            "action_trace_repair_draft_strategy_learned_avoid".to_string(),
            serde_json::Value::String(action_trace_repair_draft_strategy_learned_avoid),
        );
        record.insert(
            "action_trace_repair_draft_strategy_next_need_kind".to_string(),
            serde_json::Value::String(action_trace_repair_draft_strategy_next_need_kind),
        );
        record.insert(
            "action_trace_repair_draft_strategy_next_need_query".to_string(),
            serde_json::Value::String(action_trace_repair_draft_strategy_next_need_query),
        );
        record.insert(
            "action_trace_repair_draft_strategy_effectiveness_used_count".to_string(),
            serde_json::Value::String(action_trace_repair_draft_strategy_effectiveness_used_count),
        );
        record.insert(
            "action_trace_repair_draft_strategy_effectiveness_success_rate".to_string(),
            serde_json::Value::String(
                action_trace_repair_draft_strategy_effectiveness_success_rate,
            ),
        );
        record.insert(
            "action_trace_command_strategy_status".to_string(),
            serde_json::Value::String(action_trace_command_strategy_status),
        );
        record.insert(
            "action_trace_command_strategy_focus".to_string(),
            serde_json::Value::String(action_trace_command_strategy_focus),
        );
        record.insert(
            "action_trace_command_strategy_learned_reuse".to_string(),
            serde_json::Value::String(action_trace_command_strategy_learned_reuse),
        );
        record.insert(
            "action_trace_command_strategy_learned_avoid".to_string(),
            serde_json::Value::String(action_trace_command_strategy_learned_avoid),
        );
        record.insert(
            "action_trace_command_strategy_next_need_kind".to_string(),
            serde_json::Value::String(action_trace_command_strategy_next_need_kind),
        );
        record.insert(
            "action_trace_command_strategy_next_need_query".to_string(),
            serde_json::Value::String(action_trace_command_strategy_next_need_query),
        );
        record.insert(
            "action_trace_repair_patch_draft_status".to_string(),
            serde_json::Value::String(action_trace_repair_patch_draft_status),
        );
        record.insert(
            "action_trace_repair_patch_draft_has_patch".to_string(),
            serde_json::Value::String(action_trace_repair_patch_draft_has_patch),
        );
        record.insert(
            "action_trace_repair_patch_draft_target".to_string(),
            serde_json::Value::String(action_trace_repair_patch_draft_target),
        );
        record.insert(
            "action_trace_repair_patch_draft_preview".to_string(),
            serde_json::Value::String(action_trace_repair_patch_draft_preview),
        );
        record.insert(
            "action_trace_repair_patch_review_status".to_string(),
            serde_json::Value::String(action_trace_repair_patch_review_status),
        );
        record.insert(
            "action_trace_repair_patch_review_check_status".to_string(),
            serde_json::Value::String(action_trace_repair_patch_review_check_status),
        );
        record.insert(
            "action_trace_repair_patch_review_has_patch".to_string(),
            serde_json::Value::String(action_trace_repair_patch_review_has_patch),
        );
        record.insert(
            "action_trace_repair_patch_review_target".to_string(),
            serde_json::Value::String(action_trace_repair_patch_review_target),
        );
        record.insert(
            "action_trace_repair_patch_review_summary".to_string(),
            serde_json::Value::String(action_trace_repair_patch_review_summary),
        );
        record.insert(
            "action_trace_repair_patch_apply_status".to_string(),
            serde_json::Value::String(action_trace_repair_patch_apply_status),
        );
        record.insert(
            "action_trace_repair_patch_apply_applied".to_string(),
            serde_json::Value::String(action_trace_repair_patch_apply_applied),
        );
        record.insert(
            "action_trace_repair_patch_apply_target".to_string(),
            serde_json::Value::String(action_trace_repair_patch_apply_target),
        );
        record.insert(
            "action_trace_repair_patch_apply_summary".to_string(),
            serde_json::Value::String(action_trace_repair_patch_apply_summary),
        );
        record.insert(
            "action_trace_repair_patch_apply_effectiveness_used_count".to_string(),
            serde_json::Value::String(action_trace_repair_patch_apply_effectiveness_used_count),
        );
        record.insert(
            "action_trace_repair_patch_apply_effectiveness_success_rate".to_string(),
            serde_json::Value::String(action_trace_repair_patch_apply_effectiveness_success_rate),
        );
        record.insert(
            "action_trace_repair_patch_verify_status".to_string(),
            serde_json::Value::String(action_trace_repair_patch_verify_status),
        );
        record.insert(
            "action_trace_repair_patch_verify_passed".to_string(),
            serde_json::Value::String(action_trace_repair_patch_verify_passed),
        );
        record.insert(
            "action_trace_repair_patch_verify_target".to_string(),
            serde_json::Value::String(action_trace_repair_patch_verify_target),
        );
        record.insert(
            "action_trace_repair_patch_verify_command".to_string(),
            serde_json::Value::String(action_trace_repair_patch_verify_command),
        );
        record.insert(
            "action_trace_repair_patch_verify_summary".to_string(),
            serde_json::Value::String(action_trace_repair_patch_verify_summary),
        );
        record.insert(
            "action_trace_repair_patch_verify_effectiveness_used_count".to_string(),
            serde_json::Value::String(action_trace_repair_patch_verify_effectiveness_used_count),
        );
        record.insert(
            "action_trace_repair_patch_verify_effectiveness_success_rate".to_string(),
            serde_json::Value::String(action_trace_repair_patch_verify_effectiveness_success_rate),
        );
        record.insert(
            "action_trace_repair_patch_learning_status".to_string(),
            serde_json::Value::String(action_trace_repair_patch_learning_status),
        );
        record.insert(
            "action_trace_repair_patch_learning_used_count".to_string(),
            serde_json::Value::String(action_trace_repair_patch_learning_used_count),
        );
        record.insert(
            "action_trace_repair_patch_learning_verified_count".to_string(),
            serde_json::Value::String(action_trace_repair_patch_learning_verified_count),
        );
        record.insert(
            "action_trace_repair_patch_learning_verified_success_rate".to_string(),
            serde_json::Value::String(action_trace_repair_patch_learning_verified_success_rate),
        );
        record.insert(
            "action_trace_repair_patch_learning_top_reuse".to_string(),
            serde_json::Value::String(action_trace_repair_patch_learning_top_reuse),
        );
        record.insert(
            "action_trace_repair_patch_learning_top_avoid".to_string(),
            serde_json::Value::String(action_trace_repair_patch_learning_top_avoid),
        );
        record.insert(
            "action_trace_repair_patch_learning_next_need_kind".to_string(),
            serde_json::Value::String(action_trace_repair_patch_learning_next_need_kind),
        );
        record.insert(
            "action_trace_repair_patch_learning_next_need_query".to_string(),
            serde_json::Value::String(action_trace_repair_patch_learning_next_need_query),
        );
        record.insert(
            "action_trace_repair_patch_strategy_status".to_string(),
            serde_json::Value::String(action_trace_repair_patch_strategy_status),
        );
        record.insert(
            "action_trace_repair_patch_strategy_focus".to_string(),
            serde_json::Value::String(action_trace_repair_patch_strategy_focus),
        );
        record.insert(
            "action_trace_repair_patch_strategy_learned_reuse".to_string(),
            serde_json::Value::String(action_trace_repair_patch_strategy_learned_reuse),
        );
        record.insert(
            "action_trace_repair_patch_strategy_learned_avoid".to_string(),
            serde_json::Value::String(action_trace_repair_patch_strategy_learned_avoid),
        );
        record.insert(
            "action_trace_repair_patch_strategy_next_need_kind".to_string(),
            serde_json::Value::String(action_trace_repair_patch_strategy_next_need_kind),
        );
        record.insert(
            "action_trace_repair_patch_strategy_next_need_query".to_string(),
            serde_json::Value::String(action_trace_repair_patch_strategy_next_need_query),
        );
        record.insert(
            "action_trace_repair_decision".to_string(),
            serde_json::Value::String(action_trace_repair_decision),
        );
        record.insert(
            "action_trace_repair_decision_focus".to_string(),
            serde_json::Value::String(action_trace_repair_decision_focus),
        );
    }
    let mut handle = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&outcomes_file)
        .map_err(|error| error.to_string())?;
    writeln!(handle, "{record}").map_err(|error| error.to_string())?;
    fs::write(
        &outcome_path,
        format!(
            "# Harness Repair Outcome\n\nstatus: `{}`\nsession: `{}`\ntarget: `{}`\ncandidate: `{}`\ndraft_status: `{}`\ndraft_prefix: `{}`\ndraft_model: `{}`\nrepair_command_check: `{}`\nrepair_command_apply: `{}`\nrepair_command_score: `{}`\nrepair_effectiveness_rollup_json: `{}`\nrepair_effectiveness_rollup: status=`{}` active_sources=`{}` used=`{}` failed=`{}` success_rate=`{}` weakest=`{}`\nrepair_effectiveness_rollup_next: `{} {}`\nrepair_queue_brief_json: `{}`\nrepair_queue_brief: status=`{}` plan_status=`{}` source=`{}` blockers=`{}`\nrepair_queue_brief_next: `{} {}`\naction_trace_json: `{}`\naction_trace_status: `{}`\naction_trace_stages: `{}`\naction_trace_last: `{}`\naction_trace_recall: matches=`{}` top=`{}` reasons=`{}`\naction_trace_lessons: count=`{}` reuse=`{}` avoid=`{}` top_reuse=`{}` top_avoid=`{}`\naction_trace_repair_draft_strategy: status=`{}` focus=`{}` reuse=`{}` avoid=`{}` next=`{} {}`\naction_trace_repair_draft_strategy_effectiveness: used=`{}` success_rate=`{}`\naction_trace_command_strategy: status=`{}` focus=`{}` next=`{} {}`\naction_trace_repair_patch_draft: status=`{}` has_patch=`{}` target=`{}`\naction_trace_repair_patch_review: status=`{}` check=`{}` has_patch=`{}` target=`{}` summary=`{}`\naction_trace_repair_patch_apply: status=`{}` applied=`{}` target=`{}` summary=`{}`\naction_trace_repair_patch_apply_effectiveness: used=`{}` success_rate=`{}`\naction_trace_repair_patch_verify: status=`{}` passed=`{}` target=`{}` command=`{}` summary=`{}`\naction_trace_repair_patch_verify_effectiveness: used=`{}` success_rate=`{}`\naction_trace_repair_patch_learning: status=`{}` used=`{}` verified=`{}` verified_success_rate=`{}` top_reuse=`{}` top_avoid=`{}`\naction_trace_repair_patch_strategy: status=`{}` focus=`{}` next=`{} {}`\naction_trace_repair_decision: decision=`{}` focus=`{}`\naction_trace_harness_environment_drift: status=`{}` detail=`{}` history=`{}` next=`{} {}`\naction_trace_harness_adaptation: status=`{}` focus=`{}`\n\n{}\n\njournal: `{}`\n",
            status,
            display_path(&workspace, &session_path),
            record["target_tentacle"].as_str().unwrap_or("unknown"),
            record["candidate"].as_str().unwrap_or("none"),
            record["draft_status"].as_str().unwrap_or("unknown"),
            record["draft_prefix"].as_str().unwrap_or(""),
            record["draft_model"].as_str().unwrap_or(""),
            record["repair_command_check"].as_str().unwrap_or(""),
            record["repair_command_apply"].as_str().unwrap_or(""),
            record["repair_command_score"].as_str().unwrap_or(""),
            record["repair_effectiveness_rollup_json"]
                .as_str()
                .unwrap_or(""),
            record["repair_effectiveness_rollup_status"]
                .as_str()
                .unwrap_or(""),
            record["repair_effectiveness_rollup_active_source_count"]
                .as_str()
                .unwrap_or(""),
            record["repair_effectiveness_rollup_used_count"]
                .as_str()
                .unwrap_or(""),
            record["repair_effectiveness_rollup_failed_count"]
                .as_str()
                .unwrap_or(""),
            record["repair_effectiveness_rollup_success_rate"]
                .as_str()
                .unwrap_or(""),
            record["repair_effectiveness_rollup_weakest_source"]
                .as_str()
                .unwrap_or(""),
            record["repair_effectiveness_rollup_next_need_kind"]
                .as_str()
                .unwrap_or(""),
            record["repair_effectiveness_rollup_next_need_query"]
                .as_str()
                .unwrap_or(""),
            record["repair_queue_brief_json"].as_str().unwrap_or(""),
            record["repair_queue_brief_status"].as_str().unwrap_or(""),
            record["repair_queue_brief_plan_status"]
                .as_str()
                .unwrap_or(""),
            record["repair_queue_brief_source"].as_str().unwrap_or(""),
            record["repair_queue_brief_blockers"].as_str().unwrap_or(""),
            record["repair_queue_brief_next_need_kind"]
                .as_str()
                .unwrap_or(""),
            record["repair_queue_brief_next_need_query"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_json"].as_str().unwrap_or(""),
            record["action_trace_status"].as_str().unwrap_or("unknown"),
            record["action_trace_stage_count"].as_str().unwrap_or(""),
            record["action_trace_last_action"].as_str().unwrap_or(""),
            record["action_trace_recall_count"].as_str().unwrap_or(""),
            record["action_trace_recall_top_status"].as_str().unwrap_or(""),
            record["action_trace_recall_top_reasons"].as_str().unwrap_or(""),
            record["action_trace_lesson_count"].as_str().unwrap_or(""),
            record["action_trace_lesson_reuse_count"].as_str().unwrap_or(""),
            record["action_trace_lesson_avoid_count"].as_str().unwrap_or(""),
            record["action_trace_lesson_top_reuse"].as_str().unwrap_or(""),
            record["action_trace_lesson_top_avoid"].as_str().unwrap_or(""),
            record["action_trace_repair_draft_strategy_status"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_draft_strategy_focus"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_draft_strategy_learned_reuse"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_draft_strategy_learned_avoid"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_draft_strategy_next_need_kind"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_draft_strategy_next_need_query"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_draft_strategy_effectiveness_used_count"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_draft_strategy_effectiveness_success_rate"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_command_strategy_status"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_command_strategy_focus"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_command_strategy_next_need_kind"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_command_strategy_next_need_query"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_draft_status"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_draft_has_patch"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_draft_target"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_review_status"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_review_check_status"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_review_has_patch"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_review_target"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_review_summary"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_apply_status"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_apply_applied"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_apply_target"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_apply_summary"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_apply_effectiveness_used_count"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_apply_effectiveness_success_rate"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_verify_status"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_verify_passed"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_verify_target"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_verify_command"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_verify_summary"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_verify_effectiveness_used_count"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_verify_effectiveness_success_rate"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_learning_status"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_learning_used_count"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_learning_verified_count"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_learning_verified_success_rate"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_learning_top_reuse"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_learning_top_avoid"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_strategy_status"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_strategy_focus"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_strategy_next_need_kind"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_patch_strategy_next_need_query"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_decision"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_repair_decision_focus"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_harness_environment_drift_status"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_harness_environment_drift_detail"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_harness_environment_drift_history_count"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_harness_environment_drift_next_need_kind"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_harness_environment_drift_next_need_query"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_harness_adaptation_status"]
                .as_str()
                .unwrap_or(""),
            record["action_trace_harness_adaptation_focus"]
                .as_str()
                .unwrap_or(""),
            outcome.summary,
            display_path(&workspace, &outcomes_file)
        ),
    )
    .map_err(|error| error.to_string())?;
    Ok(Some(RepairOutcomeJournalReport {
        session_path: display_path(&workspace, &session_path),
        outcome_path: display_path(&workspace, &outcome_path),
        outcomes_file: display_path(&workspace, &outcomes_file),
    }))
}

fn metadata_path(workspace: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        workspace.join(path)
    }
}

fn display_path(workspace: &Path, path: &Path) -> String {
    path.strip_prefix(workspace)
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

fn cli_status_key(status: &Status) -> &'static str {
    match status {
        Status::Satisfied => "satisfied",
        Status::Partial => "partial",
        Status::Failed => "failed",
        Status::Unsupported => "unsupported",
    }
}

pub(crate) fn repair_report(
    state_path: &Path,
    state: &mut HarnessState,
    query: String,
) -> Result<RepairReport, String> {
    let tentacle_id = "harness-repair-agent";
    let root = resolve_tentacle_manifest_root(tentacle_id)?;
    let _ = state.install_profile(tentacle_id);
    state.install_manifest(&root, tentacle_id)?;
    state.save(state_path).map_err(|error| error.to_string())?;
    let llm_factory = if manifest_llm_enabled() {
        Some(manifest_llm_factory()?)
    } else {
        None
    };
    let query_workspace = repair_workspace_from_query(&query).ok();
    let query_has_plan = query_workspace
        .as_deref()
        .and_then(latest_repair_plan_in_workspace)
        .is_some();
    let state_workspace = repair_workspace_from_state_path(state_path);
    let state_has_plan = !query_has_plan
        && state_workspace
            .as_deref()
            .and_then(latest_repair_plan_in_workspace)
            .is_some();
    let need_kind = if query_has_plan || state_has_plan {
        NeedKind::Verify
    } else {
        NeedKind::Observe
    };
    let need_query = if query_has_plan {
        query_workspace
            .as_ref()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|| query.clone())
    } else if state_has_plan {
        state_workspace
            .as_ref()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|| query.clone())
    } else {
        query.clone()
    };
    let continue_query = need_query.clone();
    let need = Need::new(need_kind, need_query);
    let mut feed =
        feed_tentacle_with_llm_factory(&root, tentacle_id, &need, llm_factory, &state.grants)?;
    feed.metadata
        .insert("tentacle".to_string(), tentacle_id.to_string());
    state.record_pet_event_from_feed(&feed);
    let trace = state.record_feed_trace_from_feed(&feed);
    feed.metadata
        .insert("feed_trace_index".to_string(), trace.index.to_string());
    let mut queued = Vec::new();
    if let Some(next_need) = repair_next_need_from_feed(&feed) {
        queued.push(state.queue_need_suggestion(
            next_need,
            tentacle_id,
            query,
            feed.summary.clone(),
        ));
    }
    let repair_plan = repair_plan_report_from_feed(&feed);
    let queue = state.need_queue_report(8);
    let mut next = queued
        .iter()
        .map(|item| format!("octopus needs take {}", item.index))
        .collect::<Vec<_>>();
    if !queued.is_empty() {
        next.push(format!(
            "octopus repair continue {}",
            shell_arg(&continue_query)
        ));
    }
    if let Some(plan) = &repair_plan {
        if !plan.repair_queue_brief.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.repair_queue_brief)));
        }
        if !plan.repair_queue_brief_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_queue_brief_json)
            ));
        }
        if !plan.review.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.review)));
        }
        if !plan.outcome.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.outcome)));
        }
        if !plan.draft.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.draft)));
        }
        if !plan.repair_draft_effectiveness.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_draft_effectiveness)
            ));
        }
        if !plan.repair_draft_effectiveness_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_draft_effectiveness_json)
            ));
        }
        if !plan.repair_draft_strategy.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.repair_draft_strategy)));
        }
        if !plan.repair_draft_strategy_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_draft_strategy_json)
            ));
        }
        if !plan.repair_draft_strategy_effectiveness.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_draft_strategy_effectiveness)
            ));
        }
        if !plan
            .repair_draft_strategy_effectiveness_json
            .trim()
            .is_empty()
        {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_draft_strategy_effectiveness_json)
            ));
        }
        if !plan.repair_patch_draft.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.repair_patch_draft)));
        }
        if !plan.repair_patch_draft_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_draft_json)
            ));
        }
        if !plan.repair_patch_draft_effectiveness.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_draft_effectiveness)
            ));
        }
        if !plan.repair_patch_draft_effectiveness_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_draft_effectiveness_json)
            ));
        }
        if !plan.repair_patch_review.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.repair_patch_review)));
        }
        if !plan.repair_patch_review_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_review_json)
            ));
        }
        if !plan.repair_patch_review_effectiveness.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_review_effectiveness)
            ));
        }
        if !plan
            .repair_patch_review_effectiveness_json
            .trim()
            .is_empty()
        {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_review_effectiveness_json)
            ));
        }
        if !plan.repair_patch_apply.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.repair_patch_apply)));
        }
        if !plan.repair_patch_apply_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_apply_json)
            ));
        }
        if !plan.repair_patch_apply_effectiveness.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_apply_effectiveness)
            ));
        }
        if !plan.repair_patch_apply_effectiveness_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_apply_effectiveness_json)
            ));
        }
        if !plan.repair_patch_verify.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.repair_patch_verify)));
        }
        if !plan.repair_patch_verify_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_verify_json)
            ));
        }
        if !plan.repair_patch_verify_effectiveness.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_verify_effectiveness)
            ));
        }
        if !plan
            .repair_patch_verify_effectiveness_json
            .trim()
            .is_empty()
        {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_verify_effectiveness_json)
            ));
        }
        if !plan.repair_patch_learning.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.repair_patch_learning)));
        }
        if !plan.repair_patch_learning_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_learning_json)
            ));
        }
        if !plan.repair_patch_learning_effectiveness.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_learning_effectiveness)
            ));
        }
        if !plan
            .repair_patch_learning_effectiveness_json
            .trim()
            .is_empty()
        {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_learning_effectiveness_json)
            ));
        }
        if !plan.repair_patch_strategy.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.repair_patch_strategy)));
        }
        if !plan.repair_patch_strategy_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_strategy_json)
            ));
        }
        if !plan.repair_patch_strategy_effectiveness.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_strategy_effectiveness)
            ));
        }
        if !plan
            .repair_patch_strategy_effectiveness_json
            .trim()
            .is_empty()
        {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_patch_strategy_effectiveness_json)
            ));
        }
        let patch_apply_status = plan.repair_patch_apply_status.trim();
        if plan.repair_patch_review_status == "check_passed"
            && plan.repair_patch_apply_applied != "true"
            && matches!(
                patch_apply_status,
                "" | "ready_for_authorized_apply" | "needs_authorization"
            )
        {
            next.push(format!(
                "octopus repair apply {}",
                shell_arg(&continue_query)
            ));
        }
        let patch_verify_status = plan.repair_patch_verify_status.trim();
        if plan.repair_patch_apply_applied == "true"
            && !matches!(patch_verify_status, "passed" | "failed")
        {
            next.push(format!(
                "octopus repair verify {}",
                shell_arg(&continue_query)
            ));
        }
        if !plan.repair_command_effectiveness.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_command_effectiveness)
            ));
        }
        if !plan.repair_command_effectiveness_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_command_effectiveness_json)
            ));
        }
        if !plan.repair_command_strategy.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_command_strategy)
            ));
        }
        if !plan.repair_command_strategy_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_command_strategy_json)
            ));
        }
        if !plan.repair_command_strategy_effectiveness.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_command_strategy_effectiveness)
            ));
        }
        if !plan
            .repair_command_strategy_effectiveness_json
            .trim()
            .is_empty()
        {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_command_strategy_effectiveness_json)
            ));
        }
        if !plan.action_trace.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.action_trace)));
        }
        if !plan.action_trace_json.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.action_trace_json)));
        }
        if !plan.action_trace_effectiveness.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.action_trace_effectiveness)
            ));
        }
        if !plan.action_trace_effectiveness_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.action_trace_effectiveness_json)
            ));
        }
        if !plan.repair_recall.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.repair_recall)));
        }
        if !plan.repair_lessons.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.repair_lessons)));
        }
        if !plan.repair_lesson_effectiveness.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_lesson_effectiveness)
            ));
        }
        if !plan.repair_decision_effectiveness.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_decision_effectiveness)
            ));
        }
        if !plan.repair_decision_effectiveness_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_decision_effectiveness_json)
            ));
        }
        if !plan.repair_decision.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.repair_decision)));
        }
        if !plan.harness_adaptation_effectiveness.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.harness_adaptation_effectiveness)
            ));
        }
        if !plan.harness_adaptation_effectiveness_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.harness_adaptation_effectiveness_json)
            ));
        }
        if !plan.harness_environment_profile.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.harness_environment_profile)
            ));
        }
        if !plan.harness_environment_profile_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.harness_environment_profile_json)
            ));
        }
        if !plan.harness_environment_drift.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.harness_environment_drift)
            ));
        }
        if !plan.harness_environment_drift_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.harness_environment_drift_json)
            ));
        }
        if !plan
            .harness_environment_drift_effectiveness
            .trim()
            .is_empty()
        {
            next.push(format!(
                "review {}",
                shell_arg(&plan.harness_environment_drift_effectiveness)
            ));
        }
        if !plan
            .harness_environment_drift_effectiveness_json
            .trim()
            .is_empty()
        {
            next.push(format!(
                "review {}",
                shell_arg(&plan.harness_environment_drift_effectiveness_json)
            ));
        }
        if !plan.repair_effectiveness_rollup.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_effectiveness_rollup)
            ));
        }
        if !plan.repair_effectiveness_rollup_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_effectiveness_rollup_json)
            ));
        }
        if !plan
            .repair_effectiveness_rollup_effectiveness
            .trim()
            .is_empty()
        {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_effectiveness_rollup_effectiveness)
            ));
        }
        if !plan
            .repair_effectiveness_rollup_effectiveness_json
            .trim()
            .is_empty()
        {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_effectiveness_rollup_effectiveness_json)
            ));
        }
        if !plan
            .repair_effectiveness_rollup_adaptation
            .trim()
            .is_empty()
        {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_effectiveness_rollup_adaptation)
            ));
        }
        if !plan
            .repair_effectiveness_rollup_adaptation_json
            .trim()
            .is_empty()
        {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_effectiveness_rollup_adaptation_json)
            ));
        }
        if !plan.repair_queue_brief_effectiveness.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_queue_brief_effectiveness)
            ));
        }
        if !plan.repair_queue_brief_effectiveness_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_queue_brief_effectiveness_json)
            ));
        }
        if !plan.repair_queue_brief_adaptation.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_queue_brief_adaptation)
            ));
        }
        if !plan.repair_queue_brief_adaptation_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.repair_queue_brief_adaptation_json)
            ));
        }
        if !plan.environment_profile_journal.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.environment_profile_journal)
            ));
        }
        if !plan.harness_adaptation.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.harness_adaptation)));
        }
        if !plan.harness_adaptation_json.trim().is_empty() {
            next.push(format!(
                "review {}",
                shell_arg(&plan.harness_adaptation_json)
            ));
        }
        if !plan.code_context.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.code_context)));
        }
        if !plan.adapter_context.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.adapter_context)));
        }
        if !plan.field_trajectory.trim().is_empty() {
            next.push(format!("review {}", shell_arg(&plan.field_trajectory)));
        }
        for command in [
            &plan.check_command,
            &plan.grant_command,
            &plan.apply_command,
        ] {
            if !command.trim().is_empty() {
                next.push(command.to_string());
            }
        }
        for command in &plan.score_commands {
            if !command.trim().is_empty() {
                next.push(command.to_string());
            }
        }
    }
    next.push("octopus needs".to_string());
    next.push("octopus beat 200".to_string());
    next.push("octopus traces".to_string());
    next.sort();
    next.dedup();
    Ok(RepairReport {
        state_path: state_path.to_string_lossy().to_string(),
        tentacle: tentacle_id.to_string(),
        feed,
        repair_plan,
        queued,
        queue,
        next,
    })
}

pub(crate) fn repair_continue_report(
    state_path: &Path,
    state: &mut HarnessState,
    query: String,
) -> Result<RepairContinueReport, String> {
    let repair = repair_report(state_path, state, query)?;
    let Some(queued) = repair.queued.first() else {
        state.save(state_path).map_err(|error| error.to_string())?;
        return Ok(RepairContinueReport {
            repair,
            taken: None,
            feedback: None,
            score: None,
            next: vec![
                "octopus repair .".to_string(),
                "octopus report".to_string(),
                "octopus beat 200".to_string(),
            ],
        });
    };
    let taken = state.take_queued_need(queued.index)?;
    let need = Need::new(taken.item.need.kind.clone(), taken.item.need.query.clone());
    let loaded = std::mem::take(state);
    let mut harness = harness_for_need(loaded, &need.kind)?;
    let feedback = harness.feed(&[need]);
    let trace_index = feedback_trace_index(&feedback);
    *state = harness.state;
    state.save(state_path).map_err(|error| error.to_string())?;
    let mut next = repair_score_commands(trace_index.as_deref());
    next.push("octopus report".to_string());
    next.push("octopus beat 200".to_string());
    if let Some(index) = trace_index {
        next.insert(
            0,
            format!("octopus feedback {index} satisfied \"continued repair Feed worked\""),
        );
    }
    Ok(RepairContinueReport {
        repair,
        taken: Some(taken),
        feedback: Some(feedback),
        score: None,
        next,
    })
}

pub(crate) fn feedback_trace_index(feedback: &Feedback) -> Option<String> {
    feedback
        .feeds
        .iter()
        .find_map(|feed| feed.metadata.get("feed_trace_index").cloned())
}

fn repair_score_command(trace_index: Option<&str>) -> String {
    let index = trace_index.unwrap_or("<trace-index>");
    format!("octopus repair score {index} satisfied \"repair improved harness\"")
}

fn repair_score_commands(trace_index: Option<&str>) -> Vec<String> {
    let index = trace_index.unwrap_or("<trace-index>");
    vec![
        repair_score_command(trace_index),
        format!("octopus repair score {index} partial \"repair partly improved harness\""),
        format!("octopus repair score {index} failed \"repair did not improve harness\""),
    ]
}

pub(crate) fn repair_score_commands_from_plan(command: &str) -> Vec<String> {
    let command = command.trim();
    if command.is_empty() {
        return Vec::new();
    }
    let Some(index) = repair_score_command_index(command) else {
        return vec![command.to_string()];
    };
    vec![
        command.to_string(),
        format!("octopus repair score {index} partial \"repair partly improved Feed\""),
        format!("octopus repair score {index} failed \"repair did not improve Feed\""),
    ]
}

fn repair_score_command_index(command: &str) -> Option<&str> {
    let mut parts = command.split_whitespace();
    match (parts.next()?, parts.next()?, parts.next()?) {
        ("octopus", "repair", "score") => parts.next(),
        _ => None,
    }
}

fn parse_repair_continue_args(args: &[String]) -> Result<RepairContinueArgs, String> {
    let score_index = args.iter().position(|value| value == "--score");
    let query_parts = score_index.map_or(args, |index| &args[..index]);
    let query = if query_parts.is_empty() {
        ".".to_string()
    } else {
        query_parts.join(" ")
    };
    let score = if let Some(index) = score_index {
        let status = args
            .get(index + 1)
            .ok_or_else(|| "repair continue --score requires a status".to_string())
            .and_then(|value| parse_status(value))?;
        let summary = args
            .get(index + 2..)
            .filter(|values| !values.is_empty())
            .map(|values| values.join(" "))
            .unwrap_or_else(|| "continued repair Feed worked".to_string());
        Some(RepairContinueScoreArgs { status, summary })
    } else {
        None
    };
    Ok(RepairContinueArgs { query, score })
}

pub(crate) fn repair_score_report(
    state: &mut HarnessState,
    trace_selector: &str,
    status: Status,
    summary: String,
    cwd: &Path,
) -> Result<RepairScoreReport, String> {
    let trace_index = resolve_feed_trace_selector(trace_selector, state)?;
    let trace = state
        .feed_traces
        .iter()
        .find(|trace| trace.index == trace_index)
        .cloned();
    let outcome = state.record_repair_outcome(Some(trace_index), status, summary)?;
    let should_continue_repair = matches!(outcome.status, Status::Failed | Status::Partial);
    let evolution = mirror_repair_score_to_evolution_outcome(state, trace.as_ref(), &outcome);
    let journal = write_repair_score_journal(cwd, trace.as_ref(), &outcome)?;
    let mut next = vec![
        "octopus repair .".to_string(),
        "octopus report".to_string(),
        "octopus beat 200".to_string(),
    ];
    let mut recommendation = None;
    let mut apply_artifact = None;
    let mut evolution_artifact = None;
    let mut recommendation_error = None;
    if let Some(evolution) = &evolution {
        let followup = evolution_score_report(
            &evolution.tentacle_id,
            &evolution.summary,
            state,
            cwd,
            evolution.clone(),
        );
        if followup.recommendation.is_some() {
            next = followup.next.clone();
        } else {
            next.insert(
                0,
                format!(
                    "octopus evolve recommend {}",
                    shell_arg(&evolution.tentacle_id)
                ),
            );
        }
        recommendation = followup.recommendation;
        apply_artifact = followup.apply_artifact;
        evolution_artifact = followup.evolution_artifact;
        recommendation_error = followup.recommendation_error;
    } else if should_continue_repair {
        next.insert(0, "octopus repair continue .".to_string());
    }
    Ok(RepairScoreReport {
        outcome,
        journal,
        recent: state.recent_repair_outcomes(5),
        evolution,
        recommendation,
        apply_artifact,
        evolution_artifact,
        recommendation_error,
        next,
    })
}

fn latest_repair_plan_in_workspace(workspace: &Path) -> Option<PathBuf> {
    let repair_root = workspace.join(".octopus/harness-repair");
    let mut latest: Option<(SystemTime, PathBuf)> = None;
    for entry in fs::read_dir(repair_root).ok()? {
        let path = entry.ok()?.path().join("REPAIR_PLAN.json");
        if !path.exists() {
            continue;
        }
        let modified = fs::metadata(&path)
            .and_then(|metadata| metadata.modified())
            .unwrap_or(UNIX_EPOCH);
        if latest.as_ref().is_none_or(|(time, _)| modified > *time) {
            latest = Some((modified, path));
        }
    }
    latest.map(|(_, path)| path)
}

fn repair_workspace_from_state_path(state_path: &Path) -> Option<PathBuf> {
    let parent = state_path.parent()?;
    let workspace = if parent.file_name().is_some_and(|name| name == ".octopus") {
        parent.parent().unwrap_or(parent)
    } else {
        parent
    };
    Some(
        workspace
            .canonicalize()
            .unwrap_or_else(|_| workspace.to_path_buf()),
    )
}

fn repair_workspace_from_query(query: &str) -> Result<PathBuf, String> {
    let token = query
        .split_whitespace()
        .next()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(".");
    let mut path = PathBuf::from(token);
    if !path.is_absolute() {
        path = env::current_dir()
            .map_err(|error| error.to_string())?
            .join(path);
    }
    if path.is_file() {
        path = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
    }
    if !path.exists() || !path.is_dir() {
        return env::current_dir().map_err(|error| error.to_string());
    }
    path.canonicalize().map_err(|error| error.to_string())
}

pub(crate) fn repair_patch_apply_report(
    state_path: &Path,
    state: &HarnessState,
    query: String,
) -> Result<RepairPatchApplyReport, String> {
    let workspace = repair_workspace_from_query(&query)
        .or_else(|_| {
            repair_workspace_from_state_path(state_path)
                .ok_or_else(|| "cannot resolve repair workspace".to_string())
        })
        .or_else(|_| env::current_dir().map_err(|error| error.to_string()))?;
    let plan_path = latest_repair_plan_in_workspace(&workspace)
        .ok_or_else(|| "repair apply requires an existing REPAIR_PLAN.json".to_string())?;
    let plan: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&plan_path).map_err(|error| error.to_string())?)
            .map_err(|error| format!("invalid repair plan JSON: {error}"))?;
    let inputs = plan
        .get("inputs")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "repair plan has no inputs".to_string())?;
    let patch_review_value = inputs
        .get("repair_patch_review_json")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "repair plan has no repair_patch_review_json".to_string())?;
    let patch_review_path = metadata_path(&workspace, patch_review_value);
    let patch_review: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&patch_review_path).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("invalid repair patch review JSON: {error}"))?;
    let patch_apply_value = inputs
        .get("repair_patch_apply_json")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            patch_review_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("REPAIR_PATCH_APPLY.json")
                .to_string_lossy()
                .to_string()
        });
    let patch_apply_path = metadata_path(&workspace, &patch_apply_value);
    let patch_apply_md = patch_apply_path.with_extension("md");
    let patch_file_value = json_string(&patch_review, "patch_file");
    let patch_file = metadata_path(&workspace, &patch_file_value);
    let target_tentacle = plan
        .get("target_tentacle")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            patch_review
                .get("target")
                .and_then(|target| target.get("tentacle"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown")
        })
        .to_string();
    let target_tool = plan
        .get("target_tool")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            patch_review
                .get("target")
                .and_then(|target| target.get("tool"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
        })
        .to_string();
    let target = if target_tool.is_empty() {
        target_tentacle.clone()
    } else {
        format!("{target_tentacle}/{target_tool}")
    };
    let review_status = json_string(&patch_review, "status");
    let review_check = json_string(&patch_review, "check_status");
    let has_patch = json_bool(&patch_review, "has_patch");
    let (required_grant, grant_command, authorized) =
        repair_patch_apply_grant(state, &target_tentacle);
    let check_command = format!(
        "git apply --check --whitespace=nowarn {}",
        shell_arg(&patch_file.to_string_lossy())
    );
    let apply_command = format!(
        "git apply --whitespace=nowarn {}",
        shell_arg(&patch_file.to_string_lossy())
    );
    let status: String;
    let mut applied = false;
    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut command = String::new();
    let mut returncode = String::new();
    let mut next = Vec::new();

    if !has_patch {
        status = "blocked_no_patch".to_string();
        next.push(format!(
            "review {}",
            shell_arg(&display_path(&workspace, &patch_review_path))
        ));
        next.push("octopus repair .".to_string());
    } else if review_status != "check_passed" || review_check != "passed" {
        status = "blocked_patch_review".to_string();
        next.push(format!(
            "review {}",
            shell_arg(&display_path(&workspace, &patch_review_path))
        ));
        next.push("octopus repair .".to_string());
    } else if patch_file_value.trim().is_empty() || !patch_file.exists() {
        status = "missing_patch".to_string();
        next.push(format!(
            "review {}",
            shell_arg(&display_path(&workspace, &patch_review_path))
        ));
    } else if !authorized {
        status = "needs_authorization".to_string();
        next.push(grant_command.clone());
        next.push(format!("octopus repair apply {}", shell_arg(&query)));
    } else {
        let check = Command::new("git")
            .arg("apply")
            .arg("--check")
            .arg("--whitespace=nowarn")
            .arg(&patch_file)
            .current_dir(&workspace)
            .output();
        match check {
            Ok(output) if output.status.success() => {
                let output = Command::new("git")
                    .arg("apply")
                    .arg("--whitespace=nowarn")
                    .arg(&patch_file)
                    .current_dir(&workspace)
                    .output();
                command = apply_command.clone();
                match output {
                    Ok(output) => {
                        applied = output.status.success();
                        status = if applied { "applied" } else { "failed" }.to_string();
                        returncode = output
                            .status
                            .code()
                            .map_or_else(|| "signal".to_string(), |code| code.to_string());
                        stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    }
                    Err(error) => {
                        status = "failed_to_start".to_string();
                        stderr = error.to_string();
                    }
                }
            }
            Ok(output) => {
                status = "check_failed".to_string();
                command = check_command.clone();
                returncode = output
                    .status
                    .code()
                    .map_or_else(|| "signal".to_string(), |code| code.to_string());
                stdout = String::from_utf8_lossy(&output.stdout).to_string();
                stderr = String::from_utf8_lossy(&output.stderr).to_string();
            }
            Err(error) => {
                status = "failed_to_start".to_string();
                command = check_command.clone();
                stderr = error.to_string();
            }
        }
        if applied {
            if target_tentacle != "unknown" {
                next.push(format!("octopus repair verify {}", shell_arg(&query)));
                next.push(format!("octopus check {}", shell_arg(&target_tentacle)));
            }
            next.push("octopus repair score latest satisfied \"repair patch applied\"".to_string());
        } else {
            next.push(
                "octopus repair score latest failed \"repair patch apply failed\"".to_string(),
            );
            next.push("octopus repair .".to_string());
        }
    }

    let summary = match status.as_str() {
        "applied" => "repair patch applied".to_string(),
        "needs_authorization" => "repair patch apply needs harness write grant".to_string(),
        "blocked_no_patch" => "repair patch apply has no provider patch".to_string(),
        "blocked_patch_review" => "repair patch review did not pass".to_string(),
        "missing_patch" => "repair patch file is missing".to_string(),
        "check_failed" => "repair patch apply check failed".to_string(),
        "failed" => "repair patch apply failed".to_string(),
        "failed_to_start" => "repair patch apply command failed to start".to_string(),
        _ => "repair patch apply is ready".to_string(),
    };
    let next_need_query = if applied {
        "score the applied repair patch outcome"
    } else if status == "needs_authorization" {
        "grant harness write before repair patch apply"
    } else {
        "repair failed repair patch apply"
    };
    let apply_record = serde_json::json!({
        "schema_version": "octopus-harness-repair-patch-apply-v1",
        "status": status,
        "applied": applied,
        "authorized": authorized,
        "target": {
            "tentacle": target_tentacle,
            "tool": target_tool,
        },
        "target_label": target,
        "patch_file": display_path(&workspace, &patch_file),
        "patch_review": display_path(&workspace, &patch_review_path),
        "patch_review_status": review_status,
        "patch_review_check_status": review_check,
        "required_grant": required_grant,
        "grant_command": grant_command,
        "apply_command": format!("octopus repair apply {}", shell_arg(&query)),
        "check_command": check_command,
        "command": command,
        "returncode": returncode,
        "stdout": truncate_chars(&stdout, 2000),
        "stderr": truncate_chars(&stderr, 2000),
        "summary": summary,
        "next_need": {
            "kind": if applied { "verify" } else { "execute" },
            "query": next_need_query,
        },
    });
    if let Some(parent) = patch_apply_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(
        &patch_apply_path,
        serde_json::to_string_pretty(&apply_record).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        &patch_apply_md,
        render_repair_patch_apply_markdown(&workspace, &patch_apply_path, &apply_record),
    )
    .map_err(|error| error.to_string())?;

    Ok(RepairPatchApplyReport {
        workspace: workspace.to_string_lossy().to_string(),
        plan: display_path(&workspace, &plan_path),
        patch_review: display_path(&workspace, &patch_review_path),
        apply: display_path(&workspace, &patch_apply_path),
        status,
        applied,
        authorized,
        required_grant: json_string(&apply_record, "required_grant"),
        grant_command: json_string(&apply_record, "grant_command"),
        check_command,
        command: json_string(&apply_record, "command"),
        patch_file: display_path(&workspace, &patch_file),
        target,
        stdout: truncate_chars(&stdout, 2000),
        stderr: truncate_chars(&stderr, 2000),
        next,
    })
}

pub(crate) fn repair_patch_verify_report(
    state_path: &Path,
    state: &mut HarnessState,
    query: String,
) -> Result<RepairPatchVerifyReport, String> {
    let workspace = repair_workspace_from_query(&query)
        .or_else(|_| {
            repair_workspace_from_state_path(state_path)
                .ok_or_else(|| "cannot resolve repair workspace".to_string())
        })
        .or_else(|_| env::current_dir().map_err(|error| error.to_string()))?;
    let plan_path = latest_repair_plan_in_workspace(&workspace)
        .ok_or_else(|| "repair verify requires an existing REPAIR_PLAN.json".to_string())?;
    let plan: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&plan_path).map_err(|error| error.to_string())?)
            .map_err(|error| format!("invalid repair plan JSON: {error}"))?;
    let inputs = plan
        .get("inputs")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "repair plan has no inputs".to_string())?;
    let patch_apply_value = inputs
        .get("repair_patch_apply_json")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            plan_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("REPAIR_PATCH_APPLY.json")
                .to_string_lossy()
                .to_string()
        });
    let patch_apply_path = metadata_path(&workspace, &patch_apply_value);
    let patch_apply: serde_json::Value = if patch_apply_path.exists() {
        serde_json::from_str(
            &fs::read_to_string(&patch_apply_path).map_err(|error| error.to_string())?,
        )
        .map_err(|error| format!("invalid repair patch apply JSON: {error}"))?
    } else {
        serde_json::json!({})
    };
    let patch_verify_value = inputs
        .get("repair_patch_verify_json")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            plan_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("REPAIR_PATCH_VERIFY.json")
                .to_string_lossy()
                .to_string()
        });
    let patch_verify_path = metadata_path(&workspace, &patch_verify_value);
    let patch_verify_md = patch_verify_path.with_extension("md");
    let target_tentacle = patch_apply
        .get("target")
        .and_then(|target| target.get("tentacle"))
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            plan.get("target_tentacle")
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or("unknown")
        .to_string();
    let target_tool = patch_apply
        .get("target")
        .and_then(|target| target.get("tool"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| plan.get("target_tool").and_then(serde_json::Value::as_str))
        .unwrap_or("")
        .to_string();
    let target = if target_tool.is_empty() {
        target_tentacle.clone()
    } else {
        format!("{target_tentacle}/{target_tool}")
    };
    let command = if target_tentacle == "unknown" {
        String::new()
    } else {
        format!("octopus check {}", shell_arg(&target_tentacle))
    };
    let mut status = "blocked_patch_apply".to_string();
    let mut passed = false;
    let mut check = None;
    let mut error = String::new();
    let mut next = Vec::new();

    if !patch_apply_path.exists() {
        status = "missing_patch_apply".to_string();
        next.push(format!("octopus repair apply {}", shell_arg(&query)));
    } else if !json_bool(&patch_apply, "applied") {
        next.push(format!("octopus repair apply {}", shell_arg(&query)));
    } else if target_tentacle == "unknown" {
        status = "missing_target".to_string();
        next.push("octopus repair .".to_string());
    } else {
        let original_dir = env::current_dir().map_err(|error| error.to_string())?;
        env::set_current_dir(&workspace).map_err(|error| error.to_string())?;
        let check_result = check_report(&target_tentacle, None);
        let restore_result = env::set_current_dir(&original_dir);
        if let Err(error) = restore_result {
            return Err(error.to_string());
        }
        match check_result {
            Ok(mut check_report) => {
                record_check_report(state, &mut check_report);
                passed = check_report.passed;
                status = if passed { "passed" } else { "failed" }.to_string();
                check = Some(check_report);
                if passed {
                    next.push(
                        "octopus repair score latest satisfied \"repair patch verified\""
                            .to_string(),
                    );
                } else {
                    next.push(
                        "octopus repair score latest failed \"repair patch verify failed\""
                            .to_string(),
                    );
                    next.push("octopus repair .".to_string());
                }
            }
            Err(check_error) => {
                status = "failed_to_start".to_string();
                error = check_error;
                next.push(
                    "octopus repair score latest failed \"repair patch verify failed\"".to_string(),
                );
                next.push("octopus repair .".to_string());
            }
        }
    }

    let summary = match status.as_str() {
        "passed" => "repair patch verification passed".to_string(),
        "failed" => "repair patch verification failed".to_string(),
        "failed_to_start" => format!("repair patch verification failed to start: {error}"),
        "missing_patch_apply" => "repair patch apply record is missing".to_string(),
        "missing_target" => "repair patch verification target is missing".to_string(),
        _ => "repair patch must be applied before verification".to_string(),
    };
    let verify_record = serde_json::json!({
        "schema_version": "octopus-harness-repair-patch-verify-v1",
        "status": status,
        "passed": passed,
        "target": {
            "tentacle": target_tentacle,
            "tool": target_tool,
        },
        "target_label": target,
        "patch_apply": display_path(&workspace, &patch_apply_path),
        "command": command,
        "summary": summary,
        "error": error,
        "check": check.as_ref().map(serde_json::to_value).transpose().map_err(|error| error.to_string())?,
        "next_need": {
            "kind": if passed { "verify" } else { "execute" },
            "query": if passed { "score verified repair patch outcome" } else { "repair failed post-apply verification" },
        },
    });
    if let Some(parent) = patch_verify_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(
        &patch_verify_path,
        serde_json::to_string_pretty(&verify_record).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        &patch_verify_md,
        render_repair_patch_verify_markdown(&workspace, &patch_verify_path, &verify_record),
    )
    .map_err(|error| error.to_string())?;

    Ok(RepairPatchVerifyReport {
        workspace: workspace.to_string_lossy().to_string(),
        plan: display_path(&workspace, &plan_path),
        apply: display_path(&workspace, &patch_apply_path),
        verify: display_path(&workspace, &patch_verify_path),
        status,
        passed,
        target,
        command: json_string(&verify_record, "command"),
        summary,
        check,
        next,
    })
}

fn repair_patch_apply_grant(state: &HarnessState, target_tentacle: &str) -> (String, String, bool) {
    let scope = if target_tentacle.trim().is_empty() || target_tentacle == "unknown" {
        "harness:write".to_string()
    } else {
        format!("evolve:{target_tentacle}")
    };
    let required = format!("octopus:{scope}");
    let command = if scope == "harness:write" {
        "octopus oauth octopus harness:write".to_string()
    } else {
        format!("octopus oauth octopus {} harness:write", shell_arg(&scope))
    };
    let authorized = state.active_grant("octopus", &scope).is_some_and(|grant| {
        grant
            .permissions
            .iter()
            .any(|permission| permission == "harness:write")
    });
    (required, command, authorized)
}

fn render_repair_patch_apply_markdown(
    workspace: &Path,
    apply_path: &Path,
    record: &serde_json::Value,
) -> String {
    format!(
        "# Harness Repair Patch Apply\n\njson: `{}`\nstatus: `{}`\napplied: `{}`\nauthorized: `{}`\ntarget: `{}`\npatch_file: `{}`\nrequired_grant: `{}`\ngrant_command: `{}`\napply_command: `{}`\ncheck_command: `{}`\ncommand: `{}`\nreturncode: `{}`\nsummary: {}\n\n## Stdout\n\n```text\n{}\n```\n\n## Stderr\n\n```text\n{}\n```\n",
        display_path(workspace, apply_path),
        json_string(record, "status"),
        json_bool(record, "applied"),
        json_bool(record, "authorized"),
        json_string(record, "target_label"),
        json_string(record, "patch_file"),
        json_string(record, "required_grant"),
        json_string(record, "grant_command"),
        json_string(record, "apply_command"),
        json_string(record, "check_command"),
        json_string(record, "command"),
        json_string(record, "returncode"),
        json_string(record, "summary"),
        json_string(record, "stdout"),
        json_string(record, "stderr"),
    )
}

fn render_repair_patch_verify_markdown(
    workspace: &Path,
    verify_path: &Path,
    record: &serde_json::Value,
) -> String {
    let check = record.get("check").unwrap_or(&serde_json::Value::Null);
    let check_summary = if check.is_null() {
        "none".to_string()
    } else {
        format!(
            "passed={} results={}",
            check
                .get("passed")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            check
                .get("results")
                .and_then(serde_json::Value::as_array)
                .map_or(0, Vec::len)
        )
    };
    format!(
        "# Harness Repair Patch Verify\n\njson: `{}`\nstatus: `{}`\npassed: `{}`\ntarget: `{}`\ncommand: `{}`\nsummary: {}\ncheck: `{}`\n\n",
        display_path(workspace, verify_path),
        json_string(record, "status"),
        json_bool(record, "passed"),
        json_string(record, "target_label"),
        json_string(record, "command"),
        json_string(record, "summary"),
        check_summary,
    )
}

fn json_string(value: &serde_json::Value, key: &str) -> String {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn json_bool(value: &serde_json::Value, key: &str) -> bool {
    value
        .get(key)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

pub(crate) fn repair_plan_report_from_feed(feed: &Feed) -> Option<RepairPlanReport> {
    let metadata = &feed.metadata;
    let path = metadata.get("repair_plan")?.to_string();
    let score_command = metadata_value(metadata, "score_command");
    let score_command = metadata
        .get("feed_trace_index")
        .map(|index| score_command.replace("<trace-index>", index))
        .unwrap_or(score_command);
    let score_commands = repair_score_commands_from_metadata(metadata)
        .unwrap_or_else(|| repair_score_commands_from_plan(&score_command));
    Some(RepairPlanReport {
        path,
        repair_queue_brief: metadata_value(metadata, "repair_queue_brief"),
        repair_queue_brief_json: metadata_value(metadata, "repair_queue_brief_json"),
        repair_queue_brief_status: metadata_value(metadata, "repair_queue_brief_status"),
        repair_queue_brief_plan_status: metadata_value(metadata, "repair_queue_brief_plan_status"),
        repair_queue_brief_source: metadata_value(metadata, "repair_queue_brief_source"),
        repair_queue_brief_next_need_kind: metadata_value(
            metadata,
            "repair_queue_brief_next_need_kind",
        ),
        repair_queue_brief_next_need_query: metadata_value(
            metadata,
            "repair_queue_brief_next_need_query",
        ),
        repair_queue_brief_blockers: metadata_value(metadata, "repair_queue_brief_blockers"),
        repair_queue_brief_review_first: metadata_value(
            metadata,
            "repair_queue_brief_review_first",
        ),
        review: metadata_value(metadata, "review"),
        outcome: metadata_value(metadata, "outcome"),
        outcome_status: metadata_value(metadata, "outcome_status"),
        outcome_summary: metadata_value(metadata, "outcome_summary"),
        draft: metadata_value(metadata, "draft"),
        draft_status: metadata_value(metadata, "draft_status"),
        draft_prefix: metadata_value(metadata, "draft_prefix"),
        draft_model: metadata_value(metadata, "draft_model"),
        repair_draft_effectiveness: metadata_value(metadata, "repair_draft_effectiveness"),
        repair_draft_effectiveness_json: metadata_value(
            metadata,
            "repair_draft_effectiveness_json",
        ),
        repair_draft_effectiveness_used_count: metadata_value(
            metadata,
            "repair_draft_effectiveness_used_count",
        ),
        repair_draft_effectiveness_satisfied_count: metadata_value(
            metadata,
            "repair_draft_effectiveness_satisfied_count",
        ),
        repair_draft_effectiveness_partial_count: metadata_value(
            metadata,
            "repair_draft_effectiveness_partial_count",
        ),
        repair_draft_effectiveness_failed_count: metadata_value(
            metadata,
            "repair_draft_effectiveness_failed_count",
        ),
        repair_draft_effectiveness_success_rate: metadata_value(
            metadata,
            "repair_draft_effectiveness_success_rate",
        ),
        repair_draft_effectiveness_failure_rate: metadata_value(
            metadata,
            "repair_draft_effectiveness_failure_rate",
        ),
        repair_draft_effectiveness_top_reuse: metadata_value(
            metadata,
            "repair_draft_effectiveness_top_reuse",
        ),
        repair_draft_effectiveness_top_avoid: metadata_value(
            metadata,
            "repair_draft_effectiveness_top_avoid",
        ),
        repair_draft_strategy: metadata_value(metadata, "repair_draft_strategy"),
        repair_draft_strategy_json: metadata_value(metadata, "repair_draft_strategy_json"),
        repair_draft_strategy_status: metadata_value(metadata, "repair_draft_strategy_status"),
        repair_draft_strategy_focus: metadata_value(metadata, "repair_draft_strategy_focus"),
        repair_draft_strategy_learned_reuse: metadata_value(
            metadata,
            "repair_draft_strategy_learned_reuse",
        ),
        repair_draft_strategy_learned_avoid: metadata_value(
            metadata,
            "repair_draft_strategy_learned_avoid",
        ),
        repair_draft_strategy_strategy_learned_reuse: metadata_value(
            metadata,
            "repair_draft_strategy_strategy_learned_reuse",
        ),
        repair_draft_strategy_strategy_learned_avoid: metadata_value(
            metadata,
            "repair_draft_strategy_strategy_learned_avoid",
        ),
        repair_draft_strategy_next_need_kind: metadata_value(
            metadata,
            "repair_draft_strategy_next_need_kind",
        ),
        repair_draft_strategy_next_need_query: metadata_value(
            metadata,
            "repair_draft_strategy_next_need_query",
        ),
        repair_draft_strategy_effectiveness: metadata_value(
            metadata,
            "repair_draft_strategy_effectiveness",
        ),
        repair_draft_strategy_effectiveness_json: metadata_value(
            metadata,
            "repair_draft_strategy_effectiveness_json",
        ),
        repair_draft_strategy_effectiveness_used_count: metadata_value(
            metadata,
            "repair_draft_strategy_effectiveness_used_count",
        ),
        repair_draft_strategy_effectiveness_satisfied_count: metadata_value(
            metadata,
            "repair_draft_strategy_effectiveness_satisfied_count",
        ),
        repair_draft_strategy_effectiveness_partial_count: metadata_value(
            metadata,
            "repair_draft_strategy_effectiveness_partial_count",
        ),
        repair_draft_strategy_effectiveness_failed_count: metadata_value(
            metadata,
            "repair_draft_strategy_effectiveness_failed_count",
        ),
        repair_draft_strategy_effectiveness_success_rate: metadata_value(
            metadata,
            "repair_draft_strategy_effectiveness_success_rate",
        ),
        repair_draft_strategy_effectiveness_failure_rate: metadata_value(
            metadata,
            "repair_draft_strategy_effectiveness_failure_rate",
        ),
        repair_draft_strategy_effectiveness_top_reuse: metadata_value(
            metadata,
            "repair_draft_strategy_effectiveness_top_reuse",
        ),
        repair_draft_strategy_effectiveness_top_avoid: metadata_value(
            metadata,
            "repair_draft_strategy_effectiveness_top_avoid",
        ),
        repair_patch_draft: metadata_value(metadata, "repair_patch_draft"),
        repair_patch_draft_json: metadata_value(metadata, "repair_patch_draft_json"),
        repair_patch_draft_status: metadata_value(metadata, "repair_patch_draft_status"),
        repair_patch_draft_draft_status: metadata_value(
            metadata,
            "repair_patch_draft_draft_status",
        ),
        repair_patch_draft_has_patch: metadata_value(metadata, "repair_patch_draft_has_patch"),
        repair_patch_draft_target: metadata_value(metadata, "repair_patch_draft_target"),
        repair_patch_draft_next_need_kind: metadata_value(
            metadata,
            "repair_patch_draft_next_need_kind",
        ),
        repair_patch_draft_next_need_query: metadata_value(
            metadata,
            "repair_patch_draft_next_need_query",
        ),
        repair_patch_draft_effectiveness: metadata_value(
            metadata,
            "repair_patch_draft_effectiveness",
        ),
        repair_patch_draft_effectiveness_json: metadata_value(
            metadata,
            "repair_patch_draft_effectiveness_json",
        ),
        repair_patch_draft_effectiveness_used_count: metadata_value(
            metadata,
            "repair_patch_draft_effectiveness_used_count",
        ),
        repair_patch_draft_effectiveness_satisfied_count: metadata_value(
            metadata,
            "repair_patch_draft_effectiveness_satisfied_count",
        ),
        repair_patch_draft_effectiveness_partial_count: metadata_value(
            metadata,
            "repair_patch_draft_effectiveness_partial_count",
        ),
        repair_patch_draft_effectiveness_failed_count: metadata_value(
            metadata,
            "repair_patch_draft_effectiveness_failed_count",
        ),
        repair_patch_draft_effectiveness_success_rate: metadata_value(
            metadata,
            "repair_patch_draft_effectiveness_success_rate",
        ),
        repair_patch_draft_effectiveness_failure_rate: metadata_value(
            metadata,
            "repair_patch_draft_effectiveness_failure_rate",
        ),
        repair_patch_draft_effectiveness_top_reuse: metadata_value(
            metadata,
            "repair_patch_draft_effectiveness_top_reuse",
        ),
        repair_patch_draft_effectiveness_top_avoid: metadata_value(
            metadata,
            "repair_patch_draft_effectiveness_top_avoid",
        ),
        repair_patch_review: metadata_value(metadata, "repair_patch_review"),
        repair_patch_review_json: metadata_value(metadata, "repair_patch_review_json"),
        repair_patch_review_status: metadata_value(metadata, "repair_patch_review_status"),
        repair_patch_review_check_status: metadata_value(
            metadata,
            "repair_patch_review_check_status",
        ),
        repair_patch_review_has_patch: metadata_value(metadata, "repair_patch_review_has_patch"),
        repair_patch_review_target: metadata_value(metadata, "repair_patch_review_target"),
        repair_patch_review_summary: metadata_value(metadata, "repair_patch_review_summary"),
        repair_patch_review_patch_file: metadata_value(metadata, "repair_patch_review_patch_file"),
        repair_patch_review_next_need_kind: metadata_value(
            metadata,
            "repair_patch_review_next_need_kind",
        ),
        repair_patch_review_next_need_query: metadata_value(
            metadata,
            "repair_patch_review_next_need_query",
        ),
        repair_patch_review_effectiveness: metadata_value(
            metadata,
            "repair_patch_review_effectiveness",
        ),
        repair_patch_review_effectiveness_json: metadata_value(
            metadata,
            "repair_patch_review_effectiveness_json",
        ),
        repair_patch_review_effectiveness_used_count: metadata_value(
            metadata,
            "repair_patch_review_effectiveness_used_count",
        ),
        repair_patch_review_effectiveness_satisfied_count: metadata_value(
            metadata,
            "repair_patch_review_effectiveness_satisfied_count",
        ),
        repair_patch_review_effectiveness_partial_count: metadata_value(
            metadata,
            "repair_patch_review_effectiveness_partial_count",
        ),
        repair_patch_review_effectiveness_failed_count: metadata_value(
            metadata,
            "repair_patch_review_effectiveness_failed_count",
        ),
        repair_patch_review_effectiveness_success_rate: metadata_value(
            metadata,
            "repair_patch_review_effectiveness_success_rate",
        ),
        repair_patch_review_effectiveness_failure_rate: metadata_value(
            metadata,
            "repair_patch_review_effectiveness_failure_rate",
        ),
        repair_patch_review_effectiveness_top_reuse: metadata_value(
            metadata,
            "repair_patch_review_effectiveness_top_reuse",
        ),
        repair_patch_review_effectiveness_top_avoid: metadata_value(
            metadata,
            "repair_patch_review_effectiveness_top_avoid",
        ),
        repair_patch_apply: metadata_value(metadata, "repair_patch_apply"),
        repair_patch_apply_json: metadata_value(metadata, "repair_patch_apply_json"),
        repair_patch_apply_status: metadata_value(metadata, "repair_patch_apply_status"),
        repair_patch_apply_applied: metadata_value(metadata, "repair_patch_apply_applied"),
        repair_patch_apply_authorized: metadata_value(metadata, "repair_patch_apply_authorized"),
        repair_patch_apply_target: metadata_value(metadata, "repair_patch_apply_target"),
        repair_patch_apply_patch_file: metadata_value(metadata, "repair_patch_apply_patch_file"),
        repair_patch_apply_required_grant: metadata_value(
            metadata,
            "repair_patch_apply_required_grant",
        ),
        repair_patch_apply_grant_command: metadata_value(
            metadata,
            "repair_patch_apply_grant_command",
        ),
        repair_patch_apply_apply_command: metadata_value(
            metadata,
            "repair_patch_apply_apply_command",
        ),
        repair_patch_apply_summary: metadata_value(metadata, "repair_patch_apply_summary"),
        repair_patch_apply_effectiveness: metadata_value(
            metadata,
            "repair_patch_apply_effectiveness",
        ),
        repair_patch_apply_effectiveness_json: metadata_value(
            metadata,
            "repair_patch_apply_effectiveness_json",
        ),
        repair_patch_apply_effectiveness_used_count: metadata_value(
            metadata,
            "repair_patch_apply_effectiveness_used_count",
        ),
        repair_patch_apply_effectiveness_satisfied_count: metadata_value(
            metadata,
            "repair_patch_apply_effectiveness_satisfied_count",
        ),
        repair_patch_apply_effectiveness_partial_count: metadata_value(
            metadata,
            "repair_patch_apply_effectiveness_partial_count",
        ),
        repair_patch_apply_effectiveness_failed_count: metadata_value(
            metadata,
            "repair_patch_apply_effectiveness_failed_count",
        ),
        repair_patch_apply_effectiveness_success_rate: metadata_value(
            metadata,
            "repair_patch_apply_effectiveness_success_rate",
        ),
        repair_patch_apply_effectiveness_failure_rate: metadata_value(
            metadata,
            "repair_patch_apply_effectiveness_failure_rate",
        ),
        repair_patch_apply_effectiveness_top_reuse: metadata_value(
            metadata,
            "repair_patch_apply_effectiveness_top_reuse",
        ),
        repair_patch_apply_effectiveness_top_avoid: metadata_value(
            metadata,
            "repair_patch_apply_effectiveness_top_avoid",
        ),
        repair_patch_verify: metadata_value(metadata, "repair_patch_verify"),
        repair_patch_verify_json: metadata_value(metadata, "repair_patch_verify_json"),
        repair_patch_verify_status: metadata_value(metadata, "repair_patch_verify_status"),
        repair_patch_verify_passed: metadata_value(metadata, "repair_patch_verify_passed"),
        repair_patch_verify_target: metadata_value(metadata, "repair_patch_verify_target"),
        repair_patch_verify_command: metadata_value(metadata, "repair_patch_verify_command"),
        repair_patch_verify_summary: metadata_value(metadata, "repair_patch_verify_summary"),
        repair_patch_verify_effectiveness: metadata_value(
            metadata,
            "repair_patch_verify_effectiveness",
        ),
        repair_patch_verify_effectiveness_json: metadata_value(
            metadata,
            "repair_patch_verify_effectiveness_json",
        ),
        repair_patch_verify_effectiveness_used_count: metadata_value(
            metadata,
            "repair_patch_verify_effectiveness_used_count",
        ),
        repair_patch_verify_effectiveness_satisfied_count: metadata_value(
            metadata,
            "repair_patch_verify_effectiveness_satisfied_count",
        ),
        repair_patch_verify_effectiveness_partial_count: metadata_value(
            metadata,
            "repair_patch_verify_effectiveness_partial_count",
        ),
        repair_patch_verify_effectiveness_failed_count: metadata_value(
            metadata,
            "repair_patch_verify_effectiveness_failed_count",
        ),
        repair_patch_verify_effectiveness_success_rate: metadata_value(
            metadata,
            "repair_patch_verify_effectiveness_success_rate",
        ),
        repair_patch_verify_effectiveness_failure_rate: metadata_value(
            metadata,
            "repair_patch_verify_effectiveness_failure_rate",
        ),
        repair_patch_verify_effectiveness_top_reuse: metadata_value(
            metadata,
            "repair_patch_verify_effectiveness_top_reuse",
        ),
        repair_patch_verify_effectiveness_top_avoid: metadata_value(
            metadata,
            "repair_patch_verify_effectiveness_top_avoid",
        ),
        repair_patch_learning: metadata_value(metadata, "repair_patch_learning"),
        repair_patch_learning_json: metadata_value(metadata, "repair_patch_learning_json"),
        repair_patch_learning_status: metadata_value(metadata, "repair_patch_learning_status"),
        repair_patch_learning_used_count: metadata_value(
            metadata,
            "repair_patch_learning_used_count",
        ),
        repair_patch_learning_verified_count: metadata_value(
            metadata,
            "repair_patch_learning_verified_count",
        ),
        repair_patch_learning_verified_satisfied_count: metadata_value(
            metadata,
            "repair_patch_learning_verified_satisfied_count",
        ),
        repair_patch_learning_verified_failed_count: metadata_value(
            metadata,
            "repair_patch_learning_verified_failed_count",
        ),
        repair_patch_learning_success_rate: metadata_value(
            metadata,
            "repair_patch_learning_success_rate",
        ),
        repair_patch_learning_verified_success_rate: metadata_value(
            metadata,
            "repair_patch_learning_verified_success_rate",
        ),
        repair_patch_learning_top_reuse: metadata_value(
            metadata,
            "repair_patch_learning_top_reuse",
        ),
        repair_patch_learning_top_avoid: metadata_value(
            metadata,
            "repair_patch_learning_top_avoid",
        ),
        repair_patch_learning_next_need_kind: metadata_value(
            metadata,
            "repair_patch_learning_next_need_kind",
        ),
        repair_patch_learning_next_need_query: metadata_value(
            metadata,
            "repair_patch_learning_next_need_query",
        ),
        repair_patch_learning_effectiveness: metadata_value(
            metadata,
            "repair_patch_learning_effectiveness",
        ),
        repair_patch_learning_effectiveness_json: metadata_value(
            metadata,
            "repair_patch_learning_effectiveness_json",
        ),
        repair_patch_learning_effectiveness_used_count: metadata_value(
            metadata,
            "repair_patch_learning_effectiveness_used_count",
        ),
        repair_patch_learning_effectiveness_satisfied_count: metadata_value(
            metadata,
            "repair_patch_learning_effectiveness_satisfied_count",
        ),
        repair_patch_learning_effectiveness_partial_count: metadata_value(
            metadata,
            "repair_patch_learning_effectiveness_partial_count",
        ),
        repair_patch_learning_effectiveness_failed_count: metadata_value(
            metadata,
            "repair_patch_learning_effectiveness_failed_count",
        ),
        repair_patch_learning_effectiveness_success_rate: metadata_value(
            metadata,
            "repair_patch_learning_effectiveness_success_rate",
        ),
        repair_patch_learning_effectiveness_failure_rate: metadata_value(
            metadata,
            "repair_patch_learning_effectiveness_failure_rate",
        ),
        repair_patch_learning_effectiveness_top_reuse: metadata_value(
            metadata,
            "repair_patch_learning_effectiveness_top_reuse",
        ),
        repair_patch_learning_effectiveness_top_avoid: metadata_value(
            metadata,
            "repair_patch_learning_effectiveness_top_avoid",
        ),
        repair_patch_strategy: metadata_value(metadata, "repair_patch_strategy"),
        repair_patch_strategy_json: metadata_value(metadata, "repair_patch_strategy_json"),
        repair_patch_strategy_status: metadata_value(metadata, "repair_patch_strategy_status"),
        repair_patch_strategy_focus: metadata_value(metadata, "repair_patch_strategy_focus"),
        repair_patch_strategy_learned_reuse: metadata_value(
            metadata,
            "repair_patch_strategy_learned_reuse",
        ),
        repair_patch_strategy_learned_avoid: metadata_value(
            metadata,
            "repair_patch_strategy_learned_avoid",
        ),
        repair_patch_strategy_next_need_kind: metadata_value(
            metadata,
            "repair_patch_strategy_next_need_kind",
        ),
        repair_patch_strategy_next_need_query: metadata_value(
            metadata,
            "repair_patch_strategy_next_need_query",
        ),
        repair_patch_strategy_effectiveness: metadata_value(
            metadata,
            "repair_patch_strategy_effectiveness",
        ),
        repair_patch_strategy_effectiveness_json: metadata_value(
            metadata,
            "repair_patch_strategy_effectiveness_json",
        ),
        repair_patch_strategy_effectiveness_used_count: metadata_value(
            metadata,
            "repair_patch_strategy_effectiveness_used_count",
        ),
        repair_patch_strategy_effectiveness_satisfied_count: metadata_value(
            metadata,
            "repair_patch_strategy_effectiveness_satisfied_count",
        ),
        repair_patch_strategy_effectiveness_partial_count: metadata_value(
            metadata,
            "repair_patch_strategy_effectiveness_partial_count",
        ),
        repair_patch_strategy_effectiveness_failed_count: metadata_value(
            metadata,
            "repair_patch_strategy_effectiveness_failed_count",
        ),
        repair_patch_strategy_effectiveness_success_rate: metadata_value(
            metadata,
            "repair_patch_strategy_effectiveness_success_rate",
        ),
        repair_patch_strategy_effectiveness_failure_rate: metadata_value(
            metadata,
            "repair_patch_strategy_effectiveness_failure_rate",
        ),
        repair_patch_strategy_effectiveness_top_reuse: metadata_value(
            metadata,
            "repair_patch_strategy_effectiveness_top_reuse",
        ),
        repair_patch_strategy_effectiveness_top_avoid: metadata_value(
            metadata,
            "repair_patch_strategy_effectiveness_top_avoid",
        ),
        repair_command_effectiveness: metadata_value(metadata, "repair_command_effectiveness"),
        repair_command_effectiveness_json: metadata_value(
            metadata,
            "repair_command_effectiveness_json",
        ),
        repair_command_effectiveness_used_count: metadata_value(
            metadata,
            "repair_command_effectiveness_used_count",
        ),
        repair_command_effectiveness_satisfied_count: metadata_value(
            metadata,
            "repair_command_effectiveness_satisfied_count",
        ),
        repair_command_effectiveness_partial_count: metadata_value(
            metadata,
            "repair_command_effectiveness_partial_count",
        ),
        repair_command_effectiveness_failed_count: metadata_value(
            metadata,
            "repair_command_effectiveness_failed_count",
        ),
        repair_command_effectiveness_success_rate: metadata_value(
            metadata,
            "repair_command_effectiveness_success_rate",
        ),
        repair_command_effectiveness_failure_rate: metadata_value(
            metadata,
            "repair_command_effectiveness_failure_rate",
        ),
        repair_command_effectiveness_top_reuse: metadata_value(
            metadata,
            "repair_command_effectiveness_top_reuse",
        ),
        repair_command_effectiveness_top_avoid: metadata_value(
            metadata,
            "repair_command_effectiveness_top_avoid",
        ),
        repair_command_strategy: metadata_value(metadata, "repair_command_strategy"),
        repair_command_strategy_json: metadata_value(metadata, "repair_command_strategy_json"),
        repair_command_strategy_status: metadata_value(metadata, "repair_command_strategy_status"),
        repair_command_strategy_focus: metadata_value(metadata, "repair_command_strategy_focus"),
        repair_command_strategy_learned_reuse: metadata_value(
            metadata,
            "repair_command_strategy_learned_reuse",
        ),
        repair_command_strategy_learned_avoid: metadata_value(
            metadata,
            "repair_command_strategy_learned_avoid",
        ),
        repair_command_strategy_current_apply: metadata_value(
            metadata,
            "repair_command_strategy_current_apply",
        ),
        repair_command_strategy_next_need_kind: metadata_value(
            metadata,
            "repair_command_strategy_next_need_kind",
        ),
        repair_command_strategy_next_need_query: metadata_value(
            metadata,
            "repair_command_strategy_next_need_query",
        ),
        repair_command_strategy_effectiveness: metadata_value(
            metadata,
            "repair_command_strategy_effectiveness",
        ),
        repair_command_strategy_effectiveness_json: metadata_value(
            metadata,
            "repair_command_strategy_effectiveness_json",
        ),
        repair_command_strategy_effectiveness_used_count: metadata_value(
            metadata,
            "repair_command_strategy_effectiveness_used_count",
        ),
        repair_command_strategy_effectiveness_satisfied_count: metadata_value(
            metadata,
            "repair_command_strategy_effectiveness_satisfied_count",
        ),
        repair_command_strategy_effectiveness_partial_count: metadata_value(
            metadata,
            "repair_command_strategy_effectiveness_partial_count",
        ),
        repair_command_strategy_effectiveness_failed_count: metadata_value(
            metadata,
            "repair_command_strategy_effectiveness_failed_count",
        ),
        repair_command_strategy_effectiveness_success_rate: metadata_value(
            metadata,
            "repair_command_strategy_effectiveness_success_rate",
        ),
        repair_command_strategy_effectiveness_failure_rate: metadata_value(
            metadata,
            "repair_command_strategy_effectiveness_failure_rate",
        ),
        repair_command_strategy_effectiveness_top_reuse: metadata_value(
            metadata,
            "repair_command_strategy_effectiveness_top_reuse",
        ),
        repair_command_strategy_effectiveness_top_avoid: metadata_value(
            metadata,
            "repair_command_strategy_effectiveness_top_avoid",
        ),
        action_trace: metadata_value(metadata, "action_trace"),
        action_trace_json: metadata_value(metadata, "action_trace_json"),
        action_trace_status: metadata_value(metadata, "action_trace_status"),
        action_trace_stage_count: metadata_value(metadata, "action_trace_stage_count"),
        action_trace_last_action: metadata_value(metadata, "action_trace_last_action"),
        action_trace_effectiveness: metadata_value(metadata, "action_trace_effectiveness"),
        action_trace_effectiveness_json: metadata_value(
            metadata,
            "action_trace_effectiveness_json",
        ),
        action_trace_effectiveness_used_count: metadata_value(
            metadata,
            "action_trace_effectiveness_used_count",
        ),
        action_trace_effectiveness_satisfied_count: metadata_value(
            metadata,
            "action_trace_effectiveness_satisfied_count",
        ),
        action_trace_effectiveness_partial_count: metadata_value(
            metadata,
            "action_trace_effectiveness_partial_count",
        ),
        action_trace_effectiveness_failed_count: metadata_value(
            metadata,
            "action_trace_effectiveness_failed_count",
        ),
        action_trace_effectiveness_success_rate: metadata_value(
            metadata,
            "action_trace_effectiveness_success_rate",
        ),
        action_trace_effectiveness_failure_rate: metadata_value(
            metadata,
            "action_trace_effectiveness_failure_rate",
        ),
        action_trace_effectiveness_top_reuse: metadata_value(
            metadata,
            "action_trace_effectiveness_top_reuse",
        ),
        action_trace_effectiveness_top_avoid: metadata_value(
            metadata,
            "action_trace_effectiveness_top_avoid",
        ),
        repair_recall: metadata_value(metadata, "repair_recall"),
        repair_recall_match_count: metadata_value(metadata, "repair_recall_match_count"),
        repair_recall_top_status: metadata_value(metadata, "repair_recall_top_status"),
        repair_recall_top_score: metadata_value(metadata, "repair_recall_top_score"),
        repair_recall_top_reasons: metadata_value(metadata, "repair_recall_top_reasons"),
        repair_recall_top_summary: metadata_value(metadata, "repair_recall_top_summary"),
        repair_lessons: metadata_value(metadata, "repair_lessons"),
        repair_lessons_json: metadata_value(metadata, "repair_lessons_json"),
        repair_lessons_count: metadata_value(metadata, "repair_lessons_count"),
        repair_lessons_reuse_count: metadata_value(metadata, "repair_lessons_reuse_count"),
        repair_lessons_avoid_count: metadata_value(metadata, "repair_lessons_avoid_count"),
        repair_lessons_top_reuse: metadata_value(metadata, "repair_lessons_top_reuse"),
        repair_lessons_top_avoid: metadata_value(metadata, "repair_lessons_top_avoid"),
        repair_lesson_effectiveness: metadata_value(metadata, "repair_lesson_effectiveness"),
        repair_lesson_effectiveness_json: metadata_value(
            metadata,
            "repair_lesson_effectiveness_json",
        ),
        repair_lesson_effectiveness_used_count: metadata_value(
            metadata,
            "repair_lesson_effectiveness_used_count",
        ),
        repair_lesson_effectiveness_satisfied_count: metadata_value(
            metadata,
            "repair_lesson_effectiveness_satisfied_count",
        ),
        repair_lesson_effectiveness_partial_count: metadata_value(
            metadata,
            "repair_lesson_effectiveness_partial_count",
        ),
        repair_lesson_effectiveness_failed_count: metadata_value(
            metadata,
            "repair_lesson_effectiveness_failed_count",
        ),
        repair_lesson_effectiveness_success_rate: metadata_value(
            metadata,
            "repair_lesson_effectiveness_success_rate",
        ),
        repair_lesson_effectiveness_failure_rate: metadata_value(
            metadata,
            "repair_lesson_effectiveness_failure_rate",
        ),
        repair_lesson_effectiveness_top_reuse: metadata_value(
            metadata,
            "repair_lesson_effectiveness_top_reuse",
        ),
        repair_lesson_effectiveness_top_avoid: metadata_value(
            metadata,
            "repair_lesson_effectiveness_top_avoid",
        ),
        repair_decision: metadata_value(metadata, "repair_decision"),
        repair_decision_json: metadata_value(metadata, "repair_decision_json"),
        repair_decision_status: metadata_value(metadata, "repair_decision_status"),
        repair_decision_kind: metadata_value(metadata, "repair_decision_kind"),
        repair_decision_focus: metadata_value(metadata, "repair_decision_focus"),
        repair_decision_next_need_kind: metadata_value(metadata, "repair_decision_next_need_kind"),
        repair_decision_next_need_query: metadata_value(
            metadata,
            "repair_decision_next_need_query",
        ),
        repair_decision_effectiveness: metadata_value(metadata, "repair_decision_effectiveness"),
        repair_decision_effectiveness_json: metadata_value(
            metadata,
            "repair_decision_effectiveness_json",
        ),
        repair_decision_effectiveness_used_count: metadata_value(
            metadata,
            "repair_decision_effectiveness_used_count",
        ),
        repair_decision_effectiveness_satisfied_count: metadata_value(
            metadata,
            "repair_decision_effectiveness_satisfied_count",
        ),
        repair_decision_effectiveness_partial_count: metadata_value(
            metadata,
            "repair_decision_effectiveness_partial_count",
        ),
        repair_decision_effectiveness_failed_count: metadata_value(
            metadata,
            "repair_decision_effectiveness_failed_count",
        ),
        repair_decision_effectiveness_success_rate: metadata_value(
            metadata,
            "repair_decision_effectiveness_success_rate",
        ),
        repair_decision_effectiveness_failure_rate: metadata_value(
            metadata,
            "repair_decision_effectiveness_failure_rate",
        ),
        repair_decision_effectiveness_top_reuse: metadata_value(
            metadata,
            "repair_decision_effectiveness_top_reuse",
        ),
        repair_decision_effectiveness_top_avoid: metadata_value(
            metadata,
            "repair_decision_effectiveness_top_avoid",
        ),
        harness_adaptation_effectiveness: metadata_value(
            metadata,
            "harness_adaptation_effectiveness",
        ),
        harness_adaptation_effectiveness_json: metadata_value(
            metadata,
            "harness_adaptation_effectiveness_json",
        ),
        harness_adaptation_effectiveness_used_count: metadata_value(
            metadata,
            "harness_adaptation_effectiveness_used_count",
        ),
        harness_adaptation_effectiveness_satisfied_count: metadata_value(
            metadata,
            "harness_adaptation_effectiveness_satisfied_count",
        ),
        harness_adaptation_effectiveness_partial_count: metadata_value(
            metadata,
            "harness_adaptation_effectiveness_partial_count",
        ),
        harness_adaptation_effectiveness_failed_count: metadata_value(
            metadata,
            "harness_adaptation_effectiveness_failed_count",
        ),
        harness_adaptation_effectiveness_success_rate: metadata_value(
            metadata,
            "harness_adaptation_effectiveness_success_rate",
        ),
        harness_adaptation_effectiveness_failure_rate: metadata_value(
            metadata,
            "harness_adaptation_effectiveness_failure_rate",
        ),
        harness_adaptation_effectiveness_top_reuse: metadata_value(
            metadata,
            "harness_adaptation_effectiveness_top_reuse",
        ),
        harness_adaptation_effectiveness_top_avoid: metadata_value(
            metadata,
            "harness_adaptation_effectiveness_top_avoid",
        ),
        harness_environment_profile: metadata_value(metadata, "harness_environment_profile"),
        harness_environment_profile_json: metadata_value(
            metadata,
            "harness_environment_profile_json",
        ),
        harness_environment_profile_status: metadata_value(
            metadata,
            "harness_environment_profile_status",
        ),
        harness_environment_profile_id: metadata_value(metadata, "harness_environment_profile_id"),
        harness_environment_profile_execution_mode: metadata_value(
            metadata,
            "harness_environment_profile_execution_mode",
        ),
        harness_environment_profile_provider_mode: metadata_value(
            metadata,
            "harness_environment_profile_provider_mode",
        ),
        harness_environment_profile_desktop_mode: metadata_value(
            metadata,
            "harness_environment_profile_desktop_mode",
        ),
        harness_environment_profile_capabilities: metadata_value(
            metadata,
            "harness_environment_profile_capabilities",
        ),
        harness_environment_profile_constraints: metadata_value(
            metadata,
            "harness_environment_profile_constraints",
        ),
        harness_environment_profile_installed_profiles: metadata_value(
            metadata,
            "harness_environment_profile_installed_profiles",
        ),
        harness_environment_profile_next_need_kind: metadata_value(
            metadata,
            "harness_environment_profile_next_need_kind",
        ),
        harness_environment_profile_next_need_query: metadata_value(
            metadata,
            "harness_environment_profile_next_need_query",
        ),
        harness_environment_drift: metadata_value(metadata, "harness_environment_drift"),
        harness_environment_drift_json: metadata_value(metadata, "harness_environment_drift_json"),
        harness_environment_drift_status: metadata_value(
            metadata,
            "harness_environment_drift_status",
        ),
        harness_environment_drift_summary: metadata_value(
            metadata,
            "harness_environment_drift_summary",
        ),
        harness_environment_drift_detail: metadata_value(
            metadata,
            "harness_environment_drift_detail",
        ),
        harness_environment_drift_history_count: metadata_value(
            metadata,
            "harness_environment_drift_history_count",
        ),
        harness_environment_drift_gained_count: metadata_value(
            metadata,
            "harness_environment_drift_gained_count",
        ),
        harness_environment_drift_lost_count: metadata_value(
            metadata,
            "harness_environment_drift_lost_count",
        ),
        harness_environment_drift_next_need_kind: metadata_value(
            metadata,
            "harness_environment_drift_next_need_kind",
        ),
        harness_environment_drift_next_need_query: metadata_value(
            metadata,
            "harness_environment_drift_next_need_query",
        ),
        harness_environment_drift_effectiveness: metadata_value(
            metadata,
            "harness_environment_drift_effectiveness",
        ),
        harness_environment_drift_effectiveness_json: metadata_value(
            metadata,
            "harness_environment_drift_effectiveness_json",
        ),
        harness_environment_drift_effectiveness_used_count: metadata_value(
            metadata,
            "harness_environment_drift_effectiveness_used_count",
        ),
        harness_environment_drift_effectiveness_satisfied_count: metadata_value(
            metadata,
            "harness_environment_drift_effectiveness_satisfied_count",
        ),
        harness_environment_drift_effectiveness_partial_count: metadata_value(
            metadata,
            "harness_environment_drift_effectiveness_partial_count",
        ),
        harness_environment_drift_effectiveness_failed_count: metadata_value(
            metadata,
            "harness_environment_drift_effectiveness_failed_count",
        ),
        harness_environment_drift_effectiveness_success_rate: metadata_value(
            metadata,
            "harness_environment_drift_effectiveness_success_rate",
        ),
        harness_environment_drift_effectiveness_failure_rate: metadata_value(
            metadata,
            "harness_environment_drift_effectiveness_failure_rate",
        ),
        harness_environment_drift_effectiveness_top_reuse: metadata_value(
            metadata,
            "harness_environment_drift_effectiveness_top_reuse",
        ),
        harness_environment_drift_effectiveness_top_avoid: metadata_value(
            metadata,
            "harness_environment_drift_effectiveness_top_avoid",
        ),
        repair_effectiveness_rollup: metadata_value(metadata, "repair_effectiveness_rollup"),
        repair_effectiveness_rollup_json: metadata_value(
            metadata,
            "repair_effectiveness_rollup_json",
        ),
        repair_effectiveness_rollup_status: metadata_value(
            metadata,
            "repair_effectiveness_rollup_status",
        ),
        repair_effectiveness_rollup_source_count: metadata_value(
            metadata,
            "repair_effectiveness_rollup_source_count",
        ),
        repair_effectiveness_rollup_active_source_count: metadata_value(
            metadata,
            "repair_effectiveness_rollup_active_source_count",
        ),
        repair_effectiveness_rollup_used_count: metadata_value(
            metadata,
            "repair_effectiveness_rollup_used_count",
        ),
        repair_effectiveness_rollup_satisfied_count: metadata_value(
            metadata,
            "repair_effectiveness_rollup_satisfied_count",
        ),
        repair_effectiveness_rollup_partial_count: metadata_value(
            metadata,
            "repair_effectiveness_rollup_partial_count",
        ),
        repair_effectiveness_rollup_failed_count: metadata_value(
            metadata,
            "repair_effectiveness_rollup_failed_count",
        ),
        repair_effectiveness_rollup_success_rate: metadata_value(
            metadata,
            "repair_effectiveness_rollup_success_rate",
        ),
        repair_effectiveness_rollup_failure_rate: metadata_value(
            metadata,
            "repair_effectiveness_rollup_failure_rate",
        ),
        repair_effectiveness_rollup_strongest_source: metadata_value(
            metadata,
            "repair_effectiveness_rollup_strongest_source",
        ),
        repair_effectiveness_rollup_weakest_source: metadata_value(
            metadata,
            "repair_effectiveness_rollup_weakest_source",
        ),
        repair_effectiveness_rollup_top_reuse: metadata_value(
            metadata,
            "repair_effectiveness_rollup_top_reuse",
        ),
        repair_effectiveness_rollup_top_avoid: metadata_value(
            metadata,
            "repair_effectiveness_rollup_top_avoid",
        ),
        repair_effectiveness_rollup_next_need_kind: metadata_value(
            metadata,
            "repair_effectiveness_rollup_next_need_kind",
        ),
        repair_effectiveness_rollup_next_need_query: metadata_value(
            metadata,
            "repair_effectiveness_rollup_next_need_query",
        ),
        repair_effectiveness_rollup_effectiveness: metadata_value(
            metadata,
            "repair_effectiveness_rollup_effectiveness",
        ),
        repair_effectiveness_rollup_effectiveness_json: metadata_value(
            metadata,
            "repair_effectiveness_rollup_effectiveness_json",
        ),
        repair_effectiveness_rollup_effectiveness_used_count: metadata_value(
            metadata,
            "repair_effectiveness_rollup_effectiveness_used_count",
        ),
        repair_effectiveness_rollup_effectiveness_satisfied_count: metadata_value(
            metadata,
            "repair_effectiveness_rollup_effectiveness_satisfied_count",
        ),
        repair_effectiveness_rollup_effectiveness_partial_count: metadata_value(
            metadata,
            "repair_effectiveness_rollup_effectiveness_partial_count",
        ),
        repair_effectiveness_rollup_effectiveness_failed_count: metadata_value(
            metadata,
            "repair_effectiveness_rollup_effectiveness_failed_count",
        ),
        repair_effectiveness_rollup_effectiveness_success_rate: metadata_value(
            metadata,
            "repair_effectiveness_rollup_effectiveness_success_rate",
        ),
        repair_effectiveness_rollup_effectiveness_failure_rate: metadata_value(
            metadata,
            "repair_effectiveness_rollup_effectiveness_failure_rate",
        ),
        repair_effectiveness_rollup_effectiveness_top_reuse: metadata_value(
            metadata,
            "repair_effectiveness_rollup_effectiveness_top_reuse",
        ),
        repair_effectiveness_rollup_effectiveness_top_avoid: metadata_value(
            metadata,
            "repair_effectiveness_rollup_effectiveness_top_avoid",
        ),
        repair_effectiveness_rollup_adaptation: metadata_value(
            metadata,
            "repair_effectiveness_rollup_adaptation",
        ),
        repair_effectiveness_rollup_adaptation_json: metadata_value(
            metadata,
            "repair_effectiveness_rollup_adaptation_json",
        ),
        repair_effectiveness_rollup_adaptation_status: metadata_value(
            metadata,
            "repair_effectiveness_rollup_adaptation_status",
        ),
        repair_effectiveness_rollup_adaptation_focus: metadata_value(
            metadata,
            "repair_effectiveness_rollup_adaptation_focus",
        ),
        repair_effectiveness_rollup_adaptation_rollup_status: metadata_value(
            metadata,
            "repair_effectiveness_rollup_adaptation_rollup_status",
        ),
        repair_effectiveness_rollup_adaptation_guidance_used_count: metadata_value(
            metadata,
            "repair_effectiveness_rollup_adaptation_guidance_used_count",
        ),
        repair_effectiveness_rollup_adaptation_guidance_satisfied_count: metadata_value(
            metadata,
            "repair_effectiveness_rollup_adaptation_guidance_satisfied_count",
        ),
        repair_effectiveness_rollup_adaptation_guidance_partial_count: metadata_value(
            metadata,
            "repair_effectiveness_rollup_adaptation_guidance_partial_count",
        ),
        repair_effectiveness_rollup_adaptation_guidance_failed_count: metadata_value(
            metadata,
            "repair_effectiveness_rollup_adaptation_guidance_failed_count",
        ),
        repair_effectiveness_rollup_adaptation_guidance_success_rate: metadata_value(
            metadata,
            "repair_effectiveness_rollup_adaptation_guidance_success_rate",
        ),
        repair_effectiveness_rollup_adaptation_guidance_failure_rate: metadata_value(
            metadata,
            "repair_effectiveness_rollup_adaptation_guidance_failure_rate",
        ),
        repair_effectiveness_rollup_adaptation_next_need_kind: metadata_value(
            metadata,
            "repair_effectiveness_rollup_adaptation_next_need_kind",
        ),
        repair_effectiveness_rollup_adaptation_next_need_query: metadata_value(
            metadata,
            "repair_effectiveness_rollup_adaptation_next_need_query",
        ),
        repair_queue_brief_effectiveness: metadata_value(
            metadata,
            "repair_queue_brief_effectiveness",
        ),
        repair_queue_brief_effectiveness_json: metadata_value(
            metadata,
            "repair_queue_brief_effectiveness_json",
        ),
        repair_queue_brief_effectiveness_used_count: metadata_value(
            metadata,
            "repair_queue_brief_effectiveness_used_count",
        ),
        repair_queue_brief_effectiveness_satisfied_count: metadata_value(
            metadata,
            "repair_queue_brief_effectiveness_satisfied_count",
        ),
        repair_queue_brief_effectiveness_partial_count: metadata_value(
            metadata,
            "repair_queue_brief_effectiveness_partial_count",
        ),
        repair_queue_brief_effectiveness_failed_count: metadata_value(
            metadata,
            "repair_queue_brief_effectiveness_failed_count",
        ),
        repair_queue_brief_effectiveness_success_rate: metadata_value(
            metadata,
            "repair_queue_brief_effectiveness_success_rate",
        ),
        repair_queue_brief_effectiveness_failure_rate: metadata_value(
            metadata,
            "repair_queue_brief_effectiveness_failure_rate",
        ),
        repair_queue_brief_effectiveness_top_reuse: metadata_value(
            metadata,
            "repair_queue_brief_effectiveness_top_reuse",
        ),
        repair_queue_brief_effectiveness_top_avoid: metadata_value(
            metadata,
            "repair_queue_brief_effectiveness_top_avoid",
        ),
        repair_queue_brief_adaptation: metadata_value(metadata, "repair_queue_brief_adaptation"),
        repair_queue_brief_adaptation_json: metadata_value(
            metadata,
            "repair_queue_brief_adaptation_json",
        ),
        repair_queue_brief_adaptation_status: metadata_value(
            metadata,
            "repair_queue_brief_adaptation_status",
        ),
        repair_queue_brief_adaptation_focus: metadata_value(
            metadata,
            "repair_queue_brief_adaptation_focus",
        ),
        repair_queue_brief_adaptation_queue_used_count: metadata_value(
            metadata,
            "repair_queue_brief_adaptation_queue_used_count",
        ),
        repair_queue_brief_adaptation_queue_satisfied_count: metadata_value(
            metadata,
            "repair_queue_brief_adaptation_queue_satisfied_count",
        ),
        repair_queue_brief_adaptation_queue_partial_count: metadata_value(
            metadata,
            "repair_queue_brief_adaptation_queue_partial_count",
        ),
        repair_queue_brief_adaptation_queue_failed_count: metadata_value(
            metadata,
            "repair_queue_brief_adaptation_queue_failed_count",
        ),
        repair_queue_brief_adaptation_queue_success_rate: metadata_value(
            metadata,
            "repair_queue_brief_adaptation_queue_success_rate",
        ),
        repair_queue_brief_adaptation_queue_failure_rate: metadata_value(
            metadata,
            "repair_queue_brief_adaptation_queue_failure_rate",
        ),
        repair_queue_brief_adaptation_next_need_kind: metadata_value(
            metadata,
            "repair_queue_brief_adaptation_next_need_kind",
        ),
        repair_queue_brief_adaptation_next_need_query: metadata_value(
            metadata,
            "repair_queue_brief_adaptation_next_need_query",
        ),
        environment_profile_journal: metadata_value(metadata, "environment_profile_journal"),
        harness_adaptation: metadata_value(metadata, "harness_adaptation"),
        harness_adaptation_json: metadata_value(metadata, "harness_adaptation_json"),
        harness_adaptation_status: metadata_value(metadata, "harness_adaptation_status"),
        harness_adaptation_focus: metadata_value(metadata, "harness_adaptation_focus"),
        harness_adaptation_next_need_kind: metadata_value(
            metadata,
            "harness_adaptation_next_need_kind",
        ),
        harness_adaptation_next_need_query: metadata_value(
            metadata,
            "harness_adaptation_next_need_query",
        ),
        harness_adaptation_target: metadata_value(metadata, "harness_adaptation_target"),
        harness_adaptation_environment: metadata_value(metadata, "harness_adaptation_environment"),
        code_context: metadata_value(metadata, "code_context"),
        adapter_context: metadata_value(metadata, "adapter_context"),
        adapter_context_status: metadata_value(metadata, "adapter_context_status"),
        adapter_context_missing_core: metadata_value(metadata, "adapter_context_missing_core"),
        adapter_context_provider_env: metadata_value(metadata, "adapter_context_provider_env"),
        adapter_context_provider_keys: metadata_value(metadata, "adapter_context_provider_keys"),
        adapter_context_desktop: metadata_value(metadata, "adapter_context_desktop"),
        field_trajectory: metadata_value(metadata, "field_trajectory"),
        field_trajectory_field: metadata_value(metadata, "field_trajectory_field"),
        field_trajectory_mini_task: metadata_value(metadata, "field_trajectory_mini_task"),
        field_trajectory_verifier_status: metadata_value(
            metadata,
            "field_trajectory_verifier_status",
        ),
        field_trajectory_verifier_error: metadata_value(
            metadata,
            "field_trajectory_verifier_error",
        ),
        status: metadata_value(metadata, "repair_plan_status"),
        schema: metadata_value(metadata, "repair_plan_schema"),
        session: metadata_value(metadata, "repair_plan_session"),
        target_tentacle: metadata_value(metadata, "target_tentacle"),
        target_tool: metadata_value(metadata, "target_tool"),
        candidate: metadata_value(metadata, "candidate"),
        review_boundary: metadata_value(metadata, "review_boundary"),
        check_command: metadata_value(metadata, "check_command"),
        grant_command: metadata_value(metadata, "grant_command"),
        apply_command: metadata_value(metadata, "apply_command"),
        score_command,
        score_commands,
        suggested_commands: metadata_value(metadata, "suggested_commands"),
        next_need_kind: metadata_value(metadata, "next_need_kind"),
        next_need_query: metadata_value(metadata, "next_need_query"),
    })
}

fn metadata_value(metadata: &BTreeMap<String, String>, key: &str) -> String {
    metadata.get(key).cloned().unwrap_or_default()
}

pub(crate) fn repair_score_commands_from_metadata(
    metadata: &BTreeMap<String, String>,
) -> Option<Vec<String>> {
    let value = metadata.get("score_commands")?;
    let commands = value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            metadata
                .get("feed_trace_index")
                .map(|index| line.replace("<trace-index>", index))
                .unwrap_or_else(|| line.to_string())
        })
        .collect::<Vec<_>>();
    (!commands.is_empty()).then_some(commands)
}

pub(crate) fn repair_next_need_from_feed(feed: &Feed) -> Option<GoalNeedSuggestion> {
    let query = feed
        .metadata
        .get("next_need_query")
        .or_else(|| feed.metadata.get("next_need"))?
        .trim();
    if query.is_empty() {
        return None;
    }
    let kind = feed
        .metadata
        .get("next_need_kind")
        .and_then(|value| parse_kind(value).ok())
        .unwrap_or(NeedKind::Verify);
    Some(GoalNeedSuggestion {
        kind,
        query: query.to_string(),
    })
}
