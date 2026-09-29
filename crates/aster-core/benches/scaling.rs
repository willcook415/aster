//! Workload shape and scaling matrix; see docs/performance.md for timing scope.
use aster_core::*;
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box, time::Duration};

fn limit(side: Side, ticks: u64, units: u64) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(1),
        side,
        OrderType::Limit {
            price: PriceTicks::new(ticks).unwrap(),
        },
        Quantity::new(units).unwrap(),
    ))
}
fn market(side: Side, units: u64) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(2),
        side,
        OrderType::Market,
        Quantity::new(units).unwrap(),
    ))
}
fn preload(n: u64, side: Side, many_levels: bool) -> AsterEngine {
    let mut engine = AsterEngine::new();
    for i in 0..n {
        engine.process_command(limit(side, 100 + if many_levels { i } else { 0 }, 2));
    }
    engine.clear_event_log();
    engine
}
fn process(engine: &mut AsterEngine, commands: &[EngineCommand]) {
    for command in commands {
        black_box(engine.process_command(*command));
    }
}
fn insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling/insertion");
    for n in [100, 1000, 4000] {
        for many in [false, true] {
            let shape = if many { "many_levels" } else { "one_level" };
            let commands: Vec<_> = (0..n)
                .map(|i| limit(Side::Buy, 100 + if many { i } else { 0 }, 1))
                .collect();
            group.throughput(Throughput::Elements(n));
            group.bench_with_input(BenchmarkId::new(shape, n), &commands, |b, commands| {
                b.iter_batched_ref(
                    AsterEngine::new,
                    |engine| process(engine, commands),
                    BatchSize::LargeInput,
                );
            });
        }
    }
    group.finish();
}
fn cancel_ids(n: u64, shape: &str) -> Vec<u64> {
    let mut ids: Vec<_> = (1..=n).collect();
    match shape {
        "tail" => ids.reverse(),
        "middle" => {
            let mut remaining = ids;
            ids = Vec::new();
            while !remaining.is_empty() {
                ids.push(remaining.remove(remaining.len() / 2));
            }
        }
        "random" => {
            let mut state = 0x4153544552_u64;
            for i in (1..ids.len()).rev() {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                ids.swap(i, state as usize % (i + 1));
            }
        }
        _ => {}
    }
    ids
}
fn cancellation(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling/cancel");
    for n in [1000, 4000] {
        for shape in ["head", "middle", "tail", "random"] {
            let commands: Vec<_> = cancel_ids(n, shape)
                .into_iter()
                .map(|id| EngineCommand::cancel_order(OrderId::new(id), ParticipantId::new(1)))
                .collect();
            group.throughput(Throughput::Elements(n));
            group.bench_function(BenchmarkId::new(shape, n), |b| {
                b.iter_batched_ref(
                    || preload(n, Side::Buy, false),
                    |engine| process(engine, &commands),
                    BatchSize::LargeInput,
                );
            });
        }
    }
    group.finish();
}
fn sweeps(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling/sweep");
    for n in [1000, 4000] {
        for many in [false, true] {
            for single in [false, true] {
                let name = format!(
                    "{}_{}",
                    if many { "many_levels" } else { "one_level" },
                    if single {
                        "single_market"
                    } else {
                        "small_markets"
                    }
                );
                let commands = if single {
                    vec![market(Side::Buy, n * 2)]
                } else {
                    vec![market(Side::Buy, 2); n as usize]
                };
                group.throughput(Throughput::Elements(n)); // fills, not inbound commands
                group.bench_function(BenchmarkId::new(name, n), |b| {
                    b.iter_batched_ref(
                        || preload(n, Side::Sell, many),
                        |engine| process(engine, &commands),
                        BatchSize::LargeInput,
                    );
                });
            }
        }
    }
    group.finish();
    c.bench_function("boundary/partial_fill_1000", |b| {
        b.iter_batched_ref(
            || preload(1000, Side::Sell, false),
            |engine| {
                black_box(engine.process_command(market(Side::Buy, 1)));
            },
            BatchSize::LargeInput,
        )
    });
    c.bench_function("boundary/capacity_reject", |b| {
        b.iter_batched_ref(
            || {
                let mut e = AsterEngine::new();
                e.process_command(limit(Side::Buy, 100, u64::MAX));
                e
            },
            |engine| {
                black_box(engine.process_command(limit(Side::Buy, 99, 1)));
            },
            BatchSize::LargeInput,
        )
    });
    let records: Vec<_> = (0..1000)
        .map(|_| CommandRecordV1::from(limit(Side::Buy, 100, 1)))
        .collect();
    c.bench_function("schema/serialize_1000_commands", |b| {
        b.iter(|| black_box(serde_json::to_vec(&records).unwrap()))
    });
}
criterion_group! {name=benches; config=Criterion::default().sample_size(10).warm_up_time(Duration::from_millis(500)).measurement_time(Duration::from_secs(1)); targets=insertion,cancellation,sweeps}
criterion_main!(benches);
