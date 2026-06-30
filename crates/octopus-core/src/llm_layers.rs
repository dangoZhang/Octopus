use super::*;

pub(crate) fn chat_llm_client() -> Result<Box<dyn ChatClient>, String> {
    Ok(provider_client(&chat_llm_prefix())?.4)
}

pub(crate) fn clean_brain_explore_llm_client() -> Result<Box<dyn ChatClient>, String> {
    Ok(provider_client(&clean_brain_explore_llm_prefix())?.4)
}

pub(crate) fn clean_brain_intent_llm_client() -> Result<Box<dyn ChatClient>, String> {
    Ok(provider_client(&clean_brain_intent_llm_prefix())?.4)
}

pub(crate) fn clean_brain_brief_llm_client() -> Result<Box<dyn ChatClient>, String> {
    Ok(provider_client(&clean_brain_brief_llm_prefix())?.4)
}

pub(crate) fn clean_brain_deliberate_llm_client() -> Result<Box<dyn ChatClient>, String> {
    Ok(provider_client(&clean_brain_deliberate_llm_prefix())?.4)
}

pub(crate) fn clean_brain_clarify_llm_client() -> Result<Box<dyn ChatClient>, String> {
    Ok(provider_client(&clean_brain_clarify_llm_prefix())?.4)
}

pub(crate) fn clean_brain_agenda_llm_client() -> Result<Box<dyn ChatClient>, String> {
    Ok(provider_client(&clean_brain_agenda_llm_prefix())?.4)
}

pub(crate) fn clean_brain_scout_llm_client() -> Result<Box<dyn ChatClient>, String> {
    Ok(provider_client(&clean_brain_scout_llm_prefix())?.4)
}

pub(crate) fn clean_brain_reflect_llm_client() -> Result<Box<dyn ChatClient>, String> {
    Ok(provider_client(&clean_brain_reflect_llm_prefix())?.4)
}

pub(crate) fn clean_brain_align_llm_client() -> Result<Box<dyn ChatClient>, String> {
    Ok(provider_client(&clean_brain_align_llm_prefix())?.4)
}

pub(crate) fn clean_brain_memory_llm_client() -> Result<Box<dyn ChatClient>, String> {
    Ok(provider_client(&clean_brain_memory_llm_prefix())?.4)
}

pub(crate) fn clean_brain_synthesize_llm_client() -> Result<Box<dyn ChatClient>, String> {
    Ok(provider_client(&clean_brain_synthesize_llm_prefix())?.4)
}

pub(crate) fn clean_brain_goal_llm_client() -> Result<Box<dyn ChatClient>, String> {
    Ok(provider_client(&clean_brain_goal_llm_prefix())?.4)
}

pub(crate) fn clean_brain_rewrite_llm_client() -> Result<Box<dyn ChatClient>, String> {
    Ok(provider_client(&clean_brain_rewrite_llm_prefix())?.4)
}

pub(crate) fn clean_brain_queue_llm_client() -> Result<Box<dyn ChatClient>, String> {
    Ok(provider_client(&clean_brain_queue_llm_prefix())?.4)
}

pub(crate) fn manifest_llm_factory() -> Result<ChatClientFactory, String> {
    provider_client_factory(&manifest_llm_prefix())
}

pub(crate) fn doctor_llm_prefix() -> String {
    if brain_llm_enabled() {
        brain_llm_prefix()
    } else if chat_llm_enabled() {
        chat_llm_prefix()
    } else if manifest_llm_enabled() {
        manifest_llm_prefix()
    } else if evolve_llm_enabled() {
        evolve_llm_prefix()
    } else {
        "OCTOPUS_LLM".to_string()
    }
}

pub(crate) fn chat_llm_prefix() -> String {
    env::var("OCTOPUS_CHAT_LLM_PREFIX").unwrap_or_else(|_| "OCTOPUS_LLM".to_string())
}

pub(crate) fn brain_llm_prefix() -> String {
    env::var("OCTOPUS_BRAIN_LLM_PREFIX").unwrap_or_else(|_| "OCTOPUS_LLM".to_string())
}

