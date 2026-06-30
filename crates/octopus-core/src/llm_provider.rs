use super::{kind_key, trim_output, Need, Plan, Planner, ToolSpec};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

impl ChatMessage {
    pub fn new(role: ChatRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChatResponse {
    pub content: String,
    pub metadata: BTreeMap<String, String>,
}

pub trait ChatClient {
    fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String>;
}

pub type ChatClientFactory = Arc<dyn Fn() -> Result<Box<dyn ChatClient>, String>>;

impl<T> ChatClient for Box<T>
where
    T: ChatClient + ?Sized,
{
    fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
        (**self).chat(messages)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct OpenAiCompatibleConfig {
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: String,
    pub timeout_seconds: u64,
    pub retry_count: u32,
    pub retry_delay_ms: u64,
    pub curl_command: String,
    #[serde(default)]
    pub tuning: OpenAiCompatibleTuning,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct OpenAiCompatibleTuning {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra_body: BTreeMap<String, serde_json::Value>,
}

impl OpenAiCompatibleConfig {
    pub fn from_env() -> Result<Self, String> {
        Self::from_env_prefix("OCTOPUS_LLM")
    }

    pub fn from_env_prefix(prefix: &str) -> Result<Self, String> {
        let model_key = format!("{prefix}_MODEL");
        let model = env::var(&model_key)
            .map_err(|_| format!("{model_key} is required for provider chat"))?;
        let api_key = env::var(format!("{prefix}_API_KEY"))
            .ok()
            .filter(|value| !value.is_empty());
        let base_url = env::var(format!("{prefix}_BASE_URL"))
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let timeout_seconds = env::var(format!("{prefix}_TIMEOUT"))
            .ok()
            .map(|value| {
                value
                    .parse::<u64>()
                    .map_err(|_| format!("{prefix}_TIMEOUT must be an integer"))
            })
            .transpose()?
            .unwrap_or(60);
        let retry_count = env::var(format!("{prefix}_RETRIES"))
            .ok()
            .map(|value| {
                value
                    .parse::<u32>()
                    .map_err(|_| format!("{prefix}_RETRIES must be an integer"))
            })
            .transpose()?
            .unwrap_or(1);
        let retry_delay_ms = env::var(format!("{prefix}_RETRY_MS"))
            .ok()
            .map(|value| {
                value
                    .parse::<u64>()
                    .map_err(|_| format!("{prefix}_RETRY_MS must be an integer"))
            })
            .transpose()?
            .unwrap_or(750);
        let curl_command =
            env::var(format!("{prefix}_CURL")).unwrap_or_else(|_| "curl".to_string());
        let tuning = OpenAiCompatibleTuning::from_env_prefix(prefix)?;
        Ok(Self {
            model,
            api_key,
            base_url,
            timeout_seconds,
            retry_count,
            retry_delay_ms,
            curl_command,
            tuning,
        })
    }

    pub fn endpoint(&self) -> String {
        format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
    }
}

impl OpenAiCompatibleTuning {
    fn from_env_prefix(prefix: &str) -> Result<Self, String> {
        Ok(Self {
            temperature: optional_f64_env(prefix, "TEMPERATURE")?,
            top_p: optional_f64_env(prefix, "TOP_P")?,
            max_tokens: optional_u64_env(prefix, "MAX_TOKENS")?,
            reasoning_effort: env::var(format!("{prefix}_REASONING_EFFORT"))
                .ok()
                .filter(|value| !value.trim().is_empty()),
            extra_body: optional_json_object_env(prefix, "EXTRA_BODY")?,
        })
    }
}

fn optional_f64_env(prefix: &str, suffix: &str) -> Result<Option<f64>, String> {
    let key = format!("{prefix}_{suffix}");
    env::var(&key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            let parsed = value
                .parse::<f64>()
                .map_err(|_| format!("{key} must be a number"))?;
            if parsed.is_finite() {
                Ok(parsed)
            } else {
                Err(format!("{key} must be finite"))
            }
        })
        .transpose()
}

fn optional_u64_env(prefix: &str, suffix: &str) -> Result<Option<u64>, String> {
    let key = format!("{prefix}_{suffix}");
    env::var(&key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|_| format!("{key} must be an integer"))
        })
        .transpose()
}

fn optional_json_object_env(
    prefix: &str,
    suffix: &str,
) -> Result<BTreeMap<String, serde_json::Value>, String> {
    let key = format!("{prefix}_{suffix}");
    let Some(payload) = env::var(&key).ok().filter(|value| !value.trim().is_empty()) else {
        return Ok(BTreeMap::new());
    };
    match serde_json::from_str::<serde_json::Value>(&payload)
        .map_err(|error| format!("{key} must be JSON: {error}"))?
    {
        serde_json::Value::Object(object) => Ok(object.into_iter().collect()),
        _ => Err(format!("{key} must be a JSON object")),
    }
}

pub struct OpenAiCompatibleChatClient {
    config: OpenAiCompatibleConfig,
}

impl OpenAiCompatibleChatClient {
    pub fn new(config: OpenAiCompatibleConfig) -> Self {
        Self { config }
    }

