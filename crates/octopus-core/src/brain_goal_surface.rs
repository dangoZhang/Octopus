use super::*;

pub(crate) fn handle_brain_command(
    rest: &[String],
    state: &Path,
    json: bool,
    language: Language,
) -> Result<(), String> {
    let mut live = false;
    let mut save = false;
    let mut refine_goal = false;
    let mut session = false;
    let mut rewrite = false;
    let mut deliberate = false;
    let mut synthesize = false;
    let mut council = false;
    let mut reflect = false;
    let mut align = false;
    let mut memory = false;
    let mut clarify = false;
    let mut agenda = false;
    let mut scout = false;
    let mut brief = false;
    let mut intent = false;
    let mut focus_kind: Option<NeedKind> = None;
    let mut apply_path = None;
    let mut apply_json = None;
    let mut llm_prefix_override = None;
    let mut council_prefixes_override = None;
    let mut prompt = Vec::new();
    let mut brain_index = 1;
    while brain_index < rest.len() {
        match rest[brain_index].as_str() {
            "--live" => live = true,
            "--save" => save = true,
            "--goal" => refine_goal = true,
            "--session" => session = true,
            "--rewrite" => rewrite = true,
            "--deliberate" => deliberate = true,
            "--synthesize" => synthesize = true,
            "--council" => council = true,
            "--reflect" => reflect = true,
            "--align" => align = true,
            "--memory" => memory = true,
            "--clarify" => clarify = true,
            "--agenda" => agenda = true,
            "--scout" => scout = true,
            "--brief" => brief = true,
            "--intent" => intent = true,
            "--focus" => {
                brain_index += 1;
                let Some(kind) = rest.get(brain_index) else {
                    return Err("brain --focus requires a Need kind".to_string());
                };
                focus_kind = Some(parse_kind(kind)?);
            }
            "--llm-prefix" | "--provider-prefix" => {
                brain_index += 1;
                let Some(prefix) = rest.get(brain_index) else {
                    return Err("brain --llm-prefix requires an env prefix".to_string());
                };
                llm_prefix_override = Some(parse_brain_llm_prefix(prefix)?);
            }
            "--models" => {
                brain_index += 1;
                let Some(prefixes) = rest.get(brain_index) else {
                    return Err("brain --models requires comma-separated env prefixes".to_string());
                };
                council_prefixes_override = Some(parse_brain_llm_prefixes(prefixes)?);
            }
            "--apply" => {
                brain_index += 1;
                let Some(path) = rest.get(brain_index) else {
                    return Err("brain --apply requires a path or -".to_string());
                };
                apply_path = Some(path.clone());
            }
            "--apply-json" => {
                brain_index += 1;
                let Some(payload) = rest.get(brain_index) else {
                    return Err("brain --apply-json requires JSON".to_string());
                };
                apply_json = Some(payload.clone());
            }
            value => prompt.push(value.to_string()),
        }
        brain_index += 1;
    }
    let has_apply_payload = apply_json.is_some() || apply_path.is_some();
    if rewrite && refine_goal {
        return Err("brain --rewrite cannot be combined with --goal".to_string());
    }
    if deliberate && refine_goal {
        return Err("brain --deliberate cannot be combined with --goal".to_string());
    }
    if deliberate && rewrite {
        return Err("brain --deliberate cannot be combined with --rewrite".to_string());
    }
    if synthesize && refine_goal {
        return Err("brain --synthesize cannot be combined with --goal".to_string());
    }
    if synthesize && rewrite {
        return Err("brain --synthesize cannot be combined with --rewrite".to_string());
    }
    if synthesize && deliberate {
        return Err("brain --synthesize cannot be combined with --deliberate".to_string());
    }
    if council && refine_goal {
        return Err("brain --council cannot be combined with --goal".to_string());
    }
    if council && rewrite {
        return Err("brain --council cannot be combined with --rewrite".to_string());
    }
    if council && deliberate {
        return Err("brain --council cannot be combined with --deliberate".to_string());
    }
    if council && synthesize {
        return Err("brain --council cannot be combined with --synthesize".to_string());
    }
    if council && session {
        return Err("brain --council writes a direct council report, not a session".to_string());
    }
    if council && has_apply_payload {
        return Err("brain --council collects live drafts and does not accept --apply".to_string());
    }
    if reflect && refine_goal {
        return Err("brain --reflect cannot be combined with --goal".to_string());
    }
    if reflect && rewrite {
        return Err("brain --reflect cannot be combined with --rewrite".to_string());
    }
    if reflect && deliberate {
        return Err("brain --reflect cannot be combined with --deliberate".to_string());
    }
    if reflect && synthesize {
        return Err("brain --reflect cannot be combined with --synthesize".to_string());
    }
    if reflect && council {
        return Err("brain --reflect cannot be combined with --council".to_string());
    }
    if align
        && (refine_goal
            || rewrite
            || deliberate
            || synthesize
            || council
            || reflect
            || memory
            || clarify
            || agenda
            || scout
            || brief
            || intent
            || focus_kind.is_some())
    {
        return Err("brain --align cannot be combined with another brain mode".to_string());
    }
    if memory && refine_goal {
        return Err("brain --memory cannot be combined with --goal".to_string());
    }
    if memory && rewrite {
        return Err("brain --memory cannot be combined with --rewrite".to_string());
    }
    if memory && deliberate {
        return Err("brain --memory cannot be combined with --deliberate".to_string());
    }
    if memory && synthesize {
        return Err("brain --memory cannot be combined with --synthesize".to_string());
    }
    if memory && council {
        return Err("brain --memory cannot be combined with --council".to_string());
    }
    if memory && reflect {
        return Err("brain --memory cannot be combined with --reflect".to_string());
    }
    if clarify && refine_goal {
        return Err("brain --clarify cannot be combined with --goal".to_string());
    }
    if clarify && rewrite {
        return Err("brain --clarify cannot be combined with --rewrite".to_string());
    }
    if clarify && deliberate {
        return Err("brain --clarify cannot be combined with --deliberate".to_string());
    }
    if clarify && synthesize {
        return Err("brain --clarify cannot be combined with --synthesize".to_string());
    }
    if clarify && council {
        return Err("brain --clarify cannot be combined with --council".to_string());
    }
    if clarify && reflect {
        return Err("brain --clarify cannot be combined with --reflect".to_string());
    }
    if clarify && memory {
        return Err("brain --clarify cannot be combined with --memory".to_string());
    }
    if agenda && refine_goal {
        return Err("brain --agenda cannot be combined with --goal".to_string());
    }
    if agenda && rewrite {
        return Err("brain --agenda cannot be combined with --rewrite".to_string());
    }
    if agenda && deliberate {
        return Err("brain --agenda cannot be combined with --deliberate".to_string());
    }
    if agenda && synthesize {
        return Err("brain --agenda cannot be combined with --synthesize".to_string());
    }
    if agenda && council {
        return Err("brain --agenda cannot be combined with --council".to_string());
    }
    if agenda && reflect {
        return Err("brain --agenda cannot be combined with --reflect".to_string());
    }
    if agenda && memory {
        return Err("brain --agenda cannot be combined with --memory".to_string());
    }
    if agenda && clarify {
        return Err("brain --agenda cannot be combined with --clarify".to_string());
    }
    if scout
        && (refine_goal
            || rewrite
            || deliberate
            || synthesize
            || council
            || reflect
            || align
            || memory
            || clarify
            || agenda
            || brief
            || intent
            || focus_kind.is_some())
    {
        return Err("brain --scout cannot be combined with another brain mode".to_string());
    }
    if brief
        && (refine_goal
            || rewrite
            || deliberate
            || synthesize
            || council
            || reflect
            || align
            || memory
            || clarify
            || agenda
            || scout
            || intent
            || focus_kind.is_some())
    {
        return Err("brain --brief cannot be combined with another brain mode".to_string());
    }
    if intent
        && (refine_goal
            || rewrite
            || deliberate
            || synthesize
            || council
            || reflect
            || align
            || memory
            || clarify
            || agenda
            || scout
            || brief
            || focus_kind.is_some())
    {
        return Err("brain --intent cannot be combined with another brain mode".to_string());
    }
    if let Some(kind) = &focus_kind {
        let label = need_label(kind);
        if refine_goal
            || rewrite
            || deliberate
            || synthesize
            || council
            || reflect
            || align
            || memory
            || clarify
            || agenda
            || scout
            || brief
        {
            return Err(format!(
                "brain --focus {label} cannot be combined with another brain mode"
            ));
        }
    }
    if council_prefixes_override.is_some() && !council {
        return Err("brain --models requires --council".to_string());
    }
    if rewrite && !has_apply_payload {
        return Err("brain --rewrite requires --apply or --apply-json".to_string());
    }
    if synthesize && !session && !has_apply_payload {
        return Err("brain --synthesize requires --apply or --apply-json".to_string());
    }
    if llm_prefix_override.is_some() {
        live = true;
    }
    let _brain_prefix_guard = llm_prefix_override
        .as_deref()
        .map(BrainLlmPrefixOverride::apply)
        .transpose()?;
    let prompt = (!prompt.is_empty()).then(|| prompt.join(" "));
    let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
    let prompt = prompt
        .or_else(|| loaded.goal.as_ref().map(|goal| goal.objective.clone()))
        .unwrap_or_else(|| "what should the brain ask next?".to_string());
    let apply_payload = if let Some(payload) = apply_json {
        Some(payload)
    } else if let Some(path) = apply_path {
        Some(read_brain_apply_payload(&path)?)
    } else {
        None
    };
    if council {
        let report =
            run_brain_council(&loaded, &prompt, live, council_prefixes_override.as_deref())?;
        if save {
            let saved = save_brain_council(&mut loaded, report);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_council_save(&saved, language);
            }
        } else if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_brain_council(&report, language);
        }
    } else if session {
        let report = if rewrite {
            let payload = apply_payload
                .as_deref()
                .ok_or_else(|| "brain --rewrite --session requires a reply payload".to_string())?;
            let draft = parse_brain_reply::<BrainExploreDraft>(payload, "clean-brain rewrite")?;
            write_brain_rewrite_session(&loaded, &state, &prompt, draft, live)?
        } else if synthesize {
            let payload = apply_payload.as_deref().ok_or_else(|| {
                "brain --synthesize --session requires a drafts payload".to_string()
            })?;
            let input = parse_brain_reply::<BrainSynthesisInput>(payload, "clean-brain synthesis")?;
            write_brain_synthesis_session(&loaded, &state, &prompt, input, live)?
        } else if clarify {
            write_brain_clarification_session(&loaded, &state, &prompt, live)?
        } else if agenda {
            write_brain_agenda_session(&loaded, &state, &prompt, live)?
        } else if scout {
            write_brain_scout_session(&loaded, &state, &prompt, live)?
        } else if reflect {
            write_brain_reflection_session(&loaded, &state, &prompt, live)?
        } else if align {
            write_brain_alignment_session(&loaded, &state, &prompt, live)?
        } else if memory {
            write_brain_memory_session(&loaded, &state, &prompt, live)?
        } else if brief {
            write_brain_brief_session(&loaded, &state, &prompt, live)?
        } else if intent {
            write_brain_intent_session(&loaded, &state, &prompt, live)?
        } else if let Some(kind) = focus_kind.clone() {
            write_brain_focus_session(&loaded, &state, &prompt, kind, live)?
        } else if deliberate {
            write_brain_deliberation_session(&loaded, &state, &prompt, live)?
        } else {
            write_brain_session(&loaded, &state, &prompt, refine_goal, live)?
        };
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_brain_session(&report, language);
        }
    } else if let Some(payload) = apply_payload {
        if refine_goal {
            let refinement = parse_brain_reply::<GoalRefinement>(&payload, "clean-brain goal")?;
            let report = loaded.clean_brain_goal_from_refinement(prompt.clone(), 6, refinement);
            if save {
                let saved = loaded.queue_goal_report(&report);
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                    );
                } else {
                    print_brain_goal_save(&saved, language);
                }
            } else {
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                    );
                } else {
                    print_brain_goal(&report, language);
                }
            }
        } else if synthesize {
            let input =
                parse_brain_reply::<BrainSynthesisInput>(&payload, "clean-brain synthesis")?;
            let report = if live {
                let mut client = clean_brain_synthesize_llm_client()?;
                loaded.clean_brain_synthesize_with_client(prompt.clone(), 6, input, &mut client)?
            } else {
                loaded.clean_brain_synthesize_from_input(prompt.clone(), 6, input)
            };
            if save {
                let saved = loaded.queue_synthesis_report(&report);
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                    );
                } else {
                    print_brain_synthesis_save(&saved, language);
                }
            } else if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_synthesis(&report, language);
            }
        } else if reflect {
            let draft =
                parse_brain_reply::<BrainReflectionDraft>(&payload, "clean-brain reflection")?;
            let report = loaded.clean_brain_reflect_from_draft(prompt.clone(), 6, draft);
            if save {
                let saved = loaded.queue_reflection_report(&report);
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                    );
                } else {
                    print_brain_reflection_save(&saved, language);
                }
            } else if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_reflection(&report, language);
            }
        } else if align {
            let draft =
                parse_brain_reply::<BrainReflectionDraft>(&payload, "clean-brain alignment")?;
            let report = loaded.clean_brain_align_from_draft(prompt.clone(), 6, draft);
            if save {
                let saved = loaded.queue_reflection_report(&report);
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                    );
                } else {
                    print_brain_reflection_save(&saved, language);
                }
            } else if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_reflection(&report, language);
            }
        } else if memory {
            let draft = parse_brain_reply::<BrainExploreDraft>(&payload, "clean-brain memory")?;
            let report = loaded.clean_brain_memory_from_draft(prompt.clone(), 6, draft);
            if save {
                let saved = loaded.queue_exploration_report(&report);
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                    );
                } else {
                    print_brain_memory_save(&saved, language);
                }
            } else if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_memory(&report, language);
            }
        } else if brief {
            let draft = parse_brain_reply::<BrainExploreDraft>(&payload, "clean-brain brief")?;
            let report = loaded.clean_brain_brief_from_draft(prompt.clone(), 6, draft);
            if save {
                let saved = loaded.queue_exploration_report(&report);
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                    );
                } else {
                    print_need_queue_save(&saved, language);
                }
            } else if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_explore(&report, language);
            }
        } else if intent {
            let draft = parse_brain_reply::<BrainExploreDraft>(&payload, "clean-brain intent")?;
            let report = loaded.clean_brain_intent_from_draft(prompt.clone(), 6, draft);
            if save {
                let saved = loaded.queue_exploration_report(&report);
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                    );
                } else {
                    print_need_queue_save(&saved, language);
                }
            } else if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_explore(&report, language);
            }
        } else if let Some(kind) = focus_kind.clone() {
            let draft = parse_brain_reply::<BrainExploreDraft>(&payload, "clean-brain focus")?;
            let report = loaded.clean_brain_focus_from_draft(kind, prompt.clone(), 6, draft);
            if save {
                let saved = loaded.queue_exploration_report(&report);
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                    );
                } else {
                    print_need_queue_save(&saved, language);
                }
            } else if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_explore(&report, language);
            }
        } else if clarify {
            let draft =
                parse_brain_reply::<BrainDeliberationDraft>(&payload, "clean-brain clarification")?;
            let report = loaded.clean_brain_clarify_from_draft(prompt.clone(), 6, draft);
            if save {
                let saved = loaded.queue_deliberation_report(&report);
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                    );
                } else {
                    print_brain_clarification_save(&saved, language);
                }
            } else if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_clarification(&report, language);
            }
        } else if agenda {
            let draft =
                parse_brain_reply::<BrainDeliberationDraft>(&payload, "clean-brain agenda")?;
            let report = loaded.clean_brain_agenda_from_draft(prompt.clone(), 6, draft);
            if save {
                let saved = loaded.queue_deliberation_report(&report);
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                    );
                } else {
                    print_brain_agenda_save(&saved, language);
                }
            } else if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_agenda(&report, language);
            }
        } else if scout {
            let draft = parse_brain_reply::<BrainDeliberationDraft>(&payload, "clean-brain scout")?;
            let report = loaded.clean_brain_scout_from_draft(prompt.clone(), 6, draft);
            if save {
                let saved = loaded.queue_deliberation_report(&report);
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                    );
                } else {
                    print_brain_deliberation_save(&saved, language);
                }
            } else if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_deliberation(&report, language);
            }
        } else if deliberate {
            let draft =
                parse_brain_reply::<BrainDeliberationDraft>(&payload, "clean-brain deliberation")?;
            let report = loaded.clean_brain_deliberate_from_draft(prompt.clone(), 6, draft);
            if save {
                let saved = loaded.queue_deliberation_report(&report);
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                    );
                } else {
                    print_brain_deliberation_save(&saved, language);
                }
            } else if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_deliberation(&report, language);
            }
        } else {
            let draft = parse_brain_reply::<BrainExploreDraft>(&payload, "clean-brain explore")?;
            let report = if rewrite {
                if live || clean_brain_llm_enabled() {
                    let mut client = clean_brain_rewrite_llm_client()?;
                    loaded.clean_brain_rewrite_with_client(prompt.clone(), 6, draft, &mut client)?
                } else {
                    loaded.clean_brain_rewrite_from_draft(prompt.clone(), 6, draft)
                }
            } else {
                loaded.clean_brain_explore_from_draft(prompt.clone(), 6, draft)
            };
            if save {
                let saved = loaded.queue_exploration_report(&report);
                loaded.save(&state).map_err(|error| error.to_string())?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                    );
                } else {
                    print_need_queue_save(&saved, language);
                }
            } else if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_explore(&report, language);
            }
        }
    } else if reflect {
        let mut client = clean_brain_reflect_llm_client()?;
        let report = loaded.clean_brain_reflect_with_client(prompt.clone(), 6, &mut client)?;
        if save {
            let saved = loaded.queue_reflection_report(&report);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_reflection_save(&saved, language);
            }
        } else if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_brain_reflection(&report, language);
        }
    } else if align {
        let mut client = clean_brain_align_llm_client()?;
        let report = loaded.clean_brain_align_with_client(prompt.clone(), 6, &mut client)?;
        if save {
            let saved = loaded.queue_reflection_report(&report);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_reflection_save(&saved, language);
            }
        } else if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_brain_reflection(&report, language);
        }
    } else if memory {
        let mut client = clean_brain_memory_llm_client()?;
        let report = loaded.clean_brain_memory_with_client(prompt.clone(), 6, &mut client)?;
        if save {
            let saved = loaded.queue_exploration_report(&report);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_memory_save(&saved, language);
            }
        } else if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_brain_memory(&report, language);
        }
    } else if brief {
        let mut client = clean_brain_brief_llm_client()?;
        let report = loaded.clean_brain_brief_with_client(prompt.clone(), 6, &mut client)?;
        if save {
            let saved = loaded.queue_exploration_report(&report);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                );
            } else {
                print_need_queue_save(&saved, language);
            }
        } else if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_brain_explore(&report, language);
        }
    } else if intent {
        let mut client = clean_brain_intent_llm_client()?;
        let report = loaded.clean_brain_intent_with_client(prompt.clone(), 6, &mut client)?;
        if save {
            let saved = loaded.queue_exploration_report(&report);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                );
            } else {
                print_need_queue_save(&saved, language);
            }
        } else if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_brain_explore(&report, language);
        }
    } else if let Some(kind) = focus_kind {
        let mut client = clean_brain_explore_llm_client()?;
        let report = loaded.clean_brain_focus_with_client(kind, prompt.clone(), 6, &mut client)?;
        if save {
            let saved = loaded.queue_exploration_report(&report);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                );
            } else {
                print_need_queue_save(&saved, language);
            }
        } else if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_brain_explore(&report, language);
        }
    } else if clarify {
        let mut client = clean_brain_clarify_llm_client()?;
        let report = loaded.clean_brain_clarify_with_client(prompt.clone(), 6, &mut client)?;
        if save {
            let saved = loaded.queue_deliberation_report(&report);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_clarification_save(&saved, language);
            }
        } else if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_brain_clarification(&report, language);
        }
    } else if agenda {
        let mut client = clean_brain_agenda_llm_client()?;
        let report = loaded.clean_brain_agenda_with_client(prompt.clone(), 6, &mut client)?;
        if save {
            let saved = loaded.queue_deliberation_report(&report);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_agenda_save(&saved, language);
            }
        } else if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_brain_agenda(&report, language);
        }
    } else if scout {
        let mut client = clean_brain_scout_llm_client()?;
        let report = loaded.clean_brain_scout_with_client(prompt.clone(), 6, &mut client)?;
        if save {
            let saved = loaded.queue_deliberation_report(&report);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_deliberation_save(&saved, language);
            }
        } else if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_brain_deliberation(&report, language);
        }
    } else if deliberate {
        let mut client = clean_brain_deliberate_llm_client()?;
        let report = loaded.clean_brain_deliberate_with_client(prompt.clone(), 6, &mut client)?;
        if save {
            let saved = loaded.queue_deliberation_report(&report);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_deliberation_save(&saved, language);
            }
        } else if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_brain_deliberation(&report, language);
        }
    } else if refine_goal {
        let mut client = clean_brain_goal_llm_client()?;
        let report = loaded.clean_brain_goal_with_client(prompt.clone(), 6, &mut client)?;
        if save {
            let saved = loaded.queue_goal_report(&report);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_goal_save(&saved, language);
            }
        } else {
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_brain_goal(&report, language);
            }
        }
    } else if live || save {
        let mut client = clean_brain_explore_llm_client()?;
        let report = loaded.clean_brain_explore_with_client(prompt, 6, &mut client)?;
        if save {
            let saved = loaded.queue_exploration_report(&report);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?
                );
            } else {
                print_need_queue_save(&saved, language);
            }
        } else if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_brain_explore(&report, language);
        }
    } else {
        let report = loaded.clean_brain_prompt(prompt, 6);
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        } else {
            print_brain_prompt(&report, language);
        }
    }
    Ok(())
}

