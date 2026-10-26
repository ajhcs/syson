# F03: public namespace import evaluation on observations

`syson-imports` evaluates one bounded, resolved non-Type namespace family using
caller-owned immutable observation snapshots. It is stacked on F02 at
`15501a5ff2459cd1056809b70f23d0d2ae895b8a`; the upstream source pin remains
`6ba2ac64e6b16ee2b05752ad55395eebb87d4910`. The F02 source manifest pins the six
metamodel blobs and method ranges used here. Expected corpus results remain
manually source-derived, rather than Java/EMF runtime goldens.

## Separate evaluation modes

- `Visible` evaluates `MembershipComputer.visibleMemberships` with public owned
  memberships before public imports, shared visited state and recursive results
  appended after direct results. Private/protected entries are skipped before
  touching their member/import target observations.
- `Import` models a direct NamespaceImport or MembershipImport entrypoint. A
  nonrecursive membership import retains its member even if its namespace target
  is excluded; recursive import of an excluded namespace retains neither.
- `Imported` models `Namespace.importedMemberships`: owned effective names seed
  one conflict filter, self joins exclusions, and each top-level import gets a
  fresh helper. This outer method does not filter owned-import visibility; the
  nested helper does filter public visibility. These contexts remain distinct.
- `Supplied` accepts pre-observed ordered import results and owned membership
  identities. It performs the outer seed/filter stage and never claims that
  supplied lists were computed from a graph or that their exclusion metadata was
  revalidated. F02 retains their source call/exclusion metadata separately.
- `Filter` exposes the observed name-conflict predicate for the frozen trace
  cases. `MembershipsOfVisibility` explicitly returns `NullUnimplemented`, matching
  the generated native TODO; it is not replaced by an empty list.

A direct Base Import returns source null, represented here by adapter disposition
`Error::NullImport`. In the outer Namespace pipeline that null would fail stream
flattening. In contrast, MembershipComputer's internal dispatch returns an empty
list for an unrecognized/base import. Tests preserve this context distinction;
`NullImport` itself is not a claim that the direct native method throws.

## Identity, lazy reads and admitted scope

Namespace/member tokens denote supplied object identities, never UUID equality.
Snapshots own their data in an immutable Arc; a new snapshot cannot alter an old
one. The caller must bind those observations to the correct native resource,
revision and object instances. Tokens are scoped by caller convention to one
snapshot; this API does not detect a token reused from a foreign snapshot.
No native adapter, stale-handle mechanism, getter
execution or proxy resolution is present.

`Alias` retains the supplied Java String reference observation: equal text with
different references may take the alias branch. Alias comparison is not inferred
from Rust text equality. Effective name and short name reads remain lazy; a
primary conflict does not read an unresolved short name. A short-name conflict
retains the earlier inserted primary name. Owned-name seeding reads both names
and ignores the alias branch. Getter side effects and unstable repeated getter
results are outside these immutable observations.

Reached unresolved observations, missing identities, Type namespaces and
`includeAll` branches return explicit errors. Unknown or unsupported data in an
unreached private/excluded branch remains untouched. Missing target (`None`) is
separate from unresolved target. Null membership objects in a membership list,
Only well-formed Unicode strings fit the Rust String observations. Type
inheritance/conjugation, includeAll/private import families, native cache
lifecycle, URI loading, malformed Java strings with unpaired UTF-16 surrogates
and whole-area import parity remain unsupported.

Rust-specific admission limits count evaluator steps, including visibility scan
items, and limit recursive depth to 1..128. Budget errors return no partial
success; they do not claim upstream limits, a hard RSS ceiling or a wall-clock
cancellation deadline. Input allocations, string comparison and snapshot vector
clones are not bounded in bytes by the step counter. Duplicate observation IDs
are rejected during snapshot construction.

## Exact corpus replay and focused checks

`generate_fixtures.py` verifies the exact F02 fixture SHA-256, translates its inputs
and copies its expected results into generated typed Rust data. It does not
calculate graph expected results. The full 19-case corpus is mapped: 18 retain
the original list/null/error-boundary disposition; `visibility-include-all`
returns explicitly unsupported because that family is excluded from F03. This
is not a claim of 19 successful native graph evaluations: some cases exercise
supplied lists, local filtering, null stubs or unsupported boundaries.

The handwritten suite adds lazy failures, hidden/excluded unknowns, source-context
base dispatch, separate helper lifetimes, immutable snapshots, duplicate tokens
and deep/work-budget cases. The production evaluator is 410 formatted lines;
its coherent scope and explicit limit/lazy-error handling are a small exception
to the proposed 400-line target. Generated corpus declarations and compiler
captures are separate from handwritten logic.

Reproduce with the installed toolchain and pinned Git objects:

```sh
python3 -B porting/area-inventories/DOMAIN-QUERY/imports/validate.py
python3 -B porting/follow-ons/F03/generate_fixtures.py --check
cargo fmt --manifest-path rust/crates/syson-imports/Cargo.toml -- --check
cargo test --offline --locked --manifest-path rust/crates/syson-imports/Cargo.toml -- --test-threads=1
cargo clippy --offline --locked --manifest-path rust/crates/syson-imports/Cargo.toml --all-targets -- -D warnings
```

Compiler checks are coordinator-controlled and recorded in validation receipts.
No Java application, EMF runtime, native graph adapter, package installation,
service, shared manifest or held factory path is changed or run by this PR.
The new crate location deliberately preserves the held native graph-adapter lease.
