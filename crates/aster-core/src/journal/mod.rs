//! Single-writer, sync-before-acknowledgement command journal.
//!
//! Complete frames are verified by checksum, sequence and deterministic replay.
//! Only an incomplete final frame is discarded when opening for append. Read-only
//! recovery reports that tail without changing the file. See docs/recovery.md.
mod error;
mod frame;
use crate::{CommandBatchV2, EngineCommand, EngineSnapshot, SequencedEngine};
pub use error::JournalError;
use std::{
    fs::{File, TryLockError},
    io::{Seek, SeekFrom, Write},
    path::Path,
};

/// Recovery evidence. A recovered complete frame may not have been acknowledged
/// before process termination; this API does not provide request deduplication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryReport {
    pub snapshot: EngineSnapshot,
    pub command_count: u64,
    pub event_count: u64,
    pub committed_bytes: u64,
    pub incomplete_tail_bytes: u64,
}
impl RecoveryReport {
    fn from_scan(scan: &frame::Scan) -> Self {
        Self {
            snapshot: scan.engine.snapshot(),
            command_count: scan.engine.next_command_sequence() - 1,
            event_count: scan.engine.next_event_sequence() - 1,
            committed_bytes: scan.committed_bytes,
            incomplete_tail_bytes: scan.tail_bytes,
        }
    }
}

/// Exclusive journal owner. Locks are released by the OS when the file closes.
#[derive(Debug)]
pub struct CommandJournal {
    file: File,
    engine: SequencedEngine,
    digest: [u8; 32],
    committed_bytes: u64,
    recovered_tail_bytes: u64,
    poisoned: bool,
}

fn lock(file: &File) -> Result<(), JournalError> {
    file.try_lock().map_err(|e| match e {
        TryLockError::WouldBlock => JournalError::Locked,
        TryLockError::Error(e) => JournalError::Io(e),
    })
}

impl CommandJournal {
    /// Creates a new journal without overwriting an existing path.
    pub fn create(path: impl AsRef<Path>) -> Result<Self, JournalError> {
        let mut file = File::create_new(path)?;
        lock(&file)?;
        file.write_all(frame::HEADER)?;
        file.sync_all()?;
        Ok(Self {
            file,
            engine: SequencedEngine::new(),
            digest: [0; 32],
            committed_bytes: 16,
            recovered_tail_bytes: 0,
            poisoned: false,
        })
    }

    /// Replays committed frames and truncates only an incomplete final frame.
    /// Corrupt complete frames fail without modifying the file.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, JournalError> {
        let mut file = File::options().read(true).write(true).open(path)?;
        lock(&file)?;
        let scan = frame::scan(&mut file)?;
        if scan.tail_bytes > 0 {
            file.set_len(scan.committed_bytes)?;
            file.sync_all()?;
        }
        file.seek(SeekFrom::Start(scan.committed_bytes))?;
        Ok(Self {
            file,
            engine: scan.engine,
            digest: scan.digest,
            committed_bytes: scan.committed_bytes,
            recovered_tail_bytes: scan.tail_bytes,
            poisoned: false,
        })
    }

    pub fn snapshot(&self) -> EngineSnapshot {
        self.engine.snapshot()
    }
    pub fn next_command_sequence(&self) -> u64 {
        self.engine.next_command_sequence()
    }
    pub fn recovered_tail_bytes(&self) -> u64 {
        self.recovered_tail_bytes
    }

    /// Stages state on a clone, writes a checksummed frame, synchronizes it, then
    /// installs state and returns the response. Any I/O failure poisons this handle.
    pub fn append(&mut self, command: EngineCommand) -> Result<CommandBatchV2, JournalError> {
        if self.poisoned {
            return Err(JournalError::Poisoned);
        }
        let mut candidate = self.engine.clone();
        let batch = candidate.process_command(command)?;
        let (bytes, digest) = frame::encode(&batch, &self.digest)?;
        // Never continue writing through a handle whose previous outcome is uncertain.
        self.poisoned = true;
        if self.file.metadata()?.len() != self.committed_bytes {
            return Err(JournalError::Corrupt {
                offset: self.committed_bytes,
                reason: "file changed outside the journal owner".into(),
            });
        }
        frame::write_frame(&mut self.file, &bytes)?;
        self.engine = candidate;
        self.digest = digest;
        self.committed_bytes += bytes.len() as u64;
        self.poisoned = false;
        Ok(batch)
    }
}

/// Verifies a journal without truncating an incomplete tail or creating a file.
pub fn recover_journal(path: impl AsRef<Path>) -> Result<RecoveryReport, JournalError> {
    let mut file = File::open(path)?;
    lock(&file)?;
    let scan = frame::scan(&mut file)?;
    Ok(RecoveryReport::from_scan(&scan))
}
