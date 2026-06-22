# CONTINUITY.md

## Snapshot

- 2026-06-22 [USER]: Goal is Aster, a Rust central limit order book / exchange matching engine project.
- 2026-06-22 [USER]: Current task is focused single-price `PriceLevel` FIFO queue only; full order book, matching, cancellation execution, replay, and persistence must not be implemented yet.
- 2026-06-22 [USER]: First credible MVP should prioritize deterministic matching, event logs, replayability, tests, benchmarks, and docs.
- 2026-06-22 [CODE]: Workspace scaffold contains root Cargo workspace, `aster-core`, `aster-cli`, docs skeletons, README, `.gitignore`, and this ledger.
- 2026-06-22 [CODE]: `aster-core` includes typed integer primitives, order request/accepted order models, small domain errors, and validation helpers.
- 2026-06-22 [CODE]: `aster-core` includes `EngineCommand` and `EngineEvent` skeletons separating input intentions from emitted facts.
- 2026-06-22 [CODE]: `aster-core` includes `PriceLevel`, a FIFO queue for valid resting limit orders at one price.
- 2026-06-22 [ASSUMPTION]: Initial workspace uses Rust 2021 edition and no third-party dependencies.

## Decisions

- D001 ACTIVE 2026-06-22 [USER]: Project name is Aster.
- D002 ACTIVE 2026-06-22 [USER]: Aster is a standalone Rust repo.
- D003 ACTIVE 2026-06-22 [USER]: Core engine comes first; no dashboard/UI yet.
- D004 ACTIVE 2026-06-22 [USER]: Deterministic matching is a hard invariant.
- D005 ACTIVE 2026-06-22 [USER]: Future price representation should use integer ticks, not floating-point prices.
- D006 ACTIVE 2026-06-22 [USER]: Future matching priority should use engine sequence numbers, not wall-clock timestamps.

## Done (recent)

- 2026-06-22 [CODE]: Created initial Cargo workspace with core library and CLI binary crates.
- 2026-06-22 [CODE]: Added documentation scaffold for architecture, matching rules, testing, performance, and limitations.
- 2026-06-22 [CODE]: Added finance-safe domain primitives and order modelling skeletons without order book or matching logic.
- 2026-06-22 [CODE]: Added command/event model skeletons without execution, matching, replay, persistence, or serialization.
- 2026-06-22 [CODE]: Added single-price `PriceLevel` with FIFO, validation for resting limit orders, local removal, and tests; no full book or matching.

## Working set

- 2026-06-22 [CODE]: `Cargo.toml`
- 2026-06-22 [CODE]: `crates/aster-core/src/lib.rs`
- 2026-06-22 [CODE]: `crates/aster-core/src/command.rs`
- 2026-06-22 [CODE]: `crates/aster-core/src/event.rs`
- 2026-06-22 [CODE]: `crates/aster-core/src/price_level.rs`
- 2026-06-22 [CODE]: `crates/aster-core/src/types.rs`
- 2026-06-22 [CODE]: `crates/aster-core/src/order.rs`
- 2026-06-22 [CODE]: `crates/aster-core/src/errors.rs`
- 2026-06-22 [CODE]: `crates/aster-core/src/validation.rs`
- 2026-06-22 [CODE]: `crates/aster-cli/src/main.rs`
- 2026-06-22 [CODE]: `README.md`
- 2026-06-22 [CODE]: `docs/`
- 2026-06-22 [CODE]: `CONTINUITY.md`

## Next

- 2026-06-22 [USER]: Next likely milestone is a minimal bid/ask order book shell for price-level organization before matching logic.

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
