# Preallocated containment and same-transaction initializer candidate

Source and 31 owner tests prepared NOT_RUN: graph/naming18 + staged bridge5 + model containment8. No new Cargo allocation, GUI, worker, upstream or application run.

API: Transaction::attach_preallocated_subtree(parent semantic ID, ContainmentFeature, subtree Value) clones the staged document through existing Transaction.apply, checks canonical known concrete classifier/UUID spelling and typed containment, appends in order, validates global semantic/resource ID uniqueness, and poisons failure. Existing cancel/commit identity/stale/undo/redo machinery is retained.

The bridge initialize_staged takes ownership of that SAME already attached Transaction, builds NativeContext from Transaction.document, plans supported scalar defaults and returns the same transaction. Caller commits once. apply_staged_plan supports caller-supplied declarative steps without certifying plan origin. Errors/cancellation drop the owned transaction. Native queries therefore include the newly attached root before exact-class counting/naming. No intermediate live-editor creation or extra history entry.

Preallocated input is synthetic or separately supplied. This hook does not call or accept the held F1 factory, assign fresh IDs, project RawObject cached state to JSON, establish UUID entropy/version provenance, execute EObject inverse notifications/proxies, or validate every Ecore constraint/reference/serializer behavior. It implements the two known raw containment features. NodeIds admission checks spelling only, as documented in the existing model API.

The end-to-end fixture is actual model attach -> staged graph query/plan -> defaults -> one editor commit -> undo/redo. Negative fixtures cover duplicate semantic/resource/nested IDs, invalid parent/child/nested containment, malformed/unknown/abstract node admission, cancellation, stale transactions, late scalar failure preserving redo, late reference refusal, and an unsupported allocation requirement refusing partial creation.

The previous naming 33-file NOT_RUN freeze is preserved in its own source commit before dependency edits; its source commit is recorded in this candidate's provenance. Original graph nine-test freeze remains unchanged with source at bad9b9ea9af590e4ebdb488a47058d3d87b56ec2. No previous test result is reassigned to this changed closure. Full upstream headers, including distinct years/contributors, are in graph NOTICE and the translated containment module.

Qualification update: repaired combined39-file candidate passed31 owner tests in attempt02 at2026-10-23T05:56:58.351476..05:56:59.921516Z. Earlier preparation NOT_RUN text remains historical. See QUALIFICATION.json; this does not relabel the standalone naming33-file freeze. Remote publication is stopped under the current parent restriction.
