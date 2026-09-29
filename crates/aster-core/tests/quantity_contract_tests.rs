use aster_core::*;

fn order(id: u64, side: Side, price: u64, units: u64) -> AcceptedOrder {
    AcceptedOrder::new(
        OrderId::new(id),
        SequenceNumber::new(id),
        OrderRequest::new(
            ParticipantId::new(1),
            side,
            OrderType::Limit {
                price: PriceTicks::new(price).unwrap(),
            },
            Quantity::new(units).unwrap(),
        ),
    )
}

#[test]
fn level_increase_is_atomic_and_equal_quantity_preserves_priority() {
    let mut level = PriceLevel::new(PriceTicks::new(100).unwrap());
    level.push_back(order(1, Side::Buy, 100, 5)).unwrap();
    level.push_back(order(2, Side::Buy, 100, 3)).unwrap();
    let before = level.clone();
    assert_eq!(
        level.reduce_front_quantity(Quantity::new(6).unwrap()),
        Err(AsterError::InvalidOrderState)
    );
    assert_eq!(level, before);
    level
        .reduce_front_quantity(Quantity::new(5).unwrap())
        .unwrap();
    assert_eq!(level, before);
    level
        .reduce_front_quantity(Quantity::new(2).unwrap())
        .unwrap();
    assert_eq!(level.pop_front().unwrap().order_id, OrderId::new(1));
    assert_eq!(level.pop_front().unwrap().order_id, OrderId::new(2));
}

#[test]
fn both_book_sides_reject_increases_at_global_capacity_without_mutation() {
    for side in [Side::Buy, Side::Sell] {
        let mut book = OrderBook::new();
        let other_price = if side == Side::Buy { 99 } else { 101 };
        book.add_resting_order(order(1, side, 100, 1)).unwrap();
        book.add_resting_order(order(2, side, other_price, u64::MAX - 1))
            .unwrap();
        let before = book.clone();
        let result = match side {
            Side::Buy => book.reduce_best_bid_front_quantity(Quantity::new(2).unwrap()),
            Side::Sell => book.reduce_best_ask_front_quantity(Quantity::new(2).unwrap()),
        };
        assert_eq!(result, Err(AsterError::InvalidOrderState));
        assert_eq!(book, before);
        assert_eq!(book.total_resting_quantity(), u64::MAX);
        assert_eq!(
            book.cancel_order(OrderId::new(1), ParticipantId::new(1))
                .unwrap()
                .quantity
                .as_u64(),
            1
        );
        assert_eq!(book.total_resting_quantity(), u64::MAX - 1);
    }
}
