# Completion contract for SysON parity in Rust

A required feature is complete when a user can perform it through the Rust application, its results match the agreed SysON behavior, and its data and recovery behavior have been verified on the integrated candidate. The feature must have independently reviewed evidence for its required success, failure and boundary cases.

Version `SYSON-RUST-COMPLETION/1.0.9` locks the execution priorities and acceptance criteria for the current plan. The target remains SysON source `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`. Scope includes the features enabled in that target and their effective inherited services. Reviewed improvements may change the interaction while preserving the required semantic, reference, API and persistence outcomes.

At original adoption, main was `6e54ff44891bb7fd333c1e5fe985ba69abf3e27a`. The execution ledger contains 55 bounded implementation/evidence records: 54 draft PRs and one merged archival supplement. Foundation and planning merges are tracked separately. All 55 records remain `NOT_ACCEPTED`. The original plan assigned 778 review units, including generated batches and verification work. Amendments 1.0.2/1.0.3/1.0.4 add RR08a/RR08b, U10a/U10b and C09a/C09b, respectively. Amendment 1.0.5 adds C09b1/C09b2, bringing that assignment to 786. Amendment 1.0.6 adds U11a/U11b for 788 while preserving every original obligation. Amendment 1.0.7 adds U12a/U12b for 790 while preserving all 788 previous units. Amendment 1.0.9 adds B25S1 and M06a1 source prerequisites for 792 units, preserving every earlier obligation, parent gate and owner hold. These counts describe work and evidence; they do not establish a feature-completion percentage.

Use [execution.json](execution.json) for current implementation/publication/acceptance status. Lane records retain the adopted source obligations and baseline states. The [PR index](PR-INDEX.md) provides every unit's detailed brief. The [machine contract](completion-contract.json), [acceptance-packet schema](acceptance-packet.schema.json) and [hash lock](completion-contract.lock.json) bind these requirements to each brief. Read the execution order below, then dispatch only work whose own ownership and dependency gates permit it.

The previous native artifact is labeled `b380dada415ce05155150f23cb3b74ae21cc45e4`, with a 125-path delta from the target. Five selected Java source files matched in the latest comparison; that does not establish whole-runtime correspondence. C01 must close the remaining source/artifact/configuration gap before those observations can establish exact-source parity.

**1. Use distinct completion states**

| State | Required result |
| --- | --- |
| Assigned | The source obligation has a named unit, owner, dependencies and verification cases. |
| Locally qualified | The recorded implementation passes its stated checks on its recorded source; limitations remain explicit. |
| Integrated on main | Actual prerequisites have landed, shared changes are reconciled, and affected checks/callers pass on the landing composition. |
| Feature accepted | The required native comparisons, real caller workflows, data/failure behavior and applicable UI/resource checks are independently accepted on the common candidate. |
| Program accepted | Every required primary and discovered obligation is accepted, and the complete application and release artifacts pass final qualification. |

A direct helper, source-inspection tool or generated batch may use a shared consuming-workflow packet. Its evidence must identify that caller and dependency path. Inapplicable checks require a reason supported by source/configuration evidence. Missing implementation, an unavailable comparator, a held scope or an unrun case remains open.

**2. Execute the remaining work in this order**

Reference closure and independent source work can proceed in parallel. Authors use disjoint worktrees and write scopes; one coordinator owns shared registration and serializes builds, databases and browser/runtime qualification.

1. Close reference scope and qualify safe prerequisite landings. Review model foundation #8 together with #90/RR01, #91/RR02 and #94/RR04 before exposing downstream model-import consumers. Preserve literal JSON objects, numeric spelling, identity domains, cancellation and refusal behavior.
2. Reconcile #111/RR20 history budgets with #115/RR18b3a borrowed summaries on one candidate. Verify revision/rollback, redo invalidation, traversal and actual renderer callers. Obtain the explicit owner handoff and independent disposition required by M02 and overlapping raw/factory/initialized-graph work.
3. Deliver the connected project workflow below. Extend it through the required model, language, service, view and library families.
4. Complete the remaining repairs, interoperability, usability and resource qualification. Independent PRs may land whenever their own dependencies and ownership gates permit; the 54 drafts do not share one blanket blocker.
5. Freeze and qualify the final integrated candidate and release artifacts. Deployments and production installation remain separate actions.

