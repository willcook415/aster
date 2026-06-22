//! Engine input commands.
//!
//! Commands are intentions submitted to the future matching engine. They do not
//! assert that any state transition has happened. Keeping commands separate
//! from emitted events supports deterministic replay and auditability later.

use crate::{OrderId, OrderRequest, ParticipantId};

/// Inbound command sent to the future engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineCommand {
    /// Request to submit a new order.
    SubmitOrder(OrderRequest),
    /// Request to cancel an existing order.
    CancelOrder {
        order_id: OrderId,
        participant_id: ParticipantId,
    },
}

impl EngineCommand {
    /// Creates a submit-order command.
    pub const fn submit_order(request: OrderRequest) -> Self {
        Self::SubmitOrder(request)
    }

    /// Creates a cancel-order command.
    pub const fn cancel_order(order_id: OrderId, participant_id: ParticipantId) -> Self {
        Self::CancelOrder {
            order_id,
            participant_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EngineCommand;
    use crate::{OrderId, OrderRequest, OrderType, ParticipantId, Quantity, Side};

    #[test]
    fn constructs_submit_order_command() {
        let request = OrderRequest::new(
            ParticipantId::new(1),
            Side::Buy,
            OrderType::Market,
            Quantity::new(10).expect("valid quantity"),
        );

        let command = EngineCommand::submit_order(request);

        assert_eq!(command, EngineCommand::SubmitOrder(request));
    }

    #[test]
    fn constructs_cancel_order_command() {
        let command = EngineCommand::cancel_order(OrderId::new(44), ParticipantId::new(2));

        assert_eq!(
            command,
            EngineCommand::CancelOrder {
                order_id: OrderId::new(44),
                participant_id: ParticipantId::new(2),
            }
        );
    }
}
