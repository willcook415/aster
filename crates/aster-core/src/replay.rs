//! In-memory deterministic replay helpers.
//!
//! Replay currently runs a sequence of commands through a fresh `AsterEngine`,
//! collects emitted events, and returns a deterministic final snapshot. It does
//! not perform file persistence or serialization.

use crate::{AsterEngine, EngineCommand, EngineEvent, EngineSnapshot};

/// Result of replaying a command sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayResult {
    pub events: Vec<EngineEvent>,
    pub final_snapshot: EngineSnapshot,
}

/// Replays commands through a fresh engine and returns events plus final state.
pub fn replay_commands<I>(commands: I) -> ReplayResult
where
    I: IntoIterator<Item = EngineCommand>,
{
    let mut engine = AsterEngine::new();
    let events = engine.process_commands(commands);
    let final_snapshot = engine.snapshot();

    ReplayResult {
        events,
        final_snapshot,
    }
}
