# CONTINUITY.md

## Snapshot

- 2026-06-22 [USER]: Goal is Aster, a Rust central limit order book / exchange matching engine project.
- 2026-06-29 [USER]: Current task improves repository portfolio readiness, documentation navigation, and crate entry docs without runtime changes.
- 2026-06-22 [USER]: First credible MVP should prioritize deterministic matching, event logs, replayability, tests, benchmarks, and docs.
- 2026-06-22 [CODE]: `aster-core` includes typed integer primitives, order request/accepted order models, small domain errors, and validation helpers.
- 2026-06-22 [CODE]: `aster-core` includes `EngineCommand` and `EngineEvent` skeletons separating input intentions from emitted facts.
- 2026-06-22 [CODE]: `aster-core` includes `PriceLevel`, a FIFO queue for valid resting limit orders at one price.
- 2026-06-22 [CODE]: `aster-core` includes `OrderBook`, a passive single-instrument bid/ask storage shell using deterministic price levels.
- 2026-06-22 [CODE]: `aster-core` includes `AsterEngine`, a deterministic command processor that assigns IDs/sequences, matches limit and market orders, emits trades, rests limit remainders, expires market remainders, and cancels resting orders.
- 2026-06-22 [CODE]: `aster-core` includes in-memory replay from command sequences with deterministic final snapshots.
- 2026-06-22 [CODE]: `AsterEngine` retains an in-memory append-only event log of emitted `EngineEvent`s.
- 2026-06-22 [CODE]: `aster-core` has Criterion benchmarks for passive insertion, crossing matching, market sweeps, cancellation, mixed sessions, and replay.
- 2026-06-27 [CODE]: `EngineSnapshot` includes ordered bid/ask levels, FIFO resting orders, complete accepted-order fields, and next allocation values.
- 2026-06-27 [CODE]: Mixed-session invariant tests validate uncrossed books, ordered non-empty levels, FIFO, resting-order integrity, unique IDs, cancellation, replay, event logs, and explicit quantity accounting.
- 2026-06-27 [CODE]: Allocation exhaustion and resting-quantity overflow fail explicitly without wrapping; obsolete unimplemented-feature errors were removed.
- 2026-06-27 [CODE]: Nine golden V1 fixtures lock command, event, and full-snapshot JSON shapes using compile-time test assets.
- 2026-06-27 [CODE]: `SessionRecord` captures ordered commands, emitted events, and the final full snapshot; verification distinguishes event and snapshot mismatches.
- 2026-06-27 [CODE]: `aster-cli` provides four named scenarios with ordered command/event reports, full book output, and session verification status.
- 2026-06-29 [CODE]: Limit submissions preflight projected post-match resting quantity so capacity rejection occurs before book mutation, event emission, or allocator advancement.
- 2026-06-29 [CODE]: A test-only `Vec`-based reference model independently checks exact events and full snapshots across 11 deterministic sequences and 100 commands.
- 2026-06-29 [CODE]: V1 snapshot conversion rejects structurally inconsistent summaries, levels, orders, identities, prices, FIFO order, crossed books, and allocator bounds.
- 2026-06-29 [CODE]: Completed sessions save/load as V1 command/event JSONL plus snapshot JSON and verify through fresh-engine replay.
- 2026-06-29 [CODE]: README and rustdoc now provide cold-start architecture, matching, replay, persistence, quality-gate, and limitations guidance.
- 2026-06-22 [ASSUMPTION]: Initial workspace uses Rust 2021 edition and no third-party dependencies.

## Decisions

- D001 ACTIVE 2026-06-22 [USER]: Project name is Aster.
- D002 ACTIVE 2026-06-22 [USER]: Aster is a standalone Rust repo.
- D003 ACTIVE 2026-06-22 [USER]: Core engine comes first; no dashboard/UI yet.
- D004 ACTIVE 2026-06-22 [USER]: Deterministic matching is a hard invariant.
- D005 ACTIVE 2026-06-22 [USER]: Future price representation should use integer ticks, not floating-point prices.
- D006 ACTIVE 2026-06-22 [USER]: Future matching priority should use engine sequence numbers, not wall-clock timestamps.
- D007 ACTIVE 2026-06-29 [USER]: Persisted commands are canonical replay input; saved events and snapshots are audit/verification outputs.

## Done (recent)

- 2026-06-27 [CODE]: Added deterministic full-snapshot invariant and accounting coverage for complex command sequences.
- 2026-06-27 [CODE]: Strengthened engine/schema snapshots and replay tests to compare complete deterministic visible state; no file IO or matching changes.
- 2026-06-29 [CODE]: Reproduced and fixed late-overflow partial mutation; focused tests lock snapshot, event-log, allocator, liquidity, and successful matching behavior.
- 2026-06-29 [CODE]: Added independent deterministic model comparison for matching, priority, fills, expiry, cancellation, exact events, and complete visible state; no production bug found.
- 2026-06-29 [CODE]: Hardened snapshot DTO-to-domain conversion with focused malformed-record tests while keeping command logs canonical and snapshots non-authoritative.
- 2026-06-29 [CODE]: Added V1 completed-session persistence, line-aware/missing-file errors, replay verification, focused tests, and CLI export/verify.
- 2026-06-29 [CODE]: Reworked repository landing/docs navigation and crate-level documentation for portfolio evaluation; no runtime behavior changed.

