# F20: source-frozen REST read contracts

This inventory pins the two by-ID and related-relationship read operations, the actual controller/handler/service/serializer chain, exact source hashes and upstream-test-source fixtures whose original Java/runtime execution remains unrun. `SOURCE.json` preserves full inspected Java texts and original headers; `CONTRACT.json` distinguishes concrete source decisions from unclosed runtime behavior. It is not an HTTP server or an authentication/API parity claim.

Controllers emit bare body values, not the internal success/error payload records. A missing project/context or non-success dispatched payload becomes controller404 with no body. A missing element is an error payload for by-ID but an empty successful relationship list for related-relationship reads. The commit UUID parameter is declared and bound but unused in these two bodies. The observed query-dispatch view-capability error is also a non-success payload here; no401 mapping is invented. Binding errors, context failures, twenty-second block timeouts/exceptions and outer response error bodies/status remain unqualified.

The custom SysML serializer differs materially from inherited EObjectRestSerializer: it includes all structural features, including transient ones, retains nulls and enum literals, omits the exact EAnnotations reference, filters virtual/library many-reference targets but does not filter single library targets. Source facts are read-only inputs; native getters, proxy loading, EMF caches and actual Jackson execution remain outside the pure adapter.

The Rust JSON adapter uses serde_json for validation and retains original JSON text, including unknown properties and numeric spelling. Structural JSON equality is separate from wire re-emission. No general byte/key-order/Jackson parity is claimed. The two copied JSON files are upstream test expectations, not newly executed upstream/runtime goldens.

## Local implementation checkpoint

The coordinator's fresh `f20-01` run passed all21 new native source-contract tests plus formatting and Clippy, with the executed crate bytes unchanged. The worker did not run Cargo. Results, exact hashes and original streams are retained in [VALIDATION.json](../../../follow-ons/F20/VALIDATION.json); this source contract does not promote an upstream fixture into an executed Java/runtime golden or establish complete HTTP/API acceptance.
