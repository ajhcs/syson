The parser foundation had reflection, primitive conversion and transactional AST
operations but no adapter constructing package/part declarations from text. This
adds a finite frontend for packages, part definitions and named typed part usages,
with source-shaped membership wrappers, raw primitive lexemes, byte/UTF-16 spans,
mandatory defaults and immutable kernel parenting. Qualified references stay
unresolved; final TypeReference markers use F08's positional type expectation.

The whitelist is explicit. Package/root usages use OwningMembership while usages
inside part-definition bodies use FeatureMembership. Valid source outside the
family returns Unsupported; malformed finite input returns a position. The
adapter does not perform full grammar recognition/recovery, native resource import,
semantic linking, callback application or child-cache behavior. F08's ordering
qualification remains an unqualified source-derived candidate.

Validation: thirteen tests, formatting and all-target Clippy pass offline/locked.
Four retained source documents, 14 selected obligations of 848 and 170 conservative
keyword exclusions pass source/hash checks. The original capture passed all 13
tests but failed two style lints; exact failed source/streams/runner are retained,
and the final capture passes after two expression-form refactors. The initial
zero-keyword preparation correction is also recorded without invented evidence.
No native runtime or AST differential golden was executed.

This PR targets the immutable F08+F10 dependency snapshot. Land both prerequisites
first, then retarget to merged integration, inspect the resulting diff and rerun
appropriate checks. Do not merge this child into the snapshot branch. No parent
source, shared manifest, held writer path, backlog, package baseline or service
is changed.
