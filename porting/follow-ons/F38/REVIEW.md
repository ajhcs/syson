# F38 review and validation record

Root reviewed the exact provider-category design and implementation independently
of this worker. Review retained first injected matching delegate, exact caller
order, duplicate slots and selected-null fallback to the default. Root found the
initial `MoveStatus.message: String` narrower than the source Java record, whose
message may be null. That was corrected to `Option<String>` and covered by a
focused null-message regression before compiler input freeze. Root then reported
no further source behavior findings and approved the scoped commit.

The coordinator generated the new dependency-free Cargo.lock offline and ran
`f38-01`: formatting, 17 tests and all-target Clippy with warnings denied pass.
The exhaustive truth table covers 511 combinations inside one of the 17 tests;
it is not 511 additional test functions. Original stdout/stderr, exact executed
runner and immutable before/after input hashes are retained in
`validation/f38-01`. All four captured compiler inputs still match those hashes.
No Rust edits occurred after the passing run.

Source verification pins six exact upstream blobs. An initial source-line-range
metadata check failed because three entries exceeded EOF. The first attempt was
not redirected; its output remains in the tool transcript. An unchanged
diagnostic rerun, its failed manifest and exact streams are retained in
`validation/source-check-01`. Corrected metadata passes in `source-check-02`;
the source object IDs and SHA-256 hashes did not change. This was not an upstream
or Rust runtime failure. No actual failure is omitted or relabeled as a pass.

The result remains a standalone source-derived dispatch planner. Caller-side
native capability observation, freshness, Spring injection order, Java runtime
comparison, concrete mutation, and all other extension categories are unrun or
unsupported as documented in README.md. Rust label/packet admission rules are
proposed policy, not claims about Java registration validation.

The full staged whitespace check identified the preserved Cargo test stdout's
original terminal blank line. An exact-file `.gitattributes` rule preserves that
stream byte-for-byte; no production or general whitespace checks are disabled.
