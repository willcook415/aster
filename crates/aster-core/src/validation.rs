//! Validation helpers for raw inbound values.
//!
//! Constructors on domain types enforce the current invariants. This module is
//! intentionally small and exists as the future home for command-level
//! validation that should stay separate from matching logic.

use crate::{AsterError, OrderRequest, PriceTicks, Quantity};

/// Validates and wraps a raw quantity.
pub fn validate_quantity(value: u64) -> Result<Quantity, AsterError> {
    Quantity::new(value)
}

/// Validates and wraps a raw limit price in ticks.
pub fn validate_limit_price_ticks(value: u64) -> Result<PriceTicks, AsterError> {
    PriceTicks::new(value)
}

/// Validates an already-typed order request.
///
/// The current typed request cannot represent zero quantity or zero limit
/// price. Future order-level invariants can be added here without putting
/// validation rules inside the matching path.
pub const fn validate_order_request(_request: &OrderRequest) {}
