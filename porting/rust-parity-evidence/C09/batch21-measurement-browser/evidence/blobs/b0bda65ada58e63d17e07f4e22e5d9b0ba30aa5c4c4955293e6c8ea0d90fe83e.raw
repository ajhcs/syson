# Developer DOM and Rust WASM measurement probe

This module consumes the separate numeric/build module's passing capture receipt,
WASM and shared `wasm_client.js` transport. It runs a synthetic developer fixture;
it does not load the complete SysON or Rust application.

Choose explicit installed platform paths and fresh `/workspace` output:

```sh
python3 "$BROWSER_TOOLS/run_browser.py" --build "$BUILD" \
  --methods "$METHODS/original-methods.js" --transport "$NUMERIC_TOOLS/wasm_client.js" \
  --node "$NODE" --browser "$CHROMIUM" --playwright "$PLAYWRIGHT_PACKAGE" \
  --libraries "$SHARED_LIBRARIES" --out "$FRESH_BROWSER_OUTPUT"
```

`NUMERIC_TOOLS` points at the reviewed numeric module used by the build. The
transport defaults to the sibling `batch21-measurement-numeric/wasm_client.js`
after both PRs land in one checkout. Before that composition, pass its absolute
path explicitly. The runner requires its exact source path/hash in the build
receipt; copying an unrelated transport or fabricating a receipt cannot qualify it.

Build, original-method and source bindings must still match. The runner hashes
the served transport, HTML, fixture, original methods and WASM, along with source,
SDK and installed libraries before/after execution. It takes the shared runtime
lock, serves owned loopback files, limits the child lifetime and closes its owned
browser/server. Output paths are never reused.

Full mode performs actual pointer and keyboard interactions at 320/1440 px,
two warmups/seven measured repetitions per viewport, stale and missing labels,
previous-size retention, injected measurement failure, preserved controls/text/
focus and successful retry. It records screenshots, accessibility state, actual
fonts, console/page errors and separate DOM/source/JSON-WASM timing phases.

For the original busy-state counterexample, use the new runner and qualified new
build with `--assets "$RETAINED_OLD_ASSETS" --recovery-only`; retain that failure.
Repeat with current assets and a different fresh output. The one-shot exception
occurs at the actual WASM transport call, not at an invented readiness marker.

The earlier baseline capture has 18 samples, 14 checks and no console/page errors.
The split runner and build/transport handoff need their own fresh capture before
landing. Earlier observations do not automatically accept this new composition.

Synthetic Note styling has no padding, and both overflow axes are reported against
rendered boxes. Missing CJK/emoji glyphs, remaining overflow, complete geometry,
broader browser/device and spoken assistive-technology tasks remain open. Numeric
agreement and a readable developer fixture cannot establish full application UX,
native artifact correspondence, event latency or supported product capacity.
