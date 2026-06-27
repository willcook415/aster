# Limitations

## Current Limitations

Aster is an in-memory, single-instrument matching engine project. It implements
deterministic price-time matching, cancellation, events, full snapshots, replay,
schema DTOs, tests, benchmarks, and a CLI demonstration, but it is not a
complete exchange system.

Current boundaries include:

- One in-memory order book; there is no multi-symbol routing or exchange layer.
- No file persistence, JSONL sessions, durable command/event logs, recovery
  process, or database.
- No networking, FIX, WebSockets, market-data feeds, or external protocol
  integration.
- No balances, positions, settlement, risk limits, user accounts,
  authentication, authorization, or participant onboarding.
- No concurrency model, thread-safety guarantee, latency service level,
  durability guarantee, or high-availability design.
- No operational controls, regulatory reporting, surveillance, disaster
  recovery, or other production exchange hardening.
- Only limit and market orders are supported. Advanced order instructions and
  venue-specific rules are absent.
- Property-based and independent model-based testing are not implemented.
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
