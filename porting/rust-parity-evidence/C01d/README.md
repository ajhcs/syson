# C01d: native PostgreSQL contract projection

The final Rust development tool preserves an actual native SysON PostgreSQL
catalog as deterministic B05 comparison obligations. The supplied native data
contains 15 tables, 87 columns, 26 table constraints, 16 indexes and 20 applied
migration rows: 164 obligations, each still awaiting a Rust comparison.

`captures/c01d-02/` records the final ten-test, formatting and all-target Clippy
PASS under offline/locked, single-job coordinator execution. Its nine captured
inputs are unchanged and preserved, including the historical preparation file,
SQL template, real native fixture and crate README. `c01d-cli-02/` records five
actual CLI cases and exact equality of every observed native descriptor and all
20 migration positions. The coordinator archived the executed runners and raw
streams; the binary is identified by SHA256 rather than included here.

`native-01/` retains the original fresh owned PostgreSQL/application capture. Its
overall status remains `PARTIAL_NATIVE_CAPTURE`: the catalog command exited zero
and the separate GraphQL request returned an error. The original error, logs,
configuration, exact SQL/psql bindings, tool/artifact hashes and shutdown/input
checks remain intact. The SQL template hash is not a self-hash of a substituted
query: exact metadata values are psql bindings recorded in the native receipt.
Read-only/repeatable-read describe this capture transaction, not application
write isolation. All owned processes were stopped by the coordinator.

The first nine-case PASS remains in `captures/c01d-01/`, with its original source
and outputs. Independent review later found an incomplete-FK admission defect,
which the final code and actual-native-fixture mutation regression address.
`c01d-cli-01/` contains an admission refusal caused by changed preparation metadata
before launch; no executable ran and no runtime PASS is attributed to that attempt.
Earlier formatting/edit overlap and output amplification findings are separately
recorded in `review-disposition.json`. Original PASS/refusal statuses are preserved.

The tool bounds input/output to 8 MiB, identities to a candidate 4096 bytes, and
individual collections. A counting writer charges the output envelope, each
complete obligation before retaining it, migration-order IDs and final output.
The long-prefix regression must refuse before a deliberately invalid final row;
SQL/default/type strings remain opaque and unchanged within the admitted bounds.
These limits qualify this projection; they are not universal PostgreSQL limits
or an application performance benchmark.

The coordinator approved one cohesive exception of at most 520 production and
1050 authored lines. Final counts are 518 Rust production + 457 Rust tests + 56
SQL + 11 manifest = 1042 authored lines, excluding generated lock/data and prose.
Aggregate serialization bounds and complete FK proofs belong to the same catalog
cross-reference boundary; no source minification or artificial API split was used.

`artifact-resources/` preserves the coordinator's extractor, complete inventory,
312 nested-JAR hashes, 220 Maven/build-metadata observations and 76 selected raw
resources (29 DB XML, 46 GraphQL SDL and application properties). All 29 DB XML
payloads were independently rechecked against Git objects at Sirius commit
`3191e2b8990373afee3415f6b26f8c5039a28321`. Exact resource equality is supplementary
source correspondence. Metadata coordinate/version declarations, including the
EMF JSON SNAPSHOT, do not attest the whole build or identify original build inputs.
No extraction helper or native executable was rerun by the source author.

Parent C01, whole-build source binding and B05 Rust schema comparison remain open.
Sequences, views, functions, triggers, policies, privileges, extension/type
semantics, partition/inheritance relationships and application transaction behavior
are outside this selected projection. No DB driver, durable Rust transaction,
resolver/auth/UI/usability or application performance acceptance is claimed.
The artifact label `b380dada415ce05155150f23cb3b74ae21cc45e4` remains distinct
from target `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`; the broader source delta
and effective dependency/source closure remain under parent C01.

Source preparation is historical frozen input. New verification/disposition files
supply current results without editing captured input. `SHA256SUMS` covers all
retained evidence except itself. Empty fixture directories, symlinks and native
working databases are represented by receipts/commands rather than copied; raw
regular-file inputs/outputs are retained. Runners retain their original absolute
paths and must be reviewed and adapted to a fresh owned run directory before reuse.
