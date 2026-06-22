//! FIFO resting-order queue for a single price.
//!
//! A price level holds resting limit orders at exactly one price. FIFO order
//! within the level is the "time" part of price-time priority, using the order
//! sequence already assigned by the engine. Best-price selection across levels
//! belongs to the future order book, not this module.

use std::collections::VecDeque;

use crate::{AcceptedOrder, AsterError, OrderId, OrderType, PriceTicks};

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

#[cfg(test)]
mod tests {
    use super::PriceLevel;
    use crate::{
        AcceptedOrder, AsterError, OrderId, OrderRequest, OrderType, ParticipantId, PriceTicks,
        Quantity, SequenceNumber, Side,
    };

    fn price(value: u64) -> PriceTicks {
        PriceTicks::new(value).expect("valid price")
    }

    fn quantity(value: u64) -> Quantity {
        Quantity::new(value).expect("valid quantity")
    }

    fn accepted_limit_order(
        order_id: u64,
        price: PriceTicks,
        quantity_value: u64,
    ) -> AcceptedOrder {
        let request = OrderRequest::new(
            ParticipantId::new(1),
            Side::Buy,
            OrderType::Limit { price },
            quantity(quantity_value),
        );

        AcceptedOrder::new(
            OrderId::new(order_id),
            SequenceNumber::new(order_id),
            request,
        )
    }

    fn accepted_market_order(order_id: u64) -> AcceptedOrder {
        let request = OrderRequest::new(
            ParticipantId::new(1),
            Side::Buy,
            OrderType::Market,
            quantity(10),
        );

        AcceptedOrder::new(
            OrderId::new(order_id),
            SequenceNumber::new(order_id),
            request,
        )
    }

    #[test]
    fn creates_empty_price_level() {
        let level = PriceLevel::new(price(100));

        assert_eq!(level.price(), price(100));
        assert_eq!(level.len(), 0);
        assert!(level.is_empty());
        assert_eq!(level.total_quantity(), 0);
        assert_eq!(level.front(), None);
    }

    #[test]
    fn adds_one_valid_limit_order() {
        let mut level = PriceLevel::new(price(100));
        let order = accepted_limit_order(1, price(100), 10);

        level.push_back(order).expect("valid resting order");

        assert_eq!(level.len(), 1);
        assert!(!level.is_empty());
        assert_eq!(level.front(), Some(&order));
    }

    #[test]
    fn rejects_market_order_as_resting_order() {
        let mut level = PriceLevel::new(price(100));
        let order = accepted_market_order(1);

        assert_eq!(
            level.push_back(order),
            Err(AsterError::MarketOrderCannotRest)
        );
        assert!(level.is_empty());
    }

    #[test]
    fn rejects_limit_order_at_wrong_price() {
        let mut level = PriceLevel::new(price(100));
        let order = accepted_limit_order(1, price(101), 10);

        assert_eq!(level.push_back(order), Err(AsterError::PriceLevelMismatch));
        assert!(level.is_empty());
    }

    #[test]
    fn preserves_fifo_order_for_multiple_orders_at_same_price() {
        let mut level = PriceLevel::new(price(100));
        let first = accepted_limit_order(1, price(100), 10);
        let second = accepted_limit_order(2, price(100), 20);
        let third = accepted_limit_order(3, price(100), 30);

        level.push_back(first).expect("valid first order");
        level.push_back(second).expect("valid second order");
        level.push_back(third).expect("valid third order");

        assert_eq!(level.pop_front(), Some(first));
        assert_eq!(level.pop_front(), Some(second));
        assert_eq!(level.pop_front(), Some(third));
        assert_eq!(level.pop_front(), None);
    }

    #[test]
    fn front_returns_oldest_without_removing_it() {
        let mut level = PriceLevel::new(price(100));
        let first = accepted_limit_order(1, price(100), 10);
        let second = accepted_limit_order(2, price(100), 20);

        level.push_back(first).expect("valid first order");
        level.push_back(second).expect("valid second order");

        assert_eq!(level.front(), Some(&first));
        assert_eq!(level.len(), 2);
    }

    #[test]
    fn pop_front_removes_oldest_order() {
        let mut level = PriceLevel::new(price(100));
        let first = accepted_limit_order(1, price(100), 10);
        let second = accepted_limit_order(2, price(100), 20);

        level.push_back(first).expect("valid first order");
        level.push_back(second).expect("valid second order");

        assert_eq!(level.pop_front(), Some(first));
        assert_eq!(level.front(), Some(&second));
        assert_eq!(level.len(), 1);
    }

    #[test]
    fn total_quantity_sums_resting_quantities() {
        let mut level = PriceLevel::new(price(100));

        level
            .push_back(accepted_limit_order(1, price(100), 10))
            .expect("valid first order");
        level
            .push_back(accepted_limit_order(2, price(100), 20))
            .expect("valid second order");

        assert_eq!(level.total_quantity(), 30);
    }

    #[test]
    fn contains_order_detects_present_and_missing_order_ids() {
        let mut level = PriceLevel::new(price(100));

        level
            .push_back(accepted_limit_order(1, price(100), 10))
            .expect("valid order");

        assert!(level.contains_order(OrderId::new(1)));
        assert!(!level.contains_order(OrderId::new(2)));
    }

    #[test]
    fn remove_order_removes_middle_order_and_preserves_remaining_order() {
        let mut level = PriceLevel::new(price(100));
        let first = accepted_limit_order(1, price(100), 10);
        let middle = accepted_limit_order(2, price(100), 20);
        let third = accepted_limit_order(3, price(100), 30);

        level.push_back(first).expect("valid first order");
        level.push_back(middle).expect("valid middle order");
        level.push_back(third).expect("valid third order");

        assert_eq!(level.remove_order(OrderId::new(2)), Some(middle));
        assert_eq!(level.pop_front(), Some(first));
        assert_eq!(level.pop_front(), Some(third));
        assert_eq!(level.pop_front(), None);
    }

    #[test]
    fn remove_order_returns_none_for_missing_order() {
        let mut level = PriceLevel::new(price(100));
        let order = accepted_limit_order(1, price(100), 10);

        level.push_back(order).expect("valid order");

        assert_eq!(level.remove_order(OrderId::new(2)), None);
        assert_eq!(level.pop_front(), Some(order));
    }

    #[test]
    fn empty_level_behaviour_is_predictable() {
        let mut level = PriceLevel::new(price(100));

        assert_eq!(level.front(), None);
        assert_eq!(level.pop_front(), None);
        assert_eq!(level.remove_order(OrderId::new(1)), None);
        assert!(!level.contains_order(OrderId::new(1)));
        assert_eq!(level.total_quantity(), 0);
    }
}
