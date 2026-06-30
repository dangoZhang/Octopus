use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InstalledToolRef {
    pub(crate) id: String,
    pub(crate) description: String,
    pub(crate) input: String,
    pub(crate) output: String,
    pub(crate) kind: String,
    pub(crate) entrypoint: String,
    pub(crate) contract: Option<String>,
    pub(crate) permission: Option<ToolPermission>,
}

impl InstalledToolRef {
    fn parse(value: &str) -> Option<Self> {
        let mut parts = value.splitn(3, ':');
        Some(Self {
            id: parts.next()?.to_string(),
            description: String::new(),
            input: String::new(),
            output: String::new(),
            kind: parts.next()?.to_string(),
            entrypoint: parts.next()?.to_string(),
            contract: None,
            permission: None,
        })
    }

    fn from_installed(tool: &InstalledTool) -> Self {
        Self {
            id: tool.id.clone(),
            description: tool.description.clone(),
            input: tool.input.clone(),
            output: tool.output.clone(),
            kind: tool.kind.clone(),
            entrypoint: tool.entrypoint.clone(),
            contract: tool.contract.clone(),
            permission: tool.permission.clone(),
        }
    }
}

pub(crate) fn installed_tentacle_tools(tentacle: &InstalledTentacle) -> Vec<InstalledToolRef> {
    if tentacle.tool_meta.is_empty() {
        return tentacle
            .tools
            .iter()
            .filter_map(|tool| InstalledToolRef::parse(tool))
            .collect();
    }
    tentacle
        .tool_meta
        .iter()
        .map(InstalledToolRef::from_installed)
        .collect()
}

fn tool_contract_label(tool: &InstalledToolRef) -> &str {
    tool.contract.as_deref().unwrap_or("unspecified")
}

fn tool_result_contract(tool: &InstalledToolRef, uses_json_contract: bool) -> Option<String> {
    if uses_json_contract {
        Some(
            tool.contract
                .clone()
                .unwrap_or_else(|| OCTOPUS_JSON_CONTRACT.to_string()),
        )
    } else {
        tool.contract.clone()
    }
}

pub(crate) fn tool_permission_text(permission: &ToolPermission) -> String {
    let permissions = if permission.permissions.is_empty() {
        "none".to_string()
    } else {
        permission.permissions.join(",")
    };
    format!("{}:{}:{permissions}", permission.provider, permission.scope)
}

#[derive(Serialize)]
struct ToolInvocation<'a> {
    schema_version: &'static str,
    need: &'a Need,
    tool: ToolInvocationTool<'a>,
    tentacle: ToolInvocationTentacle<'a>,
}

#[derive(Serialize)]
struct ToolInvocationTool<'a> {
    id: &'a str,
    description: &'a str,
    input: &'a str,
    output: &'a str,
    runtime: &'a str,
    entrypoint: &'a str,
    contract: Option<&'a str>,
    permission: Option<&'a ToolPermission>,
}

#[derive(Serialize)]
struct ToolInvocationTentacle<'a> {
    id: &'a str,
    brain_kind: &'a str,
    brain_prompt: &'a str,
    feedback_contract: Option<&'a str>,
}

