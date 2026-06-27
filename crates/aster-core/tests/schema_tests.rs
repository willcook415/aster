use aster_core::{
    replay_commands, AcceptedOrder, AsterError, CommandDtoV1, CommandRecordV1, EngineCommand,
    EngineEvent, EngineSnapshot, EventRecordV1, OrderId, OrderRequest, OrderType, OrderTypeDtoV1,
    ParticipantId, PriceTicks, Quantity, SequenceNumber, Side, SideDtoV1, SnapshotRecordV1,
    ASTER_SCHEMA_VERSION,
};
use serde_json::Value;

#[test]
fn submit_limit_command_round_trips_through_json() {
    let command = limit_command(11, Side::Buy, 101, 25);

    let round_tripped = round_trip_command(command);

    assert_eq!(round_tripped, command);
}

#[test]
fn submit_market_command_round_trips_through_json() {
    let command = market_command(12, Side::Sell, 30);

    let round_tripped = round_trip_command(command);

    assert_eq!(round_tripped, command);
}

#[test]
fn cancel_command_round_trips_through_json() {
    let command = EngineCommand::cancel_order(OrderId::new(99), ParticipantId::new(12));

    let round_tripped = round_trip_command(command);

    assert_eq!(round_tripped, command);
}

#[test]
fn order_accepted_event_round_trips_through_json() {
    let event = EngineEvent::OrderAccepted {
        order: accepted_order(
            1,
            10,
            Side::Sell,
            OrderType::Limit { price: price(105) },
            40,
            7,
        ),
    };

    assert_eq!(round_trip_event(event), event);
}

#[test]
fn order_rejected_event_round_trips_through_json() {
    let event = EngineEvent::OrderRejected {
        reason: AsterError::ZeroQuantity,
    };

    assert_eq!(round_trip_event(event), event);
}

#[test]
fn order_cancelled_event_round_trips_through_json() {
    let event = EngineEvent::OrderCancelled {
        order_id: OrderId::new(5),
        participant_id: ParticipantId::new(2),
    };

    assert_eq!(round_trip_event(event), event);
}

#[test]
fn cancel_rejected_event_round_trips_through_json() {
    let event = EngineEvent::CancelRejected {
        order_id: OrderId::new(5),
        participant_id: ParticipantId::new(3),
        reason: AsterError::ParticipantMismatch,
    };

    assert_eq!(round_trip_event(event), event);
}

#[test]
fn trade_executed_event_round_trips_through_json() {
    let event = EngineEvent::TradeExecuted {
        resting_order_id: OrderId::new(1),
        incoming_order_id: OrderId::new(2),
        price: price(101),
        quantity: quantity(20),
    };

    assert_eq!(round_trip_event(event), event);
}

#[test]
fn engine_snapshot_round_trips_through_json() {
    let snapshot = replay_commands(vec![
        limit_command(10, Side::Buy, 99, 20),
        limit_command(11, Side::Buy, 99, 30),
        limit_command(12, Side::Sell, 101, 40),
        market_command(13, Side::Buy, 15),
    ])
    .final_snapshot;

    let record = SnapshotRecordV1::from(snapshot.clone());
    let json = serde_json::to_string(&record).expect("snapshot record should serialize");
    let value: Value = serde_json::from_str(&json).expect("snapshot record should be JSON");
    let decoded: SnapshotRecordV1 =
        serde_json::from_str(&json).expect("snapshot record should deserialize");
    let round_tripped = EngineSnapshot::try_from(decoded).expect("snapshot record should convert");

    assert_eq!(round_tripped, snapshot);
    assert_eq!(round_tripped.bid_levels[0].orders.len(), 2);
    assert_eq!(round_tripped.ask_levels[0].orders[0].quantity.as_u64(), 25);
    assert_eq!(round_tripped.ask_levels[0].orders[0].order_id.as_u64(), 3);
    assert_eq!(
        round_tripped.ask_levels[0].orders[0]
            .sequence_number
            .as_u64(),
        3
    );
    assert_eq!(round_tripped.next_order_id.as_u64(), 5);
    assert!(value["snapshot"]["bid_levels"].is_array());
    assert!(value["snapshot"]["ask_levels"][0]["orders"].is_array());
}

