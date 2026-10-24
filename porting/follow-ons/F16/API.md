# F16 API and algorithm freeze

This is a proposed Rust layout algorithm, not an ELK translation. The API was
reported to the coordinator before implementation; the coordinator approved
canonical identity ordering, SCC condensation, directed longest-path ranks,
weak-component bands, explicit unsupported constraints and bounded execution.

`layout(nodes, edges, options, limits, cancelled)` borrows measured rectangles
and directed edges. IDs are nonempty and unique within their node/edge namespace;
unknown endpoints are errors. Right and Down are the only admitted directions.
Every node must be flat, unpinned and have zero ports; unsupported flags return
errors. The caller supplies already-visible, already-measured rectangles.

The result preserves every node's measured width/height and every original edge.
It returns canonical-ID-ordered placements, strong/weak component IDs, ranks,
internal-SCC edge classification, total bounds and charged work/storage values.
Component IDs are temporary canonical indexes, not persisted semantic IDs.

The algorithm is:

1. Canonicalize nodes and edges by ID, then build forward/reverse adjacency.
2. Compute SCCs with two iterative Kosaraju passes, avoiding recursive DFS stack
   growth. Renumber components by their smallest canonical member ID.
3. Condense and deduplicate inter-SCC arcs, then compute longest-path ranks with
   a deterministic minimum-index topological queue. Internal edges remain in the
   returned edge list, including self loops and parallel edges.
4. Identify weak components through undirected traversal. Each occupies a
   separate cross-axis band, ordered by its smallest node ID.
5. Use the largest measured primary-axis extent in each occupied layer to place
   subsequent layers. Place each layer's nodes in canonical ID order on the
   cross axis. Add gaps only between actual layers/nodes/components, not after
   the last element. Transpose primary/cross axes for Down.

All inter-SCC edges advance in rank. Their target begins after the source layer's
measured extent plus layer gap; with zero-size nodes and zero gap, equal physical
coordinates remain possible while ranks still advance. Same-layer nodes and
weak-component bands are disjoint by construction. There is no claim of minimum
crossings, minimum area or aesthetic optimality.

Failures return no partial result and do not mutate inputs. Dimensions, spacing
and padding must be finite/nonnegative. Addition overflow and an unrepresentable
positive coordinate advance return explicit errors rather than overlapping or
nonfinite successful placements. Zero dimensions and zero gaps are admitted.

## Resource policy

Limits separately bound node count, edge count, ID bytes, charged logical work
and estimated workspace. The conservative buffer admission formula is
`1024*N + 640*E` bytes; it covers the algorithm's typed Vec/heap capacities with
slack. It excludes caller-owned input/string storage, allocator bookkeeping,
stack/runtime machinery and RSS. It is not an OS memory cap or resource sandbox.
Fallible Vec/heap reservation reports allocation failure where available.

Work is a deterministic logical charge, not CPU instructions or elapsed time.
Buffer initialization is precharged; explicit graph loops and heap operations
charge/poll cooperatively. Standard sorts precharge `length² * maximum-key-bytes`
(or one for integer keys) and poll before/after. Standard sort, allocation and
string-comparison internals are not individually interruptible or exact
instruction-accounted. Bounds are admission controls, not real-time deadlines.

## Proposed-algorithm differences

The pinned useElkLayout boundary accepts arbitrary ELK options and asynchronously
calls elk.layout. It converts labels/children/ports, filters unknown or
parent-child edges, preserves pinned positions, handles connection anchors and
updates Manhattan bends. This crate implements none of those operations. It
rejects unknown endpoints/pinning/containers/ports, preserves all admitted edges,
and returns rectangles plus edge classifications without routes. F17/F18/F19
remain routing, nested-container and UI integration work.
