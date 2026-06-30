use super::*;

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct ProviderProfile {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) backend: String,
    pub(crate) base_url: String,
    pub(crate) model: String,
    pub(crate) key_env: String,
    pub(crate) notes: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct ProviderEnvReport {
    pub(crate) profile: ProviderProfile,
    pub(crate) prefix: String,
    pub(crate) exports: Vec<String>,
    pub(crate) shell: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProviderSavedEnvReport {
    pub(crate) path: String,
    pub(crate) env: ProviderEnvReport,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProviderCheckReport {
    pub(crate) ok: bool,
    pub(crate) prefix: String,
    pub(crate) model: String,
    pub(crate) base_url: String,
    pub(crate) api_key_present: bool,
    pub(crate) tuning: OpenAiCompatibleTuning,
    pub(crate) content: String,
    pub(crate) metadata: std::collections::BTreeMap<String, String>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProviderStatusReport {
    pub(crate) coverage: Vec<ProviderCoverageStatus>,
    pub(crate) layers: Vec<ProviderLayerStatus>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProviderCoverageStatus {
    pub(crate) id: String,
    pub(crate) purpose: String,
    pub(crate) ready: bool,
    pub(crate) required_layers: Vec<String>,
    pub(crate) prefixes: Vec<String>,
    pub(crate) live_proof: Vec<String>,
    pub(crate) blockers: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProviderMatrixReport {
    pub(crate) path: String,
    pub(crate) version: String,
    pub(crate) current_head: Option<String>,
    pub(crate) targets: Vec<ProviderMatrixTarget>,
    pub(crate) env_files: Vec<ProviderMatrixEnvFileReport>,
    pub(crate) commands: Vec<String>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProviderMatrixEnvFileReport {
    pub(crate) target: String,
    pub(crate) path: String,
    pub(crate) status: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProviderMatrixCheckReport {
    pub(crate) path: String,
    pub(crate) current_head: Option<String>,
    pub(crate) passed: bool,
    pub(crate) checks: Vec<PreflightCheck>,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProviderMatrixRunReport {
    pub(crate) path: String,
    pub(crate) current_head: Option<String>,
    pub(crate) results: Vec<ProviderMatrixRunTargetReport>,
    pub(crate) audit: ProviderMatrixCheckReport,
    pub(crate) next: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProviderMatrixRunTargetReport {
    pub(crate) id: String,
    pub(crate) enabled: bool,
    pub(crate) enabled_env: String,
    pub(crate) env_path: String,
    pub(crate) provider_check: String,
    pub(crate) clean_brain: String,
    pub(crate) tentacle_planning: String,
    pub(crate) harness_evolution: String,
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct ProviderMatrixTarget {
    pub(crate) id: String,
    pub(crate) kind: String,
    pub(crate) profile: String,
    pub(crate) prefix: String,
    pub(crate) env_path: String,
    pub(crate) purpose: String,
    pub(crate) enabled_env: String,
    pub(crate) check_token: String,
    pub(crate) planning_tentacle: String,
    pub(crate) planning_need_kind: NeedKind,
    pub(crate) planning_need_query: String,
    pub(crate) evolution_tentacle: String,
    pub(crate) evolution_objective: String,
    pub(crate) commands: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProviderLayerStatus {
    pub(crate) layer: String,
    pub(crate) purpose: String,
    pub(crate) enabled: bool,
    pub(crate) prefix: String,
    pub(crate) configured: bool,
    pub(crate) model: Option<String>,
    pub(crate) base_url: Option<String>,
    pub(crate) api_key_present: bool,
    pub(crate) tuning: Option<OpenAiCompatibleTuning>,
    pub(crate) curl_command: String,
    pub(crate) curl_available: bool,
    pub(crate) check_command: String,
    pub(crate) message: String,
}

pub(crate) fn provider_profiles() -> Vec<ProviderProfile> {
    vec![
        ProviderProfile {
            id: "openai".to_string(),
            name: "OpenAI-compatible cloud".to_string(),
            description: "Default hosted chat-completions provider.".to_string(),
            backend: "openai-compatible".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4.1-mini".to_string(),
            key_env: "OPENAI_API_KEY".to_string(),
            notes: vec![
                "Set OPENAI_API_KEY before sourcing the generated env.".to_string(),
                "Works for chat, manifest tool planning, and harness evolution.".to_string(),
            ],
        },
        ProviderProfile {
            id: "local".to_string(),
            name: "Local OpenAI-compatible server".to_string(),
            description: "For Ollama, LM Studio, llama.cpp server, or another local adapter."
                .to_string(),
            backend: "openai-compatible".to_string(),
            base_url: "http://localhost:11434/v1".to_string(),
            model: "llama3.1".to_string(),
            key_env: "".to_string(),
            notes: vec![
                "Change the model to whatever your local server exposes.".to_string(),
                "API key is intentionally empty for most local adapters.".to_string(),
            ],
        },
        ProviderProfile {
            id: "litellm".to_string(),
            name: "LiteLLM universal gateway".to_string(),
            description: "Self-hosted OpenAI-compatible proxy for 100+ model providers."
                .to_string(),
            backend: "openai-compatible".to_string(),
            base_url: "http://localhost:4000/v1".to_string(),
            model: "your-model".to_string(),
            key_env: "LITELLM_API_KEY".to_string(),
            notes: vec![
                "Start LiteLLM proxy, then point Octopus at the proxy base URL.".to_string(),
                "Use this for provider-native keys that are not directly OpenAI-compatible."
                    .to_string(),
            ],
        },
        ProviderProfile {
            id: "codex".to_string(),
            name: "Codex CLI OAuth".to_string(),
            description: "Uses the local Codex CLI login session instead of an API key."
                .to_string(),
            backend: "codex".to_string(),
            base_url: "codex-cli".to_string(),
            model: "gpt-5.3-codex-spark".to_string(),
            key_env: "".to_string(),
            notes: vec![
                "Run `codex login` once, then `octopus provider check` can use that OAuth session."
                    .to_string(),
                "This is useful when a user prefers Codex login over raw API keys.".to_string(),
            ],
        },
        ProviderProfile {
            id: "zai".to_string(),
            name: "Z.AI / BigModel OpenAI-compatible".to_string(),
            description: "GLM models through the BigModel OpenAI-compatible endpoint.".to_string(),
            backend: "openai-compatible".to_string(),
            base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            model: "glm-4.5-flash".to_string(),
            key_env: "ZAI_API_KEY".to_string(),
            notes: vec![
                "Set ZAI_API_KEY before sourcing the generated env.".to_string(),
                "Reasoning models may need a larger max token budget before final content appears."
                    .to_string(),
            ],
        },
        ProviderProfile {
            id: "openrouter".to_string(),
            name: "OpenRouter-compatible cloud".to_string(),
            description: "Hosted router using the OpenAI-compatible chat API.".to_string(),
            backend: "openai-compatible".to_string(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model: "openai/gpt-4.1-mini".to_string(),
            key_env: "OPENROUTER_API_KEY".to_string(),
            notes: vec![
                "Set OPENROUTER_API_KEY before sourcing the generated env.".to_string(),
                "Change the model id to any enabled router model.".to_string(),
            ],
        },
        ProviderProfile {
            id: "deepseek".to_string(),
            name: "DeepSeek OpenAI-compatible cloud".to_string(),
            description: "DeepSeek chat models through their OpenAI-compatible endpoint."
                .to_string(),
            backend: "openai-compatible".to_string(),
            base_url: "https://api.deepseek.com".to_string(),
            model: "deepseek-v4-flash".to_string(),
            key_env: "DEEPSEEK_API_KEY".to_string(),
            notes: vec![
                "Set DEEPSEEK_API_KEY before sourcing the generated env.".to_string(),
                "Good fit for clean-brain exploration and economical tool-side planning."
                    .to_string(),
            ],
        },
        ProviderProfile {
            id: "groq".to_string(),
            name: "Groq OpenAI-compatible cloud".to_string(),
            description: "Fast hosted inference through Groq's OpenAI-compatible endpoint."
                .to_string(),
            backend: "openai-compatible".to_string(),
            base_url: "https://api.groq.com/openai/v1".to_string(),
            model: "openai/gpt-oss-20b".to_string(),
            key_env: "GROQ_API_KEY".to_string(),
            notes: vec![
                "Set GROQ_API_KEY before sourcing the generated env.".to_string(),
                "Useful for low-latency Need exploration and provider checks.".to_string(),
            ],
        },
        ProviderProfile {
            id: "gemini".to_string(),
            name: "Gemini OpenAI-compatible cloud".to_string(),
            description: "Gemini models through Google's OpenAI-compatible endpoint.".to_string(),
            backend: "openai-compatible".to_string(),
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
            model: "gemini-3.5-flash".to_string(),
            key_env: "GEMINI_API_KEY".to_string(),
            notes: vec![
                "Set GEMINI_API_KEY before sourcing the generated env.".to_string(),
                "Use another Gemini model id if your account exposes it.".to_string(),
            ],
        },
        ProviderProfile {
            id: "dashscope".to_string(),
            name: "DashScope OpenAI-compatible cloud".to_string(),
            description: "Qwen models through Alibaba Cloud DashScope compatibility mode."
                .to_string(),
            backend: "openai-compatible".to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            model: "qwen-plus".to_string(),
            key_env: "DASHSCOPE_API_KEY".to_string(),
            notes: vec![
                "Set DASHSCOPE_API_KEY before sourcing the generated env.".to_string(),
                "Change the model to a Qwen id enabled for your account.".to_string(),
                "Change the base URL if your region or workspace requires a regional endpoint."
                    .to_string(),
            ],
        },
        ProviderProfile {
            id: "moonshot".to_string(),
            name: "Moonshot OpenAI-compatible cloud".to_string(),
            description: "Kimi/Moonshot models through an OpenAI-compatible endpoint.".to_string(),
            backend: "openai-compatible".to_string(),
            base_url: "https://api.moonshot.ai/v1".to_string(),
            model: "kimi-k2.6".to_string(),
            key_env: "MOONSHOT_API_KEY".to_string(),
            notes: vec![
                "Set MOONSHOT_API_KEY before sourcing the generated env.".to_string(),
                "Use a larger context model id when clean-brain memory grows.".to_string(),
            ],
        },
        ProviderProfile {
            id: "lmstudio".to_string(),
            name: "LM Studio local server".to_string(),
            description: "LM Studio's local OpenAI-compatible server.".to_string(),
            backend: "openai-compatible".to_string(),
            base_url: "http://localhost:1234/v1".to_string(),
            model: "local-model".to_string(),
            key_env: "".to_string(),
            notes: vec![
                "Start the LM Studio local server before provider check.".to_string(),
                "Replace local-model with the served model id.".to_string(),
            ],
        },
        ProviderProfile {
            id: "custom".to_string(),
            name: "Custom OpenAI-compatible endpoint".to_string(),
            description: "Template for any provider that speaks chat completions.".to_string(),
            backend: "openai-compatible".to_string(),
            base_url: "https://example.com/v1".to_string(),
            model: "your-model".to_string(),
            key_env: "OCTOPUS_PROVIDER_API_KEY".to_string(),
            notes: vec![
                "Edit base URL, model, and key env to match your provider.".to_string(),
                "Use a different prefix for chat, manifest, or evolution if needed.".to_string(),
            ],
        },
    ]
}

pub(crate) fn provider_profile(id: &str) -> Result<ProviderProfile, String> {
    provider_profiles()
        .into_iter()
        .find(|profile| profile.id == id)
        .ok_or_else(|| {
            format!(
                "unknown provider profile: {id}; expected {}",
                provider_profiles()
                    .iter()
                    .map(|profile| profile.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
}

pub(crate) fn provider_env_report(
    profile_id: &str,
    prefix: &str,
) -> Result<ProviderEnvReport, String> {
    provider_env_report_inner(profile_id, prefix, false)
}

pub(crate) fn provider_env_report_with_key(
    profile_id: &str,
    prefix: &str,
) -> Result<ProviderEnvReport, String> {
    provider_env_report_inner(profile_id, prefix, true)
}

fn provider_env_report_inner(
    profile_id: &str,
    prefix: &str,
    include_key: bool,
) -> Result<ProviderEnvReport, String> {
    if !valid_env_prefix(prefix) {
        return Err(format!(
            "invalid provider env prefix: {prefix}; use letters, digits, and underscore"
        ));
    }
    let profile = provider_profile(profile_id)?;
    let mut exports = vec![
        format!("export {prefix}_BACKEND={}", shell_value(&profile.backend)),
        format!("export {prefix}_MODEL={}", shell_value(&profile.model)),
    ];
    if profile.backend == "codex" {
        exports.push(format!("export {prefix}_CODEX_COMMAND=codex"));
    } else {
        exports.push(format!(
            "export {prefix}_BASE_URL={}",
            shell_value(&profile.base_url)
        ));
        if profile.key_env.is_empty() {
            exports.push(format!("export {prefix}_API_KEY="));
        } else if include_key {
            let key = env::var(&profile.key_env)
                .map_err(|_| format!("{} is required for provider save-key", profile.key_env))?;
            exports.push(format!("export {prefix}_API_KEY={}", shell_value(&key)));
        } else {
            exports.push(format!(
                "export {prefix}_API_KEY=\"${{{}:-}}\"",
                profile.key_env
            ));
        }
    }
    exports.extend([
        "# Optional model thinking controls. Leave blank unless the provider supports them."
            .to_string(),
        format!("# export {prefix}_REASONING_EFFORT=medium"),
        format!("# export {prefix}_MAX_TOKENS=2048"),
        format!("# export {prefix}_TEMPERATURE=0.2"),
        format!("# export {prefix}_TOP_P=0.9"),
        format!("# export {prefix}_RETRIES=1"),
        format!("# export {prefix}_RETRY_MS=750"),
        format!(
            "# export {prefix}_EXTRA_BODY='{{\"response_format\":{{\"type\":\"json_object\"}}}}'"
        ),
        "export OCTOPUS_CHAT_LLM=1".to_string(),
        "export OCTOPUS_BRAIN_LLM=1".to_string(),
        format!("export OCTOPUS_CHAT_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_BRAIN_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_BRAIN_INTENT_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_BRAIN_BRIEF_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_BRAIN_EXPLORE_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_BRAIN_CLARIFY_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_BRAIN_AGENDA_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_BRAIN_SCOUT_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_BRAIN_DELIBERATE_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_BRAIN_REFLECT_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_BRAIN_ALIGN_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_BRAIN_MEMORY_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_BRAIN_GOAL_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_BRAIN_REWRITE_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_BRAIN_QUEUE_LLM_PREFIX={prefix}"),
    ]);
    exports.extend([
        "export OCTOPUS_LLM_MANIFEST=1".to_string(),
        "export OCTOPUS_LLM_EVOLVE=1".to_string(),
        format!("export OCTOPUS_MANIFEST_LLM_PREFIX={prefix}"),
        format!("export OCTOPUS_EVOLVE_LLM_PREFIX={prefix}"),
    ]);
    let shell = exports.join("\n");
    Ok(ProviderEnvReport {
        profile,
        prefix: prefix.to_string(),
        exports,
        shell,
    })
}

pub(crate) fn save_provider_env_report(
    profile_id: &str,
    prefix: &str,
    path: &Path,
) -> Result<ProviderSavedEnvReport, String> {
    save_provider_env_report_inner(profile_id, prefix, path, false)
}

pub(crate) fn save_provider_env_report_with_key(
    profile_id: &str,
    prefix: &str,
    path: &Path,
) -> Result<ProviderSavedEnvReport, String> {
    save_provider_env_report_inner(profile_id, prefix, path, true)
}

fn save_provider_env_report_inner(
    profile_id: &str,
    prefix: &str,
    path: &Path,
    include_key: bool,
) -> Result<ProviderSavedEnvReport, String> {
    let write_env = if include_key {
        provider_env_report_with_key(profile_id, prefix)?
    } else {
        provider_env_report(profile_id, prefix)?
    };
    let report_env = if include_key {
        provider_env_report(profile_id, prefix)?
    } else {
        write_env.clone()
    };
    if let Some(parent) = path.parent().filter(|value| !value.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, format!("{}\n", write_env.shell)).map_err(|error| error.to_string())?;
    secure_provider_env_file(path)?;
    let path = path.to_string_lossy().to_string();
    let prefix = report_env.prefix.clone();
    Ok(ProviderSavedEnvReport {
        path: path.clone(),
        env: report_env,
        next: vec![
            format!("source {path}"),
            "octopus provider status".to_string(),
            format!("octopus provider check {prefix}"),
        ],
    })
}

pub(crate) fn write_provider_matrix_record(
    output_path: &Path,
) -> Result<ProviderMatrixReport, String> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let current_head = git_short_head();
    let targets = provider_matrix_targets();
    let env_files = ensure_provider_matrix_env_files(&targets)?;
    let commands = targets
        .iter()
        .flat_map(|target| target.commands.clone())
        .collect::<Vec<_>>();
    if !output_path.exists() {
        if let Some(parent) = output_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let head = current_head.as_deref().unwrap_or("unknown");
        let mut content = format!(
            r#"# Provider Matrix Record

Use this as current-head evidence for `0.0.18` provider validation.

## Machine

- Date:
- Tester:
- Machine:
- OS:
- Shell:
- Git commit: `{head}`
- Package version: `{version}`

## Coverage

- Goal chat:
- Clean brain:
- Tentacle planning:
- Harness evolution:

## Targets

"#
        );
        for target in &targets {
            content.push_str(&render_provider_matrix_target_block(target, None));
        }
        content.push_str(
            r#"## Decision

- Pass or fail:
- Follow-up:
"#,
        );
        fs::write(output_path, content).map_err(|error| error.to_string())?;
    } else {
        let head = current_head.as_deref().unwrap_or("unknown");
        let content = fs::read_to_string(output_path).map_err(|error| error.to_string())?;
        let content = set_record_field_value(&content, "Git commit", &format!("`{head}`"));
        let content = set_record_field_value(&content, "Package version", &format!("`{version}`"));
        let content = refresh_provider_matrix_target_blocks(&content, &targets);
        fs::write(output_path, content).map_err(|error| error.to_string())?;
    }
    Ok(ProviderMatrixReport {
        path: output_path.to_string_lossy().to_string(),
        version,
        current_head,
        targets,
        env_files,
        commands,
        next: vec![
            format!("review {}", shell_arg(&output_path.to_string_lossy())),
            "edit .octopus/providers/*.env for any target that needs a real key, model, base URL, or Codex command".to_string(),
            "run each target on a real machine with the matching provider available".to_string(),
            "append summarized results to docs/real-machine-test.md".to_string(),
            "octopus provider status".to_string(),
            "octopus preflight --live".to_string(),
        ],
    })
}

fn ensure_provider_matrix_env_files(
    targets: &[ProviderMatrixTarget],
) -> Result<Vec<ProviderMatrixEnvFileReport>, String> {
    let mut reports = Vec::new();
    for target in targets {
        let path = Path::new(&target.env_path);
        let status = if path.exists() {
            "exists".to_string()
        } else {
            save_provider_env_report(&target.profile, &target.prefix, path)?;
            "created".to_string()
        };
        reports.push(ProviderMatrixEnvFileReport {
            target: target.id.clone(),
            path: target.env_path.clone(),
            status,
        });
    }
    Ok(reports)
}

pub(crate) fn run_provider_matrix_record(
    state: &HarnessState,
    state_path: &Path,
    path: &Path,
) -> Result<ProviderMatrixRunReport, String> {
    if !path.exists() {
        write_provider_matrix_record(path)?;
    }
    let mut content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let targets = provider_matrix_targets();
    let mut results = Vec::new();
    for target in &targets {
        let result = run_provider_matrix_target(state, target);
        content = set_provider_matrix_target_field(
            content,
            target,
            "Provider check",
            &result.provider_check,
        )?;
        content =
            set_provider_matrix_target_field(content, target, "Clean brain", &result.clean_brain)?;
        content = set_provider_matrix_target_field(
            content,
            target,
            "Tentacle planning",
            &result.tentacle_planning,
        )?;
        content = set_provider_matrix_target_field(
            content,
            target,
            "Harness evolution",
            &result.harness_evolution,
        )?;
        results.push(result);
    }
    content = set_provider_matrix_coverage_field(
        content,
        "Goal chat",
        &provider_matrix_coverage_result(&results, |result| &result.provider_check),
    );
    content = set_provider_matrix_coverage_field(
        content,
        "Clean brain",
        &provider_matrix_coverage_result(&results, |result| &result.clean_brain),
    );
    content = set_provider_matrix_coverage_field(
        content,
        "Tentacle planning",
        &provider_matrix_coverage_result(&results, |result| &result.tentacle_planning),
    );
    content = set_provider_matrix_coverage_field(
        content,
        "Harness evolution",
        &provider_matrix_coverage_result(&results, |result| &result.harness_evolution),
    );
    let decision = if provider_matrix_results_pass(&results) {
        "pass"
    } else if !results.iter().any(|result| result.enabled) {
        "fail: no provider target enabled"
    } else {
        "fail: one or more enabled provider targets failed"
    };
    content = set_record_field_value_owned(content, "Pass or fail", decision);
    fs::write(path, content).map_err(|error| error.to_string())?;
    let audit = check_provider_matrix_record(path)?;
    let mut next = audit.next.clone();
    next.push(format!(
        "octopus --state {} provider matrix run {}",
        shell_arg(&state_path.to_string_lossy()),
        shell_arg(&path.to_string_lossy())
    ));
    next.sort();
    next.dedup();
    Ok(ProviderMatrixRunReport {
        path: path.to_string_lossy().to_string(),
        current_head: git_short_head(),
        results,
        audit,
        next,
    })
}

fn run_provider_matrix_target(
    state: &HarnessState,
    target: &ProviderMatrixTarget,
) -> ProviderMatrixRunTargetReport {
    if !env_flag(&target.enabled_env) {
        let skipped = provider_matrix_skipped(target);
        return ProviderMatrixRunTargetReport {
            id: target.id.clone(),
            enabled: false,
            enabled_env: target.enabled_env.clone(),
            env_path: target.env_path.clone(),
            provider_check: skipped.clone(),
            clean_brain: skipped.clone(),
            tentacle_planning: skipped.clone(),
            harness_evolution: skipped,
        };
    }
    let guard = ProviderMatrixEnvGuard::apply(Path::new(&target.env_path));
    if let Err(error) = guard {
        let failed = provider_matrix_fail(error);
        return ProviderMatrixRunTargetReport {
            id: target.id.clone(),
            enabled: true,
            enabled_env: target.enabled_env.clone(),
            env_path: target.env_path.clone(),
            provider_check: failed.clone(),
            clean_brain: failed.clone(),
            tentacle_planning: failed.clone(),
            harness_evolution: failed,
        };
    }
    let _guard = guard.expect("checked provider matrix env guard");
    let prefix = target.prefix.clone();
    let check_token = target.check_token.clone();
    let provider_check = provider_matrix_run_step("provider check", move || {
        provider_check_report(&prefix, &format!("Reply only {check_token}"))
            .map(|report| report.content)
    });
    let prefix = target.prefix.clone();
    let mut state_for_brain = state.clone();
    let clean_brain = provider_matrix_run_step("clean brain", move || {
        provider_client(&prefix).and_then(|(_, _, _, _, mut client)| {
            state_for_brain
                .clean_brain_goal_with_client(
                    "tighten the current objective".to_string(),
                    6,
                    &mut client,
                )
                .map(|report| format!("{} needs", report.needs.len()))
        })
    });
    let prefix = target.prefix.clone();
    let planning_tentacle = target.planning_tentacle.clone();
    let planning_need_kind = target.planning_need_kind.clone();
    let planning_need_query = target.planning_need_query.clone();
    let grants = state.grants.clone();
    let tentacle_planning = provider_matrix_run_step("tentacle planning", move || {
        provider_client_factory(&prefix).and_then(|factory| {
            let root = resolve_tentacle_manifest_root(&planning_tentacle)?;
            think_tentacle_with_llm_factory(
                root,
                &planning_tentacle,
                &Need::new(planning_need_kind, planning_need_query),
                Some(factory),
                &grants,
            )
            .map(|plan| format!("{} actions", plan.actions.len()))
        })
    });
    let prefix = target.prefix.clone();
    let evolution_tentacle = target.evolution_tentacle.clone();
    let evolution_objective = target.evolution_objective.clone();
    let state_for_evolution = state.clone();
    let harness_evolution = provider_matrix_run_step("harness evolution", move || {
        provider_client(&prefix).and_then(|(_, _, _, _, mut client)| {
            let root = resolve_tentacle_manifest_root(&evolution_tentacle)?;
            propose_tentacle_evolution_with_client(
                root,
                &evolution_tentacle,
                &evolution_objective,
                &state_for_evolution,
                &mut client,
            )
            .map(|proposal| format!("{} candidates", proposal.patch_candidates.len()))
        })
    });
    ProviderMatrixRunTargetReport {
        id: target.id.clone(),
        enabled: true,
        enabled_env: target.enabled_env.clone(),
        env_path: target.env_path.clone(),
        provider_check,
        clean_brain,
        tentacle_planning,
        harness_evolution,
    }
}

struct ProviderMatrixEnvGuard {
    saved: Vec<(String, Option<String>)>,
}

impl ProviderMatrixEnvGuard {
    fn apply(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|error| format!("cannot read {}: {error}", path.to_string_lossy()))?;
        let overlay = app_bridge::parse_env_overlay(&content);
        if overlay.is_empty() {
            return Err(format!(
                "{} has no OCTOPUS provider exports",
                path.to_string_lossy()
            ));
        }
        let saved = overlay
            .iter()
            .map(|(key, _)| (key.clone(), env::var(key).ok()))
            .collect::<Vec<_>>();
        for (key, value) in overlay {
            env::set_var(key, value);
        }
        Ok(Self { saved })
    }
}

impl Drop for ProviderMatrixEnvGuard {
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

fn provider_matrix_coverage_result(
    results: &[ProviderMatrixRunTargetReport],
    field: impl Fn(&ProviderMatrixRunTargetReport) -> &String,
) -> String {
    let enabled = results.iter().filter(|result| result.enabled).count();
    if enabled == 0 {
        return format!(
            "fail: 0/0 enabled targets passed; {} skipped",
            results.len()
        );
    }
    if results
        .iter()
        .filter(|result| result.enabled)
        .all(|result| provider_matrix_result_ready(field(result)))
    {
        let skipped = results.len().saturating_sub(enabled);
        if skipped == 0 {
            format!("pass: {enabled}/{enabled} enabled targets passed")
        } else {
            format!("pass: {enabled}/{enabled} enabled targets passed; {skipped} skipped")
        }
    } else {
        let passed = results
            .iter()
            .filter(|result| result.enabled)
            .filter(|result| provider_matrix_result_ready(field(result)))
            .count();
        format!("fail: {passed}/{enabled} enabled targets passed")
    }
}

fn provider_matrix_results_pass(results: &[ProviderMatrixRunTargetReport]) -> bool {
    results.iter().any(|result| result.enabled)
        && results
            .iter()
            .filter(|result| result.enabled)
            .all(|result| {
                provider_matrix_result_ready(&result.provider_check)
                    && provider_matrix_result_ready(&result.clean_brain)
                    && provider_matrix_result_ready(&result.tentacle_planning)
                    && provider_matrix_result_ready(&result.harness_evolution)
            })
}

fn provider_matrix_pass(summary: &str) -> String {
    let summary = provider_matrix_one_line(summary);
    if summary.is_empty() {
        "pass".to_string()
    } else {
        format!("pass: {summary}")
    }
}

fn provider_matrix_fail(summary: impl AsRef<str>) -> String {
    let summary = provider_matrix_one_line(summary.as_ref());
    if summary.is_empty() {
        "fail".to_string()
    } else {
        format!("fail: {summary}")
    }
}

fn provider_matrix_run_step<F>(label: &'static str, run: F) -> String
where
    F: FnOnce() -> Result<String, String> + Send + 'static,
{
    let timeout = provider_matrix_step_timeout();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(run());
    });
    match rx.recv_timeout(timeout) {
        Ok(Ok(summary)) => provider_matrix_pass(&summary),
        Ok(Err(error)) => provider_matrix_fail(error),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            provider_matrix_fail(format!("{label} timed out after {}s", timeout.as_secs()))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            provider_matrix_fail(format!("{label} worker disconnected"))
        }
    }
}

fn provider_matrix_step_timeout() -> Duration {
    env::var("OCTOPUS_PROVIDER_MATRIX_STEP_TIMEOUT")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|seconds| *seconds > 0)
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(90))
}

fn provider_matrix_skipped(target: &ProviderMatrixTarget) -> String {
    format!("skipped: set {}=1 to run", target.enabled_env)
}

fn provider_matrix_one_line(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(160)
        .collect()
}

fn provider_matrix_result_ready(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();
    value == "pass" || value.starts_with("pass:") || value.starts_with("passed")
}

fn provider_matrix_result_skipped(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();
    value == "skipped" || value.starts_with("skipped:")
}

pub(crate) fn set_provider_matrix_target_field(
    content: String,
    target: &ProviderMatrixTarget,
    label: &str,
    value: &str,
) -> Result<String, String> {
    let marker = format!("### {}: {}", target.kind, target.id);
    let Some(start) = content.find(&marker) else {
        return Err(format!("provider matrix target missing: {}", target.id));
    };
    let rest = &content[start..];
    let next_target = rest
        .find("\n### ")
        .filter(|index| *index > 0)
        .unwrap_or(rest.len());
    let decision = rest.find("\n## Decision").unwrap_or(rest.len());
    let end = start + next_target.min(decision);
    let before = &content[..start];
    let section = &content[start..end];
    let after = &content[end..];
    Ok(format!(
        "{before}{}{after}",
        set_record_field_value(section, label, value)
    ))
}

fn set_provider_matrix_coverage_field(content: String, label: &str, value: &str) -> String {
    let Some(end) = content.find("\n## Targets") else {
        return set_record_field_value_owned(content, label, value);
    };
    let before = &content[..end];
    let after = &content[end..];
    format!("{}{}", set_record_field_value(before, label, value), after)
}

fn set_record_field_value_owned(content: String, label: &str, value: &str) -> String {
    set_record_field_value(&content, label, value)
}

fn set_record_field_value(content: &str, label: &str, value: &str) -> String {
    let prefix = format!("- {label}:");
    let replacement = format!("- {label}: {value}");
    let mut replaced = false;
    let lines = content
        .lines()
        .map(|line| {
            if line.trim_start().starts_with(&prefix) {
                replaced = true;
                replacement.clone()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>();
    if replaced {
        let mut output = lines.join("\n");
        if content.ends_with('\n') {
            output.push('\n');
        }
        output
    } else {
        format!("{content}\n{replacement}\n")
    }
}

fn refresh_provider_matrix_target_blocks(
    content: &str,
    targets: &[ProviderMatrixTarget],
) -> String {
    let Some(targets_start) = content.find("\n## Targets") else {
        let mut refreshed = content.trim_end().to_string();
        refreshed.push_str("\n\n## Targets\n\n");
        for target in targets {
            refreshed.push_str(&render_provider_matrix_target_block(target, None));
        }
        refreshed.push_str("## Decision\n\n- Pass or fail:\n- Follow-up:\n");
        return refreshed;
    };
    let before = &content[..targets_start];
    let decision_start = content[targets_start..]
        .find("\n## Decision")
        .map(|index| targets_start + index);
    let after = decision_start
        .map(|index| &content[index..])
        .unwrap_or("\n## Decision\n\n- Pass or fail:\n- Follow-up:\n");
    let mut target_blocks = String::from("\n## Targets\n\n");
    for target in targets {
        let existing = provider_matrix_target_section(content, target);
        target_blocks.push_str(&render_provider_matrix_target_block(target, existing));
    }
    format!("{before}{target_blocks}{}", after.trim_start_matches('\n'))
}

fn render_provider_matrix_target_block(
    target: &ProviderMatrixTarget,
    existing: Option<&str>,
) -> String {
    let field = |label: &str| {
        existing
            .and_then(|section| record_field_value_in(section, label))
            .unwrap_or_default()
    };
    format!(
        r#"### {kind}: {id}

- Profile: `{profile}`
- Prefix: `{prefix}`
- Enable: `{enabled_env}=1`
- Env file: `{env_path}`
- Purpose: {purpose}

```bash
{commands}
```

Results:

- Provider check:{provider_check}
- Clean brain:{clean_brain}
- Tentacle planning:{tentacle_planning}
- Harness evolution:{harness_evolution}
- Notes:{notes}

"#,
        kind = target.kind,
        id = target.id,
        profile = target.profile,
        prefix = target.prefix,
        enabled_env = target.enabled_env,
        env_path = target.env_path,
        purpose = target.purpose,
        commands = target.commands.join("\n"),
        provider_check = provider_matrix_render_field_suffix(&field("Provider check")),
        clean_brain = provider_matrix_render_field_suffix(&field("Clean brain")),
        tentacle_planning = provider_matrix_render_field_suffix(&field("Tentacle planning")),
        harness_evolution = provider_matrix_render_field_suffix(&field("Harness evolution")),
        notes = provider_matrix_render_field_suffix(&field("Notes")),
    )
}

fn provider_matrix_render_field_suffix(value: &str) -> String {
    if value.trim().is_empty() {
        String::new()
    } else {
        format!(" {}", value.trim())
    }
}

pub(crate) fn check_provider_matrix_record(
    path: &Path,
) -> Result<ProviderMatrixCheckReport, String> {
    let current_head = git_short_head();
    let content = fs::read_to_string(path).unwrap_or_default();
    let file_exists = path.exists();
    let machine_fields = ["Date", "Tester", "Machine", "OS", "Shell"];
    let missing_machine = machine_fields
        .iter()
        .filter(|field| record_field_value(&content, field).is_none())
        .map(|field| (*field).to_string())
        .collect::<Vec<_>>();
    let coverage_fields = [
        "Goal chat",
        "Clean brain",
        "Tentacle planning",
        "Harness evolution",
    ];
    let coverage_issues = coverage_fields
        .iter()
        .filter_map(|field| match record_field_value(&content, field) {
            Some(value) if provider_matrix_result_ready(&value) => None,
            Some(value) => Some(format!("{field} not pass ({value})")),
            None => Some(format!("{field} missing")),
        })
        .collect::<Vec<_>>();
    let targets = provider_matrix_targets();
    let target_result_fields = [
        "Provider check",
        "Clean brain",
        "Tentacle planning",
        "Harness evolution",
    ];
    let target_issues = targets
        .iter()
        .filter_map(|target| {
            let section = provider_matrix_target_section(&content, target)?;
            let values = target_result_fields
                .iter()
                .map(|field| (*field, record_field_value_in(section, field)))
                .collect::<Vec<_>>();
            let missing = values
                .iter()
                .filter_map(|(field, value)| value.is_none().then(|| format!("{field} missing")))
                .collect::<Vec<_>>();
            if !missing.is_empty() {
                return Some(format!("{} {}", target.id, missing.join(", ")));
            }
            let values = values
                .iter()
                .filter_map(|(field, value)| value.as_ref().map(|value| (*field, value)))
                .collect::<Vec<_>>();
            if values
                .iter()
                .all(|(_, value)| provider_matrix_result_skipped(value))
            {
                return None;
            }
            let issues = values
                .iter()
                .filter_map(|(field, value)| {
                    if provider_matrix_result_ready(value) {
                        None
                    } else {
                        Some(format!("{field} not pass ({value})"))
                    }
                })
                .collect::<Vec<_>>();
            (!issues.is_empty()).then(|| format!("{} {}", target.id, issues.join(", ")))
        })
        .chain(targets.iter().filter_map(|target| {
            provider_matrix_target_section(&content, target)
                .is_none()
                .then(|| format!("{} section missing", target.id))
        }))
        .collect::<Vec<_>>();
    let pass_decision_value = record_field_value(&content, "Pass or fail");
    let pass_decision = pass_decision_value
        .as_deref()
        .is_some_and(record_pass_decision_ready);
    let head_ready = current_head
        .as_ref()
        .is_some_and(|head| content.contains(&format!("`{head}`")) || content.contains(head));
    let mut checks = vec![
        preflight_check(
            "matrix_file",
            file_exists,
            true,
            path.to_string_lossy(),
            "octopus provider matrix",
        ),
        preflight_check(
            "current_head",
            head_ready,
            true,
            current_head
                .as_ref()
                .map(|head| format!("record must include current head {head}"))
                .unwrap_or_else(|| "git head unavailable".to_string()),
            "rerun octopus provider matrix on the target head",
        ),
        preflight_check(
            "machine_fields",
            missing_machine.is_empty(),
            true,
            if missing_machine.is_empty() {
                "machine fields filled".to_string()
            } else {
                format!("missing {}", missing_machine.join(", "))
            },
            "fill Date, Tester, Machine, OS, and Shell",
        ),
        preflight_check(
            "coverage_fields",
            coverage_issues.is_empty(),
            true,
            if coverage_issues.is_empty() {
                "coverage fields passed".to_string()
            } else {
                coverage_issues.join("; ")
            },
            "run octopus provider matrix run or fill passing coverage results",
        ),
        preflight_check(
            "target_results",
            target_issues.is_empty(),
            true,
            if target_issues.is_empty() {
                "configured provider target results passed; skipped targets are unconfigured"
                    .to_string()
            } else {
                target_issues.join("; ")
            },
            "run octopus provider matrix run or fill passing target results",
        ),
        preflight_check(
            "pass_decision",
            pass_decision,
            true,
            pass_decision_value.unwrap_or_else(|| "missing pass/fail decision".to_string()),
            "write Pass or fail: pass after provider matrix checks pass",
        ),
    ];
    checks.sort_by(|left, right| left.id.cmp(&right.id));
    let passed = checks
        .iter()
        .filter(|check| check.required)
        .all(|check| check.status == "pass");
    let next = if passed {
        vec![
            "append summarized provider matrix results to docs/real-machine-test.md".to_string(),
            "octopus preflight --live".to_string(),
        ]
    } else {
        checks
            .iter()
            .filter(|check| check.status != "pass")
            .map(|check| check.next.clone())
            .collect::<Vec<_>>()
    };
    Ok(ProviderMatrixCheckReport {
        path: path.to_string_lossy().to_string(),
        current_head,
        passed,
        checks,
        next,
    })
}

pub(crate) fn provider_matrix_check_evidence(report: &ProviderMatrixCheckReport) -> String {
    let failed = report
        .checks
        .iter()
        .filter(|check| check.status != "pass")
        .map(|check| format!("{}={}", check.id, check.status))
        .collect::<Vec<_>>();
    if failed.is_empty() {
        format!("passed=true checks={}", report.checks.len())
    } else {
        format!(
            "passed=false checks={} blockers={}",
            report.checks.len(),
            failed.join("; ")
        )
    }
}

fn provider_matrix_target_section<'a>(
    content: &'a str,
    target: &ProviderMatrixTarget,
) -> Option<&'a str> {
    let marker = format!("### {}: {}", target.kind, target.id);
    let start = content.find(&marker)?;
    let rest = &content[start..];
    let next_target = rest
        .find("\n### ")
        .filter(|index| *index > 0)
        .unwrap_or(rest.len());
    let decision = rest.find("\n## Decision").unwrap_or(rest.len());
    Some(&rest[..next_target.min(decision)])
}

fn record_field_value_in(content: &str, label: &str) -> Option<String> {
    let prefix = format!("- {label}:");
    content.lines().find_map(|line| {
        let value = line.trim().strip_prefix(&prefix)?.trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

pub(crate) fn provider_matrix_targets() -> Vec<ProviderMatrixTarget> {
    [
        (
            "codex-oauth",
            "Codex OAuth",
            "codex",
            "OCTOPUS_MATRIX_CODEX",
            ".octopus/providers/codex.env",
            "login-session provider without a raw API key",
            "OCTOPUS_CODEX_OK",
        ),
        (
            "api-key-cloud",
            "API-key cloud",
            "openai",
            "OCTOPUS_MATRIX_OPENAI",
            ".octopus/providers/openai.env",
            "direct OpenAI-compatible cloud endpoint with an API key",
            "OCTOPUS_OPENAI_OK",
        ),
        (
            "local-openai-compatible",
            "Local model",
            "lmstudio",
            "OCTOPUS_MATRIX_LOCAL",
            ".octopus/providers/local.env",
            "local OpenAI-compatible server, usually without an API key",
            "OCTOPUS_LOCAL_OK",
        ),
        (
            "gateway-router",
            "Gateway/router",
            "litellm",
            "OCTOPUS_MATRIX_GATEWAY",
            ".octopus/providers/gateway.env",
            "LiteLLM or another OpenAI-compatible provider gateway",
            "OCTOPUS_GATEWAY_OK",
        ),
    ]
    .into_iter()
    .map(|(id, kind, profile, prefix, env_path, purpose, token)| {
        let env_arg = shell_arg(env_path);
        let (
            planning_tentacle,
            planning_need_kind,
            planning_need_query,
            evolution_tentacle,
            evolution_objective,
        ) = if id == "local-openai-compatible" {
            (
                "json-feed",
                NeedKind::Verify,
                "local provider readiness",
                "json-feed",
                "improve compact JSON Feed evidence",
            )
        } else {
            (
                "swe-agent",
                NeedKind::Observe,
                "README.md",
                "swe-agent",
                "improve Feed evidence",
            )
        };
        let goal = format!(
            "{} OCTOPUS_BRAIN_LLM=1 OCTOPUS_BRAIN_GOAL_LLM_PREFIX={} octopus brain --goal --live \"tighten the current objective\"",
            env_assignment(prefix),
            prefix
        );
        let think = format!(
            "{} OCTOPUS_LLM_MANIFEST=1 OCTOPUS_MANIFEST_LLM_PREFIX={} octopus think {} {} {}",
            env_assignment(prefix),
            prefix,
            planning_tentacle,
            need_label(&planning_need_kind),
            shell_arg(planning_need_query)
        );
        let evolve = format!(
            "{} OCTOPUS_LLM_EVOLVE=1 OCTOPUS_EVOLVE_LLM_PREFIX={} octopus evolve {} {}",
            env_assignment(prefix),
            prefix,
            evolution_tentacle,
            shell_arg(evolution_objective)
        );
        ProviderMatrixTarget {
            id: id.to_string(),
            kind: kind.to_string(),
            profile: profile.to_string(),
            prefix: prefix.to_string(),
            env_path: env_path.to_string(),
            purpose: purpose.to_string(),
            enabled_env: token.to_string(),
            check_token: token.to_string(),
            planning_tentacle: planning_tentacle.to_string(),
            planning_need_kind,
            planning_need_query: planning_need_query.to_string(),
            evolution_tentacle: evolution_tentacle.to_string(),
            evolution_objective: evolution_objective.to_string(),
            commands: vec![
                format!("octopus provider save {profile} {prefix} {env_arg}"),
                format!("export {token}=1"),
                format!("source {env_arg}"),
                "octopus provider status".to_string(),
                format!("octopus provider check {prefix} \"Reply only {token}\""),
                goal,
                think,
                evolve,
            ],
        }
    })
    .collect()
}

fn env_assignment(prefix: &str) -> String {
    format!(
        "{prefix}_BACKEND=\"${{{prefix}_BACKEND:-openai-compatible}}\" {prefix}_MODEL=\"${{{prefix}_MODEL:-model}}\" {prefix}_BASE_URL=\"${{{prefix}_BASE_URL:-http://localhost:1234/v1}}\""
    )
}

fn secure_provider_env_file(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|error| error.to_string())?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

pub(crate) fn provider_check_report(
    prefix: &str,
    message: &str,
) -> Result<ProviderCheckReport, String> {
    if !valid_env_prefix(prefix) {
        return Err(format!(
            "invalid provider env prefix: {prefix}; use letters, digits, and underscore"
        ));
    }
    let backend = provider_backend(prefix);
    let (model, base_url, api_key_present, tuning, mut client) =
        provider_client(prefix).map_err(|error| provider_check_error(prefix, &error))?;
    let response = client
        .chat(&[
            ChatMessage::new(
                ChatRole::System,
                "You are an Octopus provider check. Reply briefly and confirm the endpoint works.",
            ),
            ChatMessage::new(ChatRole::User, message),
        ])
        .map_err(|error| provider_check_error(prefix, &error))?;
    Ok(ProviderCheckReport {
        ok: true,
        prefix: prefix.to_string(),
        model,
        base_url,
        api_key_present,
        tuning,
        content: response.content,
        metadata: response
            .metadata
            .into_iter()
            .chain([("backend".to_string(), backend)])
            .collect(),
        next: vec![
            "octopus chat \"refine the goal\"".to_string(),
            "octopus need observe README.md".to_string(),
            "octopus evolve swe-agent \"improve tool-side feedback\"".to_string(),
        ],
    })
}

fn provider_check_error(prefix: &str, error: &str) -> String {
    format!(
        "provider check failed for {prefix}: {error}\nnext: octopus providers; octopus provider save openai; source .octopus/llm.env; octopus provider check {prefix}"
    )
}

pub(crate) fn provider_backend(prefix: &str) -> String {
    env::var(format!("{prefix}_BACKEND"))
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "openai-compatible".to_string())
}

pub(crate) type ProviderClient = (
    String,
    String,
    bool,
    OpenAiCompatibleTuning,
    Box<dyn ChatClient>,
);

pub(crate) fn provider_client(prefix: &str) -> Result<ProviderClient, String> {
    match provider_backend(prefix).as_str() {
        "codex" => {
            let config = CodexCliConfig::from_env_prefix(prefix)?;
            let model = config
                .model
                .clone()
                .unwrap_or_else(|| "codex-default".to_string());
            Ok((
                model,
                "codex-cli".to_string(),
                false,
                OpenAiCompatibleTuning::default(),
                Box::new(CodexCliChatClient::new(config)),
            ))
        }
        "openai-compatible" | "" => {
            let config = OpenAiCompatibleConfig::from_env_prefix(prefix)?;
            Ok((
                config.model.clone(),
                config.base_url.clone(),
                config.api_key.is_some(),
                config.tuning.clone(),
                Box::new(OpenAiCompatibleChatClient::new(config)),
            ))
        }
        other => Err(format!(
            "unknown {prefix}_BACKEND: {other}; expected openai-compatible or codex"
        )),
    }
}

pub(crate) fn provider_client_factory(prefix: &str) -> Result<ChatClientFactory, String> {
    provider_client(prefix)?;
    let prefix = prefix.to_string();
    Ok(std::sync::Arc::new(move || {
        provider_client(&prefix).map(|(_, _, _, _, client)| client)
    }))
}

pub(crate) fn provider_status_report() -> ProviderStatusReport {
    let layers = vec![
        provider_layer_status(
            "chat_goal",
            "chat refines Goal before Needs",
            chat_llm_enabled(),
            chat_llm_prefix(),
            "export OCTOPUS_CHAT_LLM=1",
        ),
        provider_layer_status(
            "clean_brain",
            "clean brain explores Goal/Mem/Need/Feed into new Needs",
            clean_brain_llm_enabled(),
            clean_brain_llm_prefix(),
            "export OCTOPUS_BRAIN_LLM=1",
        ),
        provider_layer_status(
            "clean_brain_intent",
            "cognitive intent selection from Goal/Mem/Need/Feed",
            clean_brain_llm_enabled(),
            clean_brain_intent_llm_prefix(),
            "export OCTOPUS_BRAIN_LLM=1",
        ),
        provider_layer_status(
            "clean_brain_brief",
            "compact cognitive brief from Goal/Mem/Need/Feed",
            clean_brain_llm_enabled(),
            clean_brain_brief_llm_prefix(),
            "export OCTOPUS_BRAIN_LLM=1",
        ),
        provider_layer_status(
            "clean_brain_explore",
            "Need exploration from Goal/Mem/Need/Feed",
            clean_brain_llm_enabled(),
            clean_brain_explore_llm_prefix(),
            "export OCTOPUS_BRAIN_LLM=1",
        ),
        provider_layer_status(
            "clean_brain_clarify",
            "human clarification from Goal/Mem/Need/Feed",
            clean_brain_llm_enabled(),
            clean_brain_clarify_llm_prefix(),
            "export OCTOPUS_BRAIN_LLM=1",
        ),
        provider_layer_status(
            "clean_brain_agenda",
            "cognitive agenda from Goal/Mem/Need/Feed",
            clean_brain_llm_enabled(),
            clean_brain_agenda_llm_prefix(),
            "export OCTOPUS_BRAIN_LLM=1",
        ),
        provider_layer_status(
            "clean_brain_scout",
            "cognitive landscape scouting from Goal/Mem/Need/Feed",
            clean_brain_llm_enabled(),
            clean_brain_scout_llm_prefix(),
            "export OCTOPUS_BRAIN_LLM=1",
        ),
        provider_layer_status(
            "clean_brain_deliberate",
            "cognitive deliberation before Needs",
            clean_brain_llm_enabled(),
            clean_brain_deliberate_llm_prefix(),
            "export OCTOPUS_BRAIN_LLM=1",
        ),
        provider_layer_status(
            "clean_brain_reflect",
            "goal reflection from Goal/Mem/Need/Feed",
            clean_brain_llm_enabled(),
            clean_brain_reflect_llm_prefix(),
            "export OCTOPUS_BRAIN_LLM=1",
        ),
        provider_layer_status(
            "clean_brain_align",
            "goal alignment from Goal/Mem/Need/Feed",
            clean_brain_llm_enabled(),
            clean_brain_align_llm_prefix(),
            "export OCTOPUS_BRAIN_LLM=1",
        ),
        provider_layer_status(
            "clean_brain_memory",
            "memory Needs from Goal/Mem/Need/Feed",
            clean_brain_llm_enabled(),
            clean_brain_memory_llm_prefix(),
            "export OCTOPUS_BRAIN_LLM=1",
        ),
        provider_layer_status(
            "clean_brain_synthesize",
            "multi-model clean-brain draft synthesis",
            clean_brain_llm_enabled(),
            clean_brain_synthesize_llm_prefix(),
            "export OCTOPUS_BRAIN_LLM=1",
        ),
        provider_layer_status(
            "clean_brain_goal",
            "human Goal refinement without Feed execution",
            clean_brain_llm_enabled(),
            clean_brain_goal_llm_prefix(),
            "export OCTOPUS_BRAIN_LLM=1",
        ),
        provider_layer_status(
            "clean_brain_rewrite",
            "rewrite polluted Need candidates into cognitive Needs",
            clean_brain_llm_enabled(),
            clean_brain_rewrite_llm_prefix(),
            "export OCTOPUS_BRAIN_LLM=1",
        ),
        provider_layer_status(
            "clean_brain_queue",
            "review pending Needs before Feed",
            clean_brain_llm_enabled(),
            clean_brain_queue_llm_prefix(),
            "export OCTOPUS_BRAIN_LLM=1",
        ),
        provider_layer_status(
            "tentacle_planning",
            "tentacle brain chooses tool metadata before Feed",
            manifest_llm_enabled(),
            manifest_llm_prefix(),
            "export OCTOPUS_LLM_MANIFEST=1",
        ),
        provider_layer_status(
            "harness_evolution",
            "LLM proposes prompt/meta/code/policy evolution candidates",
            evolve_llm_enabled(),
            evolve_llm_prefix(),
            "export OCTOPUS_LLM_EVOLVE=1",
        ),
    ];
    let coverage = provider_coverage_report(&layers);
    let mut next = vec!["octopus providers".to_string()];
    if coverage.iter().any(|item| !item.ready) {
        next.push("octopus provider save openai".to_string());
        next.push("source .octopus/llm.env".to_string());
    }
    for item in &coverage {
        if item.ready {
            next.extend(item.live_proof.clone());
        }
    }
    for layer in &layers {
        if layer.configured && layer.curl_available {
            next.push(layer.check_command.clone());
        } else if !layer.enabled {
            next.push(layer.message.clone());
        }
    }
    next.sort();
    next.dedup();
    ProviderStatusReport {
        coverage,
        layers,
        next,
    }
}

fn provider_coverage_report(layers: &[ProviderLayerStatus]) -> Vec<ProviderCoverageStatus> {
    vec![
        provider_coverage_status(
            "goal_chat",
            "Goal refinement from user chat",
            &["chat_goal"],
            layers,
            &["octopus chat \"tighten the goal\""],
        ),
        provider_coverage_status(
            "clean_brain",
            "Goal/Mem/Need/Feed clean-brain calls",
            &["clean_brain", "clean_brain_goal"],
            layers,
            &["octopus brain --goal --live \"tighten the current objective\""],
        ),
        provider_coverage_status(
            "tentacle_planning",
            "Tool-side LLM planning before Feed",
            &["tentacle_planning"],
            layers,
            &["octopus think swe-agent observe README.md"],
        ),
        provider_coverage_status(
            "harness_evolution",
            "LLM-assisted prompt/meta/code/policy evolution",
            &["harness_evolution"],
            layers,
            &["octopus evolve swe-agent \"improve Feed evidence\""],
        ),
    ]
}

fn provider_coverage_status(
    id: &str,
    purpose: &str,
    required_layers: &[&str],
    layers: &[ProviderLayerStatus],
    live_proof: &[&str],
) -> ProviderCoverageStatus {
    let matched = required_layers
        .iter()
        .filter_map(|required| layers.iter().find(|layer| layer.layer == *required))
        .collect::<Vec<_>>();
    let missing = required_layers
        .iter()
        .filter(|required| !layers.iter().any(|layer| layer.layer == **required))
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    let mut blockers = Vec::new();
    for layer in &matched {
        if !provider_layer_ready(layer) {
            blockers.push(format!(
                "{}: enabled={} configured={} runtime={} prefix={} message={}",
                layer.layer,
                layer.enabled,
                layer.configured,
                if layer.curl_available {
                    "available"
                } else {
                    "missing"
                },
                layer.prefix,
                layer.message
            ));
        }
    }
    for layer in &missing {
        blockers.push(format!("{layer}: missing from provider status"));
    }
    let mut prefixes = matched
        .iter()
        .map(|layer| layer.prefix.clone())
        .collect::<Vec<_>>();
    prefixes.sort();
    prefixes.dedup();
    let mut proof = matched
        .iter()
        .filter(|layer| provider_layer_ready(layer))
        .map(|layer| layer.check_command.clone())
        .chain(live_proof.iter().map(|item| (*item).to_string()))
        .collect::<Vec<_>>();
    proof.sort();
    proof.dedup();
    ProviderCoverageStatus {
        id: id.to_string(),
        purpose: purpose.to_string(),
        ready: blockers.is_empty(),
        required_layers: required_layers
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        prefixes,
        live_proof: proof,
        blockers,
    }
}

fn provider_layer_ready(layer: &ProviderLayerStatus) -> bool {
    layer.enabled && layer.configured && layer.curl_available
}

pub(crate) fn provider_coverage_ready(report: &ProviderStatusReport) -> bool {
    report.coverage.iter().all(|item| item.ready)
}

fn provider_layer_status(
    layer: &str,
    purpose: &str,
    enabled: bool,
    prefix: String,
    enable_hint: &str,
) -> ProviderLayerStatus {
    if provider_backend(&prefix) == "codex" {
        return match CodexCliConfig::from_env_prefix(&prefix) {
            Ok(config) => {
                let command_is_ready = command_ready(&config.command);
                ProviderLayerStatus {
                    layer: layer.to_string(),
                    purpose: purpose.to_string(),
                    enabled,
                    prefix: prefix.clone(),
                    configured: command_is_ready,
                    model: config.model.or_else(|| Some("codex-default".to_string())),
                    base_url: Some("codex-cli".to_string()),
                    api_key_present: false,
                    tuning: Some(OpenAiCompatibleTuning::default()),
                    curl_available: command_is_ready,
                    curl_command: config.command,
                    check_command: format!("octopus provider check {prefix}"),
                    message: if enabled {
                        "configured via Codex CLI login".to_string()
                    } else {
                        format!("configured but disabled; run `{enable_hint}`")
                    },
                }
            }
            Err(error) => ProviderLayerStatus {
                layer: layer.to_string(),
                purpose: purpose.to_string(),
                enabled,
                prefix: prefix.clone(),
                configured: false,
                model: None,
                base_url: Some("codex-cli".to_string()),
                api_key_present: false,
                tuning: None,
                curl_available: command_ready("codex"),
                curl_command: "codex".to_string(),
                check_command: format!("octopus provider check {prefix}"),
                message: error,
            },
        };
    }
    match OpenAiCompatibleConfig::from_env_prefix(&prefix) {
        Ok(config) => ProviderLayerStatus {
            layer: layer.to_string(),
            purpose: purpose.to_string(),
            enabled,
            prefix: prefix.clone(),
            configured: true,
            model: Some(config.model),
            base_url: Some(config.base_url),
            api_key_present: config.api_key.is_some(),
            tuning: Some(config.tuning),
            curl_available: command_ready(&config.curl_command),
            curl_command: config.curl_command,
            check_command: format!("octopus provider check {prefix}"),
            message: if enabled {
                "configured".to_string()
            } else {
                format!("configured but disabled; run `{enable_hint}`")
            },
        },
        Err(error) => {
            let curl_command =
                env::var(format!("{prefix}_CURL")).unwrap_or_else(|_| "curl".to_string());
            ProviderLayerStatus {
                layer: layer.to_string(),
                purpose: purpose.to_string(),
                enabled,
                prefix: prefix.clone(),
                configured: false,
                model: None,
                base_url: None,
                api_key_present: false,
                tuning: None,
                curl_available: command_ready(&curl_command),
                curl_command,
                check_command: format!("octopus provider check {prefix}"),
                message: error,
            }
        }
    }
}

pub(crate) fn valid_env_prefix(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_uppercase() || first == '_')
        && chars.all(|item| item.is_ascii_uppercase() || item.is_ascii_digit() || item == '_')
}

pub(crate) fn shell_value(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
