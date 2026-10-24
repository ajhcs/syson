# F15: typed label measurement and three node geometry contracts

`syson-node-measurement` accepts caller-provided DOM/text/font observations and
computes the source-derived package leaf, view-frame leaf and note dimensions.
It reuses `syson_editor::layout_package_leaf`; there is no duplicate leaf sizing
algorithm and no second semantic/domain model. Node IDs are representation IDs.

The host receives the exact label target, text, font specification and a caller
stamp containing layout, font and request revisions. Package/view-frame targets
retain the first visible-node index, including source findIndex's -1 when absent.
Note targets explicitly request the inside-label model bounds. Absent labels
remain absent observations; the adapter uses the source zero-height fallback.
Target or stamp mismatches are errors. Tokens detect accidental mismatches but
neither authenticate a host nor prove that its claimed bounds came from a DOM.

## Source contracts and helpers

The pinned SysON sources are the three named layout handlers. Package/view-frame
leaf minima retain 75×50, 8+label-height+8 padding, nullish defaults, and the
source's unusual force behavior: nonzero forces suppress previous-size retention
without becoming leaf dimensions; zero forces remain false for that decision.
The parent/container paths explicitly return UnsupportedParent and remain F18
work. Notes follow their distinct source contract, where non-resized forced
width/height really are dimensions; the note handler ignores direct children.

The admitted Sirius helper source is pinned at
`3191e2b8990373afee3415f6b26f8c5039a28321`. The crate ports:

- computePreviousSize's new/previous/current branches and per-axis nullish
  150×70 defaults. A previous object with missing dimensions uses defaults, not
  current-node dimensions. First matching ID wins.
- getDefaultOrMinHeight for notes, combining label/border minimum and default70.
- Numeric getNodeBorderWidth, including left-width precedence and nonfinite→0.
  It does not silently parse string CSS units or keywords.
- applyRatioOnNewNodeSizeValue for notes. Its default ratio uses truthy 150/70
  fallback, intentionally different from nullish previous-size defaults. It
  expands one dimension and leaves zero-sized dimensions unchanged.

Source correspondence is finite and source-derived. The candidate Sirius pin
is not proof that the deployed npm artifact contains those exact implementations.

## Numeric and host boundaries

The adapter admits finite nonnegative dimensions, border values and measured
bounds, with finite font metrics; negative letter spacing is allowed. Missing
values and explicit zero stay distinct. Nonfinite numeric border styles retain
the source helper's zero fallback. Other invalid values and computed overflow
return errors; all supplied dimensions are validated even when a later source
branch would not read them. These are explicit input-domain restrictions, not a
claim to preserve JavaScript NaN/Infinity/negative-size behavior.

Only the three exact renderer-type names are admitted. Unknown kinds, containers,
stale observations, wrong targets and host failures are explicit. Inputs are
borrowed/copied representation values; this function does not mutate committed
model data. There is no DOM binding, font engine, browser execution, HTML/SVG
renderer, CSS string parser, theme handling, border-port layout or accessibility
acceptance hidden in a successful result. A synthetic host fixture is not a real
DOM/font measurement or full rendered-node acceptance.

## Evidence

`source-mapping.json` records the three exact primary contracts, inherited helper
source hashes/excerpts and the reused local dependency file closure. Check them
without executing upstream code:

```sh
python3 -B porting/follow-ons/F15/check_sources.py --sirius-source /path/to/pinned/sirius
```

Seventeen coordinator-executed tests cover both leaf handlers, all previous-size branches,
first-match IDs, label targets, font/request revision changes, nullish/zero force
differences, note border/default/aspect-ratio behavior, malformed numbers and
arithmetic overflow. The coordinator's `f15-01` focused capture passed formatting,
all 17 native tests, and clippy with warnings denied. The tested new crate and
reused syson-editor source hashes remained unchanged. The implementation worker
did not execute Cargo. Static source checks passed.

[VALIDATION.json](VALIDATION.json) links the original receipt and streams;
[REVIEW.md](REVIEW.md) records the separate coordinator source review. The
coordinator is distinct from the implementation worker and also executed the
checks; a second runtime executor is not claimed. Browser/DOM measurement and
upstream runtime checks were not run.

Upstream runtime comparison and full geometry-area parity remain unverified.
Historical ledgers, held leases and existing crates/manifests remain unchanged.
