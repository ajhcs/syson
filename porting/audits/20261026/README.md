# SysON parity and usability audit of 33 scoped units

The 33 units are useful reviewed building blocks, but they do not represent 33 completed user features. Five are contract or preparation packets, 15 are algorithm components, 11 are bounded backend adapters and two directly expose browser surfaces. Four more algorithms support those surfaces indirectly. Most new adapters are not yet connected to the default browser workflow.

The audit found eight concrete repair families, now available in separate draft PRs. Selected behavior is stronger after the repairs, but overall SysON product parity and production usability remain unqualified. The highest remaining priorities are application integration, durable project workflows, wider language coverage and matched native behavior comparisons. Planning counts are not a feature-parity percentage.

## Scope and reference

All 33 units received source review against the pinned native contracts. Three parallel auditors covered model/query/names, parser/interchange and platform/persistence; the coordinator also covered the six UI/geometry units and actual browser/runtime work. This was an expert cognitive walkthrough, not a recruited-user study.

The real official SysON 2026.9.3 application ran on Java 21 with fresh PostgreSQL 15.19 state, bound only to loopback. The walkthrough exercised project/template creation, the editor, populated Batmobile General View, Arrange and a 390px viewport. The native artifact is identified by JAR SHA256 `85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1` and official image source label `b380dada415ce05155150f23cb3b74ae21cc45e4`. The Rust source comparison pin is `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`; their 125 changed paths are retained in [native-reference.json](native-reference.json). Labels and hashes do not independently attest a rebuilt source closure. Historical F39/F40 qualification is not relabeled complete by this artifact run.

The exact pinned embedded Syside CLI also ran eight identical text fixtures against the actual Rust parser/importer. Four selected declaration class/name projections matched: ordinary typing, nested FeatureMembership, forward qualified typing and quoted escapes. Comments, untyped PartUsage and typed usage bodies parse natively but are explicitly unsupported by the finite Rust parser. Native block comments can produce semantic Comment nodes, so skipping all comments would lose behavior. The malformed missing-brace CLI fixture emitted a recovered AST while Rust refused it; that does not prove the native application accepts an invalid model. See [parser-differential.json](parser-differential.json).

Native Batmobile and the small Rust preview are different workloads. Their screenshots support capability and usability comparisons, not geometric equality, native performance ratios or a complete authoring differential.

## Verified fixes and commit boundaries

Each fix has one implementation/regression commit and one audit-disposition/evidence commit. No prerequisite, snapshot or integration branch was merged. The exact base, head and dependency map are in [fix-prs.json](fix-prs.json).

