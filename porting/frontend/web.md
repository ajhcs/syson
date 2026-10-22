# Bounded Rust/WASM authoring interface

`rust/crates/syson-web` provides a browser DOM interface whose model operations and HTML rendering are implemented in Rust. It depends on the shared `syson-model` crate for model creation, stable identity, type assignment, validation, EMF JSON import/export and staged history. It uses `syson-editor` for a local preview rotation. No separate model or hidden Java/TypeScript modeling runtime is introduced.

The UI exposes root/nested packages, part definitions, part usages and optional usage typing; staged creation/rename with commit/cancel; undo/redo; element selection; a deliberately simple geometry preview; committed JSON export by copying a labeled textarea; and importing EMF JSON as a new document with clearly labeled history reset. The preview is generated from actual model element summaries. It does not claim upstream graphical-layout or appearance parity.

## Source trace and compatibility limits

Model behavior and exact upstream service mappings belong to `porting/domain/MODEL-ADAPTER.md` and the syson-model crate. The frontend merely calls that public contract. Rotation derives from the pinned `frontend/syson-components/src/extensions/rotateNodeTool/useRotateNode.ts` as documented in the editor ledger. Export/import use the model's supported nested EMF JSON 1.0 envelope; these are not the SysML interchange REST representation, full project downloads, or textual SysML import/export.

The workflow and DOM layout are a new bounded Rust adaptation, not a port of the complete Sirius shell. Native labels, button semantics, keyboard tab access, focus outlines, status/alert messages and an accessible text list accompany the SVG. Those provisions need browser and assistive-technology verification before acceptance. The SVG deliberately does not show real relationships or containment layout, and local preview geometry has no shared-model persistence/history. Selecting a preview then rotating only changes its local dimensions; it is not a full upstream diagram command.

Unported: authentication, project/database persistence, collaboration/subscriptions, server GraphQL APIs, full model metamodel behavior, all other palette tools/forms/representations, automatic layout/routing, drag/drop, rich text, complete accessibility, upstream visual fidelity, remote synchronization and production deployment. Browser data is tab memory only; users must export before closing. Import replaces the in-memory document and clears its history, which the button text states explicitly. Export serializes only committed changes; staged previews are excluded.

## Explicit browser platform glue

`web/loader.js` is checked-in browser platform glue. It fetches and instantiates the compiled WASM module, marshals generic DOM FormData and event action values to UTF-8 JSON, obtains 96 random bytes through browser `crypto.getRandomValues`, and inserts HTML that Rust generated and escaped. It contains no model creation, mutation, name/ID validation, history logic, storage interpretation, graph layout or user-facing HTML template. Rust constructs UUID strings from those random bytes and delegates their validation to syson-model.

The serde_json=1.0.151 arbitrary_precision dependency and its pinned closure match the shared model dependency audit in `porting/domain/MODEL-DEPENDENCIES.md`; those MIT/Apache/Unlicense obligations also apply to distributed web artifacts. No third-party dependency source is vendored by this crate.

No wasm-bindgen CLI/wasm-pack was available during preparation; none is installed or required. Cached wasm-bindgen/web-sys/js-sys licenses, manifests and wasm-bindgen build.rs were inspected as an initial option; this implementation uses none of those crates. The direct Rust/WASM ABI avoids unsafe input-pointer allocation. Exports accept bytes individually into a bounded Rust input buffer, then dispatch a complete action. `output_ptr/output_len` expose a read-only HTML byte slice valid until the next dispatch/initialize call; the bridge decodes synchronously without retaining its pointer. The browser owns the WASM linear memory/host boundary. A 1 MiB input cap bounds import payloads; larger projects remain unsupported. Action JSON must contain a string action and string-valued fields; the rendered output is capped at 16 MiB. The cap is checked after HTML construction, so it bounds the DOM payload rather than guaranteeing a strict allocation ceiling. Memory growth and DOM replacement on each action are limitations requiring realistic performance tests.

## Reproducible preview preparation

Build from a complete integration checkout containing syson-model, syson-domain and syson-editor:

