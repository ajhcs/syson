# Full domain factory and initialization preparation

Baseline: Eclipse SysON `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`. This directory contains static source evidence and a proposed implementation contract. It does not add a factory, initializer, model feature, menu, serializer or runtime oracle. No upstream code, compiler, package manager, server or browser was executed for this preparation. The self-authored Python extractor reads Java/Ecore as text and writes only these evidence files.

## Complete indexed source surface

| Evidence artifact | Scope and interpretation |
| --- | --- |
| `class-factory-and-dispatch.json` | All 175 domain EClasses, abstraction/interface flags, local/external ancestry, factory case/method evidence, complete generated switch order and projected effective initializer overrides |
| `initializer-overrides.json`, `initializer-helper-methods.json` | Every one of the 31 initializer case overrides; null/non-null return behavior, explicit case calls, setter/raw-create/helper references, method lines/hashes; helper method index |
| `java-fields-and-defaults.json` | All 192 protected declarations across the 175 domain implementation classes: 77 `_EDEFAULT` constants and 115 cached/list/reference/ESet fields, including declarations without an initializer |
| `declared-ecore-attributes.json` | All 65 directly declared EAttributes, including metadata type identities/default literals/flags; inherited copies are not counted again |
| `implementation-state-methods.json` | All 175 implementation constructor/superclass records plus eGet/eSet/eUnset/eIsSet method locations/hashes |
| `explorer-menu-candidates.json` | All 175 container-class switch routes and statically derived candidate membership sets, with abstract dummy-instantiation caveat and no claim about display ordering |
| `intermediate-container-switch.json` | Exact explorer intermediate-container switch method index |
| `diagram-tool-source-index.json` | All 56 main-source view tool/provider/section/palette files selected by the documented extractor criteria, including source method signatures/hashes, lexical service/provider/class references and string tokens |
| `application-palette-source-index.json`, `frontend-command-source-index.json` | Application expression-palette contributor plus 24 frontend command/menu/tool/modal/registry adapters, including textual/expression creation routes |
| `backend-create-method-index.json` | 529 non-generated main-source backend methods whose names begin `create`, including builder/AQL adapters; an index, not 529 distinct user features or proof each method is exposed |
| `source-sha256.txt` | Fingerprints for every indexed/consulted upstream source file, including factory, generated implementations/switches, initializer, creation services, menu wiring, serializer boundary and existing test evidence |
| `summary.json`, `extract.py` | Machine-readable coverage totals and reproducible static extraction with explicit structural consistency checks |

The backend source tree was compared to the baseline with `git diff`; no backend source differences were present. Outputs are source interpretations, not captured Java results. Java token indexing includes strings in comments and does not establish an active menu/precondition or transitive call graph. Dynamic labels, selection dialogs, representation visibility and evaluation of AQL remain separate runtime work. These limits are recorded rather than treating lexical source presence as behavior acceptance.

## Raw factory constructibility

The pinned Ecore has **175 EClasses, 167 concrete, 8 abstract, zero interfaces**, plus seven EEnums. Every concrete EClass has one `SysmlFactoryImpl.create(EClass)` switch arm and a no-argument `createX` implementation. The complete concrete list is in the class inventory; no concrete class is omitted. The eight abstract classes, all absent from the factory switch, are:

- ConnectorAsUsage
- ControlNode
- Element
- Expose
- Import
- InstantiationExpression
- LoopActionUsage
- Relationship

Every one of the 167 no-argument raw creation methods has the same three-operation shape: instantiate its generated implementation, assign `elementId` using `ElementUtil.generateUUID`, return the object. Each of the 175 implementation constructors calls only `super()`. Raw factory creation does **not** call `ElementInitializerSwitch`, attach a membership, create a diagram, validate all model invariants, or instantiate external Ecore annotations.

`create(EClass)` dispatches by numeric classifier ID and throws for an unmatched ID. It trusts the caller's EClass; this is not a general foreign-package validator. A typed Rust domain-class API should reject wrong-package/enum/abstract requests explicitly and report that boundary, rather than reproducing accidental foreign-ID collisions. External `EcoreUtil.create` dispatch belongs to the foreign package's factory, which is outside this domain factory.

## Default values are not serialized attribute presence

Raw Java state has multiple layers:

1. Java fields/constants/zero initialization and lazy list/reference creation.
2. Generated or custom getters, enum-null/reset behavior, and explicit ESet flags.
3. Context initializer assignments and added objects/relationships.
4. EMF JSON serialization policy using object state, eIsSet and external Sirius/EMF machinery.

The Ecore default literal alone does not define all four layers. The field and eIsSet inventories keep the Java evidence separate from metadata. In particular:

