The Rust port can lay out flat nodes but has no orthogonal router. This adds a
finite routing crate over F16 placements, using exact half-unit side-center
anchors and deterministic obstacle-avoiding grid search. Distinct-side self
edges and parallel edge identities are retained; parallel paths may coincide.

The admitted family is positive integer rectangles with bounded edges, no
interior overlap and zero clearance. Boundary/corner travel is allowed; sides
constrain anchor positions without heading constraints. Unsupported geometry,
coincident anchors, identity errors, work/grid/output budgets and cancellation
return typed failures without partial results. This is a proposed Rust algorithm,
not ELK behavior; no admitted Unroutable example or completeness proof is claimed.

Validation: 16 tests pass, including 27 generated scenarios, an independent
segment-interior oracle and actual F16 layout integration; formatting, all-target
Clippy and three dependency source pins pass. Original receipts/streams, frozen
input hashes and root source review are preserved. No upstream runtime ran.

Stacked on F16 PR #29 (`codex/syson-f16`). Merge prerequisites first, then retarget
to integration and recheck; do not merge this child into the prerequisite branch.
No shared crate, held graph-adapter path, service or backlog is changed.
