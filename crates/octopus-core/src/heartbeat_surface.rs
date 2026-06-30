use crate::llm_layers::{evolve_llm_enabled, evolve_llm_prefix};
use crate::pet_events;
use crate::provider_surface::provider_client;
use crate::{
    default_tentacles_root, localize_beat, manifest_missing_entrypoints,
    materialize_bundled_tentacles_root_in, repair_installed_seed_sources, seed_id,
    status_needs_harness_evolution, Language,
};
use octopus_core::{
    write_harness_beat_evolution_artifacts_with_client, HarnessBeatEvolution, HarnessState,
    HeartBeat, HeartbeatReport, Status,
};
use std::env;
use std::path::Path;

pub(crate) fn handle_beat_command(
    rest: &[String],
    state: &Path,
    json: bool,
    language: Language,
) -> Result<(), String> {
    let memory_keep = rest
        .get(1)
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| format!("invalid memory keep: {value}"))
        })
        .transpose()?
        .unwrap_or(200);
    let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
    loaded.record_pet_event("heartbeat", "heartbeat", "alive", Status::Satisfied);
    loaded.save(&state).map_err(|error| error.to_string())?;
    pet_events::append_latest(state, &loaded)?;
    repair_installed_seed_sources(&mut loaded)?;
    let mut report = loaded.beat(memory_keep);
    let cwd = env::current_dir().map_err(|error| error.to_string())?;
    let evolution = write_harness_beat_evolution_artifacts_with_bundled_manifest(&cwd, &loaded)?;
    if let Some(evolution) = &evolution {
        attach_harness_beat_evolution(&mut report, evolution);
        loaded.record_pet_event(
            "harness",
            "harness beat",
            format!(
                "recommended {} for {}",
                evolution.candidate_id, evolution.tentacle_id
            ),
            Status::Partial,
        );
    }
    loaded.save(&state).map_err(|error| error.to_string())?;
    pet_events::append_latest(state, &loaded)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
        );
    } else {
        print_heartbeat_report(&report, language);
    }
    Ok(())
}

pub(crate) fn attach_harness_beat_evolution(
    report: &mut HeartbeatReport,
    evolution: &HarnessBeatEvolution,
) {
    let Some(beat) = report.beats.iter_mut().find(|beat| beat.name == "harness") else {
        return;
    };
    beat.changed = true;
    beat.summary = format!(
        "{}; recommended {} for {}",
        beat.summary, evolution.candidate_id, evolution.tentacle_id
    );
    beat.data.insert(
        "evolution_source_check".to_string(),
        evolution.source_check_index.to_string(),
    );
    beat.data.insert(
        "evolution_source_kind".to_string(),
        evolution.source_kind.clone(),
    );
    beat.data.insert(
        "evolution_source_index".to_string(),
        evolution.source_index.to_string(),
    );
    beat.data.insert(
        "evolution_source_summary".to_string(),
        evolution.source_summary.clone(),
    );
    beat.data.insert(
        "evolution_tentacle".to_string(),
        evolution.tentacle_id.clone(),
    );
    beat.data.insert(
        "evolution_candidate".to_string(),
        evolution.candidate_id.clone(),
    );
    beat.data.insert(
        "evolution_score_target".to_string(),
        format!("{} {}", evolution.tentacle_id, evolution.candidate_id),
    );
    beat.data
        .insert("evolution_status".to_string(), evolution.status.clone());
    beat.data
        .insert("evolution_reason".to_string(), evolution.reason.clone());
    beat.data.insert(
        "evolution_plan".to_string(),
        evolution.apply_plan_path.clone(),
    );
    beat.data.insert(
        "evolution_plan_preview".to_string(),
        evolution.apply_plan_preview.clone(),
    );
    beat.data
        .insert("evolution_next".to_string(), evolution.next_action.clone());
    if let Some(patch) = &evolution.patch_path {
        beat.data
            .insert("evolution_patch".to_string(), patch.clone());
    }
}

