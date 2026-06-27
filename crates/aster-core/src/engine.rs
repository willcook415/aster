//! High-level deterministic command processor.
//!
//! `AsterEngine` owns order ID and sequence number allocation. At this stage it
//! accepts limit and market orders, matches them against resting liquidity, and
//! rests only remaining limit quantity, and cancels owned resting orders.

use crate::{
    AcceptedOrder, AsterError, EngineCommand, EngineEvent, OrderBook, OrderId, OrderRequest,
    OrderType, ParticipantId, PriceTicks, Quantity, SequenceNumber, Side,
};

const FIRST_ORDER_ID: u64 = 1;
const FIRST_SEQUENCE_NUMBER: u64 = 1;

/// Deterministic engine entrypoint for processing inbound commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsterEngine {
    order_book: OrderBook,
    event_log: Vec<EngineEvent>,
    next_order_id: u64,
    next_sequence_number: u64,
}

/// Resting orders at one price in deterministic FIFO order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriceLevelSnapshot {
    pub price: PriceTicks,
    pub orders: Vec<AcceptedOrder>,
}

/// Deterministic representation of complete visible engine state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineSnapshot {
    pub best_bid: Option<PriceTicks>,
    pub best_ask: Option<PriceTicks>,
    pub bid_level_count: usize,
    pub ask_level_count: usize,
    pub total_resting_quantity: u64,
    pub bid_levels: Vec<PriceLevelSnapshot>,
    pub ask_levels: Vec<PriceLevelSnapshot>,
    pub next_order_id: OrderId,
    pub next_sequence_number: SequenceNumber,
}

impl AsterEngine {
    /// Creates an empty engine with deterministic counters starting at 1.
    pub fn new() -> Self {
        Self {
            order_book: OrderBook::new(),
            event_log: Vec::new(),
            next_order_id: FIRST_ORDER_ID,
            next_sequence_number: FIRST_SEQUENCE_NUMBER,
        }
    }

    /// Processes one inbound command and returns emitted facts.
    pub fn process_command(&mut self, command: EngineCommand) -> Vec<EngineEvent> {
        let events = match command {
            EngineCommand::SubmitOrder(request) => self.process_submit_order(request),
            EngineCommand::CancelOrder {
                order_id,
                participant_id,
            } => self.process_cancel_order(order_id, participant_id),
        };

        self.event_log.extend(events.iter().copied());

        events
    }

    /// Processes commands in order and concatenates emitted events.
    pub fn process_commands<I>(&mut self, commands: I) -> Vec<EngineEvent>
    where
        I: IntoIterator<Item = EngineCommand>,
    {
        commands
            .into_iter()
            .flat_map(|command| self.process_command(command))
            .collect()
    }

    /// Returns the engine's internal passive order book.
    pub const fn order_book(&self) -> &OrderBook {
        &self.order_book
    }

    /// Returns the in-memory append-only event history.
    pub fn event_log(&self) -> &[EngineEvent] {
        &self.event_log
    }

    /// Clears only the retained event history.
    pub fn clear_event_log(&mut self) {
        self.event_log.clear();
    }

    /// Returns a deterministic state snapshot for replay comparisons.
    pub fn snapshot(&self) -> EngineSnapshot {
        EngineSnapshot {
            best_bid: self.order_book.best_bid_price(),
            best_ask: self.order_book.best_ask_price(),
            bid_level_count: self.order_book.bid_level_count(),
            ask_level_count: self.order_book.ask_level_count(),
            total_resting_quantity: self.order_book.total_resting_quantity(),
            bid_levels: self
                .order_book
                .bid_levels_in_matching_order()
                .map(snapshot_level)
                .collect(),
            ask_levels: self
                .order_book
                .ask_levels_in_matching_order()
                .map(snapshot_level)
                .collect(),
            next_order_id: OrderId::new(self.next_order_id),
            next_sequence_number: SequenceNumber::new(self.next_sequence_number),
        }
    }

