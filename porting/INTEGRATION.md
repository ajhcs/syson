# SysON Rust port integration contract

Baseline: eclipse-syson/syson 6ba2ac64e6b16ee2b05752ad55395eebb87d4910 (main, package 2026.9.3); latest release observed v2026.9.0, not selected as baseline.
Fork: https://github.com/ajhcs/syson (ajhcs is a GitHub User, not an Organization).
Integration owner: Linux chat 01a16553-a3de-7010-8c4c-4edbc613c333 on builderbob, /workspace/repos/builderbob/syson, rust/integration.
Coordinator: 01a16552-abb7-70ef-afb9-c39a73433020. No merges authorized.

## Ownership

Five isolated worktrees under /workspace/worktrees/builderbob/syson:
- source-map: porting/source-map, branch rust/source-map.
- domain: rust/crates/syson-domain and porting/domain, branch rust/domain.
- frontend: rust/crates/syson-editor and porting/frontend, branch rust/frontend.
- persistence: rust/crates/syson-persistence and porting/persistence, branch rust/persistence.
- validation: porting/validation and rust/validation, branch rust/validation; independently reviews implementation.

Integrator alone owns rust/Cargo.toml, rust/Cargo.lock and porting root files. Crates start independently buildable, with no external dependencies in the initial slices. Shared wire/storage schemas must be agreed before coupling crates. Source preservation is not a retained Java/TS implementation of the proposed Rust product: it is the pinned reference. No runnable Rust product or full parity exists yet.

## Resource and authority boundaries

Host bob-admin workspace-status confirmed Samsung SSD 990 PRO 2TB, UUID dcb87552-b22f-411c-bd86-4e2da80f1c6f, ext4 rw at /workspace. Sandbox visibility alone was missing its device alias. Initial host compiler process inventory showed no active cargo/rustc/mvn/java compilers. Cap total compilation at two jobs: domain and frontend get one each; other lanes must request a slot. Native HOME/CODEX_HOME unchanged. Builds/caches remain under /workspace/build/builderbob/syson and /workspace/cache/builderbob. No displays claimed. No bob-migration, Kernspan, MBSE-Craft or benchmark writes. SAS/SnapRAID hold and all protected services remain untouched.

## Entirely Rust dependency policy

Product domain, backend, editor state and eventual browser UI must be Rust (browser code compiled to WASM). No Java server, TS frontend, JS modeling engine, or JNI server delegation may count as converted behavior. Browser WASM loader/generated bindings, browser engine/DOM, OS, database and native system libraries are explicit platform dependencies; no such runtime choices have been accepted implicitly. Initial crates use only std and have no generated bindings. Preserved LGPL standard-library data is not relicensed. LICENSE, LICENSE-LGPL, NOTICE and source notices remain authoritative. Sirius Web/EMF/AQL/Jackson/Spring runtime behavior and React/MUI/XYFlow behavior must be independently inventoried and replaced or explicitly blocked. External upstream APIs are not proven compatible by compiling this repository.

## Acceptance

For each ledger item record pinned upstream evidence, Rust implementation, upstream/reference test, Rust test, execution status and independent review. Unported, unrun, unsupported upstream, and incompatible are distinct statuses. Naming or geometry tests are bounded source translations, not GUI, semantics, accessibility, REST, GraphQL, database, collaboration, recovery or full-product acceptance. Undo/redo, cancellation/retry, recovery and E2E claims require executable oracle evidence. No percentage from line counts.
