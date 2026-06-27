use super::{
    AcceptedOrderDtoV1, CommandDtoV1, CommandRecordV1, EngineSnapshotDtoV1, EventDtoV1,
    EventRecordV1, PriceLevelSnapshotDtoV1, SnapshotRecordV1, ASTER_SCHEMA_VERSION,
};
use crate::{
    AcceptedOrder, AsterError, EngineCommand, EngineEvent, EngineSnapshot, OrderId, OrderRequest,
    ParticipantId, PriceLevelSnapshot, PriceTicks, Quantity, SequenceNumber,
};

impl From<EngineCommand> for CommandRecordV1 {
    fn from(command: EngineCommand) -> Self {
        Self {
            schema_version: ASTER_SCHEMA_VERSION,
            command: command.into(),
        }
    }
}

impl TryFrom<CommandRecordV1> for EngineCommand {
    type Error = AsterError;

    fn try_from(record: CommandRecordV1) -> Result<Self, Self::Error> {
        validate_version(record.schema_version)?;
        record.command.try_into()
    }
}

impl From<EngineCommand> for CommandDtoV1 {
    fn from(command: EngineCommand) -> Self {
        match command {
            EngineCommand::SubmitOrder(request) => Self::SubmitOrder {
                participant_id: request.participant_id.as_u64(),
                side: request.side.into(),
                order_type: request.order_type.into(),
                quantity: request.quantity.as_u64(),
            },
            EngineCommand::CancelOrder {
                order_id,
                participant_id,
            } => Self::CancelOrder {
                order_id: order_id.as_u64(),
                participant_id: participant_id.as_u64(),
            },
        }
    }
}

impl TryFrom<CommandDtoV1> for EngineCommand {
    type Error = AsterError;

    fn try_from(command: CommandDtoV1) -> Result<Self, Self::Error> {
        match command {
            CommandDtoV1::SubmitOrder {
                participant_id,
                side,
                order_type,
                quantity,
            } => Ok(Self::submit_order(OrderRequest::new(
                ParticipantId::new(participant_id),
                side.into(),
                order_type.try_into()?,
                Quantity::new(quantity)?,
            ))),
            CommandDtoV1::CancelOrder {
                order_id,
                participant_id,
            } => Ok(Self::cancel_order(
                OrderId::new(order_id),
                ParticipantId::new(participant_id),
            )),
        }
    }
}

impl From<EngineEvent> for EventRecordV1 {
    fn from(event: EngineEvent) -> Self {
        Self {
            schema_version: ASTER_SCHEMA_VERSION,
            event: event.into(),
        }
    }
}

impl TryFrom<EventRecordV1> for EngineEvent {
    type Error = AsterError;

    fn try_from(record: EventRecordV1) -> Result<Self, Self::Error> {
        validate_version(record.schema_version)?;
        record.event.try_into()
    }
}

impl From<EngineEvent> for EventDtoV1 {
    fn from(event: EngineEvent) -> Self {
        match event {
            EngineEvent::OrderAccepted { order } => Self::OrderAccepted {
                order: order.into(),
            },
            EngineEvent::OrderRejected { reason } => Self::OrderRejected {
                reason: reason.into(),
            },
            EngineEvent::OrderCancelled {
                order_id,
                participant_id,
            } => Self::OrderCancelled {
                order_id: order_id.as_u64(),
                participant_id: participant_id.as_u64(),
            },
            EngineEvent::CancelRejected {
                order_id,
                participant_id,
                reason,
            } => Self::CancelRejected {
                order_id: order_id.as_u64(),
                participant_id: participant_id.as_u64(),
                reason: reason.into(),
            },
            EngineEvent::TradeExecuted {
                resting_order_id,
                incoming_order_id,
                price,
                quantity,
            } => Self::TradeExecuted {
                resting_order_id: resting_order_id.as_u64(),
                incoming_order_id: incoming_order_id.as_u64(),
                price: price.as_u64(),
                quantity: quantity.as_u64(),
            },
        }
    }
}

impl TryFrom<EventDtoV1> for EngineEvent {
    type Error = AsterError;

    fn try_from(event: EventDtoV1) -> Result<Self, Self::Error> {
        match event {
            EventDtoV1::OrderAccepted { order } => Ok(Self::OrderAccepted {
                order: order.try_into()?,
            }),
            EventDtoV1::OrderRejected { reason } => Ok(Self::OrderRejected {
                reason: reason.into(),
            }),
            EventDtoV1::OrderCancelled {
                order_id,
                participant_id,
            } => Ok(Self::OrderCancelled {
                order_id: OrderId::new(order_id),
                participant_id: ParticipantId::new(participant_id),
            }),
            EventDtoV1::CancelRejected {
                order_id,
                participant_id,
                reason,
            } => Ok(Self::CancelRejected {
                order_id: OrderId::new(order_id),
                participant_id: ParticipantId::new(participant_id),
                reason: reason.into(),
            }),
            EventDtoV1::TradeExecuted {
                resting_order_id,
                incoming_order_id,
                price,
                quantity,
            } => Ok(Self::TradeExecuted {
                resting_order_id: OrderId::new(resting_order_id),
                incoming_order_id: OrderId::new(incoming_order_id),
                price: PriceTicks::new(price)?,
                quantity: Quantity::new(quantity)?,
            }),
        }
    }
}

