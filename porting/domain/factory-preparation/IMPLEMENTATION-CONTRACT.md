# Proposed native factory ownership and API contract

Status: **proposal for integration-owner review**. Baseline and evidence are in this directory's README and source inventories. No implementation or additional path ownership is established by this document.

## Exact proposed ownership

| Surface | Proposed writer boundary | Required integration agreement |
| --- | --- | --- |
| Raw domain allocation, constructor state, semantic default/set-state | New `rust/crates/syson-model/src/factory.rs` and focused factory tests | A single model/factory owner; existing model module owner reviews public exports and transaction coupling |
| Context initialization and ordered child graph effects | New `rust/crates/syson-model/src/initializer.rs` and initializer tests/fixtures | Same single writer or released sequential lane; uses shared owner/membership/library query contracts |
| Explorer/object-creation route policy and candidate lists | New `rust/crates/syson-model/src/creation_routes.rs` and route tests | Distinct raw/metamodel/explorer/diagram route IDs; no replacement of policies by one broad predicate |
| Pinned default metadata not already in Ecore registry | New generated evidence/data module under `syson-metamodel`, only if its owner releases it | Existing complete registry must remain authoritative; generator input is pinned Java plus Ecore, not guessed language defaults |
| Resource/element identity reservation | Existing model identity boundary with an injected provider | Preserve separate resource IDs and semantic elementIds; no process-index persistence and no factory-reserved standard-library IDs without source context |
| Context lookup | Existing model projection plus an explicit reference/library resolution interface | Define owner/owningMembership/owningNamespace/ownedMember/proxy outcome precisely before initializer calls depend on it |
| JSON serialization and unset state | Existing persistence/serializer owner | No blanket materialization of generated defaults into persisted JSON; qualify external serializer behavior before a parity claim |
| ViewUsage representation creation and diagram exposure/reveal | Existing representation/API integration owner | Native factory reports ordered post-create effects; it cannot mark these effects complete itself |
| Web palette/forms and public command exposure | Existing frontend owner after native contract acceptance | Do not broaden constructor menus merely because a raw class exists; no UI changes in this preparation lane |

The current lane writes only `porting/domain/factory-preparation`. The proposal does not authorize editing these modules or taking ownership from existing writers. Existing three construction APIs, scalar edits and shared capability checks stay in place until reviewed integration changes them.

## Proposed public concepts

Names below are reviewable API shapes, not existing Rust symbols or code ready to call.

`RawFactoryRequest { classifier, identity_context }`

- classifier is a known domain EClass identity, not a short string that might denote an enum/foreign package.
- raw capability covers every one of the 167 concrete domain classes; eight abstract classes, enums, wrong-package identities and unknown identities have distinct explicit errors.
- allocation creates constructor-default semantic state and elementId, but no naming initializer, containment membership or diagram.
- identity provider distinguishes a newly allocated elementId from the later resource JSON ID. Fresh unattached factory allocation follows random-v4 source behavior; library-v5 regeneration is a contextual operation with pinned namespace/name/URI inputs.

`FactoryObject { classifier, semantic_identity, stored_fields, slot_state, unresolved_external_state }`

- stored_fields retains actual absent/null/value, without filling absent fields with effective defaults.
- slot_state retains generated semantic defaults and relevant ESet flags separately. An effective value is not itself proof of storage presence.
- effective defaults derive from the pinned Java implementation class/ancestor behavior and Ecore metadata, including duplicate inherited storage/default implementations such as FlowUsage portionKind.
- custom/derived getters, external EModelElement state and unavailable proxy semantics return unresolved/unsupported results until implemented. They must not silently become null/false/empty lists.
- the shared serializer owns mapping slot_state to/from JSON. Unknown external serializer omission behavior is a concrete qualification gap, not permission to invent a new persistence format.

`CreationRequest { route, classifier, container, user_arguments }`

Routes must remain distinct:

- `RawFactory`: allocation only.
- `MetamodelChild`: source `addChildInParent`, then initialization only if that caller actually invokes it.
- `ExplorerChild`: source intermediate-container switch, attachment, initializer and ViewUsage post-create representation effect.
- `ExplorerRoot`: source domain/resource selection, existing or newly created root Namespace, then child route; suggested and full menu availability are separate queries.
- `DiagramTool { source_command_id, structural_reference, endpoints, membership_context, direction, selection }`: source-specific command contract. A raw class plus an arbitrary parent is not a substitute for these arguments or guards.

`CreationPlan { ordered_effects, required_resolution, source_evidence, post_create_effects }`

Effect vocabulary should be sufficient for **all** initializer overrides and helpers, rather than a special case for three constructors:

- allocate raw object with independently reserved identities;
- attach a Relationship or an ownedRelatedElement in source order;
- assign typed scalar plus set/unset intent;
- assign reference endpoint or explicitly unresolved/null endpoint;
- inspect contextual owner/membership/namespace and exact-class sibling count;
- resolve a named standard-library type against actual available resources;
- report representation creation/exposure/reveal/selection as distinct post-create effects.

Generated switch order and direct case/helper calls define the effect sequence. Null-returning overrides continue dispatch; non-null overrides stop it. Plan generation must not recursively initialize every allocated object: source helper children often intentionally remain raw. Plans must preserve the source InterfaceDefinition/ConnectionDefinition fall-through until an explicit reviewed compatibility decision changes it.

`Transaction::create_from_plan` (or equivalent existing apply integration)

