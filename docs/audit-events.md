# V2 audit events

V1 engine events and their existing golden fixtures remain available. Opt into
`SequencedEngine` or `aster-cli run-v2 <commands.jsonl>` for audit schema 2.
Inputs are strict V1 command records; output is one V2 batch per JSONL line.

Every batch contains `schema_version: 2`, `matching_rules_version: 1`, its
`command_sequence`, the command DTO, and ordered events. Every event has a global
`event_sequence` and the same `command_sequence`. Both counters start at 1;
rejected commands consume audit sequences without consuming matching priority.

Existing facts are wrapped in `engine_event`. An accepted market order with
unfilled quantity produces `order_expired` with order ID, remaining quantity,
and reason `insufficient_liquidity`, after its trades. Every command ends with
`command_completed`, including rejections. A filled market order has no expiry.
Sequence capacity is checked before mutation using a conservative upper bound.

`verify_batch` rejects any mismatch against fresh deterministic processing,
including event order, sequence, completion, and matching-rule version. It only
installs the replayed state if the entire batch agrees. V2 is an additive API;
there is no automatic V1-to-V2 persistence migration.

The adapter clears the underlying V1 event history after each command, avoiding
unbounded cumulative log retention. It retains the current book and counters;
callers own returned batches. A single command may still produce many events.

Matching rules 1 explicitly allow self-trades. Participant IDs provide cancellation
ownership checks, not self-trade prevention or account-level risk controls.
Unknown fields are rejected in V1/V2 DTOs so unsupported instructions such as
`time_in_force` cannot be silently ignored.
