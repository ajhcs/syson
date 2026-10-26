# C01d: Project the native PostgreSQL catalog into Rust comparison obligations

B05 needs the database contract actually created by native SysON. Add a read-only
Rust CLI that preserves native table, column, table-constraint, index and applied
migration descriptors as stable B05 obligations. Validate duplicate identities,
parent/attribute references, complete foreign keys and target-owned backing indexes;
keep exact PostgreSQL type/default/expression text and migration order unchanged.
Input/output and identity limits, incremental serialization accounting and
regular-file guards bound projection without executing SQL or changing the input.

Final verification passes ten regressions, formatting and all-target Clippy
under offline/locked execution. Five actual CLI cases preserve every one of the
164 native descriptors and the 20-row migration order, and refuse malformed,
missing-argument, directory and symlink inputs with no stdout. All nine frozen
inputs remain unchanged. Independent peer review accepted the corrected source.
The supplementary 29 shipped DB XML resources match exact pinned Sirius Git bytes.

Retain the original nine-case PASS, subsequent FK finding, CLI admission refusal
before any launch, and shared native capture's overall partial status. Only its
successful catalog capture is credited here. Exact runners, raw streams, native
fixture and selected resource inventory stay reviewable. The approved cohesive
budget exception is 520 production/1050 authored lines; final counts are 518/1042
including SQL and manifest, excluding generated lock/data and prose.

Every obligation remains pending a Rust comparison. Parent C01, whole-build source
correspondence and B05 behavior/schema acceptance stay open; sequences/views,
functions/triggers/policies/privileges, type/extension and partition semantics,
application write isolation, DB driver/durability and UI/performance are separate.
The read-only capture transaction does not establish application write isolation.

Two commits separate implementation/regressions/query/native fixture from exact
execution evidence and disposition. Base `ad814c232ee24109639c1d52239cbeec6f4a3e92`;
no application manifest/wiring, dependency acquisition, service change, push or
merge by the source author.
