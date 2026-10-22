# Persistence compatibility lane

Baseline `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`; branch `rust/persistence`; isolated worktree `/workspace/worktrees/builderbob/syson/persistence`. Owns `rust/crates/syson-persistence` and this documentation only.

This independently buildable std-only Rust crate ports library-ID migration algorithms. It does not provide full SysON persistence. It neither writes files nor opens databases and introduces no replacement storage format.

## Traceable ledger

| ID | Exact upstream source | Rust implementation and tests | Scope status |
| --- | --- | --- | --- |
| PS-MAP-01 | backend/application/syson-application-configuration/src/main/java/org/eclipse/syson/application/migration/StandardLibrariesIdsResourceToMapParser.java | parse_id_map; parser_matches_java_split_trim_duplicate_and_line_end_rules; supplied_table_and_upstream_fixture_preserve_expected_string_and_action_ids | Line parsing ported, resource loader/I/O/logging excluded |
| PS-REF-01 | same migration directory, StandardLibrariesElementsReferencesMigrationParticipant.java | migrate_reference_uri; semantic_uri_from_upstream_sql_maps_to_upstream_expected_element_id; live_element_resolution_takes_precedence_over_fallback; ordinary_and_containment_references_never_call_resolver; unknown_reference_unchanged_and_missing_fragment_follows_upstream_slice_rules | Decision/string transformation ported, actual EMF URI parsing and ResourceSet lookup remain unported |
| PS-DIAG-01 | same migration directory, StandardLibrariesElementsDiagramMigrationParticipant.java | migrate_diagram_target; diagram_start_id_matches_upstream_assertion_and_is_idempotent_without_resolution; resolved_nonlibrary_element_blocks_map_fallback; non_target_and_non_string_json_attributes_never_resolve | Attribute decision ported, JSON parsing/traversal/write, IObjectSearchService and migration scheduling remain unported |
| PS-EXPORT-01 | backend/application/syson-sysml-export/src/main/java/org/eclipse/syson/sysml/export/SysMLv2EditingContextPersistenceFilter.java; backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/util/ElementUtil.java | is_standard_library_resource and should_persist_in_project_download; project_download_filter_preserves_source_prefix_and_null_semantics | Resource URI prefix predicate ported, full project download remains unported |

`source-sha256.txt` fingerprints each source, original migration table and fixture/test. Tests compile-time include the original table and SQL directly from their upstream paths; no copied table is silently substituted. The original table contains 123,773 rows and 123,773 distinct keys/target IDs. Tests reuse upstream `StandardLibrariesIdsMigrationParticipantTest` expectations: String `76028d3d-69a4-5e12-9002-ce403e0244bd` and Action::start `9a0d2905-0f9c-5bb4-af74-9780d6db1817`. Existing SQL contains the old IDs, including the semantic reference actually transformed by the Rust test.

The SQL is inspected and read as bytes only, never executed. Passing these tests does not imply the upstream Spring/Postgres integration test passed; no upstream runtime differential comparison has run.

## Contracts and precise limits

`parse_id_map` accepts caller-supplied UTF-8 text. It matches Java `String.trim` (characters through U+0020), duplicate overwrite, CR/LF/CRLF splitting and Java split("=") trailing-empty suppression. It does not implement InputStreamReader's environment-selected charset, resource loading, swallowed I/O exceptions or partial map return after an I/O error. Callers must not equate successful text parsing with successful protected storage reads.

Semantic migration takes a resolver callback returning an elementId only for an actual SysML Element, corresponding to `ResourceSet.getEObject(uri,false)`. That callback must implement actual EMF URI interpretation/resolution without resource loading; no such resolver is hidden in this crate. The upstream `sysml:` type prefix is removed for lookup only, while output retains the original prefix before `#`. Upstream's odd missing-`#` behavior is preserved. Selection is substring-based for reference migration and prefix-based for resource classification, exactly as inspected; these predicates are not generalized URI security checks.

Diagram migration returns an optional replacement string for a `targetObjectId` string attribute. Non-string values are represented by None. A found non-library Element prevents table fallback; missing/non-Element lookup allows table fallback. The resolver adapter must map a found non-Element to None, matching upstream filtering. `ResolvedElement` requires nonnull element IDs; null Java IDs are outside this boundary and not claimed compatible. No JSON schema or model semantics is invented. Published participant versions and representation kind are exact upstream constants; selection/order based on stored migration state remain unimplemented.

The production crate does not embed or load the 123,773-entry map; its caller must load the pinned resource under the applicable distribution/license requirements. Tests using `include_str!` require the preserved upstream checkout layout. Root `LICENSE`, `NOTICE`, `LICENSE-LGPL` and source/resource obligations remain intact. Ported source retains Obeo attribution and EPL-2.0 SPDX markers; no upstream library relicensing is implied.

## Remaining persistence work

Unported: Sirius Web/Postgres storage layer and schema; EMF JSON identity/reference handling and full resource set; document/project/representation CRUD; JSON readers/writers; transactional writes, concurrency, cancellation and retry; crash recovery, backup/restore and integrity; authorization and isolation; migration registry/order/version state; annotation, connector, control-node, view-usage and diagram-description migration participants; cross-version round trips; textual SysML and project archive import/export. A source inventory is not behavior acceptance. Upstream SQL integration fixtures must eventually run in an explicitly isolated reference environment and compare full resulting Rust artifacts before claiming database or migration parity.

## Observed checks

`CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/workspace/build/builderbob/syson/persistence cargo test --offline --manifest-path rust/crates/syson-persistence/Cargo.toml`: 10 passed, zero failed. No third-party dependencies, network, database, Java process, external runtime, service change or production state was used. Cargo formatting checked in the isolated lane. One compile job only.
