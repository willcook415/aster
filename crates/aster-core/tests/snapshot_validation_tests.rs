use aster_core::{
    AcceptedOrderDtoV1, AsterError, EngineSnapshot, EngineSnapshotDtoV1, OrderTypeDtoV1,
    PriceLevelSnapshotDtoV1, SideDtoV1, SnapshotRecordV1, ASTER_SCHEMA_VERSION,
};

#[test]
fn valid_snapshot_record_is_accepted() {
    let snapshot = EngineSnapshot::try_from(valid_record()).expect("valid snapshot must convert");

    assert_eq!(snapshot.bid_level_count, 1);
    assert_eq!(snapshot.ask_level_count, 1);
    assert_eq!(snapshot.total_resting_quantity, 10);
}

#[test]
fn duplicate_resting_order_id_is_rejected() {
    let mut record = valid_record();
    record.snapshot.ask_levels[0].orders[0].order_id = 1;

    assert_eq!(
        EngineSnapshot::try_from(record),
        Err(AsterError::DuplicateOrderId)
    );
}

#[test]
fn side_and_level_mismatch_is_rejected() {
    let mut record = valid_record();
    record.snapshot.bid_levels[0].orders[0].side = SideDtoV1::Sell;

    assert_eq!(
        EngineSnapshot::try_from(record),
        Err(AsterError::SnapshotSideMismatch)
    );
}

#[test]
fn resting_order_price_mismatch_is_rejected() {
    let mut record = valid_record();
    record.snapshot.bid_levels[0].orders[0].order_type = OrderTypeDtoV1::Limit { price: 99 };

    assert_eq!(
        EngineSnapshot::try_from(record),
        Err(AsterError::PriceLevelMismatch)
    );
}

#[test]
fn incorrect_summary_level_count_is_rejected() {
    let mut record = valid_record();
    record.snapshot.bid_level_count = 2;

    assert_eq!(
        EngineSnapshot::try_from(record),
        Err(AsterError::SnapshotLevelCountMismatch)
    );
}

#[test]
fn incorrect_summary_quantity_is_rejected() {
    let mut record = valid_record();
    record.snapshot.total_resting_quantity = 11;

    assert_eq!(
        EngineSnapshot::try_from(record),
        Err(AsterError::SnapshotQuantityMismatch)
    );
}

#[test]
fn levels_out_of_matching_order_are_rejected() {
    let mut record = valid_record();
    record.snapshot.bid_levels.push(PriceLevelSnapshotDtoV1 {
        price: 101,
        orders: vec![accepted_order(3, SideDtoV1::Buy, 101, 1, 3)],
    });
    record.snapshot.bid_level_count = 2;
    record.snapshot.total_resting_quantity = 11;
    record.snapshot.next_order_id = 4;
    record.snapshot.next_sequence_number = 4;

    assert_eq!(
        EngineSnapshot::try_from(record),
        Err(AsterError::SnapshotLevelOrderInvalid)
    );
}

#[test]
fn crossed_book_is_rejected() {
    let mut record = valid_record();
    record.snapshot.best_ask = Some(99);
    record.snapshot.ask_levels[0].price = 99;
    record.snapshot.ask_levels[0].orders[0].order_type = OrderTypeDtoV1::Limit { price: 99 };

    assert_eq!(
        EngineSnapshot::try_from(record),
        Err(AsterError::SnapshotBookCrossed)
    );
}

#[test]
fn next_order_id_not_ahead_of_visible_orders_is_rejected() {
    let mut record = valid_record();
    record.snapshot.next_order_id = 1;

    assert_eq!(
        EngineSnapshot::try_from(record),
        Err(AsterError::SnapshotAllocatorInvalid)
    );
}

#[test]
fn next_sequence_number_not_ahead_of_visible_orders_is_rejected() {
    let mut record = valid_record();
    record.snapshot.next_sequence_number = 1;

    assert_eq!(
        EngineSnapshot::try_from(record),
        Err(AsterError::SnapshotAllocatorInvalid)
    );
}

#[test]
fn malformed_fifo_sequence_order_is_rejected() {
    let mut record = valid_record();
    record.snapshot.bid_levels[0]
        .orders
        .push(accepted_order(3, SideDtoV1::Buy, 100, 1, 1));
    record.snapshot.total_resting_quantity = 11;
    record.snapshot.next_order_id = 4;

    assert_eq!(
        EngineSnapshot::try_from(record),
        Err(AsterError::SnapshotFifoInvalid)
    );
}

fn valid_record() -> SnapshotRecordV1 {
    SnapshotRecordV1 {
        schema_version: ASTER_SCHEMA_VERSION,
        snapshot: EngineSnapshotDtoV1 {
            best_bid: Some(100),
            best_ask: Some(102),
            bid_level_count: 1,
            ask_level_count: 1,
            total_resting_quantity: 10,
            bid_levels: vec![PriceLevelSnapshotDtoV1 {
                price: 100,
                orders: vec![accepted_order(1, SideDtoV1::Buy, 100, 4, 1)],
            }],
            ask_levels: vec![PriceLevelSnapshotDtoV1 {
                price: 102,
                orders: vec![accepted_order(2, SideDtoV1::Sell, 102, 6, 2)],
            }],
            next_order_id: 3,
            next_sequence_number: 3,
        },
    }
}

fn accepted_order(
    order_id: u64,
    side: SideDtoV1,
    price: u64,
    quantity: u64,
    sequence_number: u64,
) -> AcceptedOrderDtoV1 {
    AcceptedOrderDtoV1 {
        order_id,
        participant_id: order_id,
        side,
        order_type: OrderTypeDtoV1::Limit { price },
        quantity,
        sequence_number,
    }
}
