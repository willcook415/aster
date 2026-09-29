use aster_core::*;

fn submit(side: Side, units: u64, price: Option<u64>) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(1),
        side,
        price
            .map(|p| OrderType::Limit {
                price: PriceTicks::new(p).unwrap(),
            })
            .unwrap_or(OrderType::Market),
        Quantity::new(units).unwrap(),
    ))
}

#[test]
fn expiry_is_explicit_after_fills_and_before_command_completion() {
    let mut engine = SequencedEngine::new();
    let first = engine
        .process_command(submit(Side::Sell, 5, Some(100)))
        .unwrap();
    let second = engine.process_command(submit(Side::Buy, 8, None)).unwrap();
    assert_eq!(first.events.len(), 2);
    assert_eq!(second.command_sequence, 2);
    assert_eq!(
        second
            .events
            .iter()
            .map(|e| e.event_sequence)
            .collect::<Vec<_>>(),
        vec![3, 4, 5, 6]
    );
    assert!(second.events.iter().all(|e| e.command_sequence == 2));
    assert_eq!(
        second.events[2].event,
        AuditEventV2::OrderExpired {
            order_id: 2,
            remaining_quantity: 3,
            reason: ExpiryReasonV2::InsufficientLiquidity
        }
    );
    assert_eq!(second.events[3].event, AuditEventV2::CommandCompleted);
    let mut replay = SequencedEngine::new();
    replay.verify_batch(&first).unwrap();
    replay.verify_batch(&second).unwrap();
    assert_eq!(engine, replay);
}

#[test]
fn empty_market_expires_and_fully_filled_market_does_not() {
    let mut engine = SequencedEngine::new();
    let empty = engine.process_command(submit(Side::Buy, 3, None)).unwrap();
    assert!(matches!(
        empty.events[1].event,
        AuditEventV2::OrderExpired {
            remaining_quantity: 3,
            ..
        }
    ));
    engine
        .process_command(submit(Side::Sell, 3, Some(100)))
        .unwrap();
    let full = engine.process_command(submit(Side::Buy, 3, None)).unwrap();
    assert_eq!(full.events.len(), 3);
    assert!(!full
        .events
        .iter()
        .any(|e| matches!(e.event, AuditEventV2::OrderExpired { .. })));
}

#[test]
fn rejections_have_command_correlation_without_consuming_priority() {
    let mut engine = SequencedEngine::new();
    engine
        .process_command(submit(Side::Buy, u64::MAX, Some(100)))
        .unwrap();
    let before = engine.snapshot();
    let rejected = engine
        .process_command(submit(Side::Buy, 1, Some(99)))
        .unwrap();
    assert_eq!(rejected.command_sequence, 2);
    assert!(matches!(
        rejected.events[0].event,
        AuditEventV2::EngineEvent {
            event: EventDtoV1::OrderRejected { .. }
        }
    ));
    let cancel = engine
        .process_command(EngineCommand::cancel_order(
            OrderId::new(999),
            ParticipantId::new(1),
        ))
        .unwrap();
    assert_eq!(cancel.command_sequence, 3);
    assert_eq!(engine.snapshot(), before);
}

#[test]
fn altered_sequence_terminal_event_and_rules_version_fail_atomically() {
    let original = SequencedEngine::new()
        .process_command(submit(Side::Buy, 1, None))
        .unwrap();
    for index in 0..4 {
        let mut modified = original.clone();
        match index {
            0 => modified.events[0].event_sequence = 9,
            1 => {
                modified.events.pop();
            }
            2 => modified.matching_rules_version = 2,
            _ => modified.command_sequence = 2,
        }
        let mut engine = SequencedEngine::new();
        let before = engine.clone();
        assert!(engine.verify_batch(&modified).is_err());
        assert_eq!(engine, before);
    }
}

#[test]
fn empty_market_golden_fixture_locks_v2_wire_shape() {
    let batch = SequencedEngine::new()
        .process_command(submit(Side::Buy, 3, None))
        .unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/schema/v2/empty-market.json")).unwrap();
    assert_eq!(serde_json::to_value(&batch).unwrap(), expected);
    let decoded: CommandBatchV2 = serde_json::from_value(expected).unwrap();
    SequencedEngine::new().verify_batch(&decoded).unwrap();
}
