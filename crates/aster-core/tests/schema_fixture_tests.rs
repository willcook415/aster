use aster_core::{
    AcceptedOrderDtoV1, AsterError, AsterErrorDtoV1, CommandDtoV1, CommandRecordV1, EngineCommand,
    EngineEvent, EngineSnapshot, EngineSnapshotDtoV1, EventDtoV1, EventRecordV1, OrderType,
    OrderTypeDtoV1, PriceLevelSnapshotDtoV1, Side, SideDtoV1, SnapshotRecordV1,
    ASTER_SCHEMA_VERSION,
};
use serde::Serialize;
use serde_json::Value;

const SUBMIT_LIMIT: &str = include_str!("fixtures/schema/v1/submit_limit_order_command.json");
const SUBMIT_MARKET: &str = include_str!("fixtures/schema/v1/submit_market_order_command.json");
const CANCEL_ORDER: &str = include_str!("fixtures/schema/v1/cancel_order_command.json");
const ORDER_ACCEPTED: &str = include_str!("fixtures/schema/v1/order_accepted_event.json");
const ORDER_REJECTED: &str = include_str!("fixtures/schema/v1/order_rejected_event.json");
const TRADE_EXECUTED: &str = include_str!("fixtures/schema/v1/trade_executed_event.json");
const ORDER_CANCELLED: &str = include_str!("fixtures/schema/v1/order_cancelled_event.json");
const CANCEL_REJECTED: &str = include_str!("fixtures/schema/v1/cancel_rejected_event.json");
const FULL_SNAPSHOT: &str = include_str!("fixtures/schema/v1/full_engine_snapshot.json");

#[test]
fn command_fixtures_lock_v1_json_shapes_and_convert_to_domain_commands() {
    let cases = [
        (
            SUBMIT_LIMIT,
            CommandRecordV1 {
                schema_version: ASTER_SCHEMA_VERSION,
                command: CommandDtoV1::SubmitOrder {
                    participant_id: 11,
                    side: SideDtoV1::Buy,
                    order_type: OrderTypeDtoV1::Limit { price: 101 },
                    quantity: 25,
                },
            },
        ),
        (
            SUBMIT_MARKET,
            CommandRecordV1 {
                schema_version: ASTER_SCHEMA_VERSION,
                command: CommandDtoV1::SubmitOrder {
                    participant_id: 12,
                    side: SideDtoV1::Sell,
                    order_type: OrderTypeDtoV1::Market,
                    quantity: 30,
                },
            },
        ),
        (
            CANCEL_ORDER,
            CommandRecordV1 {
                schema_version: ASTER_SCHEMA_VERSION,
                command: CommandDtoV1::CancelOrder {
                    order_id: 99,
                    participant_id: 12,
                },
            },
        ),
    ];

    for (fixture, expected_record) in cases {
        let decoded: CommandRecordV1 =
            serde_json::from_str(fixture).expect("command fixture must deserialize");
        assert_eq!(decoded, expected_record);
        assert_json_shape(fixture, &expected_record);

        match EngineCommand::try_from(decoded).expect("fixture command must convert") {
            EngineCommand::SubmitOrder(request) if request.participant_id.as_u64() == 11 => {
                assert_eq!(request.side, Side::Buy);
                assert_eq!(request.quantity.as_u64(), 25);
                assert_eq!(
                    request.order_type,
                    OrderType::Limit {
                        price: aster_core::PriceTicks::new(101).expect("positive fixture price")
                    }
                );
            }
            EngineCommand::SubmitOrder(request) => {
                assert_eq!(request.participant_id.as_u64(), 12);
                assert_eq!(request.side, Side::Sell);
                assert_eq!(request.order_type, OrderType::Market);
                assert_eq!(request.quantity.as_u64(), 30);
            }
            EngineCommand::CancelOrder {
                order_id,
                participant_id,
            } => {
                assert_eq!(order_id.as_u64(), 99);
                assert_eq!(participant_id.as_u64(), 12);
            }
        }
    }
}