pub(crate) fn clean_brain_llm_prefix() -> String {
    if brain_llm_enabled() {
        brain_llm_prefix()
    } else if chat_llm_enabled() {
        chat_llm_prefix()
    } else {
        brain_llm_prefix()
    }
}

pub(crate) fn clean_brain_intent_llm_prefix() -> String {
    clean_brain_slot_llm_prefix("OCTOPUS_BRAIN_INTENT_LLM_PREFIX")
}

pub(crate) fn clean_brain_brief_llm_prefix() -> String {
    clean_brain_slot_llm_prefix("OCTOPUS_BRAIN_BRIEF_LLM_PREFIX")
}

pub(crate) fn clean_brain_explore_llm_prefix() -> String {
    clean_brain_slot_llm_prefix("OCTOPUS_BRAIN_EXPLORE_LLM_PREFIX")
}

pub(crate) fn clean_brain_clarify_llm_prefix() -> String {
    clean_brain_slot_llm_prefix("OCTOPUS_BRAIN_CLARIFY_LLM_PREFIX")
}

pub(crate) fn clean_brain_agenda_llm_prefix() -> String {
    clean_brain_slot_llm_prefix("OCTOPUS_BRAIN_AGENDA_LLM_PREFIX")
}

pub(crate) fn clean_brain_scout_llm_prefix() -> String {
    clean_brain_slot_llm_prefix("OCTOPUS_BRAIN_SCOUT_LLM_PREFIX")
}

pub(crate) fn clean_brain_deliberate_llm_prefix() -> String {
    clean_brain_slot_llm_prefix("OCTOPUS_BRAIN_DELIBERATE_LLM_PREFIX")
}

pub(crate) fn clean_brain_reflect_llm_prefix() -> String {
    clean_brain_slot_llm_prefix("OCTOPUS_BRAIN_REFLECT_LLM_PREFIX")
}

pub(crate) fn clean_brain_align_llm_prefix() -> String {
    clean_brain_slot_llm_prefix("OCTOPUS_BRAIN_ALIGN_LLM_PREFIX")
}

pub(crate) fn clean_brain_memory_llm_prefix() -> String {
    clean_brain_slot_llm_prefix("OCTOPUS_BRAIN_MEMORY_LLM_PREFIX")
}

pub(crate) fn clean_brain_synthesize_llm_prefix() -> String {
    clean_brain_slot_llm_prefix("OCTOPUS_BRAIN_SYNTHESIZE_LLM_PREFIX")
}

pub(crate) fn clean_brain_council_llm_prefixes() -> Vec<String> {
    let configured = env::var("OCTOPUS_BRAIN_COUNCIL_LLM_PREFIXES")
        .ok()
        .map(|value| parse_llm_prefix_list(&value))
        .filter(|prefixes| !prefixes.is_empty());
    configured.unwrap_or_else(|| {
        parse_llm_prefix_list(&format!(
            "{},{}",
            clean_brain_deliberate_llm_prefix(),
            clean_brain_explore_llm_prefix()
        ))
    })
}

