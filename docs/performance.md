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

## Non-Goals

Current benchmarks do not measure file persistence, schema serialization,
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
