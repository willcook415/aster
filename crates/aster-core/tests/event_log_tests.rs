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

fn market_request(side: Side, quantity_value: u64) -> OrderRequest {
    OrderRequest::new(
        ParticipantId::new(1),
        side,
        OrderType::Market,
        quantity(quantity_value),
    )
}

fn cancel(order_id: OrderId, participant_id: u64) -> EngineCommand {
    EngineCommand::cancel_order(order_id, ParticipantId::new(participant_id))
}

fn accepted_order_from(events: &[EngineEvent]) -> aster_core::AcceptedOrder {
    match events.first() {
        Some(EngineEvent::OrderAccepted { order }) => *order,
        other => panic!("expected first event to be OrderAccepted, got {other:?}"),
    }
}

#[test]
fn new_engine_starts_with_empty_event_log() {
    let engine = AsterEngine::new();

    assert!(engine.event_log().is_empty());
}

#[test]
fn passive_limit_order_appends_returned_order_accepted_event() {
    let mut engine = AsterEngine::new();

    let events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));

    assert_eq!(engine.event_log(), events.as_slice());
    assert!(matches!(
        engine.event_log(),
        [EngineEvent::OrderAccepted { .. }]
    ));
}

#[test]
fn crossing_limit_order_appends_accept_and_trade_in_order() {
    let mut engine = AsterEngine::new();
    engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(100),
        10,
    )));
    let previous_len = engine.event_log().len();

    let events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));

    assert_eq!(&engine.event_log()[previous_len..], events.as_slice());
    assert!(matches!(
        events.as_slice(),
        [
            EngineEvent::OrderAccepted { .. },
            EngineEvent::TradeExecuted { .. },
        ]
    ));
}

#[test]
fn market_order_appends_returned_events_in_order() {
    let mut engine = AsterEngine::new();
    engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(100),
        10,
    )));
    let previous_len = engine.event_log().len();

    let events = engine.process_command(EngineCommand::submit_order(market_request(Side::Buy, 4)));

    assert_eq!(&engine.event_log()[previous_len..], events.as_slice());
    assert!(matches!(
        events.as_slice(),
        [
            EngineEvent::OrderAccepted { .. },
            EngineEvent::TradeExecuted { .. },
        ]
    ));
}

#[test]
fn successful_cancellation_appends_order_cancelled() {
    let mut engine = AsterEngine::new();
    let accepted = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));
    let order = accepted_order_from(&accepted);
    let previous_len = engine.event_log().len();

    let events = engine.process_command(cancel(order.order_id, 1));

    assert_eq!(&engine.event_log()[previous_len..], events.as_slice());
    assert_eq!(
        events,
        vec![EngineEvent::OrderCancelled {
            order_id: order.order_id,
            participant_id: ParticipantId::new(1),
        }]
    );
}

#[test]
fn rejected_cancellation_appends_cancel_rejected() {
    let mut engine = AsterEngine::new();

    let events = engine.process_command(cancel(OrderId::new(404), 1));

    assert_eq!(engine.event_log(), events.as_slice());
    assert_eq!(
        events,
        vec![EngineEvent::CancelRejected {
            order_id: OrderId::new(404),
            participant_id: ParticipantId::new(1),
            reason: AsterError::OrderNotFound,
        }]
    );
}

#[test]
fn multiple_process_command_calls_append_without_overwriting() {
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

    let mut expected = first;
    expected.extend(second);
    assert_eq!(engine.event_log(), expected.as_slice());
}

#[test]
fn process_commands_appends_all_emitted_events_in_order() {
    let mut engine = AsterEngine::new();
    let commands = vec![
        EngineCommand::submit_order(limit_request(Side::Sell, price(100), 5)),
        EngineCommand::submit_order(market_request(Side::Buy, 5)),
        cancel(OrderId::new(999), 1),
    ];

    let events = engine.process_commands(commands);

    assert_eq!(engine.event_log(), events.as_slice());
}

#[test]
fn returned_events_match_newly_appended_slice_for_each_command() {
    let mut engine = AsterEngine::new();
    let commands = vec![
        EngineCommand::submit_order(limit_request(Side::Sell, price(100), 5)),
        EngineCommand::submit_order(limit_request(Side::Buy, price(100), 3)),
        cancel(OrderId::new(999), 1),
    ];

    for command in commands {
        let previous_len = engine.event_log().len();
        let events = engine.process_command(command);
        assert_eq!(&engine.event_log()[previous_len..], events.as_slice());
    }
}

#[test]
fn clear_event_log_clears_only_log_not_book_or_allocation() {
    let mut engine = AsterEngine::new();
    let first = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));
    let first_order = accepted_order_from(&first);

    engine.clear_event_log();

    assert!(engine.event_log().is_empty());
    assert!(engine.order_book().contains_order(first_order.order_id));

    let second = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(101),
        10,
    )));
    let second_order = accepted_order_from(&second);

    assert_eq!(second_order.order_id.as_u64(), 2);
    assert_eq!(second_order.sequence_number.as_u64(), 2);
    assert_eq!(engine.event_log(), second.as_slice());
}
