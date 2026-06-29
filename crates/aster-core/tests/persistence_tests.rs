use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use aster_core::{
    load_session_record, save_session_record, verify_session_directory, AsterError, EngineCommand,
    OrderId, OrderRequest, OrderType, ParticipantId, PersistenceError, PriceTicks, Quantity,
    SessionRecord, SessionVerificationError, Side, COMMANDS_FILE_NAME, EVENTS_FILE_NAME,
    SNAPSHOT_FILE_NAME,
};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

#[test]
fn save_creates_exact_files_with_one_record_per_jsonl_line() {
    let directory = TestDirectory::new("save-shape");
    let session = sample_session();

    save_session_record(directory.path(), &session).expect("session should save");

    let file_names: BTreeSet<String> = fs::read_dir(directory.path())
        .expect("session directory should be readable")
        .map(|entry| {
            entry
                .expect("directory entry should be readable")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(
        file_names,
        BTreeSet::from([
            COMMANDS_FILE_NAME.to_string(),
            EVENTS_FILE_NAME.to_string(),
            SNAPSHOT_FILE_NAME.to_string(),
        ])
    );
    assert_eq!(
        nonempty_line_count(&directory.path().join(COMMANDS_FILE_NAME)),
        session.commands.len()
    );
    assert_eq!(
        nonempty_line_count(&directory.path().join(EVENTS_FILE_NAME)),
        session.events.len()
    );
}

#[test]
fn loaded_session_equals_saved_domain_record() {
    let directory = TestDirectory::new("round-trip");
    let original = sample_session();
    save_session_record(directory.path(), &original).expect("session should save");

    let loaded = load_session_record(directory.path()).expect("session should load");

    assert_eq!(loaded, original);
}

#[test]
fn valid_saved_session_verifies_by_replay() {
    let directory = TestDirectory::new("verify");
    let original = sample_session();
    save_session_record(directory.path(), &original).expect("session should save");

    let verified = verify_session_directory(directory.path()).expect("saved session should verify");

    assert_eq!(verified, original);
}

#[test]
fn malformed_command_json_reports_line_number() {
    let directory = saved_sample("bad-command-json");
    let path = directory.path().join(COMMANDS_FILE_NAME);
    replace_line(&path, 2, "{not-json");

    assert!(matches!(
        load_session_record(directory.path()),
        Err(PersistenceError::JsonLine {
            record_kind: "command record",
            line: 2,
            ..
        })
    ));
}

#[test]
fn invalid_command_schema_reports_line_number() {
    let directory = saved_sample("bad-command-schema");
    let path = directory.path().join(COMMANDS_FILE_NAME);
    replace_line(
        &path,
        2,
        r#"{"schema_version":1,"command":{"type":"submit_order","participant_id":2,"side":"buy","order_type":{"type":"market"},"quantity":0}}"#,
    );

    assert!(matches!(
        load_session_record(directory.path()),
        Err(PersistenceError::SchemaLine {
            record_kind: "command record",
            line: 2,
            source: AsterError::ZeroQuantity,
            ..
        })
    ));
}

#[test]
fn malformed_event_json_reports_line_number() {
    let directory = saved_sample("bad-event-json");
    let path = directory.path().join(EVENTS_FILE_NAME);
    replace_line(&path, 3, "[]");

    assert!(matches!(
        load_session_record(directory.path()),
        Err(PersistenceError::JsonLine {
            record_kind: "event record",
            line: 3,
            ..
        })
    ));
}

#[test]
fn malformed_snapshot_json_is_rejected() {
    let directory = saved_sample("bad-snapshot-json");
    let path = directory.path().join(SNAPSHOT_FILE_NAME);
    fs::write(&path, "{broken").expect("test should corrupt snapshot");

    assert!(matches!(
        load_session_record(directory.path()),
        Err(PersistenceError::SnapshotJson { .. })
    ));
}

#[test]
fn missing_commands_file_is_rejected() {
    assert_missing_file(COMMANDS_FILE_NAME);
}

#[test]
fn missing_events_file_is_rejected() {
    assert_missing_file(EVENTS_FILE_NAME);
}

#[test]
fn missing_snapshot_file_is_rejected() {
    assert_missing_file(SNAPSHOT_FILE_NAME);
}

#[test]
fn persisted_event_mismatch_is_reported_distinctly() {
    let directory = TestDirectory::new("event-mismatch");
    let mut session = sample_session();
    session.events.pop();
    save_session_record(directory.path(), &session).expect("inconsistent session should save");

    assert!(matches!(
        verify_session_directory(directory.path()),
        Err(PersistenceError::Verification(
            SessionVerificationError::EventMismatch { .. }
        ))
    ));
}

#[test]
fn persisted_snapshot_mismatch_is_reported_distinctly() {
    let directory = TestDirectory::new("snapshot-mismatch");
    let mut session = sample_session();
    session.final_snapshot = SessionRecord::from_commands(Vec::new()).final_snapshot;
    save_session_record(directory.path(), &session).expect("inconsistent session should save");

    assert!(matches!(
        verify_session_directory(directory.path()),
        Err(PersistenceError::Verification(
            SessionVerificationError::SnapshotMismatch { .. }
        ))
    ));
}

fn assert_missing_file(file_name: &str) {
    let directory = saved_sample("missing-file");
    let missing_path = directory.path().join(file_name);
    fs::remove_file(&missing_path).expect("test should remove expected file");

    assert!(matches!(
        load_session_record(directory.path()),
        Err(PersistenceError::MissingFile { path }) if path == missing_path
    ));
}

fn saved_sample(name: &str) -> TestDirectory {
    let directory = TestDirectory::new(name);
    save_session_record(directory.path(), &sample_session()).expect("session should save");
    directory
}

fn sample_session() -> SessionRecord {
    SessionRecord::from_commands(vec![
        limit(1, Side::Sell, 101, 10),
        limit(2, Side::Buy, 101, 4),
        market(3, Side::Buy, 3),
        EngineCommand::cancel_order(OrderId::new(1), ParticipantId::new(1)),
        EngineCommand::cancel_order(OrderId::new(999), ParticipantId::new(9)),
    ])
}

fn limit(participant_id: u64, side: Side, price_ticks: u64, units: u64) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(participant_id),
        side,
        OrderType::Limit {
            price: PriceTicks::new(price_ticks).expect("test price must be positive"),
        },
        Quantity::new(units).expect("test quantity must be positive"),
    ))
}

fn market(participant_id: u64, side: Side, units: u64) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(participant_id),
        side,
        OrderType::Market,
        Quantity::new(units).expect("test quantity must be positive"),
    ))
}

fn nonempty_line_count(path: &Path) -> usize {
    fs::read_to_string(path)
        .expect("JSONL file should be readable")
        .lines()
        .filter(|line| !line.is_empty())
        .count()
}

fn replace_line(path: &Path, line_number: usize, replacement: &str) {
    let contents = fs::read_to_string(path).expect("JSONL file should be readable");
    let mut lines: Vec<&str> = contents.lines().collect();
    lines[line_number - 1] = replacement;
    fs::write(path, format!("{}\n", lines.join("\n"))).expect("test should rewrite JSONL file");
}

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(label: &str) -> Self {
        let unique = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "aster-persistence-{}-{label}-{unique}",
            std::process::id()
        ));
        if path.exists() {
            fs::remove_dir_all(&path).expect("stale test directory should be removable");
        }
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
