# Removing repeated quantity scans

Measured 2026-09-29 on Intel Core Ultra 7 265H, Windows 11 Pro 10.0.26200,
Rust/Cargo 1.98.1, release build. Baseline source is commit `062d4a8` plus
the quantity-reduction fix, strict schema decoding, and the new scaling suite;
the comparison changes only aggregate storage in the matching path.

## Hypothesis and change

Each passive insertion recomputed the whole book quantity for capacity checks,
then recomputed its price-level quantity. Adding N orders therefore accumulated
quadratic scanning work. The price map was not the main cause.

`PriceLevel` and `OrderBook` now maintain private aggregate quantities. Input
additions use checked arithmetic before mutation. Fills, removals and cancellations
subtract known quantities. Reduction methods prohibit increases. Invariant tests
sum visible orders independently, and public-mutation tests drain a cloned book
to verify totals. The reference model remains unchanged.

## Method

```sh
cargo bench --locked -p aster-core --bench scaling -- --save-baseline before-cache
# Apply aggregate caching, then pass the full test suite.
cargo bench --locked -p aster-core --bench scaling -- --save-baseline after-cache
python scripts/benchmark_report.py
```

Criterion: 10 samples, 500 ms warm-up, one-second target measurement (longer
when needed). Setup/preloading, input generation and engine destruction are
outside the `iter_batched_ref` timed routine. Processing includes per-command
returned events and retained event history. Sweep throughput counts fills, not
inbound commands. Serialization is a separate workload.

The active desktop was not pinned or power-controlled. Results are local
development evidence, not individual-order p99 latency or production capacity.
The [complete CSV](2026-09-29-cache.csv) includes all 25 workloads, mean estimates
in nanoseconds, and the reported 95% intervals. Raw artifacts remain in
`target/criterion`. The supplied script can regenerate the CSV.

## Selected results

| Workload | Before mean | After mean | Before / after |
|---|---:|---:|---:|
| Insert 1,000 orders, one level | 1.943 ms | 0.362 ms | 5.37x |
| Insert 4,000 orders, one level | 25.133 ms | 1.456 ms | 17.26x |
| Insert 1,000 orders, many levels | 13.182 ms | 0.648 ms | 20.35x |
| Insert 4,000 orders, many levels | 191.338 ms | 2.852 ms | 67.08x |
| Cancel 4,000 orders from head | 1.268 ms | 1.109 ms | 1.14x |
| Cancel 4,000 orders from tail | 19.755 ms | 22.843 ms | 0.86x |
| Cancel 4,000 orders randomly | 15.310 ms | 17.926 ms | 0.85x |
| Serialize 1,000 commands | 0.348 ms | 0.365 ms | 0.95x |

The large insertion improvements agree with the removed scans. Cancellation
still searches a FIFO queue and did not consistently improve; several measured
cases regressed. Small changes in untouched workloads are not attributed to the
cache. More controlled repetitions are needed before making small-effect claims.

## Complexity after the change

Let L be price levels, K orders at the target level, N total resting orders,
and F fills. Hash-index lookup is expected O(1).

| Operation | Main cost |
|---|---|
| Passive insertion | O(log L), plus queue/hash growth allocations |
| Cached level/book quantity | O(1) |
| Best price | BTreeMap boundary traversal, O(log L) bound |
| Market matching | O(F log L), plus event allocation |
| Crossing limit matching | Preflight visits crossed levels; matching as above |
| Cancel by ID | O(log L + K), including queue search/removal |
| Full snapshot | O(N + L) copied state |

The next performance question is cancellation indexing, not more quantity-cache
work. An arena or intrusive queue changes memory and complexity tradeoffs; measure
allocation and crowded-level cancellation before choosing it.
