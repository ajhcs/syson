# Literal-safe shared migration preparation

B25S1 adopts the complete shared `prepare`/`Prepared::to_json` API from the corrected #104 source. Its 256-line body is identical to #103. All nine original shared tests and the original 41-line probe remain unchanged. Participant exports remain with their separate B25a–d units; the complete donor and later caller bodies are retained in `main-batch33`.

Preparation uses the explicit RR24a LastWins policy. The first decoded namespace key keeps its position and final value. Matching parents select their object-valued children through the actual metamodel subtype relation; each removal pass observes a snapshot. `to_json` preserves current namespace order, literal marker objects and numeric tokens for re-entry. It produces transformed JSON, rather than retaining the original formatting.

Input admission caps raw JSON at 1 MiB, root-zero depth 64, 65,536 physical values/keys and a conservative 2-MiB scalar-copy charge. Overwritten duplicate occurrences count before collapse. Serialization rechecks the edited tree’s depth/value limits and its 1-MiB output limit. These bounds do not establish allocator/RSS or latency guarantees. Failed preparation returns no candidate and leaves the input and earlier outputs unchanged.

Five additional cases use manual marker-object expectations and raw numeric fixtures. They cover encoded node-looking strings, exact numeric spellings, namespace duplicates/order, byte/depth/physical-value boundaries and late-pass failure. The actual current Document/B01 handoff checks fixed library and diagram IDs, literal payloads, independent stamps and serialization/reparse replay. It performs no file or database persistence.

The coordinator qualifies the frozen source with offline/locked tests, formatting, all-target Clippy and the release probe. Source authoring does not establish that these checks passed. The one-shot probe runs as the same actual binary for two warmups and seven measured processes; inner preparation times and whole-process RSS remain distinct observations, with no timing veto or native/UI capacity claim.

B25S, B25d, RR24 and RR22 retain their full acceptance obligations. The later #104 preparation-reader repair, all seven preparation regressions, wire transport regression and both named literal caller gates remain required on the composed source. Native comparison, whole-project atomic publication, database restart, UI workflows and full SysON parity remain open. This adoption releases no existing hold.
