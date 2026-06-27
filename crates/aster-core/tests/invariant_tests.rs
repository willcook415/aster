use std::collections::HashSet;

use aster_core::{
    replay_commands, AsterEngine, AsterError, EngineCommand, EngineEvent, EngineSnapshot, OrderId,
    OrderRequest, OrderType, ParticipantId, PriceLevelSnapshot, PriceTicks, Quantity, Side,
};

#[test]
fn mixed_session_preserves_book_invariants_after_every_command() {
    let commands = accounting_session();
    let mut engine = AsterEngine::new();

    for (index, command) in commands.into_iter().enumerate() {
        engine.process_command(command);
        assert_snapshot_invariants(&engine.snapshot(), index);
    }
}

#[test]
fn partial_fill_preserves_identity_sequence_and_fifo_position() {
    let mut engine = AsterEngine::new();
    engine.process_command(limit_command(1, Side::Sell, 101, 10));
    engine.process_command(limit_command(2, Side::Sell, 101, 20));
    engine.process_command(limit_command(3, Side::Buy, 101, 4));

    let snapshot = engine.snapshot();
    let orders = &snapshot.ask_levels[0].orders;

    assert_eq!(orders.len(), 2);
    assert_eq!(orders[0].order_id.as_u64(), 1);
    assert_eq!(orders[0].participant_id.as_u64(), 1);
    assert_eq!(orders[0].sequence_number.as_u64(), 1);
    assert_eq!(orders[0].quantity.as_u64(), 6);
    assert_eq!(orders[1].order_id.as_u64(), 2);
    assert_eq!(orders[1].sequence_number.as_u64(), 2);
    assert_snapshot_invariants(&snapshot, 2);
}

#[test]
fn cancellation_preserves_ownership_allocation_and_terminal_state_invariants() {
    let mut engine = AsterEngine::new();
    engine.process_command(limit_command(7, Side::Buy, 99, 40));
    let before_cancel = engine.snapshot();

    let wrong_participant = engine.process_command(cancel_command(1, 99));
    assert_eq!(
        wrong_participant,
        vec![EngineEvent::CancelRejected {
            order_id: OrderId::new(1),
            participant_id: ParticipantId::new(99),
            reason: AsterError::ParticipantMismatch,
        }]
    );
    assert_eq!(engine.snapshot(), before_cancel);

    let successful = engine.process_command(cancel_command(1, 7));
    assert_eq!(
        successful,
        vec![EngineEvent::OrderCancelled {
            order_id: OrderId::new(1),
            participant_id: ParticipantId::new(7),
        }]
    );
    let after_cancel = engine.snapshot();
    assert!(!resting_order_ids(&after_cancel).contains(&1));
    assert_eq!(after_cancel.next_order_id, before_cancel.next_order_id);
    assert_eq!(
        after_cancel.next_sequence_number,
        before_cancel.next_sequence_number
    );

    let repeated = engine.process_command(cancel_command(1, 7));
    assert_eq!(
        repeated,
        vec![EngineEvent::CancelRejected {
            order_id: OrderId::new(1),
            participant_id: ParticipantId::new(7),
            reason: AsterError::OrderNotFound,
        }]
    );
    assert_eq!(engine.snapshot(), after_cancel);

    let next_events = engine.process_command(limit_command(8, Side::Buy, 98, 10));
    let accepted = accepted_order(&next_events);
    assert_eq!(accepted.order_id.as_u64(), 2);
    assert_eq!(accepted.sequence_number.as_u64(), 2);
}

#[test]
fn complex_session_event_log_replay_and_quantity_accounting_are_consistent() {
    let commands = accounting_session();
    let mut engine = AsterEngine::new();
    let mut returned_events = Vec::new();

    for command in commands.iter().copied() {
        returned_events.extend(engine.process_command(command));
    }

    assert_eq!(engine.event_log(), returned_events.as_slice());

    let replay = replay_commands(commands);
    assert_eq!(replay.events, returned_events);
    assert_eq!(replay.final_snapshot, engine.snapshot());

    let accepted_quantity: u64 = returned_events
        .iter()
        .filter_map(|event| match event {
            EngineEvent::OrderAccepted { order } => Some(order.quantity.as_u64()),
            _ => None,
        })
        .sum();
    let trade_quantities: Vec<u64> = returned_events
        .iter()
        .filter_map(|event| match event {
            EngineEvent::TradeExecuted { quantity, .. } => Some(quantity.as_u64()),
            _ => None,
        })
        .collect();
    let traded_quantity: u64 = trade_quantities.iter().sum();
    let cancelled_remaining_quantity = 40;
    let expired_market_remainder = 10;
    let final_resting_quantity = engine.snapshot().total_resting_quantity;

    assert_eq!(accepted_quantity, 480);
    assert_eq!(trade_quantities, vec![100, 20, 40, 50]);
    assert_eq!(traded_quantity, 210);
    assert_eq!(final_resting_quantity, 10);
    assert_eq!(
        accepted_quantity,
        (2 * traded_quantity)
            + cancelled_remaining_quantity
            + expired_market_remainder
            + final_resting_quantity
    );

    let final_ids = resting_order_ids(&engine.snapshot());
    assert!(
        !final_ids.contains(&1),
        "fully filled order 1 must be absent"
    );
    assert!(
        !final_ids.contains(&2),
        "fully filled order 2 must be absent"
    );
    assert!(
        !final_ids.contains(&3),
        "fully filled order 3 must be absent"
    );
    assert!(!final_ids.contains(&4), "cancelled order 4 must be absent");
    assert_eq!(final_ids, HashSet::from([7]));
}

