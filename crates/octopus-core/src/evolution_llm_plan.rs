use crate::{
    evolution_candidate, evolution_prompt, ChatClient, ChatMessage, ChatRole,
    TentacleEvolutionProposal,
};

pub(crate) fn llm_evolution_plan<C>(
    proposal: &TentacleEvolutionProposal,
    client: &mut C,
) -> Result<evolution_candidate::ParsedLlmEvolutionPlan, String>
where
    C: ChatClient,
{
    let mut retry_note = None;
    let mut last_error = None;
    for _ in 0..2 {
        match llm_evolution_plan_once(proposal, client, retry_note.as_deref()) {
            Ok(plan) => return Ok(plan),
            Err(error) => {
                retry_note = Some(error.clone());
                last_error = Some(error);
            }
        }
    }
    Err(last_error.unwrap_or_else(|| "evolution LLM planner failed".to_string()))
}

fn llm_evolution_plan_once<C>(
    proposal: &TentacleEvolutionProposal,
    client: &mut C,
    retry_note: Option<&str>,
) -> Result<evolution_candidate::ParsedLlmEvolutionPlan, String>
where
    C: ChatClient,
{
    let mut messages = vec![
        ChatMessage::new(
            ChatRole::System,
            "You are an Octopus harness evolution brain. Preserve this context policy: clean-brain LLM context is only Goal, Mem, Need, and Feed; tentacle LLM context is Need, Tool, Action, Tool, Action, then Feed. Return only JSON and no hidden reasoning.",
        ),
        ChatMessage::new(ChatRole::User, evolution_prompt::llm_evolution_prompt(proposal)?),
    ];
    if let Some(note) = retry_note {
        messages.push(ChatMessage::new(
            ChatRole::User,
            format!(
                "The previous candidate set was rejected by the manifest surface validator. Validator report: {note}. Return corrected JSON only, including candidates for every missing surface named in the report."
            ),
        ));
    }
    let response = client.chat(&messages)?;
    evolution_candidate::parse_llm_evolution_plan(proposal, &response.content)
}
