#!/usr/bin/env python3
"""Write coordinator-owned parity plan records; does not change product code."""
import argparse
import json
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--plan-dir',type=Path,default=Path(__file__).resolve().parent.parent)
parser.add_argument('--source-root',type=Path,help='Frozen integration/source-inventory checkout; documentation-only plan branches do not contain every component inventory.')
args=parser.parse_args()
DEST = args.plan_dir.resolve()
ROOT = args.source_root.resolve() if args.source_root else DEST.parents[1]
if not (ROOT/'porting/source-map').is_dir():
    parser.error('Source inventory is absent here; pass --source-root pointing to the frozen integration/source-inventory checkout.')
units = []

def add(uid, title, deps, refs, tasks, checks, *, path=None, external=None, category='ui', done=None):
    for ref in refs:
        assert (ROOT / ref).exists(), ref
    units.append(dict(id=uid, title=title, lane=category, dependencies=deps,
        external_prs=external or [], source_refs=refs,
        paths=[path or f'rust/crates/syson-web/src/{uid.lower()}/'],
        tasks=tasks, verification=checks,
        commit_plan=['Freeze the concrete source contracts and matched fixtures; keep oracle captures separate from expectations.',
                     'Implement one behavior family with meaningful regression and actual caller integration.',
                     'Record exact execution inputs, original failures, passing receipts and independent disposition.'],
        budget='Aim <=400 changed handwritten production lines and <=900 authored lines including tests. Split by named operation/widget before execution if larger; generated tables and evidence are separate commits.',
        done_when=done or 'Every named task has a matched source/runtime disposition; actual Rust caller and user journey pass the specified gates; unsupported residuals remain open under separate IDs.',
        state='PLANNED', ownership='One agent owns this module; coordinator serializes root exports, manifests and composition.'))

add('C01','Close pinned source, artifact and effective capability inventory',[],
 ['porting/source-map','porting/source-map/inherited','porting/source-map/syside-source','porting/audits/20261026/native-reference.json'] if (ROOT/'porting/audits/20261026').exists() else ['porting/source-map','porting/source-map/inherited','porting/source-map/syside-source'],
 ['Freeze 6ba2ac64e6b16ee2b05752ad55395eebb87d4910 as target; identify an exact build or artifact correspondence before native equivalence claims.',
  'Resolve the 125-path release-label delta and inherited Sirius/EMF/Syside artifacts, sources, effective GraphQL schema, HTTP routes, UI extensions, database changelogs and standard-library closure.',
  'Join every source candidate, documented feature, classifier/feature/operation, grammar rule, AST hook, API field and inherited dependency behavior to a primary implementation PR and verification case.',
  'For absent, stubbed or intentionally unsupported native features record the source and actual native behavior; do not invent simulation, Git integration or authentication parity.',
  'Expand discovered obligations into named child PRs with immutable IDs, owners, budget and dependency edges before dispatch. An unresolved entry blocks final parity.'],
 ['Reproduce pinned hashes from local Git objects; compare source/artifact labels and exact selected runtime methods.',
  'Capture effective schema/routes and populated views from the same native artifact with synthetic state; distinguish declaration inventory from behavior.',
  'Fail closure validation on unassigned, unresolved, unrun, unsupported-required or source/artifact-mismatched obligations.'],
 path='porting/rust-parity-plan-20261026/closure/',category='coordination',external=[39,66])

add('C02','Build reproducible differential and connected-workflow harnesses',['C01'],['porting/validation','integration-tests-playwright','integration-tests-cypress','porting/source-map/upstream-tests.jsonl'],
 ['Define fixture envelopes with source/artifact/build/browser hashes, inputs, outputs, nondeterminism policy and independent expected outcomes.',
  'Freeze comparator interfaces and fixtures for separate model/AST/reference/diagnostic/API/database/geometry/browser child PRs; never normalize away IDs, ordering, diagnostics or real mismatches.',
  'Specify owned loopback native and Rust runtime interfaces, disposable databases, resource caps, exact process cleanup and failure retention; implement them in the scoped harness child PRs.',
  'Give every unit fast focused checks and scheduled composed suites; replay only changed or still-unexecuted cases with fresh input hashes.'],
 ['Inject each mismatch type and prove the comparator fails with a useful diff; preserve original streams.',
  'Run the existing eight parser fixtures plus seed naming/rotation/migration journeys; capture failures as failures.',
  'Verify runner teardown and prohibit production credentials, inherited schedulers or external service writes.'],
 path='rust/validation/parity-harness/',category='verification',external=[41])

