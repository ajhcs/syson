# Initialized construction continuation

ConnectionDefinition, InterfaceDefinition, AcceptActionUsage and PortDefinition can now be created through the Rust/WASM editor with their existing initialized child graphs. The caller stages the entire edit, previews it, cancels or commits once, and can undo/redo the entire construction. Export/import retains stable identities and the selected PortDefinition references. This extends the existing port; full SysON functionality remains the goal and is incomplete.

The capability is explicitly enabled with `--features raw-factory-candidate` on `syson-web` or `syson-workflow`. It remains disabled by default. Both browser builds were exercised. This gate preserves the existing candidate boundary; exposing a helper does not qualify complete native EMF state or upstream application behavior.

## Reconciliation

The checkout, origin tracking ref and live draft PR #1 initially agreed at `6aa4d2314278ab9472818432838240b9bb79ba87`. No reset or cleanup occurred. The previous integration chat reported idle, all other source worktrees were inspected, and no active Cargo/Java compiler was observed in the host process inventory. Idle status was not treated as release of any held lease.

Four unpublished, linear commits in `rust/interface-definition-01` were integrated with explicit cherry-picks. Their original commits, dirty ledger and untracked evidence remain in their worktree:

| Existing commit | Integration commit | Scope |
| --- | --- | --- |
| `0b516bfb6` | `046b4ce95` | InterfaceDefinition helper |
| `a852cb228` | `4750bab28` | AcceptActionUsage helper |
| `44524c35f` | `b54667dbf` | PortDefinition contract |
| `e5eeb1603` | `0062b3a0e` | PortDefinition helper and typed local references |

The 81-item integration working backlog and the newer 85-item interface working backlog are both preserved byte-for-byte, with hashes in [reconciliation.json](reconciliation.json). Both passed the existing backlog checker. Their item counts are coordination counts, not feature counts. The integration backlog retains stale ConnectionDefinition activity; the newer backlog records its explicit closeout and subsequent constructor closeouts. Its remaining root DOMAIN-QUERY inventory lease is docs-only. The historical factory validator lease was neither transferred nor released. No old frozen executor was replayed. The new source scope is limited to the workflow adapter, browser wiring, their tests and this checkpoint; this record reconciles status without rewriting either private working ledger.

New coherent implementation commits are `459f986c5` (shared workflow adapter) and `56790c366` (opt-in editor). Independent workers implemented the adapter and tests while the integrator handled UI composition. A separate reviewer checked the integrated source and actual receipts. It found an InterfaceDefinition entropy shortfall and eager identity consumption; both were corrected before the recorded test runs. Final static review found no remaining blocking defect. The reviewer did not execute the owner tests.

## Implemented and locally tested

| Evidence | Actual result | Scope |
| --- | --- | --- |
| [Focused receipt](focused-receipt.json) | 31 passing tests; candidate WASM build passed | Workflow 10, browser action kernel 21 |
| [Regression receipt](regression-receipt.json) | 219 passing tests; default WASM build passed | Model 45, graph 72, workflow 2, web 20, initializer 26, metamodel 11, domain 20, editor 13, persistence 10 |
| [Browser receipt](browser-receipt.json) | PASS; zero console errors | Real WASM actions; desktop 1365×1000, mobile 390×844 |

All 250 tests ran fresh, offline and locked, with one Cargo job and one test thread. No recorded source drift or timeout occurred. No registry versions or checksums changed; lockfile additions are existing local crates. One existing initializer atomic API deprecation warning remains. Parser/AST and unrelated historical suites were not rerun for a new timestamp.

The flow tested was: open the Rust editor → create a package → stage/cancel each new kind → create/commit each kind → undo/redo → export → import into a fresh document → confirm history reset. Assertions cover InterfaceDefinition's four ends (two PortUsage ends plus two inherited ReferenceUsage ends), AcceptActionUsage's ordered payload/receiver parameters, and PortDefinition's local conjugation references. Failed parent selection leaves committed data unchanged. Native tests additionally cover provider exhaustion, semantic/resource collisions, early and mid-constructor cancellation, lazy allocation, exact sibling naming, FeatureMembership selection and actual file-backed snapshot reload. These tests establish the named selected JSON adapter behavior, not complete native object semantics.