#[derive(Deserialize)]
struct ToolOutputEnvelope {
    status: Option<Status>,
    output: String,
    #[serde(default)]
    evidence: Vec<Evidence>,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ManifestToolCall {
    tool: InstalledToolRef,
    reason: String,
    payload: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ManifestToolPlan {
    tool: InstalledToolRef,
    reason: String,
    candidates: Vec<String>,
    source: String,
    calls: Vec<ManifestToolCall>,
}

fn manifest_tool_calls_from_plan(plan: &Plan, tools: &[InstalledToolRef]) -> Vec<ManifestToolCall> {
    plan.calls
        .iter()
        .filter_map(|call| {
            tools
                .iter()
                .find(|tool| tool.id == call.tool)
                .cloned()
                .map(|tool| ManifestToolCall {
                    tool,
                    reason: call.reason.clone(),
                    payload: call.payload.clone(),
                })
        })
        .take(MAX_TENTACLE_ACTIONS)
        .collect()
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleToolCandidate {
    pub id: String,
    pub description: String,
    pub runtime: String,
    pub entrypoint: String,
    pub contract: Option<String>,
    pub permission: Option<String>,
    pub authorization_required: bool,
    pub active_grant: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleToolAction {
    pub tool: TentacleToolCandidate,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleThinkingPlan {
    pub tentacle_id: String,
    pub brain_kind: String,
    pub brain_prompt: String,
    pub feedback_contract: Option<String>,
    pub context_policy: String,
    pub need: Need,
    pub selected_tool: TentacleToolCandidate,
    pub plan_source: String,
    pub reason: String,
    #[serde(default)]
    pub actions: Vec<TentacleToolAction>,
    pub candidates: Vec<TentacleToolCandidate>,
}

pub struct ManifestTentacle {
    route_id: String,
    installed: InstalledTentacle,
    tools: Vec<InstalledToolRef>,
    llm_factory: Option<ChatClientFactory>,
    grants: Vec<CapabilityGrant>,
}

impl ManifestTentacle {
    pub fn new(installed: InstalledTentacle) -> Self {
        Self::new_with_llm(installed, None)
    }

    pub fn new_with_llm(
        installed: InstalledTentacle,
        llm_config: Option<OpenAiCompatibleConfig>,
    ) -> Self {
        Self::new_with_llm_and_grants(
            installed,
            llm_config.map(chat_client_factory_from_openai_config),
            Vec::new(),
        )
    }

    pub fn new_with_llm_factory(
        installed: InstalledTentacle,
        llm_factory: Option<ChatClientFactory>,
    ) -> Self {
        Self::new_with_llm_and_grants(installed, llm_factory, Vec::new())
    }

    pub fn new_with_llm_and_grants(
        installed: InstalledTentacle,
        llm_factory: Option<ChatClientFactory>,
        grants: Vec<CapabilityGrant>,
    ) -> Self {
        let route_id = installed.id.clone();
        let tools = if installed.tool_meta.is_empty() {
            installed
                .tools
                .iter()
                .filter_map(|tool| InstalledToolRef::parse(tool))
                .collect()
        } else {
            installed
                .tool_meta
                .iter()
                .map(InstalledToolRef::from_installed)
                .collect()
        };
        Self {
            route_id,
            installed,
            tools,
            llm_factory,
            grants,
        }
    }

    fn plan_tool(&mut self, need: &Need) -> Option<ManifestToolPlan> {
        let tools = self.available_tools(need);
        if let Some(plan) = self.structured_field_mini_task_plan(need, &tools) {
            return Some(plan);
        }
        self.llm_plan_tool(need, &tools)
    }

    fn structured_field_mini_task_plan(
        &self,
        need: &Need,
        tools: &[InstalledToolRef],
    ) -> Option<ManifestToolPlan> {
        if self.installed.id != "field-mini-task" {
            return None;
        }
        let field = need.context.get("field_pack")?;
        let mini_task = need.context.get("field_mini_task")?;
        if field.trim().is_empty() || mini_task.trim().is_empty() {
            return None;
        }
        let tool = tools
            .iter()
            .find(|tool| tool.id == "run_field_mini_task")
            .cloned()?;
        let calls = vec![ManifestToolCall {
            tool: tool.clone(),
            reason: format!("run structured field mini task {field}/{mini_task}"),
            payload: BTreeMap::new(),
        }];
        Some(ManifestToolPlan {
            tool,
            reason: format!("structured Need selected field mini task {field}/{mini_task}"),
            candidates: tools.iter().map(|tool| tool.id.clone()).collect(),
            source: "structured-need".to_string(),
            calls,
        })
    }

    pub(crate) fn thinking_plan(&mut self, need: &Need) -> Option<TentacleThinkingPlan> {
        let plan = self.plan_tool(need)?;
        let candidate_ids = plan.candidates.clone();
        let actions = plan
            .calls
            .iter()
            .map(|call| TentacleToolAction {
                tool: self.thinking_candidate(&call.tool),
                reason: call.reason.clone(),
            })
            .collect::<Vec<_>>();
        let candidates = candidate_ids
            .iter()
            .filter_map(|id| self.tools.iter().find(|tool| &tool.id == id))
            .map(|tool| self.thinking_candidate(tool))
            .collect::<Vec<_>>();
        Some(TentacleThinkingPlan {
            tentacle_id: self.route_id.clone(),
            brain_kind: self.installed.brain_kind.clone(),
            brain_prompt: self.installed.brain_prompt.clone(),
            feedback_contract: self.installed.feedback_contract.clone(),
            context_policy: TENTACLE_CONTEXT_POLICY.to_string(),
            need: need.clone(),
            selected_tool: self.thinking_candidate(&plan.tool),
            plan_source: plan.source,
            reason: plan.reason,
            actions,
            candidates,
        })
    }

    fn available_tools(&self, _need: &Need) -> Vec<InstalledToolRef> {
        self.tools
            .iter()
            .filter(|tool| self.can_feed_tool(tool))
            .cloned()
            .collect()
    }

    fn llm_plan_tool(
        &mut self,
        need: &Need,
        tools: &[InstalledToolRef],
    ) -> Option<ManifestToolPlan> {
        let factory = self.llm_factory.clone()?;
        if tools.is_empty() {
            return None;
        }
        let tool_sheet = tools
            .iter()
            .map(|tool| {
                format!(
                    "- {}: {} | runtime={} | contract={} | permission={} | input={} | output={}",
                    tool.id,
                    tool.description,
                    tool.kind,
                    tool_contract_label(tool),
                    tool_permission_label(tool.permission.as_ref()),
                    tool.input,
                    tool.output
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let need_context =
            serde_json::to_string(&need.context).unwrap_or_else(|_| "{}".to_string());
        let mut client = (factory)().ok()?;
        let mut messages = vec![
            ChatMessage::new(
                ChatRole::System,
                format!(
                    "You are the '{}' Octopus tentacle brain. {}\nReturn only JSON: {{\"calls\":[{{\"tool\":\"id\",\"reason\":\"short\",\"payload\":{{\"query\":\"optional per-action input\"}}}}],\"summary\":\"short\"}}",
                    self.route_id, self.installed.brain_prompt
                ),
            ),
            ChatMessage::new(
                ChatRole::User,
                format!(
                    "Need: {}: {}\nNeed context JSON: {}\nAvailable tools:\n{}",
                    kind_key(&need.kind),
                    need.query,
                    need_context,
                    tool_sheet
                ),
            ),
        ];
        let response = client.chat(&messages).ok()?;
        let mut plan = plan_from_llm_content(&response.content).unwrap_or_else(|error| Plan {
            calls: Vec::new(),
            summary: error,
        });
        let mut calls = manifest_tool_calls_from_plan(&plan, tools);
        let mut source = "llm".to_string();
        if calls.is_empty() {
            messages.push(ChatMessage::new(ChatRole::Assistant, response.content));
            messages.push(ChatMessage::new(
                ChatRole::User,
                format!(
                    "The previous response did not select a valid tool from [{}]. Return only valid Plan JSON with at least one call.",
                    tools
                        .iter()
                        .map(|tool| tool.id.as_str())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            ));
            if let Ok(retry_response) = client.chat(&messages) {
                if let Ok(retry_plan) = plan_from_llm_content(&retry_response.content) {
                    let retry_calls = manifest_tool_calls_from_plan(&retry_plan, tools);
                    if !retry_calls.is_empty() {
                        plan = retry_plan;
                        calls = retry_calls;
                        source = "llm-retry".to_string();
                    }
                }
            }
        }
        let first = calls.first()?;
        let tool = first.tool.clone();
        let candidates = tools.iter().map(|tool| tool.id.clone()).collect::<Vec<_>>();
        let reason = if calls.len() > 1 {
            let steps = calls
                .iter()
                .map(|call| {
                    if call.reason.trim().is_empty() {
                        call.tool.id.clone()
                    } else {
                        format!("{}({})", call.tool.id, call.reason)
                    }
                })
                .collect::<Vec<_>>()
                .join(" -> ");
            if plan.summary.trim().is_empty() {
                format!(
                    "llm planned {} actions for {}",
                    calls.len(),
                    kind_key(&need.kind)
                )
            } else {
                format!(
                    "llm planned {} actions for {}: {} :: {}",
                    calls.len(),
                    kind_key(&need.kind),
                    plan.summary,
                    steps
                )
            }
        } else if first.reason.is_empty() {
            format!("llm selected {} for {}", tool.id, kind_key(&need.kind))
        } else {
            format!(
                "llm selected {} for {}: {}",
                tool.id,
                kind_key(&need.kind),
                first.reason
            )
        };
        Some(ManifestToolPlan {
            tool,
            reason,
            candidates,
            source,
            calls,
        })
    }

    fn tool_path(&self, tool: &InstalledToolRef) -> PathBuf {
        let entrypoint = Path::new(&tool.entrypoint);
        if entrypoint.is_absolute() {
            return entrypoint.to_path_buf();
        }
        Path::new(&self.installed.source)
            .parent()
            .map(|parent| parent.join(entrypoint))
            .unwrap_or_else(|| entrypoint.to_path_buf())
    }

    fn can_feed_tool(&self, tool: &InstalledToolRef) -> bool {
        tool.kind == "static-html"
            || (tool.kind == "http"
                && (tool.entrypoint.starts_with("https://")
                    || tool.entrypoint.starts_with("http://")))
            || self.tool_path(tool).exists()
    }

    fn run_tool(&self, call: &ManifestToolCall, need: &Need) -> ToolResult {
        let tool = &call.tool;
        let active_grant = match self.authorize_tool(tool) {
            Ok(grant) => grant.map(|grant| grant.id.clone()),
            Err(result) => return result,
        };
        let mut action_need = need.clone();
        if let Some(query) = tool_call_query(call) {
            action_need.query = query.to_string();
        }
        let path = self.tool_path(tool);
        if tool.kind == "static-html" {
            let target = if path.exists() {
                path.to_string_lossy().to_string()
            } else {
                tool.entrypoint.clone()
            };
            let mut result = ToolResult::satisfied(&tool.id, format!("static artifact: {target}"));
            result.metadata.insert("entrypoint".to_string(), target);
            result
                .metadata
                .insert("runtime".to_string(), tool.kind.clone());
            if let Some(contract) = tool.contract.as_ref() {
                result
                    .metadata
                    .insert("contract".to_string(), contract.clone());
            }
            self.record_tool_permission_metadata(tool, active_grant.as_deref(), &mut result);
            return result;
        }
        if tool.kind != "http" && !path.exists() {
            return ToolResult::failed(format!(
                "missing {} tool entrypoint: {}",
                tool.kind,
                path.display()
            ));
        }

        let uses_json_contract =
            tool.contract.as_deref() == Some(OCTOPUS_JSON_CONTRACT) || tool.kind == "http";
        let mut command = match tool_command(tool, &path) {
            Ok(command) => command,
            Err(error) => return ToolResult::failed(error),
        };
        let stdin_input = tool_stdin_input(call, &action_need.query);
        if uses_json_contract {
            command.stdin(Stdio::piped());
        } else {
            for arg in tool_argv(call, tool, &action_need.query) {
                command.arg(arg);
            }
            if stdin_input.is_some() {
                command.stdin(Stdio::piped());
            }
        }
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                return ToolResult::failed(format!("{} failed to start: {error}", tool.id))
            }
        };
        if uses_json_contract {
            let input = match self.tool_invocation_json(tool, &action_need) {
                Ok(input) => input,
                Err(error) => return ToolResult::failed(error),
            };
            if let Some(mut stdin) = child.stdin.take() {
                if let Err(error) = stdin.write_all(input.as_bytes()) {
                    return ToolResult::failed(format!("{} stdin failed: {error}", tool.id));
                }
            }
        } else if let Some(input) = stdin_input {
            if let Some(mut stdin) = child.stdin.take() {
                if let Err(error) = stdin.write_all(input.as_bytes()) {
                    return ToolResult::failed(format!("{} stdin failed: {error}", tool.id));
                }
            }
        }
        let output = match child.wait_with_output() {
            Ok(output) => output,
            Err(error) => return ToolResult::failed(format!("{} failed: {error}", tool.id)),
        };
        let raw_text = String::from_utf8_lossy(if output.stdout.is_empty() {
            &output.stderr
        } else {
            &output.stdout
        })
        .to_string();
        let text = trim_output(&raw_text);
        let mut result = if output.status.success() {
            if uses_json_contract {
                tool_result_from_json(&tool.id, raw_text.trim())
                    .or_else(|| tool_result_from_json(&tool.id, &text))
                    .unwrap_or_else(|| ToolResult::satisfied(&tool.id, text))
            } else {
                ToolResult::satisfied(&tool.id, text)
            }
        } else {
            ToolResult::failed(text)
        };
        result.metadata.insert("tool".to_string(), tool.id.clone());
        result
            .metadata
            .insert("entrypoint".to_string(), path.to_string_lossy().to_string());
        result
            .metadata
            .insert("runtime".to_string(), tool.kind.clone());
        if action_need.query != need.query {
            result
                .metadata
                .insert("action_query".to_string(), action_need.query);
        }
        if let Some(contract) = tool_result_contract(tool, uses_json_contract) {
            result.metadata.insert("contract".to_string(), contract);
        }
        self.record_tool_permission_metadata(tool, active_grant.as_deref(), &mut result);
        result.metadata.insert(
            "returncode".to_string(),
            exit_code(&output.status).to_string(),
        );
        result
    }

    fn authorize_tool<'a>(
        &'a self,
        tool: &InstalledToolRef,
    ) -> Result<Option<&'a CapabilityGrant>, ToolResult> {
        let Some(permission) = &tool.permission else {
            return Ok(None);
        };
        match active_grant_for_tool(tool, &self.grants) {
            Some(grant) => Ok(Some(grant)),
            None => Err(tool_authorization_result(tool, permission)),
        }
    }

    fn record_tool_permission_metadata(
        &self,
        tool: &InstalledToolRef,
        active_grant: Option<&str>,
        result: &mut ToolResult,
    ) {
        let Some(permission) = &tool.permission else {
            return;
        };
        result
            .metadata
            .insert("authorization_required".to_string(), "true".to_string());
        result.metadata.insert(
            "permission_provider".to_string(),
            permission.provider.clone(),
        );
        result
            .metadata
            .insert("permission_scope".to_string(), permission.scope.clone());
        result.metadata.insert(
            "required_permissions".to_string(),
            permission.permissions.join(","),
        );
        if let Some(reason) = &permission.reason {
            result
                .metadata
                .insert("permission_reason".to_string(), reason.clone());
        }
        if let Some(grant) = active_grant {
            result
                .metadata
                .insert("active_grant".to_string(), grant.to_string());
        }
    }

    fn thinking_candidate(&self, tool: &InstalledToolRef) -> TentacleToolCandidate {
        let active_grant = active_grant_for_tool(tool, &self.grants).map(|grant| grant.id.clone());
        TentacleToolCandidate {
            id: tool.id.clone(),
            description: tool.description.clone(),
            runtime: tool.kind.clone(),
            entrypoint: tool.entrypoint.clone(),
            contract: tool.contract.clone(),
            permission: tool
                .permission
                .as_ref()
                .map(|permission| tool_permission_label(Some(permission))),
            authorization_required: tool.permission.is_some(),
            active_grant,
        }
    }

    fn tool_invocation_json(&self, tool: &InstalledToolRef, need: &Need) -> Result<String, String> {
        serde_json::to_string(&ToolInvocation {
            schema_version: OCTOPUS_TOOL_CALL_SCHEMA,
            need,
            tool: ToolInvocationTool {
                id: &tool.id,
                description: &tool.description,
                input: &tool.input,
                output: &tool.output,
                runtime: &tool.kind,
                entrypoint: &tool.entrypoint,
                contract: tool.contract.as_deref(),
                permission: tool.permission.as_ref(),
            },
            tentacle: ToolInvocationTentacle {
                id: &self.installed.id,
                brain_kind: &self.installed.brain_kind,
                brain_prompt: &self.installed.brain_prompt,
                feedback_contract: self.installed.feedback_contract.as_deref(),
            },
        })
        .map_err(|error| error.to_string())
    }

    fn plan_evidence(&self, tool: &InstalledToolRef, plan: &ManifestToolPlan) -> Evidence {
        let mut evidence = Evidence::new(
            "tentacle_plan",
            format!(
                "{} selected {} with {} planning",
                self.route_id, tool.id, plan.source
            ),
        );
        evidence.confidence = if plan.source == "llm" { 0.9 } else { 0.7 };
        evidence.metadata.insert(
            "context_policy".to_string(),
            TENTACLE_CONTEXT_POLICY.to_string(),
        );
        evidence
            .metadata
            .insert("tentacle".to_string(), self.route_id.clone());
        evidence
            .metadata
            .insert("tool".to_string(), tool.id.clone());
        evidence
            .metadata
            .insert("plan_source".to_string(), plan.source.clone());
        evidence
            .metadata
            .insert("reason".to_string(), plan.reason.clone());
        evidence
            .metadata
            .insert("available_tools".to_string(), plan.candidates.join(","));
        evidence.metadata.insert(
            "tools".to_string(),
            plan.calls
                .iter()
                .map(|call| call.tool.id.as_str())
                .collect::<Vec<_>>()
                .join(","),
        );
        evidence
            .metadata
            .insert("action_count".to_string(), plan.calls.len().to_string());
        evidence
    }
}

fn tool_call_query(call: &ManifestToolCall) -> Option<String> {
    call.payload
        .get("query")
        .or_else(|| call.payload.get("input"))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn tool_stdin_input(call: &ManifestToolCall, default_query: &str) -> Option<String> {
    for key in ["stdin", "script", "patch", "content", "body"] {
        if let Some(value) = call
            .payload
            .get(key)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            return Some(value);
        }
    }
    if tool_declares_stdin(&call.tool) {
        Some(default_tool_query(default_query))
    } else {
        None
    }
}

fn tool_argv(call: &ManifestToolCall, tool: &InstalledToolRef, default_query: &str) -> Vec<String> {
    for key in ["argv", "args"] {
        if let Some(value) = call.payload.get(key) {
            let args = split_tool_args(value);
            if !args.is_empty() {
                return args;
            }
        }
    }
    let numbered = (1..=16)
        .filter_map(|index| {
            call.payload
                .get(&format!("arg{index}"))
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .collect::<Vec<_>>();
    if !numbered.is_empty() {
        return numbered;
    }
    if tool_declares_stdin(tool) || tool_declares_no_input(tool) {
        return Vec::new();
    }
    tool_call_query(call)
        .map(|query| split_tool_args(&query))
        .unwrap_or_else(|| split_tool_args(default_query))
}

fn tool_declares_stdin(tool: &InstalledToolRef) -> bool {
    tool.input.to_ascii_lowercase().contains("stdin")
}

fn tool_declares_no_input(tool: &InstalledToolRef) -> bool {
    matches!(
        tool.input.trim().to_ascii_lowercase().as_str(),
        "none" | "no input"
    )
}

fn split_tool_args(value: &str) -> Vec<String> {
    let value = value.trim();
    if value.is_empty() {
        return Vec::new();
    }
    if let Ok(args) = serde_json::from_str::<Vec<String>>(value) {
        return args
            .into_iter()
            .map(|arg| arg.trim().to_string())
            .filter(|arg| !arg.is_empty())
            .collect();
    }
    value.split_whitespace().map(str::to_string).collect()
}

fn action_results_status(results: &[(ManifestToolCall, ToolResult)]) -> Status {
    if results.is_empty() {
        return Status::Failed;
    }
    let tool_results = results
        .iter()
        .map(|(_, result)| result.clone())
        .collect::<Vec<_>>();
    tool_status(&tool_results)
}

fn action_results_summary(results: &[(ManifestToolCall, ToolResult)]) -> String {
    if results.is_empty() {
        return "no tentacle action executed".to_string();
    }
    if results.len() == 1 {
        return results[0].1.output.clone();
    }
    results
        .iter()
        .enumerate()
        .map(|(index, (call, result))| {
            format!(
                "== action {}: {} ({:?}) ==\n{}",
                index + 1,
                call.tool.id,
                result.status,
                result.output
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn action_results_evidence(results: &[(ManifestToolCall, ToolResult)]) -> Vec<Evidence> {
    let mut evidence = Vec::new();
    for (index, (call, result)) in results.iter().enumerate() {
        let mut action = Evidence::new(
            "tentacle_action",
            format!(
                "action {} ran {} -> {:?}",
                index + 1,
                call.tool.id,
                result.status
            ),
        );
        action.confidence = if result.status == Status::Satisfied {
            0.9
        } else {
            0.4
        };
        action
            .metadata
            .insert("tool".to_string(), call.tool.id.clone());
        action
            .metadata
            .insert("reason".to_string(), call.reason.clone());
        action
            .metadata
            .insert("status".to_string(), status_key(&result.status).to_string());
        evidence.push(action);
        evidence.extend(result.evidence.clone());
    }
    evidence
}

impl Tentacle for ManifestTentacle {
    fn name(&self) -> &str {
        &self.route_id
    }

    fn supports(&self, need: &Need) -> bool {
        if !self
            .installed
            .needs
            .iter()
            .any(|need_key| need_key == kind_key(&need.kind))
        {
            return false;
        }
        let tools = self.available_tools(need);
        !tools.is_empty()
            && (self.llm_factory.is_some()
                || self.structured_field_mini_task_plan(need, &tools).is_some())
    }

    fn feed(&mut self, need: &Need) -> Feed {
        let Some(plan) = self.plan_tool(need) else {
            return Feed::unsupported(need, "no manifest tool supports this need");
        };
        let tool = plan.tool.clone();
        let mut action_results = Vec::new();
        for call in plan.calls.iter().take(MAX_TENTACLE_ACTIONS) {
            let result = self.run_tool(call, need);
            let should_continue = result.status == Status::Satisfied;
            action_results.push((call.clone(), result));
            if !should_continue {
                break;
            }
        }
        let status = action_results_status(&action_results);
        let mut metadata = action_results
            .last()
            .map(|(_, result)| result.metadata.clone())
            .unwrap_or_default();
        metadata.insert("tool".to_string(), tool.id.clone());
        metadata.insert("tool_description".to_string(), tool.description.clone());
        metadata.insert(
            "tools".to_string(),
            action_results
                .iter()
                .map(|(call, _)| call.tool.id.as_str())
                .collect::<Vec<_>>()
                .join(","),
        );
        metadata.insert("action_count".to_string(), action_results.len().to_string());
        metadata.insert(
            "action_plan".to_string(),
            plan.calls
                .iter()
                .map(|call| call.tool.id.as_str())
                .collect::<Vec<_>>()
                .join(" -> "),
        );
        metadata.insert("plan".to_string(), plan.reason.clone());
        metadata.insert("plan_source".to_string(), plan.source.clone());
        metadata.insert("available_tools".to_string(), plan.candidates.join(","));
        metadata.insert(
            "context_policy".to_string(),
            TENTACLE_CONTEXT_POLICY.to_string(),
        );
        metadata.insert("runtime".to_string(), tool.kind.clone());
        metadata.insert("tentacle_brain".to_string(), self.route_id.clone());
        metadata.insert(
            "brain_prompt".to_string(),
            self.installed.brain_prompt.clone(),
        );
        if let Some(contract) = &self.installed.feedback_contract {
            metadata.insert("feedback_contract".to_string(), contract.clone());
        }
        let mut evidence = action_results_evidence(&action_results);
        evidence.insert(0, self.plan_evidence(&tool, &plan));
        Feed {
            need: need.clone(),
            status,
            evidence,
            summary: action_results_summary(&action_results),
            metadata,
        }
    }
}

fn tool_command(tool: &InstalledToolRef, path: &Path) -> Result<Command, String> {
    match tool.kind.as_str() {
        "http" => {
            if !(tool.entrypoint.starts_with("https://") || tool.entrypoint.starts_with("http://"))
            {
                return Err(format!("http tool requires URL entrypoint: {}", tool.id));
            }
            let mut command = Command::new("curl");
            command
                .arg("-sS")
                .arg("-X")
                .arg("POST")
                .arg("-H")
                .arg("content-type: application/json")
                .arg("--data-binary")
                .arg("@-")
                .arg(&tool.entrypoint);
            Ok(command)
        }
        "python" => {
            let mut command = Command::new("python3");
            command.arg(path);
            Ok(command)
        }
        "node" | "javascript" => {
            let mut command = Command::new("node");
            command.arg(path);
            Ok(command)
        }
        _ => Ok(Command::new(path)),
    }
}

pub(crate) fn tool_result_from_json(tool_id: &str, text: &str) -> Option<ToolResult> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if let Ok(result) = serde_json::from_str::<ToolResult>(text) {
        return Some(result);
    }
    let envelope = serde_json::from_str::<ToolOutputEnvelope>(text).ok()?;
    let evidence = if envelope.evidence.is_empty() {
        vec![Evidence::new(tool_id, envelope.output.clone())]
    } else {
        envelope.evidence
    };
    Some(ToolResult {
        status: envelope.status.unwrap_or(Status::Satisfied),
        output: envelope.output,
        evidence,
        metadata: envelope.metadata,
    })
}

fn tool_authorization_result(tool: &InstalledToolRef, permission: &ToolPermission) -> ToolResult {
    let permissions = permission.permissions.join(" ");
    let hint = if permissions.is_empty() {
        format!("octopus oauth {} {}", permission.provider, permission.scope)
    } else {
        format!(
            "octopus oauth {} {} {}",
            permission.provider, permission.scope, permissions
        )
    };
    let mut result = ToolResult::failed(format!(
        "needs_authorization: {} requires {}. run: {}",
        tool.id,
        tool_permission_label(Some(permission)),
        hint
    ));
    result.metadata.insert("tool".to_string(), tool.id.clone());
    result
        .metadata
        .insert("authorization_required".to_string(), "true".to_string());
    result.metadata.insert(
        "permission_provider".to_string(),
        permission.provider.clone(),
    );
    result
        .metadata
        .insert("permission_scope".to_string(), permission.scope.clone());
    result.metadata.insert(
        "required_permissions".to_string(),
        permission.permissions.join(","),
    );
    result.metadata.insert("grant_hint".to_string(), hint);
    if let Some(reason) = &permission.reason {
        result
            .metadata
            .insert("permission_reason".to_string(), reason.clone());
    }
    result
}

fn tool_permission_label(permission: Option<&ToolPermission>) -> String {
    match permission {
        Some(permission) if permission.permissions.is_empty() => {
            format!("{}:{}", permission.provider, permission.scope)
        }
        Some(permission) => format!(
            "{}:{} [{}]",
            permission.provider,
            permission.scope,
            permission.permissions.join(",")
        ),
        None => "none".to_string(),
    }
}

pub(crate) fn active_grant_for_tool<'a>(
    tool: &InstalledToolRef,
    grants: &'a [CapabilityGrant],
) -> Option<&'a CapabilityGrant> {
    let permission = tool.permission.as_ref()?;
    grants.iter().find(|grant| {
        grant.status == GrantStatus::Active
            && grant.provider == permission.provider
            && grant.scope == permission.scope
            && permission
                .permissions
                .iter()
                .all(|required| grant.permissions.iter().any(|owned| owned == required))
    })
}
