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

fn accepted_order_from(events: &[EngineEvent]) -> aster_core::AcceptedOrder {
    match events {
        [EngineEvent::OrderAccepted { order }] => *order,
        other => panic!("expected one OrderAccepted event, got {other:?}"),
    }
}

fn accepted_order_from_prefix(events: &[EngineEvent]) -> aster_core::AcceptedOrder {
    match events.first() {
        Some(EngineEvent::OrderAccepted { order }) => *order,
        other => panic!("expected first event to be OrderAccepted, got {other:?}"),
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
fn crossing_buy_limit_order_emits_accept_then_trade() {
    let mut engine = AsterEngine::new();
    let resting = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(100), 10),
    )));

    let events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(105),
        10,
    )));
    let incoming = accepted_order_from_prefix(&events);

    assert_eq!(
        events,
        vec![
            EngineEvent::OrderAccepted { order: incoming },
            EngineEvent::TradeExecuted {
                resting_order_id: resting.order_id,
                incoming_order_id: incoming.order_id,
                price: price(100),
                quantity: quantity(10),
            },
        ]
    );
}

#[test]
fn crossing_sell_limit_order_emits_accept_then_trade() {
    let mut engine = AsterEngine::new();
    let resting = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Buy, price(100), 10),
    )));

    let events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(95),
        10,
    )));
    let incoming = accepted_order_from_prefix(&events);

    assert_eq!(
        events,
        vec![
            EngineEvent::OrderAccepted { order: incoming },
            EngineEvent::TradeExecuted {
                resting_order_id: resting.order_id,
                incoming_order_id: incoming.order_id,
                price: price(100),
                quantity: quantity(10),
            },
        ]
    );
}

#[test]
fn limit_orders_trade_at_resting_prices() {
    let mut engine = AsterEngine::new();
    engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(100),
        10,
    )));
    let buy_events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(105),
        5,
    )));
    engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));
    let sell_events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(95),
        5,
    )));

    assert!(matches!(
        buy_events.get(1),
        Some(EngineEvent::TradeExecuted { price: trade_price, .. }) if *trade_price == price(100)
    ));
    assert!(matches!(
        sell_events.get(1),
        Some(EngineEvent::TradeExecuted { price: trade_price, .. }) if *trade_price == price(100)
    ));
}

#[test]
fn full_fill_removes_resting_order_from_book() {
    let mut engine = AsterEngine::new();
    let resting = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(100), 10),
    )));

    engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));

    assert!(!engine.order_book().contains_order(resting.order_id));
    assert!(engine.order_book().is_empty());
}

#[test]
fn incoming_partial_fill_leaves_incoming_remainder_resting() {
    let mut engine = AsterEngine::new();
    engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(100),
        10,
    )));

    let events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(105),
        15,
    )));
    let incoming = accepted_order_from_prefix(&events);

    assert!(engine.order_book().contains_order(incoming.order_id));
    assert_eq!(engine.order_book().best_bid_price(), Some(price(105)));
    assert_eq!(
        engine
            .order_book()
            .best_bid_level()
            .and_then(|level| level.front())
            .map(|order| order.quantity.as_u64()),
        Some(5)
    );
}

#[test]
fn resting_partial_fill_reduces_resting_order_and_keeps_it_at_front() {
    let mut engine = AsterEngine::new();
    let resting = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(100), 10),
    )));

    engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        4,
    )));

    let front = engine
        .order_book()
        .best_ask_level()
        .and_then(|level| level.front())
        .expect("resting order remains");
    assert_eq!(front.order_id, resting.order_id);
    assert_eq!(front.sequence_number, resting.sequence_number);
    assert_eq!(front.quantity.as_u64(), 6);
}

#[test]
fn limit_orders_sweep_multiple_levels_in_price_priority() {
    let mut engine = AsterEngine::new();
    let high_ask = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(102), 5),
    )));
    let low_ask = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(100), 5),
    )));
    let buy_events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(102),
        10,
    )));

    let low_bid = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Buy, price(98), 5),
    )));
    let high_bid = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Buy, price(100), 5),
    )));
    let sell_events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(98),
        10,
    )));

    assert!(matches!(
        buy_events.as_slice(),
        [
            EngineEvent::OrderAccepted { .. },
            EngineEvent::TradeExecuted { resting_order_id: low_id, price: low_price, .. },
            EngineEvent::TradeExecuted { resting_order_id: high_id, price: high_price, .. },
        ] if *low_id == low_ask.order_id && *low_price == price(100)
            && *high_id == high_ask.order_id && *high_price == price(102)
    ));
    assert!(matches!(
        sell_events.as_slice(),
        [
            EngineEvent::OrderAccepted { .. },
            EngineEvent::TradeExecuted { resting_order_id: high_id, price: high_price, .. },
            EngineEvent::TradeExecuted { resting_order_id: low_id, price: low_price, .. },
        ] if *high_id == high_bid.order_id && *high_price == price(100)
            && *low_id == low_bid.order_id && *low_price == price(98)
    ));
}

