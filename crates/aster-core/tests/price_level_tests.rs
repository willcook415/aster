use aster_core::{
    AcceptedOrder, AsterError, OrderId, OrderRequest, OrderType, ParticipantId, PriceLevel,
    PriceTicks, Quantity, SequenceNumber, Side,
};

fn price(value: u64) -> PriceTicks {
    PriceTicks::new(value).expect("valid price")
}

fn quantity(value: u64) -> Quantity {
    Quantity::new(value).expect("valid quantity")
}

fn accepted_limit_order(order_id: u64, price: PriceTicks, quantity_value: u64) -> AcceptedOrder {
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
fn reduce_front_quantity_preserves_front_order_identity_and_priority() {
    let mut level = PriceLevel::new(price(100));
    let first = accepted_limit_order(1, price(100), 10);
    let second = accepted_limit_order(2, price(100), 20);

    level.push_back(first).expect("valid first order");
    level.push_back(second).expect("valid second order");
    level
        .reduce_front_quantity(quantity(4))
        .expect("front order exists");

    let front = level.front().expect("front order remains");
    assert_eq!(front.order_id, first.order_id);
    assert_eq!(front.sequence_number, first.sequence_number);
    assert_eq!(front.quantity.as_u64(), 4);
    assert_eq!(
        level.pop_front().expect("front order exists").order_id,
        first.order_id
    );
    assert_eq!(level.pop_front(), Some(second));
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
fn rejects_quantity_that_would_overflow_level_total() {
    let mut level = PriceLevel::new(price(100));
    level
        .push_back(accepted_limit_order(1, price(100), u64::MAX))
        .expect("maximum quantity fits in an empty level");

    let result = level.push_back(accepted_limit_order(2, price(100), 1));

    assert_eq!(result, Err(AsterError::QuantityOverflow));
    assert_eq!(level.len(), 1);
    assert_eq!(level.total_quantity(), u64::MAX);
}

#[test]
fn rejects_front_quantity_change_that_would_overflow_level_total() {
    let mut level = PriceLevel::new(price(100));
    level
        .push_back(accepted_limit_order(1, price(100), 1))
        .expect("first order fits");
    level
        .push_back(accepted_limit_order(2, price(100), u64::MAX - 1))
        .expect("level total remains representable");

    let result = level.reduce_front_quantity(quantity(2));

    assert_eq!(result, Err(AsterError::QuantityOverflow));
    assert_eq!(level.front().expect("front remains").quantity.as_u64(), 1);
    assert_eq!(level.total_quantity(), u64::MAX);
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
