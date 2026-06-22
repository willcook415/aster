//! Passive bid/ask order book storage.
//!
//! `OrderBook` currently stores already-accepted resting limit orders for a
//! single instrument. Best-price selection across levels is handled here, while
//! FIFO within each price level is delegated to `PriceLevel`. Crossing checks,
//! trade execution, cancellation execution, replay, and persistence belong to
//! future modules.

use std::collections::{BTreeMap, HashMap};

use crate::{AcceptedOrder, AsterError, OrderId, OrderType, PriceLevel, PriceTicks, Side};

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
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}