**3. Close one connected project workflow first**

Ownership: B02/B03/B05/B06/B08/B23/B27/B28/B31 and U01/U03/U04/U05/U07, with their declared dependencies.

Create a project through the Rust UI, select a supported blank/template path, create/select a Package, edit its name, commit and save. Reload the page, close and reopen the project, and verify the committed result without manual JSON reimport. Qualify backend restart separately. Download the model/project, reopen it and compare its semantic and representation state.

Retain the exact long-name counterexample from batch13: `Long <&> 雪🧭 ` followed by 4096 `x` characters and ` END` (4113 UTF-16 units; 4117 UTF-8 bytes). The current Rust round-trip export is 4442 bytes. Native SysON 2026.9.3 (artifact label `b380dada415ce05155150f23cb3b74ae21cc45e4`) retained the name after UI save/reselect/page reload; Rust required explicit reimport. The 125-path target delta remains open under C01; selected matching source files do not close the whole artifact boundary. That file round trip remains useful evidence while automatic persistence is open. Native page reload alone does not prove database recovery after backend restart.

The workflow closes when the actual Rust UI uses the qualified host and storage path, and names, identities, references, representations, revisions and migration state survive their required lifecycle. Inject stale versions, SQL failure, cancellation, interruption around commit and acknowledgment loss. Successful acknowledgments must represent the required durable state. Failed saves must preserve the previous committed state and give truthful recovery. This first workflow is a milestone; the full persistence family retains its other cases.

**4. Close every required behavior family**

The following families define observable outcomes. The machine contract assigns every current unit to one primary family; source/caller dependencies and shared acceptance gates continue across family boundaries.