## Working set

- 2026-06-22 [CODE]: `crates/aster-core/src/`
- 2026-06-22 [CODE]: `crates/aster-core/tests/`
- 2026-06-22 [CODE]: `crates/aster-core/src/schema.rs`
- 2026-06-22 [CODE]: `crates/aster-core/src/schema/`
- 2026-06-29 [CODE]: `crates/aster-core/tests/snapshot_validation_tests.rs`
- 2026-06-29 [CODE]: `crates/aster-core/tests/reference_model_tests.rs`
- 2026-06-29 [CODE]: `crates/aster-core/src/persistence.rs`
- 2026-06-27 [CODE]: `crates/aster-cli/src/`
- 2026-06-22 [CODE]: `README.md`
- 2026-06-22 [CODE]: `docs/architecture.md`
- 2026-06-22 [CODE]: `docs/persistence.md`
- 2026-06-22 [CODE]: `CONTINUITY.md`

## Next

- 2026-06-27 [USER]: Deterministic CLI scenario-runner milestone completed; next development direction is UNCONFIRMED.
- 2026-06-29 [USER]: Rejected-submission atomicity hardening completed; subsequent development direction remains UNCONFIRMED.
- 2026-06-29 [USER]: Independent deterministic model-testing milestone completed; subsequent development direction remains UNCONFIRMED.
- 2026-06-29 [USER]: Snapshot/schema boundary hardening completed; durable JSONL persistence remains UNCONFIRMED.
- 2026-06-29 [USER]: Completed-session JSONL persistence milestone implemented; live journaling and crash-safe durability remain future work.
- 2026-06-29 [USER]: Portfolio-readiness documentation pass completed; next technical milestone remains UNCONFIRMED.

## Open questions

- 2026-06-22 [USER]: UNCONFIRMED final public repository URL.
- 2026-06-22 [USER]: UNCONFIRMED license choice beyond placeholder workspace metadata.

## Receipts

- 2026-06-22 [TOOL]: After limit-order matching milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; 59 core tests passed.
- 2026-06-22 [TOOL]: After market order execution milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; 65 core tests passed.
- 2026-06-22 [TOOL]: After cancellation execution milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; 76 core tests passed.
- 2026-06-22 [TOOL]: After in-memory replay milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; 83 core tests passed.
- 2026-06-22 [TOOL]: After in-engine event log milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; 93 core tests passed.
- 2026-06-22 [TOOL]: After benchmark milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo bench -p aster-core` completed successfully; 93 core tests passed and 7 Criterion workloads executed.
- 2026-06-22 [TOOL]: After persistence-design doc milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` completed successfully; no benchmark code changed.
- 2026-06-22 [TOOL]: After schema DTO milestone, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo bench -p aster-core --no-run` completed successfully; 14 schema integration tests added.
- 2026-06-27 [TOOL]: Full-state snapshots passed `cargo fmt`, clippy with warnings denied, 110 workspace tests, and benchmark compilation.
- 2026-06-27 [TOOL]: Invariant hardening passed `cargo fmt`, clippy with warnings denied, all 114 workspace tests, benchmark compilation, and `git diff --check`.
- 2026-06-27 [TOOL]: Boundary cleanup passed `cargo fmt`, clippy with warnings denied, all 120 workspace tests, benchmark compilation, and `git diff --check`.
- 2026-06-27 [TOOL]: Documentation cleanup passed format check, clippy with warnings denied, all 120 workspace tests, benchmark compilation, and `git diff --check`.
- 2026-06-27 [TOOL]: Golden V1 fixtures passed `cargo fmt`, clippy with warnings denied, all 127 workspace tests, benchmark compilation, and `git diff --check`.
- 2026-06-27 [TOOL]: In-memory sessions passed `cargo fmt`, clippy with warnings denied, all 134 workspace tests, benchmark compilation, and `git diff --check`.
- 2026-06-27 [TOOL]: CLI scenario runner passed `cargo fmt`, strict clippy, all 141 workspace tests, benchmark compilation, all four scenario runs, listing, and `git diff --check`.
- 2026-06-29 [TOOL]: Atomicity hardening passed `cargo fmt --check`, strict clippy, all 146 workspace tests, benchmark compilation, and default/list/all four CLI scenario runs.
- 2026-06-29 [TOOL]: Independent model testing passed formatting, strict clippy, all 147 workspace tests, benchmark compilation, and default/list/all four CLI scenario runs.
- 2026-06-29 [TOOL]: Snapshot/schema hardening passed formatting, strict clippy, all 158 workspace tests, benchmark compilation, and default/list/all four CLI scenario runs.
- 2026-06-29 [TOOL]: V1 completed-session persistence passed formatting, strict clippy, all 170 workspace tests, benchmark compilation, existing CLI smokes, and export/verify for all four scenarios.
- 2026-06-29 [TOOL]: Portfolio-readiness pass passed formatting, strict clippy, 170 unit/integration tests plus rustdoc example, benchmark compilation, workspace docs, CLI smokes, and export/verify.