#[test]
fn unsupported_schema_version_is_rejected() {
    let mut record = CommandRecordV1::from(limit_command(1, Side::Buy, 100, 10));
    record.schema_version = ASTER_SCHEMA_VERSION + 1;

    let result = EngineCommand::try_from(record);

    assert_eq!(result, Err(AsterError::UnsupportedSchemaVersion));
}

#[test]
fn zero_quantity_in_command_record_is_rejected() {
    let record = CommandRecordV1 {
        schema_version: ASTER_SCHEMA_VERSION,
        command: CommandDtoV1::SubmitOrder {
            participant_id: 1,
            side: SideDtoV1::Buy,
            order_type: OrderTypeDtoV1::Market,
            quantity: 0,
        },
    };

    let result = EngineCommand::try_from(record);

    assert_eq!(result, Err(AsterError::ZeroQuantity));
}

#[test]
fn zero_limit_price_in_command_record_is_rejected() {
    let record = CommandRecordV1 {
        schema_version: ASTER_SCHEMA_VERSION,
        command: CommandDtoV1::SubmitOrder {
            participant_id: 1,
            side: SideDtoV1::Buy,
            order_type: OrderTypeDtoV1::Limit { price: 0 },
            quantity: 10,
        },
    };

    let result = EngineCommand::try_from(record);

    assert_eq!(result, Err(AsterError::ZeroPrice));
}

#[test]
fn integer_ticks_remain_json_integers() {
    let record = CommandRecordV1::from(limit_command(1, Side::Sell, 123, 45));
    let json = serde_json::to_string(&record).expect("command record should serialize");
    let value: Value = serde_json::from_str(&json).expect("command record should be JSON");

    let price = &value["command"]["order_type"]["price"];
    let quantity = &value["command"]["quantity"];

    assert!(price.is_u64());
    assert!(quantity.is_u64());
    assert!(!price.is_f64());
}

#[test]
fn command_records_replay_to_same_result_in_memory() {
    let commands = vec![
        limit_command(1, Side::Sell, 101, 100),
        limit_command(2, Side::Buy, 99, 80),
        limit_command(3, Side::Buy, 101, 60),
        market_command(4, Side::Sell, 40),
        EngineCommand::cancel_order(OrderId::new(2), ParticipantId::new(2)),
    ];
    let original = replay_commands(commands.clone());

    let json_lines: Vec<String> = commands
        .into_iter()
        .map(CommandRecordV1::from)
        .map(|record| serde_json::to_string(&record).expect("command record should serialize"))
        .collect();
    let decoded_commands: Vec<EngineCommand> = json_lines
        .into_iter()
        .map(|json| serde_json::from_str::<CommandRecordV1>(&json).expect("valid command JSON"))
        .map(|record| EngineCommand::try_from(record).expect("valid command record"))
        .collect();

    let replayed = replay_commands(decoded_commands);

    assert_eq!(replayed, original);
}

fn round_trip_command(command: EngineCommand) -> EngineCommand {
    let record = CommandRecordV1::from(command);
    let json = serde_json::to_string(&record).expect("command record should serialize");
    let decoded: CommandRecordV1 =
        serde_json::from_str(&json).expect("command record should deserialize");
    EngineCommand::try_from(decoded).expect("command record should convert")
}

fn round_trip_event(event: EngineEvent) -> EngineEvent {
    let record = EventRecordV1::from(event);
    let json = serde_json::to_string(&record).expect("event record should serialize");
    let decoded: EventRecordV1 =
        serde_json::from_str(&json).expect("event record should deserialize");
    EngineEvent::try_from(decoded).expect("event record should convert")
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

fn accepted_order(
    order_id: u64,
    participant_id: u64,
    side: Side,
    order_type: OrderType,
    units: u64,
    sequence_number: u64,
) -> AcceptedOrder {
    AcceptedOrder {
        order_id: OrderId::new(order_id),
        participant_id: ParticipantId::new(participant_id),
        side,
        order_type,
        quantity: quantity(units),
        sequence_number: SequenceNumber::new(sequence_number),
    }
}

fn price(value: u64) -> PriceTicks {
    PriceTicks::new(value).expect("test price should be valid")
}

fn quantity(value: u64) -> Quantity {
    Quantity::new(value).expect("test quantity should be valid")
}
