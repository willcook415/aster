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
    total_quantity: u64,
}

impl PriceLevel {
    /// Creates an empty price level.
    pub fn new(price: PriceTicks) -> Self {
        Self {
            price,
            orders: VecDeque::new(),
            total_quantity: 0,
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
        self.total_quantity
    }

    /// Adds a resting limit order to the back of the FIFO queue.
    pub fn push_back(&mut self, order: AcceptedOrder) -> Result<(), AsterError> {
        match order.order_type {
            OrderType::Limit { price } if price == self.price => {
                let total = self
                    .total_quantity
                    .checked_add(order.quantity.as_u64())
                    .ok_or(AsterError::QuantityOverflow)?;
                self.orders.push_back(order);
                self.total_quantity = total;
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
        let order = self.orders.pop_front()?;
        self.total_quantity -= order.quantity.as_u64();
        Some(order)
    }

    /// Reduces the oldest resting order without changing its FIFO priority.
    /// Equal quantity is a no-op. Increases are rejected before mutation.
    pub fn reduce_front_quantity(&mut self, new_quantity: Quantity) -> Result<(), AsterError> {
        let current_front_quantity = self
            .orders
            .front()
            .ok_or(AsterError::InvalidOrderState)?
            .quantity
            .as_u64();
        if new_quantity.as_u64() > current_front_quantity {
            return Err(AsterError::InvalidOrderState);
        }
        let front = self
            .orders
            .front_mut()
            .ok_or(AsterError::InvalidOrderState)?;

        front.quantity = new_quantity;
        self.total_quantity -= current_front_quantity - new_quantity.as_u64();

        Ok(())
    }

    /// Removes and returns an order by ID, preserving remaining FIFO order.
    pub fn remove_order(&mut self, order_id: OrderId) -> Option<AcceptedOrder> {
        let index = self
            .orders
            .iter()
            .position(|order| order.order_id == order_id)?;

        let order = self.orders.remove(index)?;
        self.total_quantity -= order.quantity.as_u64();
        Some(order)
    }

    /// Returns whether this level contains an order ID.
    pub fn contains_order(&self, order_id: OrderId) -> bool {
        self.orders.iter().any(|order| order.order_id == order_id)
    }

    /// Returns an order by ID without removing it.
    pub fn get_order(&self, order_id: OrderId) -> Option<&AcceptedOrder> {
        self.orders.iter().find(|order| order.order_id == order_id)
    }

    /// Iterates over resting orders from oldest to newest.
    pub(crate) fn orders_in_fifo_order(&self) -> impl Iterator<Item = &AcceptedOrder> {
        self.orders.iter()
    }
}
