use std::time::Duration;

use aster_core::{
    replay_commands, AsterEngine, EngineCommand, OrderId, OrderRequest, OrderType, ParticipantId,
    PriceTicks, Quantity, Side,
};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

const WORKLOAD_SIZE: u64 = 1_000;
const MIXED_SESSION_SIZE: u64 = 1_200;

fn price(value: u64) -> PriceTicks {
    PriceTicks::new(value).expect("benchmark price must be non-zero")
}

fn quantity(value: u64) -> Quantity {
    Quantity::new(value).expect("benchmark quantity must be non-zero")
}

fn limit_command(side: Side, price: PriceTicks, quantity_value: u64) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(1),
        side,
        OrderType::Limit { price },
        quantity(quantity_value),
    ))
}

fn market_command(side: Side, quantity_value: u64) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(1),
        side,
        OrderType::Market,
        quantity(quantity_value),
    ))
}

fn passive_limit_commands(count: u64) -> Vec<EngineCommand> {
    (0..count)
        .map(|_| limit_command(Side::Buy, price(100), 1))
        .collect()
}

fn crossing_limit_commands(count: u64) -> Vec<EngineCommand> {
    (0..count)
        .map(|_| limit_command(Side::Buy, price(100), 1))
        .collect()
}

fn market_buy_commands(count: u64) -> Vec<EngineCommand> {
    (0..count).map(|_| market_command(Side::Buy, 1)).collect()
}

fn market_sell_commands(count: u64) -> Vec<EngineCommand> {
    (0..count).map(|_| market_command(Side::Sell, 1)).collect()
}

fn cancellation_commands(count: u64) -> Vec<EngineCommand> {
    (1..=count)
        .map(|order_id| EngineCommand::cancel_order(OrderId::new(order_id), ParticipantId::new(1)))
        .collect()
}

fn mixed_session_commands(count: u64) -> Vec<EngineCommand> {
    let chunk = count / 4;
    let mut commands = Vec::with_capacity(count as usize);

    for _ in 0..chunk {
        commands.push(limit_command(Side::Buy, price(99), 1));
    }
    for _ in 0..chunk {
        commands.push(limit_command(Side::Sell, price(99), 1));
    }
    for _ in 0..chunk {
        commands.push(limit_command(Side::Sell, price(101), 1));
    }
    for _ in 0..(chunk / 2) {
        commands.push(market_command(Side::Buy, 1));
    }
    let first_remaining_ask_id = (chunk * 2) + (chunk / 2) + 1;
    for order_id in first_remaining_ask_id..first_remaining_ask_id + (chunk / 2) {
        commands.push(EngineCommand::cancel_order(
            OrderId::new(order_id),
            ParticipantId::new(1),
        ));
    }

    commands
}

fn engine_with_resting_asks(count: u64) -> AsterEngine {
    let mut engine = AsterEngine::new();
    for offset in 0..count {
        engine.process_command(limit_command(Side::Sell, price(100 + offset), 1));
    }
    engine.clear_event_log();
    engine
}

fn engine_with_resting_asks_at_price(count: u64, ask_price: PriceTicks) -> AsterEngine {
    let mut engine = AsterEngine::new();
    for _ in 0..count {
        engine.process_command(limit_command(Side::Sell, ask_price, 1));
    }
    engine.clear_event_log();
    engine
}

fn engine_with_resting_bids(count: u64) -> AsterEngine {
    let mut engine = AsterEngine::new();
    for offset in 0..count {
        engine.process_command(limit_command(Side::Buy, price(100 + offset), 1));
    }
    engine.clear_event_log();
    engine
}

fn engine_with_cancellable_orders(count: u64) -> AsterEngine {
    let mut engine = AsterEngine::new();
    for _ in 0..count {
        engine.process_command(limit_command(Side::Buy, price(100), 1));
    }
    engine.clear_event_log();
    engine
}

fn bench_passive_limit_order_insertion(c: &mut Criterion) {
    let commands = passive_limit_commands(WORKLOAD_SIZE);

    c.bench_function("engine/passive_limit_order_insertion_1000", |b| {
        b.iter_batched(
            AsterEngine::new,
            |mut engine| black_box(engine.process_commands(commands.clone())),
            BatchSize::LargeInput,
        );
    });
}

fn bench_crossing_limit_order_matching(c: &mut Criterion) {
    let commands = crossing_limit_commands(WORKLOAD_SIZE);

    c.bench_function("engine/crossing_limit_order_matching_1000", |b| {
        b.iter_batched(
            || engine_with_resting_asks_at_price(WORKLOAD_SIZE, price(100)),
            |mut engine| black_box(engine.process_commands(commands.clone())),
            BatchSize::LargeInput,
        );
    });
}

fn bench_market_order_sweeps(c: &mut Criterion) {
    let buy_commands = market_buy_commands(WORKLOAD_SIZE);
    let sell_commands = market_sell_commands(WORKLOAD_SIZE);

    c.bench_function("engine/market_buy_sweep_1000", |b| {
        b.iter_batched(
            || engine_with_resting_asks(WORKLOAD_SIZE),
            |mut engine| black_box(engine.process_commands(buy_commands.clone())),
            BatchSize::LargeInput,
        );
    });

    c.bench_function("engine/market_sell_sweep_1000", |b| {
        b.iter_batched(
            || engine_with_resting_bids(WORKLOAD_SIZE),
            |mut engine| black_box(engine.process_commands(sell_commands.clone())),
            BatchSize::LargeInput,
        );
    });
}

fn bench_cancellation_by_order_id(c: &mut Criterion) {
    let commands = cancellation_commands(WORKLOAD_SIZE);

    c.bench_function("engine/cancellation_by_order_id_1000", |b| {
        b.iter_batched(
            || engine_with_cancellable_orders(WORKLOAD_SIZE),
            |mut engine| black_box(engine.process_commands(commands.clone())),
            BatchSize::LargeInput,
        );
    });
}

fn bench_mixed_session(c: &mut Criterion) {
    let commands = mixed_session_commands(MIXED_SESSION_SIZE);

    c.bench_function("engine/mixed_session_1200", |b| {
        b.iter_batched(
            AsterEngine::new,
            |mut engine| black_box(engine.process_commands(commands.clone())),
            BatchSize::LargeInput,
        );
    });
}

fn bench_replay_mixed_session(c: &mut Criterion) {
    let commands = mixed_session_commands(MIXED_SESSION_SIZE);

    c.bench_function("replay/mixed_session_1200", |b| {
        b.iter(|| black_box(replay_commands(commands.clone())));
    });
}

fn criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(1))
}

criterion_group! {
    name = benches;
    config = criterion_config();
    targets =
        bench_passive_limit_order_insertion,
        bench_crossing_limit_order_matching,
        bench_market_order_sweeps,
        bench_cancellation_by_order_id,
        bench_mixed_session,
        bench_replay_mixed_session
}
criterion_main!(benches);