add('C03','Compose reviewed branches and serialize integration ownership',[],['porting/pr-split-20261024','porting/audits/20261026'] if (ROOT/'porting/audits/20261026').exists() else ['porting/pr-split-20261024'],
 ['Read back every existing component, follow-on and audit PR at its expected head/base; preserve historical source and receipts.',
  'Create exact prerequisite snapshots when several open PRs are needed; list any ordered nonidentical shared blobs explicitly.',
  'One coordinator owns Cargo workspace exports, lockfiles, route registration and default UI assembly; authors deliver module APIs and integration patch boundaries.',
  'Land prerequisites only through the reviewed repository merge process, then retarget children, inspect diffs and revalidate actual composed heads. Never merge into temporary snapshots.',
  'Do not infer permission to merge/deploy from a passing test or plan status; existing held writer scopes retain their owners.'],
 ['Validate acyclic graph, defined prerequisites, snapshot blob hashes and no unordered concurrent write overlaps.',
  'Run meaningful downstream compilation, browser flows and dependency/notice checks after composition; do not attribute old captures to new trees.'],
 path='porting/rust-parity-plan-20261026/composition/',category='coordination',external=[22,66])

for uid,title,parts in [
 ('C04','Verify Rust production artifact and process closure',['Scan generated JS bootstrap, application bundles, subprocess calls and dynamic loads; application decisions must be authored in Rust.','Classify data assets, browser bindings, native libraries and development oracles separately; reject shipped Syside/Java/Node/Python business-logic fallback.']),
 ('C05','Compare AST, CST and diagnostic observations',['Compare tagged nodes, membership order, values, spans, error severity and recovery; retain comments and UTF16/byte distinctions.','Use each pinned grammar/AST obligation corpus; recovered AST is not proof native model import succeeds.']),
 ('C06','Compare model identities, references and query outputs',['Compare node/semantic/resource/project identities, ordered ownership, inverse/derived values and failures.','Allow only named nondeterministic values under explicit identity mapping; same-class cross-domain collisions must remain visible.']),
 ('C07','Compare API, database and migration observations',['Compare effective REST/GraphQL field/null/error semantics and persisted reload output across native/Rust instances.','Test concurrency, rollback, revisions, migrations and archive reload; snapshot/schema counts alone do not qualify behavior.']),
 ('C08','Compare geometry and browser task observations',['Compare graph constraints, ports, style/labels, selection/focus/history and actual user-visible task outcomes.','Use identical graphs and font/theme/viewport inputs; report exact tolerances and justified intentional improvements without erasing defects.']),
 ('C09','Run isolated differential runtime lifecycle',['Launch only owned loopback processes against fresh synthetic state; record tools/artifacts/environment and exact owned PID start times.','Implement bounded readiness, timeout, cancellation and teardown; preserve every failed setup/run and reject blank/intermediate screenshots.'])]:
 add(uid,title,['C01','C02'],['porting/validation','porting/source-map'],parts+['Keep each comparator/runtime operation in its own module; split before exceeding the shared review budget.'],
  ['Inject divergent inputs/results and prove failure; retain raw observations and human-readable diffs.',
   'Exercise an actual existing native and Rust fixture through the public harness interface; no mocked runtime equivalence.'],
  path=f'rust/validation/parity-harness/{uid.lower()}/',category='verification')

