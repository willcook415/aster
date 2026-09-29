use aster_core::*;
use proptest::prelude::*;

fn order(id: u64, side: Side, ticks: u64, units: u64) -> AcceptedOrder {
    AcceptedOrder::new(
        OrderId::new(id),
        SequenceNumber::new(id),
        OrderRequest::new(
            ParticipantId::new(id % 3),
            side,
            OrderType::Limit {
                price: PriceTicks::new(ticks).unwrap(),
            },
            Quantity::new(units).unwrap(),
        ),
    )
}
fn check_book(book: &OrderBook) {
    let mut copy = book.clone();
    let mut sum = 0_u64;
    while let Some(order) = copy
        .pop_best_bid_front_order()
        .or_else(|| copy.pop_best_ask_front_order())
    {
        sum = sum.checked_add(order.quantity.as_u64()).unwrap();
    }
    assert_eq!(book.total_resting_quantity(), sum);
    assert_eq!(copy.total_resting_quantity(), 0);
}

#[test]
fn public_mutations_keep_totals_consistent_on_both_sides() {
    let mut book = OrderBook::new();
    for id in 1..=20 {
        let side = if id % 2 == 0 { Side::Buy } else { Side::Sell };
        book.add_resting_order(order(id, side, 100 + id % 3, id))
            .unwrap();
        check_book(&book);
    }
    book.reduce_best_bid_front_quantity(Quantity::new(1).unwrap())
        .unwrap();
    book.reduce_best_ask_front_quantity(Quantity::new(1).unwrap())
        .unwrap();
    check_book(&book);
    let before = book.clone();
    assert!(book
        .cancel_order(OrderId::new(1), ParticipantId::new(99))
        .is_err());
    assert!(book
        .reduce_best_bid_front_quantity(Quantity::new(u64::MAX).unwrap())
        .is_err());
    assert!(book.add_resting_order(order(1, Side::Buy, 100, 1)).is_err());
    assert_eq!(book, before);
    for id in [10, 3, 18, 9] {
        book.cancel_order(OrderId::new(id), ParticipantId::new(id % 3))
            .unwrap();
        check_book(&book);
    }
}

#[test]
fn level_aggregate_tracks_middle_removal_reduction_and_drain() {
    let mut level = PriceLevel::new(PriceTicks::new(100).unwrap());
    for id in 1..=5 {
        level.push_back(order(id, Side::Buy, 100, id)).unwrap();
    }
    level.remove_order(OrderId::new(3)).unwrap();
    level
        .reduce_front_quantity(Quantity::new(1).unwrap())
        .unwrap();
    assert_eq!(level.total_quantity(), 12);
    let mut actual = 0;
    while let Some(order) = level.pop_front() {
        actual += order.quantity.as_u64();
    }
    assert_eq!(actual, 12);
    assert_eq!(level.total_quantity(), 0);
}

proptest! {
    #[test]
    fn snapshot_totals_and_batch_partitioning_agree(actions in prop::collection::vec((any::<bool>(),95_u64..106,1_u64..100,0_u8..5),1..150)) {
        let commands: Vec<_> = actions.into_iter().enumerate().map(|(i,(buy,price,quantity,kind))| {
            if kind==0 {EngineCommand::cancel_order(OrderId::new(i as u64/2+1),ParticipantId::new(1))}
            else {EngineCommand::submit_order(OrderRequest::new(ParticipantId::new(1),if buy {Side::Buy} else {Side::Sell},
                if kind==1 {OrderType::Market} else {OrderType::Limit{price:PriceTicks::new(price).unwrap()}}, Quantity::new(quantity).unwrap()))}
        }).collect();
        let mut engine=AsterEngine::new();
        for command in &commands {
            engine.process_command(*command);
            let snapshot=engine.snapshot();
            let sum:u64=snapshot.bid_levels.iter().chain(&snapshot.ask_levels).flat_map(|l|&l.orders).map(|o|o.quantity.as_u64()).sum();
            prop_assert_eq!(snapshot.total_resting_quantity,sum);
        }
        let mut batched=AsterEngine::new();
        for chunk in commands.chunks(7) {batched.process_commands(chunk.iter().copied());}
        prop_assert_eq!(engine.snapshot(),batched.snapshot());
        prop_assert_eq!(engine.event_log(),batched.event_log());
    }
}
