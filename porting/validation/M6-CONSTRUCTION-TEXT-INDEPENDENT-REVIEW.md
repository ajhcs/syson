# Independent review: source-backed construction and UI text boundaries

Reviewed model: `ae1fc55bee6cd6cac021eedaff74937a5b99a98f`, library SHA256 `56f36bdd29b6b2403756def16d69848c55ef97d5c58b4f3164182bf336bc6ea6`. Reviewed web capability update `7a8f1eff911e3f02b3f56c1c222f0d7b6c82617a` and text fixes `436fdb455dcdb167fb3c23089994926494841c42` / `300815758c3396cac45860b513ced7469442f772`; final web library SHA256 `514a76105c5577c4799eb9816c39b75f0f4016aaf0b7a80deb3d5d03d3a34429`.

Disposition: accepted for the reviewed command-source rule and native text/capability boundary. This validator changed no implementation source, started no browser/display/server and added no upstream-verified behavior claim.

## Membership and typing source review

Pinned upstream `MetamodelMutationElementService.addChildInParent` at baseline `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`, lines 396-406, accepts an Element parent. It selects FeatureMembership precisely when the child is a Feature and the parent is a Type, otherwise OwningMembership; it then appends the child and membership to the two containment lists.

The adapter now applies this rule to its three constructors: Package and PartDefinition use OwningMembership; PartUsage uses FeatureMembership under known Type parents and OwningMembership otherwise. `Document::can_create_in` and `is_instance_of` use the same strict known-domain classifier resolution as the actual commands. Unknown prefixes/classes, enum declarations used as object classes and missing IDs do not acquire positive capabilities. New IDs, containment shape validation, transaction poisoning and identity preservation remain enforced. Typing accepts source-known PartUsage/PartDefinition subclasses and stores stable endpoint IDs.

The checked-in `source-membership-matrix.json` derives parent ancestry from the pinned Ecore, independently of the command implementation, with both Ecore and method-file source hashes. Its synthetic object contexts isolate the source method; they do not prove that an abstract class can be instantiated or that a resulting whole model satisfies SysML constraints.

Executed creation suite: **3 tests passed**. It performed **525 constructor/context combinations** (three supported constructors across 175 known domain EClasses), verified exact membership classes, child identity and unrelated-field preservation, and roundtripped each output. It also accepted all **36 known polymorphic typing endpoint pairs** in the pinned graph. Unknown/foreign/enum parents failed consistently with capability checks, left the last staged value intact, poisoned commit and preserved committed state/history.

## Shared UI capabilities

The web option lists and action guards now call the shared document capabilities. The independent native ABI regression requires an imported ActionUsage to appear as a parent and to actually stage/commit package creation. The prior test's allowance for a hidden unsupported parent was intentionally removed after backing support was implemented. Generic rename continues through the scalar setter and succeeds for a known imported LiteralInteger.

These checks establish capability/command agreement, not membership semantic validity, a general class factory, all initializer effects or runtime differential parity.

## Multiline and saved text preservation

The earlier M5 multiline issue is resolved at the reviewed Rust boundary: string and rename controls use textareas, add a leading LF sentinel to protect a saved leading newline, encode CR for markup and explicitly display NUL as the HTML replacement character. When a submitted value equals the saved value after the documented textarea/browser normalization (CRLF/CR to LF and NUL to U+FFFD), the handler retains the original saved spelling. Actual edited text is stored as submitted. Full arbitrary-input equivalence beyond this normalization is not asserted.

Executed public native byte-ABI suite: **4 tests passed**. Besides capability/rename checks, it verifies multiline LF changes, unchanged saved CRLF/CR/NUL values and unchanged name line endings, then decodes the actual rendered export and checks preserved JSON values. The tests supply the normalized strings explicitly and inspect rendered markup; **they do not execute the DOM or claim actual browser normalization, accessibility, keyboard or visual acceptance**. Those remain the integration lane's separate checks.

## Execution and remaining scope

Exact reviewed model/web Git blobs were copied into isolated build snapshots; cached dependencies remained unchanged. One offline/locked compiler job ran the two targeted binaries: **7 tests passed, 0 failed**. No additional unresolved correctness defect was found in these scoped contracts. Check remaining unported semantics, parser/serialization behavior, external dependencies, full API/data parity and upstream runtime status in the project ledgers; zero behaviors are promoted to verified by these checks.

Reproduce after integration:

```
CARGO_TARGET_DIR="${SYSON_BUILD_DIR}/capability-target" cargo test --offline --locked --jobs 1 --manifest-path rust/validation/Cargo.toml --test creation_regressions --test web_contract_regressions -- --nocapture
```
