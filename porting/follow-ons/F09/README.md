# F09: ordered AST dispatch plans

This crate computes dispatch plans for the pinned Syside candidate type-map
algorithms and the parser's five postprocessor registrations. It invokes no
callback, constructs no AST and makes no JavaScript/runtime parity claim.
It is stacked on F08, commit `7d1eb8a1a05c292c78ca27482158c6dfe5196c3b`.
Merge the prerequisite before retargeting this child; do not merge into its
feature branch as an integration destination.

## Three separate algorithms

The exact retained source pins and reviewed ranges are in
[source-manifest.json](source-manifest.json), candidate commit
`1326f234cae23b0e4777d5a48a069b17e9e332f1`.

- `types.ts:149-169`: most-derived selection tests the direct entry, then proper
  ancestors in order, then default; only truthy entries are selected. Expansion
  visits every generated name and omits names with no truthy result.
- `types.ts:179-203`: block merge takes every truthy direct/ancestor array and
  ignores default. Reverse order prepends entire blocks. It preserves each
  block's internal order and all duplicates. Empty arrays remain truthy.
- `types.ts:205-229`: full-chain collection appends direct/ancestor payloads and
  default. Supertype-first reverses the entire resulting chain. Array payloads
  remain individual payloads, unlike block merge.
- `parser.ts:230-247`: identifiers for fixOperatorExpression, addLoopMember,
  finalizeImport, createEmptyParametersInTransitionUsage and
  createMissingEndsInSuccessionAsUsage are expanded with most-derived selection.
  A string construct type selects from that map; symbols, undefined and unknown
  strings select nothing. No processor function is implemented or executed.

`Registry<T>` owns an Arc-backed reflection owner and registration map. Type
handles from another owner are rejected at registration and every typed lookup.
`Entry::Falsey` explicitly classifies undefined/null/false/zero/NaN/empty string;
`Entry::Truthy(payload)` is a caller assertion of a truthy source value, not a
Rust coercion. Callback identifiers are naturally truthy source-domain payloads.
Absence is represented by an unregistered key. `Registry<Vec<T>>` can distinguish
a truthy empty block from absence or falsey values.

Duplicate registration keys are rejected as an explicit Rust input restriction:
a JavaScript object literal instead exposes its final value for each key.
Duplicate payload identities under distinct ancestor keys are retained. Default
is a separate field and cannot be registered as a pseudo-type. Unknown registry
keys, inherited object/prototype properties, malformed non-array block values,
dynamic object coercions and arbitrary JS values are outside this typed API.
Payload references are borrowed, and shared Arc identities remain shared; the
crate does not freeze interior mutability supplied by a caller's payload type.

## Ordering and cache limits

The public `ordering_qualification()` returns F08's
`SourceDerivedStableShortSortCandidate`. The candidate ancestor table is not an
assertion about the original JavaScript sort runtime. Strict retained source
expectations for simple chains are tested independently; other results retain
that ordering qualification. Source/artifact correspondence to SysON's embedded
parser remains unproven.

Registrations and lookup plans are immutable. This crate neither reproduces nor
repairs parser/defaulting/child-cache side effects. In particular, retained
`parser.ts:259-262` mutates the existing `$children` array by clearing and pushing
streamed children; it must not be inferred from a dispatch plan that this cache
has been updated or replaced. Reflection metadata alone does not apply defaults.
Callback application, all postprocessors, linking, semantic validation and full
parser parity remain outside F09.

## Focused evidence

Fifteen tests cover the exact retained type-index assertions, the five processor
IDs and inherited Import dispatch, explicit falsey inputs/default omission,
all 189 generated names, empty-array truthiness, reverse block order, chain
reversal/default placement, duplicate payload identity/items, foreign owners,
registration restrictions and non-string/unknown construct inputs.

The coordinator ran the existing toolchain offline/locked with one compiler job
and one test thread. Formatting, fifteen tests and all-target Clippy with warnings
denied passed, with unchanged source hashes. Exact receipts, original streams
and executed runner are under [validation/f09-01](validation/f09-01/).
Source checks validate three retained documents and their reviewed line hashes;
no upstream script or runtime was executed. [VALIDATION.json](VALIDATION.json)
and [REVIEW.md](REVIEW.md) distinguish this local evidence from acceptance.

```sh
python3 -B porting/follow-ons/F09/check_sources.py
cargo test --offline --locked --manifest-path rust/crates/syson-ast-dispatch/Cargo.toml -- --test-threads=1
```

[Translation provenance](../../../rust/crates/syson-ast-dispatch/NOTICE.md) retains
the original Sensmetry copyright/license qualification. No package, shared
manifest, source-parent file, backlog, existing hold or service was changed.