add('U01','Move DOM binding and browser event dispatch into Rust/WASM',['C03'],['rust/crates/syson-web','frontend/syson/src/index.tsx'],
 ['Review and pin Rust WASM/DOM binding dependencies and generated bootstrap provenance with minimal used APIs and license closure.',
  'Move input extraction, action dispatch, focus/selection/caret and live-status handling from loader.js into typed Rust modules.',
  'Retain a generated instantiate/binding bootstrap only; no handwritten application decisions in JavaScript.',
  'Keep current draft, disclosure, history and geometry behavior while replacing each event family in ordered small child commits.'],
 ['Run the five composed browser journeys against the Rust-owned event path and unchanged model bytes.',
  'Scan the distributable and runtime process tree for custom JS/TS behavior and Java/Node/Python subprocesses; test CSP and error startup.',
  'Compare before/after keyboard focus, live-region identity, memory and event listener disposal.'],external=[65])

add('U02','Move canvas scheduling, measurements and viewport state into Rust',['U01'],['rust/crates/syson-web','frontend/syson-components/src/nodes'],
 ['Port DOM text measurement, font observation, requestAnimationFrame scheduling, cancellation and stale-ticket handling into Rust.',
  'Port Actual size/Fit, scroll/pan state and keyboard viewport events; keep view-only actions outside model and geometry history.',
  'Add typed lifecycle ownership for observers, callbacks and buffers; release them on route/editor disposal.'],
 ['Actual browser font-ready, delayed-font, cancellation, stale result and repeated mount/unmount checks.',
  '32-node/64-edge fixed-fixture action latency: apply workload bands and measurement rules in performance-policy.json; record raw samples and distinct main-thread task observations without a universal50ms veto.',
  'Verify 14px names/12px classes at actual size and readable zoom/scroll alternative at320/1440px.'],external=[65])

add('U03','Project home, templates and application routing',['U01'],['doc/content/modules/user-manual/pages/features/homepage.adoc','frontend/syson/src','backend/application/syson-application-configuration'],
 ['Implement Rust routes, project listing, template choice, creation progress, empty/error/retry states and deep links.',
  'Bind route/project identity to the qualified server boundary; cancel stale requests when switching projects.',
  'Preserve dirty edits through navigation confirmation and recoverable load failure.'],
 ['Matched native blank/template project create/reopen journey and Rust API/database evidence.',
  'Browser Back/Forward/reload, malformed link, slow/stale load and keyboard focus placement.',
  'Responsive320/390/640/1365/1440px and loading/error screenshots.'])

add('U04','Explorer navigation, document actions and search',['U03'],['backend/views/syson-tree-explorer-view','backend/services/syson-tree-services','doc/content/modules/user-manual/pages/features/explorer.adoc'],
 ['Retain the batch13 512-child synthetic Rust counterexample: bounded individual names still produce a roughly33000CSSpx page. Provide usable collapse/filter/windowed navigation with exact selected/fulltext/keyboard access; compare a matched native fixture before claiming large-explorer parity.',
  'Implement virtualized semantic tree with documents/libraries, containment hierarchy, selection and reveal.',
  'Add source-supported create/rename/delete/copy/move/import actions with actual server commands and read-only guards.',
  'Implement keyboard tree navigation, accessible expansion, name search and stable selection after refresh.'],
 ['Matched native explorer actions with ordered membership/identity and undo artifacts.',
  'Actual browser keyboard-only create/move/delete/cancel; stale project and read-only library refusal.',
  'Profile 1k/10k elements, bounded DOM count and memory; prove scrolling does not lose selection.'])

add('U05','Details forms, typed references and feature help',['U04'],['backend/services/syson-form-services','backend/application/syson-application-configuration','doc/content/modules/user-manual/pages/features/details.adoc'],
 ['Map effective property descriptions to Rust widgets, default/read-only/derived values and reference selectors.',
  'Implement scalar/list/enum/reference staging, clear/unset distinction, validation/help text and error recovery.',
  'Preserve unsubmitted drafts, dirty context, source-active focus and accessible live announcements.'],
 ['Matched property fixtures for every effective widget kind; null/omission/eIsSet and list ordering differences explicit.',
  'Keyboard and screen-reader task coverage, all validation states and failed command preservation.',
  'Measure real select/stage/cancel/commit events against workload-specific ranges in performance-policy.json, restoring identical state between repetitions; retain raw samples and outlier explanations. Use at least30measured samples when reporting p95; no universal timing veto.'],external=[61,62])