pub(crate) struct BrainLlmPrefixOverride {
    saved: Vec<(&'static str, Option<String>)>,
}

impl BrainLlmPrefixOverride {
    pub(crate) fn apply(prefix: &str) -> Result<Self, String> {
        let prefix = parse_brain_llm_prefix(prefix)?;
        let keys = brain_llm_override_keys();
        let saved = keys
            .iter()
            .map(|key| (*key, env::var(key).ok()))
            .collect::<Vec<_>>();
        env::set_var("OCTOPUS_BRAIN_LLM", "1");
        for key in keys
            .iter()
            .copied()
            .filter(|key| *key != "OCTOPUS_BRAIN_LLM")
        {
            env::set_var(key, &prefix);
        }
        Ok(Self { saved })
    }
}

impl Drop for BrainLlmPrefixOverride {
    fn drop(&mut self) {
        for (key, value) in self.saved.iter().rev() {
            if let Some(value) = value {
                env::set_var(key, value);
            } else {
                env::remove_var(key);
            }
        }
    }
}

fn brain_llm_override_keys() -> [&'static str; 17] {
    [
        "OCTOPUS_BRAIN_LLM",
        "OCTOPUS_BRAIN_LLM_PREFIX",
        "OCTOPUS_BRAIN_INTENT_LLM_PREFIX",
        "OCTOPUS_BRAIN_BRIEF_LLM_PREFIX",
        "OCTOPUS_BRAIN_EXPLORE_LLM_PREFIX",
        "OCTOPUS_BRAIN_CLARIFY_LLM_PREFIX",
        "OCTOPUS_BRAIN_AGENDA_LLM_PREFIX",
        "OCTOPUS_BRAIN_SCOUT_LLM_PREFIX",
        "OCTOPUS_BRAIN_DELIBERATE_LLM_PREFIX",
        "OCTOPUS_BRAIN_REFLECT_LLM_PREFIX",
        "OCTOPUS_BRAIN_ALIGN_LLM_PREFIX",
        "OCTOPUS_BRAIN_MEMORY_LLM_PREFIX",
        "OCTOPUS_BRAIN_SYNTHESIZE_LLM_PREFIX",
        "OCTOPUS_BRAIN_GOAL_LLM_PREFIX",
        "OCTOPUS_BRAIN_REWRITE_LLM_PREFIX",
        "OCTOPUS_BRAIN_QUEUE_LLM_PREFIX",
        "OCTOPUS_BRAIN_COUNCIL_LLM_PREFIXES",
    ]
}

pub(crate) fn parse_brain_llm_prefix(prefix: &str) -> Result<String, String> {
    let prefix = prefix.trim();
    if !valid_env_prefix(prefix) {
        return Err(format!(
            "invalid brain env prefix: {prefix}; use letters, digits, and underscore"
        ));
    }
    Ok(prefix.to_string())
}

pub(crate) fn parse_brain_llm_prefixes(value: &str) -> Result<Vec<String>, String> {
    let prefixes = parse_llm_prefix_list(value);
    if prefixes.is_empty() {
        return Err("brain --models requires at least one env prefix".to_string());
    }
    prefixes
        .iter()
        .map(|prefix| parse_brain_llm_prefix(prefix))
        .collect()
}

fn parse_llm_prefix_list(value: &str) -> Vec<String> {
    let mut prefixes = Vec::new();
    for prefix in value
        .split([',', ';', ' ', '\n', '\t'])
        .map(str::trim)
        .filter(|prefix| !prefix.is_empty())
    {
        if !prefixes.iter().any(|item| item == prefix) {
            prefixes.push(prefix.to_string());
        }
    }
    prefixes
}

pub(crate) fn clean_brain_goal_llm_prefix() -> String {
    clean_brain_slot_llm_prefix("OCTOPUS_BRAIN_GOAL_LLM_PREFIX")
}

pub(crate) fn clean_brain_rewrite_llm_prefix() -> String {
    clean_brain_slot_llm_prefix("OCTOPUS_BRAIN_REWRITE_LLM_PREFIX")
}

pub(crate) fn clean_brain_queue_llm_prefix() -> String {
    clean_brain_slot_llm_prefix("OCTOPUS_BRAIN_QUEUE_LLM_PREFIX")
}

fn clean_brain_slot_llm_prefix(slot_env: &str) -> String {
    env::var(slot_env).unwrap_or_else(|_| clean_brain_llm_prefix())
}

pub(crate) fn manifest_llm_prefix() -> String {
    env::var("OCTOPUS_MANIFEST_LLM_PREFIX").unwrap_or_else(|_| "OCTOPUS_LLM".to_string())
}

pub(crate) fn evolve_llm_prefix() -> String {
    env::var("OCTOPUS_EVOLVE_LLM_PREFIX").unwrap_or_else(|_| "OCTOPUS_LLM".to_string())
}

pub(crate) fn manifest_llm_enabled() -> bool {
    env::var("OCTOPUS_LLM_MANIFEST")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
}

pub(crate) fn chat_llm_enabled() -> bool {
    env::var("OCTOPUS_CHAT_LLM")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
}

pub(crate) fn brain_llm_enabled() -> bool {
    env::var("OCTOPUS_BRAIN_LLM")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
}

pub(crate) fn clean_brain_llm_enabled() -> bool {
    brain_llm_enabled() || chat_llm_enabled()
}

pub(crate) fn evolve_llm_enabled() -> bool {
    env::var("OCTOPUS_LLM_EVOLVE")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
}
