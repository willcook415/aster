//! Passive bid/ask order book storage.
//!
//! `OrderBook` currently stores already-accepted resting limit orders for a
//! single instrument. Best-price selection across levels is handled here, while
//! FIFO within each price level is delegated to `PriceLevel`. Crossing checks,
//! trade execution, cancellation execution, replay, and persistence belong to
//! future modules.

use std::collections::{BTreeMap, HashMap};

use crate::{
    AcceptedOrder, AsterError, OrderId, OrderType, ParticipantId, PriceLevel, PriceTicks, Quantity,
    Side,
};

/// Single-instrument passive order book storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderBook {
    bids: BTreeMap<PriceTicks, PriceLevel>,
    asks: BTreeMap<PriceTicks, PriceLevel>,
    order_index: HashMap<OrderId, (Side, PriceTicks)>,
}

impl OrderBook {
    /// Creates an empty order book.
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_index: HashMap::new(),
        }
    }

    /// Returns whether both sides of the book are empty.
    pub fn is_empty(&self) -> bool {
        self.bids.is_empty() && self.asks.is_empty()
    }

    /// Returns the number of bid-side price levels.
    pub fn bid_level_count(&self) -> usize {
        self.bids.len()
    }

    /// Returns the number of ask-side price levels.
    pub fn ask_level_count(&self) -> usize {
        self.asks.len()
    }

    /// Returns the highest bid price.
    pub fn best_bid_price(&self) -> Option<PriceTicks> {
        self.bids.keys().next_back().copied()
    }

    /// Returns the lowest ask price.
    pub fn best_ask_price(&self) -> Option<PriceTicks> {
        self.asks.keys().next().copied()
    }

    /// Returns the highest bid price level.
    pub fn best_bid_level(&self) -> Option<&PriceLevel> {
        self.bids.values().next_back()
    }

    /// Returns the lowest ask price level.
    pub fn best_ask_level(&self) -> Option<&PriceLevel> {
        self.asks.values().next()
    }

    /// Returns the oldest order at the highest bid price.
    pub fn best_bid_front_order(&self) -> Option<&AcceptedOrder> {
        self.best_bid_level()?.front()
    }

    /// Returns the oldest order at the lowest ask price.
    pub fn best_ask_front_order(&self) -> Option<&AcceptedOrder> {
        self.best_ask_level()?.front()
    }

    /// Returns whether the book contains an order ID on either side.
    pub fn contains_order(&self, order_id: OrderId) -> bool {
        self.order_index.contains_key(&order_id)
    }

    /// Returns total resting quantity across both sides of the book.
    pub fn total_resting_quantity(&self) -> u64 {
        self.bids
            .values()
            .chain(self.asks.values())
            .map(PriceLevel::total_quantity)
            .sum()
    }

    /// Cancels a resting order if it exists and belongs to the participant.
    pub fn cancel_order(
        &mut self,
        order_id: OrderId,
        participant_id: ParticipantId,
    ) -> Result<AcceptedOrder, AsterError> {
        let (side, price) = self
            .order_index
            .get(&order_id)
            .copied()
            .ok_or(AsterError::OrderNotFound)?;
        let order = self
            .level(side, price)
            .and_then(|level| level.get_order(order_id))
            .copied()
            .ok_or(AsterError::InvalidOrderState)?;

        if order.participant_id != participant_id {
            return Err(AsterError::ParticipantMismatch);
        }

        let removed = self
            .remove_order_at(side, price, order_id)
            .ok_or(AsterError::InvalidOrderState)?;
        self.order_index.remove(&order_id);

        Ok(removed)
    }

    /// Adds an already-accepted passive resting limit order.
    ///
    /// This method stores orders only. It does not check for crossing, execute
    /// trades, emit events, or assign engine metadata.
    pub fn add_resting_order(&mut self, order: AcceptedOrder) -> Result<(), AsterError> {
        if self.contains_order(order.order_id) {
            return Err(AsterError::DuplicateOrderId);
        }

        let OrderType::Limit { price } = order.order_type else {
            return Err(AsterError::MarketOrderCannotRest);
        };

        let levels = match order.side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        levels
            .entry(price)
            .or_insert_with(|| PriceLevel::new(price))
            .push_back(order)?;
        self.order_index.insert(order.order_id, (order.side, price));

        Ok(())
    }

    /// Removes and returns the oldest order at the highest bid price.
    pub fn pop_best_bid_front_order(&mut self) -> Option<AcceptedOrder> {
        let price = self.best_bid_price()?;
        self.pop_front_order(Side::Buy, price)
    }

    /// Removes and returns the oldest order at the lowest ask price.
    pub fn pop_best_ask_front_order(&mut self) -> Option<AcceptedOrder> {
        let price = self.best_ask_price()?;
        self.pop_front_order(Side::Sell, price)
    }

    /// Reduces the oldest order at the highest bid price.
    pub fn reduce_best_bid_front_quantity(
        &mut self,
        new_quantity: Quantity,
    ) -> Result<(), AsterError> {
        let price = self.best_bid_price().ok_or(AsterError::InvalidOrderState)?;
        self.reduce_front_quantity(Side::Buy, price, new_quantity)
    }

    /// Reduces the oldest order at the lowest ask price.
    pub fn reduce_best_ask_front_quantity(
        &mut self,
        new_quantity: Quantity,
    ) -> Result<(), AsterError> {
        let price = self.best_ask_price().ok_or(AsterError::InvalidOrderState)?;
        self.reduce_front_quantity(Side::Sell, price, new_quantity)
    }

    fn pop_front_order(&mut self, side: Side, price: PriceTicks) -> Option<AcceptedOrder> {
        let levels = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };
        let level = levels.get_mut(&price)?;
        let order = level.pop_front()?;
        self.order_index.remove(&order.order_id);
        if level.is_empty() {
            levels.remove(&price);
        }

        Some(order)
    }

    fn remove_order_at(
        &mut self,
        side: Side,
        price: PriceTicks,
        order_id: OrderId,
    ) -> Option<AcceptedOrder> {
        let levels = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };
        let level = levels.get_mut(&price)?;
        let order = level.remove_order(order_id)?;
        if level.is_empty() {
            levels.remove(&price);
        }

        Some(order)
    }

    fn level(&self, side: Side, price: PriceTicks) -> Option<&PriceLevel> {
        match side {
            Side::Buy => self.bids.get(&price),
            Side::Sell => self.asks.get(&price),
        }
    }

    fn reduce_front_quantity(
        &mut self,
        side: Side,
        price: PriceTicks,
        new_quantity: Quantity,
    ) -> Result<(), AsterError> {
        let levels = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };
        let level = levels
            .get_mut(&price)
            .ok_or(AsterError::InvalidOrderState)?;

        level.reduce_front_quantity(new_quantity)
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}
