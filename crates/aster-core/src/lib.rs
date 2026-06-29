//! Deterministic central limit order book and matching engine core for Aster.
//!
//! # Core concepts
//!
//! [`EngineCommand`] values are input intentions. [`AsterEngine`] processes them
//! in order, assigns deterministic [`OrderId`] and [`SequenceNumber`] values,
//! updates passive [`OrderBook`] state, and emits [`EngineEvent`] facts.
//! Matching uses integer [`PriceTicks`], integer [`Quantity`], ordered price
//! levels, and FIFO priority within a level. It does not use wall-clock time or
//! randomness.
//!
//! [`replay_commands`] rebuilds events and a complete [`EngineSnapshot`] through
//! a fresh engine. [`SessionRecord`] keeps commands, events, and the final
//! snapshot together for equality verification.
//!
//! # Persistence boundary
//!
//! [`save_session_record`] and [`load_session_record`] persist completed sessions
//! through versioned schema records. Commands and events use JSONL; the final
//! snapshot uses JSON. [`verify_session_directory`] treats commands as canonical
//! replay input and saved events/snapshots as audit outputs. Persistence is kept
//! outside the matching path.
//!
//! # Example
//!
//! ```
//! use aster_core::{
//!     AsterEngine, AsterError, EngineCommand, EngineEvent, OrderRequest,
//!     OrderType, ParticipantId, PriceTicks, Quantity, Side,
//! };
//!
//! # fn main() -> Result<(), AsterError> {
//! let mut engine = AsterEngine::new();
//! let sell = OrderRequest::new(
//!     ParticipantId::new(1),
//!     Side::Sell,
//!     OrderType::Limit {
//!         price: PriceTicks::new(101)?,
//!     },
//!     Quantity::new(10)?,
//! );
//! let buy = OrderRequest::new(
//!     ParticipantId::new(2),
//!     Side::Buy,
//!     OrderType::Limit {
//!         price: PriceTicks::new(102)?,
//!     },
//!     Quantity::new(4)?,
//! );
//!
//! engine.process_command(EngineCommand::submit_order(sell));
//! let events = engine.process_command(EngineCommand::submit_order(buy));
//!
//! assert!(matches!(events[0], EngineEvent::OrderAccepted { .. }));
//! assert!(matches!(
//!     events[1],
//!     EngineEvent::TradeExecuted { price, quantity, .. }
//!         if price.as_u64() == 101 && quantity.as_u64() == 4
//! ));
//! assert_eq!(engine.snapshot().total_resting_quantity, 6);
//! # Ok(())
//! # }
//! ```

pub mod command;
pub mod engine;
pub mod errors;
pub mod event;
pub mod order;
pub mod order_book;
pub mod persistence;
pub mod price_level;
pub mod replay;
pub mod schema;
pub mod session;
pub mod types;
pub mod validation;

pub use command::EngineCommand;
pub use engine::{AsterEngine, EngineSnapshot, PriceLevelSnapshot};
pub use errors::AsterError;
pub use event::EngineEvent;
pub use order::{AcceptedOrder, OrderRequest, OrderType, Side};
pub use order_book::OrderBook;
pub use persistence::{
    load_session_record, save_session_record, verify_session_directory, PersistenceError,
    COMMANDS_FILE_NAME, EVENTS_FILE_NAME, SNAPSHOT_FILE_NAME,
};
pub use price_level::PriceLevel;
pub use replay::{replay_commands, ReplayResult};
pub use schema::{
    AcceptedOrderDtoV1, AsterErrorDtoV1, CommandDtoV1, CommandRecordV1, EngineSnapshotDtoV1,
    EventDtoV1, EventRecordV1, OrderTypeDtoV1, PriceLevelSnapshotDtoV1, SideDtoV1,
    SnapshotRecordV1, ASTER_SCHEMA_VERSION,
};
pub use session::{SessionRecord, SessionVerificationError};
pub use types::{OrderId, ParticipantId, PriceTicks, Quantity, SequenceNumber};

/// Returns the project name for smoke tests and the placeholder CLI.
pub fn project_name() -> &'static str {
    "Aster"
}

#[cfg(test)]
mod tests {
    use super::project_name;

    #[test]
    fn exposes_project_name() {
        assert_eq!(project_name(), "Aster");
    }
}
