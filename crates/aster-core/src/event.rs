//! Engine output events.
//!
//! Events are facts emitted by the future engine after command processing. They
//! are intentionally distinct from command intentions so later event logs,
//! deterministic replay, and audit trails can reason about what actually
//! happened without executing matching logic in this skeleton.

use crate::{AcceptedOrder, AsterError, OrderId, ParticipantId, PriceTicks, Quantity};

/// Fact emitted by the future engine after processing an input command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineEvent {
    /// A submitted order was accepted and assigned engine metadata.
    OrderAccepted { order: AcceptedOrder },
    /// A submitted order was rejected.
    OrderRejected { reason: AsterError },
    /// A cancel command successfully cancelled an order.
    OrderCancelled {
        order_id: OrderId,
        participant_id: ParticipantId,
    },
    /// A cancel command was rejected.
    CancelRejected {
        order_id: OrderId,
        participant_id: ParticipantId,
        reason: AsterError,
    },
    /// A trade occurred between a resting order and an incoming order.
    TradeExecuted {
        resting_order_id: OrderId,
        incoming_order_id: OrderId,
        price: PriceTicks,
        quantity: Quantity,
    },
}

#[cfg(test)]
mod tests {
    use super::EngineEvent;
    use crate::{
        AcceptedOrder, AsterError, OrderId, OrderRequest, OrderType, ParticipantId, PriceTicks,
        Quantity, SequenceNumber, Side,
    };

    #[test]
    fn order_accepted_carries_accepted_order() {
        let request = OrderRequest::new(
            ParticipantId::new(10),
            Side::Sell,
            OrderType::Market,
            Quantity::new(5).expect("valid quantity"),
        );
        let order = AcceptedOrder::new(OrderId::new(100), SequenceNumber::new(7), request);

        let event = EngineEvent::OrderAccepted { order };

        assert_eq!(event, EngineEvent::OrderAccepted { order });
    }

    #[test]
    fn order_rejected_carries_domain_error() {
        let event = EngineEvent::OrderRejected {
            reason: AsterError::ZeroQuantity,
        };

        assert_eq!(
            event,
            EngineEvent::OrderRejected {
                reason: AsterError::ZeroQuantity,
            }
        );
    }

    #[test]
    fn order_cancelled_carries_order_and_participant_ids() {
        let event = EngineEvent::OrderCancelled {
            order_id: OrderId::new(11),
            participant_id: ParticipantId::new(3),
        };

        assert_eq!(
            event,
            EngineEvent::OrderCancelled {
                order_id: OrderId::new(11),
                participant_id: ParticipantId::new(3),
            }
        );
    }

    #[test]
    fn cancel_rejected_carries_ids_and_reason() {
        let event = EngineEvent::CancelRejected {
            order_id: OrderId::new(12),
            participant_id: ParticipantId::new(4),
            reason: AsterError::OrderNotFound,
        };

        assert_eq!(
            event,
            EngineEvent::CancelRejected {
                order_id: OrderId::new(12),
                participant_id: ParticipantId::new(4),
                reason: AsterError::OrderNotFound,
            }
        );
    }

    #[test]
    fn trade_executed_carries_trade_facts() {
        let price = PriceTicks::new(101).expect("valid price");
        let quantity = Quantity::new(25).expect("valid quantity");
        let event = EngineEvent::TradeExecuted {
            resting_order_id: OrderId::new(20),
            incoming_order_id: OrderId::new(21),
            price,
            quantity,
        };

        assert_eq!(
            event,
            EngineEvent::TradeExecuted {
                resting_order_id: OrderId::new(20),
                incoming_order_id: OrderId::new(21),
                price,
                quantity,
            }
        );
    }
}
