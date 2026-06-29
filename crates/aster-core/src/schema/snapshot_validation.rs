use std::collections::HashSet;

use crate::{AsterError, EngineSnapshot, OrderType, PriceLevelSnapshot, Side};

pub(super) fn validate_snapshot(snapshot: &EngineSnapshot) -> Result<(), AsterError> {
    validate_summary(snapshot)?;
    validate_level_order(&snapshot.bid_levels, Side::Buy)?;
    validate_level_order(&snapshot.ask_levels, Side::Sell)?;

    let mut order_ids = HashSet::new();
    let mut sequence_numbers = HashSet::new();
    let mut total_quantity = 0_u64;
    let mut max_order_id = 0_u64;
    let mut max_sequence_number = 0_u64;

    for (side, levels) in [
        (Side::Buy, snapshot.bid_levels.as_slice()),
        (Side::Sell, snapshot.ask_levels.as_slice()),
    ] {
        for level in levels {
            validate_level(
                level,
                side,
                &mut order_ids,
                &mut sequence_numbers,
                &mut total_quantity,
                &mut max_order_id,
                &mut max_sequence_number,
            )?;
        }
    }

    if total_quantity != snapshot.total_resting_quantity {
        return Err(AsterError::SnapshotQuantityMismatch);
    }
    if snapshot.next_order_id.as_u64() == 0
        || snapshot.next_sequence_number.as_u64() == 0
        || snapshot.next_order_id.as_u64() <= max_order_id
        || snapshot.next_sequence_number.as_u64() <= max_sequence_number
    {
        return Err(AsterError::SnapshotAllocatorInvalid);
    }

    Ok(())
}

fn validate_summary(snapshot: &EngineSnapshot) -> Result<(), AsterError> {
    if snapshot.bid_level_count != snapshot.bid_levels.len()
        || snapshot.ask_level_count != snapshot.ask_levels.len()
    {
        return Err(AsterError::SnapshotLevelCountMismatch);
    }
    if snapshot.best_bid != snapshot.bid_levels.first().map(|level| level.price)
        || snapshot.best_ask != snapshot.ask_levels.first().map(|level| level.price)
    {
        return Err(AsterError::SnapshotBestPriceMismatch);
    }
    if let (Some(best_bid), Some(best_ask)) = (snapshot.best_bid, snapshot.best_ask) {
        if best_bid >= best_ask {
            return Err(AsterError::SnapshotBookCrossed);
        }
    }

    Ok(())
}

fn validate_level_order(levels: &[PriceLevelSnapshot], side: Side) -> Result<(), AsterError> {
    let ordered = levels.windows(2).all(|pair| match side {
        Side::Buy => pair[0].price > pair[1].price,
        Side::Sell => pair[0].price < pair[1].price,
    });
    if !ordered {
        return Err(AsterError::SnapshotLevelOrderInvalid);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_level(
    level: &PriceLevelSnapshot,
    expected_side: Side,
    order_ids: &mut HashSet<u64>,
    sequence_numbers: &mut HashSet<u64>,
    total_quantity: &mut u64,
    max_order_id: &mut u64,
    max_sequence_number: &mut u64,
) -> Result<(), AsterError> {
    if level.orders.is_empty() {
        return Err(AsterError::SnapshotEmptyPriceLevel);
    }
    if !level
        .orders
        .windows(2)
        .all(|pair| pair[0].sequence_number < pair[1].sequence_number)
    {
        return Err(AsterError::SnapshotFifoInvalid);
    }

    for order in &level.orders {
        if order.side != expected_side {
            return Err(AsterError::SnapshotSideMismatch);
        }
        match order.order_type {
            OrderType::Limit { price } if price == level.price => {}
            OrderType::Limit { .. } => return Err(AsterError::PriceLevelMismatch),
            OrderType::Market => return Err(AsterError::MarketOrderCannotRest),
        }
        if order.order_id.as_u64() == 0 || order.sequence_number.as_u64() == 0 {
            return Err(AsterError::InvalidOrderState);
        }
        if !order_ids.insert(order.order_id.as_u64()) {
            return Err(AsterError::DuplicateOrderId);
        }
        if !sequence_numbers.insert(order.sequence_number.as_u64()) {
            return Err(AsterError::DuplicateSequenceNumber);
        }

        *total_quantity = total_quantity
            .checked_add(order.quantity.as_u64())
            .ok_or(AsterError::QuantityOverflow)?;
        *max_order_id = (*max_order_id).max(order.order_id.as_u64());
        *max_sequence_number = (*max_sequence_number).max(order.sequence_number.as_u64());
    }

    Ok(())
}
