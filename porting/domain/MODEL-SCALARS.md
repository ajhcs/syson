# Registry-backed scalar introspection and local editing

Baseline `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`.
`syson-model` now path-depends on `syson-metamodel`. Its manifest was inspected:
no dependencies or build.rs. No new external package was added.
Existing create/rename/typing/import/export/history API signatures stay compatible.
This is a typed EMF-JSON adapter, not complete Java eSet, Jackson input coercion,
semantic validation, API or persistence parity.

## Source evidence and supported value representations

All EAttribute contracts come from the complete pinned domain Ecore. Metadata
retains actual declared owner, multiplicity, flags, type identity and explicit
default literal. The domain has 32 EBoolean attributes, 23 EString attributes,
one EInt, one EDouble and attributes using seven EEnums.

Generated/custom source was inspected under
`backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/`:

* `impl/FeatureImpl.java`: eSet casts Boolean and FeatureDirectionKind; generated
  setIsComposite updates a primitive boolean. direction default is null, while
  setDirection(null) sets its ESet flag and eUnset clears the flag.
* `impl/LiteralIntegerImpl.java`: eSet casts Integer and setValue takes Java int.
* `impl/SysmlFactoryImpl.java`: enum convertToString calls instance.toString;
  FeatureDirectionKind/PortionKind createFromString use literal lookup.
* `FeatureDirectionKind.java`: getLiteral/toString return the literal string.
* `impl/OccurrenceUsageImpl.java`: setPortionKind(null) maps to null default but
  sets ESet. `MembershipImpl`/`ImportImpl` setVisibility(null) select different
  defaults (public/private). RequirementConstraintMembership, StateSubactionMembership,
  TransitionFeatureMembership and TriggerInvocationExpression setKind(null) select
  their generated kind defaults. These reset semantics are NOT inferred from
  Ecore lowerBound or uniformly replaced by raw JSON null.
* `impl/ElementImpl.java`: declaredName/declaredShortName scalar string setters;
  elementId is a special identity field requiring separate link-aware editing.

Actual nested EMF JSON SQL fixtures corroborate string representation:
`ActionTransitionUsages.sql` contains direction `in`,
`ConnectionUsageTests.sql` direction `out`, `GeneralView-WithTopNodes.sql` direction
`inout`, and `GeneralView-ActionUsage.sql` portionKind `snapshot`/`timeslice`, under
`backend/application/syson-application/src/test/resources/scripts/database-content/`.
Only semantic document field occurrences are relevant; diagram style strings
with similar names do not establish model schema.

## Public integration contract

`Document::inspect_node(stable_id)` returns `NodeInspection`:

* `classifier`: the actual known domain EClass ID; foreign prefixes, unknown
  classes, and enum-as-object cases return explicit UnsupportedClass.
* `attributes`: inherited, uniquely locally resolved EAttributes with their
  names, declared_by classifier, bounds, unsettable, explicit default literal,
  stored optional JSON value, ScalarType and an optional blocking reason.
  `writable_attributes()` selects descriptors supported for scalar editing.
* `diagnostics`: reference/containment edits, ambiguous lookup or unresolved
  external-ancestor semantics, preserving exact missing type identities.
* `unresolved_supertypes`: full external ancestry identities, not guessed classes.

Absent stored attribute is `value=None`; explicit JSON null is `Some(Null)`.
Introspection does not write/materialize any default, derived value or field.
Readonly/derived, many-valued and identity attributes remain visible with
explicit blocks. A schema-permitted scalar descriptor is not proof of all
cross-feature constraints, inverse updates, initialization or derived behavior.

`Transaction::set_attribute(stable_id, name, serde_json::Value)` shares existing
staged atomic commit/cancel, failed-transaction poisoning and undo/redo semantics.
It supports:

| Type | Accepted input | Explicit boundary |
| --- | --- | --- |
| EString | JSON string; null only when Ecore lowerBound=0 | No automatic parsing, naming policy or text syntax validation |
| EBoolean | JSON bool | No numeric/string coercion; null rejected |
| EInt | JSON integer in signed 32-bit range | Float spelling, strings, out-of-range and null rejected |
| EDouble | JSON number convertible to finite IEEE754 f64 | Stored value normalized to actual f64; overflow/nonfinite/null rejected |
| EEnum | Exact declared literal string | Integers/case aliases/unknown literals rejected; ALL null resets explicitly unsupported |

`elementId` edits are IdentityManaged and rejected to preserve cross-links.
Reference and containment edits are explicitly unsupported. Many-valued attrs
are unsupported. Derived or nonchangeable fields are rejected. Unknown known-schema
attributes, ambiguous candidates, unresolved external semantics and unsupported
datatypes have distinct errors; unknown imported fields still roundtrip unchanged.
An unknown-attribute error describes this known local schema, not proof that an
unresolved external superclass cannot declare such an attribute.

`Projection::find_by_type(name, ClassifierId)` uses the complete registry's known
local transitive subtype predicate. Projection namespace classification now uses
real Namespace ancestry instead of four names. Unknown or wrong-prefix imported
class identities carry an unresolved marker in the bounded projection and never
become positive typed matches. Serialized source eClass strings remain untouched.
Existing closure-based domain queries remain available.

## Conservative external-branch handling

Independent review established that registry `FeatureLookup::unique` means one
LOCAL candidate; other inheritance branches can still have unresolved external
ancestry. The method documentation now says so explicitly. Scalar edits reject
such cases as UnresolvedAttributeSemantics instead of guessing global uniqueness.
Example: AcceptActionUsage.isIndividual has one local OccurrenceUsage candidate
and a separate unresolved external EModelElement branch. Its descriptor remains
a diagnostic until the external schema engine is present. This is conservative
unsupported behavior, not evidence of an actual conflicting external declaration.

## Verification and remaining engine work

The model suite now has 27 passing tests: 15 existing transaction/model tests,
11 scalar integration tests, and one defensive unsupported-external-datatype unit
case using an explicitly synthetic identity. Tests inspect all 175 domain EClasses,
check inherited descriptors/default absence, optional strings, booleans, enum
literals/null rejection, Java int endpoints, double normalization/overflow,
reference/containment/identity/many-valued/derived rejection, unknown classes,
external-branch diagnostics, subtype queries and atomic error/history behavior.
Fmt and strict offline Clippy pass. No new upstream runtime oracle was run.

Unported: notifications, unset-vs-set flags, enum reset/default materialization,
complete class instantiation, Java semantic operations, derived field execution,
subsetting/redefinition constraints, opposite/reference changes, external schema
resolution, all cross-feature invariants and API/database semantics. Generic
scalar writes do not turn the adapter into a full 175-class modeling runtime.
The three bounded construction APIs are unchanged.
