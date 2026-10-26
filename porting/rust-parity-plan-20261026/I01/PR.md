# I01: Open native local identity wires in the selected editor

Native JSON uses local node UUIDs for `FeatureTyping.type` and `typedFeature`; the selected editor and F14 importer use semantic element UUIDs. Add an explicit `native_model::load(..., LocalJsonNodeIds)` bridge for the same six package/part classes as PR #63. Both UUID domains, containment order and opaque JSON values survive. Conversion publishes a complete owned document after native reference and selected ownership-shape validation; wrong-domain, unsupported and dangling inputs refuse without mutation.

The integration journey opens native-form JSON, renames a usage through the actual Editor, imports a new package through F14, saves/reloads through F33, and reopens with unchanged identities and correct old/new typing. Same-class identity collisions ensure lookup uses only the declared node domain.

Validation: coordinator `i01-03` passed all 36 integration tests (10 new and 26 inherited), formatting and scoped Clippy. Original `i01-01` mutation defect and `i01-02` lint failure, executed runners, main-crate sources and streams remain retained externally and hash-addressed by `verification.json`. Dependency inputs are hash-recorded and retained in the immutable base commit.

This is finite reference conversion. Native SysON runtime comparison, large-model performance, other classifiers, cross-resource references, resource loading and complete serialization remain pending C01/C02 and follow-on units. No production Java delegate, dependency or held factory/initializer change is introduced.
