Namespace imports need a frozen behavior contract before the F03 evaluator can
be implemented. This change pins six native metamodel source blobs and adds
19 ordered synthetic cases covering import/exclusion order, conflict state,
alias reference identity, cycles, visibility and unsupported outcomes. It retains
the generated membershipsOfVisibility null result explicitly.

A stdlib-only checker validates source/fixture integrity, local filter transition
proofs and trace consistency. Graph outputs remain manually source-derived;
no import evaluator or upstream runtime ran. The similarly named services helper
is outside this native API dispatch path.

Validation: final captured checks pass six source pins, 19 fixture cases and
17 checker tests, including mutation rejection. Original 18-case capture and all
raw streams are preserved. Root source review accepted the final scope. No Rust,
manifest, backlog, existing lease, package or service change is included.
