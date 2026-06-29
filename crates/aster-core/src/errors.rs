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
    /// Referenced order does not exist in engine state.
    OrderNotFound,
    /// Referenced order is owned by a different participant.
    ParticipantMismatch,
    /// A resting order price does not match its price level.
    PriceLevelMismatch,
    /// Market orders cannot rest at a price level.
    MarketOrderCannotRest,
    /// A resting order ID already exists in book storage.
    DuplicateOrderId,
    /// The engine cannot assign another order identifier.
    OrderIdExhausted,
    /// The engine cannot assign another priority sequence number.
    SequenceNumberExhausted,
    /// Resting quantity cannot be represented without overflow.
    QuantityOverflow,
    /// An accepted order or engine state failed an internal invariant.
    InvalidOrderState,
    /// Snapshot summary level counts do not match its visible levels.
    SnapshotLevelCountMismatch,
    /// Snapshot summary quantity does not match its visible resting orders.
    SnapshotQuantityMismatch,
    /// A snapshot resting order is stored on the wrong book side.
    SnapshotSideMismatch,
    /// Snapshot levels are not ordered from best price to worst price.
    SnapshotLevelOrderInvalid,
    /// Snapshot best-price summaries do not match its visible levels.
    SnapshotBestPriceMismatch,
    /// Snapshot bid and ask prices form a crossed book.
    SnapshotBookCrossed,
    /// Snapshot allocator state is not ahead of visible identities.
    SnapshotAllocatorInvalid,
    /// Snapshot FIFO sequence ordering is invalid.
    SnapshotFifoInvalid,
    /// A resting sequence number appears more than once in a snapshot.
    DuplicateSequenceNumber,
    /// A snapshot contains an empty price level.
    SnapshotEmptyPriceLevel,
    /// A persisted schema record uses an unsupported schema version.
    UnsupportedSchemaVersion,
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
            Self::OrderIdExhausted => f.write_str("order ID allocation is exhausted"),
            Self::SequenceNumberExhausted => f.write_str("sequence number allocation is exhausted"),
            Self::QuantityOverflow => f.write_str("resting quantity would overflow"),
            Self::InvalidOrderState => f.write_str("invalid order state"),
            Self::SnapshotLevelCountMismatch => {
                f.write_str("snapshot level count does not match visible levels")
            }
            Self::SnapshotQuantityMismatch => {
                f.write_str("snapshot quantity does not match visible resting orders")
            }
            Self::SnapshotSideMismatch => {
                f.write_str("snapshot resting order is on the wrong book side")
            }
            Self::SnapshotLevelOrderInvalid => {
                f.write_str("snapshot price levels are not in matching order")
            }
            Self::SnapshotBestPriceMismatch => {
                f.write_str("snapshot best price does not match visible levels")
            }
            Self::SnapshotBookCrossed => f.write_str("snapshot book is crossed"),
            Self::SnapshotAllocatorInvalid => {
                f.write_str("snapshot allocator is not ahead of visible identities")
            }
            Self::SnapshotFifoInvalid => f.write_str("snapshot FIFO sequence order is invalid"),
            Self::DuplicateSequenceNumber => {
                f.write_str("snapshot sequence number appears more than once")
            }
            Self::SnapshotEmptyPriceLevel => f.write_str("snapshot contains an empty price level"),
            Self::UnsupportedSchemaVersion => f.write_str("unsupported schema version"),
        }
    }
}

impl Error for AsterError {}
