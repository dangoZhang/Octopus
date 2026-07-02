use crate::{HarnessLearningSummary, NeedQueueItem};

pub(crate) fn goal_hint(
    has_installed_tentacles: bool,
    has_goal: bool,
    pending_need: Option<&NeedQueueItem>,
    has_active_parallel_field_goal: bool,
    harness_learning: &HarnessLearningSummary,
) -> String {
    if !has_installed_tentacles {
        return "Set a Goal; Octopus will prepare the local harness.".to_string();
    }
    if !has_goal {
        return "Describe the Goal you want Octopus to pursue.".to_string();
    }
    if let Some(item) = pending_need {
        return format!(
            "Octopus has a pending {} Need: {}",
            crate::kind_key(&item.need.kind),
            one_line(&item.need.query)
        );
    }
    if has_active_parallel_field_goal {
        return "Octopus is adapting field harnesses from the current Goal.".to_string();
    }
    if harness_learning.source != "none" {
        return "Review the latest Feed and refine the Goal if the direction changed.".to_string();
    }
    "Refine the Goal or wait for the next Feed.".to_string()
}

pub fn need_queue_hints(pending: &[NeedQueueItem]) -> Vec<String> {
    if let Some(item) = pending.first() {
        return vec![format!(
            "Octopus has a pending {} Need: {}. Keep the Goal, refine it, or let the harness continue.",
            crate::kind_key(&item.need.kind),
            one_line(&item.need.query)
        )];
    }
    vec!["Describe or refine the Goal so Octopus can produce the next Need.".to_string()]
}

pub fn product_gap_hint(id: &str, impact: &str) -> String {
    match id {
        "goal_loop_empty" => "Set one clear Goal in the app.".to_string(),
        "no_installed_tentacles" => {
            "Start Octopus once so it can prepare local harnesses.".to_string()
        }
        "broken_manifests" | "broken_installed_seed_sources" => {
            "Let Octopus repair the editable harness layer before judging product behavior."
                .to_string()
        }
        "provider_feedback" | "provider_matrix_record" => {
            "Connect a model provider, then run the Goal again.".to_string()
        }
        "feed_trace_empty" => {
            "Run the current Goal until Octopus produces its first Feed.".to_string()
        }
        "check_history_empty" => {
            "Run a real harness check so evolution can learn from failure.".to_string()
        }
        "profile_registry_invalid" => {
            "Refresh the local profile registry before installing tentacles.".to_string()
        }
        "core_harness_boundary" => {
            "Fix the stable-core/editable-harness boundary warning first.".to_string()
        }
        "evolution_scores_empty" => {
            "Record whether the latest harness change helped, then continue evolution.".to_string()
        }
        "repair_outcomes_empty" => {
            "Let a repair session finish and record the outcome.".to_string()
        }
        "environment_signal_sparse" => {
            "Give Octopus one more local project signal so routing can adapt.".to_string()
        }
        "real_machine_gate" => {
            "Record one real local run before tagging the next release.".to_string()
        }
        _ => format!("Review {id}: {}", one_line(impact)),
    }
}

pub fn field_pool_hints(
    field_slot_count: usize,
    completed_fields: usize,
    active_slot_field: Option<&str>,
) -> Vec<String> {
    if field_slot_count > 0 && completed_fields == field_slot_count {
        return vec![
            "All current field mini tasks are complete; add a harder layer if the Goal needs stronger adaptation."
                .to_string(),
        ];
    }
    if let Some(field) = active_slot_field {
        return vec![format!(
            "Watch the {field} field Feed, then refine the Goal if the direction changed."
        )];
    }
    vec!["Let Octopus continue adapting the field harnesses from the current Goal.".to_string()]
}

pub fn field_slot_hint(
    field: &str,
    completed: bool,
    next_mini_task: Option<&str>,
    needs_environment: bool,
    needs_repair: bool,
) -> String {
    if needs_environment {
        return format!(
            "Prepare the runtime environment for the {field} field, then rerun the same Feed."
        );
    }
    if needs_repair {
        return format!("Let Octopus repair the {field} harness before continuing this field.");
    }
    if completed {
        return format!("{field} field is complete for the current mini task layer.");
    }
    if let Some(task) = next_mini_task {
        return format!("Next {field} field Feed should cover {task}.");
    }
    format!("Let Octopus prepare the next {field} field Need.")
}

fn one_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
