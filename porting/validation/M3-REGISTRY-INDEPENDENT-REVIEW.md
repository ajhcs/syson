# Independent review: generated Ecore declaration registry

Reviewed registry commit: `5c73e3a8e6d4215f98e6c4c067a19c90783456b2`. Upstream baseline: `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`. Disposition: accepted as a bounded declaration/structural-query registry. This validator authored no registry implementation. No executable metamodel semantics or product parity completion is implied.

The independent `export_metamodel` Rust example observes the **compiled registry** through its public API. A separate Python checker parses the two pinned source Ecore blobs directly, compares all generated node tags/attributes/children and package provenance, computes expected type/default/flag/opposite/enum/operation/parameter metadata, and derives local inheritance and feature-name candidates. It enumerates independent inheritance paths and deduplicates declaring feature identities, rather than reusing the registry's global visited-set implementation. XML formatting whitespace is intentionally excluded; the checker rejects non-whitespace text/tail so semantic XML text cannot be silently omitted.

Executed results:

- All **186 classifiers** matched: 182 domain classifiers (175 EClasses and 7 EEnums) plus 4 separate custom-node EClasses.
- All **419 structural features** matched (415 domain plus 4 custom-node references), along with **70 operation declarations, 57 parameters, 309 annotations and 19 enum literals**.
- All **34,596 local subtype pairs** and **63,798 structural feature-name queries** matched the independently derived pinned declaration graph, including own-name shadowing per branch, multiple inheritance, diamond identity deduplication and external unresolved ancestors.
- Negative mutations removing a nested operation parameter, flipping a typed schema default, or removing a known local ancestor were each rejected by the independent comparison.
- Existing checker unit suite: **56 passed**. Native observation build used one job, offline/locked, after inspecting the dependency-free registry manifest, handwritten implementation and generator. No upstream runtime, parser, graphical editor or semantic operation was executed.

The full observation JSON is reproducible build output rather than a large checked-in product fixture. Its SHA256 and the reviewed registry/exporter/checker hashes are recorded in `M3-REGISTRY-SUMMARY.json`.

## Interpretation limits

`ClassifierId` names are typed local declaration identities. Do not serialize enum discriminants, memory positions or generated declaration ordering as stable model element IDs. Package-qualified lookup keeps the domain and custom-node namespaces distinct; external identities remain full original reference strings.

Ecore default literals are declaration values. They do not establish actual initialized object values, enum/datatype initialization, omitted JSON defaults or handwritten derived behavior. For example, Ecore Feature.isComposite false and a creation initializer setting true describe separate stages. Every declared operation is metadata only. The registry offers no invocation API and supplies no successful semantic stub result.

`FeatureLookup::unique()` means one known **local candidate**. In the checked graph, **3,917 queries** have both local candidates and unresolved external ancestry. For example AcceptActionUsage.behavior resolves locally to Step.behavior while EModelElement ancestry remains unresolved on another branch. The result is not a proof of global feature uniqueness, semantic redefinition/subsetting or EMF feature-ID ordering. This boundary was sent to the implementation owner before model integration; callers must preserve unresolved cases or explicitly reject an operation requiring external semantics.

The comparison accepts pinned declaration capture and documented structural queries only. Full EMF inheritance behavior, external features, opposite/proxy updates, derived fields, initializer effects, constraints, operations, serialization and runtime differential compatibility remain open.

## Reproduction after integration

Use an approved build directory, with the reviewed registry revision present:

```
CARGO_TARGET_DIR="${SYSON_BUILD_DIR}/registry-target" cargo run --offline --locked --jobs 1 --manifest-path rust/validation/Cargo.toml --example export_metamodel > "${SYSON_BUILD_DIR}/registry-observations.json"
PYTHONDONTWRITEBYTECODE=1 python3 rust/validation/scripts/check_compiled_metamodel.py "${SYSON_BUILD_DIR}/registry-observations.json"
```

Evidence checkers using assertions reject Python optimization mode explicitly. Run comparisons with ordinary Python, preserving all assertions.