| Family and units | Remaining behavior | Evidence required to close it |
| --- | --- | --- |
| Reference: C01/C01a–h, C02/C02a, C09 | Match the native artifact to the target; inventory inherited Sirius/EMF/Syside behavior, active schema/routes/extensions, libraries, templates and migrations. | A complete required-obligation register, artifact/source/configuration correspondence, reproducible native fixtures, mismatch-negative controls and verified owned runtime lifecycle. Newly discovered reachable behavior receives a unit and cases. |
| Landing: C03/C03a–c | Land dependencies on actual main; reconcile shared code, preserve owner holds and qualify actual callers. | Exact source/base/landing identities, an independent review, affected checks and consuming workflows on the resulting tree. Temporary snapshot/feature bases remain evidence inputs, never merge destinations. |
| Model: M01–M48 (M39 lifecycle, M40 custom schema, M41–M48 derived/operation families), plus generated batches | Implement containment/inverse ownership; scalar/default/null/unset/enum behavior; identities; references/proxies; import/name visibility; constructors/initializers; derived values, caches, notifications, standard-library unload/reload/index lifecycle, custom-node schemas and validation. | Native/Rust results for every assigned symbol and effective helper, including ordering, URI/reference identity, stale/foreign handles, cyclic/unresolved graphs, cache invalidation and late failure. Failed mutations leave graph, indexes, cache epoch and event queue unchanged. Actual services use the implementations. |
| Language: P01–P184 | Complete lexical priority/comments/literals, CST/AST construction and recovery, grammar alternatives, linking/scopes, atomic text import, serialization, libraries, completion/hover/definition/reference/rename services, semantic tokens, formatting and live-editor updates. | Compatible AST/CST, linked meaning, references, spans and diagnostic IDs/severities/order for complete, malformed and incomplete documents. Cover CR/LF/CRLF, Unicode, keyword quoting and native UTF-16 rules. Export reparses with compatible meaning; unsupported required fields never silently disappear. Stale/cancelled or recovered input does not publish an invalid model as valid. |
| Interchange: I01–I09; B01/B09–B12/B25/B26 families; U06/U07 | Complete native codecs and identity bridges, capability-controlled external loading, archives and versioned migrations through real load/save paths. | Native→Rust→edit→save→native and reverse round trips; actual downloaded bytes and fresh-context reopen. Verify IDs, reference targets, containment order, URI spelling, opaque values and migration idempotence. Invalid encoding, duplicate keys/IDs, corruption, expansion limits and cancellation cannot partially publish state. |
| Projects: B05–B08/B23/B27/B28/B31; U03 | Connect project/template management, atomic persistence, snapshots, restore and recovery. | The connected workflow above plus semantic/representation/migration atomicity, truthful save acknowledgments, page/route lifecycle, backend restart and injected storage failures. |
| Services: B02–B04/B13–B17/B24; U08/U24/U26 | Complete active API/session/admission, commands/subscriptions, semantic history, library lifecycle and reconnect behavior. | Two clients in same/different projects; delayed/duplicate/reversed/stale responses; reconnect/disposal; queue exhaustion and cancellation. Verify correlation, isolation, ordering, undo grouping, redo invalidation and library identity/dependency integrity. Match the enabled native contract. |
| Views: remaining B18–B22/B29/B30/B35–B41 and U04/U05/U09–U25; V001–V044 | Finish explorer/search, forms/references, direct edits, menus, queries, diagrams, requirements and validation. Reconcile any active inherited model-browser/chart/deck/portal/studio surfaces. | Each active tool has correct preconditions, performs the real semantic/representation command, redraws correctly, supports cancellation/history and survives required reopen. Compare populated General, Interconnection, Action Flow and State Transition views, requirements grids, geometry and actual pointer/keyboard actions. Distinguish graphical deletion from semantic deletion. Disabled surfaces need evidence-backed exclusions. |
| Usability/capacity: U27–U29 | Finish responsive, accessible, readable and scalable user workflows. | Declared browser/OS/device/assistive-tech coverage; empty/loading/populated/error/read-only/pending states; reachable controls, preserved focus/caret, actual speech tasks, zoom/contrast/touch/glyph checks and measured resource ramps. Close the roughly 33000-CSS-pixel 512-child explorer with usable collapse/filter/windowing and stable selection. |
| Rust delivery: U01/U02, B07/B32/B33, C04 | Move handwritten browser application decisions and the Python SQLite bridge into Rust; qualify packages/processes/notices/upgrades. B07 also satisfies project-storage behavior. | Real events through Rust/WASM; observer/callback disposal; applicable clean native/WASM builds; packaged startup/shutdown/upgrade/rollback. Generated browser bindings/bootstrap are permitted. Shipped Java/Syside/Node/Python or handwritten-JS application fallbacks fail this target. Browser engines, PostgreSQL and declared lower-level libraries are recorded platform dependencies. |
| Repairs: RR01–RR24 and children | Adopt the existing bounded repairs through actual consumers and finish the open repairs below. | Every retained counterexample and consuming-caller obligation passes on the repaired composition. Parent acceptance waits for its declared children and implementation inputs. Earlier source results retain their original scope. |
| Verification: C05–C08, B34 | Compare language/model/query/API/database/migration/geometry/browser outcomes and strict-seed/Vehicle workflows. | Independent expected outcomes, original raw differences, deliberately divergent negative controls and equivalent public native/Rust paths. The complete task and saved outputs must agree under declared comparison rules. |
| Final acceptance: C99 | Accept the complete required scope and qualified release. | All required obligations and release gates below close on the same target/candidate closure, with no required unresolved, unimplemented, unsupported or unrun residual. |

The model inventory assigns 186 classifiers, 419 direct features and 70 declared operations. The construction-class subset contains 179 classes, including 171 concrete and eight abstract classes; seven enum classifiers sit outside that subset. Construction must verify concrete effects and abstract refusal, while enum behavior is qualified separately.

The language inventory assigns 735 unique grammar IDs across lexical/entry-root and grammar-family owners, 848 AST tuples and 82 export-dispatch entries. P26–P143 owns 709 of those unique grammar IDs and all 848 AST tuples; other lexical/entry-root owners cover the remaining 26 IDs. Six grammar IDs have split owners, and all their owners must qualify. P170 export helper/inherited closure remains required beyond the 82 dispatch rows. P172–P183 cover libraries and editor language services; P184 closes residual helpers and mixed callers. The 44 provider batches assign 175 provider sources. These are obligation inventories, with separate assigned, implemented, native-qualified and accepted dispositions.

