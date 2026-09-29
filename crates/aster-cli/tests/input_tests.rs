use std::{path::PathBuf, process::Command};

#[test]
fn custom_input_executes_real_matching_and_replay() {
    let input = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/fifo.jsonl");
    let output = Command::new(env!("CARGO_BIN_EXE_aster-cli"))
        .arg("run")
        .arg(input)
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("TradeExecuted resting_id=1 incoming_id=3 price=101 quantity=10"));
    assert!(text.contains("remaining=5"));
    assert!(text.contains("Session verified: YES"));
}

#[test]
fn missing_input_returns_nonzero_with_path() {
    let output = Command::new(env!("CARGO_BIN_EXE_aster-cli"))
        .args(["run", "nonexistent-aster-commands.jsonl"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("nonexistent-aster-commands.jsonl"));
}

#[test]
fn v2_cli_emits_correlated_expiry_batches() {
    let input =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/market-expiry.jsonl");
    let output = Command::new(env!("CARGO_BIN_EXE_aster-cli"))
        .arg("run-v2")
        .arg(input)
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let batches: Vec<aster_core::CommandBatchV2> = text
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let mut replay = aster_core::SequencedEngine::new();
    for batch in &batches {
        replay.verify_batch(batch).unwrap();
    }
    assert_eq!(batches.len(), 2);
    assert!(text.contains("order_expired"));
    assert!(text.contains("\"remaining_quantity\":3"));
}

#[test]
fn journal_cli_creates_recovers_resumes_and_refuses_overwrite() {
    let input = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/fifo.jsonl");
    let directory = std::env::temp_dir().join(format!(
        "aster-cli-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("journal");
    let invoke = |operation: &str| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_aster-cli"));
        cmd.arg(operation);
        if operation != "recover" {
            cmd.arg(&input);
        }
        cmd.arg(&path).output().unwrap()
    };
    assert!(invoke("journal").status.success());
    let before = std::fs::read(&path).unwrap();
    assert_eq!(invoke("journal").status.code(), Some(2));
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(invoke("recover").status.success());
    assert!(invoke("resume").status.success());
    assert_eq!(aster_core::recover_journal(&path).unwrap().command_count, 6);
    std::fs::write(&path, b"corrupt").unwrap();
    assert_eq!(invoke("recover").status.code(), Some(2));
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
