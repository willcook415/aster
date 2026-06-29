# Testing Strategy

## Goal

Correctness and determinism should be proven before performance work.

## Current Coverage

The workspace currently has 147 passing tests. Coverage is divided across:

- Unit tests for typed constructors, commands, events, allocation exhaustion,
  and primitive validation.
- Engine integration tests for passive and crossing limits, market orders,
  price priority, FIFO, partial and full fills, market expiry, cancellation,
  rejection events, and allocation behavior.
- `OrderBook` and `PriceLevel` tests for ordered levels, FIFO storage, duplicate
  IDs, invalid resting orders, cancellation support, and checked quantity
  totals.
- Event-log tests for append order and equality with events returned from
  command processing.
- Replay tests comparing directly produced events and complete snapshots with
  replay results.
- Schema tests for command, event, and full-snapshot JSON round trips, malformed
  values, integer price representation, and unsupported versions.
- Golden V1 JSON fixture tests that lock command, event, and full-snapshot field
  names, enum tags, nesting, version values, and representative conversions.
- In-memory session tests covering ordered construction, exact event capture,
  full snapshots, fresh-state verification, explicit mismatch reporting, empty
  sessions, and mixed command sequences.
- CLI unit tests covering the built-in scenario catalog, verification of every
  scenario, default behavior, unknown names, and report section rendering.
- Mixed-session invariant tests covering uncrossed books, deterministic level
  order, FIFO identity, unique resting IDs, cancellation invariants, full replay
  equality, and explicit quantity accounting.
- An independent test-only reference model that compares exact events and full
  visible snapshots after every command across 11 deterministic sequences and
  100 commands, including a 60-command mixed session.

## Deterministic Replay Verification

Replay processes the same ordered command sequence through a fresh engine.
Tests compare the complete event sequence and full final snapshot, including
ordered levels, FIFO resting orders, remaining quantities, IDs, sequence
numbers, and next allocation values.

## Future Improvements

Property-based testing is not implemented yet. The deterministic reference
model is an independent matching oracle for ordinary limit orders, market
orders, fills, priority, expiry, and cancellation, but does not cover allocation
exhaustion, quantity overflow, schema conversion, or arbitrary generated input.
Replay still proves repeatability through the production engine itself and is
complementary to the reference-model comparison.

## Required Checks

```bash
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Benchmark compilation is also checked with:

```bash
cargo bench -p aster-core --no-run
```