add('U06','Connected text import and positioned validation',['U05'],['backend/application/syson-sysml-import','backend/application/syson-sysml-validation','frontend/syson-components/src/extensions/InsertTextualSysMLv2Modal.tsx'],
 ['Wire actual Rust parser/importer/identity bridge/semantic validation into an editable import preview.',
  'Show positioned diagnostics with Unicode-safe selection, preserve invalid text, support cancel and atomic commit.',
  'Stage into selected project/document identity with explicit unsupported-language feedback and unchanged prior document on failure.'],
 ['Actual browser valid/invalid/unsupported/oversize import, diagnostic navigation, cancel/commit/undo/reload.',
  'Compare matched native import AST/model/identity outcomes, not just declaration counts.',
  'Repeat with CR/LF/CRLF, supplementary Unicode, cross-resource references and same-class UUID collisions.'],external=[52,59,63])

add('U07','Real file download/upload and archive reload',['U06'],['backend/application/syson-sysml-export','doc/content/modules/user-manual/pages/features/import-export-textual.adoc','doc/content/modules/user-manual/pages/integration.adoc'],
 ['Replace copy-only export with browser File/Blob downloads and upload previews for admitted text/model/archive formats.',
  'Preserve encoding, names, node/semantic/resource identities, libraries, representations and unknown native values.',
  'Enforce upload/expanded archive/member limits, path safety, cancellation and unchanged-document recovery; split archive handling from text if budget exceeded.'],
 ['Download actual browser bytes, reopen in fresh Rust context and native reference; perform reverse direction too.',
  'Retain batch13 concrete reload difference: the matched long Package name survives native UI page reload, while the Rust prototype currently needs explicit canonical JSON reimport. Manual reimport is file round-trip evidence, not connected persistence parity; require B06/U03 integration for durable reopen.',
  'Invalid UTF8, duplicate keys, truncated files, corrupt archives and cancellation never partially publish.',
  'Inspect download UX, filename, feedback, progress and keyboard focus at all target widths.'],external=[55,56,63])

add('U08','Client history and remote command result integration',['U03'],['doc/content/modules/user-manual/pages/features/keyboard-shortcuts.adoc','rust/crates/syson-web'],
 ['Map original F30 to real server command/input IDs and separate semantic/representation history stacks.',
  'Handle undo/redo during pending commands, reconnect, empty history and rejected operations without discarding drafts.',
  'Bind keyboard shortcuts and visible availability to actual state; preserve remote result ordering and session boundaries.'],
 ['Two real clients with ordered commands, delayed/reversed responses, cancel, reconnect and replay.',
  'Browser draft retention on no-op Undo, canvas history independence and commit/export isolation.',
  'Matched native missing-ID behavior and explicit product-improvement disposition.'])

add('U09','Diagram representation lifecycle and selection',['U02','U04'],['backend/services/syson-representation-services','doc/content/modules/user-manual/pages/features/representations-view.adoc'],
 ['Create/open/rename/delete representation tabs linked to immutable description IDs and semantic targets.',
  'Persist representation selection, view state and read-only behavior; distinguish graphical deletion from semantic deletion.',
  'Implement single/multi-selection, reveal/recenter, related representation navigation and route restoration.'],
 ['Matched native project with two representations of the same semantic element and independent geometry.',
  'Reload, tab disposal, deleted target and stale selection never corrupt semantic data.',
  'Keyboard selection and accessible names remain stable through subscription updates.'])