pub(crate) fn handle_goal_command(
    rest: &[String],
    state: &Path,
    json: bool,
    language: Language,
) -> Result<(), String> {
    let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
    match rest.get(1).map(String::as_str) {
        Some("set") => {
            let (objective, constraints) = parse_goal_set_args(&rest[2..])?;
            let mut goal = Goal::new(objective);
            for constraint in constraints {
                add_goal_constraint(&mut goal, constraint);
            }
            loaded.goal = Some(goal);
            loaded.save(&state).map_err(|error| error.to_string())?;
        }
        Some("refine" | "constraint") => {
            let constraint = clean_goal_constraint(
                &rest
                    .get(2..)
                    .filter(|values| !values.is_empty())
                    .map(|values| values.join(" "))
                    .ok_or_else(|| "goal refine requires a constraint".to_string())?,
            )?;
            let goal = loaded
                .goal
                .as_mut()
                .ok_or_else(|| "goal refine requires an active goal".to_string())?;
            add_goal_constraint(goal, constraint);
            loaded.save(&state).map_err(|error| error.to_string())?;
        }
        _ => {}
    }
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&loaded.goal).map_err(|error| error.to_string())?
        );
    } else {
        print_goal(&loaded, language);
    }
    Ok(())
}

pub(crate) fn print_brain_explore(report: &BrainExploreReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus explore");
            println!("source: {}", report.source);
            println!("brain: {}", report.policy);
            println!(
                "goal: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("none")
            );
            println!("summary: {}", report.summary);
            println!("audit: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("next: {next}");
            }
        }
        Language::Zh => {
            println!("章鱼探索");
            println!("来源: {}", report.source);
            println!("主脑: {}", report.policy);
            println!(
                "目标: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("无")
            );
            println!("摘要: {}", report.summary);
            println!("审计: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("Need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("下一步: {next}");
            }
        }
    }
}

