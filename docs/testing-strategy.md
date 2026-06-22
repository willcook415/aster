# Testing Strategy

## Goal

Correctness and determinism should be proven before performance work.

## Unit Tests

TODO: Cover invalid orders, passive resting, best bid and ask updates, crossing limit orders, FIFO, better-price priority, partial fills, full fills, market sweeps, unfilled market expiry, cancellation, and replay.

## Replay Tests

TODO: Assert that the same input sequence produces identical accepted or rejected commands, trades, event logs, and final book state.

## Benchmark Tests

TODO: Add benchmarks only after the matching rules are implemented and covered by tests.

