# Explicit literal duplicate-key compatibility on current main

This adopts the complete RR24a provider change from PR #100 (`9c95c9ef5fd297f97451e3884ba2eb618f3d2396`), including all five original compatibility tests. The eleven strict tests, manifest and lock remain unchanged. The existing `parse` API still rejects duplicate keys; compatibility requires an explicit `parse_with_duplicate_keys` call with `DuplicateKeys::LastWins`.

LastWins keeps the final value at each decoded key's first insertion position. Every physical key/value/scalar occurrence is charged before collapse, including overwritten values; original raw bytes, numeric tokens and ordinary marker-named objects remain authoritative. Cancellation, syntax validation and refusal behavior remain explicit.

[qualification.json](qualification.json) records 161 passing cases across the six provider/caller crates, including the catalogue default-feature suite. [corpus-replay.json](corpus-replay.json) records a fresh Rust replay of 94 historical resources and 109,673 nodes, with complete output-byte equality to the historical Rust index. Original donor evidence and failures remain historical inputs; these results support only their named Rust cases.

The migration, project-preparation and wire adopters are not yet on this main. Their RR24/RR22 literal caller gates remain open, as do native, durable storage, UI and performance acceptance. This change releases no existing hold and touches no reserved human-contributor file.

The strict release probe uses only the unchanged public API and can run as identical source on baseline and candidate. It constructs unique-key 1,000/5,000/10,000-member objects before timing, then measures parse and materialization separately.

The compatibility probe uses four physical occurrences of each distinct key at the same physical-key sizes. It checks first-key positions, final values, full physical statistics, raw-byte identity, strict rejection and a one-key-under physical quota refusal; it is a separate workload, not a strict/native speed comparison.

Both probes use two warmups and seven measured repetitions, reporting every raw value and min/median/max. Fixtures and validation remain outside timers. [performance.json](performance.json) retains matched baseline, original donor and final Entry-optimized runs. The final 1,000-key strict parse median remains about 19% above baseline; the 5,000/10,000-key medians are about 0.5%/1% above it. These named API observations establish no timing cutoff, p95, native speedup or UI/capacity acceptance.
Materialization follows a complete validation walk of the parsed tree, so these timings cover a pre-exercised tree; the strict baseline and candidate use the same sequence.
