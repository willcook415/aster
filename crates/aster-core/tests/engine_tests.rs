use aster_core::{
    AsterEngine, AsterError, EngineCommand, EngineEvent, OrderId, OrderRequest, OrderType,
    ParticipantId, PriceTicks, Quantity, Side,
};

fn price(value: u64) -> PriceTicks {
    PriceTicks::new(value).expect("valid price")
}

fn quantity(value: u64) -> Quantity {
    Quantity::new(value).expect("valid quantity")
}

fn limit_request(side: Side, price: PriceTicks, quantity_value: u64) -> OrderRequest {
    OrderRequest::new(
        ParticipantId::new(1),
        side,
        OrderType::Limit { price },
        quantity(quantity_value),
    )
}

fn market_request(side: Side) -> OrderRequest {
    OrderRequest::new(ParticipantId::new(1), side, OrderType::Market, quantity(10))
}

fn accepted_order_from(events: &[EngineEvent]) -> aster_core::AcceptedOrder {
    match events {
        [EngineEvent::OrderAccepted { order }] => *order,
        other => panic!("expected one OrderAccepted event, got {other:?}"),
    }
}

fn rejection_reason_from(events: &[EngineEvent]) -> AsterError {
    match events {
        [EngineEvent::OrderRejected { reason }] => *reason,
        other => panic!("expected one OrderRejected event, got {other:?}"),
    }
}

#[test]
fn new_engine_starts_with_empty_book() {
    let engine = AsterEngine::new();

    assert!(engine.order_book().is_empty());
}

#[test]
fn submitting_non_crossing_buy_limit_order_emits_order_accepted() {
    let mut engine = AsterEngine::new();
    let events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));

    let order = accepted_order_from(&events);
    assert_eq!(order.side, Side::Buy);
    assert_eq!(order.order_type, OrderType::Limit { price: price(100) });
}

#[test]
fn submitting_non_crossing_sell_limit_order_emits_order_accepted() {
    let mut engine = AsterEngine::new();
    let events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(101),
        10,
    )));

    let order = accepted_order_from(&events);
    assert_eq!(order.side, Side::Sell);
    assert_eq!(order.order_type, OrderType::Limit { price: price(101) });
}

#[test]
fn accepted_orders_receive_deterministic_ids_starting_at_one() {
    let mut engine = AsterEngine::new();
    let events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));

    assert_eq!(accepted_order_from(&events).order_id.as_u64(), 1);
}

#[test]
fn accepted_orders_receive_deterministic_sequence_numbers_starting_at_one() {
    let mut engine = AsterEngine::new();
    let events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(101),
        10,
    )));

    assert_eq!(accepted_order_from(&events).sequence_number.as_u64(), 1);
}

#[test]
fn multiple_accepted_orders_increment_ids_and_sequences_deterministically() {
    let mut engine = AsterEngine::new();
    let first = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));
    let second = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(101),
        10,
    )));

    let first = accepted_order_from(&first);
    let second = accepted_order_from(&second);

    assert_eq!(first.order_id.as_u64(), 1);
    assert_eq!(first.sequence_number.as_u64(), 1);
    assert_eq!(second.order_id.as_u64(), 2);
    assert_eq!(second.sequence_number.as_u64(), 2);
}

#[test]
fn accepted_non_crossing_limit_orders_are_added_to_internal_book() {
    let mut engine = AsterEngine::new();
    let events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));
    let order = accepted_order_from(&events);

    assert!(engine.order_book().contains_order(order.order_id));
    assert_eq!(engine.order_book().best_bid_price(), Some(price(100)));
    assert_eq!(engine.order_book().total_resting_quantity(), 10);
}

#[test]
fn market_orders_are_rejected_and_do_not_change_book() {
    let mut engine = AsterEngine::new();
    let events = engine.process_command(EngineCommand::submit_order(market_request(Side::Buy)));

    assert_eq!(
        rejection_reason_from(&events),
        AsterError::MarketOrderRequiresMatching
    );
    assert!(engine.order_book().is_empty());
}

#[test]
fn crossing_buy_limit_orders_are_rejected_until_matching_exists() {
    let mut engine = AsterEngine::new();
    engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(100),
        10,
    )));

    let events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));

    assert_eq!(
        rejection_reason_from(&events),
        AsterError::CrossingOrderRequiresMatching
    );
    assert_eq!(engine.order_book().total_resting_quantity(), 10);
}

#[test]
fn crossing_sell_limit_orders_are_rejected_until_matching_exists() {
    let mut engine = AsterEngine::new();
    engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));

    let events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(100),
        10,
    )));

    assert_eq!(
        rejection_reason_from(&events),
        AsterError::CrossingOrderRequiresMatching
    );
    assert_eq!(engine.order_book().total_resting_quantity(), 10);
}

#[test]
fn rejected_orders_do_not_consume_ids_or_sequences() {
    let mut engine = AsterEngine::new();

    engine.process_command(EngineCommand::submit_order(market_request(Side::Buy)));
    let events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));
    let order = accepted_order_from(&events);

    assert_eq!(order.order_id.as_u64(), 1);
    assert_eq!(order.sequence_number.as_u64(), 1);
}

#[test]
fn cancel_commands_emit_cancel_rejected_and_do_not_change_book() {
    let mut engine = AsterEngine::new();
    let accepted = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));
    let order = accepted_order_from(&accepted);

    let cancel = engine.process_command(EngineCommand::cancel_order(
        OrderId::new(999),
        ParticipantId::new(1),
    ));

    assert_eq!(
        cancel,
        vec![EngineEvent::CancelRejected {
            order_id: OrderId::new(999),
            participant_id: ParticipantId::new(1),
            reason: AsterError::CancellationNotImplemented,
        }]
    );
    assert!(engine.order_book().contains_order(order.order_id));
    assert_eq!(engine.order_book().total_resting_quantity(), 10);
}