| PR | Repair | Validation |
|---|---|---|
| [58](https://github.com/ajhcs/syson/pull/58) | Exclude the synthetic AST root from the real 256-declaration budget | Original boundary failure retained; 19 tests, formatting and Clippy pass |
| [59](https://github.com/ajhcs/syson/pull/59) | Correct diagnostic lines and UTF-16 columns for CR, LF and CRLF | Original two failures retained; 18 tests including 27 newline combinations pass |
| [60](https://github.com/ajhcs/syson/pull/60) | Replace quadratic reconciliation scans with indexes while preserving first UUID, pointer identity and source order | 16 tests and identical release benchmark pass |
| [61](https://github.com/ajhcs/syson/pull/61) | Retain drafts during canvas history/cancel and rejected operations | 41 Rust tests plus actual browser draft/disclosure/model-byte checks pass |
| [62](https://github.com/ajhcs/syson/pull/62) | Show semantic hierarchy, disclose relationships/advanced properties and guard selection during a staged edit | 44 Rust tests; keyboard, AX, focus, 320px reflow and 44px targets pass |
| [63](https://github.com/ajhcs/syson/pull/63) | Explicitly bridge F14 semantic-ID typing to F33 native node-ID wires | 26 tests including same-class cross-domain collision and nested/cross-package actual imports pass |
| [64](https://github.com/ajhcs/syson/pull/64) | Stream request serialization into the 8 MiB budget before allocating an oversized batch | 17 tests; unchanged bridge/schema; identical actual-binary memory probe passes |
| [65](https://github.com/ajhcs/syson/pull/65) | Add readable Actual size and explicit Fit overview with keyboard scrolling | 44 Rust tests; 32-node, 320/1440px browser checks and composed journey coverage pass |

The identity bridge is explicitly caller-selected, one-way and local to the six admitted package/part classes. It never guesses between node and semantic IDs. Reverse conversion and complete native interchange remain separate work.

Fixes 61 and 62 have disjoint source changes. Fix 65 targets their exact combined snapshot. Fix 63 targets exact F14 and F33 blobs. Land prerequisites, retarget to the actual integration base, inspect the resulting diff and revalidate. Never merge a child into a prerequisite feature branch or temporary snapshot.

## Measured performance

| Workload | Before | After | Interpretation |
|---|---:|---:|---|
| 1,000 unchanged documents | 0.681 ms median | 0.069 ms median | Same release planner benchmark, five samples |
| 5,000 unchanged documents | 14.122 ms median | 0.411 ms median | Same identity/order semantics |
| 10,000 unchanged documents | 55.109 ms median | 0.897 ms median | 61.46 times faster for this workload |
| Oversized 64-operation storage batch using one borrowed near-1 MiB document | 267,536 KiB maximum RSS | 13,332 KiB maximum RSS | 261.3 to 13.0 MiB, 95.0 percent lower; both refuse with unchanged DB bytes |
| 30 real Layout clicks, 32 nodes and zero edges | No matched before capture | 48.7 ms p50, 49.0 ms p95, 49.1 ms maximum | Final composed browser UI |
| 30 real Layout clicks, 32 nodes and 64 edges | No matched before capture | 65.1 ms p50, 65.6 ms p95, 69.9 ms maximum | A second bounded graph shape, not all possible graphs |

The UI proxy measures capture-phase click through valid final geometry/no pending ticket and two animation frames. Neither UI workload recorded a main-thread long task over 50 ms. First visible name-field observation was 28 to 30 ms in these local warm-host runs. These are not INP, LCP, physical display presentation, WAN startup, cross-device guarantees or native speed comparisons. Raw samples and fixture proofs are in [ui-performance.json](ui-performance.json).

The storage RSS probe times the actual release binary, excluding compilation, with the same source fixture, process cap and fresh owned database paths. Reconciliation timings occur inside the benchmark; its wrapper RSS includes compilation and is not a planner-memory measurement. Hash indexes remove the quadratic scan paths; ordered UUID sets remain part of the algorithm. Tiny inputs and every update shape are not covered by the large no-op speed claim.

## Visual and usability results

The accepted walkthrough steps were:

1. Empty Rust editor and first creation entry.
2. Staged Package preview with committed export isolation.
3. Package, PartDefinition and typed PartUsage authoring.
4. Rename stage, cancel, commit and history.
5. Reproduction and repair of unrelated draft loss during canvas undo and rejected operations.
6. Semantic navigation, advanced-property disclosure, stable live status and staged-edit selection context.
7. Actual size, Fit overview, keyboard scrolling and return on 32 nodes at 320px and 1440px.
8. Native project/template entry, rendered editor and populated General View reference.

The flat list previously gave unnamed memberships the same prominence as user elements. The repaired outline nests semantic elements and keeps all relationship objects selectable in a labeled disclosure. Common properties remain visible, while advanced and read-only properties no longer dominate the primary task. A pending edit cannot silently move its inspector context to a different element.

Canvas undo previously changed an unsubmitted rename back to the committed name. Unrelated canvas actions and recoverable errors now preserve input and disclosure state. Default Actual size gives names 14 CSS pixels and class labels 12 CSS pixels at both tested widths. Fit overview intentionally reduces labels to show the entire diagram; users can always return to readable local scrolling without a model dispatch, history entry or geometry change.

The final composition covers the keyboard/AX, layout, authoring, scalar and draft-recovery journeys, plus the dedicated viewport test. Original composite attempts remain failed where appropriate: old browser fixtures blindly toggled import disclosures that now correctly stay open after errors. Their state-aware opening corrections preserve all data/history assertions. The first viewport fixture also incorrectly parsed an initially empty geometry export. No failure was relabeled as a passing capture. One older browser input was recovered later and verified against its original SHA256; that recovery is explicitly distinguished from an original archive.

Fresh screenshots were inspected at desktop, 640px, 390px and 320px. They and full AX trees remain external; [evidence-manifest.json](evidence-manifest.json) records exact paths and hashes. This is Chromium keyboard and viewport evidence, not screen-reader speech, physical touch, other browsers or complete accessibility certification.

## Quality gates and current qualification

[quality-gates.json](quality-gates.json) defines proposed project acceptance ranges. They are requirements for future qualification, not invented current scores.

| Area | Proposed expectation | Current evidence |
|---|---|---|
| Data safety | No partial model/SQL publication; no discarded drafts during unrelated actions; clear staged-edit target | Focused source, red/green, SQL and browser proof for the repaired families |
| Diagram readability | Readable default labels and an explicit overview alternative | 14px names and 12px class labels at 320/1440px on 32 nodes |
| Responsive interaction | No horizontal page overflow at 320/390/640/1365/1440px; 44 by 44 CSS pixel primary targets | Sampled tested states pass; arbitrary imported states remain a separate gate |
| Keyboard and accessibility | Stable focus, accessible names and announcements, reachable disclosures, suitable contrast and zoom | Scoped keyboard/AX/live evidence passes; full contrast, 200/400 percent zoom and speech remain unqualified |
| Interactive speed | Select/stage/cancel/commit p95 at most 100 ms; bounded layout p95 at most 250 ms with no task over 50 ms | Two 30-sample layout workloads pass; a dedicated 30-sample small-edit study is still needed |
| Backend scaling | Release measurements with fixed workload and raw ranges; bounded protocol allocation | Reconciliation and aggregate storage probes show the measured improvements above |
| Native parity | Same input, version, identity policy and observable output; disposition every difference | Four selected AST projections match; most units still lack matched native runtime differentials |
| Release readiness | Real project persistence/session boundaries, composed dependency/notice closure and revalidated landing tree | These draft branches do not qualify a production release |

## Per unit assessment

The detailed source evidence, acceptance expectations and unqualified areas are in [audit-matrix.json](audit-matrix.json). A browser surface is not implied by an algorithm or adapter API. The individual auditor reports and embedded finding details are preserved snapshots; earlier publication-pending wording is historical. The coordinator dispositions and linked PRs provide the final remediation state.

| Unit | Scope | Kind | Assessment |
|---|---|---|---|
| F01 | Bind GeneralView planning to resource-scoped query results | Adapter | Caller visibility remains an observation, not native import qualification. |
| F02 | Freeze namespace import and exclusion contracts | Contract | Contract packet; no evaluator or UI behavior from this unit. |
| F03 | Evaluate public namespace imports on immutable snapshots | Adapter | Public imports only; native visibility binding remains open. |
| F04 | Decode and resolve selected EMF references without loading | Adapter | Explicit registry only; no URI loading or identity fallback. |
| F05 | Construct ViewUsage with an explicit GeneralView reference | Adapter | Selected JSON construction; absent from the default creation palette. |
| F08 | Implement immutable AST reflection type tables | Algorithm | Immutable reflection tables; native runtime closure is not established. |
| F09 | Implement ordered AST postprocessor dispatch | Algorithm | Ordered plans; callbacks and native ancestor ordering remain unqualified. |
| F10 | Convert AST primitive strings and numbers explicitly | Algorithm | Selected primitive conversions, not all AST value forms. |
| F11 | Construct ASTs for package and part declaration fixtures | Adapter | Four native class/name projections match; grammar gaps remain. |
| F12 | Apply OperatorExpression postprocessing to bounded fixtures | Algorithm | Structural operator postprocessing; expression text is outside scope. |
| F13 | Link local ElementReference chains in package fixtures | Adapter | 256-declaration boundary repaired in #58. |
| F14 | Import package and part text into one staged model | Adapter | Atomic finite import; explicit native JSON bridge added in #63. |
| F15 | Node label measurement and selected leaf geometry | Algorithm | Real DOM observations in the finite UI; selected leaf/note sizing only. |
| F16 | Deterministic flat directed layout | Algorithm | Own bounded layout algorithm; no ELK equivalence. |
| F17 | Finite orthogonal obstacle routing | Algorithm | Finite zero-clearance routing; ports and broad ELK behavior excluded. |
| F18 | Package-only nested container layout | Algorithm | Package-only containers and sibling edges; retained overlaps can remain. |
| F19 | Measured canvas composition, history and WASM binding | Browser | 32 nodes/64 edges; separate model/geometry histories; no worker. |
| F20 | REST read contracts and observed serialization | Contract | Typed read contract; no running REST host supplied by this unit. |
| F21 | Session/project isolation source packet | Contract | Source preparation; effective isolation/security policy remains unqualified. |
| F25 | Document/dependency reconciliation planner | Algorithm | Source semantics preserved; 10k no-op median improved 61.46 times. |
| F26 | Candidate Liquibase declaration graph | Contract | Declaration graph only; native schema/artifact correspondence open. |
| F27 | Local selected-document SQLite storage | Adapter | Real local SQLite candidate; no native PostgreSQL/schema qualification. |
| F31 | Immutable local snapshot publication faults | Adapter | Local publication fault boundaries, not crash/distributed recovery. |
| F33 | Round-trip selected cross-resource EMF JSON references | Adapter | Selected native wires; one-way selected-model bridge now explicit. |
| F34 | Export package and part textual forms | Adapter | Finite text export with actual F14 gate; wider serializer absent. |
| F36 | Inspector keyboard, focus and persistent live status | Browser | Keyboard/AX/live region evidence; not screen-reader speech qualification. |
| F38 | Immutable move-provider dispatch planner | Algorithm | One immutable provider category, not all extension points. |
| F39 | Reference execution preflight | Contract | Source preflight; this audit ran a separately identified release artifact. |
| F43 | Rust/WASM distribution content verifier | Algorithm | Frozen distribution checks, not current composed release clearance. |
| F44 | Qualify Java-compatible naming Unicode boundaries | Algorithm | Finite Unicode boundaries; no live Java differential in this audit. |
| F45 | Qualify resource-dependent identity allocation | Algorithm | Supplied identity context/provider, not full EMF resource behavior. |
| F46 | Expose semantic owner and qualified names through the native adapter | Algorithm | Selected owner/effective names; mostly not connected to browser flows. |
| F47 | Validate one package/part containment and typing rule family | Algorithm | Two explicit rules; CR/LF/CRLF locations repaired in #59. |

## Remaining work split into reviewable handoffs

These are feature gaps or measurement candidates, not extra claims of completed implementation.

| Follow-up PR family | Dependency and write boundary | Acceptance and commit shape |
|---|---|---|
| Connect import and validation to a user-facing workflow | F14, F33, F47 and fix 63; keep shared UI wiring integrator-owned | Import real user fixtures, show positioned errors, stage/cancel/commit and preserve the previous document; one behavior family per code/test commit |
| Reverse native-to-selected identity conversion | F04/F33 and fix 63; separate module and explicit source policy | Same-class collision tests, all IDs preserved, no guessing, then an actual native JSON to selected edit to native reload journey |
| File export and reload UX | F33/F34 and their admitted policies; separate from persistence host work | Real browser download/upload, encoding/size boundaries, cancellation and unchanged-input recovery; do not label copy-only JSON as a complete native file workflow |
| Lexical line comments | F11; ordered with other parser edits | Matched native fixtures; skip only native lexical comments and preserve source positions |
| Semantic block comments | Follow lexical/source contract review; AST and model construction in ordered PRs | Native block comments can be Comment nodes; retain their bodies and membership rather than discarding them |
| Untyped usages and usage bodies | Ordered F11 families, then F14/F34 integration | Freeze each native AST/model family, positive/negative boundaries and export/reload; avoid one broad grammar expansion PR |
| Durable projects and real command/history transport | F21/F27 then the existing F22/F23/F29/F30/F32/F37 dependency map | Choose and qualify the actual host/session/project boundary, persist/reopen, separate model/geometry history and failure recovery |
| Reference and query scaling | F04/F46/F33/F11 in disjoint module scopes after measured baselines | Profile 256/1024/4096-node workloads and source-position work before selecting an index/cache repair |
| Broader accessibility and usability | After the UI fixes land | Other browsers, hardware touch, real screen-reader speech, 200/400 percent zoom, contrast states and representative engineer task sessions |
| Native differential and release closure | Preserve F39 history; extend F40/F41/F42 with exact source/artifact inputs | Matched geometry, owner/name, identity, import/export and Vehicle journeys; regenerate actual distribution notices and dependency closure |

Parser families that write the same F11 implementation must be ordered. A reverse-reference bridge, host-independent measurement work and user-facing file UX can be prepared in parallel once their prerequisites and common-file owners are explicit. Existing held factory/model/graph paths remain untouched. Source-review candidates such as no-op model Undo clearing an unsubmitted form still need a dedicated browser reproduction before being promoted to a verified defect.

Nothing was merged, deployed, registered as a service or installed into the system baseline. Existing dirty work and held leases were preserved. All owned native reference processes were stopped; synthetic artifacts and original failures were retained on the verified workspace.
