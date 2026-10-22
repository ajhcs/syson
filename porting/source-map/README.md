# Pinned source and behavior evidence

Baseline: Eclipse SysON commit `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`, fork `https://github.com/ajhcs/syson`. The Maven and npm version declarations are `2026.9.3`. This inventory reads Git blobs at that commit, independent of later worktree changes. No upstream script, server, import executable, test, or build was run by this lane.

`summary.json` reports the finite tracked-tree coverage: 3,552 tracked files, 36 Maven manifests, seven npm manifests, 186 Ecore classifiers, 390 user-documentation headings, 18 bundled OpenAPI operations, 50 GraphQL type declarations, and 534 upstream test sources/resources. These are inventory counts, not implemented or verified features. `code-surface.jsonl` is a syntactic search index, including generated Java methods and frontend files; entries are not claims of independent behavior. A heading may describe an absent or future capability. All implementation statuses remain `unassessed`; all oracle execution statuses remain `not_run`.

## Artifacts and reconciliation

| Artifact | Meaning and use |
| --- | --- |
| `source-files.jsonl` | Every tracked baseline file, SHA256, byte count, suffix, inline SPDX declarations. File hashes establish supplied bytes only. |
| `modules.jsonl` | Every Maven/npm manifest, aggregation children, scripts/engines, main/test source counts where applicable. |
| `declared-dependencies.jsonl` | Direct manifest declarations, including managed and plugin dependency entries. Versions may be inherited/property expressions; this is not a resolved lockfile or transitive SBOM. |
| `documented-features.jsonl` | Each user-manual heading outside historical release notes, exact path/line, unassessed parity and test slots. |
| `code-surface.jsonl` | Public Java callable candidates and frontend source modules, path/line where parsed. Regex extraction is not a Java parser and may miss declarations. |
| `metamodel.jsonl` | All classifiers from both tracked Ecore models, supertypes, structural features and operation declarations. Inherited operations require semantic reconciliation. |
| `graphql-types.jsonl`, `graphql-operations.jsonl` | Local schema declarations and named frontend query/mutation/subscription occurrences. Imported Sirius schema and anonymous operations are not comprehensively inventoried here. |
| `rest-endpoints.jsonl` | Operations from the bundled Sirius Web OpenAPI document, including request/response definitions or references. Operation availability and failures require oracle validation. |
| `external-java-imports.jsonl` | External imported package prefixes and referencing source files; inherited Sirius/EMF/AQL behavior must be mapped separately. |
| `migration-evidence.jsonl` | Tracked migration participants/resources/fixtures; presence does not prove migration compatibility. |
| `upstream-tests.jsonl` | Test source/resource paths, syntactic test annotations/case labels where extractable; none run. Fixtures are not tests by themselves. |
| `license-evidence.jsonl` | License/notice files and inline SPDX declarations. Missing inline markers do not imply public domain; bundled/transitive obligations need separate review. |
| `behavior-boundaries.md`, `dependency-strategy.md` | Reviewed source boundaries, cross-lane contracts, runtime exclusions, unresolved decisions and required evidence. |

Regenerate with `python3 porting/source-map/inventory.py` from a checkout containing the pinned object. The generator uses only Python's standard library and `git ls-tree`/`git show`; it writes only this subtree. Do not substitute a release image tag for the pinned source oracle. Root compose defaults to `v2026.9.0`, whereas nested application compose uses `latest`; neither identifies this commit's build.

## Completion conditions

Every feature entry needs a responsible lane, linked Rust code, positive/negative test cases, upstream oracle evidence or a precise reason no oracle is available, compatibility disposition, and independent review. Inherited Sirius behavior requires additional source pinning/inventory; this repository alone is not the whole product implementation. Accessibility, collaboration/concurrency, retry/cancellation, undo/redo, recovery and graphical layout need runtime and interaction evidence. An empty Rust-test list means missing evidence, never implicit support. No behavior, license clearance, or full port completion is asserted by this source inventory.
