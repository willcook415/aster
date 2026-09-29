# Working on Aster

Use Rust 1.89 or newer. CI tests 1.89.0 and stable on Windows and Ubuntu.
The minimum includes the contributor dependency graph and standard-library file
locking used by recovery. The repository uses MIT licensing.

```sh
cargo test --workspace --locked
cargo fmt --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo run --locked -p aster-cli -- run examples/fifo.jsonl
```

Generated documentation starts at `target/doc/aster_core/index.html`. CI uploads
the same docs as an `aster-api-docs` artifact. No hosted docs URL is assumed.

Property tests retain failing seeds under `proptest-regressions`; commit minimal
regressions with fixes. `ASTER_PROPTEST_CASES` controls the reference-model case
count (default 64). The weekly workflow runs 4,096 cases. Quantity overflow,
schema, and recovery have separate boundary tests. Run release regressions too:

```sh
cargo test --release --locked -p aster-core --test quantity_contract_tests --test aggregate_tests
```

Before optimising, capture the scaling suite and retain its Criterion baseline:

```sh
cargo bench --locked -p aster-core --bench scaling -- --save-baseline before
# Apply a measured change, rerun correctness checks, then:
cargo bench --locked -p aster-core --bench scaling -- --baseline before
```

Record CPU, OS, toolchain, workload shape and timing boundaries. CI compiles
benchmarks but does not make performance pass/fail decisions on shared runners.
Do not call batch durations individual-order tail latency.
