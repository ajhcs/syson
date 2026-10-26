# Ordered JSON observation comparator

`compare(left, right)` admits both complete JSON inputs before comparing their
full ordered trees. Object member order, array order, identity strings, types,
null versus absence and exact number lexemes participate. Thus `1`, `1.0`, `1e0`
and `-0`/`0` remain distinct. There are no ignored fields, selected subtrees,
profiles, identity mappings, sorting, tolerances or nondeterminism allowances.

Insignificant JSON whitespace and string/key escape spelling are explicitly
outside tree comparison: `"a"` and `"\u0061"` decode to the same string. Unicode
normalization, case changes and numeric spelling are compared. Duplicate decoded
object keys refuse rather than silently collapse, making JSONPointer diagnostics
unambiguous. Member-order/key differences report separate left/right pointers;
objects compare positionally, so an order change can also expose value changes.

Reports provide read-only differences. MATCH requires a complete comparison with
zero differences. At most 64 differences are retained; encountering another ends
reporting with incomplete MISMATCH. Missing subtrees are one presence difference.
Previews contain at most the first 160 Unicode scalar values plus an ellipsis,
with debug quoting for strings; emitted escaping can exceed 160 characters.
Previews never determine equality. `complete`
means difference traversal completed, rather than full scalar values printed.

Each input is at most 8 MiB, 100,000 values including pending child references,
64 levels (root depth one), 20,000 children per container and 4,096 bytes per
escaped JSONPointer. A generated hypothetical pointer for a missing counterpart
must also fit; otherwise the whole comparison refuses without a partial report.
Child reservations are bounded before auxiliary retention.
JSON parsing/nesting validation remains subject to serde_json's own limits.
UTF8-safe preview and pointer/difference caps bound output independently of input
values; compact JSON output avoids indentation amplification. Parsing uses
borrowed RawValue number tokens, with no floating-point conversion.

CLI: `syson-exact-json-diff LEFT.json RIGHT.json`. Both paths must be bounded
regular nonsymlink files. Exit 0 means MATCH, 1 means MISMATCH (including capped
reporting), and 2 means refusal/usage/I/O error. Invalid inputs refuse before JSON
report output. There is no network, subprocess or fixture-projection operation.

The actual C01f diagnostic mappings fixture is retained byte-identically for
self-comparison and a real handler-identity mutation. Such a MATCH proves only
comparison of supplied bytes under this declared policy, with no authenticity,
source/build binding, runtime completeness or native/Rust feature acceptance.
Coordinator comparisons use explicitly prepared full-payload files and separate
provenance; legitimate order differences must remain MISMATCH observations.
Parent C02/C01 and the broader differential-harness acceptance remain OPEN.