    pub fn from_env() -> Result<Self, String> {
        Ok(Self::new(OpenAiCompatibleConfig::from_env()?))
    }

    pub(crate) fn request_body(&self, messages: &[ChatMessage]) -> Result<String, String> {
        let mut body = serde_json::Map::new();
        for (key, value) in &self.config.tuning.extra_body {
            body.insert(key.clone(), value.clone());
        }
        body.insert(
            "model".to_string(),
            serde_json::Value::String(self.config.model.clone()),
        );
        body.insert(
            "messages".to_string(),
            serde_json::to_value(messages).map_err(|error| error.to_string())?,
        );
        if let Some(temperature) = self.config.tuning.temperature {
            body.insert(
                "temperature".to_string(),
                serde_json::Number::from_f64(temperature)
                    .map(serde_json::Value::Number)
                    .ok_or_else(|| "temperature must be finite".to_string())?,
            );
        }
        if let Some(top_p) = self.config.tuning.top_p {
            body.insert(
                "top_p".to_string(),
                serde_json::Number::from_f64(top_p)
                    .map(serde_json::Value::Number)
                    .ok_or_else(|| "top_p must be finite".to_string())?,
            );
        }
        if let Some(max_tokens) = self.config.tuning.max_tokens {
            body.insert(
                "max_tokens".to_string(),
                serde_json::Value::Number(max_tokens.into()),
            );
        }
        if let Some(reasoning_effort) = &self.config.tuning.reasoning_effort {
            body.insert(
                "reasoning_effort".to_string(),
                serde_json::Value::String(reasoning_effort.clone()),
            );
        }
        body.entry("stream".to_string())
            .or_insert(serde_json::Value::Bool(false));
        serde_json::to_string(&serde_json::Value::Object(body)).map_err(|error| error.to_string())
    }

