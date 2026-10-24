# Coordinator source and receipt review

Reviewed capture/profile/source/lock checks, normal-edge graph traversal, actual packaged notice/license/source/archive inclusion, unknown dependencies/payloads, symlink refusal and explicit synthetic-artifact/build-provenance distinctions. Policies and captured metadata remain reviewed trust inputs, not authenticated source truth. The verifier does not infer active dependency closure from Cargo.lock alone. No blocking issue remains within this admitted supplied-package contract.

All19 tests passed in the final freeze; the earlier successful capture is retained for its earlier generator bytes. The final policy generator also ran under optimized Python, reproduced six pinned files exactly and rejected a wrong source revision before output. The final result is in VALIDATION.json; the frozen handoff README retains its historical first-run/pending-final wording.

The coordinator is distinct from the implementation worker and also executed the captures. Source-to-WASM build correspondence, installed behavior, upstream artifact provenance and full legal clearance remain unproven. No build, registry download, archive extraction, dependency install, upstream application or production change occurred.

## Cross-PR fixture isolation

A synthetic9PR source composition reproduced the old fixture coupling: the production verifier correctly rejected the changed domain source against the fixed baseline policy, so old tests using the live checkout failed during setup. The original streams are preserved; the reproduction runner stopped before serializing its initial UTC variables, so no execution UTC is retroactively inferred.

Tests now materialize policy-pinned Git blobs into their own workspace temporary directory, checking each source hash. The corrected19 tests pass in the composed source, and the combined F44/F45 domain passed62 tests and clippy. Production verifier, frozen policy/metadata and stale-input rejection remain unchanged. This qualifies the exercised combinations, not all9 application features or upstream runtime parity.
