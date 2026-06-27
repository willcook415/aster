use aster_core::{
    AcceptedOrder, EngineCommand, EngineEvent, EngineSnapshot, OrderType, PriceLevelSnapshot,
    SessionRecord, SessionVerificationError, Side,
};

use crate::scenario::Scenario;

pub fn render_scenario(
    scenario: &Scenario,
    session: &SessionRecord,
    verification: &Result<(), SessionVerificationError>,
) -> String {
    let mut output = String::new();
    output.push_str("Aster deterministic scenario runner\n");
    output.push_str(&format!("Scenario: {}\n", scenario.name));
    output.push_str(&format!("Description: {}\n\n", scenario.description));

    output.push_str("Commands\n");
    for (index, command) in session.commands.iter().enumerate() {
        output.push_str(&format!(
            "  {:>2}. {}\n",
            index + 1,
            format_command(command)
        ));
    }

    output.push_str("\nEvents\n");
    for (index, event) in session.events.iter().enumerate() {
        output.push_str(&format!("  {:>2}. {}\n", index + 1, format_event(event)));
    }

    output.push_str("\nFinal Book\n");
    render_snapshot(&mut output, &session.final_snapshot);

    output.push_str("\nVerification\n");
    render_verification(&mut output, verification);
    output
}

fn format_command(command: &EngineCommand) -> String {
    match command {
        EngineCommand::SubmitOrder(request) => format!(
            "submit participant={} side={} type={} quantity={}",
            request.participant_id.as_u64(),
            side_name(request.side),
            order_type_name(request.order_type),
            request.quantity.as_u64()
        ),
        EngineCommand::CancelOrder {
            order_id,
            participant_id,
        } => format!(
            "cancel order_id={} participant={}",
            order_id.as_u64(),
            participant_id.as_u64()
        ),
    }
}

fn format_event(event: &EngineEvent) -> String {
    match event {
        EngineEvent::OrderAccepted { order } => format!(
            "OrderAccepted id={} participant={} side={} type={} quantity={} sequence={}",
            order.order_id.as_u64(),
            order.participant_id.as_u64(),
            side_name(order.side),
            order_type_name(order.order_type),
            order.quantity.as_u64(),
            order.sequence_number.as_u64()
        ),
        EngineEvent::OrderRejected { reason } => format!("OrderRejected reason={reason}"),
        EngineEvent::OrderCancelled {
            order_id,
            participant_id,
        } => format!(
            "OrderCancelled id={} participant={}",
            order_id.as_u64(),
            participant_id.as_u64()
        ),
        EngineEvent::CancelRejected {
            order_id,
            participant_id,
            reason,
        } => format!(
            "CancelRejected id={} participant={} reason={}",
            order_id.as_u64(),
            participant_id.as_u64(),
            reason
        ),
        EngineEvent::TradeExecuted {
            resting_order_id,
            incoming_order_id,
            price,
            quantity,
        } => format!(
            "TradeExecuted resting_id={} incoming_id={} price={} quantity={}",
            resting_order_id.as_u64(),
            incoming_order_id.as_u64(),
            price.as_u64(),
            quantity.as_u64()
        ),
    }
}

fn render_snapshot(output: &mut String, snapshot: &EngineSnapshot) {
    output.push_str(&format!(
        "  summary: best_bid={} best_ask={} resting_quantity={}\n",
        optional_price(snapshot.best_bid),
        optional_price(snapshot.best_ask),
        snapshot.total_resting_quantity
    ));
    render_side(output, "Bids (best to worst)", &snapshot.bid_levels);
    render_side(output, "Asks (best to worst)", &snapshot.ask_levels);
    output.push_str(&format!(
        "  next allocation: order_id={} sequence={}\n",
        snapshot.next_order_id.as_u64(),
        snapshot.next_sequence_number.as_u64()
    ));
}

fn render_side(output: &mut String, heading: &str, levels: &[PriceLevelSnapshot]) {
    output.push_str(&format!("  {heading}\n"));
    if levels.is_empty() {
        output.push_str("    (empty)\n");
        return;
    }

    for level in levels {
        let total: u64 = level
            .orders
            .iter()
            .map(|order| order.quantity.as_u64())
            .sum();
        output.push_str(&format!(
            "    price={} total_quantity={}\n",
            level.price.as_u64(),
            total
        ));
        for order in &level.orders {
            render_resting_order(output, order);
        }
    }
}

fn render_resting_order(output: &mut String, order: &AcceptedOrder) {
    output.push_str(&format!(
        "      order_id={} participant={} side={} remaining={} sequence={}\n",
        order.order_id.as_u64(),
        order.participant_id.as_u64(),
        side_name(order.side),
        order.quantity.as_u64(),
        order.sequence_number.as_u64()
    ));
}

fn render_verification(output: &mut String, verification: &Result<(), SessionVerificationError>) {
    match verification {
        Ok(()) => {
            output.push_str("  Event replay matched: YES\n");
            output.push_str("  Final snapshot replay matched: YES\n");
            output.push_str("  Session verified: YES\n");
        }
        Err(SessionVerificationError::EventMismatch { index, .. }) => {
            output.push_str(&format!(
                "  Event replay matched: NO (first difference at index {index})\n"
            ));
            output.push_str("  Final snapshot replay matched: NOT CHECKED\n");
            output.push_str("  Session verified: NO\n");
        }
        Err(SessionVerificationError::SnapshotMismatch { .. }) => {
            output.push_str("  Event replay matched: YES\n");
            output.push_str("  Final snapshot replay matched: NO\n");
            output.push_str("  Session verified: NO\n");
        }
    }
}

fn side_name(side: Side) -> &'static str {
    match side {
        Side::Buy => "buy",
        Side::Sell => "sell",
    }
}

fn order_type_name(order_type: OrderType) -> String {
    match order_type {
        OrderType::Limit { price } => format!("limit@{}", price.as_u64()),
        OrderType::Market => "market".to_string(),
    }
}

fn optional_price(price: Option<aster_core::PriceTicks>) -> String {
    price
        .map(|price| price.as_u64().to_string())
        .unwrap_or_else(|| "none".to_string())
}
