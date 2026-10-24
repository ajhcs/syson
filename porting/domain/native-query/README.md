# Native query and resource snapshot adapter

The existing Rust lookup algorithm now accepts independent raw containment, derived owned elements, effective names, physical-container state and public-import observations. `Model::find_by_name_and_type` delegates to that engine and retains its historical projection contract. The native adapter uses separate facts, so a named Relationship remains visible to unqualified raw traversal while qualified lookup follows its owned related elements. All owned Relationships are flattened for that derived lookup, including non-Membership relationships.

`syson_initializer_graph::native_query` adds immutable, ordered `ResourceSet`/`ResourceSnapshot` values. Results are `ObjectId { resource, element }`: caller resource keys, JSON resource-node IDs and SysML semantic IDs remain distinct. Equal semantic IDs in different resources do not alias. The adapter performs no URI loading or network/database access.

```rust,ignore
use syson_initializer_graph::{ReferenceEncoding, native_query::*};
use syson_metamodel::ClassifierId;

let resources = ResourceSet::new(vec![
    ResourceSnapshot::new("model", model_document),
    ResourceSnapshot::new("library", library_document),
])?;
let context = ResourceContext::Member(ObjectId::new("model", context_element_id));
let result = resources.find_from(
    &context, "StandardViewDefinitions::GeneralView", ClassifierId::ViewDefinition,
    ReferenceEncoding::SemanticElementIds,
)?;
```

The example describes API usage; runnable cases live in `tests/native_query.rs`. `find_general_view` supplies that exact pinned query. This is standalone lookup: `NativeContext::general_view_definition` and ViewUsage graph application are still unbound. A successful query is not a claim of full ViewUsage construction or external-reference mutation support.

## Pinned source contract

Baseline remains `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`. [PROVENANCE.json](PROVENANCE.json) records exact source-file hashes and their equality with the pinned Git objects. The selected source methods establish:

- `ElementUtil`: root order, Java-compatible split/name matching, raw preorder for unqualified lookup, actual `eContainer()==null` for unnamed top-level namespaces, first-name owned precedence before final type checking, and public-import fallback only after owned-name miss. A wrong-type or incomplete owned path does not fall back to imports or a later same-name sibling.
- `ElementImpl`: semantic owner follows the owning Relationship's owning related Element; derived owned elements flatten all owned Relationships. The existing projected `Model::qualified_name` remains separate from this adapter.
- `FeatureImpl` and the existing local name adapter: effective names/short names, own-name-or-short-name suppression of inherited naming, ordered first redefinition and explicit local-reference encoding. Unresolved names remain errors when reached; type filtering and successful earlier matches avoid unrelated getter evaluation.
- `ConjugatedPortDefinitionImpl`: effective name ignores declaredName and follows the first owned PortConjugation to the original PortDefinition, adding `~`. Source-null names, unresolved references, proxy-shaped values, wrong types and cycles remain distinct outcomes.
- Resource roots concatenate snapshots and their contents in supplied order. `Detached` and `ResourceWithoutSet` contexts yield known absence; a `Member` whose resource/object is missing is an unresolved adapter input. `find_in_roots` preserves the explicit collection order.

Inputs are selected JSON class facts, not a complete EMF object/resource implementation. Admission rejects unknown/foreign classes, enums as objects, duplicate/empty resource keys and invalid containment roles, including an empty `ownedRelatedElement` list on a non-Relationship. Nonempty external EAnnotation containment produces a traversal gap when reached. Abstract class facts are not advertised as constructible native objects. Other native/external containment, proxy behavior, notifications and caches remain unqualified.

Native `Namespace.importedMemberships` computes imports using namespace exclusions and name-conflict filtering. The adapter does not invent those results from raw JSON. No owned Imports means an empty imported set for this selected graph; an owned Import without a supplied observation remains unresolved when fallback is needed. `with_imported_memberships` accepts explicit ordered derived Membership visibility/related-element observations, including resource-scoped targets. These are caller-supplied facts, not verified native observations. Related targets are resolved lazily: a successful early public match need not resolve later targets. Reference encoding remains local to each resource; a same-ID object in another resource cannot satisfy an unresolved local redefinition.

## Validation and limits

[QUALIFICATION.json](QUALIFICATION.json) links captured source byte hashes and copied execution receipts. There are **26 new source-derived query cases**: eight engine cases and eighteen native adapter cases. Pinned `ElementUtilTest.java` contains UUID tests, not query tests; the new cases are not described as executed upstream tests. Original translated-source notices are retained in [NOTICE.md](NOTICE.md) and in the upstream source files.

Fresh runs passed 56 focused, 245 regression and 31 downstream test executions, with repeated cases across runs. Both default and candidate WASM builds passed. Every command used offline/locked Cargo, one build job and one test thread. No final capture reports source drift or timeout. Existing initialized workflows, snapshot reload, source naming, model/scalar/containment, metadata, editor and persistence regressions passed. Registry dependency versions/checksums did not change; the graph crate gains a direct dependency on the existing local domain crate. The existing initializer atomic API deprecation warning remains.

The initial focused capture retains a real failure: 28 domain tests passed, then twelve adapter cases passed and the generated-Port test failed on unsupported legacy `rename(PortDefinition)`. The test was corrected to use the existing generic attribute path; legacy rename restrictions remain intact. Independent review also found an empty invalid-containment admission bug; the owner guard was repaired before the successful second capture, with regression coverage. Original attempts and logs remain retained; no historical frozen gate was replayed.

One integration test constructs an actual PortDefinition helper graph, sets its name through the generic attribute API, commits, serializes/reloads bytes, and resolves `Pkg::Power::~Power` to the generated conjugated object. Other tests cover raw/semantic traversal, resource ordering/identity, immutable snapshots, short names, type precedence, lazy import/getter failures, invalid/proxy/cyclic references, GeneralView and malformed-input refusal. Independent static review found no remaining blocking defect; its source/receipt review is separate from the integrator's test execution.

Implemented and locally tested describe this bounded adapter. **Upstream runtime comparison is NOT_RUN; upstream parity verified is false.** Full area inventories remain incomplete and zero of the unchanged 18 areas are fully verified. Dynamic native import evaluation, full EMF ResourceSet/URI/proxy behavior, native qualified-name exposure, NativeContext/ViewUsage application and full application acceptance remain open. The Java/Maven/artifact-entitlement/isolation gates in [ORACLE-DECISION.md](../../ORACLE-DECISION.md) remain unresolved; no upstream build/runtime was launched. Browser UI code was not changed or re-exercised here; prior browser receipts retain their original scope.

Both dirty working backlogs, the held query-inventory draft and the historical validator lease remain unchanged. Their hashes and worker reconciliation are recorded in the qualification. New implementation, tests and evidence use separate paths rather than rewriting those drafts or inferring a lease release.

Reproduce after verifying the workspace volume, using new output directories and serial invocations:

```sh
python3 -B porting/run_initialized_workflow_checks.py "$SYSON_CHECK_ROOT/query" --phase native-query
python3 -B porting/run_initialized_workflow_checks.py "$SYSON_CHECK_ROOT/regression" --phase regression
python3 -B porting/run_initialized_workflow_checks.py "$SYSON_CHECK_ROOT/wasm" --phase focused
```

Next integration work is an explicitly resource-scoped planner binding for GeneralView and the remaining ViewUsage reference/application contract. It must preserve prefix visibility, snapshot identity and unresolved-versus-absent results; do not convert scoped external objects into unqualified semantic-ID tokens.
