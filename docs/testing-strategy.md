# Testing Strategy

## Goal

Correctness and determinism should be proven before performance work.

## Current Coverage

The workspace currently has 171 passing unit/integration tests plus one
compile-checked crate documentation example. Coverage is divided across:

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
- Snapshot-boundary tests rejecting inconsistent counts and quantities,
  duplicate identities, side/price mismatches, invalid price/FIFO order,
  crossed books, and allocator state behind visible orders.
- Persistence tests for exact file shape, JSONL record counts, domain
  round trips, replay verification, line-aware malformed records, missing files,
  and distinct saved-event/snapshot mismatches.
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
- Property-based state-machine tests that run 64 generated cases of 11 to 100
  commands against the same independent model. Generated tails use small
  participant, price, and quantity ranges and include limit/market orders plus
  owner, wrong-participant, and missing-order cancellation actions.

## Deterministic Replay Verification

Replay processes the same ordered command sequence through a fresh engine.
Tests compare the complete event sequence and full final snapshot, including
ordered levels, FIFO resting orders, remaining quantities, IDs, sequence
numbers, and next allocation values.

## Future Improvements

Generated state-machine coverage now supplements the deterministic reference
corpus. The model covers ordinary limit orders, market orders, fills, priority,
expiry, and cancellation, but generated tests deliberately exclude allocation
exhaustion, quantity overflow, persistence corruption, and schema conversion.
Replay still proves repeatability through the production engine itself and is
complementary to both model-based test layers.

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

## Related Documentation

- [Architecture](architecture.md)
- [Matching rules](matching-rules.md)
- [Performance](performance.md)
- [Limitations](limitations.md)
