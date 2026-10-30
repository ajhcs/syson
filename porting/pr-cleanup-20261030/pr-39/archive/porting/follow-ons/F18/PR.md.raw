Flat layout previously rejected every parent relationship. This adds a separate
package-only forest API that lays out fresh sibling groups with F16, computes
source-derived parent padding/header/default/user-resized minima, and reports
relative and absolute coordinates. Retained positions remain explicit caller
observations, including their source clamps and possible overlaps.

The API rejects cycles, unknown parents/endpoints, duplicate identities,
cross-parent edges, invalid geometry, arithmetic/precision failures, resource
limits and cancellation without returning partial layouts. F15 supplies measured
leaf dimensions in a real synthetic-host integration test. Border ports, forced
dimensions, new-position overlays and general container behavior are excluded.
Fresh sibling placement is proposed Rust behavior; no ELK/native rendering parity
is claimed.

Validation: 30 tests pass (15 new container tests, 15 existing flat tests),
including 24 forest scenarios in one test, plus formatting, all-target Clippy,
three pinned source files and committed-versus-executed input hashes. Original
streams, runner, source excerpts and root review are retained. No upstream or
browser runtime comparison occurred.

This PR targets the immutable F15/F16 dependency snapshot
`codex/syson-base-f18-20261025`. Merge both prerequisites (#27 and #29) first,
then retarget to the merged integration base, inspect the diff and revalidate.
Do not merge into the snapshot branch. Shared lib/manifest/lock wiring is narrow
and coordinator-owned; the new dependency is local and test-only. No merge,
service/package change or existing hold change is included.
