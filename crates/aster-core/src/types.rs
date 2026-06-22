//! Finance-safe primitive domain types.
//!
//! Prices and quantities are represented with integer values. Aster does not
//! use floating-point numbers for price or quantity modelling.

use crate::AsterError;

/// Engine-assigned order identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct OrderId(u64);

impl OrderId {
    /// Creates an order identifier from an engine-assigned integer.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the underlying integer value.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Participant identifier supplied by an upstream caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ParticipantId(u64);

impl ParticipantId {
    /// Creates a participant identifier.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the underlying integer value.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Engine-controlled sequence number used for deterministic priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SequenceNumber(u64);

impl SequenceNumber {
    /// Creates a sequence number from an engine-assigned integer.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the underlying integer value.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Positive integer price ticks for limit orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PriceTicks(u64);

impl PriceTicks {
    /// Creates a non-zero limit price in integer ticks.
    pub fn new(value: u64) -> Result<Self, AsterError> {
        if value == 0 {
            return Err(AsterError::ZeroPrice);
        }

        Ok(Self(value))
    }

    /// Returns the underlying integer tick value.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Positive integer order quantity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Quantity(u64);

impl Quantity {
    /// Creates a non-zero quantity in integer units.
    pub fn new(value: u64) -> Result<Self, AsterError> {
        if value == 0 {
            return Err(AsterError::ZeroQuantity);
        }

        Ok(Self(value))
    }

    /// Returns the underlying integer quantity.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{PriceTicks, Quantity};
    use crate::AsterError;

    #[test]
    fn creates_valid_quantity() {
        let quantity = Quantity::new(100).expect("valid quantity should construct");

        assert_eq!(quantity.as_u64(), 100);
    }

    #[test]
    fn rejects_zero_quantity() {
        assert_eq!(Quantity::new(0), Err(AsterError::ZeroQuantity));
    }

    #[test]
    fn creates_valid_price_ticks() {
        let price = PriceTicks::new(42).expect("valid price should construct");

        assert_eq!(price.as_u64(), 42);
    }

    #[test]
    fn rejects_zero_price_ticks() {
        assert_eq!(PriceTicks::new(0), Err(AsterError::ZeroPrice));
    }

    #[test]
    fn price_and_quantity_are_integer_wrappers() {
        assert_eq!(
            std::mem::size_of::<PriceTicks>(),
            std::mem::size_of::<u64>()
        );
        assert_eq!(std::mem::size_of::<Quantity>(), std::mem::size_of::<u64>());
    }
}
