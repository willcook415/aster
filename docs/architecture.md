# Architecture

## Purpose

Aster is a deterministic, single-instrument central limit order book and
matching engine implemented as an in-memory Rust core.

## Components

Aster is organized around explicit modules for typed domain values, order
modelling, FIFO price levels, bid/ask book state, command processing, matching,
events, replay, validation, errors, and versioned schema DTOs. `AsterEngine` is
the high-level command-processing entry point. The CLI crate provides a
deterministic demonstration rather than an interactive exchange interface.

The matching core supports limit and market orders, partial and full fills,
resting limit remainders, market remainder expiry, and owned cancellation of
resting orders.

## Determinism

`AsterEngine` assigns order IDs and priority sequence numbers. Price selection
uses ordered price maps, and FIFO within a level uses insertion order. Matching
does not depend on wall-clock timestamps, randomness, or unordered map
iteration.

In-memory replay currently runs a command sequence through a fresh `AsterEngine`,
collects emitted events, and compares a deterministic final `EngineSnapshot`.
The snapshot includes complete visible bid and ask state in matching order, FIFO
resting orders within each level, and the next engine allocation values. There
is also completed-session persistence that loads commands into the same fresh
replay path and verifies saved events and snapshots.

`AsterEngine` retains an in-memory event log of emitted facts in processing
order. The log records output events, not inbound command intentions.

## Schema Boundary

`aster-core::schema` defines version 1 DTOs for commands, events, accepted
orders, and snapshots. Conversion and JSON round trips are tested in memory.
Malformed DTO values and unsupported schema versions return explicit
`AsterError`s. Snapshot conversion also rejects structurally impossible visible
state, including inconsistent summaries, side or price mismatches, duplicate
identities, invalid level/FIFO order, crossed books, and allocator state that
does not follow visible orders.

The persistence module writes completed sessions as V1 command/event JSONL and
snapshot JSON. It is not a live command journal, crash-safe storage layer,
database, recovery service, or schema migration system. Validated snapshots are
deterministic summaries and checkpoints, not canonical recovery input. The
persistence boundaries are documented in `docs/persistence.md`.

## Data Model

Aster uses typed values including `OrderId`, `ParticipantId`, `SequenceNumber`,
`PriceTicks`, `Quantity`, `Side`, and `OrderType`. Prices and quantities are
positive integers rather than floating-point values.
