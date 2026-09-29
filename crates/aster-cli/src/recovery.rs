use aster_core::{load_command_file, recover_journal, CommandJournal, SequencedEngine};

pub fn run_v2(path: &str) -> Result<String, String> {
    let commands = load_command_file(path).map_err(|e| e.to_string())?;
    let mut engine = SequencedEngine::new();
    let mut output = String::new();
    for command in commands {
        let batch = engine.process_command(command).map_err(|e| e.to_string())?;
        output.push_str(&serde_json::to_string(&batch).map_err(|e| e.to_string())?);
        output.push('\n');
    }
    Ok(output)
}

pub fn append(input: &str, path: &str, resume: bool) -> Result<String, String> {
    let commands = load_command_file(input).map_err(|e| e.to_string())?;
    let mut journal = if resume {
        CommandJournal::open(path)
    } else {
        CommandJournal::create(path)
    }
    .map_err(|e| e.to_string())?;
    let tail = journal.recovered_tail_bytes();
    let count = commands.len();
    for command in commands {
        journal.append(command).map_err(|e| e.to_string())?;
    }
    Ok(format!("Journal: {path}\nCommands synchronized: {count}\nNext command sequence: {}\nIncomplete tail bytes removed: {tail}\n", journal.next_command_sequence()))
}

pub fn recover(path: &str) -> Result<String, String> {
    let report = recover_journal(path).map_err(|e| e.to_string())?;
    Ok(format!("Aster journal recovery (read only)\nJournal: {path}\nCommands: {}\nEvents: {}\nCommitted bytes: {}\nIncomplete tail bytes: {}\nReplay verified: YES\nFinal snapshot: {:?}\n",
        report.command_count, report.event_count, report.committed_bytes,
        report.incomplete_tail_bytes, report.snapshot))
}
