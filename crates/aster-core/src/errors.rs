//! Domain errors returned by Aster core constructors and validators.

use std::error::Error;
use std::fmt;

/// Recoverable domain errors for Aster core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AsterError {
    /// Quantities must be positive integer units.
    ZeroQuantity,
    /// Limit prices must be positive integer ticks.
    ZeroPrice,
    /// Referenced order does not exist in the future engine state.
    OrderNotFound,
    /// Referenced order is owned by a different participant.
    ParticipantMismatch,
    /// A resting order price does not match its price level.
    PriceLevelMismatch,
    /// Market orders cannot rest at a price level.
    MarketOrderCannotRest,
    /// A resting order ID already exists in book storage.
    DuplicateOrderId,
    /// Crossing limit orders require matching, which is not implemented yet.
    CrossingOrderRequiresMatching,
    /// Market orders require matching, which is not implemented yet.
    MarketOrderRequiresMatching,
    /// Cancellation execution is not implemented yet.
    CancellationNotImplemented,
    /// An accepted order or future engine state failed an invariant.
    InvalidOrderState,
}

impl fmt::Display for AsterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroQuantity => f.write_str("quantity must be greater than zero"),
            Self::ZeroPrice => f.write_str("price ticks must be greater than zero"),
            Self::OrderNotFound => f.write_str("order was not found"),
            Self::ParticipantMismatch => {
                f.write_str("participant does not match the referenced order")
            }
            Self::PriceLevelMismatch => {
                f.write_str("resting order price does not match the price level")
            }
            Self::MarketOrderCannotRest => f.write_str("market orders cannot rest"),
            Self::DuplicateOrderId => f.write_str("order ID already exists"),
            Self::CrossingOrderRequiresMatching => {
                f.write_str("crossing order requires matching, which is not implemented")
            }
            Self::MarketOrderRequiresMatching => {
                f.write_str("market order requires matching, which is not implemented")
            }
            Self::CancellationNotImplemented => {
                f.write_str("cancellation execution is not implemented")
            }
            Self::InvalidOrderState => f.write_str("invalid order state"),
        }
    }
}

impl Error for AsterError {}
