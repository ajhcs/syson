# Namespace import contract candidate (F02)

This freezes the **native metamodel API** at SysON commit
`6ba2ac64e6b16ee2b05752ad55395eebb87d4910` (2026.9.3). The port foundation is
`ad814c232ee24109639c1d52239cbeec6f4a3e92`. All expected results are manually
source-derived. No Java, EMF, SysON application, or Rust import evaluator ran.
This is preparation for F03, not implementation or qualification of namespace imports.

`contract.json` pins six complete Git blob identities and SHA-256 hashes, plus
method line ranges and range hashes. `fixtures.json` contains 19 ordered synthetic
cases. `validate.py` checks those pins, fixture references, call/exclusion order,
local conflict-predicate state transitions, and visited-state trace consistency.
It **does not compute graph traversal or prove the graph expected results**.
The source review remains necessary when changing a fixture.

## Pinned API behavior

All paths below are relative to
`backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/`.
The machine-readable pin includes exact line ranges and hashes.

| Source | Reviewed behavior |
| --- | --- |
| `impl/NamespaceImpl.java:188` | Seed effective owned names, append self to exclusions, visit owned imports in order, flatten returned lists, then filter with one shared conflict filter. |
| `impl/NamespaceImpl.java:129` | `getMembership(excluded)` filters owned memberships first, then imports, with another shared filter. This is distinct from importedMemberships alone. |
| `impl/NamespaceImpl.java:209` | `membershipsOfVisibility` is a generated TODO returning **null**, regardless of visibility. Preserve `NULL_UNIMPLEMENTED`; do not fabricate an empty list. |
| `impl/NamespaceImpl.java:90` | The private getter helper ignores its excluded argument and starts a fresh UniqueEList. |
| `impl/NamespaceImportImpl.java:180` | Constructs the deprecated metamodel MembershipComputer with supplied exclusions. |
| `impl/MembershipImportImpl.java:180` | Uses that same helper, not the similarly named services helper. |
| `helper/MembershipComputer.java:60` | Copies exclusions to a visited set; namespace entry skips visited identities before scanning owned/imported memberships. |
| `helper/MembershipComputer.java:81` | Public owned memberships precede public imports unless includeAll; recursive descendants append after direct results. Inheritance is excluded from these non-Type fixtures. |
| `helper/MembershipComputer.java:176` | Nonrecursive MembershipImport returns a resolved membership even when its namespace member is excluded. Recursive import suppresses that membership when its target namespace was visited. Null membership/member produces an empty result. |
| `helper/MembershipComputer.java:195` | NamespaceImport skips absent or visited targets; otherwise invokes visibleMemberships with import flags. |
| `helper/NameConflictingFilter.java:44` | Alias branch uses Java String **reference inequality**, not textual inequality. Caller must supply this observation. |
| `helper/NameConflictingFilter.java:62` | Name then short name use short-circuit AND. A fresh name remains inserted when the short name later conflicts; primary-name collision prevents the short-name check. |
| `helper/NameConflictingFilter.java:83` | Owned seeding uses effective name/short name without applying the alias branch. |
| `impl/ImportImpl.java:249` | Base Import returns null. If admitted into the Namespace flattening pipeline, null cannot supply Collection::stream. The fixture records a source-derived error candidate, not an observed native exception. |

The services package has separate similarly named helpers. It is not the dispatch
path selected by NamespaceImpl/NamespaceImportImpl/MembershipImportImpl in this
contract. Full helper differences and service injection remain outside F02.

## Fixture observations and limits

Object tokens (`A`, `B`, membership keys, element tokens) identify synthetic object
instances. They are not UUID-based equality. Member records carry pre-observed
getName, member-name, effective-name and effective-short-name values plus the
same-string-reference observation. Deriving those getter values, resolving proxies,
loading resources and native cache invalidation are excluded. `UNRESOLVED_NOT_ADMITTED`
is a contract boundary, not an assertion that an unresolved proxy returns empty.

Pipeline fixtures treat each import's returned list as a caller observation; they
check encounter order and filtering without recursively deriving the list. Graph
fixtures retain explicit input graphs, ordered expected lists and visited traces.
The cycle and diamond examples use one helper's shared visited set. Namespace's
separate top-level imports each construct a fresh helper, while sharing the outer
conflict filter. The positive recursive case appends child results after direct
namespace memberships; the negative recursive MembershipImport case skips an
excluded target, including the imported membership itself.

A future F03 evaluator should admit only resolved non-Type namespace observations
initially and retain typed membership-list/null/unsupported distinctions. It must
not replace Java alias identity with Rust string equality. Cache lifecycle,
inherited/conjugated Type memberships, proxy resolution, all import forms and
whole-area parity remain unqualified.

## Reproduce

From the repository root, with the pinned Git objects present:

```sh
python3 -B porting/area-inventories/DOMAIN-QUERY/imports/validate.py
python3 -B -m unittest discover -s porting/area-inventories/DOMAIN-QUERY/imports -v
```

The CLI validates the committed fixture digest before checking its contents.
The 17 unittest cases include mutation rejection for changed source hashes/anchors,
duplicate JSON keys and object identities, import/output order, exclusion order,
alias-reference behavior, short-name side effects, cycle trace, graph references,
missing obligations, and unsupported/null outcomes. These checks qualify the
contract artifacts; they do not establish native behavior parity.
