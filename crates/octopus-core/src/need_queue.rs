use super::*;

impl HarnessState {
    pub fn queue_goal_report(&mut self, report: &BrainGoalReport) -> BrainGoalSaveReport {
        let mut queued = Vec::new();
        for need in &report.audit.clean_needs {
            if let Some(existing) = self
                .need_queue
                .iter()
                .find(|item| item.status == NeedQueueStatus::Pending && item.need == *need)
            {
                queued.push(existing.clone());
                continue;
            }
            self.next_need_queue_index += 1;
            let item = NeedQueueItem {
                index: self.next_need_queue_index,
                need: need.clone(),
                context: BTreeMap::new(),
                source: report.source.clone(),
                prompt: report.prompt.clone(),
                summary: report.summary.clone(),
                status: NeedQueueStatus::Pending,
            };
            self.need_queue.push(item.clone());
            queued.push(item);
        }
        BrainGoalSaveReport {
            goal: report.clone(),
            queued,
            queue: self.need_queue_report(8),
        }
    }

    pub fn queue_exploration_report(&mut self, report: &BrainExploreReport) -> NeedQueueSaveReport {
        let mut queued = Vec::new();
        for need in &report.audit.clean_needs {
            queued.push(self.queue_need_suggestion(
                need.clone(),
                report.source.clone(),
                report.prompt.clone(),
                report.summary.clone(),
            ));
        }
        NeedQueueSaveReport {
            explore: report.clone(),
            queued,
            queue: self.need_queue_report(8),
        }
    }

    pub fn queue_deliberation_report(
        &mut self,
        report: &BrainDeliberationReport,
    ) -> BrainDeliberationSaveReport {
        let mut queued = Vec::new();
        for need in &report.audit.clean_needs {
            queued.push(self.queue_need_suggestion(
                need.clone(),
                report.source.clone(),
                report.prompt.clone(),
                report.summary.clone(),
            ));
        }
        BrainDeliberationSaveReport {
            deliberation: report.clone(),
            queued,
            queue: self.need_queue_report(8),
        }
    }

    pub fn queue_reflection_report(
        &mut self,
        report: &BrainReflectionReport,
    ) -> BrainReflectionSaveReport {
        let mut queued = Vec::new();
        for need in &report.audit.clean_needs {
            queued.push(self.queue_need_suggestion(
                need.clone(),
                report.source.clone(),
                report.prompt.clone(),
                report.summary.clone(),
            ));
        }
        BrainReflectionSaveReport {
            reflection: report.clone(),
            queued,
            queue: self.need_queue_report(8),
        }
    }

    pub fn queue_synthesis_report(
        &mut self,
        report: &BrainSynthesisReport,
    ) -> BrainSynthesisSaveReport {
        let mut queued = Vec::new();
        for need in &report.audit.clean_needs {
            queued.push(self.queue_need_suggestion(
                need.clone(),
                report.source.clone(),
                report.prompt.clone(),
                report.summary.clone(),
            ));
        }
        BrainSynthesisSaveReport {
            synthesis: report.clone(),
            queued,
            queue: self.need_queue_report(8),
        }
    }

    pub fn queue_need_suggestion(
        &mut self,
        need: GoalNeedSuggestion,
        source: impl Into<String>,
        prompt: impl Into<String>,
        summary: impl Into<String>,
    ) -> NeedQueueItem {
        self.queue_need_suggestion_with_context(need, BTreeMap::new(), source, prompt, summary)
    }

    pub fn queue_need_suggestion_with_context(
        &mut self,
        need: GoalNeedSuggestion,
        context: BTreeMap<String, String>,
        source: impl Into<String>,
        prompt: impl Into<String>,
        summary: impl Into<String>,
    ) -> NeedQueueItem {
        if let Some(existing) = self.need_queue.iter().find(|item| {
            item.status == NeedQueueStatus::Pending && item.need == need && item.context == context
        }) {
            return existing.clone();
        }
        self.next_need_queue_index += 1;
        let item = NeedQueueItem {
            index: self.next_need_queue_index,
            need,
            context,
            source: source.into(),
            prompt: prompt.into(),
            summary: summary.into(),
            status: NeedQueueStatus::Pending,
        };
        self.need_queue.push(item.clone());
        item
    }

    pub fn need_queue_report(&self, limit: usize) -> NeedQueueReport {
        let limit = limit.max(1);
        let pending = self
            .need_queue
            .iter()
            .filter(|item| item.status == NeedQueueStatus::Pending)
            .cloned()
            .collect::<Vec<_>>();
        let mut history = self
            .need_queue
            .iter()
            .filter(|item| item.status != NeedQueueStatus::Pending)
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>();
        history.reverse();
        let agent_next = if let Some(item) = pending.first() {
            vec![
                format!("octopus needs run {}", item.index),
                format!("octopus needs take {}", item.index),
                format!("octopus needs drop {}", item.index),
                format!("octopus {}", need_command(&item.need)),
            ]
        } else {
            vec![
                "octopus explore --save \"what should the brain ask next?\"".to_string(),
                "octopus goal set \"describe your goal\"".to_string(),
            ]
        };
        let user_goal_hints = user_surface::need_queue_hints(&pending);
        NeedQueueReport {
            policy: CLEAN_BRAIN_CONTEXT_POLICY.to_string(),
            pending,
            history,
            agent_next,
            user_goal_hints: user_goal_hints.clone(),
            next: user_goal_hints,
        }
    }

    pub fn take_queued_need(&mut self, index: u64) -> Result<NeedQueueTakeReport, String> {
        let item = self
            .need_queue
            .iter_mut()
            .find(|item| item.index == index)
            .ok_or_else(|| format!("unknown queued Need: {index}"))?;
        if item.status != NeedQueueStatus::Pending {
            return Err(format!(
                "queued Need is already {}",
                need_queue_status_key(&item.status)
            ));
        }
        item.status = NeedQueueStatus::Taken;
        let item = item.clone();
        let command = format!("octopus {}", need_command(&item.need));
        Ok(NeedQueueTakeReport {
            item,
            command: command.clone(),
            next: vec![command, "octopus needs".to_string()],
        })
    }

    pub fn drop_queued_need(&mut self, index: u64) -> Result<NeedQueueItem, String> {
        let item = self
            .need_queue
            .iter_mut()
            .find(|item| item.index == index)
            .ok_or_else(|| format!("unknown queued Need: {index}"))?;
        if item.status != NeedQueueStatus::Pending {
            return Err(format!(
                "queued Need is already {}",
                need_queue_status_key(&item.status)
            ));
        }
        item.status = NeedQueueStatus::Dropped;
        Ok(item.clone())
    }

    pub fn pending_need_queue_count(&self) -> usize {
        self.need_queue
            .iter()
            .filter(|item| item.status == NeedQueueStatus::Pending)
            .count()
    }
}
