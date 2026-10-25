# Inspector accessibility QA

The coordinator ran the existing installed Playwright workflow against the exact release WASM served from an owned loopback-only preview at http://127.0.0.1:38147/. Browser plugin not available is the recorded fallback reason. Desktop viewport1365x1000 and mobile390x844 passed. Each journey blocked service workers and guarded against external-origin requests; no external requests were observed.

The new keyboard journey creates Vehicle, Wheel and a typed wheel usage using native select arrows, Tab/Shift+Tab, text entry and Enter. Selection opens the named inspector, rename and a boolean property exercise stage/cancel/commit, committed exports remain unchanged until commit, and focus returns to each origin. The Chromium full accessibility tree verifies Element name textbox and Is abstract combobox names/focus, plus status/polite/atomic properties with the same persistent backend node10. A DOM observer verifies that the status object remains connected and guarded live attributes are never cleared. It records staged/cancelled/committed text updates. These are browser accessibility semantics, not captured screen-reader speech.

| Check | Result |
| --- | --- |
| Exact page identity and nonblank app | PASS |
| No error overlay | PASS, coordinator screenshot inspection |
| New journey console errors/warnings | Empty arrays |
| Keyboard tab order and origin focus | PASS |
| Persistent live object and CDP semantics | PASS |
| Mobile focus and no horizontal overflow | PASS |
| Existing canvas, authoring and scalar journeys | PASS |

The coordinator inspected desktop, scalar-detail and mobile screenshots: visible focus ring, usable labels, no overflow or overlay. Screenshots and four full AX trees remain under `/workspace/build/builderbob/syson-batch7-20261025/browser/f36-browser-01/inspector_accessibility/`; their hashes are in the browser receipt. Only small focus/live/AX-summary JSON is retained here alongside exact runners, source hashes and original streams. The served WASM hash equals the native build receipt, and every before/after input hash is unchanged.

Commands: the coordinator's frozen `run_checks.py F36 f36-02` invokes offline/locked Rust tests, formatting, Clippy and release WASM. `SYSON_BUILD_CAPTURE=f36-02 run_browser_checks.py f36-browser-01` invokes inspector_accessibility, layout, authoring and scalars sequentially. The exact argv, Node/browser hashes, stream hashes and process cleanup are retained in the copied receipts. The earlier f36-01 capture passes41native tests with no browser run; its exact inputs are retained before the observer test was strengthened.

No installed screen reader was exercised. Executable probes found no orca/espeak-ng/speech-dispatcher/accerciser on this session's PATH; no installation or session/service change was attempted. Other browsers/platforms, real assistive speech, all editors and native SysON accessibility parity remain unqualified.
