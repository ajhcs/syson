# C01e: selected source-delta reference recipe

This unit derives and verifies one comparison JAR from the pinned official artifact.
Its source-aware Rust planner maps the label-to-target delta into seven Java class
families and 114 SVG resources across three nested SysON modules. Two PNG release
screenshots and two ADOC files remain documentation-only dispositions. The exact
125-row/312-container input packet is committed once as the test fixture; repeated
full packets, archive inventories, compiled libraries/JARs and database data stay
external with hashes and paths in `external-artifacts.json`.

The original nine tests, formatting, Clippy and build PASS are retained in
`captures/c01e-01/`. Independent review then identified a selected SVG versioned
shadow admission gap. The final source adds that narrow guard; `captures/c01e-02/`
runs its isolated new test plus formatting/Clippy/build. The nine passed tests were
not replayed. All eight final inputs match committed bytes. The original planning
run and final run both pass; final `reference-plan-02/plan.json` is byte-identical
to the recipe consumed by the already completed runtime gates, as recorded in
`reference-final-recipe-link.json`. No native gate was replayed for this input-only
guard. Independent peer review accepted the final source.

The coordinator compiled exactly seven target Java sources against all 312 staged
runtime JARs with `--release 21 -proc:none -implicit:none` and an empty sourcepath.
All seven resulting classes have major version 65; no dollar companions were
emitted. Original source/library inputs remained unchanged. This is selected
source compilation, not a Maven/frontend rebuild or resolved dependency attestation.

The actual reassembly changes exactly three outer nested-JAR payloads; 309 nested
JARs remain byte-identical. Full readback verifies every unselected uncompressed
payload and declared ZIP metadata, with explicit deterministic metadata for new
entries. Entry ordering, methods, flags, timestamps, extras, comments and attributes
are preserved. CRC/uncompressed-size changes belong only to declared replacements;
physical offsets and compressed lengths are representation observations, without
a compressed-stream identity claim. Spring Boot nested JAR entries remain STORED.
The executed source of truth is the final `reference-repack-01/executed-recipe.py`,
which includes the coordinator's before/after input freeze; no earlier helper hash
is attributed to execution.

The only signature-like exception is this exact pinned artifact's zero-byte
`META-INF/BOOT.SF`, whose empty-content hash is checked and whose bytes remain
unchanged. Original filename-based candidate observations and the separate narrow
classification recipe/receipt are retained. Other signature files, digest-bearing
manifests and multi-release shadows refuse. Outer descriptor/UTF-8 flags 2056 and
changed nested UTF-8 flags 2048 are preserved, rather than normalized away.

The derived JAR SHA256 is
`7d94f1b7cd4a5d88f55a6a2c11d62eab032f833454a58ad842033d30dd704386`.
It passes 26 selected native icon/style provider cases and a fresh owned startup
serving all 114 SVG bodies byte-equal to target Git objects, plus two GraphQL
`__typename` probes. The native receipt records all owned processes stopped and
unchanged inputs. These checks establish selected reference behavior and serving;
they do not establish rendered browser appearance, general API/schema equivalence,
application usability/performance or durable transactions.

Original provider attempts remain intact. `provider-original-01` has the import
compile failure. `provider-original-02` remains FAIL, retaining 12 ItemUsage outputs
and the first null-reference result before a wrong-direction expectation failed.
`provider-original-03` passes 14 remaining selected reference/fork/join cases;
`provider-derived-01` passes all 26 target cases. No failed attempt is relabeled.
The source author performed only the explicitly authorized Git/ZIP read inspection
and staging, source formatting and metadata checks; compilers, reassembly, provider
execution and native application/database lifecycle were coordinator-owned.

The reviewed cohesive budget exception is 475 production/900 authored lines;
final Rust production is 469, tests 340 and manifest 11, totaling 820. Coordinator
oracle/inspection scripts and raw execution evidence are a separate review block.
Parent C01, independent full-build/source binding, inherited/SNAPSHOT dependency
provenance and complete Rust feature/DB/UI acceptance remain OPEN. The derived
comparison artifact is not a new official release or a Java/Python production
fallback for the Rust application. Original artifact/source labels remain distinct.
