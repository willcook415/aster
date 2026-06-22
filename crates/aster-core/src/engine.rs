//! High-level deterministic command processor.
//!
//! `AsterEngine` owns order ID and sequence number allocation. At this stage it
//! accepts and rests only non-crossing limit orders. Crossing limit orders and
//! market orders are rejected until matching is implemented, and cancellation
//! commands are rejected until cancellation execution exists.

use crate::{
    AcceptedOrder, AsterError, EngineCommand, EngineEvent, OrderBook, OrderId, OrderRequest,
    OrderType, PriceTicks, SequenceNumber, Side,
};

const FIRST_ORDER_ID: u64 = 1;
const FIRST_SEQUENCE_NUMBER: u64 = 1;

/// Deterministic engine entrypoint for processing inbound commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsterEngine {
    order_book: OrderBook,
    next_order_id: u64,
    next_sequence_number: u64,
}

impl AsterEngine {
    /// Creates an empty engine with deterministic counters starting at 1.
    pub fn new() -> Self {
        Self {
            order_book: OrderBook::new(),
            next_order_id: FIRST_ORDER_ID,
            next_sequence_number: FIRST_SEQUENCE_NUMBER,
        }
    }

    /// Processes one inbound command and returns emitted facts.
    pub fn process_command(&mut self, command: EngineCommand) -> Vec<EngineEvent> {
        match command {
            EngineCommand::SubmitOrder(request) => self.process_submit_order(request),
            EngineCommand::CancelOrder {
                order_id,
                participant_id,
            } => vec![EngineEvent::CancelRejected {
                order_id,
                participant_id,
                reason: AsterError::CancellationNotImplemented,
            }],
        }
    }

    /// Returns the engine's internal passive order book.
    pub const fn order_book(&self) -> &OrderBook {
        &self.order_book
    }

    fn process_submit_order(&mut self, request: OrderRequest) -> Vec<EngineEvent> {
        let OrderType::Limit { price } = request.order_type else {
            return vec![EngineEvent::OrderRejected {
                reason: AsterError::MarketOrderRequiresMatching,
            }];
        };

        if self.crosses_book(request.side, price) {
            return vec![EngineEvent::OrderRejected {
                reason: AsterError::CrossingOrderRequiresMatching,
            }];
        }

        let order = AcceptedOrder::new(
            OrderId::new(self.next_order_id),
            SequenceNumber::new(self.next_sequence_number),
            request,
        );

        match self.order_book.add_resting_order(order) {
            Ok(()) => {
                self.next_order_id += 1;
                self.next_sequence_number += 1;
                vec![EngineEvent::OrderAccepted { order }]
            }
            Err(reason) => vec![EngineEvent::OrderRejected { reason }],
        }
    }

    fn crosses_book(&self, side: Side, price: PriceTicks) -> bool {
        match side {
            Side::Buy => self
                .order_book
                .best_ask_price()
                .is_some_and(|best_ask| price >= best_ask),
            Side::Sell => self
                .order_book
                .best_bid_price()
                .is_some_and(|best_bid| price <= best_bid),
        }
    }
}

impl Default for AsterEngine {
    fn default() -> Self {
        Self::new()
    }
}
