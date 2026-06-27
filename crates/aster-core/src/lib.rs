//! Deterministic matching engine core for Aster.
//!
//! This crate contains the central limit order book domain model, validation,
//! matching engine, command/event types, and in-memory replay scaffolding.
//! File persistence is not implemented yet.

pub mod command;
pub mod engine;
pub mod errors;
pub mod event;
pub mod order;
pub mod order_book;
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
