# AGENTS.md

## Project identity

This workspace is for **Aster**, a Rust-based central limit order book and exchange matching engine.

The goal is to build a serious finance/fintech systems project demonstrating:

* price-time priority matching
* deterministic state transitions
* clean market-domain modelling
* event logs
* replayability
* strong tests
* benchmarks
* clear documentation

This is **not** a trading bot, stock predictor, crypto bot, dashboard-first app, or generic student fintech UI project.

Prioritise correctness, determinism and clean architecture before features.

---

## Core engineering rules

### 1. Build modular first

Keep code modular and easy to reason about.

As a default target, keep individual Rust source files under roughly **300 lines of substantive code**.

This is a design pressure, not an excuse to fragment the codebase badly. If a file grows too large, split it by domain responsibility, not randomly.

Good module boundaries include:

* `order`
* `order_book`
* `price_level`
* `matching`
* `event`
* `replay`
* `validation`
* `errors`
* `types`

Documentation, plans and markdown files may be longer.

### 2. Correctness before cleverness

Do not optimise prematurely.

First make the matching rules correct, deterministic and well-tested. Performance work should come after the behaviour is proven.

### 3. Determinism is a hard invariant

Given the same input sequence, Aster should produce the same:

* accepted/rejected orders
* trades
* emitted events
* remaining book state
* replayed final state

Avoid hidden dependence on wall-clock time, randomness or unordered iteration.

Matching priority should use engine-controlled sequence numbers, not real-time timestamps.

### 4. Use finance-safe primitive modelling

Do not use floating-point numbers for prices.

Prefer explicit domain types such as:

* `OrderId`
* `ParticipantId`
* `SequenceNumber`
* `PriceTicks`
* `Quantity`
* `Side`
* `OrderType`

Prices should be represented as integer ticks.

Quantities should be integer units unless a future requirement explicitly justifies fractional quantity handling.

### 5. Keep public entrypoints stable

Think ahead before changing public APIs.

Do not write code that obviously needs to be replaced in the next milestone. Isolate logic behind clear functions/types so matching, replay, validation and benchmarking can evolve without rewrites.

### 6. No silent fallbacks

During development, avoid default fallbacks that hide broken state.

If something fails, return a clear error or panic only where appropriate for tests/examples.

Do not silently ignore invalid input, missing state, failed parsing, failed cancellation, or impossible matching states.

### 7. Rust error handling

No swallowed errors.

Do not use empty error handlers or vague catch-all behaviour.

Prefer:

* `Result<T, AsterError>` for recoverable domain errors
* explicit rejection events for invalid order commands
* `expect(...)` only in tests/examples where failure means the test setup is wrong
* no casual `unwrap()` in library code

### 8. Dependencies

Do not reinvent the wheel unnecessarily, but keep dependencies minimal.

Before adding a new dependency, justify:

* what problem it solves
* why it is better than a small internal implementation
* whether it is actively maintained
* whether it is suitable for a serious Rust systems project

Prefer well-established Rust crates for serialization, benchmarking and testing where appropriate.

### 9. No UI unless explicitly requested

Do not add dashboards, web apps, WebSockets, charts or frontend code unless the user explicitly asks.

Aster’s first credibility milestone is the core matching engine.

---

## Matching engine scope rules

### Current intended MVP

The first credible MVP should include:

* order model
* bid/ask order book
* best bid / best ask
* price levels
* FIFO within price levels
* limit order matching
* market order matching
* partial fills
* full fills
* resting unfilled limit orders
* cancellation by order ID
* event log
* deterministic replay
* unit tests
* benchmarks
* documentation

### Avoid until after MVP

Do not add these early:

* dashboards
* WebSocket feeds
* real market data
* trading bots
* AI features
* FIX protocol
* database persistence
* multi-symbol exchange layer
* advanced risk engine
* authentication/user accounts

These may become later extensions only after the core engine is correct and documented.

---

## Testing expectations

Every matching rule should be backed by tests.

Important behaviours include:

* invalid orders are rejected
* passive limit orders rest on the correct side
* best bid and best ask update correctly
* crossing limit orders match immediately
* FIFO is preserved within the same price level
* better prices match before worse prices
* partial fills preserve the correct remainder
* fully filled orders leave the book
* market orders can sweep multiple levels
* unfilled market order quantity expires
* cancelled orders cannot later match
* replay produces identical final state

Before completing substantial code changes, run:

```bash
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If benchmarks are touched, also run the relevant benchmark command and record the result or explain why it was not run.

---

## Documentation expectations

Keep documentation honest and specific.

The repo should eventually include:

* `README.md`
* `docs/architecture.md`
* `docs/matching-rules.md`
* `docs/testing-strategy.md`
* `docs/performance.md`
* `docs/limitations.md`

Do not claim the system is production-grade financial infrastructure.

Use the limitations section as a strength: explain clearly what is simplified, omitted or future work.

---

## Continuity Ledger

Maintain a single continuity file for this workspace:

```text
CONTINUITY.md
```

`CONTINUITY.md` is the canonical project briefing designed to survive chat compaction and context loss.

Do not rely on earlier chat/tool output unless the important information is reflected there.

### Operating rule

At the start of each substantial assistant/agent turn:

1. Read `CONTINUITY.md` if it exists.
2. Use it to understand the current goal, constraints, decisions and working set.
3. Update it only when there is a meaningful project delta.

Meaningful deltas include changes to:

* goal or success criteria
* invariants or constraints
* durable decisions
* completed work
* current task
* next task
* open questions
* working set
* important tool outcomes
* important bugs/incidents

Do not update the ledger for tiny mechanical edits.

### Keep it bounded

Keep `CONTINUITY.md` short and high-signal:

* `Snapshot`: max 25 lines
* `Done (recent)`: max 7 bullets
* `Working set`: max 12 paths
* `Receipts`: keep the last 10–20 entries

If sections exceed these caps, compress older items into milestone bullets with pointers to commits, PRs, docs or log paths.

Do not paste raw logs.

### Anti-drift rules

Facts only. No transcripts.

Every durable entry should include:

* a date or ISO timestamp, e.g. `2026-06-22` or `2026-06-22T09:42Z`
* a provenance tag:

  * `[USER]`
  * `[CODE]`
  * `[TOOL]`
  * `[ASSUMPTION]`

If something is unknown, write `UNCONFIRMED`.

Never guess.

If something changes, supersede it explicitly rather than silently rewriting history.

### Decisions

Record durable choices in `Decisions` as ADR-lite entries.

Example:

```text
D001 ACTIVE 2026-06-22 [USER]: Project name is Aster.
D002 ACTIVE 2026-06-22 [USER]: Aster is a standalone Rust repo, not part of Interlinked/Meridian.
D003 ACTIVE 2026-06-22 [USER]: No dashboard/UI until the core matching engine is credible.
```

### Incidents

For recurring bugs or weirdness, create a small incident capsule:

```text
I001 ACTIVE 2026-06-22 [CODE]
Symptoms:
Evidence:
Mitigation:
Status:
```

### Plan tool vs ledger

Use short-term plans for immediate execution steps.

Use `CONTINUITY.md` for long-running continuity: what, why, current state and durable decisions.

Keep them consistent at the intent/progress level.

### Reply behaviour

For substantial work, start replies with a brief ledger snapshot:

```text
Ledger Snapshot:
Goal:
Now:
Next:
Open questions:
```

Print the full ledger only when it materially changed or when the user asks for it.