add('U10','Rust layered layout execution and worker lifecycle',['U02','U09'],['backend/views/syson-diagram-common-view','frontend/syson-components/src/nodes','doc/content/modules/user-manual/pages/features/scaling-limits.adoc'],
 ['Inventory exact effective ELK options/ports/constraints and fixture each currently excluded graph shape.',
  'Move expensive Rust layout into a cancellable WASM worker with typed request/result validation and resource bounds.',
  'Preserve deterministic node/edge identity, manual pins, coordinate transforms, nested labels and history publication.'],
 ['Matched native graph inputs with explicit geometric tolerance/invariant policy; record intentional improved arrangements separately.',
  'Cycles, disconnected graphs, nested containers, long labels, fonts, partial/manual layout and stale worker results.',
  '30-sample32/64 interactive budget and 256/1024-node scaling/memory, pan/selection responsiveness while computing.'])

add('U11','Ports, edge routing and reconnect interactions',['U10'],['backend/views/syson-diagram-common-view','backend/services/syson-diagram-services'],
 ['Implement side/position constrained ports, edge attachment, bendpoints, parallel edges, cycles and nested boundary routing.',
  'Support source-supported edge create/reconnect/delete with live preview and semantic type/ownership validation.',
  'Keep transforms/zoom and manual routing changes in representation history; refuse invalid edge commands atomically.'],
 ['Matched native geometry/semantics for each supported edge class with obstacle and port invariants.',
  'Pointer and keyboard workflows including escape/cancel, hit targets and read-only graph.',
  'Stress bounded edge counts, collision detection and selected-edge painting cost.'])

for idx,(folder,label) in enumerate([('package','package'),('note','note'),('view_frame','view frame'),('imported_package','imported package')],12):
 add(f'U{idx:02}',f'Rust {label} custom node appearance and interactions',['U09','U10'],[f'frontend/syson-components/src/nodes/{folder}','backend/metamodel/syson-siriusweb-customnodes-metamodel'],
  [f'Port the exact {label} converter, SVG/DOM structure, style properties and layout handler into Rust.',
   'Implement palette appearance edits, resize/move/rotate, selected/read-only/error states and persistence.',
   'Preserve multiline label measurement, icon/compartment hit regions and native style identity.'],
  ['Fresh matched default/selected/resized/rotated screenshots; geometry and appearance JSON differential.',
   'Actual pointer/keyboard appearance edit, cancel/undo/reload with no semantic identity changes.',
   'Contrast/focus/44px targets at320px and200/400% zoom; accessibility tree plus speech checks.'])

add('U16','Direct-edit labels and expression modal',['U05','U09'],['backend/services/syson-direct-edit-grammar','backend/services/syson-diagram-services','frontend/syson-components/src/expressions','doc/content/modules/user-manual/pages/features/expressions.adoc'] if (ROOT/'frontend/syson-components/src/expressions').exists() else ['backend/services/syson-direct-edit-grammar','backend/services/syson-diagram-services','doc/content/modules/user-manual/pages/features/expressions.adoc'],
 ['Port narrower DirectEdit.g4 behavior separately from full SysML text import.',
  'Implement direct-edit rename/type/multiplicity and expression modal with source-supported FeatureValue/default/initial behavior.',
  'Preserve invalid drafts and positioned errors; accepted empty expression deletes only the expression.'],
 ['Matched native positive/negative direct edit and expression cases including empty/null/unresolved references.',
  'Real browser edit/cancel/error/correct/commit/undo/reload and focus return.',
  'No silent data loss when opening another representation or losing a subscription.'])

add('U17','General View node/edge/tool registry and source assignment',['U09','U11'],['backend/views/syson-standard-diagrams-view','backend/views/syson-diagram-common-view','doc/content/modules/user-manual/pages/features/general-view.adoc'],
 ['Freeze every effective description ID, palette tool, precondition, node/edge mapping and AQL service dependency.',
  'Assign all concrete providers to source-scoped child PRs; port registry dispatch and stable description identity.',
  'GeneralView creation/expose/import/visibility and related-target rules must run through actual semantic services.'],
 ['Native populated GeneralView registry and fixture coverage diff; no unresolved provider accepted as a blank diagram.',
  'Existing Vehicle journey remains usable and readable with incremental feature families.',
  'Read-only, inherited and library members retain their effective source visibility.'],external=[48,54])

