use super::{AsterErrorDtoV1, OrderTypeDtoV1, SideDtoV1};
use crate::{AsterError, OrderType, PriceTicks, Side};

impl From<Side> for SideDtoV1 {
    fn from(side: Side) -> Self {
        match side {
            Side::Buy => Self::Buy,
            Side::Sell => Self::Sell,
        }
    }
}

impl From<SideDtoV1> for Side {
    fn from(side: SideDtoV1) -> Self {
        match side {
            SideDtoV1::Buy => Self::Buy,
            SideDtoV1::Sell => Self::Sell,
        }
    }
}

impl From<OrderType> for OrderTypeDtoV1 {
    fn from(order_type: OrderType) -> Self {
        match order_type {
            OrderType::Limit { price } => Self::Limit {
                price: price.as_u64(),
            },
            OrderType::Market => Self::Market,
        }
    }
}

impl TryFrom<OrderTypeDtoV1> for OrderType {
    type Error = AsterError;

    fn try_from(order_type: OrderTypeDtoV1) -> Result<Self, Self::Error> {
        match order_type {
            OrderTypeDtoV1::Limit { price } => Ok(Self::Limit {
                price: PriceTicks::new(price)?,
            }),
            OrderTypeDtoV1::Market => Ok(Self::Market),
        }
    }
}

impl From<AsterError> for AsterErrorDtoV1 {
    fn from(error: AsterError) -> Self {
        match error {
            AsterError::ZeroQuantity => Self::ZeroQuantity,
            AsterError::ZeroPrice => Self::ZeroPrice,
            AsterError::OrderNotFound => Self::OrderNotFound,
            AsterError::ParticipantMismatch => Self::ParticipantMismatch,
            AsterError::PriceLevelMismatch => Self::PriceLevelMismatch,
            AsterError::MarketOrderCannotRest => Self::MarketOrderCannotRest,
            AsterError::DuplicateOrderId => Self::DuplicateOrderId,
            AsterError::OrderIdExhausted => Self::OrderIdExhausted,
            AsterError::SequenceNumberExhausted => Self::SequenceNumberExhausted,
            AsterError::QuantityOverflow => Self::QuantityOverflow,
            AsterError::InvalidOrderState => Self::InvalidOrderState,
            AsterError::UnsupportedSchemaVersion => Self::UnsupportedSchemaVersion,
        }
    }
}

impl From<AsterErrorDtoV1> for AsterError {
    fn from(error: AsterErrorDtoV1) -> Self {
        match error {
            AsterErrorDtoV1::ZeroQuantity => Self::ZeroQuantity,
            AsterErrorDtoV1::ZeroPrice => Self::ZeroPrice,
            AsterErrorDtoV1::OrderNotFound => Self::OrderNotFound,
            AsterErrorDtoV1::ParticipantMismatch => Self::ParticipantMismatch,
            AsterErrorDtoV1::PriceLevelMismatch => Self::PriceLevelMismatch,
            AsterErrorDtoV1::MarketOrderCannotRest => Self::MarketOrderCannotRest,
            AsterErrorDtoV1::DuplicateOrderId => Self::DuplicateOrderId,
            AsterErrorDtoV1::OrderIdExhausted => Self::OrderIdExhausted,
            AsterErrorDtoV1::SequenceNumberExhausted => Self::SequenceNumberExhausted,
            AsterErrorDtoV1::QuantityOverflow => Self::QuantityOverflow,
            AsterErrorDtoV1::InvalidOrderState => Self::InvalidOrderState,
            AsterErrorDtoV1::UnsupportedSchemaVersion => Self::UnsupportedSchemaVersion,
        }
    }
}
