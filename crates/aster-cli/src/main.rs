mod report;
mod scenario;

use std::env;
use std::process::ExitCode;

use aster_core::SessionRecord;

use report::render_scenario;
use scenario::{find_scenario, scenarios};

const DEFAULT_SCENARIO: &str = "mixed-session";

fn main() -> ExitCode {
    match execute(env::args().skip(1)) {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}\n");
            eprint!("{}", usage());
            ExitCode::from(2)
        }
    }
}

fn execute<I, S>(args: I) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();

    match args.as_slice() {
        [] => run_scenario(DEFAULT_SCENARIO),
        [command] if command == "list" || command == "list-scenarios" => Ok(render_scenario_list()),
        [command] if command == "help" || command == "--help" || command == "-h" => Ok(usage()),
        [command, name] if command == "scenario" => run_scenario(name),
        [command, ..] if command == "scenario" => {
            Err("usage: aster-cli scenario <scenario-name>".to_string())
        }
        [unknown, ..] => Err(format!("unknown command '{unknown}'")),
    }
}

fn run_scenario(name: &str) -> Result<String, String> {
    let scenario = find_scenario(name).ok_or_else(|| format!("unknown scenario '{name}'"))?;
    let session = SessionRecord::from_commands(scenario.commands.clone());
    let verification = session.verify();

    Ok(render_scenario(&scenario, &session, &verification))
}

fn render_scenario_list() -> String {
    let mut output = String::from("Aster built-in deterministic scenarios\n\n");
    for scenario in scenarios() {
        output.push_str(&format!(
            "  {:<20} {}\n",
            scenario.name, scenario.description
        ));
    }
    output.push_str("\nRun: cargo run -p aster-cli -- scenario <name>\n");
    output
}

fn usage() -> String {
    format!(
        "Aster deterministic scenario runner\n\n\
         Usage:\n\
           aster-cli                         Run the default '{DEFAULT_SCENARIO}' scenario\n\
           aster-cli list                    List built-in scenarios\n\
           aster-cli scenario <name>         Run a named scenario\n\
           aster-cli help                    Show this help\n"
    )
}

#[cfg(test)]
mod tests {
    use super::{execute, DEFAULT_SCENARIO};

    #[test]
    fn no_args_runs_stable_default_scenario() {
        let output = execute(Vec::<String>::new()).expect("default scenario should run");

        assert!(output.contains(&format!("Scenario: {DEFAULT_SCENARIO}")));
        assert!(output.contains("Session verified: YES"));
    }

    #[test]
    fn list_contains_all_expected_scenarios() {
        let output = execute(["list"]).expect("scenario list should render");

        for name in [
            "fifo-partial-fill",
            "market-sweep",
            "cancellation",
            "mixed-session",
        ] {
            assert!(output.contains(name), "missing scenario {name}");
        }
    }

    #[test]
    fn unknown_scenario_returns_clear_error() {
        assert_eq!(
            execute(["scenario", "does-not-exist"]),
            Err("unknown scenario 'does-not-exist'".to_string())
        );
    }

    #[test]
    fn every_scenario_output_contains_report_sections() {
        for name in [
            "fifo-partial-fill",
            "market-sweep",
            "cancellation",
            "mixed-session",
        ] {
            let output = execute(["scenario", name]).expect("known scenario should run");

            for section in [
                "What happened",
                "Commands",
                "Events",
                "Event Summary",
                "Final Book",
                "Verification",
            ] {
                assert!(output.contains(section), "{name} missing section {section}");
            }
        }
    }

    #[test]
    fn market_sweep_reports_expired_remainder_explicitly() {
        let output = execute(["scenario", "market-sweep"]).expect("known scenario should run");

        assert!(output.contains("Accounting"));
        assert!(output.contains("accepted market quantity: 15"));
        assert!(output.contains("traded quantity: 12"));
        assert!(output.contains("expired market remainder: 3"));
    }

    #[test]
    fn help_output_keeps_stable_usage_commands() {
        let output = execute(["help"]).expect("help should render");

        assert!(output.contains("aster-cli list"));
        assert!(output.contains("aster-cli scenario <name>"));
        assert!(output.contains("aster-cli help"));
    }
}
