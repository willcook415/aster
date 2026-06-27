# Architecture

## Purpose

Aster will be a deterministic central limit order book and matching engine.

## Intended Components

Aster is organized around explicit domain modules for order modelling, price levels, order book state, command processing, events, replay, validation, errors, tests, and future benchmarks.

## Determinism

Engine-controlled order IDs and sequence numbers drive deterministic state transitions. Matching must not depend on wall-clock timestamps, randomness, or unordered iteration.

In-memory replay currently runs a command sequence through a fresh `AsterEngine`,
collects emitted events, and compares a deterministic final `EngineSnapshot`.
The snapshot includes complete visible bid and ask state in matching order, FIFO
resting orders within each level, and the next engine allocation values. There
is no file persistence yet.

`AsterEngine` also retains an in-memory append-only event log of emitted facts. The log records output events, not inbound command intentions. File persistence, serialization, and command journaling are not implemented yet.

Future persistence boundaries are documented in `docs/persistence.md`: command logs are canonical replay input, event logs are audit output, and snapshots are deterministic state summaries.

## Data Model Direction

Aster uses finance-safe primitive types such as `OrderId`, `ParticipantId`, `SequenceNumber`, `PriceTicks`, `Quantity`, `Side`, and `OrderType`.