fn print_brain_memory_save(report: &NeedQueueSaveReport, language: Language) {
    print_brain_memory(&report.explore, language);
    match language {
        Language::En => println!("queued: {}", report.queued.len()),
        Language::Zh => println!("已入队: {}", report.queued.len()),
    }
    print_need_queue(&report.queue, language);
}

fn print_brain_memory(report: &BrainExploreReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus memory");
            println!("source: {}", report.source);
            println!("brain: {}", report.policy);
            println!(
                "goal: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("none")
            );
            println!("summary: {}", report.summary);
            println!("audit: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("next: {next}");
            }
        }
        Language::Zh => {
            println!("章鱼记忆");
            println!("来源: {}", report.source);
            println!("主脑: {}", report.policy);
            println!(
                "目标: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("无")
            );
            println!("摘要: {}", report.summary);
            println!("审计: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("Need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("下一步: {next}");
            }
        }
    }
}

fn print_brain_deliberation_save(report: &BrainDeliberationSaveReport, language: Language) {
    print_brain_deliberation(&report.deliberation, language);
    match language {
        Language::En => println!("queued: {}", report.queued.len()),
        Language::Zh => println!("已入队: {}", report.queued.len()),
    }
    print_need_queue(&report.queue, language);
}

fn print_brain_deliberation(report: &BrainDeliberationReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus deliberate");
            println!("source: {}", report.source);
            println!("brain: {}", report.policy);
            println!(
                "goal: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("none")
            );
            println!("summary: {}", report.summary);
            print_list("observation", &report.observations);
            print_list("question", &report.questions);
            print_list("option", &report.options);
            print_list("risk", &report.risks);
            println!("audit: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("next: {next}");
            }
        }
        Language::Zh => {
            println!("章鱼深思");
            println!("来源: {}", report.source);
            println!("主脑: {}", report.policy);
            println!(
                "目标: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("无")
            );
            println!("摘要: {}", report.summary);
            print_list("观察", &report.observations);
            print_list("问题", &report.questions);
            print_list("选项", &report.options);
            print_list("风险", &report.risks);
            println!("审计: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("Need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("下一步: {next}");
            }
        }
    }
}

