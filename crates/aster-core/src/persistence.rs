//! Deterministic complete-file persistence for finished in-memory sessions.
//!
//! Command records are canonical replay input. Event and snapshot records are
//! saved audit outputs used to verify replay, not authoritative recovery state.

use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{
    AsterError, CommandRecordV1, EngineCommand, EngineEvent, EngineSnapshot, EventRecordV1,
    SessionRecord, SessionVerificationError, SnapshotRecordV1,
};

pub const COMMANDS_FILE_NAME: &str = "commands.jsonl";
pub const EVENTS_FILE_NAME: &str = "events.jsonl";
pub const SNAPSHOT_FILE_NAME: &str = "snapshot.json";

/// Failure while saving, loading, or verifying a persisted session.
#[derive(Debug)]
pub enum PersistenceError {
    MissingFile {
        path: PathBuf,
    },
    Io {
        operation: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    Serialize {
        record_kind: &'static str,
        path: PathBuf,
        source: serde_json::Error,
    },
    JsonLine {
        record_kind: &'static str,
        path: PathBuf,
        line: usize,
        source: serde_json::Error,
    },
    SchemaLine {
        record_kind: &'static str,
        path: PathBuf,
        line: usize,
        source: AsterError,
    },
    SnapshotJson {
        path: PathBuf,
        source: serde_json::Error,
    },
    SnapshotSchema {
        path: PathBuf,
        source: AsterError,
    },
    Verification(SessionVerificationError),
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFile { path } => {
                write!(f, "expected session file is missing: {}", path.display())
            }
            Self::Io {
                operation, path, ..
            } => write!(f, "failed to {operation} {}", path.display()),
            Self::Serialize {
                record_kind, path, ..
            } => write!(f, "failed to serialize {record_kind} to {}", path.display()),
            Self::JsonLine {
                record_kind,
                path,
                line,
                ..
            } => write!(
                f,
                "malformed {record_kind} JSON at {} line {line}",
                path.display()
            ),
            Self::SchemaLine {
                record_kind,
                path,
                line,
                source,
            } => write!(
                f,
                "invalid {record_kind} schema at {} line {line}: {source}",
                path.display()
            ),
            Self::SnapshotJson { path, .. } => {
                write!(f, "malformed snapshot JSON at {}", path.display())
            }
            Self::SnapshotSchema { path, source } => {
                write!(f, "invalid snapshot schema at {}: {source}", path.display())
            }
            Self::Verification(source) => {
                write!(f, "persisted session verification failed: {source}")
            }
        }
    }
}

impl Error for PersistenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingFile { .. } => None,
            Self::Io { source, .. } => Some(source),
            Self::Serialize { source, .. } => Some(source),
            Self::JsonLine { source, .. } => Some(source),
            Self::SchemaLine { source, .. } => Some(source),
            Self::SnapshotJson { source, .. } => Some(source),
            Self::SnapshotSchema { source, .. } => Some(source),
            Self::Verification(source) => Some(source),
        }
    }
}

/// Saves a complete session using V1 command/event JSONL and snapshot JSON.
pub fn save_session_record(
    directory: impl AsRef<Path>,
    session: &SessionRecord,
) -> Result<(), PersistenceError> {
    let directory = directory.as_ref();
    fs::create_dir_all(directory).map_err(|source| PersistenceError::Io {
        operation: "create session directory",
        path: directory.to_path_buf(),
        source,
    })?;

    write_json_lines(
        &directory.join(COMMANDS_FILE_NAME),
        "command record",
        session.commands.iter().copied().map(CommandRecordV1::from),
    )?;
    write_json_lines(
        &directory.join(EVENTS_FILE_NAME),
        "event record",
        session.events.iter().copied().map(EventRecordV1::from),
    )?;
    write_snapshot(
        &directory.join(SNAPSHOT_FILE_NAME),
        SnapshotRecordV1::from(session.final_snapshot.clone()),
    )
}

