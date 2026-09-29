# Performance

## Current Status

Aster has focused Criterion benchmarks for the in-memory matching core. No production performance claims exist yet.

## Principles

Correctness, deterministic state transitions, and clear event semantics come before optimization.

## Benchmark Suite

Run benchmarks with:

```bash
cargo bench -p aster-core
```

Current benchmark scale:

- 1,000 passive limit order submissions through `AsterEngine`
- 1,000 crossing limit matches against preloaded resting asks
- 1,000 market buy executions against asks
- 1,000 market sell executions against bids
- 1,000 cancellations by `OrderId`
- 1,200-command mixed live session
- 1,200-command mixed replay session

The mixed sessions include passive limit orders, crossing limit orders, market orders, and cancellations.

## Scaling matrix and measured improvement

`cargo bench --locked -p aster-core --bench scaling` adds 25 workloads: one-level
and many-level insertion at 100/1,000/4,000 orders; head/middle/tail/seeded-random
cancellation at 1,000/4,000; large and small market sweeps over one/many levels;
partial fills; capacity rejection; and JSON command serialization.

See the [aggregate-cache investigation](benchmarks/cache-investigation.md) and
[all measured means and 95% confidence intervals](benchmarks/2026-09-29-cache.csv).
Cached level/book totals remove repeated full-book quantity scans. Insertion
improved substantially; cancellation still scans queues, and measured tail/random
cancellation regressed. Setup and destruction are excluded with Criterion's
`iter_batched_ref`; matching event generation and retained V1 logs are included.

## Non-Goals

Current benchmarks do not measure file persistence, journal synchronization,
networking, multi-symbol routing, database I/O, UI paths, or real market data
ingestion.

## Reporting

Benchmark results depend on machine, OS, CPU power state, Rust version, and build mode. Future recorded results should include:

- date
- commit or branch
- command
- host CPU and OS
- Rust toolchain
- benchmark name and dataset shape
- Criterion summary

Do not describe Aster as production-grade financial infrastructure based on these benchmarks.

## Related Documentation

- [Architecture](architecture.md)
- [Testing strategy](testing-strategy.md)
- [Limitations](limitations.md)