**5. Finish the open repairs with their named counterexamples**

| Repair | Required completion case |
| --- | --- |
| RR09 | Bound graph indexing and owned-peer rescans on 256/1024/4096-node wide/deep graphs through actual query callers. Record work/copy counters, RSS and cancellation while preserving traversal and query results. Held-source admission remains required. |
| RR10, after RR09 | Use one resource-scoped iterative name authority. Qualify an 8192-reference chain, cycles, exact/over limits and cancellation through effective-name and qualified-name callers. |
| RR18b3d | Finish pre-copy/counting admission across complete HTML/navigation/options/SVG, scalar inspection, model/export/geometry and fallback paths. Under-1-MiB inputs that expand beyond 16 MiB must refuse before oversized materialization, preserve committed data/drafts/caret, and leave usable recovery. Measure allocation high-water behavior separately from request traffic or RSS. |
| RR19, after RR18/RR20 | Connect real command availability to applicable/disabled/busy controls. Exercise empty, staged, committed, undone, redone, exhausted and cancelled states with keyboard/AX checks. Users must understand availability without destructive trial actions. |
| RR16b | Qualify complete candidate, command, manifest/feature/target, generator, fixture, tool and external-input closure. Validate real Git/artifact identities, parsed receipt outcomes, obligation/case coverage and teardown. Source mutation, omitted inputs, stale generation, failed/unrun cases and an invalid candidate prevent acceptance. |
| RR17, after RR16 | Run the complete current-source Rust CI matrix. Negative cases must omit a required manifest/tool/test/input or substitute stale/failed evidence and be rejected. CI supplies evidence for independent feature acceptance. |

Existing RR18/RR18b/RR18b3 parents retain all child and whole-renderer obligations. A fixed scalar helper or bounded navigation label does not close complete renderer allocation, usability or capacity.

**6. Apply workload ranges and keep metric meanings separate**

[performance-policy.json](performance-policy.json) is authoritative. Its provisional event-to-visible target bands are 16–50 ms for continuous feedback, 50–150 ms for ordinary authoring, 150–350 ms for named dense authoring, and 250–1000 ms for named large import/export/recovery. Faster results are acceptable. Outliers require context, review and follow-up rather than an automatic timing veto.

Restore identical model, selection, draft and history state between repetitions. Record setup separately. New qualification uses two warmups plus seven measured samples with raw min/median/max; p95 needs at least 30 measured samples and a declared estimator. Compare equivalent native/Rust fixtures, actions, metrics and artifacts before making speed claims. Browser two-frame proxies, whole-operation latency, main-thread tasks, allocation requests, peak live memory and RSS remain separate measurements.

Resource-reviewed ramps must investigate native-documented capacity, including 10 MB models, 150000 semantic elements, 2000 representations per project, 1000 elements per representation, 1000 projects and 80 concurrent users. Those documented values are hypotheses to qualify, with definitions and applicable configuration recorded. Raising prototype limits requires measured bounded implementation, cancellation and recovery. Integrity/admission constraints remain mandatory.

**7. Require a complete acceptance packet**

[acceptance-packet.schema.json](acceptance-packet.schema.json) defines the packet structure. Each packet identifies its unit and contract, real Git candidate and target, built artifacts, native correspondence, source obligations, fixtures, commands/tool/input closure, parsed outcomes, raw comparisons and consuming caller evidence. It also records applicable UI/data/lifecycle/resource cases, independent review, intentional differences and every unresolved required item.

Every required source obligation maps to cases and evidence. All owners of a split obligation must qualify. A packet may reference shared caller evidence through its declared dependency path. A native-expected rejection, configuration exclusion or intentional improvement needs its explicit supported disposition; an unsupported Rust implementation or missing comparison remains open. Never hide a mismatch through permissive identity, numeric, ordering or diagnostic normalization.

The current completion checker validates planning consistency, labels and receipt hashes. This plan update adds static contract-integrity checks only. RR16b/RR17 still own stronger behavioral receipt/candidate/artifact/coverage enforcement. Neither schema validity nor a future zero-error index establishes that the observed behavior is correct. Independent final review must inspect the actual evidence and its limits.

