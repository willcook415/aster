//! Order request and accepted order domain types.

use crate::{OrderId, ParticipantId, PriceTicks, Quantity, SequenceNumber};

/// Side of the book targeted by an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    /// Buy-side order.
    Buy,
    /// Sell-side order.
    Sell,
}

/// Supported order type at the domain-model stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderType {
    /// Limit order with a positive integer tick price.
    Limit { price: PriceTicks },
    /// Market order with no limit price.
    Market,
}

/// Inbound order command before engine sequencing and order ID assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderRequest {
    pub participant_id: ParticipantId,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: Quantity,
}

impl OrderRequest {
    /// Creates an inbound order request from already-validated domain values.
    pub const fn new(
        participant_id: ParticipantId,
        side: Side,
        order_type: OrderType,
        quantity: Quantity,
    ) -> Self {
        Self {
            participant_id,
            side,
            order_type,
            quantity,
        }
    }
}

/// Order accepted by the engine after deterministic ID and sequence assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AcceptedOrder {
    pub order_id: OrderId,
    pub participant_id: ParticipantId,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: Quantity,
    pub sequence_number: SequenceNumber,
}

impl AcceptedOrder {
    /// Creates an accepted order from an inbound request and assigned metadata.
    ///
    /// `AsterEngine` is the authoritative allocator during normal command
    /// processing. This low-level constructor remains public for explicit book
    /// fixtures and schema conversion; callers using it are responsible for ID
    /// and sequence uniqueness.
    pub const fn new(
        order_id: OrderId,
        sequence_number: SequenceNumber,
        request: OrderRequest,
    ) -> Self {
        Self {
            order_id,
            participant_id: request.participant_id,
            side: request.side,
            order_type: request.order_type,
            quantity: request.quantity,
            sequence_number,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AcceptedOrder, OrderRequest, OrderType, Side};
    use crate::{OrderId, ParticipantId, PriceTicks, Quantity, SequenceNumber};

    #[test]
    fn creates_market_order_request() {
        let request = OrderRequest::new(
            ParticipantId::new(7),
            Side::Buy,
            OrderType::Market,
            Quantity::new(10).expect("valid quantity"),
        );

        assert_eq!(request.participant_id.as_u64(), 7);
        assert_eq!(request.side, Side::Buy);
        assert_eq!(request.order_type, OrderType::Market);
        assert_eq!(request.quantity.as_u64(), 10);
    }

    #[test]
    fn creates_limit_order_request() {
        let price = PriceTicks::new(125).expect("valid price");
        let request = OrderRequest::new(
            ParticipantId::new(8),
            Side::Sell,
            OrderType::Limit { price },
            Quantity::new(25).expect("valid quantity"),
        );

        assert_eq!(request.side, Side::Sell);
        assert_eq!(request.order_type, OrderType::Limit { price });
    }

    #[test]
    fn accepted_order_includes_engine_assigned_identity_and_sequence() {
        let request = OrderRequest::new(
            ParticipantId::new(9),
            Side::Buy,
            OrderType::Market,
            Quantity::new(15).expect("valid quantity"),
        );

        let accepted = AcceptedOrder::new(OrderId::new(1001), SequenceNumber::new(77), request);

        assert_eq!(accepted.order_id.as_u64(), 1001);
        assert_eq!(accepted.sequence_number.as_u64(), 77);
        assert_eq!(accepted.participant_id, request.participant_id);
        assert_eq!(accepted.quantity, request.quantity);
    }
}
