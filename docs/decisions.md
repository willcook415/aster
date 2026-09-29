# Engineering decisions

## 2026-09-29: public quantity reduction

Reduction methods reject increases with `InvalidOrderState` before mutation.
Equal quantities are successful no-ops. Quantity increases require a future
amend/replace command with an explicit priority policy. Existing method names
and signatures remain available.

## 2026-09-29: strict V1 decoding

All V1 record envelopes, structured variants and nested DTOs reject unknown
fields. Previously ignored fields now fail deserialization. This intentionally
tightens input validation: an unsupported execution instruction must never be
silently discarded. Existing valid V1 fixtures and event semantics are preserved.
Future extensions require a supported version, not unrecognised metadata fields.

## 2026-09-29: self-trade policy

V1 matching permits self-trades. Participant identity controls cancellation
ownership, not matching eligibility. This keeps existing replay semantics stable.
Self-trade prevention would be a separately versioned policy, not an implicit
change to historical sessions.
