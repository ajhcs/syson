# B01: selected library-ID migration preparation

This crate prepares the two standard-library-ID participant effects in Rust. It
uses the existing pinned `syson-persistence` rules, actual `Document::from_json`
admission shared with F27, and the pinned metamodel to distinguish references
from attributes and containment. It performs no I/O, loading or subprocess work.

Semantic participant `2025.8.0-202506301600` runs across all documents before
matching diagram participant `2025.8.0-202506301700`. Output includes explicit,
per-input selected-participant watermarks. These are **not native global migration
history**: existing JSON `migration` content stays preserved. Only absent or exact
selected watermarks are admitted. Unknown versions refuse. Replaying stamped
output skips the resolver and prevents repeated old→new→other mappings.

The resolver supplies an immutable no-load snapshot and returns actual Element
identity/resource, unresolved/non-Element, or a failure. Diagram hits in a
nonlibrary resource suppress ID-map fallback, matching the native participant.
Semantic references retain URI prefixes, explicit resolver precedence and mapped
fallback. Numeric/nonstring `targetObjectId` values stay unchanged. Unknown EClass,
unclassified library-bearing feature or unsupported reference shape refuses;
unknown nonlibrary metadata remains preserved. This is a bounded admission policy,
not complete EMF serialization or all migrations.

Inputs admit at most 64 combined semantic documents/representations, 1 MiB of
serialized JSON content each, 8 MiB combined content, depth 64, and 1 KiB per kind
or selected watermark string. Input checks precede cloning. Each replacement's
escaped JSON size is counted before formatting/cloning it. Cumulative positive
growth is reserved against both per-document and project remaining bytes; shrinking
a value does not refund growth. Oversized replacements or repeated expansions
refuse before the next allocation. Output passes byte budgets and actual semantic
Document admission again. Arbitrary ID spellings remain allowed as in the pure
participants; resolver-owned allocations/side effects are outside this API.

`prepare` returns a complete new envelope only on success; input documents and
watermarks never change on failure. It does not persist them. B06 must commit
semantics, representations and selected watermarks atomically. Current F27 Change
cannot durably store a complete project envelope or these migration records, so
this PR does not claim native project-load migration completion.

Native JSON fixtures are extracted as data from the pinned integration-test SQL,
without executing SQL. Four selected mapping lines come from the pinned migration
resource. A regression asserts the two stable IDs asserted by the Java integration
test. This is source-derived fixture verification, not a new Java/native runtime
differential. The original SQL, mapping, participant and native-test hashes are in
`source-provenance.json`.

Coordinator execution `b01-01` passed on 2026-10-26: 11 integration tests,
formatting and all-target Clippy with warnings denied. The offline generated lock
contains 11 registry package tuples already present in existing locks. All 40
source input hashes were equal before/after execution and match the committed
source. No production source changed after capture. The exact executed runner,
receipt and all six raw streams are retained under `captures/b01-01/`.

`validation.json` records source/stream/runner hash verification and disposition.
The coordinator retained original input snapshots externally at the path recorded
there. Build outputs were isolated under `/workspace`; execution was serial,
offline and locked. No native runtime, database, browser or performance benchmark
ran for B01. Passing budget regressions establishes bounded admission behavior,
not a measured RSS or speed claim. Complete durable migration remains separate.