    fn process_submit_order(&mut self, request: OrderRequest) -> Vec<EngineEvent> {
        let Some(next_order_id) = self.next_order_id.checked_add(1) else {
            return vec![EngineEvent::OrderRejected {
                reason: AsterError::OrderIdExhausted,
            }];
        };
        let Some(next_sequence_number) = self.next_sequence_number.checked_add(1) else {
            return vec![EngineEvent::OrderRejected {
                reason: AsterError::SequenceNumberExhausted,
            }];
        };
        let order = AcceptedOrder::new(
            OrderId::new(self.next_order_id),
            SequenceNumber::new(self.next_sequence_number),
            request,
        );
        let mut events = vec![EngineEvent::OrderAccepted { order }];

        match self.match_and_maybe_rest(order, &mut events) {
            Ok(()) => {
                self.next_order_id = next_order_id;
                self.next_sequence_number = next_sequence_number;
                events
            }
            Err(reason) => vec![EngineEvent::OrderRejected { reason }],
        }
    }

    fn process_cancel_order(
        &mut self,
        order_id: OrderId,
        participant_id: ParticipantId,
    ) -> Vec<EngineEvent> {
        match self.order_book.cancel_order(order_id, participant_id) {
            Ok(_) => vec![EngineEvent::OrderCancelled {
                order_id,
                participant_id,
            }],
            Err(reason) => vec![EngineEvent::CancelRejected {
                order_id,
                participant_id,
                reason,
            }],
        }
    }

    fn match_and_maybe_rest(
        &mut self,
        order: AcceptedOrder,
        events: &mut Vec<EngineEvent>,
    ) -> Result<(), AsterError> {
        let mut remaining_quantity = order.quantity.as_u64();
        let limit_price = match order.order_type {
            OrderType::Limit { price } => Some(price),
            OrderType::Market => None,
        };

        while remaining_quantity > 0 && self.can_match(order.side, limit_price) {
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

        if remaining_quantity > 0 && matches!(order.order_type, OrderType::Limit { .. }) {
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

    fn can_match(&self, side: Side, limit_price: Option<PriceTicks>) -> bool {
        match limit_price {
            Some(price) => self.crosses_book(side, price),
            None => self.best_opposing_front_order(side).is_some(),
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

fn snapshot_level(level: &crate::PriceLevel) -> PriceLevelSnapshot {
    PriceLevelSnapshot {
        price: level.price(),
        orders: level.orders_in_fifo_order().copied().collect(),
    }
}

impl Default for AsterEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::AsterEngine;
    use crate::{
        AsterError, EngineCommand, EngineEvent, OrderRequest, OrderType, ParticipantId, Quantity,
        Side,
    };

    fn market_submission() -> EngineCommand {
        EngineCommand::submit_order(OrderRequest::new(
            ParticipantId::new(1),
            Side::Buy,
            OrderType::Market,
            Quantity::new(1).expect("test quantity is positive"),
        ))
    }

    #[test]
    fn order_id_exhaustion_rejects_without_mutating_engine_state() {
        let mut engine = AsterEngine::new();
        engine.next_order_id = u64::MAX;
        let before = engine.snapshot();

        let events = engine.process_command(market_submission());

        assert_eq!(
            events,
            vec![EngineEvent::OrderRejected {
                reason: AsterError::OrderIdExhausted,
            }]
        );
        assert_eq!(engine.snapshot(), before);
        assert_eq!(engine.event_log(), events);
    }

    #[test]
    fn sequence_exhaustion_rejects_without_mutating_engine_state() {
        let mut engine = AsterEngine::new();
        engine.next_sequence_number = u64::MAX;
        let before = engine.snapshot();

        let events = engine.process_command(market_submission());

        assert_eq!(
            events,
            vec![EngineEvent::OrderRejected {
                reason: AsterError::SequenceNumberExhausted,
            }]
        );
        assert_eq!(engine.snapshot(), before);
        assert_eq!(engine.event_log(), events);
    }
}