#[test]
fn fifo_is_preserved_within_same_price_level() {
    let mut engine = AsterEngine::new();
    let first = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(100), 5),
    )));
    let second = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(100), 5),
    )));

    let events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));

    assert!(matches!(
        events.as_slice(),
        [
            EngineEvent::OrderAccepted { .. },
            EngineEvent::TradeExecuted { resting_order_id: first_id, .. },
            EngineEvent::TradeExecuted { resting_order_id: second_id, .. },
        ] if *first_id == first.order_id && *second_id == second.order_id
    ));
}

#[test]
fn incoming_order_stops_matching_when_limit_no_longer_crosses() {
    let mut engine = AsterEngine::new();
    engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(100),
        5,
    )));
    engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(103),
        5,
    )));

    let events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(101),
        10,
    )));
    let incoming = accepted_order_from_prefix(&events);

    assert_eq!(events.len(), 2);
    assert_eq!(engine.order_book().best_ask_price(), Some(price(103)));
    assert!(engine.order_book().contains_order(incoming.order_id));
    assert_eq!(engine.order_book().best_bid_price(), Some(price(101)));
}

#[test]
fn market_buy_emits_accept_then_trade_when_asks_exist() {
    let mut engine = AsterEngine::new();
    let resting = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(100), 10),
    )));

    let events = engine.process_command(EngineCommand::submit_order(market_request(Side::Buy, 10)));
    let incoming = accepted_order_from_prefix(&events);

    assert_eq!(
        events,
        vec![
            EngineEvent::OrderAccepted { order: incoming },
            EngineEvent::TradeExecuted {
                resting_order_id: resting.order_id,
                incoming_order_id: incoming.order_id,
                price: price(100),
                quantity: quantity(10),
            },
        ]
    );
}

#[test]
fn market_sell_emits_accept_then_trade_when_bids_exist() {
    let mut engine = AsterEngine::new();
    let resting = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Buy, price(100), 10),
    )));

    let events =
        engine.process_command(EngineCommand::submit_order(market_request(Side::Sell, 10)));
    let incoming = accepted_order_from_prefix(&events);

    assert_eq!(
        events,
        vec![
            EngineEvent::OrderAccepted { order: incoming },
            EngineEvent::TradeExecuted {
                resting_order_id: resting.order_id,
                incoming_order_id: incoming.order_id,
                price: price(100),
                quantity: quantity(10),
            },
        ]
    );
}

#[test]
fn market_orders_trade_at_resting_prices() {
    let mut engine = AsterEngine::new();
    engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(100),
        10,
    )));
    let buy_events =
        engine.process_command(EngineCommand::submit_order(market_request(Side::Buy, 5)));
    engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));
    let sell_events =
        engine.process_command(EngineCommand::submit_order(market_request(Side::Sell, 5)));

    assert!(matches!(
        buy_events.get(1),
        Some(EngineEvent::TradeExecuted { price: trade_price, .. }) if *trade_price == price(100)
    ));
    assert!(matches!(
        sell_events.get(1),
        Some(EngineEvent::TradeExecuted { price: trade_price, .. }) if *trade_price == price(100)
    ));
}

#[test]
fn market_orders_sweep_multiple_levels_in_price_priority() {
    let mut engine = AsterEngine::new();
    let high_ask = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(102), 5),
    )));
    let low_ask = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(100), 5),
    )));
    let buy_events =
        engine.process_command(EngineCommand::submit_order(market_request(Side::Buy, 10)));

    let low_bid = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Buy, price(98), 5),
    )));
    let high_bid = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Buy, price(100), 5),
    )));
    let sell_events =
        engine.process_command(EngineCommand::submit_order(market_request(Side::Sell, 10)));

    assert!(matches!(
        buy_events.as_slice(),
        [
            EngineEvent::OrderAccepted { .. },
            EngineEvent::TradeExecuted { resting_order_id: low_id, price: low_price, .. },
            EngineEvent::TradeExecuted { resting_order_id: high_id, price: high_price, .. },
        ] if *low_id == low_ask.order_id && *low_price == price(100)
            && *high_id == high_ask.order_id && *high_price == price(102)
    ));
    assert!(matches!(
        sell_events.as_slice(),
        [
            EngineEvent::OrderAccepted { .. },
            EngineEvent::TradeExecuted { resting_order_id: high_id, price: high_price, .. },
            EngineEvent::TradeExecuted { resting_order_id: low_id, price: low_price, .. },
        ] if *high_id == high_bid.order_id && *high_price == price(100)
            && *low_id == low_bid.order_id && *low_price == price(98)
    ));
}