| Source state | Verified source distinction |
| --- | --- |
| Feature.isUnique | Generated default is true, unlike the usual boolean false |
| Feature.isComposite | Raw default false; Usage initializer sets true; AttributeUsage/ReferenceUsage then set false |
| Feature.direction | Default null with a distinct directionESet flag, initially false; setDirection(null) and unsetDirection have different state |
| OccurrenceUsage/ConnectionUsage portionKind | Generated null defaults with separate unset flags |
| FlowUsage portionKind | Generated default `PortionKind.SNAPSHOT`, also with unset state; do not inherit a guessed universal null |
| Membership.visibility / Import.visibility | Generated defaults PUBLIC / PRIVATE respectively |
| RequirementConstraintMembership.kind | ASSUMPTION |
| StateSubactionMembership.kind | DO |
| TransitionFeatureMembership.kind | EFFECT |
| TriggerInvocationExpression.kind | AFTER |
| LiteralBoolean/Integer/Rational/String value | false / 0 / 0.0 / null in raw generated state |
| Element.elementId | Raw field default null, then the factory explicitly assigns the UUID; resource/representation identity is a different lifecycle |

These are semantic source defaults, not instructions to add all fields to JSON. A non-unsettable boolean assigned its default can still have eIsSet=false, whereas an unsettable enum may retain explicit-set state independently of its value. Derived getters and inverse/proxy behavior must not be replaced with cached literal guesses. `ElementImpl` extends external EMF EModelElement implementation; external defaults/annotations/notification behavior are not supplied by this repository.

The local `SysONJsonResourceProcessor` delegates serialization/value handling. `StandardLibrariesLoader` sets FORCE_DEFAULT_REFERENCE_SERIALIZATION for library references; this is not evidence that all scalar defaults are serialized. The actual Sirius JSONResourceFactory/EMF JSON implementation is a dependency boundary. No runtime default-omission oracle was run. A future factory must preserve stored-absent vs stored-null/value and semantic default/set-state separately until serialization behavior is established.

## Identity lifecycle

`ElementUtil.generateUUID` begins with random UUID v4. Standard LibraryPackage identity can become UUIDv5 of the library URI plus qualified name, and named standard-library elements can use their library namespace UUID plus qualified name. Imports, Memberships, Specializations and TypeFeaturing are excluded from that latter v5 route. Unattached freshly constructed objects normally have no qualifying library/name context, so a factory call must not invent a stable v5 identity from a future name. The later library identity lifecycle is a distinct operation. Resource JSON IDs and `elementId` must remain distinct; the raw factory source only assigns `elementId`.

## Context initialization contract

`MetamodelMutationElementService.initialize` explicitly expects the object to be attached before dispatch. `ElementInitializerSwitch` follows generated `SysmlSwitch` first-non-null dispatch, not an arbitrary walk over all ancestors. The complete 175-class order and 31-override projection are indexed. Overrides that return an object stop further ancestor initialization; explicit `this.caseX` calls are direct method calls, not another full dispatch.

| Override(s) | Ordered behavior to preserve |
| --- | --- |
| Usage | Lowercase first character of actual EClass name; remove a terminal `Usage`; append same-class owning-namespace count; set isComposite=true |
| Definition / Package | Actual EClass name plus same-class count |
| EnumerationDefinition | isVariation=true plus class/count name; stops ancestor dispatch |
| PartUsage | Run Usage; ActorMembership/StakeholderMembership may replace name with actorN/stakeholderN |
| ReferenceUsage | Run Usage; SubjectMembership gives subject; otherwise action owner gives parameterN; set isComposite=false |
| AttributeUsage | Run Usage then isComposite=false |
| RequirementUsage | Run Usage; objective membership under a use-case owner gives owner's name plus possessive objective suffix |
| Comment / Documentation / TextualRepresentation | Set the exact respective source body strings (`add comment here`, `add doc here`, `add textual representation here`) |
| Expose | visibility=PROTECTED and isImportAll=true |
| AcceptActionUsage | Run Usage; add payload INOUT and receiver IN ReferenceUsage objects under ParameterMemberships, using raw child factory creation and the helper's explicit assignments |
| PortDefinition | Class/count name; create OwningMembership -> raw ConjugatedPortDefinition -> PortConjugation; set conjugatedType/originalPortDefinition links |
| ConnectionDefinition | isSufficient=true; add source/target ReferenceUsage end members; return null, so ancestor dispatch continues to Definition naming |
| InterfaceDefinition | isSufficient=true; add source/target PortUsage end members; return generated super null, then ConnectionDefinition and Definition are also reached |
| ViewUsage | Run Usage; create FeatureTyping with typedFeature=self and type resolved by `StandardViewDefinitions::GeneralView` / ViewDefinition lookup |
| FlowEnd | Direct inherited caseFeature call (default null) then isEnd=true; stop |
| BindingConnectorAsUsage / Dependency / Element / FeatureTyping / FlowUsage / PerformActionUsage / Redefinition / ReferenceSubsetting / Specialization / Subclassification / Subsetting / SuccessionAsUsage / TransitionUsage | Return object with no assignments in that override; this can intentionally prevent generic Usage initialization |

Important source details requiring explicit tests rather than “cleanup”:

