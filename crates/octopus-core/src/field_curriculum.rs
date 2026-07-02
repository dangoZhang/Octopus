use crate::{shell_arg, FieldTrajectorySummary};

const FIELD_CURRICULUM_ORDER: &[&str] = &[
    "math",
    "write",
    "translate",
    "search",
    "code",
    "swe",
    "research",
    "computer-use",
    "ib",
    "robotics",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FieldCurriculumStep {
    pub(crate) field: String,
    pub(crate) objective: String,
    pub(crate) next_action: String,
    pub(crate) reason: String,
}

pub(crate) fn field_harder_layer_objective_for(field: &str) -> String {
    format!(
        "add a harder {field} mini task layer; update field-packs/{field} and the matching editable field-mini-task Go worker only; require artifact-backed Feed evidence"
    )
}

pub(crate) fn field_harder_layer_next_action_for_field(state_args: &str, field: &str) -> String {
    format!(
        "octopus{state_args} evolve recommend field-mini-task {}",
        shell_arg(&field_harder_layer_objective_for(field))
    )
}

pub(crate) fn select_next_harder_field(
    summaries: &[FieldTrajectorySummary],
    state_args: &str,
) -> Option<FieldCurriculumStep> {
    let summary = summaries
        .iter()
        .filter(|summary| {
            summary.ready_for_harder_task
                && !summary.needs_repair
                && summary.next_mini_task.is_none()
                && summary.mini_task_count > 0
        })
        .min_by_key(|summary| {
            (
                summary.mini_task_count,
                field_curriculum_rank(&summary.field),
                summary.field.as_str(),
            )
        })?;
    let objective = field_harder_layer_objective_for(&summary.field);
    Some(FieldCurriculumStep {
        field: summary.field.clone(),
        next_action: field_harder_layer_next_action_for_field(state_args, &summary.field),
        reason: format!(
            "{} selected for the next harder layer by field curriculum: mini_tasks={}",
            summary.field, summary.mini_task_count
        ),
        objective,
    })
}

fn field_curriculum_rank(field: &str) -> usize {
    FIELD_CURRICULUM_ORDER
        .iter()
        .position(|candidate| *candidate == field)
        .unwrap_or(FIELD_CURRICULUM_ORDER.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Status;

    fn summary(field: &str, mini_task_count: usize) -> FieldTrajectorySummary {
        FieldTrajectorySummary {
            field: field.to_string(),
            mini_task_count,
            satisfied_mini_task_count: mini_task_count,
            next_mini_task: None,
            next_mini_task_goal: None,
            trace_count: 1,
            verifier_result_count: 1,
            satisfied_verifier_count: 1,
            unsatisfied_verifier_count: 0,
            latest_trace_index: Some(1),
            latest_trace_status: Some(Status::Satisfied),
            latest_verifier_result_index: Some(1),
            latest_verifier_status: Some(Status::Satisfied),
            latest_parallel_run_index: Some(1),
            latest_mini_task: Some(format!("{field}-mini-{mini_task_count}")),
            latest_error_category: None,
            latest_pass_evidence: Some("pass".to_string()),
            latest_summary: Some("satisfied".to_string()),
            needs_repair: false,
            ready_for_harder_task: true,
            next_action: "octopus fields summary".to_string(),
        }
    }

    #[test]
    fn curriculum_selects_lowest_layer_then_declared_order() {
        let summaries = vec![
            summary("math", 5),
            summary("search", 4),
            summary("write", 3),
            summary("translate", 3),
        ];

        let selected = select_next_harder_field(&summaries, " --state .octopus/state.json")
            .expect("curriculum step");

        assert_eq!(selected.field, "write");
        assert!(selected.objective.contains("harder write mini task layer"));
        assert!(selected.next_action.contains("field-packs/write"));
        assert!(selected.next_action.contains("Go worker"));
        assert!(selected.reason.contains("mini_tasks=3"));
    }
}
