The Rust port has no typed immutable dispatch planner for the move-element
provider category. This adds a dependency-free `syson-extensions` crate that
preserves caller injection order and duplicate slots, selects the first observed
matching custom provider, and plans the default fallback when none matches or
when the selected custom provider returns null. Non-null failure statuses and
nullable messages are retained without trying another delegate.

Capability observations are immutable caller-supplied data. Missing observations
are errors only when reached; no provider code or model move executes. Six pinned
upstream Git blobs define the source-derived rules; Rust label and packet-shape
admission policies are explicitly separate.

Validation: 17 focused tests pass, including 511 exhaustive combinations in one
test, plus formatting, all-target Clippy with warnings denied, and source-pin
checks. Root source review and the nullable-message correction are recorded.
Original run streams, executed runner/input hashes, and the initial source-range
metadata failure are preserved. Java/Spring runtime comparison, native capability
adapters and whole-area extension parity remain unverified.

F38 has no outstanding future-unit prerequisites. Base is `rust/integration` at
`ad814c232ee24109639c1d52239cbeec6f4a3e92`. No shared module/manifest/backlog
wiring, package update or merge is included.
