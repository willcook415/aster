# Persistence Design

## Purpose

Aster has versioned schema DTOs and complete-file V1 persistence for finished
sessions. This document defines the implemented boundary and the durability work
that remains.

The core principle is:

```text
Command log = canonical replay input
Event log = audit output
Snapshot = deterministic state summary and checkpoint
```

Persistence should preserve the distinction between what participants asked the engine to do, what the engine actually did, and what the book looked like after processing.

## Persisted Session Records

### Command Log

The command log records inbound deterministic `EngineCommand`s in the exact order received by the engine.

It answers:

```text
What did users/participants ask the engine to do?
```

The command log should become the canonical source for replay. Given the same command log and the same engine version/semantics, Aster should be able to rebuild the same accepted/rejected orders, trades, event sequence, and final book state.

### Event Log

The event log records emitted `EngineEvent`s in the exact order produced by the engine.

It answers:

```text
What did the engine actually do?
```

The event log is an audit trail of engine facts: accepted orders, rejected orders, trades, successful cancellations, and rejected cancellations. It should be derived by processing commands, not treated as the canonical input for rebuilding state.

### Snapshot

A snapshot records deterministic engine and complete visible book state at a
point in time.

It answers:

```text
What did the book look like after processing?
```

Snapshots include summary values plus the complete visible book: bid and ask
levels in matching order, resting orders in FIFO order, and next order-ID and
sequence-number allocation values. They are useful for strong replay comparison
and may later support recovery optimisation. They should not replace command
replay as the source of truth.

## Current State

- Commands are in-memory Rust values.
- Events are in-memory Rust values.
- Replay uses the same fresh-engine path for in-memory and loaded sessions.
- `SessionRecord` assembles commands, emitted events, and a final full snapshot
  in memory and verifies both outputs by replaying the recorded commands.
- The engine event log is in-memory only.
- Versioned schema DTOs exist in `aster-core::schema`.
- The current schema version is `1`.
- Command, event, and snapshot records can round-trip through JSON in memory.
- Snapshot DTO conversion validates visible-book structure, summary values,
  matching/FIFO order, unique identities, uncrossed prices, and allocator
  bounds.
- Completed sessions can be saved and loaded as `commands.jsonl`,
  `events.jsonl`, and `snapshot.json`.
- JSONL parse and schema failures identify the record kind and line number.
- Loaded sessions can be verified by replaying commands and comparing saved
  events and final snapshots.
- There is no live append journal, crash-safe replacement, or recovery service.
- Schema version validation exists for DTO-to-engine conversion.

`SessionRecord` remains an internal domain model. The persistence layer converts
its fields through the existing V1 command, event, and snapshot records rather
than serializing the internal session struct directly.

Snapshot validation rejects obviously inconsistent records before they become
domain snapshots. This makes snapshots safer audit/checkpoint values, but does
not make them authoritative recovery state. Commands remain the canonical replay
input.

## Future Durability Work

- Crash-safe temporary-file and atomic-replacement policy.
- Live append-only command journaling.
- Recovery and checkpoint policy.
- Integrity checks or checksums where justified.
- Explicit schema migrations.

## File Shape

An exported session has this fixed V1 shape:

```text
sessions/<session-name>/
  commands.jsonl
  events.jsonl
  snapshot.json
```

Commands and events use one V1 record per line in exact processing/emission
order. The snapshot is one pretty-printed V1 record. Saving creates the target
directory and deterministically overwrites these three known files.

The schema DTOs are intentionally separate from internal engine structs:

```text
internal engine types
        <-> conversion
versioned schema DTOs
        <-> serde JSON
external saved records
```

The external saved-record layer operates on completed sessions. It does not
append while matching is running.

## Schema And Versioning Principles

- Persisted records should include a schema version.
- Command and event variants should be stable, explicit, and named.
- Prices should remain integer `PriceTicks`.
- Quantities should remain integer `Quantity` units unless a future requirement explicitly justifies fractional handling.
- Floating-point prices should not be used.
- Wall-clock timestamps should not determine matching priority.
- Engine-assigned order IDs and sequence numbers must remain deterministic.
- Migrations should be explicit and testable.
- Unknown or unsupported schema versions should fail clearly rather than falling back silently.
- Snapshot records should fail conversion when their summaries, visible orders,
  ordering, identities, prices, sides, or allocator bounds are structurally
  inconsistent.

## Replay Verification

Session verification replays recorded commands through a fresh engine and
compares the exact emitted events and complete final snapshot. It reports event
and snapshot mismatches separately for both in-memory and loaded sessions.

The file-backed verifier performs:

```text
1. Load command log
2. Replay commands into a fresh engine
3. Capture emitted events
4. Compare replayed events against saved event log
5. Compare final replay snapshot against saved snapshot
```

This flow makes the command log the replay input, the event log the audit output,
and the snapshot a deterministic state checkpoint.

## Write Guarantees

V1 persistence uses simple complete-file writes. It does not use temporary files,
filesystem synchronization, atomic directory swaps, or crash recovery. A process
or machine failure during export may leave a partial session directory; loading
then fails explicitly rather than substituting empty data.

## What Not To Persist Yet

Aster should not persist these at this stage:

- Database rows
- User accounts
- Balances
- Risk limits
- Real exchange protocol messages
- External market data
- Wall-clock matching timestamps
- UI state

Those concerns belong to later exchange-layer, risk, integration, or product milestones after the core matching engine and replay model are proven.
