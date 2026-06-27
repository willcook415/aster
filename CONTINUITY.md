# CONTINUITY.md

## Snapshot

- 2026-06-22 [USER]: Goal is Aster, a Rust central limit order book / exchange matching engine project.
- 2026-06-27 [USER]: Current task strengthens deterministic replay snapshots with complete visible book and allocator state; no persistence or matching-rule changes.
- 2026-06-22 [USER]: First credible MVP should prioritize deterministic matching, event logs, replayability, tests, benchmarks, and docs.
- 2026-06-22 [CODE]: Workspace scaffold contains root Cargo workspace, `aster-core`, `aster-cli`, docs skeletons, README, `.gitignore`, and this ledger.
- 2026-06-22 [CODE]: `aster-core` includes typed integer primitives, order request/accepted order models, small domain errors, and validation helpers.
- 2026-06-22 [CODE]: `aster-core` includes `EngineCommand` and `EngineEvent` skeletons separating input intentions from emitted facts.
- 2026-06-22 [CODE]: `aster-core` includes `PriceLevel`, a FIFO queue for valid resting limit orders at one price.
- 2026-06-22 [CODE]: `aster-core` includes `OrderBook`, a passive single-instrument bid/ask storage shell using deterministic price levels.
- 2026-06-22 [CODE]: `aster-core` includes `AsterEngine`, a deterministic command processor that assigns IDs/sequences, matches limit and market orders, emits trades, rests limit remainders, expires market remainders, and cancels resting orders.
- 2026-06-22 [CODE]: `aster-core` includes in-memory replay from command sequences with deterministic final snapshots.
- 2026-06-22 [CODE]: `AsterEngine` retains an in-memory append-only event log of emitted `EngineEvent`s.
- 2026-06-22 [CODE]: `aster-core` has Criterion benchmarks for passive insertion, crossing matching, market sweeps, cancellation, mixed sessions, and replay.
- 2026-06-22 [CODE]: `aster-cli` runs a deterministic in-memory demo session covering passive liquidity, crossing limit matching, market sweep, successful cancellation, rejected cancellation, final snapshot, and event-log length.
- 2026-06-22 [CODE]: `docs/persistence.md` documents the future boundary: command log as canonical replay input, event log as audit output, snapshot as deterministic state summary.
- 2026-06-22 [CODE]: `aster-core::schema` defines version 1 DTO records for commands, events, and snapshots with conversions to/from internal engine types.
- 2026-06-27 [CODE]: `EngineSnapshot` includes ordered bid/ask levels, FIFO resting orders, complete accepted-order fields, and next allocation values.
- 2026-06-22 [ASSUMPTION]: Initial workspace uses Rust 2021 edition and no third-party dependencies.

## Decisions

- D001 ACTIVE 2026-06-22 [USER]: Project name is Aster.
- D002 ACTIVE 2026-06-22 [USER]: Aster is a standalone Rust repo.
- D003 ACTIVE 2026-06-22 [USER]: Core engine comes first; no dashboard/UI yet.
- D004 ACTIVE 2026-06-22 [USER]: Deterministic matching is a hard invariant.
- D005 ACTIVE 2026-06-22 [USER]: Future price representation should use integer ticks, not floating-point prices.
- D006 ACTIVE 2026-06-22 [USER]: Future matching priority should use engine sequence numbers, not wall-clock timestamps.

## Done (recent)

- 2026-06-22 [CODE]: Created workspace, docs scaffold, domain primitives, command/event models, FIFO `PriceLevel`, and passive bid/ask `OrderBook`.
- 2026-06-22 [CODE]: Implemented `AsterEngine` with deterministic limit matching, market execution, cancellation, partial/full fills, and resting limit remainders.
- 2026-06-22 [CODE]: Added in-memory replay, `EngineSnapshot`, and append-only in-memory event log; no persistence, serialization, command journaling, or event-log replay.
- 2026-06-22 [CODE]: Added Criterion benchmark target for in-memory matching and replay workloads; no runtime benchmark dependency.
- 2026-06-22 [CODE]: Replaced placeholder CLI with deterministic demo output and documented `cargo run -p aster-cli`.
- 2026-06-22 [CODE]: Added persistence design documentation without implementing serialization or file IO.
- 2026-06-27 [CODE]: Strengthened engine/schema snapshots and replay tests to compare complete deterministic visible state; no file IO or matching changes.

## Working set

- 2026-06-22 [CODE]: `crates/aster-core/src/`
- 2026-06-22 [CODE]: `crates/aster-core/tests/`
- 2026-06-22 [CODE]: `crates/aster-core/src/schema.rs`
- 2026-06-22 [CODE]: `crates/aster-core/src/schema/`
- 2026-06-22 [CODE]: `crates/aster-core/tests/schema_tests.rs`
- 2026-06-22 [CODE]: `crates/aster-core/benches/engine_benchmarks.rs`
- 2026-06-22 [CODE]: `crates/aster-cli/src/main.rs`
- 2026-06-22 [CODE]: `README.md`
- 2026-06-22 [CODE]: `docs/architecture.md`
- 2026-06-22 [CODE]: `docs/persistence.md`
- 2026-06-22 [CODE]: `CONTINUITY.md`

## Next

- 2026-06-27 [USER]: Full-state replay verification milestone completed; next development direction is UNCONFIRMED.

## Open questions

- 2026-06-22 [USER]: UNCONFIRMED final public repository URL.
- 2026-06-22 [USER]: UNCONFIRMED license choice beyond placeholder workspace metadata.

## Receipts

- 2026-06-22 [TOOL]: Initial repository inspection found `README.md` and `AGENTS.md`; no existing Rust workspace.
- 2026-06-22 [TOOL]: `cargo fmt` completed successfully.
- 2026-06-22 [TOOL]: `cargo clippy --workspace --all-targets -- -D warnings` completed successfully.
- 2026-06-22 [TOOL]: `cargo test --workspace` completed successfully; 1 core smoke test passed.
- 2026-06-22 [TOOL]: After domain primitives milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; 9 core tests passed.
- 2026-06-22 [TOOL]: After command/event skeleton milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; 16 core tests passed.
- 2026-06-22 [TOOL]: After `PriceLevel` milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; 28 core tests passed.
- 2026-06-22 [TOOL]: After `OrderBook` storage milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; 40 core tests passed.
- 2026-06-22 [TOOL]: After `AsterEngine` acceptance milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; 52 core tests passed.
- 2026-06-22 [TOOL]: After limit-order matching milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; 59 core tests passed.
- 2026-06-22 [TOOL]: After market order execution milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; 65 core tests passed.
- 2026-06-22 [TOOL]: After cancellation execution milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; 76 core tests passed.
- 2026-06-22 [TOOL]: After in-memory replay milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; 83 core tests passed.
- 2026-06-22 [TOOL]: After in-engine event log milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; 93 core tests passed.
- 2026-06-22 [TOOL]: After benchmark milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo bench -p aster-core` completed successfully; 93 core tests passed and 7 Criterion workloads executed.
- 2026-06-22 [TOOL]: After persistence-design doc milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; no benchmark code changed.
- 2026-06-22 [TOOL]: After schema DTO milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo bench -p aster-core --no-run` completed successfully; 14 schema integration tests added.
- 2026-06-27 [TOOL]: Full-state snapshots passed `cargo fmt`, clippy with warnings denied, 110 workspace tests, and benchmark compilation.
