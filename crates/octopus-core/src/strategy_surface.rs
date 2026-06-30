use super::*;

pub(crate) fn strategy_diagnostics_report(
    state_path: &Path,
    state: &HarnessState,
) -> Result<diagnostics::StrategyDiagnosticReport, String> {
    let manifests =
        inspect_tentacle_manifests_with_bundled_seed_overlay(&default_tentacles_root())?;
    Ok(diagnostics::strategy_diagnostics(
        state_path, &manifests, state,
    ))
}

pub(crate) fn print_strategy_diagnostics(
    report: &diagnostics::StrategyDiagnosticReport,
    language: Language,
) {
    match language {
        Language::En => {
            println!("Octopus strategy diagnostics");
            println!("status: {}", report.status);
            for check in &report.checks {
                println!("{}: {} - {}", check.id, check.status, check.evidence);
                if check.status != "pass" {
                    println!("next: {}", check.next);
                }
            }
        }
        Language::Zh => {
            println!("章鱼策略诊断");
            println!("状态: {}", report.status);
            for check in &report.checks {
                println!("{}: {} - {}", check.id, check.status, check.evidence);
                if check.status != "pass" {
                    println!("下一步: {}", check.next);
                }
            }
        }
    }
}
