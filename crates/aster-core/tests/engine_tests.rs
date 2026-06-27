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
    limit_request_for(1, side, price, quantity_value)
}

fn limit_request_for(
    participant_id: u64,
    side: Side,
    price: PriceTicks,
    quantity_value: u64,
) -> OrderRequest {
    OrderRequest::new(
        ParticipantId::new(participant_id),
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
fn cancelling_existing_resting_buy_order_emits_order_cancelled() {
    let mut engine = AsterEngine::new();
    let accepted = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));
    let order = accepted_order_from(&accepted);

    let cancel = engine.process_command(cancel(order.order_id, 1));

    assert_eq!(
        cancel,
        vec![EngineEvent::OrderCancelled {
            order_id: order.order_id,
            participant_id: ParticipantId::new(1),
        }]
    );
}

#[test]
fn cancelling_existing_resting_sell_order_emits_order_cancelled() {
    let mut engine = AsterEngine::new();
    let accepted = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Sell,
        price(101),
        10,
    )));
    let order = accepted_order_from(&accepted);

    let cancel = engine.process_command(cancel(order.order_id, 1));

    assert_eq!(
        cancel,
        vec![EngineEvent::OrderCancelled {
            order_id: order.order_id,
            participant_id: ParticipantId::new(1),
        }]
    );
}

#[test]
fn successful_cancellation_removes_order_from_book() {
    let mut engine = AsterEngine::new();
    let order = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Buy, price(100), 10),
    )));

    engine.process_command(cancel(order.order_id, 1));

    assert!(!engine.order_book().contains_order(order.order_id));
    assert!(engine.order_book().is_empty());
}

#[test]
fn cancelled_order_cannot_be_matched_later() {
    let mut engine = AsterEngine::new();
    let order = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(100), 10),
    )));
    engine.process_command(cancel(order.order_id, 1));

    let events = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));

    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], EngineEvent::OrderAccepted { .. }));
    assert_eq!(engine.order_book().best_bid_price(), Some(price(100)));
}

#[test]
fn cancelling_missing_order_emits_order_not_found() {
    let mut engine = AsterEngine::new();

    let events = engine.process_command(cancel(OrderId::new(404), 1));

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
fn cancelling_already_cancelled_order_emits_order_not_found() {
    let mut engine = AsterEngine::new();
    let order = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Buy, price(100), 10),
    )));
    engine.process_command(cancel(order.order_id, 1));

    let events = engine.process_command(cancel(order.order_id, 1));

    assert_eq!(
        events,
        vec![EngineEvent::CancelRejected {
            order_id: order.order_id,
            participant_id: ParticipantId::new(1),
            reason: AsterError::OrderNotFound,
        }]
    );
}

#[test]
fn cancelling_fully_filled_order_emits_order_not_found() {
    let mut engine = AsterEngine::new();
    let resting = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(100), 10),
    )));
    engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));

    let events = engine.process_command(cancel(resting.order_id, 1));

    assert_eq!(
        events,
        vec![EngineEvent::CancelRejected {
            order_id: resting.order_id,
            participant_id: ParticipantId::new(1),
            reason: AsterError::OrderNotFound,
        }]
    );
}

#[test]
fn cancelling_with_wrong_participant_rejects_and_leaves_order_in_book() {
    let mut engine = AsterEngine::new();
    let order = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request_for(7, Side::Buy, price(100), 10),
    )));

    let events = engine.process_command(cancel(order.order_id, 8));

    assert_eq!(
        events,
        vec![EngineEvent::CancelRejected {
            order_id: order.order_id,
            participant_id: ParticipantId::new(8),
            reason: AsterError::ParticipantMismatch,
        }]
    );
    assert!(engine.order_book().contains_order(order.order_id));
}

#[test]
fn cancelling_only_order_at_price_removes_empty_price_level() {
    let mut engine = AsterEngine::new();
    let order = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Buy, price(100), 10),
    )));

    engine.process_command(cancel(order.order_id, 1));

    assert_eq!(engine.order_book().bid_level_count(), 0);
    assert_eq!(engine.order_book().best_bid_price(), None);
}

#[test]
fn best_bid_and_best_ask_update_after_cancelling_best_orders() {
    let mut engine = AsterEngine::new();
    let lower_bid = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Buy, price(99), 10),
    )));
    let best_bid = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Buy, price(100), 10),
    )));
    let higher_ask = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(103), 10),
    )));
    let best_ask = accepted_order_from(&engine.process_command(EngineCommand::submit_order(
        limit_request(Side::Sell, price(102), 10),
    )));

    engine.process_command(cancel(best_bid.order_id, 1));
    engine.process_command(cancel(best_ask.order_id, 1));

    assert_eq!(engine.order_book().best_bid_price(), Some(price(99)));
    assert_eq!(engine.order_book().best_ask_price(), Some(price(103)));
    assert!(engine.order_book().contains_order(lower_bid.order_id));
    assert!(engine.order_book().contains_order(higher_ask.order_id));
}

#[test]
fn cancellation_does_not_consume_order_ids_or_sequences() {
    let mut engine = AsterEngine::new();
    engine.process_command(cancel(OrderId::new(999), 1));
    let accepted = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        10,
    )));
    let order = accepted_order_from(&accepted);

    assert_eq!(order.order_id.as_u64(), 1);
    assert_eq!(order.sequence_number.as_u64(), 1);
}

#[test]
fn market_orders_with_no_resting_remainder_cannot_be_cancelled() {
    let mut engine = AsterEngine::new();
    let events = engine.process_command(EngineCommand::submit_order(market_request(Side::Buy, 10)));
    let market = accepted_order_from(&events);

    let cancel_events = engine.process_command(cancel(market.order_id, 1));

    assert_eq!(
        cancel_events,
        vec![EngineEvent::CancelRejected {
            order_id: market.order_id,
            participant_id: ParticipantId::new(1),
            reason: AsterError::OrderNotFound,
        }]
    );
}

#[test]
fn resting_quantity_overflow_emits_order_rejected_without_consuming_allocation() {
    let mut engine = AsterEngine::new();
    let first = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        u64::MAX,
    )));
    assert_eq!(accepted_order_from(&first).order_id.as_u64(), 1);

    let rejected = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(100),
        1,
    )));
    assert_eq!(
        rejected,
        vec![EngineEvent::OrderRejected {
            reason: AsterError::QuantityOverflow,
        }]
    );

    engine.process_command(cancel(OrderId::new(1), 1));
    let next = engine.process_command(EngineCommand::submit_order(limit_request(
        Side::Buy,
        price(99),
        1,
    )));
    let accepted = accepted_order_from(&next);
    assert_eq!(accepted.order_id.as_u64(), 2);
    assert_eq!(accepted.sequence_number.as_u64(), 2);
}
