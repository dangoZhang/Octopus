use octopus_core::{
    HarnessState, TentacleManifestReport, CLEAN_BRAIN_CONTEXT_POLICY, TENTACLE_CONTEXT_POLICY,
};
use serde::Serialize;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct StrategyDiagnosticReport {
    pub status: String,
    pub checks: Vec<StrategyDiagnosticCheck>,
    pub next: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct StrategyDiagnosticCheck {
    pub id: String,
    pub status: String,
    pub evidence: String,
    pub next: String,
}

pub(crate) fn strategy_diagnostics(
    state_path: &Path,
    manifests: &[TentacleManifestReport],
    state: &HarnessState,
) -> StrategyDiagnosticReport {
    let mut checks = vec![
        clean_brain_context_check(),
        user_surface_policy_check(state),
        tentacle_intelligence_check(manifests),
        editable_goal_driven_check(manifests, state),
        field_evolution_surface_check(manifests),
        desktop_pet_observer_check(state_path, state),
        evolution_feedback_chain_check(state),
    ];
    let status = if checks.iter().any(|check| check.status == "fail") {
        "fail"
    } else if checks.iter().any(|check| check.status == "warn") {
        "warn"
    } else {
        "pass"
    }
    .to_string();
    let next = checks
        .iter_mut()
        .filter(|check| check.status != "pass")
        .map(|check| check.next.clone())
        .collect::<Vec<_>>();
    StrategyDiagnosticReport {
        status,
        checks,
        next,
    }
}

fn user_surface_policy_check(state: &HarnessState) -> StrategyDiagnosticCheck {
    let status_report = state.status_report();
    let context = state.context_report(None, 2);
    let queue = state.need_queue_report(2);
    let mut leaked = Vec::new();
    if looks_internal_command(&status_report.next_action) {
        leaked.push("status.next_action".to_string());
    }
    if let Some(pool) = &status_report.field_pool {
        leaked.extend(
            pool.next
                .iter()
                .filter(|item| looks_internal_command(item))
                .map(|_| "field_pool.next".to_string()),
        );
        leaked.extend(
            pool.slots
                .iter()
                .filter(|slot| looks_internal_command(&slot.next_action))
                .map(|_| "field_pool.slots.next_action".to_string()),
        );
    }
    leaked.extend(
        context
            .next
            .iter()
            .filter(|item| looks_internal_command(item))
            .map(|_| "context.next".to_string()),
    );
    leaked.extend(
        queue
            .next
            .iter()
            .filter(|item| looks_internal_command(item))
            .map(|_| "need_queue.next".to_string()),
    );
    leaked.sort();
    leaked.dedup();
    StrategyDiagnosticCheck {
        id: "user_surface_policy".to_string(),
        status: status(leaked.is_empty()),
        evidence: if leaked.is_empty() {
            "user next fields contain Goal hints only".to_string()
        } else {
            format!("internal commands leaked through {}", join_or_none(&leaked))
        },
        next:
            "route user-visible next fields through user_surface.rs and keep commands in agent_next"
                .to_string(),
    }
}

fn looks_internal_command(value: &str) -> bool {
    let trimmed = value.trim_start();
    trimmed.starts_with("octopus ")
        || trimmed.starts_with("source ")
        || trimmed.starts_with("cargo ")
        || trimmed.starts_with("OCTOPUS_")
        || trimmed.contains("; octopus ")
}

fn clean_brain_context_check() -> StrategyDiagnosticCheck {
    let pass = CLEAN_BRAIN_CONTEXT_POLICY == "Goal + Mem + Need + Feed"
        && TENTACLE_CONTEXT_POLICY == "Need + Tool + Action + Tool + Action -> Feed";
    StrategyDiagnosticCheck {
        id: "clean_brain_context".to_string(),
        status: status(pass),
        evidence: format!(
            "brain={}; tentacle={}",
            CLEAN_BRAIN_CONTEXT_POLICY, TENTACLE_CONTEXT_POLICY
        ),
        next: "restore clean-brain and tentacle context policies before adding product surface"
            .to_string(),
    }
}

fn tentacle_intelligence_check(manifests: &[TentacleManifestReport]) -> StrategyDiagnosticCheck {
    let llm_tool_tentacles = manifests
        .iter()
        .filter(|manifest| manifest.brain_kind == "llm" && manifest.tool_count > 0)
        .map(|manifest| manifest.id.clone())
        .collect::<Vec<_>>();
    StrategyDiagnosticCheck {
        id: "tentacle_intelligence".to_string(),
        status: status(!llm_tool_tentacles.is_empty()),
        evidence: format!("llm_tool_tentacles={}", join_or_none(&llm_tool_tentacles)),
        next:
            "install or repair at least one LLM tentacle with tool metadata and executable harness"
                .to_string(),
    }
}

fn editable_goal_driven_check(
    manifests: &[TentacleManifestReport],
    state: &HarnessState,
) -> StrategyDiagnosticCheck {
    let editable_tentacles = manifests
        .iter()
        .filter(|manifest| !manifest.editable.is_empty() && !manifest.evolution_surfaces.is_empty())
        .map(|manifest| manifest.id.clone())
        .collect::<Vec<_>>();
    let has_goal = state.goal.is_some();
    let pass = has_goal && !editable_tentacles.is_empty();
    let check_status = if pass {
        "pass"
    } else if has_goal || !editable_tentacles.is_empty() {
        "warn"
    } else {
        "fail"
    };
    StrategyDiagnosticCheck {
        id: "editable_goal_driven".to_string(),
        status: check_status.to_string(),
        evidence: format!(
            "goal={}; editable_tentacles={}",
            if has_goal { "set" } else { "missing" },
            join_or_none(&editable_tentacles)
        ),
        next: "set a user goal and route evolution through editable harness surfaces, not direct brain commands"
            .to_string(),
    }
}

fn field_evolution_surface_check(manifests: &[TentacleManifestReport]) -> StrategyDiagnosticCheck {
    let field = manifests
        .iter()
        .find(|manifest| manifest.id == "field-mini-task");
    let pass = field
        .map(|manifest| {
            manifest
                .evolution_surfaces
                .contains(&"field_pack_tasks".to_string())
                && manifest
                    .evolution_surfaces
                    .contains(&"runtime_code".to_string())
                && manifest
                    .editable
                    .iter()
                    .any(|entry| entry.contains("repair-templates"))
        })
        .unwrap_or(false);
    StrategyDiagnosticCheck {
        id: "field_evolution_surface".to_string(),
        status: status(pass),
        evidence: field
            .map(|manifest| {
                format!(
                    "surfaces={}; editable={}",
                    join_or_none(&manifest.evolution_surfaces),
                    join_or_none(&manifest.editable)
                )
            })
            .unwrap_or_else(|| "field-mini-task manifest missing".to_string()),
        next: "restore field-mini-task manifest surfaces for field packs, runtime code, and repair templates"
            .to_string(),
    }
}

fn desktop_pet_observer_check(state_path: &Path, state: &HarnessState) -> StrategyDiagnosticCheck {
    let last = state.last_pet_event.as_ref();
    let fresh = last.map(pet_event_fresh).unwrap_or(false);
    let healthy_state = last
        .map(|event| {
            matches!(
                event.state.as_str(),
                "idle"
                    | "need"
                    | "action"
                    | "feed"
                    | "memory"
                    | "harness"
                    | "evolution"
                    | "blocked"
                    | "success"
            )
        })
        .unwrap_or(false);
    let check_status = if state_path.exists() && healthy_state && fresh {
        "pass"
    } else if state_path.exists() && last.is_some() {
        "warn"
    } else {
        "fail"
    };
    StrategyDiagnosticCheck {
        id: "desktop_pet_observer".to_string(),
        status: check_status.to_string(),
        evidence: last
            .map(|event| {
                format!(
                    "state_path={}; fresh={}; last={} {}/{}",
                    state_path.display(),
                    fresh,
                    event.state,
                    event.source,
                    event.summary
                )
            })
            .unwrap_or_else(|| format!("state_path={}; last=none", state_path.display())),
        next:
            "run a real goal/evolution step and confirm it records a live pet event in state.json"
                .to_string(),
    }
}

fn pet_event_fresh(event: &octopus_core::PetEvent) -> bool {
    const FRESH_SECONDS: u64 = 300;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    event.timestamp_secs > 0
        && event.timestamp_secs <= now
        && now.saturating_sub(event.timestamp_secs) <= FRESH_SECONDS
}

fn evolution_feedback_chain_check(state: &HarnessState) -> StrategyDiagnosticCheck {
    let has_trace = !state.feed_traces.is_empty();
    let has_parallel = !state.parallel_evolution_runs.is_empty();
    let has_outcome = !state.evolution_outcomes.is_empty();
    let has_check = !state.check_history.is_empty();
    let pass = has_trace && (has_parallel || has_outcome || has_check);
    let check_status = if pass {
        "pass"
    } else if has_trace || has_parallel || has_outcome || has_check {
        "warn"
    } else {
        "fail"
    };
    StrategyDiagnosticCheck {
        id: "evolution_feedback_chain".to_string(),
        status: check_status.to_string(),
        evidence: format!(
            "feed_traces={}; parallel_runs={}; outcomes={}; checks={}",
            state.feed_traces.len(),
            state.parallel_evolution_runs.len(),
            state.evolution_outcomes.len(),
            state.check_history.len()
        ),
        next: "drive a Need through a tentacle and let harness evolution consume the resulting trace/check data"
            .to_string(),
    }
}

fn status(pass: bool) -> String {
    if pass {
        "pass".to_string()
    } else {
        "fail".to_string()
    }
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octopus_core::{HarnessState, Status, TentacleManifestReport};
    use std::path::Path;

    #[test]
    fn strategy_diagnostics_reports_core_surface_failures() {
        let report = strategy_diagnostics(
            Path::new("missing-state.json"),
            &[],
            &HarnessState::default(),
        );
        assert_eq!(report.status, "fail");
        assert!(report
            .checks
            .iter()
            .any(|check| check.id == "tentacle_intelligence" && check.status == "fail"));
        assert!(report
            .checks
            .iter()
            .any(|check| check.id == "user_surface_policy" && check.status == "pass"));
    }

    #[test]
    fn strategy_diagnostics_passes_core_boundaries_with_live_evidence() {
        let mut state = HarnessState::default();
        state.goal = Some(octopus_core::Goal::new("evolve math field harness"));
        state.record_pet_event("evolution", "test", "planning", Status::Partial);
        state.feed_traces.push(octopus_core::FeedTraceRecord {
            index: 1,
            need_kind: octopus_core::NeedKind::Execute,
            need_query: "run mini task".to_string(),
            status: Status::Satisfied,
            field: Some("math".to_string()),
            tentacle: Some("field-mini-task".to_string()),
            tool: Some("run-mini-task".to_string()),
            plan_source: Some("llm".to_string()),
            route: Some("field-mini-task".to_string()),
            evidence_count: 1,
            summary: "ok".to_string(),
            metadata: Default::default(),
        });
        state
            .evolution_outcomes
            .push(octopus_core::EvolutionOutcome {
                index: 1,
                tentacle_id: "field-mini-task".to_string(),
                candidate_id: "field_pack_tasks".to_string(),
                status: Status::Satisfied,
                score: 6.0,
                summary: "ok".to_string(),
            });
        let manifests = vec![TentacleManifestReport {
            id: "field-mini-task".to_string(),
            name: "Field Mini Task".to_string(),
            path: "tentacles/field-mini-task/manifest.json".to_string(),
            brain_kind: "llm".to_string(),
            runtime_kinds: vec!["python".to_string()],
            needs: vec!["execute".to_string()],
            tool_count: 1,
            editable: vec![
                "field-packs/*.json".to_string(),
                "repair-templates/*/*.pyfrag".to_string(),
            ],
            evolution_surfaces: vec!["field_pack_tasks".to_string(), "runtime_code".to_string()],
            missing_entrypoints: Vec::new(),
        }];
        let report = strategy_diagnostics(Path::new("Cargo.toml"), &manifests, &state);
        assert_eq!(report.status, "pass");
    }
}
