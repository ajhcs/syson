# F16: deterministic flat directed layout

`syson-layout` computes actual graph-dependent positions for measured rectangles.
It canonicalizes identities, finds strong components iteratively, ranks the
condensation DAG by longest path, and places measured layers inside separated
weak-component bands. Cycles, self loops and parallel edges remain explicit in
returned classifications. Direction Right/Down, layer/node/component spacing
and padding are typed options. See [API.md](API.md) for the frozen contract,
algorithm, numeric restrictions and resource accounting.

This is a source-independent proposed Rust layout algorithm. It is not an ELK
port, grid/identity-layout substitute or claim of equivalent positions. A changed
edge direction changes ranks and positions. The pinned original useElkLayout
transport/options/filter boundary is retained as source evidence, not executed.

Unsupported containers, ports and pinned nodes are errors. Unknown endpoints,
duplicate/empty IDs, malformed numbers, cancellation, budgets, arithmetic overflow
and lost positive-coordinate precision have explicit outcomes. Inputs are
borrowed and never changed; failures expose no partial layout. Standard sorting
and allocations are not individually interruptible. Logical work and estimated
buffer admission do not claim a real-time deadline, exact allocator usage or RSS
cap. Source/artifact correspondence and upstream layout parity remain open.

## Verification scope

Fifteen coordinator-executed native test functions cover connected chains/diamonds, unequal
measured layer sizes, cycles, self/parallel edges, disconnected gutters, both
directions, canonical permutation invariance, unsupported inputs, all budget
classes, cancellation, zero-size geometry, overflow/precision and unused-gap
arithmetic. One generated property test enumerates all 512 directed graphs on
three unequal-sized nodes, including self edges, in both directions: **1024
scenarios, one test function**. It checks independent reachability-based SCC
membership, longest-path rank recurrence, inter-SCC rank/position advancement,
rectangle non-overlap, total bounds and preservation of every original edge.
Those are proposed-algorithm invariants, not upstream runtime goldens.

The cohesive API/algorithm exceeds the approximate 400 production-line target;
the coordinator accepted the routine size exception rather than removing
required behavior or obscuring formatting. Source and tests remain separate and
readable. No new dependency, registry package or existing crate is changed.

The implementation worker ran only static source/hash checks and did not run
Cargo. The coordinator executed two separate captures. The original `f16-01`
capture passed formatting and all 15 tests (including the 1024 scenarios), then
failed only four test-loop clippy lints. Iterator/enumerate corrections changed
only test source. The fresh `f16-02` capture passed formatting, all 15 tests and
clippy with warnings denied, with unchanged final source hashes. Production
algorithm bytes were unchanged across both captures.

[VALIDATION.json](VALIDATION.json) links both original receipts and streams;
[REVIEW.md](REVIEW.md) records coordinator source review. The coordinator is
distinct from the implementation worker and also executed the checks; a second
runtime executor is not claimed. Browser and upstream runtime checks were not
run, and original-layout parity remains unverified.

```sh
python3 -B porting/follow-ons/F16/check_sources.py --sirius-source /path/to/pinned/sirius
```

F17 routing, F18 nested containers, F19 browser integration, crossing optimization,
label/port/anchor handling and full geometry-area acceptance remain incomplete.
