use super::*;

#[derive(serde::Serialize)]
pub(crate) struct DoctorReport {
    pub(crate) state_path: String,
    pub(crate) state_exists: bool,
    pub(crate) status: StatusReport,
    pub(crate) environment: EnvironmentReport,
    pub(crate) manifest_count: usize,
    pub(crate) broken_manifests: Vec<String>,
    pub(crate) profile_registry: ProfileRegistryReport,
    pub(crate) llm: DoctorLlmReport,
    pub(crate) runtime: ProductRuntimeReport,
    pub(crate) pet: DoctorPetReport,
    pub(crate) self_iteration_mode: String,
    pub(crate) warnings: Vec<String>,
    pub(crate) next: Vec<String>,
}

#[derive(serde::Serialize)]
pub(crate) struct DoctorLlmReport {
    pub(crate) configured: bool,
    pub(crate) config_prefix: String,
    pub(crate) chat_prefix: String,
    pub(crate) manifest_prefix: String,
    pub(crate) evolve_prefix: String,
    pub(crate) model: Option<String>,
    pub(crate) base_url: Option<String>,
    pub(crate) api_key_present: bool,
    pub(crate) curl_command: String,
    pub(crate) curl_available: bool,
    pub(crate) chat_goal_refinement_enabled: bool,
    pub(crate) manifest_planning_enabled: bool,
    pub(crate) evolution_planning_enabled: bool,
    pub(crate) message: String,
}

#[derive(serde::Serialize)]
pub(crate) struct DoctorPetReport {
    pub(crate) path: String,
    pub(crate) exists: bool,
}

pub(crate) fn doctor_report(
    state: &HarnessState,
    state_path: PathBuf,
) -> Result<DoctorReport, String> {
    let cwd = env::current_dir().map_err(|error| error.to_string())?;
    let environment = EnvironmentReport::detect(&cwd);
    let manifests =
        inspect_tentacle_manifests_with_bundled_seed_overlay(&default_tentacles_root())?;
    let broken_manifests = manifests
        .iter()
        .filter(|manifest| !manifest.missing_entrypoints.is_empty())
        .map(|manifest| {
            format!(
                "{} missing {}",
                manifest.id,
                manifest.missing_entrypoints.join(",")
            )
        })
        .collect::<Vec<_>>();
    let status = state.status_report_with_state(Some(&state_path));
    let llm = doctor_llm_report();
    let runtime = go_runtime_report();
    let profile_registry = profile_registry_report(&state_path);
    let pet_path = repo_root().join("docs/pet.html");
    let pet = DoctorPetReport {
        path: pet_path.to_string_lossy().to_string(),
        exists: pet_path.exists(),
    };
    let self_iteration = state.self_iteration_plan("dangoZhang/Octopus", Some("doctor"));
    let mut warnings = status.warnings.clone();
    warnings.extend(broken_manifests.iter().cloned());
    if !llm.configured {
        warnings.push("llm provider not configured".to_string());
    } else if !llm.curl_available {
        warnings.push(format!(
            "llm curl command unavailable: {}",
            llm.curl_command
        ));
    }
    if !pet.exists {
        warnings.push("pet page missing".to_string());
    }
    if !runtime.go_ready
        && status.field_pool.as_ref().is_some_and(|pool| {
            pool.slots
                .iter()
                .any(|slot| slot.needs_environment && !slot.completed)
        })
    {
        warnings.push(runtime.message.clone());
    }
    if !profile_registry.ok {
        warnings.push(format!(
            "profile registry invalid: {}",
            profile_registry.error.as_deref().unwrap_or("unknown error")
        ));
    }
    warnings.sort();
    warnings.dedup();

    let mut next = vec![status.user_goal_hint.clone()];
    next.push("octopus provider status".to_string());
    if !llm.configured {
        next.push("octopus providers".to_string());
        next.push("octopus provider save openai".to_string());
    } else if llm.curl_available {
        next.push(format!("octopus provider check {}", llm.config_prefix));
    }
    next.push("octopus start --open".to_string());
    next.push("octopus update".to_string());
    next.extend(runtime.next.iter().cloned());
    next.extend(profile_registry.next.iter().cloned());
    next.push(format!("open {}", pet.path));
    next.sort();
    next.dedup();

    Ok(DoctorReport {
        state_path: state_path.to_string_lossy().to_string(),
        state_exists: state_path.exists(),
        status,
        environment,
        manifest_count: manifests.len(),
        broken_manifests,
        profile_registry,
        llm,
        runtime,
        pet,
        self_iteration_mode: self_iteration.mode,
        warnings,
        next,
    })
}

