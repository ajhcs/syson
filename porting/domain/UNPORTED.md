# Domain milestone / blocker ledger

Baseline: `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`.
This is the domain lane's honest remaining scope; integrator owns whole-product
inventory. No row below is implied complete by a passing primitive test suite.

| Milestone | State | Required next evidence |
| --- | --- | --- |
| D01 name/identity/query primitives | Implemented bounded source projection | Independent review and integration checks |
| D02 complete metamodel | Unported | Inventory every EClass/feature/opposite/derived operation and subclass override from Ecore + generated/custom Java; independent fixture coverage |
| D03 resolution and scope | Partial primitives only | Full membership/import visibility, inheritance, redefinition/subsetting, library resolution, cyclic/error behavior |
| D04 textual syntax and semantics | Unported | Parser and AST, direct-edit grammar, validation, serializers and roundtrips including Unicode/error recovery |
| D05 mutation and transactions | Unported | Mutation services, initializer defaults, rename/delete/type edits, atomic errors/cancellation, undo/redo and retry contract |
| D06 standard libraries and identity policy | UUIDv5 primitive only | Complete generated UUID selection, random v4 source, library namespace policy and shipped library/license fixtures |
| D07 backend services | Unported | GraphQL/REST, Sirius event processing, jobs, WebSocket collaboration, conflict/concurrency/auth contracts |
| D08 persistence and EMF compatibility | Unported; another lane contract needed | XMI/resource/proxy/opposite fidelity, migrations, imported-resource flags, stable identity and recovery |
| D09 graphical semantic integration | Unported; frontend contract needed | Exact services exposed to editors/layout/tools and realistic model creation |
| D10 executable differential oracle | Unrun | Pinned isolated upstream runtime with recorded dependencies, input/output fixture hashes and results |

## Decisions pending integrator reconciliation

1. Complete Rust metamodel representation and source generation strategy. The
   small `Model` projection must not become an undocumented alternate SysML schema.
2. API/persistence stable ID and null contracts, including frontend-to-server
   command and cancellation/retry/transaction envelopes. No adapter API agreed yet.
3. Full semantic engine: translate custom Java behavior and record pre-existing
   upstream TODOs separately from Rust gaps. For example ElementImpl.path returns
   null with TODO upstream; do not invent semantics or claim a functioning path op.
4. Independently approved dependency strategy for full async backend, serialization,
   cryptographic/random identity source and WASM interoperability. Initial crate
   needs no fetched dependencies; that does not resolve whole-product feasibility.

No blocker authorizes hidden Java/TypeScript runtime implementations, unapproved
software execution, access changes, or treating source-level tests as runtime parity.
