# Complete Ecore metadata registry

Crate `rust/crates/syson-metamodel` is a pure Rust metadata registry generated from
exact upstream Git revision `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`. It has no external dependencies,
no build.rs, no project-runtime execution and no source modification outside its
assigned crate/docs. This is complete schema metadata, not a complete metamodel
runtime, semantic engine, editor, API or Java/EMF substitute.

## Source scope and provenance

| Source | SHA256 of pinned Git blob | Preserved scope |
| --- | --- | --- |
| `backend/metamodel/syson-sysml-metamodel/src/main/resources/model/sysml.ecore` | `82463c8f21d1a688603866ec02c6d4880b4e9224e34e9833a464aed0bd8f5440` | 182 domain classifiers: 175 EClasses + 7 EEnums |
| `backend/metamodel/syson-siriusweb-customnodes-metamodel/src/main/resources/model/sysml-customnodes.ecore` | `80006ba6cf30c146c1a8050ab7218adf52bdca92e04cbb45dd839b727790b266` | 4 separate Sirius custom-node EClasses |

Domain metadata includes 415 declared structural features, 70 operations,
57 operation parameters, 309 subset/redefine annotations and 19 enum literals.
Eight domain EClasses are abstract. Custom-node metadata is kept in a separate
package and includes four background references. Root LICENSE, LICENSE-LGPL and
NOTICE remain unchanged; generated and handwritten derivative files retain EPL-2.0
and Obeo attribution. Pinned original Ecore files remain editable upstream source.

## Preservation and public interfaces

`DOMAIN_CLASSIFIERS`, `CUSTOM_NODE_CLASSIFIERS`, and `SOURCES` are separate static
registries. `ClassifierId` provides typed IDs for every classifier; EEnums remain
explicitly distinguished from EClasses. Package-qualified `lookup` avoids mixing
domain names with custom-node names. Callers using glob imports should disambiguate
schema `Package` versus `ClassifierId::Package` (for example alias `SchemaPackage`).

Every original XML attribute, including namespaced attributes, and every metadata
child is retained in `MetadataNode`. Package attributes are retained by `SOURCES`.
No feature, operation, parameter, annotation, reference string or enum literal is
filtered out. The generator rejects unexpected text and unsupported classifier
kinds rather than silently omitting them. XML whitespace/attribute order and lexical
formatting are not modeled; raw pinned source hashes retain byte provenance.

Typed views expose:

* Class abstract/interface flags and full direct supertype identities.
* Feature kind, type reference, lower/upper multiplicity (-1 remains unbounded),
  ordering/uniqueness, containment, proxies, derived/volatile/transient/changeable,
  unsettable/ID flags, opposite references and explicit default literals.
* Operation return metadata and full ordered parameter metadata.
* All annotation source/reference strings and children (including multi-reference
  subsets/redefines) through the lossless node tree.
* Enum declared order, integer values, literal spellings and Ecore implicit zero.

Omitted XML flags expose Ecore schema defaults. `explicit_default_literal` returns
only the literal present in Ecore; it does not run an initializer, derive a value,
choose an enum default, or synthesize a Java/external datatype value. Example:
Feature.isComposite has Ecore false, while ElementInitializerSwitch.caseUsage sets
true during creation. These are separate facts.

## Inheritance and lookup contract

`is_subtype(child, parent)` proves reflexive or transitive LOCAL EClass inheritance
using complete direct supertype edges and a visited set for multiple inheritance
and diamonds. Enums are not classes. False means no known local proof; callers
must inspect `unresolved_supertypes` when they need a conclusion involving unknown
external ancestry. The registry does not invent inheritance for external types.

`inherited_feature` performs structural name lookup across local supertype branches.
A directly declared same-name feature stops that branch; shared diamond ancestors
are deduplicated. Independent candidates are retained rather than silently choosing
one. `FeatureLookup::unique` returns a feature only for one candidate; unresolved
external ancestors remain visible. This lookup does not execute annotations or
model redefinition/subsetting semantics and is not proof of EMF feature-ID ordering.

Full external identities are retained. In particular Element extends
`platform:/plugin/org.eclipse.emf.ecore/model/Ecore.ecore#//EModelElement`.
Custom node classes extend the external relative Sirius
`diagram.ecore#//NodeStyleDescription`; background references target external
`view.ecore#//UserColor`. Standard Ecore datatypes such as EBoolean/EString/EInt
also remain external type references, including the original `ecore:EDataType`
qualifier. They are never conflated with domain classes of similar names.

## Generation and verification

`tools/generate.py` was inspected before execution. It uses only Python stdlib;
its only subprocess is read-only `git show <hard-pinned-SHA>:<path>`. It reads
neither mutable worktree Ecore bytes nor project imports, invokes no Java or
upstream scripts, and writes only `src/generated.rs`. `--check` compares expected
bytes without writing. It rejects non-ASCII/control metadata requiring new Rust
escaping. Its current pinned inputs are ASCII.

```
python3 rust/crates/syson-metamodel/tools/generate.py --check
CARGO_TARGET_DIR="${SYSON_BUILD_DIR}" CARGO_BUILD_JOBS=1 cargo test --offline --manifest-path rust/crates/syson-metamodel/Cargo.toml
```

Verification: generator reproducibility check passes; 11 Rust tests pass; cargo fmt
and strict offline Clippy (`--all-targets -- -D warnings`) pass. Tests cover exact
class/enum/feature/operation/parameter/annotation totals, PartUsage -> ItemUsage ->
OccurrenceUsage -> Usage -> Feature -> Type -> Namespace -> Element, multiple
inheritance, diamond de-duplication, inherited defaults, containment/opposites,
unbounded multiplicity, derived flags, enum zero/order, operation parameters,
subset/redefine reference strings and unresolved external identities. Initial test
compile found a test-only glob import name ambiguity; fixed by package alias.

## Explicitly unimplemented behavior

All 70 declared operations are metadata only; none is executable through this
registry. Handwritten Java operation bodies, derived features, constraints,
subsetting/redefinition validation, semantic types, initializer side effects,
EMF inverse links/proxies/notifications/resources, external superclass features,
full object construction and serialization defaults remain unported. The registry
provides no invocation API and no successful placeholder operation result.

This milestone is not yet connected to syson-model class validation or domain
query predicates. That integration needs review of external classes, inherited
feature resolution and the bounded model adapter's schema. Upstream runtime
oracle and independent metadata review remain separate evidence gates.
