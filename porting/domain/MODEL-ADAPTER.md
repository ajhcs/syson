# Stable model and edit adapter checkpoint

Baseline upstream `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`.
Crate: `rust/crates/syson-model`, path-depending on `syson-domain`.
This is a Rust adapter transaction proposal with source-grounded model writes.
It is not an implementation of upstream API, database, or server transactions.

## Exact selected schema and fixture

Persistence-format evidence is the document `content` field of the SQL INSERT
in `backend/application/syson-application/src/test/resources/scripts/database-content/SysMLv2-SimpleProject.sql`.
The extracted JSON is checked in as `rust/crates/syson-model/tests/fixtures/simple-project.emf.json`.
Its SHA256 including one trailing newline is `42cb4e3b3c3c7fe65c12abbcec28243ef91d7a1be7330553d88c5dab63ad773d`.
It contains 11 objects including the root Namespace and Relationship objects.

The schema is `{"json":{"version":"1.0","encoding":"utf-8"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[...]}`.
Each contained object has `id`, `eClass: "sysml:ClassName"`, and `data`.
SysML identity is `data.elementId`, separate from resource `id`.
`data.ownedRelationship` contains Relationships, and each Relationship's
`data.ownedRelatedElement` contains the owned elements. FeatureTyping links
`data.type` and `data.typedFeature` by stable elementId string.

`simple_project.json` and `package1.json` were inspected for the matching semantic
links. Those use flat REST `@id`/`@type` and derived `owner`/`qualifiedName` fields;
they are NOT EMF persistence documents. This adapter rejects them as the wrong
envelope. No REST roundtrip or SQL/database restore acceptance is claimed.

## Source-traced writes

Under `backend/services/syson-sysml-metamodel-services/src/main/java/org/eclipse/syson/sysml/metamodel/services/`:

| Source | Adapter behavior | Limitation |
| --- | --- | --- |
| MetamodelMutationElementService.addChildInParent | PartUsage (Feature) inside PartDefinition/PartUsage (Type) uses FeatureMembership; Package and non-Feature children use OwningMembership | Only selected Package/PartDefinition/PartUsage classes exposed; no complete type lattice |
| MetamodelMutationElementService.createMembership | Package uses OwningMembership; other contexts use FeatureMembership | Adapter follows addChildInParent when child class known; these two upstream APIs have different input contracts |
| ElementInitializerSwitch.caseUsage/casePartUsage | New PartUsage sets isComposite=true | Names explicitly supplied by caller; automatic count-based names, actors/stakeholders and all other initializer defaults unported |
| MetamodelMutationElementService.applyPrefixMetadata and SQL FeatureTyping fixture | Typing Relationship contains type and typedFeature stable IDs and is owned by usage | Only PartUsage -> PartDefinition endpoints accepted; not a complete typing service |
| ElementImpl.getOwnedElement and getOwner | Query projection flattens every contained ownedRelationship; semantic child owner is its relationship's owning element | Relationship objects excluded from query projection; full EMF containment search, imports and subclass operations not provided |

Unknown envelope, object and data fields survive reads and edits as JSON Values.
`arbitrary_precision` retains number values beyond machine integer widths.
This is semantic JSON equality, not byte preservation: whitespace, key order,
duplicate-key spelling and numeric formatting are not guaranteed preserved.
Input is deliberately restricted to the evidenced sysml namespace and JSON1.0
envelope; arbitrary EMF resources and absent elementId are unsupported.
Imported node IDs are retained verbatim; new `NodeIds` require canonical-shaped
36-character UUID strings. Resource IDs and elementIds are checked for duplicates
separately and case-insensitively. Cross-resource references are preserved, not
resolved or globally validated. No random-ID source is selected by this crate.

## Public integration API

* `NodeIds::new(resource, element)` validates distinct identity fields; using the
  same UUID value is allowed, but their roles remain separate.
* `Document::empty`, `from_json`, `to_json`, `value`, `element(stable_id)`.
* `Document::elements` gives `ElementSummary { resource_id, element_id, class,
  name, owner }`. Listing includes Relationships. Filter classes explicitly for
  editor presentation. `owner` is semantic owner, not raw EMF container.
* `Document::projection` returns domain `Model` and explicit `stable_id(handle)` /
  `handle(stable_id)` mapping. Process-local indices never become serialized IDs.
* `Editor::new`, `document`, `begin`, `commit`, `undo`, `redo`.
* `Transaction::create_package(parent?, ids, membership?, name)`,
  `create_part_definition(parent, ids, membership, name)`,
  `create_part_usage(parent, ids, membership, name)`,
  `type_part_usage(usage, definition, typing_ids)`, `rename`, `document`, `cancel`.
  Parent/endpoints use stable elementIds. Root creation needs no membership;
  child creation requires explicit new membership IDs.

## Proposed local transaction rules

`begin` snapshots the document. Each operation builds a fresh staged candidate,
validates it, and replaces staged state only after success. An error poisons the
whole transaction; commit then rejects it. Cancellation consumes staged state.
Successful commit changes the document once and records one undo snapshot.
No-op commit returns false and does not disturb history. Commit rejects transactions from another editor using a private instance token,
and rejects stale revision/base documents. Undo/redo advance revision; a new successful edit clears
redo. These are local in-memory snapshot rules, NOT proven Sirius transaction,
remote concurrency, cancellation or persistence semantics. Memory use grows with
full snapshots/history; no bound or production performance guarantee is claimed.

## Validation and remaining scope

Use cached Cargo registry and Samsung output paths configured by the environment:

```
CARGO_TARGET_DIR="${SYSON_BUILD_DIR}" CARGO_BUILD_JOBS=1 cargo test --offline --manifest-path rust/crates/syson-model/Cargo.toml
```

Tests cover exact source fixture Value roundtrip; stable separate identities;
projection lookup; unknown fields and arbitrary precision numbers; actual create
and typing structures; correct membership classes; staging/cancellation/error
atomicity; duplicate/malformed IDs; wrong endpoints; stale transactions;
undo/redo and redo invalidation. Dependency closure inspection is recorded in
MODEL-DEPENDENCIES.md. The locked direct serde_json version is 1.0.151; active
transitives are serde_core1.0.229, memchr2.8.3, itoa1.0.18 and zmij1.0.23.
Cargo.lock also includes inactive target-cfg/derive metadata dependencies;
no proc macros are compiled by this crate.

Upstream Java oracle, full SysML metamodel/defaults/constraints, imported scopes,
resource proxies, text parser/serializer, migrations, delete/move semantics,
secure random IDs, server API, database durability/recovery and collaborative
transactions remain unported or unrun. A successful adapter JSON roundtrip is
not evidence of successful upstream EMF import. Independent review and real
cross-crate workflow/UI tests are separate required milestones.
