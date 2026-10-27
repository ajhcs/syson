# RR18b3c: counted scalar and escaping helpers

Implementation checkpoint `4040c3947530d2e9770f6846e5e2bd489dd1c662` adds a two-pass bounded byte sink, streaming JSON/HTML escaping and complete scalar-inspector emission. Root propagates typed failures through preview, navigation and paint to existing committed recovery. Each helper is fully counted before one exact output reservation. Sticky errors prevent a trusted emitter from ignoring a refused write and publishing a partial fragment. Existing model inspection supplies one owned snapshot to both passes; its Value clones remain. No second model/literal authority is introduced.

The formatted review budget is conservatively 400 production additions and 876 authored lines including root wiring, tests, shared allocator, browser recipe and preparation. Captured legacy golden JSON and immutable runtime evidence are a separately reviewed data block. The actual aggregate helper-error consumer is split into required verification child RR18b3c2; its discovered host defect has separate repair RR18b3c3. This producer does not close those consumers by itself.

## Exact source and ordinary verification

All 4,804 source bindings match before/after qualification and the implementation tree. Fresh default web qualification passes 72 cases (61 library, then 5/3/2/1 integration), scoped nonrecursive formatting, all-target Clippy, a fresh release WASM build and recipe syntax. WASM SHA256 is `cd2528781cf0659ac939c3cda28bb296b3cb4141b575732182da7f388f15b945`. No raw-factory, initialized-graph or held factory execution occurs. Selected executable identities are recorded; complete dynamic/sysroot/environment qualification remains RR16b open.

Ten escape, ten inspector and 122 field outputs were captured from the old compiled helpers, with exact full HTML and raw model strings. All 142 outputs pass exact-fit and one-byte-less complete-wrapper limits. Separate manual oracles cover UTF8, CR/LF/NUL/sentinel behavior, split-byte JSON emission, raw negative zero/exponent/huge integers and marker objects distinct from strings/numbers. Whole-inspector admission refuses when separately fitting fields exceed the aggregate limit. Fault injection checks swallowed errors in count and emission.

## Allocation measurements and limits

The actual accepted under-1MiB opaque import dispatch requests 88,951,293 allocation bytes on the original code and 32,944,233 on the final producer. HTML length remains 3,504,258 and canonical model export remains exact. Largest request remains 7,006,834: this is not a per-request global bound. The old standalone instrument and the final shared instrument both reproduce the identical original failure, so the oracle was not weakened during boilerplate deduplication. Both failed baseline attempts and exact source are retained.

Prepared direct-helper tests observe zero requested allocations before refusing an oversized 1.7-million-character scalar or 4-million-character escaped input. Input/descriptor construction occurs before this measurement and those direct inputs exceed the browser's action domain. These are successful-request aggregate/largest observations, charging full realloc requests; they are not RSS, live/high-water heap or whole-app memory measurements. A single under-1MiB scalar cannot alone produce over16MiB through one6xescape. The existing 700258-byte/700000-ampersand Package is instead a global duplicated-renderer counterexample.

The complete renderer still materializes navigation/options/SVG/model export/geometry and final HTML before its outer cap. Selected inspection also adds a full-model json_size scan before owned cloning. Those costs and global preallocation limits remain RR18b3d obligations; no O(1), clone-free or whole-renderer claim follows.

## Actual rendered checks

Three fresh original-binary baseline scripts and thirteen final-binary browser scripts pass. They exercise real 320/1440 native scalar controls, AX names/value/multiline/read-only states, actual Tab/Enter, exact CRLF/NUL preservation, staged/cancelled/committed edits, undo/redo, null versus absent, recovery drafts, canvas transport/cancellation/faults/long IDs, native textareas, downloads and persistent accessibility. Browser plugin was unavailable; installed Playwright1.64.1/Chromiumheadless1248 with pinned Node24.21.0 ran fresh owned loopback contexts. Every invoked script, served asset, fixture and owned descendant cleanup is bound; no model, HTML or WASM response is mocked. Native SysON runtime comparison is not executed.

Measured final scalar tasks reach52/52ms, phase64/64ms, ordinary recovery116/100ms and dense ownership178/169ms at320/1440. CDP Layout maxima are26.264/26.107ms while RunTask maxima are64.863/64.019ms. These individual observations are not statistical/global speedups; tracing can perturb timing and inclusive durations are not additive. Required<50ms acceptance remains open. Missing glyphs, oversized labels, saved-value paragraph walls and general visible-focus usability remain explicit separate repairs.

## Consuming caller and landing gates

The actual RR18b3c2 journey exposed a pre-existing loader quota defect before its attribute request was encoded: a pristine LF-normalized850k textarea was compared to a raw CRLF default and falsely charged as dirty alongside the850k short-name draft. That timeout is a retained FAILURE, not helper-error execution or part of the thirteen passes. RR18b3c3 repairs only native default comparison, and RR18b3c2 must pass the unchanged actual recovery/download/history/font-confirmation journey on an exact repaired composition. Their final evidence remains separately attributable.

Code PRs remain review-only until actual main prerequisites, historical EXPLICIT_ONLY holds, shared RR20/model reconciliation and exact landing-tree qualification permit. Frozen source snapshots are never merge destinations. Full SysON parity, release, physical devices, other browsers, full native glyph coverage and global performance/usability are NOT_ACCEPTED.
