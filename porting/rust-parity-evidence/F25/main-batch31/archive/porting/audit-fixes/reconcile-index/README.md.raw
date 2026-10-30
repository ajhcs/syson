# Reconciliation identity indexes

The audit found quadratic work in a common no-op document update: the selected planner repeatedly scans pointer identities, the existing UUID list and the output list. With N unique unchanged document observations it performs 2N²−N pointer/UUID comparisons before field comparisons and ordered-set work. The source at F25 remains the semantic baseline; this repair must preserve its caller-supplied iteration order and Java object-identity behavior.

`examples/reconcile_bench.rs` is a pure release-mode benchmark with no filesystem or database operation. It prepares 1,000, 5,000 and 10,000 unique documents, then runs one warmup and five measured unchanged plans for each size. The same frozen example is used before and after repair. `black_box` and an explicit Unchanged assertion keep the full planner decision observable. Timing samples and process RSS come from the coordinator's actual captures, not source estimates.

The intended repair uses standard-library pointer sets and a first-existing-UUID index while building output in original incoming order. Duplicate UUIDs must consult the first existing observation. Kept objects deduplicate by pointer; pointer-distinct incoming objects with equal values remain distinct, and separate rebuild actions remain separate. Old/new ID-set comparison and requested effect order remain unchanged. Hash-index iteration must never select output order.

No clock, event, database, native Java application or UI integration is added. Native source semantics are pinned by the original F25 source contract. Local benchmark results will qualify this Rust implementation and host only; they do not assert a native Java performance comparison.

The coordinator captured `rec-before-01`, then `rec-after-01` passed all 16 tests but failed two style lints in the new regression. That exact source, runner and output remain retained. Test-only `.is_multiple_of(257)` spelling fixes leave production and benchmark bytes unchanged. `rec-after-02` passes formatting, all 16 tests, all-target Clippy and the identical release benchmark. All executed inputs remain unchanged within each capture.

| Documents | Before median | After median | Median speedup |
|---:|---:|---:|---:|
| 1,000 | 0.680922 ms | 0.069463 ms | 9.80× |
| 5,000 | 14.121751 ms | 0.411136 ms | 34.35× |
| 10,000 | 55.109110 ms | 0.896634 ms | 61.46× |

These are five-sample release medians after one warmup for each size. The measured 10k/1k growth falls from about 80.9× to 12.9×. Raw ranges and exact runner/input hashes are retained with each capture; `benchmark-summary.json` reports derived ratios. This does not establish a universal speedup for tiny collections. The time utility wraps Cargo compilation plus the benchmark, so its RSS is not a planner-only memory measurement. Dependency add/remove planners and native event/clock execution are unchanged.