add('U18','Interconnection View and connection endpoint workflows',['U17'],['backend/views/syson-standard-diagrams-view','doc/content/modules/user-manual/pages/features/interconnection-view.adoc'],
 ['Port InterconnectionView provider/query/create services and eligible semantic contexts.',
  'Implement nested ports/items/parts/interfaces/connections, connection ends and binding/flow creation.',
  'Persist representation and navigation independently from GeneralView with actual command history.'],
 ['Matched native connected fixture create/reconnect/reveal/export/reload; identity and endpoint order checked.',
  'Keyboard and pointer tool flows with unavailable/invalid target feedback.',
  'Nested port/label routing and responsive inspection on realistic diagrams.'])

add('U19','Action Flow View and control-flow tools',['U17'],['backend/views/syson-standard-diagrams-view','doc/content/modules/user-manual/pages/features/action-flow-view.adoc'],
 ['Port ActionFlowView contexts, compartments and parameter/action/control-node descriptions.',
  'Implement start/done/fork/join/decision/merge, succession and flow tools using qualified action semantics.',
  'Preserve direction, guard/expression, nested action and representation history behavior.'],
 ['Matched native branch/join/loop/subaction fixtures and create/delete/cancel/undo/export/reload journey.',
  'Inspect arrow/guard/icon readability and keyboard access to every tool.',
  'Layout invariants and performance on branching/cyclic graphs with long labels.'])

add('U20','State Transition View and transition tools',['U17','U16'],['backend/views/syson-standard-diagrams-view','doc/content/modules/user-manual/pages/features/state-transition-view.adoc'],
 ['Port state-machine contexts, nested states, entry/do/exit and initial/final presentation.',
  'Implement transitions, triggers, guards and effects through actual semantic commands.',
  'Retain nested-state ownership and representation state on move/delete/undo.'],
 ['Matched native hierarchical state fixture including invalid transition and unresolved expression.',
  'Real browser create/connect/edit/delete/cancel/undo/reload; no screenshot-only acceptance.',
  'Accessible labels and hit targets for dense guarded transitions at zoomed/narrow viewports.'])

add('U21','Requirements table and requirement authoring',['U05','U09'],['backend/views/syson-table-requirements-view','backend/services/syson-table-services','doc/content/modules/user-manual/pages/features/requirement-management.adoc'],
 ['Port requirement table description, row/column queries and create/import existing requirement executors.',
  'Implement keyboard cell editing, selection, sort/filter, semantic versus graphical delete and row virtualization.',
  'Keep requirement text/ID/stakeholder/subject and satisfaction references coherent with diagram/details edits.'],
 ['Matched native row ordering and column values; create/edit/import/delete/undo/export/reload.',
  'Keyboard grid and screen-reader row/column semantics, wrapping/contrast/error states.',
  '1k/10k rows with bounded DOM/memory and measured sort/filter/input responsiveness.'])

add('U22','Validation panel and positioned problem navigation',['U06','U09'],['backend/application/syson-sysml-validation','doc/content/modules/user-manual/pages/features/validation.adoc'],
 ['Expose applicable-rule IDs, severity, source/model positions and incremental validation lifecycle.',
  'Select/reveal problem targets across explorer, diagram and text without losing drafts.',
  'Show stale/pending/cancelled results and distinguish unsupported validation from a clean model.'],
 ['Matched native diagnostics on valid/invalid/cross-resource fixtures with explicit rule coverage.',
  'Actual browser issue selection, correction, error recovery and live announcements.',
  'Measure incremental revalidation on1k/10k elements and cancel stale project results.'])

