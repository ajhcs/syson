# Source, numeric and Rust build qualification

This development module compares selected pinned Package, ViewFrame and Note
methods with the actual Rust measurement API. It contains no shipped application
behavior. The browser fixture is a separate PR/module that consumes this module's
qualified build receipt and shared transport.

Set explicit absolute paths for `TOOLS`, the current repository `REPO`, pinned
source checkout `SYSON`, pinned Sirius files `SIRIUS`, source mapping `MAPPING`,
and the installed Node executable `NODE`. All working outputs must be fresh paths
on `/workspace`. Existing Rustup, native/WASM toolchains and offline Cargo cache
are prerequisites; these commands do not install tools or fetch packages.

```sh
python3 "$TOOLS/prepare_originals.py" --mapping "$MAPPING" --syson "$SYSON" \
  --sirius "$SIRIUS" --node "$NODE" --out "$METHODS"
python3 "$TOOLS/measurement_cases.py" "$CORPUS"
python3 "$TOOLS/build_capture.py" --repo "$REPO" --corpus "$CORPUS" \
  --out "$BUILD" --target-dir "$CARGO_TARGET" --cargo-home "$INSTALLED_CARGO_CACHE"
"$NODE" "$TOOLS/measurement_original.cjs" \
  "$METHODS/original-methods.js" "$CORPUS" "$ORIGINAL_OUT"
"$NODE" "$TOOLS/replay_wasm.cjs" "$BUILD/measurement_bridge.wasm" \
  "$TOOLS/wasm_client.js" "$CORPUS" "$WASM_OUT"
python3 "$TOOLS/compare_measurement.py" --corpus "$CORPUS" \
  --native "$ORIGINAL_OUT" --rust "$BUILD/rust.json" --out "$NATIVE_REPORT" --self-test
python3 "$TOOLS/compare_measurement.py" --corpus "$CORPUS" \
  --native "$ORIGINAL_OUT" --rust "$WASM_OUT" --out "$WASM_REPORT" --self-test
```

The coordinator executes the build producer. It creates a standalone consumer
manifest and lock only inside the fresh capture directory. Cargo runs offline,
locked after lock generation, with one job and explicit native/WASM targets. The
shared runtime lock serializes compilation and native execution. Every command
retains arguments, output, error, exit status, lifetime and owned process identity.
Timed-out children are terminated as an owned process group and reaped.

The receipt binds compiler/Cargo versions and executables, module and actual
measurement/editor sources, resolved package sources, generated manifest/lock,
Cargo configuration, corpus and copied native/WASM artifacts. Preparation drift,
failed commands and source drift produce a terminal failure. A successful build
status establishes capture only; the independent comparison must still pass.

Earlier coordinator captures retain 30 exact size comparisons, five typed Rust
refusals and seven divergence controls; seven old malformed-envelope defaults
are repaired with valid null-label behavior preserved. Source/excerpt/build guards
also have retained normal/optimized-Python counterexamples. Preserve those original
versions and qualify this relocated producer through fresh executions before
landing; a source copy does not transfer an earlier result to a new composition.

Float values use exact lowercase hexadecimal bits, including signed zero. Missing
labels use explicit `bounds: null`; malformed envelopes return no size or host
calls. The caller owns input memory; returned output owns a separate allocation.
The transport reacquires memory after calls and frees each buffer once. Unsafe
exports require valid live pointer/length pairs; the 1-MiB developer input quota
does not establish an application memory bound.

The original-method extraction records full source and excerpt hashes and uses
Node's built-in TypeScript erasure with only export/class wrapper adaptation.
Supplied numeric facts do not prove actual DOM/font behavior, deployed SysON
artifact correspondence, complete geometry consumers, application usability or
native speed. Those acceptance gates remain separate.

`--cargo-home` must name the existing installed workspace cache. The producer sets it only for owned child processes; it neither installs packages nor falls back to the root disk. Rustup invocation names are preserved while both the selected wrapper and actual tool binaries are hashed.
