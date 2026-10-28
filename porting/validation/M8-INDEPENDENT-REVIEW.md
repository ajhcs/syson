# M8 independent AST operation-kernel review

Ten independent native tests pass against the fixed kernel. Two concrete
original defects are resolved; no additional bounded-contract defect remains
confirmed in this review. This is generic construction-operation evidence,
not grammar evaluation, upstream runtime differential testing, semantic AST
parity, linking, CLI serialization or SysON import qualification.

## Exact source and execution

Original reviewed implementation:
`5c3755b13cb4bbb4fe02ff96a3192103a94a56e2`.
Original `rust/crates/syson-ast/src/lib.rs` SHA-256:
`5f38adb5723ed89f76dc41c37d9717d8ef5e0116fe9794ea6c09caeded435af8`.

Fixed reviewed and executed implementation:
`9e2337e4ec0a74c675f9e53c98df008a3dc50036`.
Fixed library SHA-256:
`3317db9c1042d4d5c49aefa1b660ab1eef2e8409861345ed476d7f50f11d74c1`.
Crate tree: `b095afc364899c55011e7bb9ad5e88c88647de71`.
Independent harness:
`183d77bf06e7a40a926cf73aefac918c0742c3bf`.
Every staged implementation/harness file was compared with its exact Git blob
before execution; [the receipt](ast-review-01/receipt.json) preserves the hashes
and genuine executed revisions, rather than relabeling cherry-picked commits.

The actual native interval was
`2026-10-22T22:00:30.716661+00:00` through
`2026-10-22T22:00:31.235398+00:00`, exit zero, ten tests passed.
The command was `cargo test --offline --locked --jobs 1 --manifest-path
rust/validation/ast/Cargo.toml --test kernel_regressions` in an exact source
snapshot. Both manifests and the std-only dependency graph were inspected;
there are no dependencies beyond the reviewed kernel and no build scripts.
No upstream software, package installation, browser or service was executed.

[stdout](ast-review-01/stdout.txt) and [stderr](ast-review-01/stderr.txt) are
captured streams. Only the selected build-directory prefix was replaced in
portable stderr; the receipt separately records raw and portable hashes. Raw
bytes were captured in memory. Prior owner failure and passing-run artifacts
were not overwritten or treated as this independent execution.

## Resolved findings

**P1: foreign handles could bind unrelated nodes.** The original NodeId held
only a per-builder index. Multiple builders started at index zero, and numeric
lookups could read or mutate the wrong builder's local node, associate the
wrong CST element, or finish/lookup an unrelated arena. Rollback nonreuse
within one builder did not establish ownership across builders.

The fix makes NodeId's owner/index fields private, checks owner identity in
builder and arena lookups, and reserves owner namespaces using a monotonic
checked atomic counter. `Builder::new` returns a Result for exhausted owner
space. `value()` remains a local index and cannot reconstruct a handle. Arena
clones retain their logical identity. Static review found no wrap/reuse path
in the checked reservation loop; process-wide exhaustion is not independently
runtime-reproduced by this harness.

The independent foreign-handle test exercises lookup, target assignment,
node-valued assignment, nodes inside flat arrays, defaults, both merge sides,
subrule adoption/assignment, CST association, reference target owner, finish
and immutable-arena lookup. All reject foreign handles without aliasing; a
cloned arena still accepts its own handles. Original owner reproduction
artifacts remain in `porting/ast/ownership-repro-*`; they are separate evidence.

**P2: Append could create a nested array while returning success.** Incoming
flat Array values passed validation, after which Append pushed the entire
array into the property's array. Only consuming finish rejected the now nested
state. This violated the declared flat-property-array kernel contract and
turned an invalid operation into a later destructive freeze failure.

The fix rejects an Array-valued Append before field mutation, including empty
arrays. Independent tests verify the exact error and unchanged existing array
and scalar slots, then successfully finish the retained state. Set(flat array)
remains supported. Consumed work is intentionally retained on this rejection.
The owner preserved its original failing repro in `porting/ast/flat-array-repro-*`.
This is a local Rust supported-shape restriction, not a claim that upstream
JavaScript universally rejects nested arrays.

## Independent operation coverage

[The public-API harness](../../rust/validation/ast/tests/kernel_regressions.rs)
checks nested transaction success/failure, inner success followed by outer
rollback, escaped-node invalidation, retained ID/work consumption and rollback
of frames, fields and explicit CST retargets. Cancellation requested inside a
transaction restores construction state while retaining the external flag and
already consumed work.

Action failure rolls back defaults added before unknown-type failure. Valid
nested wrappers derive scalar/array containment links. Source-first merge
preserves target scalar precedence, explicit null booleans, array order and
mixed scalar/node array indexes; unreachable temporary nodes and their
unretargeted CST associations are pruned. Explicit CST retargeting remains
separate from merge. Positive subrule adoption switches the active frame and
preserves source-first order; self-adoption errors without changing the frame.

Other checks cover datatype frame errors/concatenation, zero-length subrule
assignment suppression, positive subrule containment, frame/node limits,
UTF-8 versus UTF-16 spans, invalid spans, missing/null/boolean/array defaults,
first-name metadata precedence, raw presence lexemes and unresolved references
that do not manufacture containment or resolved targets. Number(f64) accepts
an explicitly supplied NaN as an input slot; that test establishes no
JavaScript conversion or wire-format behavior.

## Source contract and limits

Source tracing uses the pinned SysON bundle
`6ba2ac64e6b16ee2b05752ad55395eebb87d4910`, SHA-256
`47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd`,
construction/assignment/action/default behavior around lines 41020–41185, and
retained candidate parser/reflection source pinned at
`1326f234cae23b0e4777d5a48a069b17e9e332f1`. The complete retained source and
AST-VALUE-CONVERSION.md define later obligations. Static grammar declaration
correspondence is not proof of candidate-service/runtime equivalence.

The kernel receives caller-selected operations and converted values. It does
not select a parser derivation, execute a grammar/CST walk, omit recovered
inserted tokens automatically, derive reflection inheritance, invoke Syside
hooks, sort semantic children, resolve references or project CLI JSON. Its
lexeme fields are caller-provided source evidence rather than an input-owning
parser validation boundary. ID ownership is process local, not a serialized
identity protocol. CST identity numbers are explicit caller identities, not
automatically inferred tree ownership.

Transactions roll back returned errors; panics are outside that contract.
Caller allocation, snapshot cloning and arbitrary callback work are not fully
metered. Work/node/frame limits and cancellation therefore do not constitute
a wall-clock or whole-process memory sandbox. The tests do not broaden these
documented limits or claim exhaustive operation-sequence verification.

The implementation authorship and independent harness/review remain separate.
All 848 AST obligation dispositions and 735 rule parity dispositions remain
UNPORTED. Zero upstream-verified strict ledger rows are claimed.
