//! Deterministic matching engine core for Aster.
//!
//! This crate will contain the central limit order book domain model,
//! validation, matching, event log, and replay logic. It is intentionally
//! limited to finance-safe domain primitives at this stage and does not yet
//! implement order book storage, matching rules, event logs, or replay.

pub mod errors;
pub mod order;
pub mod types;
pub mod validation;

pub use errors::AsterError;
pub use order::{AcceptedOrder, OrderRequest, OrderType, Side};
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
