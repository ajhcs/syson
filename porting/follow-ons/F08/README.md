# F08: immutable AST reflection tables

`rust/crates/syson-reflection` provides immutable, owner-qualified reflection for
all 189 generated AST names and the 184-name list filtered by the five explicit
NON_TYPES markers. It exposes proper ancestor/subtype lists, same-type-inclusive
subtyping, effective local-before-inherited mandatory properties and checked
reference-position target typing. The eight generated reference-property IDs
remain a separate API from the positional override.

Every database creates a unique Arc owner. Handles retain that owner and cannot
be used with another database, even after their original database is dropped.
Indexes are private and are not persisted semantic IDs. Tables can be shared
through Arc without mutable registry state. Unknown names return an explicit
error; this is a deliberate restriction of the upstream unknown-string domain,
whose isSubtype method accepts equality of arbitrary unknown strings.

## Ordering is an explicit candidate policy

`ordering_qualification()` returns
`SourceDerivedStableShortSortCandidate`. TypesIndex partitions ancestor names
into namespaces, relationships and NON_TYPES unions, orders each partition,
then inserts relationships and unions at their source-defined positions. Its
subtype comparator is a partial order and is not a strict weak order. A generic
Rust sort is not used for that comparator.

The frozen Rust policy detects and reverses an initial strictly descending run,
retains an initial nondecreasing run, then performs stable upper-bound binary
insertion for the remaining names. All 567 retained input partitions (three for
each of 189 names) are source-mapped; the maximum length is 13 and is guarded in
the implementation. The independent static fixture generator uses a DFS closure
and CPython's stable comparison sort. Agreement of those finite candidate tables
is local static evidence. It does not establish a unique ECMAScript order for
an inconsistent comparator or prove the order of the original Node runtime.

Four exact ordered inheritance expectations retained in the source tests are
asserted separately: Element, Comment, Feature and OperatorExpression. All 189
candidate ancestor lists, reverse subtype lists, metadata declaration precedence
and all 35,721 known-type subtype pairs are covered by coordinator-executed Rust assertions.
Original source test files were inspected, not executed.

## References and metadata

Generated property lookup accepts only the eight pinned `<ReferenceType>:parts`
IDs and returns Element. The override uses a validated container and position:
all earlier parts target Element, and the last part selects Type, Classifier,
Metaclass, Membership, Namespace, Feature or ConjugatedPortDefinition for the
seven corresponding reference kinds. ElementReference and other known container
types fall back to Element. It determines a target type, not a resolved identity.
Zero-length/out-of-bounds positions are rejected; usize bounds are intentionally
narrower than the source's unguarded dynamic indices.

Mandatory metadata contains the local declarations first, then each candidate
ancestor's first declaration of a previously unseen property, and finally the
unowned `$children: array` override. The owning type is retained per declaration.
This crate returns metadata only. Array/boolean default application, child-cache
updates, parenting, parser dispatch, linking and semantic validation are not
implemented here.

## Reproduction and evidence

The generator verifies all four retained source-document hashes plus the exact
prepared declarations and summary before writing output:

```sh
python3 -B rust/crates/syson-reflection/tools/generate.py --check
```

The data generator and static check passed locally. A first static regeneration
check caught rustfmt rewriting the generated tables; explicit rustfmt skip
attributes fixed that formatting drift. No source behavior was changed by the
format repair. The implementation worker did not execute Cargo. The coordinator
executed the separate `f08-01` focused capture: formatting, eight native tests
and clippy with warnings denied all passed, with unchanged source hashes.
[VALIDATION.json](VALIDATION.json) links the original receipt and streams;
[REVIEW.md](REVIEW.md) records the coordinator's separate source review. The
coordinator also executed the captured tests; a second runtime executor is not
claimed. Browser and upstream runtime checks were not run.

`source-mapping.json` records exact pins, source hashes, all input partitions and
the ordering qualification. `qualification.json` keeps local and upstream status
separate. Upstream runtime comparison is NOT_RUN; upstream ordering parity and
full area parity are not verified. All historical grammar/AST obligation statuses
and existing execution-backlog ownership are unchanged.