impl From<EngineSnapshot> for SnapshotRecordV1 {
    fn from(snapshot: EngineSnapshot) -> Self {
        Self {
            schema_version: ASTER_SCHEMA_VERSION,
            snapshot: snapshot.into(),
        }
    }
}

impl TryFrom<SnapshotRecordV1> for EngineSnapshot {
    type Error = AsterError;

    fn try_from(record: SnapshotRecordV1) -> Result<Self, Self::Error> {
        validate_version(record.schema_version)?;
        record.snapshot.try_into()
    }
}

impl From<EngineSnapshot> for EngineSnapshotDtoV1 {
    fn from(snapshot: EngineSnapshot) -> Self {
        Self {
            best_bid: snapshot.best_bid.map(PriceTicks::as_u64),
            best_ask: snapshot.best_ask.map(PriceTicks::as_u64),
            bid_level_count: snapshot.bid_level_count,
            ask_level_count: snapshot.ask_level_count,
            total_resting_quantity: snapshot.total_resting_quantity,
            bid_levels: snapshot
                .bid_levels
                .into_iter()
                .map(PriceLevelSnapshotDtoV1::from)
                .collect(),
            ask_levels: snapshot
                .ask_levels
                .into_iter()
                .map(PriceLevelSnapshotDtoV1::from)
                .collect(),
            next_order_id: snapshot.next_order_id.as_u64(),
            next_sequence_number: snapshot.next_sequence_number.as_u64(),
        }
    }
}

impl TryFrom<EngineSnapshotDtoV1> for EngineSnapshot {
    type Error = AsterError;

    fn try_from(snapshot: EngineSnapshotDtoV1) -> Result<Self, Self::Error> {
        Ok(Self {
            best_bid: snapshot.best_bid.map(PriceTicks::new).transpose()?,
            best_ask: snapshot.best_ask.map(PriceTicks::new).transpose()?,
            bid_level_count: snapshot.bid_level_count,
            ask_level_count: snapshot.ask_level_count,
            total_resting_quantity: snapshot.total_resting_quantity,
            bid_levels: snapshot
                .bid_levels
                .into_iter()
                .map(PriceLevelSnapshot::try_from)
                .collect::<Result<_, _>>()?,
            ask_levels: snapshot
                .ask_levels
                .into_iter()
                .map(PriceLevelSnapshot::try_from)
                .collect::<Result<_, _>>()?,
            next_order_id: OrderId::new(snapshot.next_order_id),
            next_sequence_number: SequenceNumber::new(snapshot.next_sequence_number),
        })
    }
}

impl From<PriceLevelSnapshot> for PriceLevelSnapshotDtoV1 {
    fn from(level: PriceLevelSnapshot) -> Self {
        Self {
            price: level.price.as_u64(),
            orders: level
                .orders
                .into_iter()
                .map(AcceptedOrderDtoV1::from)
                .collect(),
        }
    }
}

impl TryFrom<PriceLevelSnapshotDtoV1> for PriceLevelSnapshot {
    type Error = AsterError;

    fn try_from(level: PriceLevelSnapshotDtoV1) -> Result<Self, Self::Error> {
        Ok(Self {
            price: PriceTicks::new(level.price)?,
            orders: level
                .orders
                .into_iter()
                .map(AcceptedOrder::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl From<AcceptedOrder> for AcceptedOrderDtoV1 {
    fn from(order: AcceptedOrder) -> Self {
        Self {
            order_id: order.order_id.as_u64(),
            participant_id: order.participant_id.as_u64(),
            side: order.side.into(),
            order_type: order.order_type.into(),
            quantity: order.quantity.as_u64(),
            sequence_number: order.sequence_number.as_u64(),
        }
    }
}

impl TryFrom<AcceptedOrderDtoV1> for AcceptedOrder {
    type Error = AsterError;

    fn try_from(order: AcceptedOrderDtoV1) -> Result<Self, Self::Error> {
        Ok(Self {
            order_id: OrderId::new(order.order_id),
            participant_id: ParticipantId::new(order.participant_id),
            side: order.side.into(),
            order_type: order.order_type.try_into()?,
            quantity: Quantity::new(order.quantity)?,
            sequence_number: SequenceNumber::new(order.sequence_number),
        })
    }
}

fn validate_version(schema_version: u16) -> Result<(), AsterError> {
    if schema_version == ASTER_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(AsterError::UnsupportedSchemaVersion)
    }
}
