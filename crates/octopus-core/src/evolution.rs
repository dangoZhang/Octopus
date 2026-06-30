use crate::{EvolutionPatchCandidate, EvolutionRequirement, TentacleEvolutionProposal};
use std::collections::BTreeMap;

pub(crate) fn required_surfaces_for_objective(proposal: &TentacleEvolutionProposal) -> Vec<String> {
    let mut surfaces = Vec::new();
    for requirement in &proposal.requirements {
        if requirement_matches_objective(requirement, &proposal.objective) {
            for surface in &requirement.required_surfaces {
                if !surfaces.contains(surface) {
                    surfaces.push(surface.clone());
                }
            }
        }
    }
    surfaces
}

pub(crate) fn validate_candidates_for_objective(
    proposal: &TentacleEvolutionProposal,
    candidates: &[EvolutionPatchCandidate],
) -> Result<(), String> {
    let required = required_surfaces_for_objective(proposal);
    let missing = required
        .iter()
        .filter(|surface| {
            !candidates
                .iter()
                .any(|candidate| &candidate.surface_id == *surface)
        })
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "evolution LLM missing required manifest surfaces: {}",
            serde_json::json!({
                "tentacle_id": proposal.tentacle_id,
                "objective": proposal.objective,
                "required_surfaces": required,
                "missing_surfaces": missing,
                "matched_requirements": matched_requirement_ids(proposal)
            })
        ));
    }
    Ok(())
}

pub(crate) fn uniquify_candidate_ids(candidates: &mut [EvolutionPatchCandidate]) {
    let mut seen = BTreeMap::<String, usize>::new();
    for candidate in candidates {
        let base = candidate.id.clone();
        let count = seen.entry(base.clone()).or_insert(0);
        *count += 1;
        if *count > 1 {
            candidate.id = format!("{base}-{count}");
            candidate.draft.path = format!("patches/{}.patch.md", candidate.id);
        }
    }
}

fn matched_requirement_ids(proposal: &TentacleEvolutionProposal) -> Vec<String> {
    proposal
        .requirements
        .iter()
        .filter(|requirement| requirement_matches_objective(requirement, &proposal.objective))
        .map(|requirement| requirement.id.clone())
        .collect()
}

fn requirement_matches_objective(requirement: &EvolutionRequirement, objective: &str) -> bool {
    if requirement.objective_markers_any.is_empty() {
        return false;
    }
    let lower = objective.to_ascii_lowercase();
    requirement
        .objective_markers_any
        .iter()
        .any(|marker| lower.contains(&marker.to_ascii_lowercase()))
}
