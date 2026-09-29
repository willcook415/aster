use crate::{report::render_scenario, scenario::Scenario};
use aster_core::{load_command_file, save_session_record, SessionRecord};

pub fn run_file(path: &str, output: Option<&str>) -> Result<String, String> {
    // Parse the entire input before processing or exporting any commands.
    let commands = load_command_file(path).map_err(|e| e.to_string())?;
    let session = SessionRecord::from_commands(commands.clone());
    session.verify().map_err(|e| e.to_string())?;
    if let Some(directory) = output {
        save_session_record(directory, &session).map_err(|e| e.to_string())?;
    }
    let scenario = Scenario {
        name: "custom-jsonl",
        description: "Strict V1 commands supplied by the caller",
        notes: &[],
        accounting: None,
        commands,
    };
    Ok(format!(
        "Input: {path}\n{}",
        render_scenario(&scenario, &session, &Ok(()))
    ))
}
