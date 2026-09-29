use aster_core::*;
use serde_json::json;

#[test]
fn unknown_fields_are_rejected_at_all_command_boundaries() {
    let valid = json!({"schema_version":1,"command":{"type":"submit_order","participant_id":1,"side":"buy","order_type":{"type":"limit","price":100},"quantity":5}});
    for path in [vec![], vec!["command"], vec!["command", "order_type"]] {
        let mut record = valid.clone();
        let mut target = &mut record;
        for key in path {
            target = &mut target[key];
        }
        target["time_in_force"] = json!("immediate_or_cancel");
        assert!(serde_json::from_value::<CommandRecordV1>(record).is_err());
    }
    let mut missing = valid.clone();
    missing["command"]
        .as_object_mut()
        .unwrap()
        .remove("quantity");
    assert!(serde_json::from_value::<CommandRecordV1>(missing).is_err());
    let mut future = valid;
    future["schema_version"] = json!(2);
    let parsed: CommandRecordV1 = serde_json::from_value(future).unwrap();
    assert_eq!(
        EngineCommand::try_from(parsed),
        Err(AsterError::UnsupportedSchemaVersion)
    );
}

#[test]
fn event_and_snapshot_extensions_also_fail_closed() {
    let snapshot = SnapshotRecordV1::from(AsterEngine::new().snapshot());
    let mut record = serde_json::to_value(snapshot).unwrap();
    record["snapshot"]["unknown"] = json!(true);
    assert!(serde_json::from_value::<SnapshotRecordV1>(record).is_err());
    let event = json!({"schema_version":1,"event":{"type":"cancel_rejected","order_id":1,"participant_id":1,"reason":"order_not_found","unknown":true}});
    assert!(serde_json::from_value::<EventRecordV1>(event).is_err());
}

#[test]
fn self_trading_is_explicitly_allowed_under_v1_matching_rules() {
    let mut engine = AsterEngine::new();
    for side in [Side::Sell, Side::Buy] {
        engine.process_command(EngineCommand::submit_order(OrderRequest::new(
            ParticipantId::new(7),
            side,
            OrderType::Limit {
                price: PriceTicks::new(100).unwrap(),
            },
            Quantity::new(3).unwrap(),
        )));
    }
    assert!(
        matches!(engine.event_log().last(), Some(EngineEvent::TradeExecuted {quantity, ..}) if quantity.as_u64() == 3)
    );
    assert!(engine.order_book().is_empty());
}
