//! Deterministic matching engine core for Aster.
//!
//! This crate will contain the central limit order book domain model,
//! validation, matching, event log, and replay logic. It is intentionally
//! limited to finance-safe domain primitives and command/event skeletons at
//! this stage. It has a focused single-price FIFO level, but does not yet
//! implement full bid/ask book storage, matching rules,
//! event persistence, or replay.

pub mod command;
pub mod errors;
pub mod event;
pub mod order;
pub mod price_level;
pub mod types;
pub mod validation;

pub use command::EngineCommand;
pub use errors::AsterError;
pub use event::EngineEvent;
pub use order::{AcceptedOrder, OrderRequest, OrderType, Side};
pub use price_level::PriceLevel;
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
