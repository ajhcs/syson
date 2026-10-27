# RR18a atomic preview admission and reachable recovery

SOURCE_PREPARED_NOT_EXECUTED_BY_AUTHOR. Root split RR18 into RR18a/RR18b before
source mutation; RR18b depends on RR18a. Common base is
b1951bbaf15a1abcfb855702dc06f4230fb5263b: exact latest RR04#94 model plus
qualified #65 web/layout assets/browser prerequisites, incorporating #61 recovery.
Root owns source snapshots, Cargo/lock, formatter/compiler/browser, independent
review and publication. Author scope is web lib.rs/layout_controller.rs, new
rr18_regression.rs and evidence/RR18a only. Loader caret wiring is an explicitly
registered root-owned path; author did not edit it. Held raw/initializer candidate
features remain disabled; no held source is changed or newly qualified.

## Narrow behavior and remaining work

An import parses through the existing RR04 model authority, constructs a private
candidate, and admits its final presentation before swapping the active Editor.
Candidate message, focus, cleared selection and projected next canvas revision
match successful publication. The next revision is checked without mutation;
epoch exhaustion refuses before import/new-discard mutation, preserving old
Editor/history/selection/canvas. No epoch is reset, avoiding ticket reuse. This
is selected import/new-action atomicity, not a fix for every existing command's
post-canvas-error behavior or an arbitrary side-effect rollback mechanism.

Normal candidate/render HTML STILL allocates fully before its16MiB length check.
This limitation and dense/deep/escape renderer bounds stay RR18b OPEN. Existing
normal handle/render model comparisons and geometry encode/admit intermediates
also remain; no global allocation, RSS, latency or normal-render cap claim follows.
RR18a repairs destructive acceptance and reachable recovery, not all RR18 work.

The recovery shell exports committed model bytes via a native data-URI download,
independent of normal preview/selection/staged content. RR04 json_size/write_json
uses its admitted4MiB cap (never an invalid16MiB writer argument), followed by
one exact reservation for3x canonical bytes plus fixed HTML, below16MiB. Every
UTF8 byte is percent-encoded directly; no raw JSON String clone is constructed.
The download excludes pending edits and geometry. Export-error controls still
provide recover and two-step discard/keep/confirm independently of the writer.
Recover explicitly cancels staged edits/selection, retaining committed model,
model history and current canvas; it can remain in recovery if committed preview
still exceeds the cap. Explicitly confirmed discard clears document/history and
canvas. New foreground recovery returns keyboard focus to the name input.

Canvas model serializations use the existing1MiB canvas-document limit before
output allocation. scene/export return immediately for an absent frame, avoiding
model serialization entirely. Existing canvas encoding/geometry boundaries and
larger policy remain separate; successful geometry/model history semantics stay
covered by inherited tests and real UI checks.

Rejected import keeps the normal DOM controls, so drafts/focus/caret recovery is
qualified there. Generic unrelated DOM draft caching through a recovery shell
is NOT repaired and remains RR18b's explicit loader obligation. Root's one-line
caret fix restores error-preserved selection only to the same focused text control
(id/tag/type), following a fresh focused requestSubmit counterexample. No author
loader edit or arbitrary blurred-caret promise is made.

## Counterexample and prepared verification

Exact root browser fixture escaped-import.json is700258bytes SHA256
f767e5c4e8d61a24214ea9ec8e31ec4b57278fa372c9dcfbfea12913a910e399;
before.json is SHA256d0435d8f25e768999a3cd3ffa394f1d9bdf8f53419b696d652491259eb5b9100.
Both are copied byte-exact, not regenerated. Baseline-browser-02 directly observed
preview fallback/only Undo/no export after import. Its oldDataPreserved:false
was inferred from missing export, not private Editor inspection; no baseline Undo
click occurred. Source established replacement/history clearing. Baseline errors[]
recorded page errors only, not console events. Baseline build was one WASM command,
zero tests despite its generic label; setup01 KeyError and all failures remain.
Root's baseline-claim-boundaries.json is preserved verbatim, not reinterpreted.

The first public-ABI regression was frozen at7b545fa351091abbe8934074dff94d2ef0bd3a2a715a6e030861da668515e070
before production and copied by root. Fresh baseline WASM/browser already reached
the counterexample; no separate host red is claimed. Five public-ABI tests prepare
refused-import export/actualUndo preservation, staged fallback byte export/recover,
explicit keep/confirm discard, accepted import/history clearing and absent-frame
zero requested allocations. Four private tests cover exact final presentation/
inclusive cap comparison, epochMAX import/discard refusal/history, export-error
confirmation reachability and percent-decoded Unicode/CR/LF/literal-number bytes.
The boundary comparison uses a measured private candidate cap, not a claim of a
synthetic exact16MiB browser run. The allocator delegates original System layouts
and pointers unchanged; constant TLS tracks requests only during absent-frame calls.

The CJS is source-prepared only: actual default WASM at320/1440, focused native
form.requestSubmit, malformed and expanding refusal, current model/canvas/drafts/
caret2..5, actual Undo/Redo after refusal, accepted900kB opaque metadata, staged
expansion fallback, keyboard native download/recover/keep/confirm and cleared
history. Data-URI length is above2MiB; actual downloaded JSON payload bytes are
separately counted (roughly900kB), not misreported as a2MiB download. Console
errors/warnings and page errors are collected; LongTask durations are observations
without a pass threshold or INP/native-performance claim. Root must inspect
screenshots, served WASM/JS/CSS hashes, byte-exact download and retained failures.

Root gates: scoped Rust formatting, default-feature inherited+new Rust tests and
Clippy, release wasm32 build; serve exact four assets on a fresh owned loopback
origin; run preview_recovery.cjs with existing Playwright/Node and owned artifact
folder, capture downloads/screenshots/console/tasks; independent source/evidence
review and final committed input bindings. No all-features/raw-factory run or new
service/package/DB/native SysON launch. Budgets400production/900authored; root will
measure after formatting. Author ran no formatter/compiler/tests/runner/browser,
network, Git mutation, commit or publication. Parent RR18/RR18b, full UI/C01/native,
physical touch/other browsers and complete feature acceptance remain OPEN.
