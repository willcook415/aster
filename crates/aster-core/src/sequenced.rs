//! Opt-in V2 audit envelopes. V1 matching and persisted event semantics stay stable.
use crate::{
    AsterEngine, AsterError, CommandDtoV1, CommandRecordV1, EngineCommand, EngineEvent,
    EngineSnapshot, EventDtoV1, EventRecordV1, OrderType,
};
use serde::{Deserialize, Serialize};

pub const AUDIT_SCHEMA_VERSION: u16 = 2;
pub const MATCHING_RULES_VERSION: u16 = 1;

/// One command and its complete, ordered audit response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandBatchV2 {
    pub schema_version: u16,
    pub matching_rules_version: u16,
    pub command_sequence: u64,
    pub command: CommandDtoV1,
    pub events: Vec<SequencedEventV2>,
}

/// Globally sequenced event correlated with its input command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SequencedEventV2 {
    pub event_sequence: u64,
    pub command_sequence: u64,
    pub event: AuditEventV2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditEventV2 {
    /// An unchanged V1 engine event inside the correlated envelope.
    EngineEvent { event: EventDtoV1 },
    /// Unfilled market quantity expires after all fills for this command.
    OrderExpired {
        order_id: u64,
        remaining_quantity: u64,
        reason: ExpiryReasonV2,
    },
    /// Always last, including for rejected submissions and cancellations.
    CommandCompleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpiryReasonV2 {
    InsufficientLiquidity,
}

/// A deterministic audit adapter with bounded event retention.
///
/// Callers own the returned batches. The internal V1 event log is drained after
/// every command. Order priority and command/event sequencing are independent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequencedEngine {
    engine: AsterEngine,
    next_command_sequence: u64,
    next_event_sequence: u64,
}

impl Default for SequencedEngine {
    fn default() -> Self {
        Self::new()
    }
}
impl SequencedEngine {
    pub fn new() -> Self {
        Self {
            engine: AsterEngine::new(),
            next_command_sequence: 1,
            next_event_sequence: 1,
        }
    }
    pub fn snapshot(&self) -> EngineSnapshot {
        self.engine.snapshot()
    }
    pub fn next_command_sequence(&self) -> u64 {
        self.next_command_sequence
    }
    pub fn next_event_sequence(&self) -> u64 {
        self.next_event_sequence
    }

    /// Reserves worst-case sequence capacity before changing engine state.
    pub fn process_command(
        &mut self,
        command: EngineCommand,
    ) -> Result<CommandBatchV2, AsterError> {
        let next_command = self
            .next_command_sequence
            .checked_add(1)
            .ok_or(AsterError::SequenceNumberExhausted)?;
        let maximum_events = u64::try_from(self.engine.order_book().resting_order_count())
            .ok()
            .and_then(|n| n.checked_add(3))
            .ok_or(AsterError::SequenceNumberExhausted)?;
        self.next_event_sequence
            .checked_add(maximum_events)
            .ok_or(AsterError::SequenceNumberExhausted)?;
        let facts = self.engine.process_command(command);
        self.engine.clear_event_log();
        let mut payloads: Vec<_> = facts
            .iter()
            .copied()
            .map(|event| AuditEventV2::EngineEvent {
                event: EventRecordV1::from(event).event,
            })
            .collect();
        if let Some(EngineEvent::OrderAccepted { order }) = facts.first() {
            if order.order_type == OrderType::Market {
                let filled: u64 = facts
                    .iter()
                    .filter_map(|event| match event {
                        EngineEvent::TradeExecuted { quantity, .. } => Some(quantity.as_u64()),
                        _ => None,
                    })
                    .sum();
                let remaining = order.quantity.as_u64() - filled;
                if remaining > 0 {
                    payloads.push(AuditEventV2::OrderExpired {
                        order_id: order.order_id.as_u64(),
                        remaining_quantity: remaining,
                        reason: ExpiryReasonV2::InsufficientLiquidity,
                    });
                }
            }
        }
        payloads.push(AuditEventV2::CommandCompleted);
        let sequence = self.next_command_sequence;
        let events = payloads
            .into_iter()
            .map(|event| {
                let record = SequencedEventV2 {
                    event_sequence: self.next_event_sequence,
                    command_sequence: sequence,
                    event,
                };
                self.next_event_sequence += 1;
                record
            })
            .collect();
        self.next_command_sequence = next_command;
        Ok(CommandBatchV2 {
            schema_version: AUDIT_SCHEMA_VERSION,
            matching_rules_version: MATCHING_RULES_VERSION,
            command_sequence: sequence,
            command: CommandRecordV1::from(command).command,
            events,
        })
    }

    /// Replay one saved batch, rejecting incompatible versions or any output drift.
    /// Verification failure leaves the receiver unchanged.
    pub fn verify_batch(&mut self, batch: &CommandBatchV2) -> Result<(), AsterError> {
        if batch.schema_version != AUDIT_SCHEMA_VERSION
            || batch.matching_rules_version != MATCHING_RULES_VERSION
        {
            return Err(AsterError::UnsupportedSchemaVersion);
        }
        if batch.command_sequence != self.next_command_sequence {
            return Err(AsterError::InvalidOrderState);
        }
        let command = EngineCommand::try_from(CommandRecordV1 {
            schema_version: 1,
            command: batch.command.clone(),
        })?;
        let mut candidate = self.clone();
        if candidate.process_command(command)? != *batch {
            return Err(AsterError::InvalidOrderState);
        }
        *self = candidate;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sequence_exhaustion_is_atomic() {
        for command_exhausted in [true, false] {
            let mut engine = SequencedEngine::new();
            if command_exhausted {
                engine.next_command_sequence = u64::MAX;
            } else {
                engine.next_event_sequence = u64::MAX;
            }
            let before = engine.clone();
            let command =
                EngineCommand::cancel_order(crate::OrderId::new(1), crate::ParticipantId::new(1));
            assert_eq!(
                engine.process_command(command),
                Err(AsterError::SequenceNumberExhausted)
            );
            assert_eq!(engine, before);
        }
    }
}
