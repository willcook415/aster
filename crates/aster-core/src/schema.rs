//! Versioned serialization boundary types.
//!
//! These DTOs are intentionally separate from the internal engine model so
//! durable records can evolve without exposing implementation details as the
//! persistence format.

use serde::{Deserialize, Serialize};

mod conversions;
mod primitive_conversions;
mod snapshot_validation;

/// Current version for persisted Aster schema records.
pub const ASTER_SCHEMA_VERSION: u16 = 1;

/// Versioned command record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandRecordV1 {
    pub schema_version: u16,
    pub command: CommandDtoV1,
}

/// Serializable command payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CommandDtoV1 {
    SubmitOrder {
        participant_id: u64,
        side: SideDtoV1,
        order_type: OrderTypeDtoV1,
        quantity: u64,
    },
    CancelOrder {
        order_id: u64,
        participant_id: u64,
    },
}

/// Versioned event record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventRecordV1 {
    pub schema_version: u16,
    pub event: EventDtoV1,
}

/// Serializable event payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventDtoV1 {
    OrderAccepted {
        order: AcceptedOrderDtoV1,
    },
    OrderRejected {
        reason: AsterErrorDtoV1,
    },
    OrderCancelled {
        order_id: u64,
        participant_id: u64,
    },
    CancelRejected {
        order_id: u64,
        participant_id: u64,
        reason: AsterErrorDtoV1,
    },
    TradeExecuted {
        resting_order_id: u64,
        incoming_order_id: u64,
        price: u64,
        quantity: u64,
    },
}

/// Serializable accepted-order payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptedOrderDtoV1 {
    pub order_id: u64,
    pub participant_id: u64,
    pub side: SideDtoV1,
    pub order_type: OrderTypeDtoV1,
    pub quantity: u64,
    pub sequence_number: u64,
}

/// Versioned snapshot record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotRecordV1 {
    pub schema_version: u16,
    pub snapshot: EngineSnapshotDtoV1,
}

/// Serializable engine snapshot payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineSnapshotDtoV1 {
    pub best_bid: Option<u64>,
    pub best_ask: Option<u64>,
    pub bid_level_count: usize,
    pub ask_level_count: usize,
    pub total_resting_quantity: u64,
    pub bid_levels: Vec<PriceLevelSnapshotDtoV1>,
    pub ask_levels: Vec<PriceLevelSnapshotDtoV1>,
    pub next_order_id: u64,
    pub next_sequence_number: u64,
}

/// Serializable price level with resting orders in FIFO order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceLevelSnapshotDtoV1 {
    pub price: u64,
    pub orders: Vec<AcceptedOrderDtoV1>,
}

/// Serializable side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SideDtoV1 {
    Buy,
    Sell,
}

/// Serializable order type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrderTypeDtoV1 {
    Limit { price: u64 },
    Market,
}

/// Stable error representation for schema records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AsterErrorDtoV1 {
    ZeroQuantity,
    ZeroPrice,
    OrderNotFound,
    ParticipantMismatch,
    PriceLevelMismatch,
    MarketOrderCannotRest,
    DuplicateOrderId,
    OrderIdExhausted,
    SequenceNumberExhausted,
    QuantityOverflow,
    InvalidOrderState,
    SnapshotLevelCountMismatch,
    SnapshotQuantityMismatch,
    SnapshotSideMismatch,
    SnapshotLevelOrderInvalid,
    SnapshotBestPriceMismatch,
    SnapshotBookCrossed,
    SnapshotAllocatorInvalid,
    SnapshotFifoInvalid,
    DuplicateSequenceNumber,
    SnapshotEmptyPriceLevel,
    UnsupportedSchemaVersion,
}
