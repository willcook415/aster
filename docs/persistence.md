# Persistence Design

## Purpose

Aster has versioned schema DTOs for commands, events, and snapshots. It does not implement file persistence yet. This document defines the intended persistence boundaries so future work can add durable records without weakening deterministic matching.

The core principle is:

```text
Command log = canonical replay input
Event log = audit output
Snapshot = deterministic state summary and checkpoint
```

Persistence should preserve the distinction between what participants asked the engine to do, what the engine actually did, and what the book looked like after processing.

## What To Persist Later

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
- Replay is in-memory only.
- `SessionRecord` assembles commands, emitted events, and a final full snapshot
  in memory and verifies both outputs by replaying the recorded commands.
- The engine event log is in-memory only.
- Versioned schema DTOs exist in `aster-core::schema`.
- The current schema version is `1`.
- Command, event, and snapshot records can round-trip through JSON in memory.
- Snapshot DTO conversion validates visible-book structure, summary values,
  matching/FIFO order, unique identities, uncrossed prices, and allocator
  bounds.
- There is no file IO.
- There are no JSONL session files.
- There is no durable command journal or recovery workflow.
- Schema version validation exists for DTO-to-engine conversion.

The session record is an internal domain model, not a persisted session DTO. It
does not perform file I/O and does not define a JSON or JSONL session format.

Snapshot validation rejects obviously inconsistent records before they become
domain snapshots. This makes snapshots safer audit/checkpoint values, but does
not make them authoritative recovery state. Commands remain the canonical replay
input.

## Future State

- Durable command logs.
- Durable event logs.
- Optional durable snapshots.
- Deterministic replay from saved command logs.
- Verification that replayed events match saved event logs.
- Verification that final replay snapshots match saved snapshots.
- Explicit schema migrations.

## Possible File Shapes

A future file-backed session might look like this:

```text
sessions/
  session-001.commands.jsonl
  session-001.events.jsonl
  session-001.snapshot.json
```

JSONL is a likely fit for append-only command and event logs because each command or event can be recorded as one line. That shape would allow streaming reads, partial inspection, and straightforward append behaviour. This is a design direction, not a permanent commitment; future benchmark, tooling, and audit requirements may justify a different durable format.

The schema DTOs are intentionally separate from internal engine structs:

```text
internal engine types
        <-> conversion
versioned schema DTOs
        <-> serde JSON
external saved records later
```

Only the middle boundary exists today. The external saved-record layer is still future work.

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

Current in-memory session verification replays recorded commands through a
fresh engine and compares the exact emitted events and complete final snapshot.
It reports event and snapshot mismatches separately.

The in-memory replay tests also exercise the schema boundary by converting
command records through JSON strings before replay. A future file-backed
persistence verifier should perform the same logical checks after loading
durable records:

```text
1. Load command log
2. Replay commands into a fresh engine
3. Capture emitted events
4. Compare replayed events against saved event log
5. Compare final replay snapshot against saved snapshot
```

This flow would make the command log the replay input, the event log the audit output, and the snapshot a deterministic state checkpoint. It is future work and is not implemented yet.

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