fn print_brain_clarification_save(report: &BrainDeliberationSaveReport, language: Language) {
    print_brain_clarification(&report.deliberation, language);
    match language {
        Language::En => println!("queued: {}", report.queued.len()),
        Language::Zh => println!("已入队: {}", report.queued.len()),
    }
    print_need_queue(&report.queue, language);
}

fn print_brain_clarification(report: &BrainDeliberationReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus clarify");
            println!("source: {}", report.source);
            println!("brain: {}", report.policy);
            println!(
                "goal: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("none")
            );
            println!("summary: {}", report.summary);
            print_list("observation", &report.observations);
            print_list("question", &report.questions);
            print_list("option", &report.options);
            print_list("risk", &report.risks);
            println!("audit: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("next: {next}");
            }
        }
        Language::Zh => {
            println!("章鱼澄清");
            println!("来源: {}", report.source);
            println!("主脑: {}", report.policy);
            println!(
                "目标: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("无")
            );
            println!("摘要: {}", report.summary);
            print_list("观察", &report.observations);
            print_list("问题", &report.questions);
            print_list("选项", &report.options);
            print_list("风险", &report.risks);
            println!("审计: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("Need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("下一步: {next}");
            }
        }
    }
}

fn print_brain_agenda_save(report: &BrainDeliberationSaveReport, language: Language) {
    print_brain_agenda(&report.deliberation, language);
    match language {
        Language::En => println!("queued: {}", report.queued.len()),
        Language::Zh => println!("已入队: {}", report.queued.len()),
    }
    print_need_queue(&report.queue, language);
}

