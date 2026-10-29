# F12: structural operator-expression postprocessing

The module ports the structural branch of the retained Syside
`fixOperatorExpression` handler. It operates on synthetic, already-constructed
operation-kernel nodes. F11's finite text parser still accepts only its package
and part family; this change adds no expression-text parser or semantic linker.

The [manifest](source-manifest.json) pins three retained source documents and
exact line ranges at captured source candidate
`1326f234cae23b0e4777d5a48a069b17e9e332f1`. The corresponding text is retained in
[source-excerpts.json](source-excerpts.json). The preparation script also names
19 static SysML `ClassificationExpression` obligation tuples from the existing
848-entry inventory. Those associations are not claims to implement the complete
classification grammar, and no inventory status is promoted.

## Exact structural branch

The parser source at `parser.ts:57-87` is narrower than a general unary/binary
rewriter. Falsey `operands` becomes an empty array. Only an empty array with the
exact operator `istype`, `hastype`, `@` or `as` creates this chain:

```text
OperatorExpression.operands[0]
  -> FeatureReferenceExpression.expression
       -> ReturnParameterMembership.target
            -> Feature
```

The source gives none of those three implicit nodes a CST. Nonempty operands
preserve their encounter order and skip the operator condition. Ordinary unary,
binary and already-repaired nested expressions receive no extra operands.
`@@`, `meta`, case variations and whitespace-modified operator strings do not
match this handler's four-value list.

`OperatorPostprocessor::pinned()` owns F08 reflection and F09's actual parser
registry. `apply(builder, node)` runs only when that registry selects
`FixOperatorExpression`. It does not invoke the other four registered handlers.
The metadata builder exposes previously implemented F08 generated declarations,
not native constructor acceptance for every declared kind. F08's source-derived
ancestor ordering remains a candidate, with no JavaScript runtime qualification.

Absent, null, false, positive/negative zero, NaN and the empty string are the
representable falsey operand values. Existing arrays are admitted. Arbitrary
truthy non-arrays produce `UnsupportedOperands`; this is an explicit Rust input
restriction, not a claim that JavaScript would throw or normalize those values.
For an empty array, non-string operator values cannot equal any of the four
source strings. Input observations execute no JavaScript getter or proxy.

## Parenting, caches and transaction boundary

`createNode` and `assignNode` in the retained custom reflection source perform
mandatory defaults, immediate container assignment and native cache updates.
The adapter applies the same F08 mandatory metadata to the three implicit nodes
and writes the structural `operands`, `expression` and `target` fields. Calling
`Builder::finish` establishes immutable parent/property/index relationships.
Immediate mutation of native `$container` fields is not represented.

Native `$children` and `$childIndex` cache aliases are **unsupported**. The
operation kernel treats node-valued arrays as structural containment, so copying
those aliases would create repeated containment. Cache defaults remain empty;
the source's cache duplication/index quirks are not silently normalized or
claimed to be reproduced. CST association and parser-wide cache collection also
remain outside this unit.

Each selected application uses one existing kernel transaction. Errors roll
back all new nodes and structural field writes, including operand normalization;
spent IDs and work are not recycled. This atomic failure policy is a Rust
adapter guarantee, not a claim that the upstream JavaScript handler rolls back
its partial mutations. Unsupported operand admission is checked before mutation.
Cancellation and allocation/work budgets use the existing kernel's controls.
No partial immutable arena is returned by this module.

## Checks and limits

The twelve new test functions cover all four insertions, explicit empty/falsey
operands, exact nonmatches, unary/binary/nested order, idempotence, skipped other
handlers, typed admission errors, foreign handles, cancellation and forced
allocation failure after partial creation. They also check non-recycled IDs,
final structural parenting, source CST retention, implicit-node defaults and
the deliberately unimplemented cache aliases. Expected structural behavior is
manually source-derived, not a runtime AST golden.

Source preparation uses Python stdlib and reads only retained source files:

```sh
python3 -B porting/follow-ons/F12/prepare.py --check
```

Compiler checks were coordinator-run serially, offline and locked. All 25 test
functions passed: 12 new operator tests plus 13 inherited F11 tests. Formatting
and all-target Clippy also passed in `f12-01`, with source unchanged. Exact commands,
38 input hashes, original streams and results are recorded in `VALIDATION.json`
and the validation packet. The worker did not run Cargo. No JavaScript, Langium, Syside,
upstream SysON, browser or differential oracle ran for this unit. Whole-handler
native equivalence, expression parsing, all remaining construction obligations,
semantic import and whole-area parity remain unverified.

This branch targets the frozen F09/F11 dependency snapshot. Land both
prerequisites first, retarget to integration, inspect the resulting diff and run
appropriate checks. Never merge the child into the snapshot branch. Existing
held graph and validator paths remain unchanged.