**8. Accept and change the plan deliberately**

C99 closes when C01's scope is complete, all required primary/discovered obligations have accepted dispositions on one candidate, real application workflows and applicable failure/usability/resource matrices pass, and clean release rebuild/package/process/notices/upgrade evidence is accepted. Valid unchanged-component evidence may be reused with explicit unchanged-input proof; affected and composed workflows must apply to the final tree. Production deployment remains a separate decision.

The hash lock binds this contract, schema, performance policy, lane/overlay obligations and plan-generation/verification tools. Generators never refresh it. A source/gate/scope change requires a versioned reviewed amendment with the changed obligations, rationale, compatibility impact, new owners/dependencies/cases and preserved prior evidence. C01 discovery adds required units before coding. Progress and evidence updates belong in the execution ledger and do not release holds or rewrite old receipts.

Routine implementation choices within the adopted scope follow the existing autonomy and ownership rules. This adoption neither promotes a feature to accepted nor releases any raw/factory/initialized-graph hold.


**Amendment 1.0.1: narrow legacy prerequisite budgets**

[Amendment 1.0.1](amendments/1.0.1-budget-disposition.md) binds the exact donor source and rationale for F10 #25 (`48ee69e41c4f29346f1d95075278d84aefc48057`) at 420 handwritten production/900 authored lines and F11 #40 (`ca1c4b3ab364db7dc9da835c66cba72912c3417f`) at 460/900. Observed Rust implementation/tests are 409/372 and 447/358 respectively. F11's 173 generated keyword lines remain a separately reviewed generated block. The machine contract binds the amendment note by SHA-256. These two exceptions take effect only with independent review, explicit adoption and the refreshed versioned hash lock, before any F10/F11 execution on frozen composition inputs.

These are source review caps, with unchanged APIs and all 16 F10 and 13 F11 tests preserved. Any additional code or changed dependency input requires review in the exact composition, even below the caps; exceeding a cap requires another reviewed disposition or named split. The 778 units, 55 historical implementation/evidence records and every other budget, obligation, dependency, case, owner, hold and performance requirement remain unchanged. No native correspondence, resource/application support, executed result or acceptance state follows from this amendment. The actual F08/F10/F11/F13/F04/F14 finite-import path and declared downstream caller gates remain required; held graph consumers and F33's mandatory F05 GeneralView round trip retain their gates.


**Amendment 1.0.2: sequential RR08 repair children**

[Amendment 1.0.2](amendments/1.0.2-rr08-split.md) delegates RR08 kernel/test implementation to RR08a (selected copied-value admission, estimated 166 changed production lines) followed by RR08b (complete journal/atomic publication and finish-label charging, estimated 334). Both retain the ordinary 400 changed-production/900 authored caps; additions plus deletions count, and exact ordinary-formatted Git counts are required before execution. There is no RR08 budget exception.

RR08 remains the whole-repair integration/evidence owner, preserving its original task, checks and downstream consumers. Both child acceptances and all original parent obligations remain required: nine original regressions, current 37 AST/27 adapter cases, actual 100/1000/5000 construction/F12 workloads, real parser and operator callers, nested failure/cancellation/refusal, foreign handles, CST restoration and spent IDs. Sequential exclusive source leases and coordinator-only wiring remain mandatory. The amendment grants no source/execution lease, changes no held ownership and establishes no feature acceptance. All 55 historical execution records and prior evidence retain their original identities.


**Amendment 1.0.3: bounded flat-layout prerequisites**

[Amendment 1.0.3](amendments/1.0.3-layout-kernel-split.md) assigns U10a canonical flat-graph admission, then U10b additive SCC/rank/placement and all 15 original F16 tests. Each retains the ordinary 400 changed production/900 authored caps, with additions plus deletions and coordinator wiring counted. A further meaningful split is required before coding if the actual formatted scope exceeds either cap. No layout budget exception is added.

U10 retains its original source paths, U02/U09 dependencies, tasks and all effective native/ELK, real measurement→layout→routing, numeric provenance, identity/revision, worker, pin/port/container, routing, history, usability and capacity obligations. Its implementation additionally consumes U10b; acceptance additionally requires both children. The standalone finite kernel may qualify its typed API without claiming that the application or native geometry is complete. Both child results remain prerequisites, not parent acceptance. Existing holds, all 55 historical records, prior evidence, performance ranges and RR08 gates remain unchanged; the amendment grants no source or runtime lease.


