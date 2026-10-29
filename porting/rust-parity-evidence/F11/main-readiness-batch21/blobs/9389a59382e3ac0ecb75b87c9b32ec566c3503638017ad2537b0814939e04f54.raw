# F11 source and implementation review

The root coordinator independently reviewed the finite whitelist, source wrapper
context, F08/F10/kernel integration, test cases and explicit exclusions. The finite
family was approved before implementation. The final production-size exception
is one coherent lexer/parser/builder adapter, not an expansion to all grammar.

A source review refinement preserves body-dependent membership wrappers:
`UsageMember` in root/package bodies yields `OwningMembership`, while selected
`OccurrenceUsageMember` in a part-definition body yields `FeatureMembership`.
The exact source rows are retained in source-manifest.json. All 14 selected
obligations remain a finite subset of 848; original inventory statuses are not
promoted to native coverage.

Before compiler execution, the author identified the distinction between grammar
cross-reference type `Element` and the custom final TypeReference expectation
`Type`. Root held the compiler and approved integration with F08's positional API.
The test queries F08 independently for every emitted marker and additionally
asserts Element/Type expectations. These are kernel metadata and unresolved
references; no native link or runtime golden is claimed.

The initial keyword producer walked the serialized `decoded_json` string and
produced zero words. That preparation draft was rejected, corrected to walk the
`grammar` object, and bounded by known-keyword and nonzero checks. The corrected
producer verifies 170 conservative keyword exclusions and exact source hashes.
The first output is retained verbatim; draft bytes are explicitly labeled as an
exact-write-command reconstruction, without invented execution timestamps.

The first coordinator capture, f11-01, passed formatting and all 13 tests but failed
Clippy on two `needless_late_init` warnings. Its full source/stream/runner packet
is retained. Root approved only two expression-form refactors, with no behavior
change or lint suppression. The exact failed-source bytes were hash-checked before
editing. The final capture's actual outcome belongs in VALIDATION.json.

Scope excludes native parsing/recovery comparison, remaining grammar families,
CST identity, native child caches, processor callbacks, semantic link/validation,
resource mutation and engineering acceptance. Local passing tests cannot qualify
those boundaries. The immutable dependency snapshot is a review base; both F08
and F10 must land before retargeting this child to integration.

Exact-file whitespace attributes retain the original test transcript terminators
and Clippy's trailing spaces in its suggested removed-code examples. Original
streams are not normalized or reconstructed. No broad whitespace rule is added.
