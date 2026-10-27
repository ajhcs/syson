# RR20 qualification

Default local undo/redo now shares a maximum64 entries and128MiB capacity-charge policy. Both limits apply together. Typed state reports availability, discarded counts and truncation reasons; zero disables retention, changed commits can succeed without retaining an oversized prior snapshot, and nearest history is retained with deterministic oldest-undo/farthest-redo eviction. Number compaction preserves exact lexemes and literal marker objects. Revision exhaustion refuses before publication/pop. The charge is an explicit conservative policy, not a portable heap or RSS guarantee.

Two source authors worked on disjoint RR18a/RR20 branches; root owns compilation/runtime/publication, third reviewer independently inspects source/logs/seals read-only. Prerequisite base is exact RR04 #94 plus pinned #65 web/layout/measurement/routing source with root-owned offline lock refresh, recorded in `porting/rust-parity-prerequisites/BATCH9/source-manifest.json`. Held initializer/raw/factory modules are unchanged and never enabled. Snapshot base must not be merged as a production integration branch; land prerequisites and retarget/requalify.

## Verified behavior

- Actual1000-element/256-commit baseline reaches the retained256-undo assertion failure in history-baseline-red-02. The first root mini-consumer failed setup on InvalidId(resource); its incorrect initial receipt label is preserved alongside an explicit overriding attribution correction. It is not counted as a valid regression. A separate pre-execution Python syntax typo executed no product command.
- Final default suites pass70model tests (including11newintegration cases and1private exhaustion case) and44web tests. Both all-target default Clippy gates, scoped nonrecursive formatting and fresh release WASM build pass, all7commands0. No test is skipped by this repair; inherited cfg-gated candidate suites remain unrun under active holds.
- Root runs the exact same independently reviewed external System allocator probe a918e88e41a2567cf7d9ff86c6258711c4b031046aaf7ee68bfcdcc979092dc7 in four fresh host processes per baseline/final implementation. Probe binaries and Rust source closures are bound in their receipts; all6commands per profile set succeed. Arithmetic/capacity/zero/exact/one-less/asymmetric/noop/foreign/stale/poison/cancel/publication/number-token/revision cases pass on final source.
- Actual default WASM through rendered controls at1365x1000 and390x844 creates a Package, stages/commits70renames, keyboard-undo64 to Revision6, checks unchanged exhaustion, keyboard-redo64 and exact export restoration. Visible names and exports match at each step. No console errors/page errors, no horizontal overflow, no recorded longtasks in this small-document journey. Root visually inspected both screenshots; this is not large-document rendered acceptance. Observed maximum end-to-end automation commit durations were68.46/68.26ms, not an interaction SLA.

## Paired host observations

Each row is one fresh process, not p95 or a SysON performance comparison. Retained live is main-thread requested allocation storage observed after edits, excluding the original fixture, allocator overhead and other threads. RSS is whole-process Linux observation. 40/9retained steps demonstrate the byte charge may bind before64.

| Nodes | Commits | Baseline retained bytes | Final retained bytes | Baseline RSS KiB | Final RSS KiB | Final undo/redo steps |
| --- | --- | --- | --- | --- | --- | --- |
| 1000 | 64 | 92683148 | 58462959 | 110992 | 74628 | 40 |
| 1000 | 256 | 366455145 | 58463000 | 417436 | 74704 | 40 |
| 4096 | 64 | 379450148 | 58377286 | 446564 | 95776 | 9 |
| 4096 | 256 | 1500287745 | 58377296 | 1700348 | 95808 | 9 |

All four final cancellation live deltas are0 and final undo/redo restore the exact current name; this measures explicit transaction cancel after synchronous rename, not cooperative interruption or responsiveness. Begin/apply requested allocation work is unchanged (for4096/256,2,988,883,746/1,494,450,469bytes); it is not exact copied-byte accounting. Undo may allocate history-container capacity but moves Documents; redo allocated0requested bytes in these observations. Current documents/open transactions and process memory remain outside the retained-charge limit.

## Evidence and remaining gates

`executed-source-binding.json` binds every captured candidate file to tested bytes; publication commits are independently compared to these blobs. Captures preserve stdout/stderr/recipes/receipts, metadata, exact baseline/final probe consumers and browser screenshots/JSON. Duplicate served web mirrors, built WASM, compiler targets and caches remain external on the approved workspace; receipts bind their hashes. The original failed fixture and its corrected attribution are retained verbatim. Later evidence-only publication adds metadata without replaying passed tests.

RR19 still owns visible typed truncation/disabled controls/kind affordances; the existing UI reports generic No history available. Transaction copy optimization, large rendered workloads, preemptive cancellation, arbitrary feature unions, native SysON comparison, full integration/release and feature parity remain OPEN. All current implementation claims are LOCALLY_VERIFIED_BOUNDED/NOT_ACCEPTED. No merge/deployment or held-source release.