#[test]
fn market_order_preserves_fifo_within_same_price_level() {
    let mut engine = AsterEngine::new();
    let first = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(100), 5),
    )));
    let second = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(100), 5),
    )));

    let events = engine.process_command(EngineCommand::submit_order(market_request(Side::Buy, 10)));

    assert!(matches!(
        events.as_slice(),
        [
            EngineEvent::OrderAccepted { .. },
            EngineEvent::TradeExecuted { resting_order_id: first_id, .. },
            EngineEvent::TradeExecuted { resting_order_id: second_id, .. },
        ] if *first_id == first.order_id && *second_id == second.order_id
    ));
}

#[test]
fn market_full_fill_removes_resting_orders() {
    let mut engine = AsterEngine::new();
    let resting = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(100), 10),
    )));

    engine.process_command(EngineCommand::submit_order(market_request(Side::Buy, 10)));

    assert!(!engine.order_book().contains_order(resting.order_id));
    assert!(engine.order_book().is_empty());
}

#[test]
fn market_resting_partial_fill_reduces_resting_order_and_keeps_it_at_front() {
    let mut engine = AsterEngine::new();
    let resting = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(100), 10),
    )));

    engine.process_command(EngineCommand::submit_order(market_request(Side::Buy, 4)));

    let front = engine
        .order_book()
        .best_ask_level()
        .and_then(|level| level.front())
        .expect("resting order remains");
    assert_eq!(front.order_id, resting.order_id);
    assert_eq!(front.sequence_number, resting.sequence_number);
    assert_eq!(front.quantity.as_u64(), 6);
}

#[test]
fn oversized_market_order_consumes_available_liquidity_and_expires_remainder() {
    let mut engine = AsterEngine::new();
    engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(100),
        5,
    )));

    let events = engine.process_command(EngineCommand::submit_order(market_request(Side::Buy, 10)));

    assert_eq!(events.len(), 2);
    assert!(engine.order_book().is_empty());
}

#[test]
fn market_order_with_no_opposing_liquidity_emits_only_order_accepted() {
    let mut engine = AsterEngine::new();
    let events = engine.process_command(EngineCommand::submit_order(market_request(Side::Buy, 10)));
    let incoming = accepted_order_from_prefix(&events);

    assert_eq!(events, vec![EngineEvent::OrderAccepted { order: incoming }]);
}

#[test]
fn market_orders_never_rest_in_the_book() {
    let mut engine = AsterEngine::new();
    let events = engine.process_command(EngineCommand::submit_order(market_request(Side::Buy, 10)));
    let incoming = accepted_order_from_prefix(&events);

    assert!(!engine.order_book().contains_order(incoming.order_id));
    assert!(engine.order_book().is_empty());
}

#[test]
fn accepted_market_orders_consume_deterministic_ids_and_sequences() {
    let mut engine = AsterEngine::new();
    engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(100),
        5,
    )));

    let events = engine.process_command(EngineCommand::submit_order(market_request(Side::Buy, 5)));
    let incoming = accepted_order_from_prefix(&events);

    assert_eq!(incoming.order_id.as_u64(), 2);
    assert_eq!(incoming.sequence_number.as_u64(), 2);
}

#[test]
fn cancellation_remains_rejected_and_does_not_consume_ids_or_sequences() {
    let mut engine = AsterEngine::new();
    let cancel = engine.process_command(EngineCommand::cancel_order(
        OrderId::new(999),
        ParticipantId::new(1),
    ));
    let accepted = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));
    let order = accepted_order_from(&accepted);

    assert_eq!(
        cancel,
        vec![EngineEvent::CancelRejected {
            order_id: OrderId::new(999),
            participant_id: ParticipantId::new(1),
            reason: AsterError::CancellationNotImplemented,
        }]
    );
    assert_eq!(order.order_id.as_u64(), 1);
    assert_eq!(order.sequence_number.as_u64(), 1);
    assert!(engine.order_book().contains_order(order.order_id));
}