pub(crate) fn doctor_llm_report() -> DoctorLlmReport {
    let config_prefix = doctor_llm_prefix();
    let chat_prefix = chat_llm_prefix();
    let manifest_prefix = manifest_llm_prefix();
    let evolve_prefix = evolve_llm_prefix();
    match OpenAiCompatibleConfig::from_env_prefix(&config_prefix) {
        Ok(config) => DoctorLlmReport {
            configured: true,
            config_prefix,
            chat_prefix,
            manifest_prefix,
            evolve_prefix,
            model: Some(config.model),
            base_url: Some(config.base_url),
            api_key_present: config.api_key.is_some(),
            curl_available: command_ready(&config.curl_command),
            curl_command: config.curl_command,
            chat_goal_refinement_enabled: chat_llm_enabled(),
            manifest_planning_enabled: manifest_llm_enabled(),
            evolution_planning_enabled: evolve_llm_enabled(),
            message: "provider env configured; run provider check for live validation".to_string(),
        },
        Err(error) => {
            let curl_command =
                env::var(format!("{config_prefix}_CURL")).unwrap_or_else(|_| "curl".to_string());
            DoctorLlmReport {
                configured: false,
                config_prefix,
                chat_prefix,
                manifest_prefix,
                evolve_prefix,
                model: None,
                base_url: None,
                api_key_present: false,
                curl_available: command_ready(&curl_command),
                curl_command,
                chat_goal_refinement_enabled: chat_llm_enabled(),
                manifest_planning_enabled: manifest_llm_enabled(),
                evolution_planning_enabled: evolve_llm_enabled(),
                message: error,
            }
        }
    }
}

pub(crate) fn print_doctor_report(report: &DoctorReport, language: Language) {
    match language {
        Language::En => {
            println!("Octopus doctor");
            println!(
                "state: {} ({})",
                report.state_path,
                if report.state_exists { "exists" } else { "new" }
            );
            println!(
                "environment: manifests={}, commands={}, recommended={}",
                join_or_none(&report.environment.manifests),
                join_or_none(&report.environment.commands),
                join_or_none(&report.environment.recommended_profiles)
            );
            println!(
                "tentacles: installed={}, manifests={}, broken={}",
                status_tentacles(&report.status),
                report.manifest_count,
                join_or_none(&report.broken_manifests)
            );
            println!("llm: {}", doctor_llm_line(&report.llm));
            println!("runtime: {}", doctor_runtime_line(&report.runtime));
            println!(
                "profile_registry: source={} path={} ok={} profiles={}",
                report.profile_registry.source,
                report.profile_registry.path,
                report.profile_registry.ok,
                report.profile_registry.profile_count
            );
            println!(
                "pet: {} ({})",
                report.pet.path,
                if report.pet.exists {
                    "exists"
                } else {
                    "missing"
                }
            );
            println!("self-iteration: {}", report.self_iteration_mode);
            println!("warnings: {}", join_or_none(&report.warnings));
            println!("next: {}", join_or_none(&report.next));
        }
        Language::Zh => {
            println!("章鱼体检");
            println!(
                "状态: {} ({})",
                report.state_path,
                if report.state_exists {
                    "存在"
                } else {
                    "新建"
                }
            );
            println!(
                "环境: 项目={}, 命令={}, 推荐={}",
                join_or_none(&report.environment.manifests),
                join_or_none(&report.environment.commands),
                join_or_none(&report.environment.recommended_profiles)
            );
            println!(
                "触手: 已安装={}, manifests={}, 异常={}",
                status_tentacles(&report.status),
                report.manifest_count,
                join_or_none(&report.broken_manifests)
            );
            println!("LLM: {}", doctor_llm_line(&report.llm));
            println!("运行时: {}", doctor_runtime_line(&report.runtime));
            println!(
                "profile registry: 来源={} 路径={} 正常={} profiles={}",
                report.profile_registry.source,
                report.profile_registry.path,
                report.profile_registry.ok,
                report.profile_registry.profile_count
            );
            println!(
                "桌宠: {} ({})",
                report.pet.path,
                if report.pet.exists {
                    "存在"
                } else {
                    "缺失"
                }
            );
            println!("自迭代: {}", report.self_iteration_mode);
            println!("警告: {}", join_or_none(&report.warnings));
            println!("下一步: {}", join_or_none(&report.next));
        }
    }
}

fn doctor_runtime_line(report: &ProductRuntimeReport) -> String {
    format!(
        "go_ready={}, command={}, version={}, message={}",
        report.go_ready,
        report.go_command,
        report.go_version.as_deref().unwrap_or("none"),
        report.message
    )
}

fn doctor_llm_line(report: &DoctorLlmReport) -> String {
    if report.configured {
        format!(
            "configured prefix={} model={} base_url={} api_key={} curl={}({}) chat_refinement={}({}) manifest_planning={}({}) evolution_planning={}({})",
            report.config_prefix,
            report.model.as_deref().unwrap_or("none"),
            report.base_url.as_deref().unwrap_or("none"),
            if report.api_key_present {
                "present"
            } else {
                "empty"
            },
            report.curl_command,
            if report.curl_available {
                "ok"
            } else {
                "missing"
            },
            report.chat_goal_refinement_enabled,
            report.chat_prefix,
            report.manifest_planning_enabled,
            report.manifest_prefix,
            report.evolution_planning_enabled,
            report.evolve_prefix
        )
    } else {
        format!(
            "not configured prefix={} ({}) curl={}({}) chat_refinement={}({}) manifest_planning={}({}) evolution_planning={}({})",
            report.config_prefix,
            report.message,
            report.curl_command,
            if report.curl_available {
                "ok"
            } else {
                "missing"
            },
            report.chat_goal_refinement_enabled,
            report.chat_prefix,
            report.manifest_planning_enabled,
            report.manifest_prefix,
            report.evolution_planning_enabled,
            report.evolve_prefix
        )
    }
}
