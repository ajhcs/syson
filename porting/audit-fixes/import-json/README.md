# Explicit selected-model JSON reference bridge

F14's selected model stores local FeatureTyping references as semantic element
IDs. Native F33 loading interprets bare UUIDs as JSON node identities. Those
contracts need an explicit conversion: a caller that mislabels selected data as
native can get an unresolved target, or a wrong same-class target when identity
domains collide. Native Bundle::load remains unchanged.

Call selected_model::load with SourceEncoding::LocalSemanticElementIds. The tag
is the caller's representation assertion, not provenance attestation or automatic
encoding detection. The bridge admits six package/part/membership/typing classes,
one typing per usage and exact selected ownership shapes. It resolves local
semantic IDs, checks the containing usage and PartDefinition target, and rewrites
only type/typedFeature on a private clone before normal native bundle admission.
Node IDs, semantic IDs, envelope fields and opaque unknown values stay intact.
Unknown reference families, external targets, malformed IDs, unsupported shapes,
missing typing or late failures publish no partial bundle or caller mutation.

Coordinator capture bridge-01 passed 25 integration tests, including the 14
inherited native-bundle tests. The additional nested FeatureMembership and
cross-package fixture then passed in bridge-02: 26 integration tests, formatting
and scoped all-target Clippy, serially offline/locked. Both passing attempts keep
their exact original source, runner, receipt and streams externally; no prior
attempt is replaced. All 114 final capture input hashes match committed bytes.

[VALIDATION.json](VALIDATION.json) records the compact disposition and canonical
external artifact hashes/paths. No new registry input was added. The F14 local
dependency is test-only. The historical published F33 evidence is unchanged.

The bridge is one resource, local references and one direction only. It is not a
full native serializer, a SysML validity verdict, foreign library loading or a
native-to-selected bridge for textual export. Native EMF runtime parity remains
unverified. This PR follows F33 #55 and F14 #52 on their exact frozen snapshot;
land prerequisites, retarget to integration, inspect and revalidate. Never merge
into the snapshot or a prerequisite feature branch.