add('U23','Query, related elements and cross-project search',['U04','U09'],['doc/content/modules/user-manual/pages/features/query-view.adoc','doc/content/modules/user-manual/pages/features/related-elements.adoc','doc/content/modules/user-manual/pages/features/cross-project-search.adoc','backend/services/syson-model-services'],
 ['Port effective AQL query/selection/editingContext service surface with a Rust evaluator and resource boundaries.',
  'Implement related elements/representations and source-supported cross-project search with cancellation/pagination.',
  'Present typed results, errors and navigation while keeping read-only/library/session context explicit.'],
 ['Matched native values, ordering, null/error behavior for each effective query/service family.',
  'Real browser query/error/cancel/select/reveal and cross-project stale-result refusal.',
  'Profile indexed search and large result virtualization; no raw evaluator PASS counted as UI parity.'])

add('U24','Library import, version selection and publication UX',['U03','U07'],['doc/content/modules/user-manual/pages/features/library-import.adoc','doc/content/modules/user-manual/pages/features/publication.adoc','frontend/syson-components/src/extensions/omnibox'],
 ['Implement available library/version/dependency browsing, import progress and read-only imported content.',
  'Publish only eligible SysML-root documents and their representations with source-supported name/namespace/version policy.',
  'Handle duplicate version/name, missing dependency, removal/reload and cancellation with recoverable feedback.'],
 ['Matched native library import/publish/reimport fixture with dependency/identity/representation artifacts.',
  'Browser duplicate publication, read-only refusal, cancellation and project reload.',
  'Inspect large library chooser search, focus, accessible progress and clear resulting scope.'])

add('U25','Omnibox, keyboard shortcuts, menus and extension composition',['U03','U09'],['frontend/syson-components/src/extensions','frontend/syson-components/src/extensions/registry','doc/content/modules/user-manual/pages/features/keyboard-shortcuts.adoc'],
 ['Port effective extension registry merge/precedence, menu entries, navigation/footer/icon contributions and commands.',
  'Implement omnibox search, shortcuts, context menus and tool availability with typed Rust registries.',
  'Port show/hide icons/inherited members/library members and URL-controlled initial settings without stale optimistic state.'],
 ['Matched native registry ordering and command availability in read-only/selection/diagram contexts.',
  'Keyboard open/filter/execute/escape and nested menu focus return; no shortcut fires during unrelated text entry.',
  'Transport failure restores settings/announces errors; persisted settings survive reopen.'])

add('U26','Collaboration, reconnect and multi-client editor states',['U08','U09'],['doc/content/modules/user-manual/pages/features/collaboration.adoc','frontend/syson-components/src/extensions'],
 ['Implement observed shared-session subscriptions, concurrent edit notifications and reconnect semantics.',
  'Preserve last committed state through disconnect/retry, dispose listeners and avoid duplicate acknowledgements.',
  'State conflict behavior from native captures; do not invent CRDT/offline merge as parity.'],
 ['Two isolated browser contexts edit same/different project, disconnect/reconnect and reorder responses.',
  'Matched native ordering/conflict behavior with persisted model/history artifacts.',
  'Soak subscription churn and inspect memory/listener growth and accessible reconnect feedback.'])

add('U27','Accessible responsive application qualification',['U04','U05','U07','U08','U18','U19','U20','U21','U22','U23','U24','U25','U26'],['doc/content/modules/user-manual/pages/user-interface-overview.adoc','doc/content/modules/user-manual/pages/tour.adoc'],
 ['Audit every route/widget in loading/empty/populated/error/read-only/pending states at target viewports.',
  'Qualify keyboard-only tasks, actual screen-reader speech,200/400% browser zoom, contrast, focus visibility and reduced motion.',
  'Run representative engineer tasks with consented participants: completion/errors/recovery/time and observations; fixes get separate behavior PRs.'],
 ['Retain screenshots, full AX trees, speech transcript/environment and defect/task matrix; automated tools supplement manual evidence.',
  'No horizontal page overflow at320/390/640/1365/1440px;44px primary targets and readable diagram alternative.',
  'Retest each repaired flow on Chromium/Firefox/WebKit and physical touch where supported; publish exact limits.'])

