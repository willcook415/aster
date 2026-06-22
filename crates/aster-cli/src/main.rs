use aster_core::{
    AcceptedOrder, AsterEngine, AsterError, EngineCommand, EngineEvent, EngineSnapshot, OrderId,
    OrderRequest, OrderType, ParticipantId, PriceTicks, Quantity, Side,
};

const PASSIVE_SELLER: ParticipantId = ParticipantId::new(1);
const PASSIVE_BUYER: ParticipantId = ParticipantId::new(2);
const AGGRESSIVE_TRADER: ParticipantId = ParticipantId::new(3);
const CANCELLER: ParticipantId = ParticipantId::new(4);

fn main() -> Result<(), AsterError> {
    println!("Aster - deterministic Rust matching engine demo");
    println!();
    println!("Participants:");
    println!("  1: passive seller");
    println!("  2: passive buyer");
    println!("  3: aggressive trader");
    println!("  4: canceller");
    println!();

    let mut engine = AsterEngine::new();

    println!("1. Resting passive liquidity");
    submit_and_print(
        &mut engine,
        "passive seller: sell 100 @ 101",
        limit_order(PASSIVE_SELLER, Side::Sell, 101, 100)?,
    );
    let second_ask = submit_and_print(
        &mut engine,
        "passive seller: sell 150 @ 102",
        limit_order(PASSIVE_SELLER, Side::Sell, 102, 150)?,
    );
    submit_and_print(
        &mut engine,
        "passive buyer: buy 80 @ 99",
        limit_order(PASSIVE_BUYER, Side::Buy, 99, 80)?,
    );
    let cancellable_bid = submit_and_print(
        &mut engine,
        "canceller: buy 120 @ 98",
        limit_order(CANCELLER, Side::Buy, 98, 120)?,
    );

    let cancelled_order_id = accepted_order_id(&cancellable_bid)?;
    let rejected_cancel_order_id = accepted_order_id(&second_ask)?;
    println!();

    println!("2. Crossing limit order");
    submit_and_print(
        &mut engine,
        "aggressive trader: buy 180 @ 102",
        limit_order(AGGRESSIVE_TRADER, Side::Buy, 102, 180)?,
    );
    println!();

    println!("3. Market sweep");
    submit_and_print(
        &mut engine,
        "aggressive trader: market sell 90",
        market_order(AGGRESSIVE_TRADER, Side::Sell, 90)?,
    );
    println!();

    println!("4. Cancellation");
    submit_and_print(
        &mut engine,
        "canceller: cancel remaining resting bid",
        EngineCommand::cancel_order(cancelled_order_id, CANCELLER),
    );
    println!();

    println!("5. Rejected cancellation");
    submit_and_print(
        &mut engine,
        "canceller: cancel another participant's order",
        EngineCommand::cancel_order(rejected_cancel_order_id, CANCELLER),
    );
    println!();

    println!("6. Final snapshot");
    print_snapshot(&engine.snapshot());
    println!();

    println!("7. Event log summary");
    println!("  total events retained: {}", engine.event_log().len());
    println!();

    println!("Next steps:");
    println!("  cargo test --workspace");
    println!("  cargo clippy --workspace --all-targets -- -D warnings");
    println!("  cargo bench -p aster-core");

    Ok(())
}

fn submit_and_print(
    engine: &mut AsterEngine,
    label: &str,
    command: EngineCommand,
) -> Vec<EngineEvent> {
    println!("  Command: {label}");
    let events = engine.process_command(command);
    print_events(&events);
    events
}

fn print_events(events: &[EngineEvent]) {
    for event in events {
        match event {
            EngineEvent::OrderAccepted { order } => print_accepted(order),
            EngineEvent::OrderRejected { reason } => {
                println!("    OrderRejected: {reason}");
            }
            EngineEvent::OrderCancelled {
                order_id,
                participant_id,
            } => {
                println!(
                    "    OrderCancelled: order_id={}, participant_id={}",
                    order_id.as_u64(),
                    participant_id.as_u64()
                );
            }
            EngineEvent::CancelRejected {
                order_id,
                participant_id,
                reason,
            } => {
                println!(
                    "    CancelRejected: order_id={}, participant_id={}, reason={}",
                    order_id.as_u64(),
                    participant_id.as_u64(),
                    reason
                );
            }
            EngineEvent::TradeExecuted {
                resting_order_id,
                incoming_order_id,
                price,
                quantity,
            } => {
                println!(
                    "    TradeExecuted: resting_order_id={}, incoming_order_id={}, price={}, quantity={}",
                    resting_order_id.as_u64(),
                    incoming_order_id.as_u64(),
                    price.as_u64(),
                    quantity.as_u64()
                );
            }
        }
    }
}

fn print_accepted(order: &AcceptedOrder) {
    println!(
        "    OrderAccepted: order_id={}, participant_id={}, side={}, type={}, quantity={}, sequence={}",
        order.order_id.as_u64(),
        order.participant_id.as_u64(),
        side_name(order.side),
        order_type_name(order.order_type),
        order.quantity.as_u64(),
        order.sequence_number.as_u64()
    );
}

fn print_snapshot(snapshot: &EngineSnapshot) {
    println!("  best_bid: {}", price_or_empty(snapshot.best_bid));
    println!("  best_ask: {}", price_or_empty(snapshot.best_ask));
    println!("  bid_level_count: {}", snapshot.bid_level_count);
    println!("  ask_level_count: {}", snapshot.ask_level_count);
    println!(
        "  total_resting_quantity: {}",
        snapshot.total_resting_quantity
    );
}

fn limit_order(
    participant_id: ParticipantId,
    side: Side,
    price: u64,
    quantity: u64,
) -> Result<EngineCommand, AsterError> {
    Ok(EngineCommand::submit_order(OrderRequest::new(
        participant_id,
        side,
        OrderType::Limit {
            price: PriceTicks::new(price)?,
        },
        Quantity::new(quantity)?,
    )))
}

fn market_order(
    participant_id: ParticipantId,
    side: Side,
    quantity: u64,
) -> Result<EngineCommand, AsterError> {
    Ok(EngineCommand::submit_order(OrderRequest::new(
        participant_id,
        side,
        OrderType::Market,
        Quantity::new(quantity)?,
    )))
}

fn accepted_order_id(events: &[EngineEvent]) -> Result<OrderId, AsterError> {
    events
        .iter()
        .find_map(|event| match event {
            EngineEvent::OrderAccepted { order } => Some(order.order_id),
            _ => None,
        })
        .ok_or(AsterError::InvalidOrderState)
}

fn side_name(side: Side) -> &'static str {
    match side {
        Side::Buy => "buy",
        Side::Sell => "sell",
    }
}

fn order_type_name(order_type: OrderType) -> String {
    match order_type {
        OrderType::Limit { price } => format!("limit @ {}", price.as_u64()),
        OrderType::Market => "market".to_string(),
    }
}

fn price_or_empty(price: Option<PriceTicks>) -> String {
    match price {
        Some(price) => price.as_u64().to_string(),
        None => "none".to_string(),
    }
}