fn print_brain_agenda(report: &BrainDeliberationReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus agenda");
            println!("source: {}", report.source);
            println!("brain: {}", report.policy);
            println!(
                "goal: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("none")
            );
            println!("summary: {}", report.summary);
            print_list("observation", &report.observations);
            print_list("question", &report.questions);
            print_list("option", &report.options);
            print_list("risk", &report.risks);
            println!("audit: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("next: {next}");
            }
        }
        Language::Zh => {
            println!("章鱼议程");
            println!("来源: {}", report.source);
            println!("主脑: {}", report.policy);
            println!(
                "目标: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("无")
            );
            println!("摘要: {}", report.summary);
            print_list("观察", &report.observations);
            print_list("问题", &report.questions);
            print_list("选项", &report.options);
            print_list("风险", &report.risks);
            println!("审计: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("Need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("下一步: {next}");
            }
        }
    }
}

fn print_brain_reflection_save(report: &BrainReflectionSaveReport, language: Language) {
    print_brain_reflection(&report.reflection, language);
    match language {
        Language::En => println!("queued: {}", report.queued.len()),
        Language::Zh => println!("已入队: {}", report.queued.len()),
    }
    print_need_queue(&report.queue, language);
}

fn print_brain_reflection(report: &BrainReflectionReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus reflect");
            println!("source: {}", report.source);
            println!("brain: {}", report.policy);
            println!("goal_state: {}", report.goal_state);
            println!(
                "goal: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("none")
            );
            println!("summary: {}", report.summary);
            print_list("evidence", &report.evidence);
            print_list("gap", &report.gaps);
            print_list("question", &report.questions);
            println!("audit: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("next: {next}");
            }
        }
        Language::Zh => {
            println!("章鱼反思");
            println!("来源: {}", report.source);
            println!("主脑: {}", report.policy);
            println!("目标状态: {}", report.goal_state);
            println!(
                "目标: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("无")
            );
            println!("摘要: {}", report.summary);
            print_list("证据", &report.evidence);
            print_list("缺口", &report.gaps);
            print_list("问题", &report.questions);
            println!("审计: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("Need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("下一步: {next}");
            }
        }
    }
}