Browser page identity, nonblank rendering and interactions passed. Desktop screenshot inspection found the expected editor and native model nodes. Mobile horizontal overflow is explicitly asserted. Both screenshot viewports passed local inspection; keyboard/screen-reader acceptance and other browsers remain unqualified. Images and synthetic exported models are retained on the approved build volume, indexed by [evidence-index.json](evidence-index.json), and excluded from the public source package.

The Browser plugin was absent. Installed Playwright 1.64.1, Node 24.21.0 and Chromium 156.0.8078.4 were used with the installed library sysroot. Preliminary launch attempts exposed Node 18 incompatibility, an unset staged browser path, then a missing library path; these are recorded task outputs, not failed product assertions. No packages were installed. The first successful [browser run](browser-first-pass-receipt.json) remains preserved; the later run only improves screenshot framing. Tests served frozen artifacts at an ephemeral loopback port and closed the browser/server afterward.

## Upstream comparison and remaining scope

Pinned source remains `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`. The existing helper contracts and qualifications remain authoritative for their bounded source mapping. New membership selection follows `MetamodelMutationElementService.addChildInParent`; source hashes are in the reconciliation record. The outer membership/root allocation protocol and optional explicit name override are proposed Rust adapter behavior. They are not asserted to match upstream UI identity allocation or naming commands.

**Upstream runtime comparison: NOT_RUN. Upstream parity verified: false. Fully verified areas: zero of the unchanged 18 anchors; inventory remains incomplete.** Selected JSON writes omit native caches, flags/eIsSet, notifications, inverses/proxies and complete serializer behavior. Diagram layout/edges, database/API/collaboration, full history/recovery, full interchange and user acceptance remain open.

The application oracle is blocked by the existing separate build/runtime gates in [ORACLE-DECISION.md](../../ORACLE-DECISION.md): current PATH Java is 26.0.2.1 (workflow requires Java 21), Maven is absent from PATH, system Node/npm are 18.19.1/9.2.0 (application pins 24.20.0/11.19.0), artifact correspondence for `sirius.emfjson.version=2.5.5-SNAPSHOT` remains unresolved, package access is unverified, and an isolated synthetic database/reference launch has not been admitted. Existing Node 24.21.0 qualified this local browser test only. The default compose image is another release, so it is not a substitute oracle. No upstream application build, service, credentials, security settings, package baseline, merge or deployment was changed.

Next independent work is the existing DOMAIN-QUERY adapter inventory: distinguish raw containment traversal from semantic ownership/effective names, then implement the admitted native query bridge. The existing ordered query algorithms must be reused. ResourceSet/library resolution and ViewUsage remain blocked; do not infer their support from the constructors delivered here. No user decision is required to inspect that source and continue bounded local implementation; a full upstream launch still needs its reviewed action-specific preparation.

## Reproduce

After verifying Samsung workspace identity, use new output directories on that volume:

```sh
python3 -B porting/run_initialized_workflow_checks.py "$SYSON_CHECK_ROOT/focused" --phase focused
# Preserve target/wasm32-unknown-unknown/debug/syson_web.wasm as the candidate artifact.
python3 -B porting/run_initialized_workflow_checks.py "$SYSON_CHECK_ROOT/regression" --phase regression
```

The runner refuses an existing output directory. Its parent `target` directory is shared, so serialize invocations. Source/manifest/lock hashes, actual UTC intervals, command results and original stdout/stderr hashes are recorded. Public receipts are byte copies of the original JSON receipts; streams stay in their original run directories. These byte hashes do not independently prove upstream truth. Place the candidate/default WASM files beside the corresponding `index.html`, `loader.js` and `style.css` in separate preview directories. Run `rust/browser-tests/initialized_creation.cjs` with explicit `SYSON_PREVIEW_DIR`, `SYSON_DEFAULT_PREVIEW_DIR`, `SYSON_BROWSER_ARTIFACT_DIR`, and installed `SYSON_PLAYWRIGHT_MODULE`; use the existing runtime's browser/library paths and a workspace-backed `TMPDIR`. It starts and closes its own loopback test server. No persistent service is required.
