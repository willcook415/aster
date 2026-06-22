use aster_core::{
    replay_commands, AsterEngine, EngineCommand, EngineEvent, EngineSnapshot, OrderId,
    OrderRequest, OrderType, ParticipantId, PriceTicks, Quantity, Side,
};

fn price(value: u64) -> PriceTicks {
    PriceTicks::new(value).expect("valid price")
}

fn quantity(value: u64) -> Quantity {
    Quantity::new(value).expect("valid quantity")
}

fn limit_command(
    participant_id: u64,
    side: Side,
    price: PriceTicks,
    quantity_value: u64,
) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(participant_id),
        side,
        OrderType::Limit { price },
        quantity(quantity_value),
    ))
}

fn market_command(participant_id: u64, side: Side, quantity_value: u64) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(participant_id),
        side,
        OrderType::Market,
        quantity(quantity_value),
    ))
}

fn assert_replay_matches_direct(
    commands: Vec<EngineCommand>,
) -> (Vec<EngineEvent>, EngineSnapshot) {
    let mut engine = AsterEngine::new();
    let direct_events = engine.process_commands(commands.clone());
    let direct_snapshot = engine.snapshot();

    let replay = replay_commands(commands);

    assert_eq!(direct_events, replay.events);
    assert_eq!(direct_snapshot, replay.final_snapshot);

    (direct_events, direct_snapshot)
}

#[test]
fn replaying_empty_command_list_produces_no_events_and_empty_snapshot() {
    let replay = replay_commands(Vec::new());

    assert_eq!(replay.events, Vec::new());
    assert_eq!(
        replay.final_snapshot,
        EngineSnapshot {
            best_bid: None,
            best_ask: None,
            bid_level_count: 0,
            ask_level_count: 0,
            total_resting_quantity: 0,
        }
    );
}

#[test]
fn replaying_passive_limit_orders_matches_direct_execution() {
    assert_replay_matches_direct(vec![
        limit_command(1, Side::Buy, price(100), 10),
        limit_command(2, Side::Sell, price(105), 15),
    ]);
}

#[test]
fn replaying_crossing_limit_matching_matches_direct_execution() {
    assert_replay_matches_direct(vec![
        limit_command(1, Side::Sell, price(100), 10),
        limit_command(2, Side::Buy, price(100), 4),
    ]);
}

#[test]
fn replaying_market_order_execution_matches_direct_execution() {
    assert_replay_matches_direct(vec![
        limit_command(1, Side::Sell, price(100), 5),
        limit_command(2, Side::Sell, price(101), 5),
        market_command(3, Side::Buy, 8),
    ]);
}

#[test]
fn replaying_cancellations_matches_direct_execution() {
    assert_replay_matches_direct(vec![
        limit_command(1, Side::Buy, price(99), 10),
        limit_command(2, Side::Buy, price(100), 10),
        EngineCommand::cancel_order(OrderId::new(2), ParticipantId::new(2)),
    ]);
}

#[test]
fn replay_preserves_deterministic_order_ids_and_sequence_numbers() {
    let (events, _) = assert_replay_matches_direct(vec![
        limit_command(1, Side::Buy, price(100), 10),
        market_command(2, Side::Sell, 3),
        limit_command(3, Side::Sell, price(101), 7),
    ]);

    let accepted: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            EngineEvent::OrderAccepted { order } => Some(order),
            _ => None,
        })
        .collect();

    assert_eq!(accepted[0].order_id.as_u64(), 1);
    assert_eq!(accepted[0].sequence_number.as_u64(), 1);
    assert_eq!(accepted[1].order_id.as_u64(), 2);
    assert_eq!(accepted[1].sequence_number.as_u64(), 2);
    assert_eq!(accepted[2].order_id.as_u64(), 3);
    assert_eq!(accepted[2].sequence_number.as_u64(), 3);
}

#[test]
fn replaying_mixed_session_matches_direct_execution() {
    assert_replay_matches_direct(vec![
        limit_command(1, Side::Buy, price(99), 10),
        limit_command(2, Side::Sell, price(105), 10),
        limit_command(3, Side::Sell, price(100), 6),
        limit_command(4, Side::Buy, price(101), 4),
        market_command(5, Side::Sell, 3),
        EngineCommand::cancel_order(OrderId::new(2), ParticipantId::new(2)),
        EngineCommand::cancel_order(OrderId::new(999), ParticipantId::new(9)),
    ]);
}
