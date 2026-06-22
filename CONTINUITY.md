# CONTINUITY.md

## Snapshot

- 2026-06-22 [USER]: Goal is Aster, a Rust central limit order book / exchange matching engine project.
- 2026-06-22 [USER]: Current task is initial workspace scaffold only; matching engine logic must not be implemented yet.
- 2026-06-22 [USER]: First credible MVP should prioritize deterministic matching, event logs, replayability, tests, benchmarks, and docs.
- 2026-06-22 [CODE]: Workspace scaffold contains root Cargo workspace, `aster-core`, `aster-cli`, docs skeletons, README, `.gitignore`, and this ledger.
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

## Working set

- 2026-06-22 [CODE]: `Cargo.toml`
- 2026-06-22 [CODE]: `crates/aster-core/src/lib.rs`
- 2026-06-22 [CODE]: `crates/aster-cli/src/main.rs`
- 2026-06-22 [CODE]: `README.md`
- 2026-06-22 [CODE]: `docs/`
- 2026-06-22 [CODE]: `CONTINUITY.md`

## Next

- 2026-06-22 [USER]: Next likely milestone is domain primitive modelling and validation skeletons before matching logic.

## Open questions

- 2026-06-22 [USER]: UNCONFIRMED final public repository URL.
- 2026-06-22 [USER]: UNCONFIRMED license choice beyond placeholder workspace metadata.

## Receipts

- 2026-06-22 [TOOL]: Initial repository inspection found `README.md` and `AGENTS.md`; no existing Rust workspace.
- 2026-06-22 [TOOL]: `cargo fmt` completed successfully.
- 2026-06-22 [TOOL]: `cargo clippy --workspace --all-targets -- -D warnings` completed successfully.
- 2026-06-22 [TOOL]: `cargo test --workspace` completed successfully; 1 core smoke test passed.