**Amendment 1.0.4: strict comment-capture and comparison modules**

[Amendment 1.0.4](amendments/1.0.4-comment-comparison-split.md) assigns C09a strict P01 capture/schema/provenance admission, then C09b selected projection/CLI and all 35 original composed controls. The frozen 686-line prototype remains unexecuted preparation until reviewed partitioning and execution leases. Both children keep 400 changed production/900 authored caps; development-tool helpers, imports, CLI and shared wiring receive no evidence-directory or observer discount.

C09 retains its original paths/tasks, C01/C02 dependencies, normalized RR15 implementation and acceptance obligations, B05/B28/B32/B34 joint inputs and complete runtime/native/UI/teardown requirements. Its implementation additionally consumes C09b; its acceptance additionally requires both children. The finite tools preserve both15/25 corpus epochs, source/import pins, explicit unavailable/mismatch behavior and the receipt's absent-output-digest causality limitation. Neither hashes nor a selected comparison substitute for independent raw artifact/stdout review or parent feature acceptance. All 782 prior units, 55 historical objects, prior history, RR08/U10 policies, performance ranges and held scopes remain unchanged; no source or runtime lease follows from the amendment.

**Amendment 1.0.5: partial-field comparison and the complete control harness**

[Amendment 1.0.5](amendments/1.0.5-partial-comparison-split.md) assigns C09b1 fresh source-pinned raw-capture admission and field-specific comparison, followed by C09b2 checked kernel loading, CLI and all 35 original plus ten additional controls. Preserve known absent names and every available reference part/text/span when another field is unavailable. The rejected 400-line candidate and actual failures remain distinct evidence; no comparison result is accepted by this amendment.

Both children retain hard 400 changed production/900 authored caps, counting all helpers/imports/controls/wiring and additions plus deletions. C09b2 implementation follows reviewed C09b1 on main, with local qualification complete and its exact source pinned; full feature acceptance is not an implementation prerequisite. Parent C09b retains every original task/path/gate and additionally requires both accepted children; C09/C01/C02/RR15 and native/UI/runtime obligations remain unchanged. All 784 prior units, 55 historical objects, earlier policy objects, performance ranges and ownership holds are preserved. A plan merge grants no implementation or execution lease.

**Amendment 1.0.6: finite integer routing prerequisites**

[Amendment 1.0.6](amendments/1.0.6-routing-kernel-split.md) assigns U11a canonical routing admission, then U11b continued search/publication and all 16 original tests (439 unchanged lines). Each retains ordinary 400 changed-production/900 authored caps, counting additions, deletions and coordinator wiring. U11a follows actual landed U10b; U11b follows reviewed U11a on main, locally qualified and pinned. Both carry an opaque original work/cancellation continuation; full parent feature acceptance is not their source prerequisite.

U11 preserves its original U10 dependency and every native/fractional/measurement/caller/worker/history/UI/capacity obligation. Implementation additionally consumes U11b; acceptance requires both children. Integer geometry restrictions remain explicit. Forest and fractional numeric/caller adoption require separate scope review. No source/runtime lease, budget exception, ownership release or native/UI acceptance follows from this amendment; every prior policy, all 786 previous units and 55 historical records remain.

The [forest amendment](amendments/1.0.7-forest-kernel-split.md) separates finite forest admission and placement under ordinary 400/900 caps. U12 retains all original native/package UI gates; adopted children are not feature acceptance or a source/runtime lease.


**Amendment 1.0.8: team execution**

[Team adoption](amendments/1.0.8-team-execution.md) assigns all 790 units to seven portfolios and derives cross-team implementation/acceptance handoffs without changing any prerequisite, budget, hold or required behavior. [TEAM-EXECUTION.md](TEAM-EXECUTION.md) defines interfaces and the first durable workflow. Three authors plus one coordinator remain the execution ceiling; runtime execution remains serialized. Team ownership is not source admission or independent feature acceptance.
