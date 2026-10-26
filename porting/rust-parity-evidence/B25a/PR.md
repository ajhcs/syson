# B25a: prepare Annotation reference migrations

Old SysON resources serialize `Annotation.annotatingElement` and its `AnnotatingElement.annotation` opposite, which became derived before 2025.4.0. Add the exact participant version 2025.4.0-202503140000 and ordered removals using B25S. Matching preserves the native first namespace entry and object-child/subtype traversal behavior. Preparation is immutable; callers use `Prepared.to_json()` to retain namespace ordering before replay or document admission.

The actual Document→B01 regression admits the migrated resource and prepares it without resource loading, preserving identities and note text. Nineteen identical input cases match Java source-compiled native observations for output values, version, resource predicate and success/refusal. All 12 Rust tests (3 new + 9 shared), formatting and all-target Clippy pass at 49 unchanged input hashes. Exact unit runner/receipt/streams and19 native/Rust outputs are retained; source predictions remain labeled separately.

B25S supplies the shared 728 subtype and 4 version-ordered composition comparisons, identified dependency JARs and bounded memory fix. This evidence qualifies selected participant preparation. Whole release source/artifact closure, global migration history, framework activation, native deserialization, durable publication, SQL and GUI remain open. Native exception text is retained without claiming equivalent Rust error text.

Depends on `codex/syson-rp-b25-support` at cbf8a9f7a56f459a2374962e96010233fe711801. Land prerequisites through review, retarget this child and revalidate affected integration. No held factory/model/initializer paths change.
