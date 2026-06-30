use crate::pet_events;
use crate::{parse_status, resolve_feed_trace_selector, Language};
use octopus_core::{FeedFeedbackOutcome, HarnessState};
use std::path::Path;

pub(crate) fn handle_feedback_command(
    rest: &[String],
    state: &Path,
    json: bool,
    language: Language,
) -> Result<(), String> {
    let status = rest
        .get(2)
        .ok_or_else(|| "feedback requires a status".to_string())
        .and_then(|value| parse_status(value))?;
    let summary = rest
        .get(3..)
        .filter(|values| !values.is_empty())
        .map(|values| values.join(" "))
        .unwrap_or_else(|| "recorded feed feedback".to_string());
    let mut loaded = HarnessState::load(state).map_err(|error| error.to_string())?;
    let trace_index = rest
        .get(1)
        .ok_or_else(|| "feedback requires a feed trace index or latest".to_string())
        .and_then(|value| resolve_feed_trace_selector(value, &loaded))?;
    let outcome = loaded.record_feed_feedback(trace_index, status, summary)?;
    loaded.save(state).map_err(|error| error.to_string())?;
    pet_events::append_latest(state, &loaded)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&outcome).map_err(|error| error.to_string())?
        );
    } else {
        print_feed_feedback_outcome(&outcome, language);
    }
    Ok(())
}

pub(crate) fn print_feed_feedback_outcome(outcome: &FeedFeedbackOutcome, language: Language) {
    match language {
        Language::En => {
            println!("recorded feed feedback #{}", outcome.trace.index);
            println!("status: {:?}", outcome.status);
            if let Some(score) = outcome.route_score {
                println!("route_score: {score:.2}");
            }
            println!("pet: {}", outcome.pet_event.state);
        }
        Language::Zh => {
            println!("已记录 Feed 反馈 #{}", outcome.trace.index);
            println!("状态: {:?}", outcome.status);
            if let Some(score) = outcome.route_score {
                println!("路由分数: {score:.2}");
            }
            println!("章鱼状态: {}", outcome.pet_event.state);
        }
    }
}
