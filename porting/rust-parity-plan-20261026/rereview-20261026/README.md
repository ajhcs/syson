# Completed-work re-review and future repair plan

Review started 2026-10-26 UTC and completed 2026-10-27 UTC.

This review found concrete data-integrity and resource-admission defects, plus delivery and verification gaps. It adds **24 future repair units (RR01–RR24)** and strengthens **50 existing units**. No production repair is implemented here. Original bounded verification remains historical evidence; no unit is promoted to feature acceptance.

The inventory covers **all 87 published PRs**, including the original rollup, component splits, follow-ons, audit repairs and planning PRs. These overlap; 87 is not a feature count. Three independent reviewers examined model/query, parser/interchange and platform/composition work. Their initial packets cover 70 PRs; a geometry supplement reviews five further PRs. Root reviewed the remaining source/evidence/plan dispositions and executed selected counterexamples and UI journeys. [Coverage matrix](coverage-matrix.json) records the depth and omissions. This is not an exhaustive line-by-line audit of every diff, every test or every native workflow. A no-new-finding disposition is not acceptance.

Target SysON source remains `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`; canonical Rust integration remains `ad814c232ee24109639c1d52239cbeec6f4a3e92`. [Published inventory](published-inventory.json) pins actual heads, bases and paths. The previous plan snapshot was `4ac8ff1a12dd3772a7a67530f5c28226e20bad74`. All local implementation PRs remain bounded drafts, not completed product parity. Native artifact/source binding and independent complete source build remain open.

## Reproduced counterexamples

| Finding | Fresh result | Future repairs |
|---|---|---|
| MODEL-RR-001, opaque JSON | A valid vendor object with literal `$serde_json::private::Number` key becomes numeric `123`. The model probe executes an immutable dependency copy with the exact affected `Document::from_json` implementation, rather than rebuilding the original component PR separately. | RR01/RR02, RR22/RR23/RR24 caller adoption |
| MODEL-RR-002, physical catalogue | A JSON string under literal `$serde_json::private::RawValue` becomes a physical Namespace entry at `/vendor`: zero real content nodes, one indexed node. Fresh CLI execution uses the retained, hash-verified PR85 executable. | RR01/RR03 |
| PARSER-RR-01, JSON bundle | A 500481-byte input returns its 1 MiB serialized-limit error only after a **16777216-byte single allocation**, with 21011477 additional requested-live bytes. | RR04/RR05 |
| PARSER-RR-02, text export | A 2 MiB name returns its 65536-byte output-limit error after a **4194320-byte single allocation**, with 4194672 additional requested-live bytes. | RR06 |

The allocation measurements use a development-only counting allocator around the API call. They describe requested layouts, not RSS, allocator internals, native performance ratios or application SLAs. Cases ran offline with one compiler job, 512 MiB address-space limits, CPU/wall deadlines and unchanged source hashes. [First attempt](counterexamples/counterexamples-01/receipt.json) preserves the two successful allocation observations **and the root model-probe compile failure** (missing Result unwrap). [Focused second attempt](counterexamples/counterexamples-02/receipt.json) executes only the corrected model probe. The first aggregate run remains failed. [Catalogue receipt](counterexamples/catalogue-private-token-01/receipt.json) retains exact input/binary hashes and output.

[Executed probe sources](counterexamples/executed-probe-sources/README.md) and the [initial proposal](counterexamples/proposed-probes/README.md) are distinct. Runner copies are archival records with original workspace paths; they are not a portable root-level test command. Immutable source exports and build products remain under `/workspace/build/builderbob/syson-rereview-20261026`. Future repairs must first preserve these failures, then run new regressions on their exact changed source. Tests expecting these reserved keys must use an independent raw-token or manually constructed object oracle, avoiding the same generic Value conversion under test.

## Other actionable findings

All findings below are source-review findings unless explicitly marked as rendered evidence. They require the named reproduction and acceptance gates before closure.

| Family | Concrete issue or unfinished obligation | Planned response |
|---|---|---|
| Data and interchange | Value-backed pre-parses can reintroduce marker corruption before Document admission; actual migration/wire/readers require caller inventory. | RR01/RR02/RR22/RR23/RR24; I03/I06 |
| Parser/export composition | F14 accepts quoted `class`; F34 follows the narrower NameHelper keyword set, so its generated unquoted token fails actual importer admission. It refuses export, rather than returning a successful invalid roundtrip. | RR07; P170/P171 |
| AST transactions | Small operations clone the entire State while charging one logical step; repeated construction/postprocessing can accumulate quadratic copying. | RR08; P12/P14/P18/P184 |
| Graph/query | Per-node recursive subtree clones and repeated whole-map peer counting increase memory/work; a flat acyclic conjugation chain has cycle detection but no traversal bound. | RR09/RR10; held-owner reconciliation before writes |
| Local history | Whole Document snapshots accumulate without a retained-byte/entry budget. | RR20; M03/B17/U08 |
| Validation | Full serialization/reparse happens before a bounded selected projection refuses unrelated opaque input. | RR21; M32/I09 |
| SQLite identity | Observations bind path/Arc/key/revision, allowing a valid replacement DB with the same keys/revisions to remain a stale-mutation risk. A stat before reopen is insufficient transaction identity. | RR11; B06 |
| Storage bounds | Update names are cloned before validation; oversized stored metadata can be materialized/encoded before response caps. | RR12; B05/B06 |
| File admission | Snapshot, composition-manifest and old syntax-observer paths can open FIFO/nonregular inputs before establishing regular-file admission. Snapshot output also checks after full serialization. | RR13–RR15 |
| Integration evidence | Fixed integration inventories and missing before/after closure allow incomplete receipts; repository CI does not yet gate the complete Rust/tool/WASM closure. | RR16/RR17; C02/C03/C04 |
| UI preview/recovery | Full model/HTML output precedes its cap; import can replace Editor/history before preview failure leaves an Undo-only fallback. This large-input trigger was not executed in the browser in this pass. | RR18; B31/U03/U18 |
| Libraries | Selected 32-resource/default-depth adapters cannot establish actual 94-resource, depth-208 library loading/resolution. Catalogue identity matches do not close GeneralView/type-picker/create/open callers or native diagnostics. | M15/M30/M38/I06 |
| Geometry and extensions | Fractional measurement/layout versus integral routing, conservative default work charging and synthetic host/provider observations need an actual caller contract and measured capacities. No new material algorithm defect was established in the geometry supplement. | U02/U10/U11/U29/B29/M20 |
| Cancellation | Long name/escape output loops poll only at endpoints. | P08, actual long-input caller latency gates |

