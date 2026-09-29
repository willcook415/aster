# Limitations

## Current Limitations

Aster is an in-memory, single-instrument matching engine project. It implements
deterministic price-time matching, cancellation, events, full snapshots, replay,
schema DTOs, tests, benchmarks, and a CLI demonstration, but it is not a
complete exchange system.

Current boundaries include:

- One in-memory order book; there is no multi-symbol routing or exchange layer.
- V1 completed sessions use non-atomic complete-file JSON/JSONL writes. The
  optional journal supports verified process-crash recovery, with explicit
  [limits](recovery.md): no exactly-once requests, power-loss guarantee, replication,
  snapshot restoration, or database.
- Validated snapshots remain comparison/checkpoint records; no snapshot loading
  or recovery authority is implemented.
- No networking, FIX, WebSockets, market-data feeds, or external protocol
  integration.
- No balances, positions, settlement, risk limits, user accounts,
  authentication, authorization, or participant onboarding.
- No concurrency model, thread-safety guarantee, latency service level,
  production durability guarantee, or high-availability design.
- No operational controls, regulatory reporting, surveillance, disaster
  recovery, or other production exchange hardening.
- Only limit and market orders are supported. Advanced order instructions and
  venue-specific rules are absent.
- Property-based matching tests use bounded generated command sequences against
  the independent model. Boundary regressions and exhaustive final-frame byte
  truncation tests exist; there is no coverage-guided fuzzing campaign.
- Self-trades are allowed by matching rules 1; no prevention policy is implemented.
- V1 retains its event log in memory. The V2 adapter clears it between commands.
- Queue-position cancellation is linear, including middle/tail removals.
- Benchmarks are focused development workloads, not production capacity or
  latency claims.

## Construction Boundary

`AsterEngine` is the normal authority for order IDs and sequence numbers.
Low-level `AcceptedOrder` construction remains public for explicit order-book
fixtures and schema reconstruction. Code using that low-level boundary is
responsible for identity and sequence uniqueness.

## Production Status

Aster has not been hardened or validated as production financial
infrastructure. Its current value is the explicit, deterministic core and the
ability to inspect and test its simplified rules honestly.

## Related Documentation

- [Architecture](architecture.md)
- [Matching rules](matching-rules.md)
- [Persistence](persistence.md)
- [Testing strategy](testing-strategy.md)