fn print_brain_synthesis_save(report: &BrainSynthesisSaveReport, language: Language) {
    print_brain_synthesis(&report.synthesis, language);
    match language {
        Language::En => println!("queued: {}", report.queued.len()),
        Language::Zh => println!("已入队: {}", report.queued.len()),
    }
    print_need_queue(&report.queue, language);
}

fn print_brain_synthesis(report: &BrainSynthesisReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus synthesize");
            println!("source: {}", report.source);
            println!("brain: {}", report.policy);
            println!("drafts: {}", report.draft_count);
            println!(
                "goal: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("none")
            );
            println!("summary: {}", report.summary);
            print_list("observation", &report.observations);
            print_list("question", &report.questions);
            print_list("option", &report.options);
            print_list("risk", &report.risks);
            println!("audit: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("next: {next}");
            }
        }
        Language::Zh => {
            println!("章鱼合成");
            println!("来源: {}", report.source);
            println!("主脑: {}", report.policy);
            println!("草稿数: {}", report.draft_count);
            println!(
                "目标: {}",
                report
                    .goal
                    .as_ref()
                    .map(|goal| goal.objective.as_str())
                    .unwrap_or("无")
            );
            println!("摘要: {}", report.summary);
            print_list("观察", &report.observations);
            print_list("问题", &report.questions);
            print_list("选项", &report.options);
            print_list("风险", &report.risks);
            println!("审计: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("Need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("下一步: {next}");
            }
        }
    }
}

