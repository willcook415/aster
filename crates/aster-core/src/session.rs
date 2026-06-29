//! In-memory deterministic session records and replay verification.
//!
//! A session records input commands, emitted events, and the complete final
//! snapshot. The type does not serialize itself or perform file I/O; the
//! `persistence` module converts its fields through versioned schema records.

use std::error::Error;
use std::fmt;

use crate::{replay_commands, EngineCommand, EngineEvent, EngineSnapshot};

/// Complete in-memory record of one deterministic engine session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecord {
    pub commands: Vec<EngineCommand>,
    pub events: Vec<EngineEvent>,
    pub final_snapshot: EngineSnapshot,
}

impl SessionRecord {
    /// Processes commands through a fresh engine and records the resulting session.
    pub fn from_commands<I>(commands: I) -> Self
    where
        I: IntoIterator<Item = EngineCommand>,
    {
        let commands: Vec<_> = commands.into_iter().collect();
        let replay = replay_commands(commands.iter().copied());

        Self {
            commands,
            events: replay.events,
            final_snapshot: replay.final_snapshot,
        }
    }

    /// Replays recorded commands and verifies events and complete final state.
    pub fn verify(&self) -> Result<(), SessionVerificationError> {
        let replay = replay_commands(self.commands.iter().copied());

        if replay.events != self.events {
            let index = first_event_difference(&self.events, &replay.events);
            return Err(SessionVerificationError::EventMismatch {
                index,
                recorded: self.events.get(index).copied(),
                replayed: replay.events.get(index).copied(),
            });
        }

        if replay.final_snapshot != self.final_snapshot {
            return Err(SessionVerificationError::SnapshotMismatch {
                recorded: Box::new(self.final_snapshot.clone()),
                replayed: Box::new(replay.final_snapshot),
            });
        }

        Ok(())
    }
}

/// Reason a recorded session does not match deterministic command replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionVerificationError {
    /// Recorded and replayed event streams first differ at `index`.
    EventMismatch {
        index: usize,
        recorded: Option<EngineEvent>,
        replayed: Option<EngineEvent>,
    },
    /// Events match, but complete final engine snapshots differ.
    SnapshotMismatch {
        recorded: Box<EngineSnapshot>,
        replayed: Box<EngineSnapshot>,
    },
}

impl fmt::Display for SessionVerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EventMismatch { index, .. } => {
                write!(f, "session events differ at index {index}")
            }
            Self::SnapshotMismatch { .. } => {
                f.write_str("session final snapshot does not match replay")
            }
        }
    }
}

impl Error for SessionVerificationError {}

fn first_event_difference(recorded: &[EngineEvent], replayed: &[EngineEvent]) -> usize {
    recorded
        .iter()
        .zip(replayed)
        .position(|(recorded, replayed)| recorded != replayed)
        .unwrap_or_else(|| recorded.len().min(replayed.len()))
}
