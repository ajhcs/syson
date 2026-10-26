# Reference-binding policy core (C01a)

`admit(request, &UnverifiedFacts)` is a pure policy validator. A successful result expressly does not mean files or Git objects were inspected, receipts are authentic, the native application is source-bound, or parent C01 is closed. Actual filesystem/Git inspection and CLI consumption belong to ordered child C01b.

The request schema and retained input example are in [selected-request.json](examples/selected-request.json). That historical example intentionally refuses: its original review harness hash differs from the later compiled oracle. Preserve it as a diagnostic example. Actual inspection must supply a new scoped final review receipt with the final harness and exact source/tool/dependency/configuration bindings, while keeping the historical review unchanged.

Tests retain that historical mismatch as a refusal and label coherent positive facts explicitly synthetic. Additional negatives cover stale/invalid pins, missing/changed hashes, source/dependency/class partial proofs, claim escalation, inconsistent revision pairs/deltas and missing/mismatched selected build/runtime configuration. All nine tests, formatting and scoped Clippy passed in the root capture. [Evidence and limits](../../../porting/rust-parity-evidence/C01a/README.md) distinguish pure policy qualification from actual inspection and parent native parity.
