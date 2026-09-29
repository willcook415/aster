use aster_core::*;
use std::{fs, process::Command};

fn command() -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(1),
        Side::Buy,
        OrderType::Market,
        Quantity::new(3).unwrap(),
    ))
}

#[test]
fn recovery_preserves_real_book_fifo_allocators_and_next_results() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("journal");
    let commands = load_command_file(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/fifo.jsonl"
    ))
    .unwrap();
    let mut expected = SequencedEngine::new();
    let mut journal = CommandJournal::create(&path).unwrap();
    for command in commands {
        assert_eq!(
            journal.append(command).unwrap(),
            expected.process_command(command).unwrap()
        );
    }
    drop(journal);
    let mut journal = CommandJournal::open(&path).unwrap();
    assert_eq!(journal.snapshot(), expected.snapshot());
    assert_eq!(
        journal.append(command()).unwrap(),
        expected.process_command(command()).unwrap()
    );
}

#[test]
fn valid_checksum_does_not_bypass_schema_or_replay_validation() {
    use sha2::{Digest, Sha256};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("journal");
    let mut journal = CommandJournal::create(&path).unwrap();
    journal.append(command()).unwrap();
    drop(journal);
    let original = fs::read(&path).unwrap();
    let payload_end = original.len() - 40;
    let value: serde_json::Value = serde_json::from_slice(&original[24..payload_end]).unwrap();
    for variant in 0..3 {
        let mut modified = value.clone();
        match variant {
            0 => modified["matching_rules_version"] = 99.into(),
            1 => modified["events"][1]["event"]["remaining_quantity"] = 99.into(),
            _ => modified["unsupported"] = true.into(),
        }
        let payload = serde_json::to_vec(&modified).unwrap();
        let len = payload.len() as u32;
        let sizes = [len.to_le_bytes(), (!len).to_le_bytes()].concat();
        let mut hash = Sha256::new();
        hash.update([0; 32]);
        hash.update(&sizes);
        hash.update(&payload);
        let bytes = [
            &original[..16],
            &sizes,
            &payload,
            &hash.finalize(),
            b"ASTRCMIT",
        ]
        .concat();
        fs::write(&path, &bytes).unwrap();
        assert!(CommandJournal::open(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn round_trip_exclusive_lock_and_continue() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("journal");
    let mut journal = CommandJournal::create(&path).unwrap();
    journal.append(command()).unwrap();
    assert!(CommandJournal::open(&path).is_err());
    assert!(recover_journal(&path).is_err());
    assert!(CommandJournal::create(&path).is_err());
    let snapshot = journal.snapshot();
    drop(journal);
    let report = recover_journal(&path).unwrap();
    assert_eq!(report.command_count, 1);
    assert_eq!(report.event_count, 3);
    assert_eq!(report.snapshot, snapshot);
    let mut journal = CommandJournal::open(&path).unwrap();
    let batch = journal.append(command()).unwrap();
    assert_eq!(batch.command_sequence, 2);
    assert_eq!(batch.events[0].event_sequence, 4);
}

#[test]
fn every_incomplete_final_frame_recovers_only_the_committed_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    let mut journal = CommandJournal::create(&source).unwrap();
    journal.append(command()).unwrap();
    drop(journal);
    let prefix = fs::metadata(&source).unwrap().len() as usize;
    let mut journal = CommandJournal::open(&source).unwrap();
    journal.append(command()).unwrap();
    drop(journal);
    let bytes = fs::read(&source).unwrap();
    let path = dir.path().join("torn");
    for end in prefix..bytes.len() {
        fs::write(&path, &bytes[..end]).unwrap();
        let report = recover_journal(&path).unwrap();
        assert_eq!(report.command_count, 1, "cut at {end}");
        assert_eq!(report.incomplete_tail_bytes, (end - prefix) as u64);
        assert_eq!(fs::metadata(&path).unwrap().len(), end as u64);
    }
    let mut journal = CommandJournal::open(&path).unwrap();
    assert!(journal.recovered_tail_bytes() > 0);
    assert_eq!(journal.append(command()).unwrap().command_sequence, 2);
    drop(journal);
    let report = recover_journal(&path).unwrap();
    assert_eq!(report.command_count, 2);
    assert_eq!(report.incomplete_tail_bytes, 0);
}

#[test]
fn complete_corruption_is_rejected_without_modification() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("journal");
    let mut journal = CommandJournal::create(&path).unwrap();
    journal.append(command()).unwrap();
    drop(journal);
    let bytes = fs::read(&path).unwrap();
    // Header, redundant length, payload, digest, and commit marker.
    for offset in [0, 20, 30, bytes.len() - 9, bytes.len() - 1] {
        let mut corrupt = bytes.clone();
        corrupt[offset] ^= 1;
        fs::write(&path, &corrupt).unwrap();
        assert!(CommandJournal::open(&path).is_err(), "offset {offset}");
        assert_eq!(fs::read(&path).unwrap(), corrupt);
    }
}

#[test]
fn removed_or_reordered_frames_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("journal");
    let mut journal = CommandJournal::create(&path).unwrap();
    journal.append(command()).unwrap();
    drop(journal);
    let first = fs::read(&path).unwrap();
    let mut journal = CommandJournal::open(&path).unwrap();
    journal.append(command()).unwrap();
    drop(journal);
    let all = fs::read(&path).unwrap();
    let second = &all[first.len()..];
    let removed = [&all[..16], second].concat();
    let reordered = [&all[..16], second, &first[16..]].concat();
    for bytes in [removed, reordered] {
        fs::write(&path, bytes).unwrap();
        assert!(recover_journal(&path).is_err());
    }
}

#[test]
fn child_crash_writer() {
    if let Some(path) = std::env::var_os("ASTER_CRASH_TEST_PATH") {
        let mut journal = CommandJournal::create(path).unwrap();
        journal.append(command()).unwrap();
        // Skip destructors: the process and its OS file lock disappear together.
        std::process::exit(99);
    }
}

#[test]
fn acknowledged_command_survives_process_exit_without_drop() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("journal");
    let status = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "child_crash_writer"])
        .env("ASTER_CRASH_TEST_PATH", &path)
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(99));
    assert_eq!(recover_journal(&path).unwrap().command_count, 1);
    assert_eq!(
        CommandJournal::open(&path).unwrap().next_command_sequence(),
        2
    );
}
