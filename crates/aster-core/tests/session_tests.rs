use aster_core::{
    AsterEngine, EngineCommand, EngineEvent, OrderId, OrderRequest, OrderType, ParticipantId,
    PriceTicks, Quantity, SessionRecord, SessionVerificationError, Side,
};

#[test]
fn construction_preserves_commands_events_and_complete_snapshot() {
    let commands = vec![
        limit_command(1, Side::Sell, 101, 10),
        limit_command(2, Side::Buy, 101, 4),
        market_command(3, Side::Buy, 2),
    ];
    let mut engine = AsterEngine::new();
    let expected_events = engine.process_commands(commands.clone());
    let expected_snapshot = engine.snapshot();

    let session = SessionRecord::from_commands(commands.clone());

    assert_eq!(session.commands, commands);
    assert_eq!(session.events, expected_events);
    assert_eq!(session.final_snapshot, expected_snapshot);
    assert_eq!(session.events, engine.event_log());
}

#[test]
fn empty_session_constructs_and_verifies() {
    let session = SessionRecord::from_commands(Vec::new());
    let empty_snapshot = AsterEngine::new().snapshot();

    assert!(session.commands.is_empty());
    assert!(session.events.is_empty());
    assert_eq!(session.final_snapshot, empty_snapshot);
    assert_eq!(session.verify(), Ok(()));
}

#[test]
fn valid_complex_mixed_session_verifies() {
    let session = SessionRecord::from_commands(complex_commands());

    assert_eq!(session.verify(), Ok(()));
    assert!(session
        .events
        .iter()
        .any(|event| matches!(event, EngineEvent::TradeExecuted { .. })));
    assert!(session
        .events
        .iter()
        .any(|event| matches!(event, EngineEvent::OrderCancelled { .. })));
    assert!(session
        .events
        .iter()
        .any(|event| matches!(event, EngineEvent::CancelRejected { .. })));
}

#[test]
fn removed_modified_and_reordered_events_fail_verification() {
    let original = SessionRecord::from_commands(vec![
        limit_command(1, Side::Sell, 101, 10),
        limit_command(2, Side::Buy, 101, 4),
    ]);

    let mut removed = original.clone();
    removed.events.pop();
    assert!(matches!(
        removed.verify(),
        Err(SessionVerificationError::EventMismatch { .. })
    ));

    let mut modified = original.clone();
    modified.events[0] = modified.events[1];
    assert_eq!(
        modified.verify(),
        Err(SessionVerificationError::EventMismatch {
            index: 0,
            recorded: Some(modified.events[0]),
            replayed: Some(original.events[0]),
        })
    );

    let mut reordered = original.clone();
    reordered.events.swap(1, 2);
    assert!(matches!(
        reordered.verify(),
        Err(SessionVerificationError::EventMismatch { index: 1, .. })
    ));
}

#[test]
fn modified_final_snapshot_fails_after_events_match() {
    let mut session = SessionRecord::from_commands(vec![limit_command(1, Side::Buy, 99, 10)]);
    let replayed_snapshot = session.final_snapshot.clone();
    session.final_snapshot.total_resting_quantity = 11;

    assert_eq!(
        session.verify(),
        Err(SessionVerificationError::SnapshotMismatch {
            recorded: Box::new(session.final_snapshot.clone()),
            replayed: Box::new(replayed_snapshot),
        })
    );
}

#[test]
fn full_snapshot_verification_detects_fifo_composition_difference() {
    let mut session = SessionRecord::from_commands(vec![
        limit_command(1, Side::Buy, 100, 4),
        limit_command(2, Side::Buy, 100, 6),
    ]);
    session.final_snapshot.bid_levels[0].orders.swap(0, 1);

    assert!(matches!(
        session.verify(),
        Err(SessionVerificationError::SnapshotMismatch { .. })
    ));
}

#[test]
fn construction_and_verification_use_fresh_replay_state() {
    let mut unrelated_engine = AsterEngine::new();
    unrelated_engine.process_command(limit_command(99, Side::Sell, 500, 99));

    let session = SessionRecord::from_commands(vec![limit_command(1, Side::Buy, 100, 10)]);

    assert_ne!(session.events, unrelated_engine.event_log());
    assert_eq!(session.final_snapshot.best_ask, None);
    assert_eq!(session.verify(), Ok(()));
}

fn complex_commands() -> Vec<EngineCommand> {
    vec![
        limit_command(1, Side::Sell, 101, 10),
        limit_command(2, Side::Sell, 101, 20),
        limit_command(3, Side::Sell, 103, 30),
        limit_command(4, Side::Buy, 99, 15),
        limit_command(5, Side::Buy, 102, 18),
        market_command(6, Side::Buy, 12),
        EngineCommand::cancel_order(OrderId::new(4), ParticipantId::new(4)),
        EngineCommand::cancel_order(OrderId::new(999), ParticipantId::new(9)),
    ]
}

fn limit_command(participant_id: u64, side: Side, price_ticks: u64, units: u64) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(participant_id),
        side,
        OrderType::Limit {
            price: PriceTicks::new(price_ticks).expect("test price must be positive"),
        },
        Quantity::new(units).expect("test quantity must be positive"),
    ))
}

fn market_command(participant_id: u64, side: Side, units: u64) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(participant_id),
        side,
        OrderType::Market,
        Quantity::new(units).expect("test quantity must be positive"),
    ))
}
