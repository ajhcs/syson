The public-import contract now has a bounded Rust evaluator on immutable caller
observations. It preserves ordered public traversal, exclusions, per-helper
visited state, alias reference observations and lazy name-conflict reads.
Supplied import-result mode remains distinct from graph traversal. Reached
unsupported or unresolved data returns explicit errors without partial success.

The implementation preserves a source-context quirk: nested helper dispatch
returns empty for Base Import; the direct API's null is represented by a Rust
null disposition, while an outer Namespace pipeline cannot flatten that null.
Native bindings, proxies, Type/includeAll families and URI loading are excluded.
The standalone crate avoids the existing held graph-adapter paths.

Validation: 15 tests pass, including replay of the actual 19 F02 fixture cases
in one test (18 original expected dispositions plus one explicit unsupported
includeAll case), formatting and all-target Clippy. Source, stream and runner
hashes are retained. Expectations are manually source-derived, not Java/EMF
runtime goldens. No upstream runtime comparison or whole-area parity is claimed.

Stacked on F02 / PR #34 at `15501a5ff2459cd1056809b70f23d0d2ae895b8a`.
Merge/review prerequisite first; after it lands, retarget this PR to the merged
integration base and inspect/revalidate the resulting diff. Do not merge into
the temporary prerequisite branch. No merge or shared manifest change is included.
