# Independent review: initial Rust compatibility slices

Status: incomplete parity; source-derived native regression checks only. No upstream runtime oracle, browser, database, graphical editor, parser/typechecker, authorization, collaboration, cancellation/retry, undo/redo or end-to-end acceptance was executed by this lane.

Validator identity: `/root/validation`, isolated branch `rust/validation`, worktree `/workspace/worktrees/builderbob/syson/validation`. No implementation edits by validator. Pinned upstream: `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`. Host-context `bob-admin workspace-status` confirmed Samsung SSD 990 PRO `/dev/nvme0n1p1` mounted read-write on `/workspace`; sandbox missing alias did not trigger repair.

## Reviewed scope and evidence

- Domain commit `253d010da8f2121f343b471c2e44b6ed78f843b4`: names, projected ownership/name lookup, UUIDv5. Read against `backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/util/ElementUtil.java` and `.../impl/ElementImpl.java` (getOwnedElement lines 398-404). Java strip predicate now preserves nonbreaking spaces and removes U+001C..U+001F; independent test passes. General relationship semantics remain unsupported. Explicit adapter contract requires preflattening non-Membership relationships; neither EMF inheritance nor effective-name overrides are established by this projection. This is a bounded input contract, not complete metamodel parity.
- Editor initial reviewed source SHA256 `0a8781351d5f01a5ea39c0133e4adc4e04de75dd82dfb264d64fa0f3e7bfab25`: rotation, package leaf formulas, note paths and presentation. Sources: `frontend/syson-components/src/extensions/rotateNodeTool/useRotateNode.ts`, `nodes/package/SysMLPackageNodeLayoutHandler.ts`, and `nodes/note/SysMLNoteNode.tsx`. Renderer, DOM measurement, layout engine, transport and a working Rust/WASM UI remain unimplemented by this slice. SVG number spelling is a documented boundary, not tested equivalence for arbitrary IEEE754/CSS input.
- Persistence commit `6110a516c67f0d7da8331d45a3325cb363f3aa7e`: ID map parsing, reference URI and diagram target migration, download exclusion. Read against `backend/application/syson-application-configuration/src/main/java/org/eclipse/syson/application/migration/StandardLibrariesIdsResourceToMapParser.java`, `StandardLibrariesElementsReferencesMigrationParticipant.java`, `StandardLibrariesElementsDiagramMigrationParticipant.java`. Resolver adapters must preserve no-load behavior and Element filtering. No store/database implementation, recovery guarantee or actual JSON traversal is supplied. Native source-derived URI substring priority, nonlibrary fallback blocking and Java trim/split cases pass independent tests.

## Findings sent to editor owner

1. P2: `finish_rotation` uses selected membership instead of upstream Array.find result truthiness. A matching empty string is false upstream; Rust adopts laid-out node. Edge selection must preserve *first* match: selection [empty, x], source empty, target x must preserve prior edge because the first matched value is empty. Independent node and edge regression tests fail on initial snapshot.
2. P2: leaf old-size retention is upstream `if minimum > previous { minimum } else { previous }`, not Math.max. A NaN measured label creates NaN minimum; upstream keeps finite previous height 60, Rust returns NaN. Independent regression fails. NaN previous value already propagates in either implementation; test specifically uses NaN computed minimum.

These are scoped algorithm defects. They do not establish UI reachability or production severity. No evidence-backed input validation currently excludes either case.

## Initial execution

All Cargo manifests and library source files inspected; no dependencies or build scripts in reviewed libraries. Copied exact reviewed source to `/workspace/build/builderbob/syson/validation/review-01`, leaving implementation worktrees unchanged. Ran:

`CARGO_TARGET_DIR=/workspace/build/builderbob/syson/validation/target cargo test --offline --jobs 1 --manifest-path /workspace/build/builderbob/syson/validation/review-01/rust/validation/Cargo.toml`

Result: 7 tests, 4 passed, 3 failed, exit 101. Compiler slot released after run. Regression source committed in `rust/validation/tests/source_regressions.rs`; Cargo dependencies resolve against sibling `rust/crates` after integration. Independent expectations were read from pinned upstream source; Java and TypeScript software were not executed. This is not a runtime differential oracle.

## Corrected editor recheck

Editor owner independently authored fix commit `429afb9242fd72fdc1f4e0591dcd7fd79cbc1da4` after the above findings. Validator read the diff: first-match selection truthiness retained for nodes and edges; previous-size branch restored exactly. Fixed editor source SHA256 `7b003258448bacc1bcfff3319139ddac8dc2595c9e3a3530c345d61f7de19c01`. Initial source was commit `4992b03c5`.

Rebuilt the harness in `review-02` with the domain and persistence snapshots unchanged, changing only the reviewed editor source and formatting validator test code. Same offline one-job command with `review-02` path: **7 tests passed, 0 failed**, exit 0. Both P2 findings are resolved for the tested inputs. This is the final bounded independent review result; it does not promote the overall port to parity complete. Compiler slot released.

Formatting: `cargo fmt` in isolated validator checkout could not resolve not-yet-integrated sibling crates; direct `rustfmt --edition 2024 rust/validation/tests/source_regressions.rs` completed. No implementation files changed.

## Reproduction after integration

`CARGO_TARGET_DIR=/workspace/build/builderbob/syson/validation/target cargo test --offline --jobs 1 --manifest-path rust/validation/Cargo.toml`

The validator manifest deliberately resolves sibling `rust/crates` from the integrated repository. In the isolated validation worktree those crates are absent; use inspected copies of the exact revisions listed above in a fresh Samsung-backed build tree. `Cargo.lock` pins only the three local crates; there are no registry dependencies.

## Remaining gates

Missing Maven, Java/Node version mismatch and unverified package repository access remain separate upstream-oracle conditions per integration ledger. No package install, credentials, network change or upstream script execution attempted here. Full feature inventory/dependency closure and independently validated integrated model creation/persistence/editor path remain incomplete. New implementation changes require reviewing and recording exact revisions before rerunning this harness.
