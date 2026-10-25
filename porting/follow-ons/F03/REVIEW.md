# F03 source review

Root independently reviewed the source, handwritten regressions and translator
before compiler allocation. Review identified context-specific Base Import
behavior: MembershipComputer internal dispatch returns an empty list for base
imports; the direct polymorphic Import API returns null, and the outer Namespace
pipeline cannot flatten that null. The implementation and regression retain all
three contexts. Direct `Error::NullImport` is a Rust null disposition, not a claim
that the direct native method throws.

Review also requested per-visibility-item budget charges and changed the F02
base-import fixture adapter to exercise actual `Query::Imported(A)` with a base
import, rather than bypassing the outer pipeline with a direct call. Those fixes
preceded the first compiler run; they are review corrections, not runtime failures.
The root accepted the bounded coherent evaluator's small source/review-size
exception (410 formatted production lines, 444 handwritten tests, 92 translator
lines, generated fixture declarations separate). No family expansion is implied.

The compiler capture passes 15 test functions, including all 19 inherited F02
cases in one replay test. Eighteen preserve source-derived expected dispositions;
the includeAll case is explicitly unsupported. Initial discussion accidentally
counted 16 test functions; actual raw output is 15 and all final reporting uses it.
Original source/stream/runner hashes match. No native runtime, graph adapter,
Type inheritance or whole-area parity is qualified by these checks.

The graph-adapter brief paths overlap an existing held lease. The root instead
authorized a new standalone std-only `syson-imports` crate, with no shared module,
manifest, backlog or native adapter edits. The future native binding is separate.
