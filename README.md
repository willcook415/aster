# Aster

Aster is a Rust-based central limit order book and exchange matching engine project. The project is intended to demonstrate deterministic price-time priority matching, clean market-domain modelling, event logs, replayability, strong tests, benchmarks, and honest documentation.

## What This Is

Aster is a core matching-engine project. Its first milestone is a small, deterministic engine that can accept order commands, produce explicit events, maintain bid and ask books, and replay the same input sequence to the same final state.

## What This Is Not

Aster is not a trading bot, stock predictor, crypto bot, dashboard-first application, market-data product, or generic fintech UI project. Dashboards, WebSockets, persistence, authentication, FIX, and advanced exchange layers are intentionally out of scope until the core engine is credible.

## Implemented Core

The current in-memory core includes:

- Order model
- Bid and ask order book
- Best bid and best ask
- Price levels
- FIFO within price levels
- Limit order matching
- Market order matching
- Partial and full fills
- Resting unfilled limit orders
- Cancellation by order ID
- Ordered in-memory event log of emitted facts
- Deterministic command replay
- Full deterministic snapshots of visible book and allocator state
- Versioned schema DTOs with in-memory JSON round-trip tests
- Unit, integration, replay, schema, and invariant-heavy tests
- Focused Criterion benchmarks

## Repository Layout

```text
.
|-- Cargo.toml
|-- crates/
|   |-- aster-core/
|   |   `-- src/lib.rs
|   `-- aster-cli/
|       `-- src/main.rs
|-- docs/
|   |-- architecture.md
|   |-- matching-rules.md
|   |-- testing-strategy.md
|   |-- performance.md
|   |-- persistence.md
|   `-- limitations.md
|-- AGENTS.md
|-- CONTINUITY.md
`-- README.md
```

## Running Checks

```bash
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Running The Scenario Runner

```bash
cargo run -p aster-cli
```

With no arguments, the CLI runs the deterministic `mixed-session` scenario.
List all built-in scenarios with:

```bash
cargo run -p aster-cli -- list
```

Run a named scenario with:

```bash
cargo run -p aster-cli -- scenario fifo-partial-fill
cargo run -p aster-cli -- scenario market-sweep
cargo run -p aster-cli -- scenario cancellation
cargo run -p aster-cli -- scenario mixed-session
```

Each report shows commands in input order, events in emission order, bid and ask
levels in matching order, resting-order details, allocator state, and replay
verification for both events and the final full snapshot. These are built-in
in-memory demonstrations; the CLI does not read or write session files.

## Running Benchmarks

```bash
cargo bench -p aster-core
```

## Persistence Direction

File persistence is not implemented yet. The planned boundary is documented in [docs/persistence.md](docs/persistence.md): command logs are canonical replay input, event logs are audit output, and snapshots are deterministic state summaries. Versioned schema DTOs now support in-memory JSON round trips.

## Current Status

Aster has an in-memory matching core with limit and market orders, cancellation,
explicit events, full snapshots, command replay, checked allocation and quantity
boundaries, focused benchmarks, and a deterministic CLI demo. Versioned command,
event, and snapshot DTOs can be converted to and from JSON in memory.

There is no file I/O, JSONL session format, durable command or event log,
database, networking, or production exchange hardening. Aster should not be
treated as production-grade financial infrastructure.