- The count is exact runtime EClass equality, not subtype count. Attached first siblings usually include themselves and yield suffix 1; unattached objects have no owning namespace and yield 0.
- Actor/stakeholder filters test the current object's membership in the second predicate, not each sibling's membership. Consequently the source counts same-class owned members, not a repaired actor-only/stakeholder-only subset.
- Newly created initializer children are **not** recursively initialized indiscriminately. PortDefinition's raw conjugated child is intentionally not reinitialized; recursion would create more conjugated ports.
- Accept-action raw helper references do not call Usage initialization. Connection-definition reference ends call the ReferenceUsage override directly. Interface-definition port ends call inherited `casePortUsage` directly, which returns null and does not automatically run Usage.
- The InterfaceDefinition dispatch projection implies two port ends **and** two reference ends, then Definition naming. This is direct source-order evidence, not a Java-run assertion; capture an upstream oracle before accepting or changing that behavior.
- ViewUsage library lookup is contextual. A missing library/type must remain unresolved/null according to the source setter path, with an explicit dependency diagnostic; it must not be assigned an invented standard-library ID.

`NamespaceImpl.getOwnedMember`, Element owner/owning-membership/owning-namespace getters, relationship subtype handling, proxy resolution and actual library lookup are required dependencies for faithful initialization. An ownedMember counter over guessed JSON nesting is insufficient until those rules are reconciled.

## Menu/service route boundaries

`SysMLv2EditService` root suggestions offer Package only. Its full root menu uses Namespace's package-like candidates (84 class candidates in the static projection), rather than all 167 raw factory classes. Existing root Namespace selection/creation and resource-domain fallback are separate service behavior. Child menu candidates come from `GetChildCreationSwitch`, then labels are sorted case-insensitively. Projected examples: Package/Namespace 84, OwningMembership 147, FeatureMembership 79, PartUsage 16, ViewUsage 6, EnumerationDefinition 2, RequirementDefinition 57. The JSON ledger provides every container class's candidate set and source route; it does not substitute source candidate ordering for resolved label ordering.

The menu switch's EClass overload creates a raw dummy object before dispatch; abstract EClass requests therefore fail the dummy factory route even if a projected ancestor rule exists. Menu restriction is also not a complete service authorization check: `createChild` accepts a prefixed class command and a domain Element, creates the raw object, chooses an intermediate container, attaches it, initializes it and optionally creates a ViewUsage diagram. It does not first re-check the displayed candidate list.

Explorer intermediate membership rules differ from `addChildInParent`:

- Definition/Package/Namespace/Comment/Documentation/TextualRepresentation -> OwningMembership.
- EnumerationUsage -> VariantMembership.
- ConstraintUsage under requirement -> RequirementConstraintMembership; ConcernUsage under requirement -> FramedConcernMembership.
- Feature under Definition or Usage -> FeatureMembership; Feature under Membership -> no new intermediate; other Feature contexts -> OwningMembership.
- LiteralExpression -> FeatureValue.
- When no intermediate is selected, a Relationship is attached to ownedRelationship; an Element under Membership is attached to ownedRelatedElement.

The broad metamodel service separately uses FeatureMembership iff child is Feature and parent is Type. Diagram compartment tools use structural-reference-specific membership selection, including requirement kind assignment, VariantMembership, documentation owning membership and framed concerns. Directed compartment creation initializes then sets direction and appends the capitalized direction to the resolved name. Connection/interface end tools, accept parameters, state/control/action tools, connectors, actor/subject/objective/stakeholder tools and model-level composite commands have their own guarded services. Their source indexes are supplied; they are not reduced to one generic constructor. Application expression-palette availability also checks read-only status, inherited content and expression support; its frontend hook invokes createExpression with textual expression/properties. Textual SysML insertion and inherited Sirius menus are separate routes, not raw EClass factory aliases. The supplemental application/frontend indexes preserve those dependencies. `AbstractCompartmentNodeToolProvider` also performs exposure/reveal/selection work, which belongs to representation behavior, not a raw object factory.

ViewUsage creation's General View diagram side effect belongs to the edit/diagram service, beyond `ElementInitializerSwitch`. Model-only factory success cannot claim that diagram was created or persisted.

## Existing evidence and next decision

Inspected existing tests cover actor/stakeholder and package/definition/usage naming, enumeration variation/name, expose flags, contextual GeneralView typing and child-menu membership sets. `ObjectCreationTests` additionally asserts a General View representation is created alongside ViewUsage. None was executed here. The next implementation should use those source tests and independently captured graph/default/serialization goldens across every concrete class and initializer branch, not claim completion from a 167-name registry.

The proposed exact ownership/API boundary and required qualification are in `IMPLEMENTATION-CONTRACT.md`. This checkpoint requests review of that boundary; it grants no new model/web ownership and does not mark factory, initializer, menu or serializer parity complete.

Upstream source/fixture notices, EPL-2.0 and root LICENSE/NOTICE obligations remain unchanged. The extraction indexes retain source paths and hashes; they do not relicense copied source or standard libraries.