pub(crate) fn print_heartbeat_report(report: &HeartbeatReport, language: Language) {
    for beat in &report.beats {
        match language {
            Language::En => println!("{}: {}", beat.name, beat.summary),
            Language::Zh => println!("{}: {}", localize_beat(&beat.name), beat.summary),
        }
        if beat.name == "harness" {
            print_harness_beat_evolution(beat, language);
        }
    }
}

pub(crate) fn print_harness_beat_evolution(beat: &HeartBeat, language: Language) {
    let Some(candidate) = beat.data.get("evolution_candidate") else {
        return;
    };
    let tentacle = beat
        .data
        .get("evolution_tentacle")
        .map(String::as_str)
        .unwrap_or("unknown");
    let status = beat
        .data
        .get("evolution_status")
        .map(String::as_str)
        .unwrap_or("unknown");
    let next = beat
        .data
        .get("evolution_next")
        .map(String::as_str)
        .unwrap_or("review evolution plan");
    let plan = beat
        .data
        .get("evolution_plan")
        .map(String::as_str)
        .unwrap_or("none");
    let source_kind = beat
        .data
        .get("evolution_source_kind")
        .map(String::as_str)
        .unwrap_or("check_history");
    let source_index = beat
        .data
        .get("evolution_source_index")
        .map(String::as_str)
        .unwrap_or("unknown");
    match language {
        Language::En => {
            println!("  recommendation: {tentacle} {candidate} ({status})");
            println!("  source: {source_kind} #{source_index}");
            println!("  plan: {plan}");
            println!("  next: {next}");
        }
        Language::Zh => {
            println!("  推荐: {tentacle} {candidate} ({status})");
            println!("  来源: {source_kind} #{source_index}");
            println!("  计划: {plan}");
            println!("  下一步: {next}");
        }
    }
}

pub(crate) fn write_harness_beat_evolution_artifacts_with_bundled_manifest(
    cwd: &Path,
    state: &HarnessState,
) -> Result<Option<HarnessBeatEvolution>, String> {
    if !evolve_llm_enabled() {
        return Ok(None);
    }
    let (_, _, _, _, mut client) = provider_client(&evolve_llm_prefix())?;
    let root = default_tentacles_root();
    let broken_local_seed =
        harness_beat_next_tentacle_id(state)
            .as_deref()
            .is_some_and(|tentacle_id| {
                seed_id(tentacle_id)
                    && manifest_missing_entrypoints(&root, tentacle_id)
                        .map(|missing| !missing.is_empty())
                        .unwrap_or(false)
            });
    if !broken_local_seed {
        if let Some(evolution) =
            write_harness_beat_evolution_artifacts_with_client(&root, cwd, state, &mut client)?
        {
            return Ok(Some(evolution));
        }
    }
    let bundled_root = materialize_bundled_tentacles_root_in(cwd)?;
    if bundled_root == root {
        return Ok(None);
    }
    write_harness_beat_evolution_artifacts_with_client(bundled_root, cwd, state, &mut client)
}

fn harness_beat_next_tentacle_id(state: &HarnessState) -> Option<String> {
    state
        .check_history
        .iter()
        .rev()
        .find(|record| status_needs_harness_evolution(&record.status))
        .map(|record| record.tentacle_id.clone())
        .or_else(|| {
            state
                .feed_traces
                .iter()
                .rev()
                .find(|trace| status_needs_harness_evolution(&trace.status))
                .and_then(|trace| trace.tentacle.clone())
        })
        .or_else(|| {
            state
                .repair_outcomes
                .iter()
                .rev()
                .find(|outcome| status_needs_harness_evolution(&outcome.status))
                .map(|outcome| {
                    outcome
                        .target_tentacle
                        .clone()
                        .unwrap_or_else(|| outcome.tentacle_id.clone())
                })
        })
}
