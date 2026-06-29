use aster_core::{
    AsterEngine, AsterError, EngineCommand, EngineEvent, OrderRequest, OrderType, ParticipantId,
    PriceTicks, Quantity, Side,
};

#[test]
fn late_resting_overflow_rejects_without_partial_mutation() {
    let mut engine = AsterEngine::new();
    engine.process_command(limit_command(1, Side::Buy, 1, u64::MAX - 1));
    engine.process_command(limit_command(2, Side::Sell, 2, 1));

    let before = engine.snapshot();
    let event_log_before = engine.event_log().to_vec();

    let rejected = engine.process_command(limit_command(3, Side::Buy, 2, u64::MAX));

    assert_eq!(
        rejected,
        vec![EngineEvent::OrderRejected {
            reason: AsterError::QuantityOverflow,
        }]
    );
    assert_eq!(engine.snapshot(), before);
    assert_eq!(
        engine.event_log(),
        [event_log_before.as_slice(), rejected.as_slice(),].concat()
    );
    assert_eq!(engine.snapshot().next_order_id, before.next_order_id);
    assert_eq!(
        engine.snapshot().next_sequence_number,
        before.next_sequence_number
    );
    assert_eq!(engine.snapshot().best_ask, Some(price(2)));
    assert_eq!(engine.snapshot().total_resting_quantity, u64::MAX);
    assert!(engine.event_log()[event_log_before.len()..]
        .iter()
        .all(|event| matches!(event, EngineEvent::OrderRejected { .. })));
}

#[test]
fn successful_crossing_limit_still_accepts_then_trades() {
    let mut engine = AsterEngine::new();
    engine.process_command(limit_command(1, Side::Sell, 100, 5));

    let events = engine.process_command(limit_command(2, Side::Buy, 100, 8));

    assert!(matches!(events[0], EngineEvent::OrderAccepted { .. }));
    assert!(matches!(
        events[1],
        EngineEvent::TradeExecuted {
            quantity,
            ..
        } if quantity.as_u64() == 5
    ));
    assert_eq!(engine.snapshot().best_bid, Some(price(100)));
    assert_eq!(engine.snapshot().best_ask, None);
    assert_eq!(engine.snapshot().total_resting_quantity, 3);
}

fn limit_command(participant_id: u64, side: Side, price_ticks: u64, units: u64) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(participant_id),
        side,
        OrderType::Limit {
            price: price(price_ticks),
        },
        Quantity::new(units).expect("test quantity must be positive"),
    ))
}

fn price(value: u64) -> PriceTicks {
    PriceTicks::new(value).expect("test price must be positive")
}
