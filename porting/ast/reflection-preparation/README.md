# Reflection and type-index preparation

Lease: `PARSER-REFLECTION-PREP-01`. This directory is source preparation only. It contains no Rust implementation and does not change the construction kernel, syntax recognizer, semantic model, per-rule dispositions or parity statuses.

The reproducible static preparation retains nine candidate source documents and six exact modules from the pinned SysON CLI. It indexes all 189 names returned by generated SysML reflection, all 184 subtype switch cases, 20 local metadata cases containing 31 mandatory-property declarations, and eight generated reference-property IDs. Five NON_TYPES markers are excluded by TypesIndex.getAllTypes, leaving 184 names in that particular filtered list. These are AST reflection names, not Ecore classifier counts, grammar rule counts, runtime dependencies, tests or coverage.

Candidate: Syside source `1326f234cae23b0e4777d5a48a069b17e9e332f1`. Baseline: SysON `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`, embedded CLI SHA256 `47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd`. Exact documents and hashes are preserved in `candidate-source-documents.jsonl` and `embedded-module-documents.jsonl`. The candidate grammar objects match the baseline grammar objects, as separately checked; corresponding reflection/service implementation behavior and complete artifact/notices provenance remain unproven. This preparation retains the two source layers separately.

Run the static comparison from an existing checkout containing the pinned baseline Git object:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 porting/ast/reflection-preparation/prepare_reflection.py
```

`--write` regenerates the six machine-readable source inventories. The tool reads retained source text and Git blobs, verifies hashes, and extracts literal declaration shapes; it never loads JavaScript, resolves modules, runs a parser, compiles, downloads or invokes upstream scripts. It asserts complete case-name coverage inside the selected generated switches and rejects unknown property/expression shapes. It deliberately does not compute effective inheritance ordering using a different language's sort implementation.

## The distinct reflection layers

| Layer | Source | Contract |
| --- | --- | --- |
| Langium base | Embedded syntax-tree.js at line 12011 | isInstance tests structural AstNode shape then subtype; isSubtype returns true for equal strings before consulting cached computeIsSubtype; getAllSubTypes scans all generated names and includes self |
| Generated SysML | Candidate generated/ast.ts:2395; embedded generated/ast.js at line 43992 | Ordered list of 189 names, generated subtype switch, eight valid reference IDs, local mandatory arrays/booleans; unknown metadata returns an empty mandatory list |
| Syside type index | Candidate model/types.ts; embedded model/types.js at line 86709 | Precomputed ordered proper-supertypes and reverse subtype sets, registry expansion/merge/chaining and NON_TYPES filtering |
| Syside override | Candidate services/sysml-ast-reflection.ts; embedded same module at line 107179 | Chain-position reference typing, inherited defaults, synthetic nodes/parenting and `$children`/`$childIndex` caches |
| Parser use | Candidate services/parser/parser.ts; embedded parser.js at line 108805 | Derived-type postprocessing dispatch, child recollection and nonfragment CST retarget patch |

Do not conflate Langium getAllSubTypes (self-inclusive) with Syside getSubtypes (proper descendants built through ancestor chains). Generated getAllTypes returns all names, while TypesIndex.getAllTypes filters only the five explicit NON_TYPES constants. expandToDerivedTypes/expandAndMerge/chain still iterate the complete generated list, including those five. The isUnion method uses a separate string heuristic (`startsWith("Named")` or contains `Or`), not membership of NON_TYPES; it must not replace that filter.

The five markers are TransparentElement, FeatureRelationship, InlineExpression, NonOwnerType and TypeRelationship. The generated declarations include their subtype relationships; excluding a marker from one public type list does not erase it from inheritance/default/dispatch computation.

## Exact inheritance and registry ordering

TypesIndex constructor loops the generated type list in its original order. For each type it scans that complete list again, excludes self, and retains every generated supertype found by base reflection. It partitions these into NON_TYPES unions, namespace-subtype types and other relationships, in that priority order. It sorts each partition using subtype comparisons: a subtype sorts before its supertype; unrelated pairs compare equal. It then merges sorted relationships into the namespace-like list, followed by sorted unions. For each item being merged, its insertion position is the first existing item that is not its subtype, or the list end. Those positions are sorted numerically before splice operations with the insertion-count offset.

The result is stored as an insertion-ordered Set. The reverse subtype sets acquire names in the outer generated-type iteration order. There is no declared arbitrary alphabetical tie-break. A generic graph topological sort or Rust BTreeSet cannot substitute for the ordered algorithm. Exact behavior for comparator ties/mixed inheritance needs qualification against the pinned runtime, rather than assuming a Rust sorting implementation chooses the same order.

Candidate `model/__tests__/type-index.test.ts` retains four strict chain expectations (Element, Comment, Feature and OperatorExpression), three subtype-membership cases and two expansion/merge tests. These sources are retained but were not executed. Some use partial matchers/arrayContaining rather than asserting the entire registry or every array order. They therefore do not close the full 189-type ordering/dispatch surface.

Registry contracts differ:

- expandToDerivedTypes tries the direct entry, then the ordered ancestor chain, then default. Each selection uses JavaScript truthiness; a present false/zero/empty-string value does not win. Object/array/function entries are truthy. Missing and intentionally falsy entries must not collapse into a Rust Option-only contract without a declared supported-value domain.
- expandAndMerge initializes every generated type to an empty array, appends direct then each ancestor array, with no default registry entry. With reverse=true each array is prepended as a block, reversing block order but preserving within-block element order.
- chain collects direct, ancestor and default entries in that order; supertype-first reverses the complete collected array, including where default ends up. It is not simply reverse traversal of the inheritance graph.
- isSubtype returns true for equal names even when neither is registered. getInheritanceChain for an unknown name returns a fresh empty Set; getSubtypes creates/caches an empty set. Generated unknown metadata also returns an empty list. The strict proposed Rust type resolver below rejects unknown names separately and must not be described as literal unknown-name compatibility.

The embedded Langium base uses ordinary JavaScript object caches, while Syside uses Map/Set. Prototype-key and mutation-alias behavior for arbitrary malformed names/externally mutated returned objects is outside the proposed strict typed domain. Qualify it if that input domain is later claimed. The exact source texts remain available rather than silently replacing them with stronger assumptions.

## Defaults and reference typing

SysMLAstReflection.getTypeMetaData starts with generated *local* mandatory properties for the requested type, then visits the entire ordered ancestor chain and adds only names not already present. It appends `$children: array`, caches the resulting metadata object, and returns it. The local-property Map preserves insertion order; duplicate local names, if present, would take the last mapped type at the first insertion position. This differs from later ancestor first-name-wins logic. The current generated metadata has no duplicate local mandatory names; future schema drift must be rejected or dispositioned.

Array defaults replace any non-array value, including null or scalars. Boolean defaults apply only to undefined, preserving null. The programmatic creation helper additionally supplies `$childIndex = 0` only when undefined. The parser's base default-filling call and the reflection helper are separate call sites; do not apply every helper side effect indiscriminately. The existing Rust kernel accepts caller-declared flattened metadata and has no qualified inheritance table or child-cache implementation.

Generated getReferenceType accepts eight `<ReferenceType>:parts` IDs and returns Element; other property IDs throw. The override instead ignores the property name and inspects the reference's position in container.parts. Earlier parts target Element. The last part targets Type, Classifier, Metaclass, Membership, Namespace, Feature or ConjugatedPortDefinition for the seven corresponding reference types; ElementReference and other types fall back to Element. This determines target *type*, not target identity, scope, resolution or metamodel readiness. Invalid/negative/out-of-range positions are not guarded by the source helper; a strict Rust checked-position API must label its narrower input domain.

## Implicit construction and parser dispatch

Reflection createNode spreads supplied values after `$type`, clones a supplied CST by prototype plus property descriptors, points the clone at the new node and adds a present `$implicit` marker. It assigns defaults, then calls assignNode. TypeScript restrictions are not runtime validation of malicious spread properties. A safe Rust API should reject structural/control-field overrides and state that restriction explicitly.

assignNode returns without parenting when parent or property is absent. For array properties, an explicit index shifts later `$containerIndex` values then uses JavaScript splice; otherwise it appends. For scalar properties, an explicit index throws. It sets `$container` and `$containerProperty`, but does not explicitly clear a stale scalar `$containerIndex`. JavaScript negative/out-of-range/nonfinite numeric splice inputs require their own policy if supported; a Rust usize index plus bounds validation is intentionally narrower.

If parent.$children exists, assignNode looks for a source-later child using `node.$cstNode.offset > child.$cstNode.end`. When found it shifts cache indexes and inserts the child there, then unconditionally pushes the same child again and sets its `$childIndex` to the final index. This can duplicate a cached child. Do not silently fix that behavior while claiming literal parity. A child cache is separate from true containment properties; the kernel's repeated-containment rejection does not resolve this source cache behavior.

isProgrammaticNode returns true with no CST or when `$implicit` is *present* in the CST, including inherited/present-false markers; it does not test boolean truth. Parser collectChildren later clears `$children`, fills it from Langium streamContents, sorts by compareRanges and assigns each child index. compareRanges uses (0,0) for an absent range and compares starts only. Recollection may therefore replace earlier cache effects. Exact construction timing is part of the contract.

The parser postprocessing registry has five direct entries: OperatorExpression, WhileLoopActionUsage, Import, TransitionUsage and SuccessionAsUsage. It expands through TypesIndex.expandToDerivedTypes and runs a selected handler after base CST construct, for string $type only. Those handlers create/retype/reparent placeholders; reflection creation alone does not implement them. Parser fillNode/construct then recollect children. The prototype merge patch can also fill defaults/recollect when adopting a type and lazily retarget CST element associations for a nonfragment rule-call merge. An adapter must sequence these steps from a qualified parser construction trace, not choose a convenient CST derivation.

Implicit *semantic specializations* in model/implicits.ts are another mechanism: metamodel decorators populate IMPLICIT_MAP, expandAndMerge collects ancestor entries, and reversed iteration gives derived entries precedence in a type/kind map. It is distinct from `$implicit` CST marking and parser placeholder hooks. Its decorator/library/linking closure remains outside reflection preparation.

## Proposed Rust API and ownership

These signatures are a proposal, not implemented code or an assignment to modify other crates:

```rust
struct ReflectionDb { /* immutable pinned declarations, ordered tables, qualification */ }
struct TypeHandle { /* private nonrecycled table owner + local index */ }
struct QualifiedProperty { /* table-owned property name/type/source */ }
enum MandatoryKind { Array, Boolean }
enum ReflectionError { UnknownType, ForeignTable, InvalidReferencePosition,
    UnqualifiedInheritanceOrder, UnsupportedDispatchValue, SourceDrift }
