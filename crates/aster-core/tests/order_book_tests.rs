use aster_core::{
    AcceptedOrder, AsterError, OrderBook, OrderId, OrderRequest, OrderType, ParticipantId,
    PriceLevel, PriceTicks, Quantity, SequenceNumber, Side,
};

fn price(value: u64) -> PriceTicks {
    PriceTicks::new(value).expect("valid price")
}

fn quantity(value: u64) -> Quantity {
    Quantity::new(value).expect("valid quantity")
}

fn accepted_limit_order(
    order_id: u64,
    side: Side,
    price: PriceTicks,
    quantity_value: u64,
) -> AcceptedOrder {
    let request = OrderRequest::new(
        ParticipantId::new(1),
        side,
        OrderType::Limit { price },
        quantity(quantity_value),
    );

    AcceptedOrder::new(
        OrderId::new(order_id),
        SequenceNumber::new(order_id),
        request,
    )
}

fn accepted_market_order(order_id: u64, side: Side) -> AcceptedOrder {
    let request = OrderRequest::new(ParticipantId::new(1), side, OrderType::Market, quantity(10));

    AcceptedOrder::new(
        OrderId::new(order_id),
        SequenceNumber::new(order_id),
        request,
    )
}

#[test]
fn creates_empty_order_book() {
    let book = OrderBook::new();

    assert!(book.is_empty());
    assert_eq!(book.bid_level_count(), 0);
    assert_eq!(book.ask_level_count(), 0);
    assert_eq!(book.best_bid_price(), None);
    assert_eq!(book.best_ask_price(), None);
    assert_eq!(book.best_bid_level(), None);
    assert_eq!(book.best_ask_level(), None);
    assert_eq!(book.total_resting_quantity(), 0);
}

#[test]
fn adding_buy_limit_order_creates_bid_level() {
    let mut book = OrderBook::new();
    let order = accepted_limit_order(1, Side::Buy, price(100), 10);

    book.add_resting_order(order).expect("valid bid order");

    assert!(!book.is_empty());
    assert_eq!(book.bid_level_count(), 1);
    assert_eq!(book.ask_level_count(), 0);
    assert_eq!(book.best_bid_price(), Some(price(100)));
    assert_eq!(
        book.best_bid_level().and_then(PriceLevel::front),
        Some(&order)
    );
}

#[test]
fn adding_sell_limit_order_creates_ask_level() {
    let mut book = OrderBook::new();
    let order = accepted_limit_order(1, Side::Sell, price(101), 10);

    book.add_resting_order(order).expect("valid ask order");

    assert_eq!(book.bid_level_count(), 0);
    assert_eq!(book.ask_level_count(), 1);
    assert_eq!(book.best_ask_price(), Some(price(101)));
    assert_eq!(
        book.best_ask_level().and_then(PriceLevel::front),
        Some(&order)
    );
}

#[test]
fn market_orders_cannot_rest() {
    let mut book = OrderBook::new();
    let order = accepted_market_order(1, Side::Buy);

    assert_eq!(
        book.add_resting_order(order),
        Err(AsterError::MarketOrderCannotRest)
    );
    assert!(book.is_empty());
}

#[test]
fn multiple_buy_levels_choose_highest_best_bid() {
    let mut book = OrderBook::new();

    book.add_resting_order(accepted_limit_order(1, Side::Buy, price(99), 10))
        .expect("valid bid");
    book.add_resting_order(accepted_limit_order(2, Side::Buy, price(101), 10))
        .expect("valid bid");

    assert_eq!(book.best_bid_price(), Some(price(101)));
    assert_eq!(book.bid_level_count(), 2);
}

#[test]
fn multiple_sell_levels_choose_lowest_best_ask() {
    let mut book = OrderBook::new();

    book.add_resting_order(accepted_limit_order(1, Side::Sell, price(105), 10))
        .expect("valid ask");
    book.add_resting_order(accepted_limit_order(2, Side::Sell, price(103), 10))
        .expect("valid ask");

    assert_eq!(book.best_ask_price(), Some(price(103)));
    assert_eq!(book.ask_level_count(), 2);
}

#[test]
fn preserves_fifo_within_same_bid_price_level() {
    let mut book = OrderBook::new();
    let first = accepted_limit_order(1, Side::Buy, price(100), 10);
    let second = accepted_limit_order(2, Side::Buy, price(100), 20);

    book.add_resting_order(first).expect("valid first bid");
    book.add_resting_order(second).expect("valid second bid");

    let level = book.best_bid_level().expect("bid level exists");
    assert_eq!(level.front(), Some(&first));
    assert_eq!(level.len(), 2);
}

#[test]
fn preserves_fifo_within_same_ask_price_level() {
    let mut book = OrderBook::new();
    let first = accepted_limit_order(1, Side::Sell, price(100), 10);
    let second = accepted_limit_order(2, Side::Sell, price(100), 20);

    book.add_resting_order(first).expect("valid first ask");
    book.add_resting_order(second).expect("valid second ask");

    let level = book.best_ask_level().expect("ask level exists");
    assert_eq!(level.front(), Some(&first));
    assert_eq!(level.len(), 2);
}

#[test]
fn same_price_orders_reuse_price_level() {
    let mut book = OrderBook::new();

    book.add_resting_order(accepted_limit_order(1, Side::Buy, price(100), 10))
        .expect("valid bid");
    book.add_resting_order(accepted_limit_order(2, Side::Buy, price(100), 20))
        .expect("valid bid");

    assert_eq!(book.bid_level_count(), 1);
    assert_eq!(book.best_bid_level().map(PriceLevel::len), Some(2));
}

#[test]
fn contains_order_detects_present_and_missing_orders() {
    let mut book = OrderBook::new();

    book.add_resting_order(accepted_limit_order(1, Side::Buy, price(100), 10))
        .expect("valid bid");

    assert!(book.contains_order(OrderId::new(1)));
    assert!(!book.contains_order(OrderId::new(2)));
}

#[test]
fn total_resting_quantity_sums_across_both_sides() {
    let mut book = OrderBook::new();

    book.add_resting_order(accepted_limit_order(1, Side::Buy, price(100), 10))
        .expect("valid bid");
    book.add_resting_order(accepted_limit_order(2, Side::Sell, price(101), 15))
        .expect("valid ask");

    assert_eq!(book.total_resting_quantity(), 25);
}

#[test]
fn duplicate_order_id_is_rejected() {
    let mut book = OrderBook::new();

    book.add_resting_order(accepted_limit_order(1, Side::Buy, price(100), 10))
        .expect("valid bid");

    assert_eq!(
        book.add_resting_order(accepted_limit_order(1, Side::Sell, price(101), 20)),
        Err(AsterError::DuplicateOrderId)
    );
}
