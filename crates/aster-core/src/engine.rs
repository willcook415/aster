//! High-level deterministic command processor.
//!
//! `AsterEngine` owns order ID and sequence number allocation. At this stage it
//! accepts limit orders, matches them against resting liquidity, and rests any
//! remaining limit quantity. Market orders are rejected until market execution
//! is implemented, and cancellation commands are rejected until cancellation
//! execution exists.

use crate::{
    AcceptedOrder, AsterError, EngineCommand, EngineEvent, OrderBook, OrderId, OrderRequest,
    OrderType, PriceTicks, Quantity, SequenceNumber, Side,
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

        let order = AcceptedOrder::new(
            OrderId::new(self.next_order_id),
            SequenceNumber::new(self.next_sequence_number),
            request,
        );
        let mut events = vec![EngineEvent::OrderAccepted { order }];

        match self.match_and_maybe_rest(order, price, &mut events) {
            Ok(()) => {
                self.next_order_id += 1;
                self.next_sequence_number += 1;
                events
            }
            Err(reason) => vec![EngineEvent::OrderRejected { reason }],
        }
    }

    fn match_and_maybe_rest(
        &mut self,
        order: AcceptedOrder,
        limit_price: PriceTicks,
        events: &mut Vec<EngineEvent>,
    ) -> Result<(), AsterError> {
        let mut remaining_quantity = order.quantity.as_u64();

        while remaining_quantity > 0 && self.crosses_book(order.side, limit_price) {
            let resting_order = self
                .best_opposing_front_order(order.side)
                .ok_or(AsterError::InvalidOrderState)?;
            let resting_quantity = resting_order.quantity.as_u64();
            let fill_quantity = remaining_quantity.min(resting_quantity);
            let trade_quantity = Quantity::new(fill_quantity)?;
            let OrderType::Limit { price } = resting_order.order_type else {
                return Err(AsterError::InvalidOrderState);
            };

            events.push(EngineEvent::TradeExecuted {
                resting_order_id: resting_order.order_id,
                incoming_order_id: order.order_id,
                price,
                quantity: trade_quantity,
            });

            if fill_quantity == resting_quantity {
                self.pop_best_opposing_front_order(order.side)
                    .ok_or(AsterError::InvalidOrderState)?;
            } else {
                self.reduce_best_opposing_front_quantity(
                    order.side,
                    Quantity::new(resting_quantity - fill_quantity)?,
                )?;
            }

            remaining_quantity -= fill_quantity;
        }

        if remaining_quantity > 0 {
            let mut resting_remainder = order;
            resting_remainder.quantity = Quantity::new(remaining_quantity)?;
            self.order_book.add_resting_order(resting_remainder)?;
        }

        Ok(())
    }

    fn best_opposing_front_order(&self, incoming_side: Side) -> Option<AcceptedOrder> {
        match incoming_side {
            Side::Buy => self.order_book.best_ask_front_order().copied(),
            Side::Sell => self.order_book.best_bid_front_order().copied(),
        }
    }

    fn pop_best_opposing_front_order(&mut self, incoming_side: Side) -> Option<AcceptedOrder> {
        match incoming_side {
            Side::Buy => self.order_book.pop_best_ask_front_order(),
            Side::Sell => self.order_book.pop_best_bid_front_order(),
        }
    }

    fn reduce_best_opposing_front_quantity(
        &mut self,
        incoming_side: Side,
        new_quantity: Quantity,
    ) -> Result<(), AsterError> {
        match incoming_side {
            Side::Buy => self.order_book.reduce_best_ask_front_quantity(new_quantity),
            Side::Sell => self.order_book.reduce_best_bid_front_quantity(new_quantity),
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