- applies an accepted plan to one staged document with stale/foreign transaction checks, duplicate-ID validation and failure poisoning already provided by the shared model adapter;
- cancellation leaves the committed document and external effects unchanged;
- commit/undo/redo preserve all affected nodes, references and storage/set-state distinctions;
- does not perform an irreversible representation/database side effect during staging;
- returns an outcome that separates model creation, unresolved dependencies and pending external post-create effects. It must not return full route success when required diagram/library/serialization work is absent.

No new external runtime, Java fallback, TypeScript semantic implementation, database/service registration or package install is proposed. A new dependency is not justified by this document; reuse the reviewed model/registry/JSON closure where possible.

## Required source semantics before implementing contextual effects

1. `NamespaceImpl.getOwnedMember` derives from Membership relationships and ownedRelatedElement, not arbitrary recursive JSON children. Element owner and owning namespace use relationship/membership semantics and can involve proxy resolution.
2. Default names use the runtime EClass name and exact-class counts after attachment. Preserve count-includes-self, non-null stopping, actor/stakeholder predicate behavior and action/subject/objective naming.
3. Raw factory child IDs are generated for every helper-created membership/reference/conjugation/typing object, not reused from the parent. Inverse/reference consistency and source-order child graph construction must be explicit.
4. ViewUsage initializes a FeatureTyping with a contextual GeneralView lookup. When the lookup is missing, preserve source null/unresolved behavior rather than a fabricated stable ID. A strict product policy may block that outcome, but that policy is distinct from source behavior.
5. Raw booleans/enums/literals and set/unset flags require source-specific defaults. Metadata absence is not an enum's first literal, and null enum reset is not uniform across classes.
6. EClass menu availability, constructor availability and semantic model validity are separate answers. The complete class registry does not implement initializers, factory defaults, constraints, imports or menu commands.

## Independently buildable implementation milestones

| Milestone | Complete native scope | Remaining claim boundary |
| --- | --- | --- |
| F1 | All 167 raw domain allocation cases; all eight abstract/enum/foreign/unknown refusal cases; source default/state representation | No contextual initialization or serialization parity claim |
| F2 | Every one of 31 initializer overrides/helpers with exact generated dispatch across all 175 classes, contextual queries, graph effects and dependency outcomes | No explorer/diagram/representation completion claim |
| F3 | Every explorer menu candidate route and intermediate-membership rule; root-domain/resource behavior; explicit ViewUsage post-create contract | No AQL/browser/representation acceptance claim |
| F4 | Source-specific diagram/model create command adapters inventoried in the tool/method ledgers, with actual guards/selection arguments and required post-effects | Every unported command remains named as incomplete; wrappers do not count as implementations |
| F5 | Qualified serialization/default omission, restart/roundtrip/set-state preservation and representation/API integration; independent differential and end-to-end review | Only after this can the combined factory/initialization route claim full addressed scope |

These are bounded milestones toward the full implementation, not permission to call a registry-backed generic JSON builder “the full factory.” Existing unsupported features remain visible until tested.

## Qualification and precise open gates

- **Raw factory matrix:** enumerate all 175 EClasses plus seven EEnums. For each concrete class check actual class, distinct identity, effective constructor defaults and initial set-state. Do not count abstract ancestors as constructible. Compare existing source shape and generated factory switch directly.
- **Default/state matrix:** cover all 77 Java default constants, 115 cached/list/reference/ESet fields and all 65 directly declared attributes with inheritance/overrides. Include Feature.isUnique, FlowUsage snapshot default, visibility/kind defaults, numeric endpoints, null strings, lazy lists and enum set-null vs unset behavior.
- **Initialization matrix:** exercise every override and helper, including attached/unattached names; mixed-class/mixed-membership counters; AttributeUsage/ReferenceUsage flags; accept payload/receiver graph/directions; port conjugation without recursive reinitialization; both connection/interface end graphs; objective naming; expose flags; positive/missing GeneralView lookup; deliberately no-op Usage subtypes.
- **Route matrix:** all projected menu sets and source restrictions; direct command IDs outside visible menus; FeatureValue/Variant/RequirementConstraint/FramedConcern and ordinary membership selection; directed compartments and specialized model/diagram tools. Root Namespace wrapping is not replaced by root JSON content insertion.
- **External dependency gate:** obtain/pin actual Sirius JSONResourceFactory/EMF JSON source/configuration and qualified library/reference resolver behavior. Current repository source delegates these; absence must remain an exact blocker for persistence/default-omission acceptance.
- **Reference oracle gate:** upstream factory/initializer/menu/serialization tests have not run. A future isolated, authorized reference run must capture graphs/defaults/ESet/JSON presence and errors. Canonicalize nondeterministic fresh IDs only when comparing graph structure; retain actual stable-library IDs and link equivalence. Do not modify the baseline to force a desired behavior and then call it the oracle.
- **Persistence/history:** cancellation, stale/foreign transaction rejection, failed-operation rollback, undo/redo, restart and roundtrip retain links, IDs, unknown fields and actual saved states. A manifest/hash alone is not runtime equivalence.
- **Representation gate:** ViewUsage must ultimately produce its General View representation through the actual route, with required error/cancellation behavior. Merely creating FeatureTyping or returning a pending effect is not full user-facing creation parity.
- **Independent review:** a reviewer checks all inventory-to-feature links, the non-null switch semantics, raw-vs-initialized child distinctions, external gaps and the actual tests/artifacts before the factory milestone is marked complete.

Current exact decision requested: release one model/factory writer for the proposed new native modules and identity/query/serializer contracts, or amend the boundaries. This is a review proposal, not a request to run upstream software or broaden infrastructure authority. Until reviewed, this lane remains source preparation only.
