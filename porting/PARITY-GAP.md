# From here to full parity

This is the contributor-facing map of what remains between the Rust port and full SysON parity, with one or more open issues for every piece. It was written on 2026-11-18 from `main` at the merge of #306. The detailed, evidence-oriented register remains [`rust-parity-plan-20261026/REMAINING-WORK.md`](rust-parity-plan-20261026/REMAINING-WORK.md); this page is the short version, organized by what a contributor can pick up.

## The bar

Full parity is reached when the **acceptance journey** passes in the Rust application:

> Create or import a project, edit it, save, reload, close, restart the Rust backend, reopen, and find the same element IDs, references, representations and history, without manual re-import. Every SysON workflow used on the way (import, editing, diagrams, explorer, details, tables, validation, export, project download/upload) behaves as it does in SysON.

The SysON side of that journey is scripted and verified ([`oracle/restart`](oracle/restart/README.md), #298). The Rust side is automated in [#422](https://github.com/ajhcs/syson/issues/422).

## Where things stand (2026-11-18)

| Area | Verified on `main` | Main gap |
|---|---|---|
| SysML text import | Full-grammar recognizer; a finite importer. Golden replay of 62 cases: **12 match, 1 differs, 49 refused**; 2 of the 19 bundled SysON examples match. Two wire-format differences remain (`elementId` references #248, root `Namespace` wrapper #249). | Most language constructs (#246 and the import issues below) |
| Model | Pinned metamodel registry; EMF JSON model with transactions and undo; names and IDs; a physical index of all 94 standard-library files (`rust/tools/native-library-catalog`) | Derived features and operations; semantic library loading (#411) |
| Storage | Local snapshot store reopened across separate processes (#304) and after writer kills (#305); verified reads of SysON's PostgreSQL rows; 6 of SysON's 8 migration participants (`syson-persistence`, `syson-migration-pipeline`, `syson-migrations` `b25a`–`b25d`) | A project model, a chosen backend, the 2 diagram migrations (#418), representations |
| Interchange | Project archive inventoried and round-tripped through SysON's upload (#289, #296); Rust EMF JSON reopens in SysON (#293) | Rust read/write of whole archives (#271) |
| Server and API | None (no HTTP, GraphQL or REST in Rust) | Everything in "Server and APIs" |
| Views | Layout, orthogonal routing, label measurement, rotate-node; an opt-in WASM editor on `rust/integration` only | Every diagram, explorer, details and table view |
| Validation | None | 204 rules |
| CI | Workflow covers all 23 crates (#291), but **GitHub Actions is disabled on the repository** (#453) | Restore CI |

## Work streams

Each stream lists its issues roughly in dependency order. **Bold** issues are on the critical path to the acceptance journey.

### 1. Language: import everything SysON imports (Phase 1, [#269](https://github.com/ajhcs/syson/issues/269))

- **#246** build the importer on the full-grammar recognizer
- Constructs: #260 comments, #261 usage bodies, #262 imports, #263 attributes, #264 values, #265 specialization, #266 aliases, #267 other definition kinds, #268 visibility and multiplicity, #396 connections and flows, #397 state machines, #398 action flows, #399 expressions, #400 enumerations, #401 constraints and requirement bodies, #402 use cases and views, #403 metadata, #404 port directions and conjugation, #405 KerML files
- Fidelity: #247 recognizer budget, #248 and #249 wire format, #250 library references, #252 tolerant import, #253 escaped quotes
- #406 insert textual SysML into an existing element; #407 diagram label (direct-edit) grammar
- Hardening: #449 fuzzing, #450 benchmarks

### 2. Export: text back out

- #408 textual SysML export; #409 export differential and round trip; #443 expression text in the editor

### 3. Model semantics and libraries

- **#410** derived features and operations; **#411** standard libraries; #412 library publication and import
- #413 validation inventory and differential; #414 validation engine
- #423 undo/redo semantics; #424 duplicate and move

### 4. Projects and storage (Phase 2, [#279](https://github.com/ajhcs/syson/issues/279))

- **#415** decide the storage backend; **#416** project lifecycle; #417 templates
- **#420** representations stored byte for byte; **#271** read and write SysON project archives
- Migrations: #418, the two remaining diagram migrations (the other six are on `main`)
- #421 snapshot retry bug after an interrupted writer
- **#422** automate the acceptance journey; #259 differential testing past import

### 5. Server and APIs (Phase 3, [#280](https://github.com/ajhcs/syson/issues/280))

- **#274** decide the API; #273 GraphQL inventory (partly done in #294)
- **#425** server crate; #426 and #427 SysML v2 REST API read and write
- **#428** GraphQL for the project journey; #429 live subscriptions; #430 search

### 6. Views and editor (Phase 3)

- **#444** decide the UI strategy; #452 bring the WASM editor crates to `main`; #275 views inventory
- Diagrams: **#431** General View model, #432 node tools, #433 edge tools, #434 show/hide and delete semantics, #435 Interconnection, #436 Action Flow, #437 State Transition, #438 custom node styles, #439 layout comparison with ELK, #445 CJK and emoji labels
- **#440** explorer, **#441** details, #442 requirements table

### 7. Project health

- #453 restore GitHub Actions (maintainer); #454 triage the draft PR backlog; #456 linker boundary fix and #457 Unicode tests recovered from drafts
- #446 Cargo workspace; #447 golden replay in CI; #448 WebAssembly build check; #451 published API docs
- Docs: #277 crate map, #278 SysML primer, #196–#200 examples and guides, #201 second-environment build

### Then: MBSECraft (Phase 4, [#281](https://github.com/ajhcs/syson/issues/281))

## Critical path

The shortest route to a passing acceptance journey:

1. Language: #246 plus enough constructs that the Batmobile template imports (#247, #261, #263–#268, #396, #399), and #411 for library references.
2. Storage: #415 → #416 → #420 → #271, with #418 so existing projects open.
3. Server: #274 → #425 → #428 (if SysON's GraphQL is kept, the SysON frontend can drive the journey before a Rust UI exists).
4. Editing: #431, #440 and #441 for the edits the journey performs, with #423 for history.
5. #422 runs the journey against Rust.

Validation, export, the remaining views and the REST API complete feature parity after that, and can proceed in parallel at any time.

## Not counted as parity evidence

A crate test, a helper, or a standalone experiment shows that piece works; it does not show the application works. Keep the distinction the [remaining-work guide](rust-parity-plan-20261026/REMAINING-WORK.md#current-status-and-the-limits-of-the-evidence) draws: library-level, native-reference, application and UI acceptance are reported separately.
