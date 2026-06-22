//! FIFO resting-order queue for a single price.
//!
//! A price level holds resting limit orders at exactly one price. FIFO order
//! within the level is the "time" part of price-time priority, using the order
//! sequence already assigned by the engine. Best-price selection across levels
//! belongs to the future order book, not this module.

use std::collections::VecDeque;

use crate::{AcceptedOrder, AsterError, OrderId, OrderType, PriceTicks, Quantity};

/// Resting limit orders at one price, maintained in FIFO order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriceLevel {
    price: PriceTicks,
    orders: VecDeque<AcceptedOrder>,
}

impl PriceLevel {
    /// Creates an empty price level.
    pub fn new(price: PriceTicks) -> Self {
        Self {
            price,
            orders: VecDeque::new(),
        }
    }

    /// Returns the price represented by this level.
    pub const fn price(&self) -> PriceTicks {
        self.price
    }

    /// Returns the number of resting orders in this level.
    pub fn len(&self) -> usize {
        self.orders.len()
    }

    /// Returns whether this level has no resting orders.
    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    /// Returns the sum of resting quantities in this level.
    pub fn total_quantity(&self) -> u64 {
        self.orders
            .iter()
            .map(|order| order.quantity.as_u64())
            .sum()
    }

    /// Adds a resting limit order to the back of the FIFO queue.
    pub fn push_back(&mut self, order: AcceptedOrder) -> Result<(), AsterError> {
        match order.order_type {
            OrderType::Limit { price } if price == self.price => {
                self.orders.push_back(order);
                Ok(())
            }
            OrderType::Limit { .. } => Err(AsterError::PriceLevelMismatch),
            OrderType::Market => Err(AsterError::MarketOrderCannotRest),
        }
    }

    /// Returns the oldest resting order without removing it.
    pub fn front(&self) -> Option<&AcceptedOrder> {
        self.orders.front()
    }

    /// Removes and returns the oldest resting order.
    pub fn pop_front(&mut self) -> Option<AcceptedOrder> {
        self.orders.pop_front()
    }

    /// Reduces the oldest resting order without changing its FIFO priority.
    pub fn reduce_front_quantity(&mut self, new_quantity: Quantity) -> Result<(), AsterError> {
        let front = self
            .orders
            .front_mut()
            .ok_or(AsterError::InvalidOrderState)?;

        front.quantity = new_quantity;

        Ok(())
    }

    /// Removes and returns an order by ID, preserving remaining FIFO order.
    pub fn remove_order(&mut self, order_id: OrderId) -> Option<AcceptedOrder> {
        let index = self
            .orders
            .iter()
            .position(|order| order.order_id == order_id)?;

        self.orders.remove(index)
    }

    /// Returns whether this level contains an order ID.
    pub fn contains_order(&self, order_id: OrderId) -> bool {
        self.orders.iter().any(|order| order.order_id == order_id)
    }
}