#[test]
fn event_fixtures_lock_v1_json_shapes_and_convert_to_domain_events() {
    let cases = [
        (
            ORDER_ACCEPTED,
            EventRecordV1 {
                schema_version: ASTER_SCHEMA_VERSION,
                event: EventDtoV1::OrderAccepted {
                    order: accepted_order(1, 10, SideDtoV1::Sell, 105, 40, 1),
                },
            },
        ),
        (
            ORDER_REJECTED,
            EventRecordV1 {
                schema_version: ASTER_SCHEMA_VERSION,
                event: EventDtoV1::OrderRejected {
                    reason: AsterErrorDtoV1::OrderIdExhausted,
                },
            },
        ),
        (
            TRADE_EXECUTED,
            EventRecordV1 {
                schema_version: ASTER_SCHEMA_VERSION,
                event: EventDtoV1::TradeExecuted {
                    resting_order_id: 1,
                    incoming_order_id: 2,
                    price: 101,
                    quantity: 20,
                },
            },
        ),
        (
            ORDER_CANCELLED,
            EventRecordV1 {
                schema_version: ASTER_SCHEMA_VERSION,
                event: EventDtoV1::OrderCancelled {
                    order_id: 5,
                    participant_id: 2,
                },
            },
        ),
        (
            CANCEL_REJECTED,
            EventRecordV1 {
                schema_version: ASTER_SCHEMA_VERSION,
                event: EventDtoV1::CancelRejected {
                    order_id: 5,
                    participant_id: 3,
                    reason: AsterErrorDtoV1::ParticipantMismatch,
                },
            },
        ),
    ];

    for (fixture, expected_record) in cases {
        let decoded: EventRecordV1 =
            serde_json::from_str(fixture).expect("event fixture must deserialize");
        assert_eq!(decoded, expected_record);
        assert_json_shape(fixture, &expected_record);

        let event = EngineEvent::try_from(decoded).expect("fixture event must convert");
        match event {
            EngineEvent::OrderAccepted { order } => {
                assert_eq!(order.order_id.as_u64(), 1);
                assert_eq!(order.participant_id.as_u64(), 10);
                assert_eq!(order.side, Side::Sell);
                assert_eq!(order.quantity.as_u64(), 40);
                assert_eq!(order.sequence_number.as_u64(), 1);
            }
            EngineEvent::OrderRejected { reason } => {
                assert_eq!(reason, AsterError::OrderIdExhausted);
            }
            EngineEvent::TradeExecuted {
                resting_order_id,
                incoming_order_id,
                price,
                quantity,
            } => {
                assert_eq!(resting_order_id.as_u64(), 1);
                assert_eq!(incoming_order_id.as_u64(), 2);
                assert_eq!(price.as_u64(), 101);
                assert_eq!(quantity.as_u64(), 20);
            }
            EngineEvent::OrderCancelled {
                order_id,
                participant_id,
            } => {
                assert_eq!(order_id.as_u64(), 5);
                assert_eq!(participant_id.as_u64(), 2);
            }
            EngineEvent::CancelRejected {
                order_id,
                participant_id,
                reason,
            } => {
                assert_eq!(order_id.as_u64(), 5);
                assert_eq!(participant_id.as_u64(), 3);
                assert_eq!(reason, AsterError::ParticipantMismatch);
            }
        }
    }
}

#[test]
fn full_snapshot_fixture_locks_complete_ordered_book_shape() {
    let expected = full_snapshot_record();
    let decoded: SnapshotRecordV1 =
        serde_json::from_str(FULL_SNAPSHOT).expect("snapshot fixture must deserialize");

    assert_eq!(decoded, expected);
    assert_json_shape(FULL_SNAPSHOT, &expected);

    let snapshot = EngineSnapshot::try_from(decoded).expect("snapshot fixture must convert");
    assert_eq!(snapshot.best_bid.expect("best bid").as_u64(), 100);
    assert_eq!(snapshot.best_ask.expect("best ask").as_u64(), 102);
    assert_eq!(snapshot.total_resting_quantity, 150);
    assert_eq!(snapshot.next_order_id.as_u64(), 6);
    assert_eq!(snapshot.next_sequence_number.as_u64(), 6);
    assert_eq!(
        snapshot
            .bid_levels
            .iter()
            .map(|level| level.price.as_u64())
            .collect::<Vec<_>>(),
        vec![100, 99]
    );
    assert_eq!(
        snapshot
            .ask_levels
            .iter()
            .map(|level| level.price.as_u64())
            .collect::<Vec<_>>(),
        vec![102, 103]
    );
    assert_eq!(
        snapshot.bid_levels[0]
            .orders
            .iter()
            .map(|order| order.order_id.as_u64())
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(snapshot.bid_levels[0].orders[0].participant_id.as_u64(), 10);
    assert_eq!(snapshot.bid_levels[0].orders[0].side, Side::Buy);
    assert_eq!(snapshot.bid_levels[0].orders[0].quantity.as_u64(), 40);
    assert_eq!(snapshot.bid_levels[0].orders[0].sequence_number.as_u64(), 1);
}

#[test]
fn missing_required_command_field_fails_deserialization() {
    let missing_quantity = r#"{
        "schema_version": 1,
        "command": {
            "type": "submit_order",
            "participant_id": 1,
            "side": "buy",
            "order_type": {"type": "market"}
        }
    }"#;

    assert!(serde_json::from_str::<CommandRecordV1>(missing_quantity).is_err());
}

