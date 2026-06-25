use octopus_core::{
    default_permissions, default_tentacle_profiles, CapabilityGrant, EnvironmentReport, GoalChat,
    Harness, HarnessState, Need, NeedKind, SelfIterationPlan,
};
use std::env;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Language {
    En,
    Zh,
}

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err(usage());
    }

    let mut state = PathBuf::from(".octopus/state.json");
    let mut json = false;
    let mut language = Language::En;
    let mut rest = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--state" => {
                index += 1;
                let Some(path) = args.get(index) else {
                    return Err("--state requires a path".to_string());
                };
                state = PathBuf::from(path);
            }
            "--lang" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("--lang requires en or zh".to_string());
                };
                language = parse_language(value)?;
            }
            "--json" => json = true,
            value => rest.push(value.to_string()),
        }
        index += 1;
    }

    match rest.first().map(String::as_str) {
        Some("need") => {
            let kind = rest
                .get(1)
                .ok_or_else(|| "need requires a kind".to_string())
                .and_then(|value| parse_kind(value))?;
            let query = rest
                .get(2..)
                .filter(|values| !values.is_empty())
                .map(|values| values.join(" "))
                .ok_or_else(|| "need requires a query".to_string())?;
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let mut harness = Harness::with_state(loaded);
            let feedback = harness.feed(&[Need::new(kind, query)]);
            harness
                .state
                .save(&state)
                .map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&feedback).map_err(|error| error.to_string())?
                );
            } else {
                println!("{}", localize_summary(&feedback.summary, language));
            }
            Ok(())
        }
        Some("routes") => {
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&loaded.routes.scores)
                    .map_err(|error| error.to_string())?
            );
            Ok(())
        }
        Some("chat") => {
            let message = rest
                .get(1..)
                .filter(|values| !values.is_empty())
                .map(|values| values.join(" "))
                .ok_or_else(|| "chat requires a message".to_string())?;
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let mut harness = Harness::with_state(loaded);
            let chat = harness.chat(message);
            harness
                .state
                .save(&state)
                .map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&chat).map_err(|error| error.to_string())?
                );
            } else {
                print_chat(&chat, language);
            }
            Ok(())
        }
        Some("goal") => {
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
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
        Some("oauth") => {
            let provider = rest
                .get(1)
                .ok_or_else(|| "oauth requires a provider".to_string())?;
            if provider == "revoke" {
                let grant = rest
                    .get(2)
                    .ok_or_else(|| "oauth revoke requires a grant id".to_string())?;
                let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
                if !loaded.revoke_grant(grant) {
                    return Err(format!("unknown grant: {grant}"));
                }
                loaded.save(&state).map_err(|error| error.to_string())?;
                match language {
                    Language::En => println!("revoked {grant}"),
                    Language::Zh => println!("已撤销 {grant}"),
                }
                return Ok(());
            }
            let scope = rest
                .get(2)
                .ok_or_else(|| "oauth requires a scope".to_string())?;
            let permissions = rest
                .get(3..)
                .filter(|values| !values.is_empty())
                .map_or_else(
                    || default_permissions(provider),
                    |values| values.iter().map(String::from).collect(),
                );
            let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let grant = loaded.grant_oauth(provider, scope, permissions);
            loaded.save(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&grant).map_err(|error| error.to_string())?
                );
            } else {
                print_grant(&grant, language);
            }
            Ok(())
        }
        Some("self-iterate") => {
            let repository = rest
                .get(1)
                .ok_or_else(|| "self-iterate requires a repository".to_string())?;
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            let plan = loaded.self_iteration_plan(repository.as_str());
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&plan).map_err(|error| error.to_string())?
                );
            } else {
                print_self_iteration_plan(&plan, language);
            }
            Ok(())
        }
        Some("catalog") => {
            let profiles = default_tentacle_profiles();
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&profiles).map_err(|error| error.to_string())?
                );
            } else {
                print_catalog(&profiles, language);
            }
            Ok(())
        }
        Some("env") => {
            let cwd = env::current_dir().map_err(|error| error.to_string())?;
            let report = EnvironmentReport::detect(cwd);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
                );
            } else {
                print_environment(&report, language);
            }
            Ok(())
        }
        Some("install") => {
            let profile = rest
                .get(1)
                .ok_or_else(|| "install requires a profile id".to_string())?;
            let mut loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            loaded.install_profile(profile)?;
            loaded.save(&state).map_err(|error| error.to_string())?;
            match language {
                Language::En => println!("installed {profile}"),
                Language::Zh => println!("已安装 {profile}"),
            }
            Ok(())
        }
        Some("installed") => {
            let loaded = HarnessState::load(&state).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&loaded.installed_profiles)
                        .map_err(|error| error.to_string())?
                );
            } else if loaded.installed_profiles.is_empty() {
                match language {
                    Language::En => println!("no tentacles installed"),
                    Language::Zh => println!("尚未安装触手"),
                }
            } else {
                println!("{}", loaded.installed_profiles.join(", "));
            }
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn parse_language(value: &str) -> Result<Language, String> {
    match value {
        "en" => Ok(Language::En),
        "zh" | "zh-CN" | "zh_Hans" => Ok(Language::Zh),
        _ => Err(format!("unknown language: {value}")),
    }
}

