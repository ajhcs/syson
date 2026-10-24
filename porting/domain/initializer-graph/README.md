# Bounded native initializer graph candidate

Nine owner regressions actually PASS in attempt-01; independent/upstream acceptance remains pending. Pinned upstream baseline: 6ba2ac64e6b16ee2b05752ad55395eebb87d4910.

Uses existing public Document and Editor/Transaction APIs. The raw two-feature containment index retains resource IDs separately from semantic elementIds. ElementImpl owner/owningMembership/owningNamespace and NamespaceImpl ownedMember filtering are distinct; no flattened projection or nearest-ancestor substitution.

NativeContext replays allocation-free scalar prefixes on a private document clone. Known scalar plans bind a document snapshot and commit through one existing Transaction. Cancellation, stale snapshots and unsupported/invalid late steps do not commit. Binding a caller-supplied plan does not certify its source; native planning and binding must use the same snapshot.

Supported naming is FeatureImpl direct declaredName/declaredShortName behavior and no-owned-Redefinition null fallback. Redefinition naming resolution, non-Feature effective-name dispatch, library/GeneralView lookup, allocation, containment edits, references/inverses, proxies, other containment features, notifications, full Java getter/setter dispatch and serialization parity remain open. No F1 factory candidate or validator path is changed. ApplicationStatus on the original declarative planner remains Unported; this separate bounded adapter does not relabel that frozen crate.

Native tests cover exact-count with broader Membership, immediate namespace, semantic/resource identity, naming prefix visibility, scalar success/undo/redo, atomic invalid/unsupported failures, cancellation/class mismatch, short-name suppression, explicit naming/library gaps, and stale snapshot refusal. No GUI/upstream/independent acceptance.

Executed 2026-10-23T05:17:14.254275+00:00 through 05:17:17.033925+00:00; exit0, no timeout or input changes. The frozen source-kind NOT_RUN label is retained as historical prelaunch provenance. This gate ran nine new graph tests, not the earlier 26 planner tests. Compiler allocation was released immediately after completion.
