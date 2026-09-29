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

## 2026-09-29: licensing

The owner selected MIT only. Workspace metadata and the LICENSE file use MIT.

## 2026-09-29: audit V2 and recovery dependencies

V2 audit envelopes add command/event correlation, explicit market expiry and a
command-completed boundary while preserving V1 matching/event replay. The
matching-rules version remains 1 and is distinct from audit schema version 2.
The sequenced adapter clears its internal event history after each command;
callers own the returned audit batches.

Recovery uses SHA-256 from RustCrypto's `sha2` 0.10 crate, whose maintained
implementation avoids writing a homegrown checksum. It detects accidental frame
corruption and chains record order; it is not authentication. File locking uses
the standard library (stable since Rust 1.89), avoiding another OS wrapper.
The supported minimum is therefore explicitly Rust 1.89 for the whole workspace.

## Test and CLI dependencies

`tempfile` is a core test-only dependency for isolated journal files with reliable
cleanup. The CLI uses the already-present `serde_json` package to emit V2 batches;
it adds no new serialization format.