fn parse_kind(value: &str) -> Result<NeedKind, String> {
    match value {
        "verify" => Ok(NeedKind::Verify),
        "reproduce" => Ok(NeedKind::Reproduce),
        "compare" => Ok(NeedKind::Compare),
        "remember" => Ok(NeedKind::Remember),
        "forget" => Ok(NeedKind::Forget),
        "execute" => Ok(NeedKind::Execute),
        "recall" => Ok(NeedKind::Recall),
        "observe" => Ok(NeedKind::Observe),
        _ => Err(format!("unknown need kind: {value}")),
    }
}

fn localize_summary(summary: &str, language: Language) -> String {
    if language == Language::En {
        return summary.to_string();
    }
    if let Some(id) = summary.strip_prefix("remembered ") {
        return format!("已记住 {id}");
    }
    if let Some(count) = summary
        .strip_prefix("forgot ")
        .and_then(|value| value.strip_suffix(" memories"))
    {
        return format!("已遗忘 {count} 条记忆");
    }
    match summary {
        "nothing recalled" => "没有召回内容".to_string(),
        "no tentacle supports this need" => "没有触手支持这个需求".to_string(),
        _ => summary.to_string(),
    }
}

fn print_catalog(profiles: &[octopus_core::TentacleProfile], language: Language) {
    match language {
        Language::En => println!("Installable tentacles:"),
        Language::Zh => println!("可安装触手:"),
    }
    for profile in profiles {
        println!("- {}: {}", profile.id, profile.description);
    }
}