fn print_brain_council_save(report: &BrainCouncilSaveReport, language: Language) {
    print_brain_council(&report.council, language);
    match language {
        Language::En => println!("queued: {}", report.queued.len()),
        Language::Zh => println!("已入队: {}", report.queued.len()),
    }
    print_need_queue(&report.queue, language);
}

fn print_brain_council(report: &BrainCouncilReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus brain council");
            println!("brain: {}", report.policy);
            println!("models: {}", report.prefixes.join(", "));
            println!("drafts: {}", report.draft_count);
            print_brain_synthesis(&report.synthesis, language);
            for next in &report.next {
                println!("next: {next}");
            }
        }
        Language::Zh => {
            println!("章鱼主脑委员会");
            println!("主脑: {}", report.policy);
            println!("模型: {}", report.prefixes.join(", "));
            println!("草稿数: {}", report.draft_count);
            print_brain_synthesis(&report.synthesis, language);
            for next in &report.next {
                println!("下一步: {next}");
            }
        }
    }
}

fn print_list(label: &str, items: &[String]) {
    for item in items {
        println!("{label}: {item}");
    }
}

fn print_brain_goal_save(report: &BrainGoalSaveReport, language: Language) {
    print_brain_goal(&report.goal, language);
    match language {
        Language::En => println!("queued: {}", report.queued.len()),
        Language::Zh => println!("已入队: {}", report.queued.len()),
    }
    print_need_queue(&report.queue, language);
}

