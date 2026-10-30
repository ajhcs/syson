# F17: finite orthogonal routing for flat integer layouts

`syson-routing` computes real obstacle-dependent polylines from borrowed F16
placements and explicit side-center endpoint requests. It is a proposed Rust
routing algorithm, not an ELK translation or an upstream routing-parity claim.
The inherited F16 source mapping records the candidate Sirius Manhattan/ELK
boundary; it supplies context, not a routing oracle or deployed-artifact proof.

## Admitted family

Each node has a unique nonempty ID, integer x/y/width/height, positive dimensions,
and every rectangle edge within ±1,000,000 input units. A dimension may reach
2,000,000 when spanning both bounds. Fractional/nonfinite/degenerate/out-of-range
geometry and intersecting open rectangle interiors produce typed Unsupported.
F16 can produce fractional positions; those are intentionally not admitted here.

Requested Left/Right/Top/Bottom sides constrain anchor **position**, not departure
or arrival heading. The anchor is the exact side center. Returned coordinates
are doubled integers, retaining half-unit centers for odd widths/heights without
floating comparison tolerances. Nodes touching along boundaries or corners are
allowed: clearance is zero, and routes may run on node boundaries, including the
endpoint node. Distinct-side self edges are supported. Coincident anchors are
Unsupported rather than a fabricated zero-length self loop. Parallel edge IDs
remain distinct; their independent routes may have identical geometry. There is
no lane separation, edge-crossing, label/decorator or port-placement guarantee.

The grid contains rectangle borders, anchor coordinates and an exterior half-unit
band. Deterministic breadth-first search minimizes grid hops, not metric length
or bend count. Every visited candidate segment is checked against every open
rectangle interior, and collinear vertices are compressed. Node/edge ID sorting
makes results independent of input permutations. Inputs are never modified;
errors and cancellation return no partial route collection.

All known distinct admitted anchor pairs are connected under this zero-clearance
exterior-band family. `Unroutable` remains a defensive search-failure outcome;
no admitted negative example or formal completeness proof is claimed. Positive
clearance, constrained routing windows, nested containers and heading constraints
need separate families; a finite-window no-route fixture is not fabricated here.

## Bounds and verification

Node/edge/ID/grid/output/work limits are explicit. Coordinate products are checked;
grid size and work are charged before allocating bulk grid buffers. All explicit
vectors use fallible reservations. Work is a conservative logical charge, not
CPU time; allocation/sorting cannot be interrupted internally, and these limits
do not constitute a process RSS cap. Cancellation is checked between operations.

Sixteen test functions cover exact endpoints, odd-size/negative-coordinate
anchors, a real obstacle detour, self and parallel edges, corner/boundary contact,
permutation determinism, geometry/identity errors, empty/no-edge cases, all six
limit classes, exact work/grid boundaries, initial/late cancellation and actual
F16 layout output. One test enumerates 27 obstacle/self-side scenarios. A separate
geometric oracle checks every returned segment, open-interior exclusion, exact
endpoints, absence of zero segments and collinear compression. These are proposed
algorithm invariants, not upstream goldens. Root coordinator source review and
serial offline compiler execution are recorded in [VALIDATION.json](VALIDATION.json).

The cohesive production source is 403 formatted lines, three above the approximate
400-line target; guards and explicit outcomes remain readable. No existing crate,
shared manifest/lock, graph-adapter path or backlog is changed. F17 stacks on F16
PR #29 and should be retargeted after that prerequisite lands. No merge occurs.

```sh
python3 -B porting/follow-ons/F17/check_sources.py
cargo fmt --manifest-path rust/crates/syson-routing/Cargo.toml -- --check
cargo test --offline --locked --manifest-path rust/crates/syson-routing/Cargo.toml -- --test-threads=1
cargo clippy --offline --locked --manifest-path rust/crates/syson-routing/Cargo.toml --all-targets -- -D warnings
```
