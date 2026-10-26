# Readable canvas viewport audit fix

The preview previously scaled the whole scene into the current column, making
labels difficult to read on narrow screens. Actual size now keeps admitted SVG
units at 1:1 (14px names, 12px classes). Fit overview is a separate, explicit
choice. A labelled focusable viewport contains native scrolling, with 44px view
controls and layout/history controls outside it.

Presentation toggles return before WASM dispatch. They retain the same DOM
controls, drafts, caret and disclosures and leave model/geometry/history alone.
View mode and earlier Actual-size pan survive repaints while Fit is active.
Background caret restoration follows the existing focused-input contract;
restoring arbitrary blurred selections is not promised. State is local to this
page; no persisted pan/zoom model or solver was introduced.

Coordinator checks pass 44 inherited Rust integration tests, formatting, scoped
Clippy and release WASM. The dedicated 32-node browser journey passes at 320px
and 1440px, measuring CTM 1, 14px names and 12px classes. It covers Actual/Fit/
Actual, native keyboard scroll, local overflow, pressed/focus states, zero toggle
WASM dispatches, drafts and exact model/geometry bytes. Actual/Fit screenshots
were inspected at mobile and desktop sizes.

Five existing journeys collectively pass on identical production bytes:
inspector and layout in composed-02; authoring, scalars and draft recovery in
composed-03. Composed-02 remains a failed run because its later legacy authoring
fixture failed. Composed-03 ran three scripts, not all five. Viewport-01 failed
because its fixture parsed an initially empty export; composed-01/02 exposed
legacy blind disclosure toggles after correctly preserved error states. The
state-aware fixture fixes change no assertions or production behavior. Every
original failure, script, runner and stream remains externally retained.

[VALIDATION.json](VALIDATION.json) is the compact capture disposition. Its local
32-node/zero-edge performance reference measured 30 Layout clicks: p50 48.7ms,
p95 49.0ms and maximum 49.1ms, with no observed Long Task entry. This is a local
DOM/two-frame opportunity proxy, not INP, LCP, physical presentation or native
SysON performance. Physical touch, screen-reader speech and other platforms
remain unqualified.

This follows recovery #61 and navigation #62 on their exact snapshot. Land the
prerequisites, retarget to integration, inspect and revalidate; never merge into
the snapshot or prerequisite feature branches. Historical evidence is unchanged.
