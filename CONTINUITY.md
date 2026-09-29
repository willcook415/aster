# CONTINUITY.md

## Snapshot

- 2026-06-22 [USER]: Aster is a standalone Rust central limit order book / exchange matching engine; correctness and deterministic integer matching come first.
- 2026-09-29 [USER]: Supersedes audit-only task: implement the seven recommended milestones in order. MIT-only licensing selected.
- 2026-09-29 [TOOL]: Remote is willcook415/aster, branch improvements/audit-roadmap; baseline 062d4a8. User authorized targeted commits and push, excluding the private preview.
- 2026-09-29 [CODE]: Quantity-reduction increases fail atomically; equal quantity is a no-op. Private book/level aggregates replace repeated scans.
- 2026-09-29 [CODE]: Strict V1/V2 decoding rejects unknown fields. Matching rules 1 explicitly allow self-trades.
- 2026-09-29 [CODE]: V2 adapter adds command/global event sequences, market expiry and completion; V1 matching and event fixtures remain available.
- 2026-09-29 [CODE]: CommandJournal synchronizes checksummed chained frames before acknowledgement; verified replay, exclusive ownership, incomplete-tail recovery, poisoned failure handles.
- 2026-09-29 [CODE]: V1 exports remain non-atomic complete files. Journal recovery is process-crash scope; no exactly-once requests, snapshot restore, authenticated integrity, or power-loss promise.
- 2026-09-29 [CODE]: Custom JSONL CLI, examples, MIT license, contributor guide, benchmark case study and Windows/Ubuntu stable/1.89 CI are present.
- 2026-09-29 [TOOL]: All seven audit milestones implemented and locally verified. Remote CI results remain unconfirmed until the publication run completes.

## Decisions

- D001 ACTIVE 2026-06-22 [USER]: Project name is Aster; standalone Rust repository.
- D003 SUPERSEDED IN PART 2026-09-29 [USER]: Original no-dashboard instruction now permits the expressly requested local-only review interface.
- D004 ACTIVE 2026-06-22 [USER]: Determinism is a hard invariant; integer ticks and engine priority sequences, not floating prices or wall-clock ordering.
- D007 ACTIVE 2026-06-29 [USER]: Commands are canonical replay input; events and snapshots are audit/verification outputs.
- D008 ACTIVE 2026-06-29 [USER]: Proptest is a core dev-dependency for bounded state-machine comparisons.
- D009 ACTIVE 2026-09-29 [USER]: Private browser exploration approved; the later publication request explicitly excludes the local preview.
- D010 ACTIVE 2026-09-29 [USER]: MIT only supersedes MIT OR Apache-2.0 metadata and prior unconfirmed license choice.
- D011 ACTIVE 2026-09-29 [CODE]: Rust 1.89 minimum supports standard File locks; sha2 supplies vetted checksum implementation. Dependency rationale in docs/decisions.md.
- D012 ACTIVE 2026-09-29 [CODE]: Audit schema 2 and matching rules 1 are separate. V2 is opt-in; strict V1 unknown-field rejection is a compatibility tightening.

## Done (recent)

- 2026-09-29 [CODE]: Repaired public reduction contract and added debug/release capacity regressions on both sides.
- 2026-09-29 [CODE]: Defined strict-schema and self-trade policies with tests.
- 2026-09-29 [TOOL]: Ran 25-workload Criterion matrix before/after aggregate caching; preserved means/95% intervals and honest cancellation regressions.
- 2026-09-29 [CODE]: Added cached totals, independent recomputation and batch-partition properties.
- 2026-09-29 [CODE]: Added custom-input CLI, examples/diagram, MIT, contributor docs and expanded CI.
- 2026-09-29 [CODE]: Added opt-in correlated V2 audit events and browser presentation, with fixtures/replay/sequence tests.
- 2026-09-29 [CODE]: Added scoped command journal and recovery CLI; corruption, fault injection, byte-truncation and process-exit tests pass.

## Working set

- 2026-09-29 [CODE]: crates/aster-core/src/{order_book,price_level,schema,sequenced}.rs
- 2026-09-29 [CODE]: crates/aster-core/src/journal/
- 2026-09-29 [CODE]: crates/aster-core/tests/
- 2026-09-29 [CODE]: crates/aster-core/benches/scaling.rs
- 2026-09-29 [CODE]: crates/aster-cli/src/ and tests/
- 2026-09-29 [CODE]: docs/{decisions,audit-events,recovery}.md
- 2026-09-29 [CODE]: docs/benchmarks/ and scripts/benchmark_report.py
- 2026-09-29 [CODE]: README.md, CONTRIBUTING.md, LICENSE, CHANGELOG.md
- 2026-09-29 [CODE]: .github/workflows/
- 2026-09-29 [CODE]: examples/ and docs/assets/fifo.svg

## Next

- 2026-09-29 [TOOL]: Publish focused commits on improvements/audit-roadmap. Release/tag creation is not requested.
- 2026-09-29 [CODE]: Future investigation: cancellation indexing, atomic V1 export generations, request deduplication, checkpoint compaction and coverage-guided fuzzing. These are beyond the completed seven-milestone scope.

## Open questions

- 2026-09-29 [USER]: None blocking. MIT license choice is confirmed.

## Receipts

- 2026-09-29 [TOOL]: Initial audit: 171 unit/integration tests plus doctest passed on Rust 1.98.1; original quantity defect reproduced in debug/release.
- 2026-09-29 [TOOL]: Focused quantity contract and strict-schema tests passed before optimization.
- 2026-09-29 [TOOL]: Criterion before-cache and after-cache completed all 25 workloads; target/scaling-before.log and target/scaling-after.log.
- 2026-09-29 [TOOL]: 4k one-level insertion 25.133 -> 1.456 ms (17.26x); many-level 191.338 -> 2.852 ms (67.08x). Tail cancellation 19.755 -> 22.843 ms. Local active Windows desktop; docs/benchmarks has full method and intervals.
- 2026-09-29 [TOOL]: Stable full suite: 198 unit/integration tests plus doctest, zero failures; target/tests-final.log.
- 2026-09-29 [TOOL]: Rust 1.89.0 full suite: same 198 plus doctest pass; target/tests-msrv.log.
- 2026-09-29 [TOOL]: Release quantity/aggregate/V2 regressions pass; final eight journal tests pass in release, including every final-frame truncation point and process exit without Drop.
- 2026-09-29 [TOOL]: ASTER_PROPTEST_CASES=4096 and PROPTEST_CASES=2048 extended model/aggregate run passed; target/tests-extended.log.
- 2026-09-29 [TOOL]: cargo fmt --check, strict workspace/all-targets Clippy, rustdoc with warnings denied and all benchmark compilation pass.
- 2026-09-29 [TOOL]: CLI subprocess tests verify custom FIFO, missing-input errors, V2 expiry JSONL, journal create/recover/resume, no overwrite and corruption errors.
- 2026-09-29 [TOOL]: Local-review release bridge rebuilt; browser shows expiry and correlated sequences. Unknown time_in_force rejected; valid session remains and replay succeeds. No native desktop interaction.
- 2026-09-29 [TOOL]: Git whitespace checks pass. Private review artifacts and preview source are excluded from publication.

## Incidents

I001 RESOLVED 2026-09-29 [CODE]: reduce_front_quantity previously allowed increases and could make book totals overflow. Increase rejection and checked cached aggregates now preserve invariants; both sides tested at u64::MAX in debug/release. Regression tests assert the fixed behavior.
