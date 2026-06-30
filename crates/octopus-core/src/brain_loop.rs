use super::*;

impl HarnessState {
    pub fn clean_brain_explore_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainExploreReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_explore_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_explore_report(brain, prompt, "llm", draft.summary, draft.needs))
    }

    pub fn clean_brain_explore_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainExploreDraft,
    ) -> BrainExploreReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_explore_report(brain, prompt, "external_chat", draft.summary, draft.needs)
    }

    pub fn clean_brain_brief_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainExploreReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_brief_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_brief_report(brain, prompt, "llm_brief", draft.summary, draft.needs))
    }

    pub fn clean_brain_brief_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainExploreDraft,
    ) -> BrainExploreReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_brief_report(brain, prompt, "external_brief", draft.summary, draft.needs)
    }

    pub fn clean_brain_intent_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainExploreReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_intent_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_intent_report(brain, prompt, "llm_intent", draft.summary, draft.needs))
    }

    pub fn clean_brain_intent_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainExploreDraft,
    ) -> BrainExploreReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_intent_report(brain, prompt, "external_intent", draft.summary, draft.needs)
    }

    pub fn clean_brain_focus_with_client<C>(
        &self,
        kind: NeedKind,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainExploreReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_focus_from_chat(&brain, &kind, &prompt, client)?;
        Ok(self.brain_focus_report(
            brain,
            prompt,
            &kind,
            &format!("llm_focus_{}", kind_key(&kind)),
            draft.summary,
            draft.needs,
        ))
    }

    pub fn clean_brain_focus_from_draft(
        &self,
        kind: NeedKind,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainExploreDraft,
    ) -> BrainExploreReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_focus_report(
            brain,
            prompt,
            &kind,
            &format!("external_focus_{}", kind_key(&kind)),
            draft.summary,
            draft.needs,
        )
    }

    pub fn clean_brain_memory_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainExploreReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_memory_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_memory_report(brain, prompt, "llm_memory", draft.summary, draft.needs))
    }

    pub fn clean_brain_memory_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainExploreDraft,
    ) -> BrainExploreReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_memory_report(brain, prompt, "external_memory", draft.summary, draft.needs)
    }

    pub fn clean_brain_clarify_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainDeliberationReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_clarification_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_clarification_report(brain, prompt, "llm_clarify", draft))
    }

    pub fn clean_brain_clarify_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainDeliberationDraft,
    ) -> BrainDeliberationReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_clarification_report(brain, prompt, "external_clarify", draft)
    }

    pub fn clean_brain_agenda_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainDeliberationReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_agenda_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_agenda_report(brain, prompt, "llm_agenda", draft))
    }

    pub fn clean_brain_agenda_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainDeliberationDraft,
    ) -> BrainDeliberationReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_agenda_report(brain, prompt, "external_agenda", draft)
    }

    pub fn clean_brain_scout_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainDeliberationReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_scout_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_scout_report(brain, prompt, "llm_scout", draft))
    }

    pub fn clean_brain_scout_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainDeliberationDraft,
    ) -> BrainDeliberationReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_scout_report(brain, prompt, "external_scout", draft)
    }

    pub fn clean_brain_deliberate_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainDeliberationReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_deliberation_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_deliberation_report(brain, prompt, "llm_deliberation", draft))
    }

    pub fn clean_brain_deliberate_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainDeliberationDraft,
    ) -> BrainDeliberationReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_deliberation_report(brain, prompt, "external_deliberation", draft)
    }

    pub fn clean_brain_reflect_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainReflectionReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_reflection_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_reflection_report(brain, prompt, "llm_reflection", draft))
    }

    pub fn clean_brain_reflect_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainReflectionDraft,
    ) -> BrainReflectionReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_reflection_report(brain, prompt, "external_reflection", draft)
    }

    pub fn clean_brain_align_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainReflectionReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft = brain_alignment_from_chat(&brain, &prompt, client)?;
        Ok(self.brain_alignment_report(brain, prompt, "llm_align", draft))
    }

    pub fn clean_brain_align_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainReflectionDraft,
    ) -> BrainReflectionReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.brain_alignment_report(brain, prompt, "external_align", draft)
    }

    pub fn clean_brain_synthesize_from_input(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        input: BrainSynthesisInput,
    ) -> BrainSynthesisReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let (draft_count, summary, observations, questions, options, risks, needs) =
            merge_brain_synthesis_input(input);
        self.brain_synthesis_report(
            brain,
            prompt,
            "external_synthesis",
            draft_count,
            summary,
            observations,
            questions,
            options,
            risks,
            needs,
        )
    }

    pub fn clean_brain_synthesize_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        input: BrainSynthesisInput,
        client: &mut C,
    ) -> Result<BrainSynthesisReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let draft_count = brain_synthesis_draft_count(&input);
        let synthesis = brain_synthesis_from_chat(&brain, &prompt, &input, client)?;
        let (merged_count, summary, observations, questions, options, risks, needs) =
            merge_brain_synthesis_input(synthesis);
        Ok(self.brain_synthesis_report(
            brain,
            prompt,
            "llm_synthesis",
            draft_count.max(merged_count),
            summary,
            observations,
            questions,
            options,
            risks,
            needs,
        ))
    }

    pub fn clean_brain_rewrite_from_draft(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainExploreDraft,
    ) -> BrainExploreReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let raw_audit = audit_clean_brain_needs(&draft.needs);
        let summary = if raw_audit.issue_count == 0 {
            draft.summary
        } else {
            format!(
                "rewrite review accepted {} clean Need(s); {} polluted Need(s) require live rewrite",
                raw_audit.clean_count, raw_audit.issue_count
            )
        };
        self.brain_explore_report(
            brain,
            prompt,
            "rewrite_review",
            summary,
            raw_audit.clean_needs,
        )
    }

    pub fn clean_brain_rewrite_with_client<C>(
        &self,
        prompt: impl Into<String>,
        limit: usize,
        draft: BrainExploreDraft,
        client: &mut C,
    ) -> Result<BrainExploreReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let raw_audit = audit_clean_brain_needs(&draft.needs);
        if raw_audit.issue_count == 0 {
            return Ok(self.brain_explore_report(
                brain,
                prompt,
                "rewrite_clean",
                draft.summary,
                draft.needs,
            ));
        }
        let rewrite = brain_rewrite_from_chat(&brain, &prompt, &draft, &raw_audit, client)?;
        Ok(self.brain_explore_report(brain, prompt, "llm_rewrite", rewrite.summary, rewrite.needs))
    }

    pub fn clean_brain_goal_with_client<C>(
        &mut self,
        prompt: impl Into<String>,
        limit: usize,
        client: &mut C,
    ) -> Result<BrainGoalReport, String>
    where
        C: ChatClient,
    {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let refinement = brain_goal_from_chat(&brain, &prompt, client)?;
        Ok(self.apply_clean_brain_goal(brain, prompt, "llm", refinement))
    }

    pub fn clean_brain_goal_from_refinement(
        &mut self,
        prompt: impl Into<String>,
        limit: usize,
        refinement: GoalRefinement,
    ) -> BrainGoalReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        self.apply_clean_brain_goal(brain, prompt, "external_chat", refinement)
    }

    pub fn clean_brain_prompt(&self, prompt: impl Into<String>, limit: usize) -> BrainPromptReport {
        let prompt = prompt.into();
        let brain = self.context_report(None, limit).brain;
        let context = serde_json::json!({
            "policy": brain.policy,
            "slots": brain.slots,
            "goal": brain.goal,
            "mem": brain.mem,
            "recent_need_feed": brain.turns,
        });
        let context_text = serde_json::to_string_pretty(&context)
            .unwrap_or_else(|_| "{\"policy\":\"Goal + Mem + Need + Feed\"}".to_string());
        let system = "You are the Octopus clean brain. Use only Goal, Mem, Need, and Feed. Express cognitive Needs only. Return JSON with {\"summary\":\"short\",\"needs\":[{\"kind\":\"observe|verify|reproduce|compare|remember|forget|recall|execute\",\"query\":\"short cognitive request\"}]}."
            .to_string();
        let user =
            format!("Clean brain prompt: {prompt}\nClean brain context JSON:\n{context_text}");
        let messages = vec![
            ChatMessage::new(ChatRole::System, system),
            ChatMessage::new(ChatRole::User, user),
        ];
        let prompt_text = messages
            .iter()
            .map(|message| format!("{:?}:\n{}", message.role, message.content))
            .collect::<Vec<_>>()
            .join("\n\n");
        let next = vec![
            "paste messages into any chat-completions-compatible model".to_string(),
            format!("octopus explore {}", shell_arg(&prompt)),
            "octopus context observe .".to_string(),
        ];
        BrainPromptReport {
            policy: brain.policy,
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            messages,
            prompt_text,
            next,
        }
    }

    fn apply_clean_brain_goal(
        &mut self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        refinement: GoalRefinement,
    ) -> BrainGoalReport {
        let previous_goal = self.goal.clone();
        let objective = clean_optional(refinement.objective.as_deref())
            .or_else(|| previous_goal.as_ref().map(|goal| goal.objective.as_str()))
            .or_else(|| clean_optional(Some(&prompt)))
            .unwrap_or("clean-brain goal")
            .to_string();
        let mut goal = previous_goal
            .clone()
            .unwrap_or_else(|| Goal::new(objective.clone()));
        goal.objective = objective;
        for constraint in &refinement.constraints {
            if let Some(constraint) = clean_optional(Some(constraint.as_str())) {
                if !goal.constraints.iter().any(|item| item == constraint) {
                    goal.refine(constraint.to_string());
                }
            }
        }
        goal.signals
            .insert("brain_goal_source".to_string(), source.to_string());
        let audit = audit_clean_brain_needs(&refinement.needs);
        if !audit.clean_needs.is_empty() {
            let suggested = audit
                .clean_needs
                .iter()
                .map(|need| format!("{}: {}", kind_key(&need.kind), need.query))
                .collect::<Vec<_>>()
                .join(" | ");
            goal.signals
                .insert("suggested_needs".to_string(), suggested);
        }
        let summary = clean_optional(refinement.summary.as_deref())
            .map(str::to_string)
            .unwrap_or_else(|| "goal refined by clean brain".to_string());
        let turn = GoalTurn {
            index: self.goal_turns.len() as u64 + 1,
            message: prompt.clone(),
            summary: summary.clone(),
            status: Status::Satisfied,
        };
        self.goal = Some(goal.clone());
        self.goal_turns.push(turn);
        let next = if audit.clean_needs.is_empty() {
            let mut next = vec![
                "octopus brain --live \"what should the brain ask next?\"".to_string(),
                "octopus context".to_string(),
            ];
            if audit.issue_count > 0 {
                next.insert(
                    0,
                    "revise Need suggestions as cognitive requests before Feed".to_string(),
                );
            }
            next
        } else {
            let mut next = audit
                .clean_needs
                .iter()
                .map(|need| {
                    format!(
                        "octopus need {} {}",
                        kind_key(&need.kind),
                        shell_arg(&need.query)
                    )
                })
                .collect::<Vec<_>>();
            next.push(format!(
                "octopus brain --goal --save {}",
                shell_arg(&prompt)
            ));
            next
        };
        BrainGoalReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            previous_goal,
            goal,
            summary,
            audit,
            needs: refinement.needs,
            next,
        }
    }

    fn brain_explore_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        summary: impl Into<String>,
        needs: Vec<GoalNeedSuggestion>,
    ) -> BrainExploreReport {
        let audit = audit_clean_brain_needs(&needs);
        let next = if audit.clean_needs.is_empty() {
            if audit.issue_count > 0 {
                vec![
                    "revise Need suggestions as cognitive requests before Feed".to_string(),
                    "octopus brain --session \"rewrite these Needs cleanly\"".to_string(),
                ]
            } else {
                vec!["octopus goal set \"describe your goal\"".to_string()]
            }
        } else {
            audit
                .clean_needs
                .iter()
                .map(|need| {
                    format!(
                        "octopus need {} {}",
                        kind_key(&need.kind),
                        shell_arg(&need.query)
                    )
                })
                .collect()
        };
        BrainExploreReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            summary: summary.into(),
            needs,
            audit,
            next,
        }
    }

    fn brain_intent_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        summary: impl Into<String>,
        needs: Vec<GoalNeedSuggestion>,
    ) -> BrainExploreReport {
        let mut report = self.brain_explore_report(brain, prompt.clone(), source, summary, needs);
        report
            .next
            .push("octopus brain --intent --session".to_string());
        if report.audit.clean_needs.is_empty() {
            if report.audit.issue_count == 0 {
                report
                    .next
                    .push(format!("octopus brain --intent {}", shell_arg(&prompt)));
            }
        } else {
            report.next.push(format!(
                "octopus brain --intent --save {}",
                shell_arg(&prompt)
            ));
        }
        report.next.sort();
        report.next.dedup();
        report
    }

    fn brain_brief_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        summary: impl Into<String>,
        needs: Vec<GoalNeedSuggestion>,
    ) -> BrainExploreReport {
        let mut report = self.brain_explore_report(brain, prompt.clone(), source, summary, needs);
        report
            .next
            .push("octopus brain --brief --session".to_string());
        if report.audit.clean_needs.is_empty() {
            if report.audit.issue_count == 0 {
                report
                    .next
                    .push(format!("octopus brain --brief {}", shell_arg(&prompt)));
            }
        } else {
            report.next.push(format!(
                "octopus brain --brief --save {}",
                shell_arg(&prompt)
            ));
        }
        report.next.sort();
        report.next.dedup();
        report
    }

    fn brain_memory_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        summary: impl Into<String>,
        needs: Vec<GoalNeedSuggestion>,
    ) -> BrainExploreReport {
        let mut report = self.brain_explore_report(brain, prompt.clone(), source, summary, needs);
        report
            .next
            .push("octopus brain --memory --session".to_string());
        if report.audit.clean_needs.is_empty() {
            if report.audit.issue_count == 0 {
                report
                    .next
                    .push("octopus brain --memory \"what should be remembered?\"".to_string());
            }
        } else {
            report.next.push(format!(
                "octopus brain --memory --save {}",
                shell_arg(&prompt)
            ));
        }
        report.next.sort();
        report.next.dedup();
        report
    }

    fn brain_focus_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        kind: &NeedKind,
        source: &str,
        summary: impl Into<String>,
        needs: Vec<GoalNeedSuggestion>,
    ) -> BrainExploreReport {
        let label = kind_key(kind);
        let mut report = self.brain_explore_report(brain, prompt.clone(), source, summary, needs);
        report
            .next
            .push(format!("octopus brain --focus {label} --session"));
        if report.audit.clean_needs.is_empty() {
            if report.audit.issue_count == 0 {
                report.next.push(format!(
                    "octopus brain --focus {label} {}",
                    shell_arg(&prompt)
                ));
            }
        } else {
            report.next.push(format!(
                "octopus brain --focus {label} --save {}",
                shell_arg(&prompt)
            ));
        }
        report.next.sort();
        report.next.dedup();
        report
    }

    fn brain_deliberation_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        draft: BrainDeliberationDraft,
    ) -> BrainDeliberationReport {
        let audit = audit_clean_brain_needs(&draft.needs);
        let mut next = vec!["octopus brain --deliberate --session".to_string()];
        if audit.clean_needs.is_empty() {
            if audit.issue_count > 0 {
                next.push(
                    "rewrite deliberation Needs as cognitive requests before Feed".to_string(),
                );
            } else {
                next.push(
                    "octopus brain --deliberate \"what should the brain examine next?\""
                        .to_string(),
                );
            }
        } else {
            next.extend(audit.clean_needs.iter().map(|need| {
                format!(
                    "octopus need {} {}",
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                )
            }));
            next.push(format!(
                "octopus brain --deliberate --save {}",
                shell_arg(&prompt)
            ));
        }
        next.sort();
        next.dedup();
        BrainDeliberationReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            summary: draft.summary,
            observations: draft.observations,
            questions: draft.questions,
            options: draft.options,
            risks: draft.risks,
            needs: draft.needs,
            audit,
            next,
        }
    }

    fn brain_clarification_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        draft: BrainDeliberationDraft,
    ) -> BrainDeliberationReport {
        let audit = audit_clean_brain_needs(&draft.needs);
        let mut next = vec!["octopus brain --clarify --session".to_string()];
        if audit.clean_needs.is_empty() {
            if audit.issue_count > 0 {
                next.push(
                    "rewrite clarification Needs as cognitive requests before Feed".to_string(),
                );
            } else {
                next.push(format!("octopus goal set {}", shell_arg(&prompt)));
            }
        } else {
            next.extend(audit.clean_needs.iter().map(|need| {
                format!(
                    "octopus need {} {}",
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                )
            }));
            next.push(format!(
                "octopus brain --clarify --save {}",
                shell_arg(&prompt)
            ));
        }
        next.sort();
        next.dedup();
        BrainDeliberationReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            summary: draft.summary,
            observations: draft.observations,
            questions: draft.questions,
            options: draft.options,
            risks: draft.risks,
            needs: draft.needs,
            audit,
            next,
        }
    }

    fn brain_agenda_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        draft: BrainDeliberationDraft,
    ) -> BrainDeliberationReport {
        let audit = audit_clean_brain_needs(&draft.needs);
        let mut next = vec!["octopus brain --agenda --session".to_string()];
        if audit.clean_needs.is_empty() {
            if audit.issue_count > 0 {
                next.push("rewrite agenda Needs as cognitive requests before Feed".to_string());
            } else {
                next.push("octopus brain --clarify \"what should the user clarify?\"".to_string());
            }
        } else {
            next.extend(audit.clean_needs.iter().map(|need| {
                format!(
                    "octopus need {} {}",
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                )
            }));
            next.push(format!(
                "octopus brain --agenda --save {}",
                shell_arg(&prompt)
            ));
        }
        next.sort();
        next.dedup();
        BrainDeliberationReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            summary: draft.summary,
            observations: draft.observations,
            questions: draft.questions,
            options: draft.options,
            risks: draft.risks,
            needs: draft.needs,
            audit,
            next,
        }
    }

    fn brain_scout_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        draft: BrainDeliberationDraft,
    ) -> BrainDeliberationReport {
        let audit = audit_clean_brain_needs(&draft.needs);
        let mut next = vec!["octopus brain --scout --session".to_string()];
        if audit.clean_needs.is_empty() {
            if audit.issue_count > 0 {
                next.push("rewrite scout Needs as cognitive requests before Feed".to_string());
            } else {
                next.push("octopus brain --scout \"what should the brain map next?\"".to_string());
            }
        } else {
            next.extend(audit.clean_needs.iter().map(|need| {
                format!(
                    "octopus need {} {}",
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                )
            }));
            next.push(format!(
                "octopus brain --scout --save {}",
                shell_arg(&prompt)
            ));
        }
        next.sort();
        next.dedup();
        BrainDeliberationReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            summary: draft.summary,
            observations: draft.observations,
            questions: draft.questions,
            options: draft.options,
            risks: draft.risks,
            needs: draft.needs,
            audit,
            next,
        }
    }

    fn brain_reflection_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        draft: BrainReflectionDraft,
    ) -> BrainReflectionReport {
        let audit = audit_clean_brain_needs(&draft.needs);
        let mut next = vec!["octopus brain --reflect --session".to_string()];
        if audit.clean_needs.is_empty() {
            if audit.issue_count > 0 {
                next.push("rewrite reflection Needs as cognitive requests before Feed".to_string());
            } else {
                next.push("octopus brain --reflect \"what goal evidence is missing?\"".to_string());
            }
        } else {
            next.extend(audit.clean_needs.iter().map(|need| {
                format!(
                    "octopus need {} {}",
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                )
            }));
            next.push(format!(
                "octopus brain --reflect --save {}",
                shell_arg(&prompt)
            ));
        }
        next.sort();
        next.dedup();
        BrainReflectionReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            summary: draft.summary,
            goal_state: draft.goal_state,
            evidence: draft.evidence,
            gaps: draft.gaps,
            questions: draft.questions,
            needs: draft.needs,
            audit,
            next,
        }
    }

    fn brain_alignment_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        draft: BrainReflectionDraft,
    ) -> BrainReflectionReport {
        let audit = audit_clean_brain_needs(&draft.needs);
        let mut next = vec!["octopus brain --align --session".to_string()];
        if audit.clean_needs.is_empty() {
            if audit.issue_count > 0 {
                next.push("rewrite alignment Needs as cognitive requests before Feed".to_string());
            } else {
                next.push("octopus brain --align \"does this still follow the goal?\"".to_string());
            }
        } else {
            next.extend(audit.clean_needs.iter().map(|need| {
                format!(
                    "octopus need {} {}",
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                )
            }));
            next.push(format!(
                "octopus brain --align --save {}",
                shell_arg(&prompt)
            ));
        }
        next.sort();
        next.dedup();
        BrainReflectionReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            summary: draft.summary,
            goal_state: draft.goal_state,
            evidence: draft.evidence,
            gaps: draft.gaps,
            questions: draft.questions,
            needs: draft.needs,
            audit,
            next,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn brain_synthesis_report(
        &self,
        brain: BrainContextReport,
        prompt: String,
        source: &str,
        draft_count: usize,
        summary: String,
        observations: Vec<String>,
        questions: Vec<String>,
        options: Vec<String>,
        risks: Vec<String>,
        needs: Vec<GoalNeedSuggestion>,
    ) -> BrainSynthesisReport {
        let audit = audit_clean_brain_needs(&needs);
        let mut next = vec!["octopus brain --synthesize --session".to_string()];
        if audit.clean_needs.is_empty() {
            if audit.issue_count > 0 {
                next.push("rewrite synthesis Needs as cognitive requests before Feed".to_string());
            } else {
                next.push("octopus brain --synthesize --session --apply <drafts.json>".to_string());
            }
        } else {
            next.extend(audit.clean_needs.iter().map(|need| {
                format!(
                    "octopus need {} {}",
                    kind_key(&need.kind),
                    shell_arg(&need.query)
                )
            }));
            next.push("octopus needs script".to_string());
        }
        next.sort();
        next.dedup();
        BrainSynthesisReport {
            policy: brain.policy,
            source: source.to_string(),
            prompt,
            goal: brain.goal,
            mem: brain.mem,
            recent: brain.turns,
            summary,
            draft_count,
            observations,
            questions,
            options,
            risks,
            needs,
            audit,
            next,
        }
    }
}
