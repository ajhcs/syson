# F18: package-only nested coordinates and dimensions

`syson_layout::containers::layout_packages` lays out an immutable package forest
bottom-up and then computes both relative and absolute positions. It preserves
the existing flat-layout API. Fresh sibling groups use the proposed F16 layout
algorithm. Edges between different parent groups are rejected; no implicit
ancestor projection or cross-container routing is performed.

## Source and proposed behavior

The [source manifest](source-manifest.json) pins the SysON package parent handler
at `6ba2ac64e6b16ee2b05752ad55395eebb87d4910` and two Sirius helper files at
`3191e2b8990373afee3415f6b26f8c5039a28321`. Exact Git blobs, whole-file hashes,
reviewed line ranges and original excerpts are retained. The Sirius candidate
source is not proof of the deployed npm artifact's contents.

The finite sizing/clamping contract is source-derived:

- The package body reserves eight units of padding, the supplied border width,
  and a top-header footprint. The footprint is an already observed number; the
  Sirius helper includes a matching header label's height and, when enabled,
  separator border. This function does not measure or authenticate that value.
- Fresh F16 sibling coordinates receive `8 + border` horizontally and
  `8 + border + headerFootprint` vertically. This placement strategy is proposed
  Rust behavior; it does not port source `getChildNodePosition`, newly-added-node
  overlays, or all layout-engine positioning rules.
- Retained child positions use the source previous-position branch: a negative
  x becomes eight; y is at least `8 + headerFootprint`, without adding border.
  Root retained positions clamp negative axes to zero as a proposed Rust policy.
  Retained positions may overlap; the function does not silently reflow them.
- Parent content bounds include each child's final position and dimensions.
  Computed minima add trailing eight-unit padding and twice the border width.
  Automatic parents retain the source default/nullish 150-by-70 behavior and
  default-before-border contribution. Explicit zero stays distinct from missing.
- For a user-resized parent, `user_minimum` is a precomputed previous-size
  observation. It keeps larger previous dimensions while still growing to fit
  children. That branch ignores default dimensions, matching the source branch.
  `user_minimum` applies only to parents; leaf dimensions are already measured.

`leaf_size` comes from the caller, for example from F15 `measure_node`. The real
F15 synthetic-host integration test measures a package leaf before nesting it;
it does not duplicate F15's leaf-sizing algorithm. Removing a parent's last child
uses the supplied leaf dimensions. No DOM or font engine executes in this crate.
The root-owned dependency is test-only; production layout has no new dependency.

## Inputs, coordinates and limits

Nodes have explicit representation IDs and an optional parent ID. The output is
canonical by ID, with relative coordinates within the parent and absolute
coordinates obtained from the ancestor chain. IDs are not semantic identities.
Unknown parents, cycles, duplicate IDs/edges, unknown endpoints, cross-parent
edges, invalid dimensions, arithmetic/precision loss, limits and cancellation
return typed errors without a partial layout. Parent walks are iterative.

Dimensions, defaults, header footprints and borders must be finite/nonnegative.
Retained x/y may be negative and are clamped as above, but must be finite.
All supplied geometry is validated even when a later branch would not read it;
this is an explicit Rust input restriction. Leaves keep supplied measured size.
Every sibling group's F16 work is charged to the same overall budget. The memory
value is a conservative logical admission estimate, not allocator overhead or
an operating-system RSS guarantee. Cancellation is polled between operations;
these guards do not establish a wall-clock deadline.

Identical inputs produce identical results. Passing a prior output's relative
coordinates back as retained positions preserves geometry when measurements
stay unchanged; enlarged children expand parent bounds. This is a finite
incremental geometry contract, not a native cache/freshness or persistence API.

Border ports, outside labels/margins, non-package containers, aspect-ratio
behavior, forced dimensions, newly-created-position overlays, disconnected
resource discovery, native rendering and general hierarchical layout parity
remain unsupported. No upstream application, JavaScript, browser or ELK runtime
comparison occurred.

## Validation and stacking

The coordinator's first compiler capture passed **30 Rust test functions**:
15 new container tests and 15 existing flat tests. One new test checks 24 forest
scenarios in both directions. Other tests cover exact nested coordinates,
source clamping/default/user minima, parent-to-leaf changes, repeated incremental
geometry, retained overlap, sibling edges, permutations, malformed data, numeric
failures, all logical limits, exact work boundaries, early/late cancellation and
actual F15 measurement integration. Formatting and all-target Clippy also pass.
[VALIDATION.json](VALIDATION.json) links exact inputs, original streams and the
executed runner. The two-line scoped transcript attribute preserves original
terminal blank lines; other files retain normal whitespace checks.

The 368 production lines and 553 test lines exceed the 900 authored Rust-line
review target by 21; the coordinator accepted that finite exception to retain
focused error and integration regressions. Generated source captures and raw
compiler receipts remain separate from authored implementation logic.

This PR targets the frozen F15/F16 dependency snapshot
`codex/syson-base-f18-20261025` at `6d67c932a7348aeba7f04cf66b850feeae7cad80`.
Both [F15 #27](https://github.com/ajhcs/syson/pull/27) and
[F16 #29](https://github.com/ajhcs/syson/pull/29) must land before retargeting to
the merged integration base and checking the resulting diff. Never merge this
PR into the dependency snapshot branch. No merge is included or implied.

Reproduce source checks from the worktree with the pinned Sirius Git objects:

```sh
python3 -B porting/follow-ons/F18/check_sources.py --sirius-source /path/to/pinned/sirius
```

The exact compiler commands are retained in the run receipt. Local checks do not
establish full SysON parity, deployed artifact correspondence or user acceptance.