    fn chat_once(&self, body: &str) -> Result<ChatResponse, String> {
        let mut command = Command::new(&self.config.curl_command);
        command
            .arg("-sS")
            .arg("--max-time")
            .arg(self.config.timeout_seconds.to_string())
            .arg("-X")
            .arg("POST")
            .arg(self.config.endpoint())
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("--data-binary")
            .arg(body);
        if let Some(api_key) = &self.config.api_key {
            command
                .arg("-H")
                .arg(format!("Authorization: Bearer {api_key}"));
        }
        let output = command.output().map_err(|error| {
            format!(
                "{} failed to start: {error}",
                self.config.curl_command.as_str()
            )
        })?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success() {
            let message = if stderr.trim().is_empty() {
                stdout
            } else {
                stderr
            };
            return Err(trim_output(&message));
        }
        let data = serde_json::from_str::<serde_json::Value>(&stdout)
            .map_err(|error| format!("invalid chat response JSON: {error}"))?;
        if let Some(error) = data.get("error") {
            return Err(chat_error_message(error));
        }
        let content = extract_chat_content(&data)?;
        Ok(ChatResponse {
            content,
            metadata: BTreeMap::from([
                ("provider".to_string(), "openai-compatible".to_string()),
                ("model".to_string(), self.config.model.clone()),
                ("base_url".to_string(), self.config.base_url.clone()),
            ]),
        })
    }
}

impl ChatClient for OpenAiCompatibleChatClient {
    fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
        let body = self.request_body(messages)?;
        let mut last_error = String::new();
        for attempt in 0..=self.config.retry_count {
            match self.chat_once(&body) {
                Ok(response) => return Ok(response),
                Err(error) => {
                    last_error = error;
                    if attempt < self.config.retry_count {
                        thread::sleep(Duration::from_millis(self.config.retry_delay_ms));
                    }
                }
            }
        }
        Err(last_error)
    }
}

pub fn chat_client_factory_from_openai_config(config: OpenAiCompatibleConfig) -> ChatClientFactory {
    Arc::new(move || Ok(Box::new(OpenAiCompatibleChatClient::new(config.clone()))))
}

fn chat_error_message(error: &serde_json::Value) -> String {
    error
        .get("message")
        .and_then(serde_json::Value::as_str)
        .or_else(|| error.get("code").and_then(serde_json::Value::as_str))
        .map(|value| format!("provider error: {value}"))
        .unwrap_or_else(|| format!("provider error: {error}"))
}