add('U28','Cross-view performance and lifecycle qualification',['U27','U10'],['doc/content/modules/user-manual/pages/features/scaling-limits.adoc','integration-tests-playwright'],
 ['Measure cold/warm startup, small edit feedback, import/export, selection/search, layout and project reopen on fixed datasets.',
  'Profile browser main-thread/worker/server/database time and memory separately on32/256/1024-node shapes plus1k/10k element projects.',
  'Fix measured bottlenecks in independent PRs; set supported limits from repeatable evidence and clear overload/cancel UX.'],
 ['At least30samples for p50/p95/max; raw samples, environment and noise controls; use real browser events.',
  'Use ordinary/dense/capacity-specific target ranges in performance-policy.json; capture distributions, outliers, actual input responsiveness and matched native metrics; no single timing veto.',
  'Repeated mount/dispose/import/reconnect soak has bounded memory, listener and pending-task counts.'])

add('U29','Qualify native documented capacity and remove prototype-only limits',['U28'],['doc/content/modules/user-manual/pages/features/scaling-limits.adoc'],
 ['Inventory every Rust parser/document/model/diagram/API/request limit; current256-declaration,32-node and near1MiB-document bounds cannot qualify whole-product scaling.',
  'Construct staged synthetic workloads around documented10MB model,150000 semantic elements,2000 representations/project,1000 elements/representation,1000 projects and80 concurrent users.',
  'Observe native behavior on the same fixed capacity workloads and provision a reviewed resource allocation before large/concurrent runs; documentation numbers are claims to test, not measured guarantees.',
  'Increase Rust supported limits only with bounded allocation, cancellation/backpressure, virtualized UI, durable storage and repeatable native/composed evidence; split independent limit repairs into child PRs.'],
 ['Ramp1x/0.5x/near-boundary/over-boundary datasets and report p50/p95/max/RSS with exact host/storage/browser/client count.',
  'No silent truncation, lost edits, partial publication, unbounded memory or frozen input under overload; capture recovery/reopen.',
  'Budget-relative comparisons use matched workloads, version and hardware; any unmet native capacity remains a named parity blocker.'])

# Exact source-file batches make the broad view families reviewable rather than a single huge diagram PR.
provider_files = sorted(str(x.relative_to(ROOT)) for base in ['backend/views/syson-standard-diagrams-view','backend/views/syson-diagram-common-view'] for x in (ROOT/base).rglob('*.java') if '/src/main/' in str(x) and any(s in x.name for s in ['NodeDescription','EdgeDescription','NodeTool','EdgeTool','ToolProvider']))
for index in range(0,len(provider_files),4):
    batch=provider_files[index:index+4]; uid=f'V{index//4+1:03}'
    add(uid,'Port concrete diagram providers: '+', '.join(Path(x).stem for x in batch),['U17'],batch,
     ['Freeze each provider method, description ID, semantic target, precondition, label/style, tool action and inherited behavior dependency.',
      'Implement this exact provider batch in Rust typed descriptors; split any provider with >400 production-line equivalent into named behavior children.',
      'Run the real semantic service/registry and actual browser creation/edit flow for every reachable contribution; preserve graphical versus semantic changes.'],
     ['Matched native descriptor and model/representation outputs for positive/read-only/unresolved contexts.',
      'Browser screenshots and keyboard/pointer tool paths for each affected node/edge/compartment.',
      'Register coverage per provider method; unresolved dependencies or merely constructing a descriptor cannot close the unit.'],
     path=f'rust/crates/syson-view-descriptions/src/{uid.lower()}/',category='diagram-providers')

(DEST/'lanes/ui.json').write_text(json.dumps({'schema':1,'scope':'Coordinator UI/diagram/acceptance plan; source obligations are assignments, not coverage proof.','units':units},indent=2)+'\n')
print(json.dumps({'units':len(units),'provider_files':len(provider_files),'provider_batches':(len(provider_files)+3)//4}))