#[test]
fn malformed_command_primitive_fails_domain_conversion() {
    let record = CommandRecordV1 {
        schema_version: ASTER_SCHEMA_VERSION,
        command: CommandDtoV1::SubmitOrder {
            participant_id: 1,
            side: SideDtoV1::Buy,
            order_type: OrderTypeDtoV1::Limit { price: 0 },
            quantity: 1,
        },
    };

    assert_eq!(EngineCommand::try_from(record), Err(AsterError::ZeroPrice));
}

#[test]
fn unsupported_snapshot_version_fails_conversion() {
    let mut record = full_snapshot_record();
    record.schema_version = ASTER_SCHEMA_VERSION + 1;

    assert_eq!(
        EngineSnapshot::try_from(record),
        Err(AsterError::UnsupportedSchemaVersion)
    );
}

#[test]
fn malformed_snapshot_level_price_fails_conversion() {
    let mut record = full_snapshot_record();
    record.snapshot.bid_levels[0].price = 0;

    assert_eq!(EngineSnapshot::try_from(record), Err(AsterError::ZeroPrice));
}

fn full_snapshot_record() -> SnapshotRecordV1 {
    SnapshotRecordV1 {
        schema_version: ASTER_SCHEMA_VERSION,
        snapshot: EngineSnapshotDtoV1 {
            best_bid: Some(100),
            best_ask: Some(102),
            bid_level_count: 2,
            ask_level_count: 2,
            total_resting_quantity: 150,
            bid_levels: vec![
                PriceLevelSnapshotDtoV1 {
                    price: 100,
                    orders: vec![
                        accepted_order(1, 10, SideDtoV1::Buy, 100, 40, 1),
                        accepted_order(2, 11, SideDtoV1::Buy, 100, 20, 2),
                    ],
                },
                PriceLevelSnapshotDtoV1 {
                    price: 99,
                    orders: vec![accepted_order(3, 12, SideDtoV1::Buy, 99, 30, 3)],
                },
            ],
            ask_levels: vec![
                PriceLevelSnapshotDtoV1 {
                    price: 102,
                    orders: vec![accepted_order(4, 20, SideDtoV1::Sell, 102, 25, 4)],
                },
                PriceLevelSnapshotDtoV1 {
                    price: 103,
                    orders: vec![accepted_order(5, 21, SideDtoV1::Sell, 103, 35, 5)],
                },
            ],
            next_order_id: 6,
            next_sequence_number: 6,
        },
    }
}

fn accepted_order(
    order_id: u64,
    participant_id: u64,
    side: SideDtoV1,
    price: u64,
    quantity: u64,
    sequence_number: u64,
) -> AcceptedOrderDtoV1 {
    AcceptedOrderDtoV1 {
        order_id,
        participant_id,
        side,
        order_type: OrderTypeDtoV1::Limit { price },
        quantity,
        sequence_number,
    }
}

fn assert_json_shape<T: Serialize>(fixture: &str, value: &T) {
    let fixture_value: Value =
        serde_json::from_str(fixture).expect("golden fixture must contain valid JSON");
    let serialized_value =
        serde_json::to_value(value).expect("equivalent DTO must serialize to JSON");
    assert_eq!(serialized_value, fixture_value);
}
