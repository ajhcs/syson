# M7 independent bounded syntax review

The documented syntax-only contract has no confirmed defect in this review.
Eight independent native tests pass. The same 37-byte fixture accepted by the
separately executed bundled reference parser is accepted by the Rust
recognizer, which explicitly reports `SemanticStatus::NotPerformed` and
`ast_status: NotPerformed`. This establishes one agreement on syntax acceptance,
not AST equality, semantic parity or completed parser replacement.

## Exact reviewed and executed revisions

Implementation source reviewed:
`a13682348e60451269973fda6d30a16aafa2c1b2`, integrated as
`f3f93117563f2159542aff7c3b11fcc056cc881a`. The crate tree is
`04b6eb836b02bc00096cf34095e7d33d4d6dce33`.
Independent harness authored in `4ee4b9334`; the final execution snapshots the
published integration revision
`f1890270ef6d11110c5e94aa5b1134f4565e1cfa` for both implementation and harness.
[The receipt](../runs/syntax-review-01/receipt.json) hashes every snapshot file,
the controlled runner, and each output stream. The exact original worker SHA
is not substituted for the reachable executed revision.

The source review covered the manifest, fixed terminal contract, all four Rust
source modules, generated grammar data, owner tests and declared limits. The
crate has no build script and uses the already reviewed cached serde_json
stack. Execution used `--offline --locked --jobs 1`; upstream Node, Maven,
package installation, services and browsers were not executed by this review.

## Independent checks

[The harness](../../rust/validation/syntax/tests/recognizer_regressions.rs)
checks:

- All 2,047 parenthesis strings of lengths zero through ten against an
  independent balance counter, including rejection of negative prefixes.
- All 511 two-symbol strings through length eight against the language `a* b*`
  under nullable left recursion, plus late nullable completions.
- Ambiguous nullable recursion, complete consumption, deterministic selected
  derivations and absence of orphaned CST nodes.
- Raw terminal boundaries for incomplete exponents, decimal/range ambiguity,
  repeated exponent signs, unrestricted-name escapes, raw string quotes,
  astral UTF-16 offsets and line-comment handling of U+2028.
- Malformed input and twelve grammar mutations: unsupported constructs,
  cardinalities, parameters, hidden-token extensions, imports, invalid/local
  and external references, fragment calls, terminal changes, unsupported
  action operators and duplicate rules.
- Exact input/token budget boundaries, low work/chart/CST budgets and
  cancellation requested from another thread while lexing a large synthetic
  note. No successful partial result is accepted.
- Actual KerML/SysML malformed documents, positive SysML examples and the exact
  shared reference fixture.

Selected CST checks cover valid byte boundaries, parent containment, reachable
children and lossless recovery of input from all visible and hidden token
spans. These assertions do not qualify upstream CST shape or ambiguity order.
The custom small grammars test the generalized CFG engine; they are not
additional SysML coverage claims.

The final eight-test run occurred at
`2026-10-22T21:42:51.989360+00:00` through
`2026-10-22T21:42:54.481404+00:00`. The native observation followed at
`2026-10-22T21:42:54.481598+00:00` through
`2026-10-22T21:42:54.511675+00:00`. Both exited zero. Earlier exploratory runs
are not substituted for these recorded results.

## Same-fixture bounded comparison

The input is exactly `package Oracle { part def Vehicle; }` followed by LF:
37 bytes, SHA-256
`ebe5c225b064119eb2584a9a7d4f268b0a10e8de0e15df77a9240c12f05cbdf8`.
[The native observation](../runs/syntax-review-01/observation.stdout.json)
contains 15 tokens, eight visible tokens and a selected 99-node CST, with root
span byte 0 through 37. It records 174 state-history merges; that number is
not an exact ambiguity count. Its SHA-256 is
`5ee8988ac5015dc73aafa1c149851324d8ad15cd99f334d7fd5dfd45b644fe08`.

The coordinator's [separate reference receipt](../runs/parser-oracle-01/receipt.json)
records an actual exit-zero run of the pinned bundle using the secondary Java
wrapper diagnostic flags, `dump --validate --stdlib none`, and empty stderr.
This review read that evidence and checked that its fixture bytes equal the
native fixture; it did not rerun upstream software. The reviewed reference
receipt hash is
`b355aa43cf2ba1fe20b4fc8963b0d5a2fbda86c77de31e24c93374812dc6de2f`.
The reference emits a semantic AST while Rust emits a syntax CST; their JSON
outputs were not compared for equality. No strict behavior-ledger row is
promoted by this independent single fixture.

## Source declaration correspondence and limits

An independent static extraction decoded the two grammar template literals
from baseline bundle
`6ba2ac64e6b16ee2b05752ad55395eebb87d4910` and compared their complete JSON
values with the retained candidate documents and Rust data. Both compare
equal: KerML has 282 total rules and SysML has 453, including twelve terminals
each. Candidate generated source bytes were separately read from Git at
`1326f234cae23b0e4777d5a48a069b17e9e332f1`; SHA-256
`fca1a64377f64c1aefb5476e89227cbc24f064ca40b7a58d6608646cae66149d`.
The pinned expression-terminal source SHA-256 is
`d13636ac574ea912eb5b29f9fca0e8af3e69ef662d5f4aac0154273132671b9e`.
No JavaScript was evaluated in this comparison.

The baseline bundle SHA-256 is
`47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd`.
Rust KerML/SysML data hashes are respectively
`ddbc07f673e2be095931c3f5a059ecdd75ec51be474184a80577ebdcbd14a0c6`
and
`d00a3a2afe891974ead47e3a36d9540d461cac015c66d7977d0d1eecd6301620`.
Structural declaration equality does not prove equivalent lexer generation,
Chevrotain ordering/greediness, parser recovery, actions, AST construction,
linking, validation, standard library loading or complete bundle provenance.

The Earley implementation deliberately merges alternative histories by
production/dot/start and returns one discovered derivation. Its membership
algorithm and nullable scheduling passed the independent finite oracles;
this does not remove the documented differences from the upstream parser.
Resource limits are cooperative work/state/node limits, not a wall-clock or
whole-process memory guarantee. Quoted lexical errors and token coordinates
retain the documented bounded behavior; upstream diagnostics and CST spans
remain unqualified.

All 735 source rule dispositions remain **UNPORTED**. Individual rule
qualification, AST import, semantic behavior, full application comparison and
completion remain open. Zero upstream-verified strict ledger rows are claimed.