enum SourceQualification { CandidateOnly, EmbeddedDeclarationsCompared,
    RuntimeDifferentialQualified }

impl ReflectionDb {
    fn resolve_type(&self, name: &str) -> Result<TypeHandle, ReflectionError>;
    fn all_generated_types(&self) -> &[TypeHandle];
    fn ast_types_filtered(&self) -> &[TypeHandle];
    fn proper_supertypes(&self, ty: TypeHandle)
        -> Result<&[TypeHandle], ReflectionError>;
    fn proper_subtypes(&self, ty: TypeHandle)
        -> Result<&[TypeHandle], ReflectionError>;
    fn is_subtype(&self, sub: TypeHandle, sup: TypeHandle)
        -> Result<bool, ReflectionError>;
    fn effective_mandatory(&self, ty: TypeHandle)
        -> Result<&[QualifiedProperty], ReflectionError>;
    fn reference_target_type(&self, position: CheckedReferencePosition)
        -> Result<TypeHandle, ReflectionError>;
}
```

The table owner must be nonrecycled and validated, as AST arena owners now are. TypeHandle and NodeId are different types; numeric indexes must never cross arenas/tables or become persisted IDs. A ReflectionDb can be shared immutably through Arc. A builder receives an Arc to exactly one qualified table and converts only known mandatory Array/Boolean properties into its existing metadata interface. Mutable construction state remains owned by one builder/transaction, and completion produces an immutable arena. Do not let a registry hold executable JavaScript or a mutable global singleton.

A separate `DispatchRegistry<T>` proposal accepts typed callback/enum identifiers and an explicit supported truthiness policy. It exposes most-derived selection, block expansion/merge and full chain order as distinct methods. It must reject unqualified inheritance order rather than return a convenient first handler. ImplicitNodeFactory should validate typed structural fields, emit a transactional construction-operation sequence, and maintain CST markers/true containment/derived child caches separately. Source-literal cache quirks and a proposed normalized child-cache policy need separate dispositions; no root product policy is selected here.

The existing kernel's owner-qualified handles, budgets, rollback and pre-mutation flat-array rejection remain authoritative. No new API may weaken them to mimic JavaScript object aliasing. Where a safety/input-domain restriction differs from source, report an explicit restriction or unsupported outcome. No $meta/resolved reference, parsing/linking/validation/CLI/import status is manufactured.

Implementation ownership must be leased separately by the integrator. Proposed table generation/reflection implementation belongs in a future dedicated crate or explicitly leased syson-ast module; kernel event application remains with syson-ast, syntax selection with syson-syntax, and semantic/linking/model services with their own owners. This preparation changes none of those paths. Dependency sequencing belongs to the root execution DAG; eventual semantic integration gates do not imply cyclic source-kernel build dependencies.

## Qualification and next evidence

`qualification-gaps.json` lists open source/behavior boundaries. Required future evidence includes full table/order comparison for every generated name, unknown/foreign type rejection, local-versus-inherited default precedence, all reference chain positions, array/scalar parenting and implicit-cache edges, all derived hook dispatch, empty/missing source ranges, merge CST retarget timing and separate semantic implicit registration.

Use source-derived native tests first, labeled as such. Pinned runtime differential execution needs a separately authorized oracle and captured input/bundle/source/tool/output hashes, UTC intervals, exit/diagnostics and explicit supported domains. Source fixtures are unrun here. No grammar rule or construction declaration status is promoted by this preparation.