fn print_chat(chat: &GoalChat, language: Language) {
    match language {
        Language::En => {
            println!("goal: {}", chat.goal.objective);
            println!("turn {}: {}", chat.turn.index, chat.turn.summary);
            println!("refinements: {}", chat.goal.constraints.len());
        }
        Language::Zh => {
            println!("目标: {}", chat.goal.objective);
            println!(
                "第 {} 轮: {}",
                chat.turn.index,
                localize_summary(&chat.turn.summary, language)
            );
            println!("调整: {}", chat.goal.constraints.len());
        }
    }
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

fn print_grant(grant: &CapabilityGrant, language: Language) {
    match language {
        Language::En => {
            println!("oauth grant active: {}", grant.id);
            println!("permissions: {}", grant.permissions.join(", "));
        }
        Language::Zh => {
            println!("OAuth 授权已启用: {}", grant.id);
            println!("权限: {}", grant.permissions.join(", "));
        }
    }
}

fn print_self_iteration_plan(plan: &SelfIterationPlan, language: Language) {
    match language {
        Language::En => {
            println!("repo: {}", plan.repository);
            println!("mode: {}", plan.mode);
            println!("authorized: {}", plan.authorized);
            println!("next: {}", plan.steps.join(" -> "));
        }
        Language::Zh => {
            println!("仓库: {}", plan.repository);
            println!("模式: {}", plan.mode);
            println!("已授权: {}", plan.authorized);
            println!("下一步: {}", plan.steps.join(" -> "));
        }
    }
}

fn print_environment(report: &EnvironmentReport, language: Language) {
    match language {
        Language::En => {
            println!("Environment:");
            println!("manifests: {}", report.manifests.join(", "));
            println!("commands: {}", report.commands.join(", "));
            println!("recommended: {}", report.recommended_profiles.join(", "));
        }
        Language::Zh => {
            println!("环境:");
            println!("项目特征: {}", report.manifests.join(", "));
            println!("可用命令: {}", report.commands.join(", "));
            println!("推荐触手: {}", report.recommended_profiles.join(", "));
        }
    }
}

fn usage() -> String {
    "usage: octopus-core [--state path] [--lang en|zh] [--json] need <kind> <query> | chat <message> | goal | oauth <provider> <scope> [permissions...] | oauth revoke <grant> | self-iterate <repo> | routes | catalog | env | install <profile> | installed".to_string()
}

#[cfg(test)]
mod tests {
    use super::{localize_summary, run, Language};
    use std::fs;

    #[test]
    fn cli_persists_memory_between_runs() {
        let path =
            std::env::temp_dir().join(format!("octopus-cli-state-{}.json", std::process::id()));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "need".to_string(),
            "remember".to_string(),
            "clean".to_string(),
            "brain".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state,
            "need".to_string(),
            "recall".to_string(),
            "clean".to_string(),
        ])
        .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("clean brain"));
        assert!(content.contains("recall:memory"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_rejects_unknown_need_kind() {
        let error = run(vec![
            "need".to_string(),
            "unknown".to_string(),
            "query".to_string(),
        ])
        .unwrap_err();

        assert!(error.contains("unknown need kind"));
    }

    #[test]
    fn cli_localizes_builtin_feedback() {
        assert_eq!(localize_summary("remembered m1", Language::Zh), "已记住 m1");
        assert_eq!(
            localize_summary("forgot 2 memories", Language::Zh),
            "已遗忘 2 条记忆"
        );
        assert_eq!(
            localize_summary("nothing recalled", Language::Zh),
            "没有召回内容"
        );
    }

    #[test]
    fn cli_catalog_and_env_commands_run() {
        run(vec!["catalog".to_string()]).unwrap();
        run(vec![
            "--lang".to_string(),
            "zh".to_string(),
            "env".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn cli_installs_profile_into_state() {
        let path =
            std::env::temp_dir().join(format!("octopus-install-state-{}.json", std::process::id()));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "install".to_string(),
            "research".to_string(),
        ])
        .unwrap();
        run(vec!["--state".to_string(), state, "installed".to_string()]).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("research"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_chat_refines_goal() {
        let path =
            std::env::temp_dir().join(format!("octopus-chat-state-{}.json", std::process::id()));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "chat".to_string(),
            "build".to_string(),
            "octopus".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "chat".to_string(),
            "make".to_string(),
            "tools".to_string(),
            "think".to_string(),
        ])
        .unwrap();
        run(vec!["--state".to_string(), state, "goal".to_string()]).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"objective\": \"build octopus\""));
        assert!(content.contains("make tools think"));
        assert!(content.contains("goal_turns"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cli_oauth_unlocks_self_iteration_plan() {
        let path =
            std::env::temp_dir().join(format!("octopus-oauth-state-{}.json", std::process::id()));
        let state = path.to_string_lossy().to_string();
        let _ = fs::remove_file(&path);

        run(vec![
            "--state".to_string(),
            state.clone(),
            "self-iterate".to_string(),
            "dangoZhang/Octopus".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "oauth".to_string(),
            "github".to_string(),
            "dangoZhang/Octopus".to_string(),
        ])
        .unwrap();
        run(vec![
            "--state".to_string(),
            state.clone(),
            "self-iterate".to_string(),
            "dangoZhang/Octopus".to_string(),
        ])
        .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("github:dangoZhang/Octopus"));
        assert!(content.contains("pull_request:write"));
        let _ = fs::remove_file(path);
    }
}
