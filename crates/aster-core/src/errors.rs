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
    /// An accepted order or future engine state failed an invariant.
    InvalidOrderState,
}

impl fmt::Display for AsterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroQuantity => f.write_str("quantity must be greater than zero"),
            Self::ZeroPrice => f.write_str("price ticks must be greater than zero"),
            Self::InvalidOrderState => f.write_str("invalid order state"),
        }
    }
}

impl Error for AsterError {}