fn extract_chat_content(data: &serde_json::Value) -> Result<String, String> {
    let content = data.pointer("/choices/0/message/content");
    if let Some(text) = content.and_then(serde_json::Value::as_str) {
        let text = text.trim().to_string();
        if !text.is_empty() {
            return Ok(text);
        }
    }
    if let Some(parts) = content.and_then(serde_json::Value::as_array) {
        let text = parts
            .iter()
            .filter_map(|part| {
                part.get("text")
                    .or_else(|| part.get("content"))
                    .and_then(serde_json::Value::as_str)
            })
            .collect::<Vec<_>>()
            .join("");
        let text = text.trim().to_string();
        if !text.is_empty() {
            return Ok(text);
        }
    }
    if let Some(text) = data
        .pointer("/choices/0/text")
        .and_then(serde_json::Value::as_str)
    {
        let text = text.trim().to_string();
        if !text.is_empty() {
            return Ok(text);
        }
    }
    if data
        .pointer("/choices/0/message/reasoning_content")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
    {
        return Err(
            "chat response content is empty after reasoning_content; increase max_tokens or disable thinking"
                .to_string(),
        );
    }
    Err("chat response missing non-empty choices[0].message.content".to_string())
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CodexCliConfig {
    pub model: Option<String>,
    pub command: String,
    pub timeout_seconds: u64,
    pub retry_count: u32,
    pub retry_delay_ms: u64,
}

impl CodexCliConfig {
    pub fn from_env_prefix(prefix: &str) -> Result<Self, String> {
        let model = env::var(format!("{prefix}_MODEL"))
            .ok()
            .filter(|value| !value.trim().is_empty());
        let command =
            env::var(format!("{prefix}_CODEX_COMMAND")).unwrap_or_else(|_| "codex".to_string());
        let timeout_seconds = env::var(format!("{prefix}_TIMEOUT"))
            .ok()
            .map(|value| {
                value
                    .parse::<u64>()
                    .map_err(|_| format!("{prefix}_TIMEOUT must be an integer"))
            })
            .transpose()?
            .unwrap_or(120);
        let retry_count = env::var(format!("{prefix}_RETRIES"))
            .ok()
            .map(|value| {
                value
                    .parse::<u32>()
                    .map_err(|_| format!("{prefix}_RETRIES must be an integer"))
            })
            .transpose()?
            .unwrap_or(1);
        let retry_delay_ms = env::var(format!("{prefix}_RETRY_MS"))
            .ok()
            .map(|value| {
                value
                    .parse::<u64>()
                    .map_err(|_| format!("{prefix}_RETRY_MS must be an integer"))
            })
            .transpose()?
            .unwrap_or(750);
        Ok(Self {
            model,
            command,
            timeout_seconds,
            retry_count,
            retry_delay_ms,
        })
    }
}

pub struct CodexCliChatClient {
    config: CodexCliConfig,
}

impl CodexCliChatClient {
    pub fn new(config: CodexCliConfig) -> Self {
        Self { config }
    }

    fn prompt(messages: &[ChatMessage]) -> String {
        let mut prompt = String::from(
            "You are an Octopus LLM provider. Answer only the user's requested content. Do not run tools unless explicitly requested.\n\n",
        );
        for message in messages {
            let role = match message.role {
                ChatRole::System => "system",
                ChatRole::User => "user",
                ChatRole::Assistant => "assistant",
                ChatRole::Tool => "tool",
            };
            prompt.push_str(role);
            prompt.push_str(": ");
            prompt.push_str(&message.content);
            prompt.push('\n');
        }
        prompt
    }
}

impl ChatClient for CodexCliChatClient {
    fn chat(&mut self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
        let mut last_error = String::new();
        for attempt in 0..=self.config.retry_count {
            match self.chat_once(messages) {
                Ok(response) => return Ok(response),
                Err(error) => {
                    last_error = error;
                    if attempt < self.config.retry_count {
                        thread::sleep(Duration::from_millis(self.config.retry_delay_ms));
                    }
                }
            }
        }
        Err(last_error)
    }
}

impl CodexCliChatClient {
    fn chat_once(&self, messages: &[ChatMessage]) -> Result<ChatResponse, String> {
        let temp_suffix = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_millis())
                .unwrap_or_default()
        );
        let output_path = env::temp_dir().join(format!("octopus-codex-provider-{temp_suffix}.txt"));
        let prompt_path =
            env::temp_dir().join(format!("octopus-codex-provider-{temp_suffix}.prompt"));
        let stdout_path =
            env::temp_dir().join(format!("octopus-codex-provider-{temp_suffix}.stdout"));
        let stderr_path =
            env::temp_dir().join(format!("octopus-codex-provider-{temp_suffix}.stderr"));
        let prompt = Self::prompt(messages);
        fs::write(&prompt_path, prompt)
            .map_err(|error| format!("failed to write codex prompt file: {error}"))?;
        let mut command = Command::new(&self.config.command);
        let prompt_file = fs::File::open(&prompt_path)
            .map_err(|error| format!("failed to open codex prompt file: {error}"))?;
        let stdout_file = fs::File::create(&stdout_path)
            .map_err(|error| format!("failed to create codex stdout capture: {error}"))?;
        let stderr_file = fs::File::create(&stderr_path)
            .map_err(|error| format!("failed to create codex stderr capture: {error}"))?;
        command
            .arg("exec")
            .arg("--ephemeral")
            .arg("--skip-git-repo-check")
            .arg("--disable")
            .arg("image_generation")
            .arg("--sandbox")
            .arg("read-only")
            .arg("--output-last-message")
            .arg(&output_path);
        if let Some(model) = &self.config.model {
            command.arg("--model").arg(model);
        }
        let mut child = command
            .arg("-")
            .stdin(Stdio::from(prompt_file))
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .spawn()
            .map_err(|error| format!("{} failed to start: {error}", self.config.command))?;
        let deadline = Instant::now() + Duration::from_secs(self.config.timeout_seconds.max(1));
        let status = loop {
            if let Some(status) = child
                .try_wait()
                .map_err(|error| format!("{} failed: {error}", self.config.command))?
            {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                let _ = fs::remove_file(&prompt_path);
                let _ = fs::remove_file(&output_path);
                let _ = fs::remove_file(&stdout_path);
                let _ = fs::remove_file(&stderr_path);
                return Err(format!(
                    "codex provider timed out after {}s",
                    self.config.timeout_seconds.max(1)
                ));
            }
            thread::sleep(Duration::from_millis(50));
        };
        let _ = fs::remove_file(&prompt_path);
        let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
        let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
        let _ = fs::remove_file(&stdout_path);
        let _ = fs::remove_file(&stderr_path);
        if !status.success() {
            let message = if stderr.trim().is_empty() {
                stdout
            } else {
                stderr
            };
            let _ = fs::remove_file(&output_path);
            return Err(trim_output(&message));
        }
        let content = fs::read_to_string(&output_path)
            .unwrap_or_else(|_| stdout.clone())
            .trim()
            .to_string();
        let _ = fs::remove_file(&output_path);
        if content.is_empty() {
            return Err("codex provider returned empty content".to_string());
        }
        Ok(ChatResponse {
            content,
            metadata: BTreeMap::from([
                ("provider".to_string(), "codex-cli".to_string()),
                (
                    "model".to_string(),
                    self.config
                        .model
                        .clone()
                        .unwrap_or_else(|| "codex-default".to_string()),
                ),
                ("base_url".to_string(), "codex-cli".to_string()),
            ]),
        })
    }
}

