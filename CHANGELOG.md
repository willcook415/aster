# Changelog

## Unreleased

- Reject quantity increases in reduction helpers; cache validated level/book totals.
- Reject unknown V1 schema fields. Callers previously sending ignored extension
  fields must remove them or adopt an explicitly supported schema.
- Add opt-in V2 command/event sequences, explicit market expiry and completion.
- Add checksummed, synchronized command journaling and verified process-crash
  recovery, with exclusive ownership and explicit incomplete-tail handling.
- Add custom JSONL and journal CLI commands and runnable examples.
- Expand scaling/cancellation benchmarks and publish a measured cache case study.
- Test Rust 1.89 and stable on Windows/Ubuntu; add extended model checks and API artifacts.
- Declare Rust 1.89 as the supported minimum, replacing the unverified 1.75 claim.
- Select MIT licensing and include its text.

V1 event shapes and matching rules remain available. V1 session exports remain
non-atomic. Read docs/recovery.md for the journal's acknowledgement and failure
contract; this is not a production durability or exactly-once processing claim.
