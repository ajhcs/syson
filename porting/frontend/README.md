# Rust editor lane checkpoint

Pinned source: `6ba2ac64e6b16ee2b05752ad55395eebb87d4910` from Eclipse SysON.
Owner branch: `rust/frontend`; isolated worktree `/workspace/worktrees/builderbob/syson/frontend`.
Crate: `rust/crates/syson-editor`, dependency-free Rust, native and wasm32 compatible.
This checkpoint implements bounded algorithms, not a working user interface or full frontend parity.

## Traceable behavior ledger

| ID | Upstream evidence relative to repository root | Rust behavior | Validation | Remaining boundary |
| --- | --- | --- | --- | --- |
| FE-ROT-01 | frontend/syson-components/src/extensions/rotateNodeTool/useRotateNode.ts | prepare_rotation preserves center, swaps first selected node sizes, marks user resize, clears handles and adjacent bends | rotation_preserves_center_payload_and_only_rotates_first_selection; absent_dimensions_use_zero; empty_or_unknown_selection_does_not_modify_snapshot; two_rotations_restore_geometry_but_do_not_restore_handles_or_bends | ReactFlow store, real layout engine, history and layout synchronization absent |
| FE-ROT-02 | same hook, layout callback | finish_rotation adopts selected nodes and incident edges from real caller-provided layout by ID, retains others/order and missing results | layout_merge_adopts_all_selected_and_incident_edges_only; layout_merge_missing_results_falls_back_without_deleting_objects | Caller supplies real layout; no identity-layout substitute or successful server synchronization is claimed |
| FE-LEAF-01 | frontend/syson-components/src/nodes/package/SysMLPackageNodeLayoutHandler.ts, handleLeafNode | 75/50 minima, 8+label+8 height, defaults, user size retention, force truthiness | leaf_defaults_measurement_and_explicit_zero_defaults; user_resized_leaves_retain_larger_old_size_and_ignore_defaults; forced_dimensions_suppress_retention_but_are_not_applied_as_leaf_sizes | Parent layout, border nodes, DOM measurements, computePreviousSize helper unported |
| FE-LEAF-02 | frontend/syson-components/src/nodes/imported_package/SysMLImportedPackageNodeLayoutHandler.ts, handleLeafNode | same leaf formulas | same tests | Same helper/DOM boundary |
| FE-LEAF-03 | frontend/syson-components/src/nodes/view_frame/SysMLViewFrameNodeLayoutHandler.ts, handleLeafNode | same leaf formulas | same tests | Parent layout unported |
| FE-NOTE-01 | frontend/syson-components/src/nodes/note/SysMLNoteNode.tsx and SysMLNoteNode.test.tsx | note_paths returns body/fold geometry; note_appearance separates resolved fill/border, fade and selection | note_regression_background_change_preserves_border_and_container_transparency (adapted upstream assertion); note_paths_and_faded_selection | Theme resolution, parseInt CSS string input, DOM renderer, SVG stroke width/vector-effect, padding, handles, decorators unported |

The tests are source-derived Rust contracts. Except the adapted two upstream note assertions, cases are newly authored from inspected upstream formulas. They are not captured upstream-runtime goldens and no live differential test has run. `source-sha256.txt` fingerprints the exact inspected files.

## Shared contract

Node/edge identifiers are representation IDs, not semantic model IDs. Generic payloads preserve caller-owned uninterpreted fields; handles use strings for this bounded adapter and require a typed shape before full integration. Connection-handle clearing and edge-data absence are explicit. `prepare_rotation` and `finish_rotation` operate on owned snapshots, with no model writes, async runtime, network, Java or TypeScript dependency.

The caller must implement asynchronous cancellation, retry, request UUID generation, layout transport, `UNCHANGED` synchronization semantics and concurrency conflict policy. These APIs intentionally do not fabricate those behaviors. Upstream only mutates first selection before layout, but merges all selected IDs afterward; this distinction is preserved. Returned snapshots alone are not upstream undo/redo integration.

Dimensions use IEEE-754 f64 to match JavaScript numbers for finite ordinary diagram dimensions. No JSON/GraphQL/WASM ABI serialization is implemented. Negative/nonfinite dimensions, signed zero spelling, scientific notation and JavaScript String(number) formatting are not accepted parity evidence; note SVG formatting is verified only for normal finite dimensions in the tested fixtures. `note_paths` takes numeric CSS widths and truncates to mirror parseInt for that input subset; strings and token colors must be resolved at an explicit renderer boundary. Fading retains separate container, stroke and fill opacities rather than collapsing the upstream layered appearance.

## Dependency strategy and unported frontend surface

The crate currently uses Rust standard library only. wasm32 check does not prove a browser can instantiate or display it. A Rust/WASM frontend still needs a chosen Rust DOM/SVG binding strategy, browser host interface, layout/routing solution, measurement/font handling, UI state/effects and accessibility behavior. No generated JavaScript or JS package is required by this crate; the full product strategy remains an integration decision.

Inspected upstream `frontend/syson-components/package.json` declares React, MUI, XYFlow, Apollo/GraphQL, Sirius components, Lexical, ELK, pathfinding, i18n, tables/tree/chart/deck/Gantt/omnibox and image/export dependencies. None of those behaviors is supplied by this bounded crate. Upstream source remains in this fork as evidence and does not count as the Rust runtime implementation.

Still unported: application shell/routing, auth UI and permissions, projects/explorer/forms, semantic direct edit, rich/textual expression editing, import/export and textual SysML insertion, all custom-node renderers and parent layout, palettes/tools/connections, theme/appearance mutation, extension registry, display preference queries, notifications/error surfaces, keyboard/screen-reader/accessible focus behavior, collaborative subscriptions, history/undo/redo, persistence recovery and realistic end-to-end model creation. The central parity ledger must keep those incomplete.

## Local checks observed

- `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/workspace/build/builderbob/syson/frontend cargo test --offline --manifest-path rust/crates/syson-editor/Cargo.toml`: 11 passed.
- Same environment, `cargo check --offline --target wasm32-unknown-unknown --manifest-path rust/crates/syson-editor/Cargo.toml`: passed.
- No upstream scripts, displays, browsers, paid calls, runtime oracle or screenshot comparisons executed.

Rust source ports retain Obeo copyright attribution and EPL-2.0 SPDX identification. Root upstream `LICENSE`, `NOTICE`, `LICENSE-LGPL` and library obligations remain authoritative and unchanged. This crate does not relicense upstream SysML/KerML libraries.