The sealed [model findings](model-review/findings.json), [parser findings](parser-review/findings.json) and [platform findings](platform-review/findings.json) provide exact source anchors, hashes, expected/actual behavior and verification requirements. The separate [geometry supplement](model-review/geometry-supplement.json) preserves the original model packet unchanged. No database replacement or FIFO reproduction ran here; these remain source-supported tasks, not measured runtime failures.

## Fresh UI and usability observations

Chrome DevTools exercised retained exact PR65 WASM (`b9ff6d03c8bd84a598149823d3f7b7398bbfe195e61307c73a4b8ddd1c702eab`) and three Git-bound assets from `394b82a808f1650b63eeb2ef32516cc7ce99d4a8`. The previously qualified build is explicitly reused; browser interactions and screenshots are fresh. The browser plugin was unavailable, so Chrome DevTools was the fallback; no Playwright was invoked. This pass did not run a new native SysON browser comparison, screen-reader speech study or full accessibility certification.

| Step | General health | Output |
|---|---|---|
| Desktop entry, 1440×960 | Usable; unavailable history/canvas commands still enabled. | [Screenshot](ui/01-desktop-start.png) |
| Stage package | Selected staging, preview and commit focus work. | [Screenshot](ui/02-staged-package.png) |
| Commit, then 390×844 | Commit works; creation form starts around y=1216 and Stage around y=1542. No horizontal overflow observed, but primary editing is buried. | [Screenshot](ui/03-mobile-after-commit.png) |
| Reload | Committed tab-local content clears from one element to zero without an effective exit guard. The initial copy already says tab-local: this is a delivery gap, not a broken advertised durable store. | [Screenshot](ui/04-reload-cleared-model.png), [before](ui/before-refresh-dom.json), [after](ui/after-refresh-dom.json) |

[UI findings](ui-findings.json) assign durable state/exit/reopen to U03/B08/B06, task-first responsive navigation to U01/U03/U04/U05, and command availability/kind-specific controls to RR19/U08/U05. Only a favicon 404 was observed in the console; no WASM runtime error appeared in these short journeys. This does not establish general browser health or performance.

## Execution and verification of future repairs

[Repair map](repair-map.json) and [individual RR briefs](../PR-INDEX.md) define exact paths, source prerequisites, two-commit targets and validation. The [re-review overlay](../rereview-overlay.json) appends tasks without rewriting original lane snapshots. New repair units are required acceptance children. Unimplemented consumers also acquire source prerequisites; already implemented bounded children keep their original source/evidence attribution and remain NOT_ACCEPTED.

Prioritize data integrity RR01→RR02/RR03, with RR04 following RR02. Independent RR06, RR08, RR11, RR15 and RR16 can be authored in separate scopes; choose at most three after checking leases and actual prerequisites. RR09/RR10/RR02/RR04/RR20 touch held graph/model families and need explicit owner reconciliation. No task silently releases those holds. Shared Cargo/lock/export edits remain coordinator-owned; builds and native/DB/browser epochs are serialized. Existing PR prerequisites must land, then children must be retargeted and requalified; do not merge into snapshots or dependency feature branches.

Each proposed implementation stays within 400 handwritten production lines and 900 authored lines, targeting a focused implementation/regression commit and an evidence/review commit. Split newly discovered behavior before exceeding those bounds. Every repair must preserve the original failing receipt, add a meaningful independent regression, run all affected suites on final source, exercise actual consumers and undergo independent review. File/DB/browser cases use fresh owned fixtures and teardown, with allocation, cancellation and performance measurements distinguished from logical work limits.

The graph now has **757 review units / 1514 readiness nodes**, with no unordered write conflicts. These are scheduling units, not features or a completion percentage. All 20 published new-plan implementation entries remain unchanged and NOT_ACCEPTED. C01/C03/C99, full native-source binding, connected durable Rust workflows, UX/performance/capacity and release acceptance remain open. [Validation](validation.json) records graph, reproducibility, local-link and negative-gate checks. Nothing is merged or deployed.
