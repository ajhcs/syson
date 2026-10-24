# Connected Rust workflow checkpoint

The subsequent [native query adapter](domain/native-query/README.md) separates raw containment from semantic ownership and adds ordered resource snapshots with scoped identities. Source-derived local checks pass; native imports, URI/proxy behavior and ViewUsage application remain explicit gaps.

The [October 24 continuation](restarts/2026-10-24/README.md) adds an opt-in initialized construction workflow for ConnectionDefinition, InterfaceDefinition, AcceptActionUsage and PortDefinition. It joins the existing helpers to browser staging, cancellation, commit, undo/redo and selected JSON reload. The earlier checkpoint below retains its original scope; full application and upstream runtime parity remain open.

The `syson-workflow` crate connects the source-traced model, editor and library-resource filter. The exercised sequence creates a Vehicle package, a Wheel definition and typed wheel usage, stages and cancels a rename, commits another rename, undoes/redoes it, saves a new snapshot, opens it in a fresh store handle, verifies semantic JSON equality and qualified lookup by stable identity, then rotates its editor node while retaining center. These are local native Rust actions; no upstream application acceptance is implied.

## Source and implementation contract

Model structure and ID separation follow `porting/domain/MODEL-ADAPTER.md` and its pinned EMF SQL fixture. Rotation follows `porting/frontend/README.md`. Project-download library filtering follows `porting/persistence/README.md`. In-memory transaction behavior and immutable filesystem snapshot storage are explicitly proposed Rust adapter facilities. They are not translations of the upstream PostgreSQL schema, database transaction semantics, archive export or collaborative history.

Snapshots contain the selected EMF JSON document only. Diagram geometry, undo/redo history, migration scheduler state and cross-resource data are not persisted. Snapshot keys are bounded ASCII identifiers. Writes use create-new staging, synced file bytes, an atomic hard link which cannot overwrite a previous snapshot, and directory sync on Unix. Staged files are retained on failure, never automatically promoted. Read accepts at most16MiB; save uses the same explicit adapter limit, which is not an upstream limit. The filesystem must support same-directory hard links. A filesystem error after publication can have an uncertain result: check the immutable target before retrying. No automatic lock recovery or multi-user store is claimed.

The restart check uses a fresh store object and reloads actual file bytes; no process kill, power loss, filesystem fault injection or database recovery was performed. Corrupt/unfinished JSON is rejected, a valid earlier snapshot remains readable, duplicate writes fail, and invalid path-like keys are rejected. Test artifacts remain in the caller-selected build volume for inspection.

## Reproduce

Set `SYSON_BUILD_DIR` and `SYSON_TEST_DIR` to separate directories on the approved development volume. Run:

```
CARGO_TARGET_DIR="${SYSON_BUILD_DIR}" CARGO_BUILD_JOBS=1 cargo test --offline --manifest-path rust/crates/syson-workflow/Cargo.toml
```

The initial integrated run passed both connected workflow tests. Source-derived unit and independent regression suites remain separately counted. Browser UI, accessibility, HTTP/GraphQL, textual parser, database, concurrency/retry and pinned upstream runtime comparisons remain unrun or unported.