pub struct ChatPlanner<C>
where
    C: ChatClient,
{
    client: C,
}

impl<C> ChatPlanner<C>
where
    C: ChatClient,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }
}

impl<C> Planner for ChatPlanner<C>
where
    C: ChatClient,
{
    fn plan(&mut self, need: &Need, tools: &[ToolSpec]) -> Plan {
        let tool_sheet = tools
            .iter()
            .map(|tool| format!("- {}: {}", tool.name, tool.description))
            .collect::<Vec<_>>()
            .join("\n");
        let messages = vec![
            ChatMessage::new(
                ChatRole::System,
                "You are an Octopus tentacle brain. Return only JSON for Plan.",
            ),
            ChatMessage::new(
                ChatRole::User,
                format!(
                    "Need: {}: {}\nTools:\n{}\nReturn JSON: {{\"calls\":[{{\"tool\":\"name\",\"reason\":\"short\",\"payload\":{{}}}}],\"summary\":\"short\"}}",
                    kind_key(&need.kind),
                    need.query,
                    tool_sheet
                ),
            ),
        ];
        match self.client.chat(&messages) {
            Ok(response) => match plan_from_llm_content(&response.content) {
                Ok(plan) if !plan.calls.is_empty() => plan,
                Ok(_) => Plan {
                    calls: Vec::new(),
                    summary: "llm planning returned no tool calls".to_string(),
                },
                Err(error) => Plan {
                    calls: Vec::new(),
                    summary: format!("llm planning returned invalid JSON: {error}"),
                },
            },
            Err(error) => Plan {
                calls: Vec::new(),
                summary: format!("llm planning failed: {error}"),
            },
        }
    }
}

pub(crate) fn plan_from_llm_content(content: &str) -> Result<Plan, String> {
    serde_json::from_str::<Plan>(content)
        .or_else(|direct_error| {
            extract_first_json_object(content)
                .ok_or_else(|| direct_error.to_string())
                .and_then(|json| {
                    serde_json::from_str::<Plan>(json)
                        .map_err(|extract_error| extract_error.to_string())
                })
        })
        .map_err(|error| format!("invalid Plan JSON: {error}"))
}

fn extract_first_json_object(content: &str) -> Option<&str> {
    let start = content.find('{')?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, ch) in content[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let end = start + offset + ch.len_utf8();
                    return Some(&content[start..end]);
                }
            }
            _ => {}
        }
    }
    None
}