```
CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR="${SYSON_BUILD_DIR}/web" cargo build --offline --target wasm32-unknown-unknown --manifest-path rust/crates/syson-web/Cargo.toml
```

Copy the resulting `wasm32-unknown-unknown/debug/syson_web.wasm` alongside `web/index.html`, `web/loader.js` and `web/style.css` in an isolated preview output directory under `${SYSON_BUILD_DIR}`. Preview requires an HTTP origin serving these static files; fetch of WASM from `file://` is not supported by this loader. Do not start a server or browser until the single integration owner releases the display/server lane. No server, browser, network/security changes or installation was performed by this lane during preparation.

Five native adapter tests, strict clippy and a wasm32 build passed after integration with the shared model crate. Browser acceptance remains unrun; no server or display was launched. Source presence and compilation alone are not acceptance.

## Rotated label containment

The local SVG preview clips kind/name labels to each node's padded interior. After a 180-by-70 node is rotated to 70-by-180, its label clip is 46-by-164. Full escaped labels remain in SVG titles and the model list. A renderer regression test checks the narrow PartUsage/frontWheel case. Browser verification of this correction belongs to the integration owner's existing preview session; this lane does not launch a competing display.

## Registry-driven scalar property inspector

The Properties section calls `Document::inspect_node` from the shared model adapter and renders controls from each writable descriptor's actual scalar type. Parent and definition choices use the real pinned registry subtype predicate; they are not inferred from similar class names. All imported elements are selectable, including literal and relationship objects. The bounded SVG preview still draws only the previously supported package/part kinds.

| Stored type | Browser control | Rust conversion before shared-model validation |
| --- | --- | --- |
| String | Text input | Exact input string, including empty string |
| Boolean | Required true/false choice | JSON boolean; no string/numeric aliases |
| Signed 32-bit integer | Number input with integral step and bounds | Parse to i32, then JSON integer; overflow/fractions rejected |
| Finite double | Number input, any step | Parse finite f64, then JSON number; NaN/infinity/overflow rejected |
| Enum | Exact declared literal choices | Exact JSON string; unlisted/case-alias values rejected |

None/absent storage is labeled “Not set”; explicit JSON null is labeled separately. Metadata defaults are never placed into controls as saved values. An absent boolean/enum gets an unselected placeholder requiring an explicit choice. Optional strings offer a separate “Clear value (null)” mode. Removing a saved field, ESet/unset flags and enum reset are unsupported and have no successful action. A scalar change uses the shared transaction and appears in the staged preview; commit/cancel and undo/redo use the same model history. The committed JSON export excludes staged changes.

Calculated, nonchangeable, collection and identifier attributes are read only. Reference/containment and unresolved/ambiguous properties have explanatory entries under “Read-only and unavailable properties”; they have no edit forms. Unknown imported classes remain visible/selectable with unavailable-property text and no rename/property controls. The displayed metadata does not imply execution of upstream calculated values or all semantic constraints. Arbitrary form strings, property labels, stored values, enum literals and imported names pass through HTML escaping. The browser bridge remains unchanged: typed conversion and edit decisions happen in Rust.

Field forms carry `data-action="attribute"`, `data-attribute="<source attribute name>"`; controls have stable IDs `scalar-<name>` and `scalar-mode-<name>`. The integration owner's browser session can exercise actual controls through these selectors. Imported LiteralInteger/LiteralRational nodes expose their typed value controls; importing a supported document does not grant construction support for those classes.

Native verification now covers 13 UI tests including boolean staging/cancel/history; absent/null/empty/default distinctions; actual integer/double JSON writes and rejection; exact enum/null rejection; blocked/reference/collection/identifier/calculated edit rejection; unknown-class boundaries; escaping; previous model workflows and label containment. Strict offline Clippy and the wasm32 build are checked separately. New scalar inspector browser interaction and screen-reader verification remain unrun by this lane. Dependency addition is the existing dependency-free `syson-metamodel` path crate; its manifest has no build script or external dependency. No new external package, service, display or network change is introduced.