/// Loads a complete persisted session and validates every schema record.
pub fn load_session_record(directory: impl AsRef<Path>) -> Result<SessionRecord, PersistenceError> {
    let directory = directory.as_ref();
    let commands_path = directory.join(COMMANDS_FILE_NAME);
    let events_path = directory.join(EVENTS_FILE_NAME);
    let snapshot_path = directory.join(SNAPSHOT_FILE_NAME);

    let commands =
        read_json_lines::<CommandRecordV1, EngineCommand>(&commands_path, "command record")?;
    let events = read_json_lines::<EventRecordV1, EngineEvent>(&events_path, "event record")?;
    let final_snapshot = read_snapshot(&snapshot_path)?;

    Ok(SessionRecord {
        commands,
        events,
        final_snapshot,
    })
}

/// Loads and verifies a persisted session by replaying its canonical commands.
pub fn verify_session_directory(
    directory: impl AsRef<Path>,
) -> Result<SessionRecord, PersistenceError> {
    let session = load_session_record(directory)?;
    session.verify().map_err(PersistenceError::Verification)?;
    Ok(session)
}

fn write_json_lines<T>(
    path: &Path,
    record_kind: &'static str,
    records: impl IntoIterator<Item = T>,
) -> Result<(), PersistenceError>
where
    T: Serialize,
{
    let file = create_file(path)?;
    let mut writer = BufWriter::new(file);
    for record in records {
        serde_json::to_writer(&mut writer, &record).map_err(|source| {
            PersistenceError::Serialize {
                record_kind,
                path: path.to_path_buf(),
                source,
            }
        })?;
        writer
            .write_all(b"\n")
            .map_err(|source| PersistenceError::Io {
                operation: "write",
                path: path.to_path_buf(),
                source,
            })?;
    }
    writer.flush().map_err(|source| PersistenceError::Io {
        operation: "flush",
        path: path.to_path_buf(),
        source,
    })
}

fn write_snapshot(path: &Path, record: SnapshotRecordV1) -> Result<(), PersistenceError> {
    let file = create_file(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &record).map_err(|source| {
        PersistenceError::Serialize {
            record_kind: "snapshot record",
            path: path.to_path_buf(),
            source,
        }
    })?;
    writer
        .write_all(b"\n")
        .and_then(|()| writer.flush())
        .map_err(|source| PersistenceError::Io {
            operation: "write snapshot",
            path: path.to_path_buf(),
            source,
        })
}

fn create_file(path: &Path) -> Result<File, PersistenceError> {
    File::create(path).map_err(|source| PersistenceError::Io {
        operation: "create",
        path: path.to_path_buf(),
        source,
    })
}

fn read_json_lines<R, T>(path: &Path, record_kind: &'static str) -> Result<Vec<T>, PersistenceError>
where
    R: DeserializeOwned,
    T: TryFrom<R, Error = AsterError>,
{
    let reader = BufReader::new(open_expected_file(path)?);
    reader
        .lines()
        .enumerate()
        .map(|(index, line)| {
            let line_number = index + 1;
            let line = line.map_err(|source| PersistenceError::Io {
                operation: "read",
                path: path.to_path_buf(),
                source,
            })?;
            let record: R =
                serde_json::from_str(&line).map_err(|source| PersistenceError::JsonLine {
                    record_kind,
                    path: path.to_path_buf(),
                    line: line_number,
                    source,
                })?;
            T::try_from(record).map_err(|source| PersistenceError::SchemaLine {
                record_kind,
                path: path.to_path_buf(),
                line: line_number,
                source,
            })
        })
        .collect()
}

fn read_snapshot(path: &Path) -> Result<EngineSnapshot, PersistenceError> {
    let reader = BufReader::new(open_expected_file(path)?);
    let record: SnapshotRecordV1 =
        serde_json::from_reader(reader).map_err(|source| PersistenceError::SnapshotJson {
            path: path.to_path_buf(),
            source,
        })?;
    EngineSnapshot::try_from(record).map_err(|source| PersistenceError::SnapshotSchema {
        path: path.to_path_buf(),
        source,
    })
}

fn open_expected_file(path: &Path) -> Result<File, PersistenceError> {
    File::open(path).map_err(|source| {
        if source.kind() == io::ErrorKind::NotFound {
            PersistenceError::MissingFile {
                path: path.to_path_buf(),
            }
        } else {
            PersistenceError::Io {
                operation: "open",
                path: path.to_path_buf(),
                source,
            }
        }
    })
}