fn assert_snapshot_invariants(snapshot: &EngineSnapshot, command_index: usize) {
    let context = || format!("after command index {command_index}");

    if let (Some(best_bid), Some(best_ask)) = (snapshot.best_bid, snapshot.best_ask) {
        assert!(best_bid < best_ask, "book is crossed {}", context());
    }

    assert_eq!(
        snapshot.bid_level_count,
        snapshot.bid_levels.len(),
        "bid level count mismatch {}",
        context()
    );
    assert_eq!(
        snapshot.ask_level_count,
        snapshot.ask_levels.len(),
        "ask level count mismatch {}",
        context()
    );
    assert_eq!(
        snapshot.best_bid,
        snapshot.bid_levels.first().map(|level| level.price),
        "best bid mismatch {}",
        context()
    );
    assert_eq!(
        snapshot.best_ask,
        snapshot.ask_levels.first().map(|level| level.price),
        "best ask mismatch {}",
        context()
    );

    assert!(
        snapshot
            .bid_levels
            .windows(2)
            .all(|levels| levels[0].price > levels[1].price),
        "bid levels are not best-to-worst {}",
        context()
    );
    assert!(
        snapshot
            .ask_levels
            .windows(2)
            .all(|levels| levels[0].price < levels[1].price),
        "ask levels are not best-to-worst {}",
        context()
    );

    let mut order_ids = HashSet::new();
    for level in &snapshot.bid_levels {
        assert_level_invariants(level, Side::Buy, &mut order_ids, command_index);
    }
    for level in &snapshot.ask_levels {
        assert_level_invariants(level, Side::Sell, &mut order_ids, command_index);
    }

    let visible_quantity: u64 = snapshot
        .bid_levels
        .iter()
        .chain(&snapshot.ask_levels)
        .flat_map(|level| &level.orders)
        .map(|order| order.quantity.as_u64())
        .sum();
    assert_eq!(
        snapshot.total_resting_quantity,
        visible_quantity,
        "resting quantity summary mismatch {}",
        context()
    );
}

fn assert_level_invariants(
    level: &PriceLevelSnapshot,
    expected_side: Side,
    order_ids: &mut HashSet<u64>,
    command_index: usize,
) {
    assert!(
        !level.orders.is_empty(),
        "empty price level after command index {command_index}"
    );
    assert!(
        level
            .orders
            .windows(2)
            .all(|orders| orders[0].sequence_number < orders[1].sequence_number),
        "FIFO sequence order violated after command index {command_index}"
    );

    for order in &level.orders {
        assert_eq!(order.side, expected_side);
        assert!(order.quantity.as_u64() > 0);
        assert!(
            order_ids.insert(order.order_id.as_u64()),
            "duplicate resting order ID {} after command index {command_index}",
            order.order_id.as_u64()
        );
        match order.order_type {
            OrderType::Limit { price } => assert_eq!(price, level.price),
            OrderType::Market => {
                panic!("market order rests after command index {command_index}")
            }
        }
    }
}

fn accounting_session() -> Vec<EngineCommand> {
    vec![
        limit_command(1, Side::Sell, 101, 100),
        limit_command(2, Side::Sell, 101, 60),
        limit_command(3, Side::Sell, 103, 50),
        limit_command(4, Side::Buy, 99, 40),
        limit_command(5, Side::Buy, 102, 120),
        market_command(6, Side::Buy, 100),
        cancel_command(4, 99),
        cancel_command(4, 4),
        cancel_command(4, 4),
        limit_command(7, Side::Buy, 98, 10),
    ]
}

fn limit_command(participant_id: u64, side: Side, price_ticks: u64, units: u64) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(participant_id),
        side,
        OrderType::Limit {
            price: price(price_ticks),
        },
        quantity(units),
    ))
}

fn market_command(participant_id: u64, side: Side, units: u64) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(participant_id),
        side,
        OrderType::Market,
        quantity(units),
    ))
}

fn cancel_command(order_id: u64, participant_id: u64) -> EngineCommand {
    EngineCommand::cancel_order(OrderId::new(order_id), ParticipantId::new(participant_id))
}

fn accepted_order(events: &[EngineEvent]) -> &aster_core::AcceptedOrder {
    events
        .iter()
        .find_map(|event| match event {
            EngineEvent::OrderAccepted { order } => Some(order),
            _ => None,
        })
        .expect("submission should emit OrderAccepted")
}

fn resting_order_ids(snapshot: &EngineSnapshot) -> HashSet<u64> {
    snapshot
        .bid_levels
        .iter()
        .chain(&snapshot.ask_levels)
        .flat_map(|level| &level.orders)
        .map(|order| order.order_id.as_u64())
        .collect()
}

fn price(value: u64) -> PriceTicks {
    PriceTicks::new(value).expect("test price must be positive")
}

fn quantity(value: u64) -> Quantity {
    Quantity::new(value).expect("test quantity must be positive")
}
