# Command journal and recovery

The optional `CommandJournal` API adds single-writer process-crash recovery.
Matching remains deterministic and synchronous. The V1 three-file export remains
a separate completed-session format, with no atomic replacement guarantee.

```bash
cargo run --locked -p aster-cli -- journal examples/fifo.jsonl target/demo.aster
cargo run --locked -p aster-cli -- recover target/demo.aster
cargo run --locked -p aster-cli -- resume examples/market-expiry.jsonl target/demo.aster
```

`journal` creates a new file and refuses to overwrite. `recover` verifies without
changing bytes. `resume` acquires the file lock, verifies the entire committed
prefix, removes an incomplete final frame if present, and appends commands.
Inputs are fully parsed before opening the journal. A failure during a multi-command
run can leave a committed prefix; the run is not one transaction.

## Acknowledgement contract

For each append, state is staged on a clone. The journal writes the command and
its complete V2 event batch, then calls `File::sync_all`. Only after success does
it install staged state and return a successful response. A write or sync error
poisons that handle: close it and recover before doing more work.

A process can terminate after writing a complete frame but before returning its
response. Recovery accepts that frame if its checksum and deterministic replay
match. Callers must inspect recovered command sequences before retrying an
uncertain request. This is **not exactly-once request processing**; there is no
client request-ID deduplication protocol.

The tested boundary is process termination and torn final writes on local files.
This does not promise power-loss durability across hardware/filesystem failures,
network filesystems, or directory-entry loss during file creation. Parent
directories are not fsynced. The OS file lock is cooperative single-writer
coordination, not an access-control or malicious-tampering boundary.

## Format 1

The 16-byte header is `ASTERJNL`, journal version 1 (u32 little-endian), audit
schema 2 (u16 little-endian), and matching rules 1 (u16 little-endian).
Each frame contains:

1. Payload length and its bitwise complement (two u32 little-endian values).
2. UTF-8 JSON of one `CommandBatchV2`, at most 16 MiB.
3. SHA-256 of previous frame digest, both length fields, and payload (32 bytes).
4. The eight-byte marker `ASTRCMIT`.

The first previous digest is 32 zero bytes. No full header, unknown versions,
invalid lengths, invalid complete markers, mismatched hashes, skipped/reordered
sequences, or replay differences are accepted. Complete corrupt frames fail
without modifying the file. Only a validly sized but incomplete final frame (or
fewer than eight final header bytes) is treated as a torn tail. Removing an entire
suffix is indistinguishable from an earlier valid journal; no external trusted
checkpoint is available. Hashes detect corruption, not authenticated tampering.

## Scope and evidence

Tests cover every byte truncation of a final frame, corrupted complete frame
components, frame removal/reordering, lock exclusion, no-overwrite creation,
restart and continued sequences, process exit without destructors, and injected
partial-write and synchronization failures. V2 tests additionally reject altered
event sequences, missing completion events, and unsupported matching rules.

Recovery replays from the beginning; snapshots are not restoration authorities.
Each append stages a full engine clone, and each replay batch also verifies on
a clone. These correctness-first costs and a sync per command make this unsuitable
as a low-latency durability claim. No group commit, segmentation, replication,
snapshot compaction, format migration, or storage performance measurements exist.
