# B25c: Prepare the native ViewUsage exposedElement migration

Older resources can retain ViewUsage `exposedElement` values that became derived.
Prepare participant `2025.6.0-202506110000` in Rust, removing only that exact
serialized feature from the native helper's selected object-valued children.
The source comment's `exposeElement` misspelling, neighboring properties, node
and semantic identities, and unrelated classes remain preserved.

The B25S prerequisite supplies bounded native-compatible traversal and namespace
handling. Callers use `Prepared::to_json()` between participants and resource
admission. This PR returns a complete owned result and selected participant
version; B06 still must publish project state and migration records atomically.

Validation: 18 integration tests (9 new plus 9 shared), formatting and all-target
Clippy pass offline/locked; all 51 committed input hashes match `b25c-01`.
Thirteen identical-input cases match the selected Java source runtime for
success/refusal, version, resource predicate and JSON values. Tests also run
actual Document admission and B01 preparation with stable identities and replay.
The shared parent retains four exact pinned Java classes compiled with annotation
processing off against nine identified release dependency JARs; 728 predicate
and four ordered-composition cases are common evidence.

This does not qualify the whole SysON artifact/source closure, complete model
deserialization, SQL publication or GUI workflow. Native exception text/classes
and non-namespace object-key order are not asserted equal. Exact root inputs,
streams and this unit's native/Rust observations are retained separately from
source predictions and the original prepared packet.

Base: `codex/syson-rp-b25-support`, exact
`cbf8a9f7a56f459a2374962e96010233fe711801`, after B01/#70. Land prerequisites
first, then retarget, inspect and revalidate; never merge into the support branch.
Two commits separate source/regressions from exact evidence.
