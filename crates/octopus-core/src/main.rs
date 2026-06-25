use octopus_core::{Harness, HarnessState, Need, NeedKind};
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

fn usage() -> String {
    "usage: octopus-core [--state path] [--lang en|zh] [--json] need <kind> <query> | routes"
        .to_string()
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
}
