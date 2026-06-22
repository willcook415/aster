# Architecture

## Purpose

Aster will be a deterministic central limit order book and matching engine.

## Intended Components

TODO: Define module boundaries for order modelling, price levels, order book state, matching, validation, events, replay, errors, tests, and benchmarks.

## Determinism

TODO: Document how engine-controlled sequence numbers will drive priority and replay. Matching must not depend on wall-clock timestamps, randomness, or unordered iteration.

## Data Model Direction

TODO: Introduce finance-safe primitive types such as `OrderId`, `ParticipantId`, `SequenceNumber`, `PriceTicks`, `Quantity`, `Side`, and `OrderType`.

