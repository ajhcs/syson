# F12 review and validation

The implementation worker traced the retained parser handler and custom
reflection's structural/default/parent/cache behavior before writing code. The
parent coordinator reviewed the source plan and later the production module and
tests independently of that authoring work. The coordinator also performed the
Rust execution; no second independent runtime executor is claimed.

The review preserves the handler's narrow condition: only an empty operand list
and `istype`, `hastype`, `@` or `as` inserts the three implicit nodes. Nonempty
operands retain order. F09's real registry selects the handler; the other four
registered processor identifiers execute nothing in this module.

The source's arbitrary JavaScript truthy non-array behavior is outside the Rust
input domain and returns a typed admission error. Native `$children`/`$childIndex`
aliases and immediate parent-field mutation are explicitly unsupported. Kernel
finish provides structural parenting. Kernel transaction rollback is a deliberate
Rust failure policy, not attributed to upstream JavaScript.

`f12-01` passed all 25 test functions (12 new, 13 existing F11), formatting and
all-target Clippy without warnings. Original six output streams, receipt and
exact executed runner are preserved. All 38 captured source inputs matched the
current files, and the runner SHA-256 matches its receipt. The new source module
and tests have not changed since that capture. No failed compiler attempt occurred.

The separate source-pin capture passed three source documents and 19 associated
static obligation tuples, with all five inputs unchanged. It proves retained
source/correspondence integrity; it does not run a JavaScript handler or prove
classification grammar, native caches, full AST equivalence or semantic results.
The immutable prerequisite snapshot remains the review base. All shared
lib/manifest/lock wiring was made by the coordinator and is confined to the new
module registration and existing local dispatcher dependency.