fn print_brain_goal(report: &BrainGoalReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus brain goal");
            println!("source: {}", report.source);
            println!("brain: {}", report.policy);
            println!("goal: {}", report.goal.objective);
            println!("constraints: {}", report.goal.constraints.len());
            println!("summary: {}", report.summary);
            println!("audit: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("next: {next}");
            }
        }
        Language::Zh => {
            println!("章鱼主脑目标");
            println!("来源: {}", report.source);
            println!("主脑: {}", report.policy);
            println!("目标: {}", report.goal.objective);
            println!("约束: {}", report.goal.constraints.len());
            println!("摘要: {}", report.summary);
            println!("审计: {}", brain_audit_line(&report.audit));
            for need in clean_brain_print_needs(&report.audit, &report.needs) {
                println!("Need: {} {}", need_label(&need.kind), need.query);
            }
            print_polluted_need_count(&report.audit, language);
            for next in &report.next {
                println!("下一步: {next}");
            }
        }
    }
}

fn clean_brain_print_needs<'a>(
    audit: &'a octopus_core::BrainNeedAudit,
    raw_needs: &'a [GoalNeedSuggestion],
) -> &'a [GoalNeedSuggestion] {
    if audit.issue_count == 0 {
        raw_needs
    } else {
        &audit.clean_needs
    }
}

fn print_polluted_need_count(audit: &octopus_core::BrainNeedAudit, language: Language) {
    if audit.issue_count == 0 {
        return;
    }
    match language {
        Language::En => println!("blocked_needs: {}", audit.issue_count),
        Language::Zh => println!("已阻止Need: {}", audit.issue_count),
    }
}

fn brain_audit_line(audit: &octopus_core::BrainNeedAudit) -> String {
    if audit.issue_count == 0 {
        return audit.summary.clone();
    }
    let signals = audit
        .issues
        .iter()
        .map(|issue| format!("#{} {}", issue.index, issue.signal))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{} ({signals})", audit.summary)
}

fn print_goal(state: &HarnessState, language: Language) {
    let Some(goal) = &state.goal else {
        match language {
            Language::En => println!("no active goal"),
            Language::Zh => println!("没有活跃目标"),
        }
        return;
    };
    match language {
        Language::En => {
            println!("goal: {}", goal.objective);
            if !goal.constraints.is_empty() {
                println!("refinements:");
                for item in &goal.constraints {
                    println!("- {item}");
                }
            }
        }
        Language::Zh => {
            println!("目标: {}", goal.objective);
            if !goal.constraints.is_empty() {
                println!("调整:");
                for item in &goal.constraints {
                    println!("- {item}");
                }
            }
        }
    }
}
