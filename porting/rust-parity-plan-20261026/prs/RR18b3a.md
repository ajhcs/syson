# RR18b3a: Borrow element summaries in the actual HTML renderer before amplification

Lane: rereview-repairs. State: PLANNED_NOT_IMPLEMENTED.

## Source preparation prerequisites

- RR18b1
- RR18b2

Existing PR prerequisites: [#61](https://github.com/ajhcs/syson/pull/61), [#65](https://github.com/ajhcs/syson/pull/65), [#112](https://github.com/ajhcs/syson/pull/112)

## Implementation readiness

- RR18b1
- RR18b2

## Feature acceptance dependencies

- **joint gates:** None declared.
- **children:** None declared.
- **joint implementation inputs:** None declared.
- **expanded implementation inputs:** None declared.

## Write scope

- rust/crates/syson-model/src/element_summary.rs
- rust/crates/syson-model/tests/rr18b3a_regression.rs
- rust/crates/syson-web/src/lib.rs
- rust/crates/syson-web/tests/rr18b3a_regression.rs
- porting/rust-parity-evidence/RR18b3a/

## Coordinator-owned wiring

- **paths:** - rust/crates/syson-model/src/lib.rs
- **policy:** Coordinator-owned shared visitor adapter and private module/public export integration. The initial historical author grant and prepared adapter remain accurately recorded; root reviewed and serially integrated the frozen proposal plus its own module/export hook after prior RR20 source lease release. This is not a retroactive authorship rewrite. Future authors prepare new-module/test proposals; no author may write this shared file during coordinator integration. Reconcile RR20 and A frozen proposals before joint/main landing and requalify affected callers; current A does NOT contain RR20 history changes.
- **verification:** Parallel-path verifier preserves this declaration but does not prove coordinator scheduling. Actual admission/release/freeze/root commit records are the manual serialization proof.

## Tasks

- Expose borrowed summary views using one shared existing traversal; keep current owned summary API and exact ordering/owner semantics. Adopt borrowed views and owner/navigation keys in actual App.render without cloning arbitrary owner IDs.
- Root alone registers additive model module/export after author freeze. Preserve the legacy API; no new model engine or admission policy.
- Coordinator integrates the author-prepared shared collect_summaries visitor adapter and root mod/pubuse registration under explicit serialized ownership; preserve the original author grant/actual writes and do not invent RR20 source adoption.

## Verification

- Reproduce actual renderer/import aggregate allocation amplification with an under-1MiB valid 64KiB parent-ID/512-child fixture. Measure aggregate/live requested bytes, not largest alone. Verify borrowed pointer identity, exact manual preorder/owner tuples, old API compatibility and actual rendered navigation/import/history.
- Freeze red inputs before source changes, then run affected model/web default cases, scoped nonrecursive format, Clippy, WASM and actual mobile/desktop UI. This foundation cannot close full HTML or <50ms acceptance. [TIMING POLICY SUPERSEDED 2026-10-28: Measure workload-specific latency ranges and outliers under performance-policy.json; report distinct event-to-visible latency, main-thread task durations and allocation metrics. Preserve raw timing evidence; no single50ms veto or automatic parity acceptance. Keep actual responsiveness, cancellation and exact-data checks.]
- Before main landing, compose actual RR20 history changes and A borrowed-summary adapter on one reviewed candidate, reconcile shared lib.rs and run appropriate affected tests/UI gates. Existing124tests qualify only the current40b3-based A source, not that future composition.

Current timing policy: [workload-specific target ranges](../performance-policy.json) supersede legacy timing cutoffs; original observations remain historical.

## Commit plan

- Focused source implementation and meaningful regression/caller fixtures
- Independently reviewed exact verification evidence
- Data-only addition of exact sealed HTTP logs omitted by Git ignore; preserve earlier local commit and verify every artifact against Git

## Review budget

- **production lines max:** 400
- **authored lines max:** 900
- **expected commits:** 3
- **split rule:** Split into numbered behavior-family children before exceeding either limit; generated/evidence data is a separate review block.

## Completion

All named counterexamples/regressions and actual consuming caller gates pass on exact reviewed source; unsupported or unrun acceptance remains OPEN.

## Source evidence

- rereview-20261026/model-review/findings.json
- rereview-20261026/parser-review/findings.json
- rereview-20261026/platform-review/findings.json
- rereview-20261026/ui-findings.json

Original lane record: [lanes/rereview.json](../lanes/rereview.json). Source snapshots stay immutable; plan.json adds the explicit coordinator dependency overlay.
