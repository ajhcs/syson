# Independent review: local model, snapshot workflow and web boundary

Disposition: bounded local API review accepted after the transaction ownership fix; full SysON behavior parity remains incomplete. No implementation changes authored by this validator. The schema ledger still has zero verified rows: these local checks supply no paired upstream runtime oracle.

## Reviewed implementation

- `syson-model`: EMF JSON envelope/value preservation, projected ownership/names and local staged editing. Initial integrated snapshot `29583e6fac7416d012f6f62ac281c7372abbfad7` contains model implementation from `69dd69265f63794cb0d0617e455cd7f2a87dd5f0`. Initial library SHA256: `1d2766bc921660548b733a207bc58179c04da6aa478d701f5b4228b2f6e7a64d`.
- `syson-workflow`: implementation `05e2665bbd94aeddd2eb5dec0beafd2d67c5db25`, immutable snapshot adapter. Library SHA256: `29f73e391449e3340e97760cd76cc404ecd8af75813ec29b7b4bb2e45b4fed97`.
- `syson-web`: source review of `659450a464b1122787a86655d772a471d842e50e` plus label containment correction `0e440d7a9152e5cb7234dc79777f3f1c87848876`; raw byte ABI and loader, request validation, escaping, staging/import/history boundaries and rendered string construction. Browser evidence collected by the integration lane is separate; this validator did not start a competing browser or execute browser checks.

## Confirmed defect and correction

P2: a transaction begun on one editor could commit into another editor with identical document and revision. The original guard checked only value equality and revision; independent test `transaction_cannot_commit_to_another_identical_editor` reproduced the violation of the proposed editor-owned API contract. Destination model/history were changed by a foreign transaction.

Owner fix `bccd73f622888bb7a28aae94f63a47672f672e96` adds a private reference-counted identity token shared with its transactions, compares token identity before mutations and returns `ForeignTransaction`. The validator independently read this change and reran the same test. Fixed library SHA256: `3d44fc172abf54a7edba8b33c21d52d9769fb59a5d9ac522c3880c094d42553e`. Finding resolved for the tested contract. This local transaction behavior is not claimed to reproduce an upstream transaction API.

## Executed independent regressions

Seven new tests in `rust/validation/tests/model_workflow_regressions.rs` cover foreign-editor ownership, stale transactions after an edit/undo ABA sequence, unchanged transactions preserving redo, poisoned operations retaining the previous staged value and committed history, malformed envelope/containment input returning errors without panicking, duplicate snapshot save preserving published bytes, and interrupted staging staying unpublished while other keys remain usable.

Initial M2 run: **6 passed, 1 failed** (foreign editor accepted). Corrected run: **7 passed, 0 failed**, plus all **7 earlier source-derived regressions passed**; total **14 passed**. Exact reviewed files were copied from Git objects to fresh build trees, leaving implementation checkouts unchanged. The corrected snapshot differs only in the model library fix. Cargo ran offline, one compiler job, with the cached dependency closure inspected before execution: manifests and active serde_json/serde_core/zmij build scripts. Build scripts emit configuration, query rustc version and write generated serde private metadata under build output; no product runtime or upstream script was launched.

Reproduce after integration with an approved build directory:

```
SYSON_TEST_DIR="${SYSON_BUILD_DIR}/test-artifacts" CARGO_TARGET_DIR="${SYSON_BUILD_DIR}/validation/target" cargo test --offline --locked --jobs 1 --manifest-path rust/validation/Cargo.toml
```

## Source review boundaries

Document input validates its accepted envelope and recursively checks containment and distinct resource/element identity sets. Model creation revalidates caller-supplied UUIDs. Mutations clone staged state before changes; failure poisons the transaction and leaves committed state/history unchanged. Revision increments across undo/redo prevent an old transaction from becoming valid merely because the document returns to its earlier value.

Projection intentionally recognizes only four namespace class names and flattens semantic ownership rather than retaining every EMF relationship object as a queryable node. Full namespace inheritance, imported memberships, proxies, effective-name overrides, relation queries, typechecking and parser semantics remain unverified/unimplemented; accepting an EMF JSON document does not establish complete semantic interpretation. Unknown JSON values surviving a roundtrip are a data preservation claim only.

Snapshot publication uses a new staging file, file synchronization and a no-clobber hard link; the published bytes survived a duplicate-save attempt in the independent test. Failed publication retains staging, and loading ignores staging. Retrying the same interrupted key fails because the staging file is retained; use a different key or an explicit future recovery protocol. The adapter is a local immutable snapshot facility, not the upstream database/archive format, a cross-platform durability qualification, concurrent store, or complete recovery system. Failures after publication can leave a destination present even if a later sync/cleanup returns an error; callers must inspect the key rather than infer absence from any error.

The web loader supplies browser entropy and forwards encoded fields/events; model interpretation and rendered HTML generation remain in Rust. Request size/type checks protect the accepted byte ABI. Rust escapes dynamic names/IDs/classes/messages and committed JSON before HTML/attribute/textarea insertion. Output pointer/length are consumed synchronously before the next state-changing ABI call. Label clipping keeps complete strings in the separate model list/title. These source observations and owner tests are not independent accessibility, keyboard, visual, browser-memory, load or full editor acceptance.

No additional unresolved defect was found in these bounded reviewed contracts. Full inventory/dependency closure, real upstream oracle runs, complete modeling semantics, collaboration/authentication, migrations and graphical editing remain open in the project ledgers.
