# SysON PR split and parallel work handoffs

The original implementation is split into **20 open draft PRs**, with **47 planned follow-on PR briefs**. This index changes no implementation or parity status. The future units are proposals, not open implementation PRs, assignments, compiler allocations or leases.

Frozen rollup: [PR #1](https://github.com/ajhcs/syson/pull/1), source `ad814c232ee24109639c1d52239cbeec6f4a3e92`, upstream `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`. The rollup and its original commits remain intact. Existing dirty/untracked work, branch heads and historical evidence are not swept into this split.

## Existing implementation PRs

Review these components independently. Some PRs contain large immutable source inventories, field tables or original receipts; their size is not a feature-completion measure. The 7,728-line raw factory candidate is deliberately isolated in S08 and stays default off, with independent acceptance pending. It is a review-size exception, not the template for future work.

| Unit | Draft PR | Prerequisites | Changed files | Source/script lines |
| --- | --- | --- | ---: | ---: |
| S01 | [Pinned source and dependency evidence](https://github.com/ajhcs/syson/pull/2) | None | 78 | 774 |
| S02 | [Rust dependency notices and byte-preservation settings](https://github.com/ajhcs/syson/pull/3) | None | 15 | 0 |
| S03 | [Domain naming, identities and shared query algorithms](https://github.com/ajhcs/syson/pull/4) | S02 | 10 | 1267 |
| S04 | [Diagram rotation and bounded geometry algorithms](https://github.com/ajhcs/syson/pull/5) | S02 | 6 | 490 |
| S05 | [Standard-library ID migration algorithms](https://github.com/ajhcs/syson/pull/6) | S02 | 6 | 248 |
| S06 | [Static metamodel registry and generator](https://github.com/ajhcs/syson/pull/7) | S02 | 9 | 978 |
| S07 | [Transactional model editing and typed scalar capabilities](https://github.com/ajhcs/syson/pull/8) | S03, S06 | 15 | 1584 |
| S08 | [Default-off raw factory and selected graph admission](https://github.com/ajhcs/syson/pull/9) | S07 | 36 | 9026 |
| S09 | [Grammar-driven syntax recognizer and independent regressions](https://github.com/ajhcs/syson/pull/10) | S01, S02 | 31 | 1999 |
| S10 | [Transactional AST operation kernel and independent regressions](https://github.com/ajhcs/syson/pull/11) | S01, S02 | 50 | 1660 |
| S11 | [Parser reference corpus and native observation tooling](https://github.com/ajhcs/syson/pull/12) | S09 | 109 | 368 |
| S12 | [Ordered declarative initializer planner](https://github.com/ajhcs/syson/pull/13) | S06 | 33 | 963 |
| S13 | [Atomic initialized graph construction](https://github.com/ajhcs/syson/pull/14) | S08, S12 | 96 | 2532 |
| S14 | [Resource-scoped native query adapter](https://github.com/ajhcs/syson/pull/15) | S13 | 13 | 1517 |
| S15 | [Connected workflow and immutable local snapshots](https://github.com/ajhcs/syson/pull/16) | S04, S05, S14 | 7 | 889 |
| S16 | [Rust WASM authoring UI and browser checks](https://github.com/ajhcs/syson/pull/17) | S15 | 20 | 1867 |
| S17 | [Behavior and execution-backlog integrity gates](https://github.com/ajhcs/syson/pull/18) | S01 | 74 | 1873 |
| S18 | [Integrated independent Rust regression harness](https://github.com/ajhcs/syson/pull/19) | S10, S11, S16, S17 | 15 | 1196 |
| S19 | [Backend, history, recovery and interchange source inventories](https://github.com/ajhcs/syson/pull/20) | S01 | 18 | 0 |
| S20 | [Historical integration checkpoints and receipt archive](https://github.com/ajhcs/syson/pull/21) | S18, S19 | 47 | 104 |

Source/script lines include tests and declarative Rust tables; they are neither handwritten-production counts nor review estimates. Shared files change only in explicit stacks: S07 → S08 for the model, and S13 → S14 for the query adapter. Both start from coherent historical source snapshots, preserving exact bytes.

## Merge and branch protocol

1. Keep PR #1 as the preserved rollup/evidence record. Review and eventually merge the component PRs, not both the rollup and its duplicate component changes. No merge is performed or authorized by this preparation.
2. S01 and S02 target main. The code foundation PRs share S02 for notices/configuration and can be reviewed in parallel. Merge prerequisites before dependents.
3. A PR with one prerequisite targets that prerequisite branch. After the prerequisite lands, retarget the dependent PR to main and inspect its new diff.
4. A PR with multiple prerequisites targets `codex/syson-base-<unit>-20261024`. This is an immutable dependency snapshot, not a merge destination. Once all prerequisites land, retarget that PR to main. Keep these PRs draft until this check is complete.
5. Before retargeting or implementation, inspect the actual base/head and active owners again. Do not force-push existing writer branches, infer lease release, activate old frozen executors or treat inactivity as permission.
6. Reconcile each branch against the current merged base and run the checks appropriate to its final diff under an allocated resource slot. Historical passing receipts remain attributed to their original executed revisions; they do not qualify new split SHAs.

Temporary bases contain the exact prerequisite blobs with a single synthetic snapshot parent. They do not integrate into or rewrite rust/integration. [published-prs.json](published-prs.json) records every head, base and tree identity.

## Parallel follow-on work

Each [brief](briefs/) specifies one behavior family, acceptance cases, excluded scope, a proposed path boundary and dependencies. Aim for at most 400 changed handwritten production lines and 900 authored lines including tests. Split a unit further if source closure exceeds that budget. Keep generated declarations, implementation logic and runtime captures distinguishable during review.

Recommended independent first tranche after source/lease checks: **F01** scoped GeneralView binding, **F08** reflection tables, **F10** primitive conversions, **F15** node measurement, **F20** API read contracts, **F25** document reconciliation, **F36** inspector accessibility, **F39** reference preflight and **F43** distribution notices. The source base already contains their foundation code; waiting for all split PRs to merge is not a technical requirement.

**F06 is held by the existing factory validator.** It is not available for a replacement writer. Other dependency-free units may still need source-contract preparation, admitted dependencies, exact execution review or an explicit lease. Existing resource allocation remains authoritative.

Implementation can run on disjoint modules/crates. The integrator alone wires shared lib.rs exports, Cargo manifests/locks, browser entry points and the execution backlog. A worker proposes a frozen API and requests that wiring; it does not edit those shared files concurrently. Branch independence does not grant compiler/display capacity.

### Dependency wave 0

| Planned PR | Proposed lane | Future prerequisites |
| --- | --- | --- |
| [F01: Bind GeneralView planning to resource-scoped query results](briefs/F01.md) | query | None; gates still apply |
| [F02: Freeze namespace import and exclusion contracts](briefs/F02.md) | query | None; gates still apply |
| [F04: Decode and resolve selected EMF references without loading](briefs/F04.md) | references | None; gates still apply |
| [F06: Qualify raw factory defaults and selected admission](briefs/F06.md) | model-review | None; gates still apply |
| [F08: Implement immutable AST reflection type tables](briefs/F08.md) | parser-reflection | None; gates still apply |
| [F10: Convert AST primitive strings and numbers explicitly](briefs/F10.md) | parser-values | None; gates still apply |
| [F15: Measure package, view-frame and note nodes through a typed host boundary](briefs/F15.md) | geometry | None; gates still apply |
| [F16: Layout a flat directed graph of measured rectangular nodes](briefs/F16.md) | layout | None; gates still apply |
| [F20: Freeze REST read DTOs and error contracts](briefs/F20.md) | api-read | None; gates still apply |
| [F21: Close authentication, session and project-isolation source contracts](briefs/F21.md) | api-security-contract | None; gates still apply |
| [F25: Plan document reconciliation as a pure function](briefs/F25.md) | storage-planning | None; gates still apply |
| [F26: Freeze the resolved schema and synthetic migration sequence](briefs/F26.md) | storage-schema | None; gates still apply |
| [F31: Inject failures into isolated immutable snapshot publication](briefs/F31.md) | recovery-local | None; gates still apply |
| [F36: Qualify keyboard and screen-reader use of the property inspector](briefs/F36.md) | accessibility | None; gates still apply |
| [F38: Implement a typed extension registry with deterministic precedence](briefs/F38.md) | extensions | None; gates still apply |
| [F39: Prepare an immutable upstream build and launch preflight](briefs/F39.md) | reference-preparation | None; gates still apply |
| [F43: Verify Rust/WASM distribution dependency and notice closure](briefs/F43.md) | dependencies | None; gates still apply |
| [F44: Qualify Java-compatible naming Unicode boundaries](briefs/F44.md) | names | None; gates still apply |
| [F45: Qualify resource-dependent identity allocation](briefs/F45.md) | identities | None; gates still apply |

### Dependency wave 1

| Planned PR | Proposed lane | Future prerequisites |
| --- | --- | --- |
| [F03: Evaluate public namespace imports on immutable snapshots](briefs/F03.md) | query | F02 |
| [F05: Construct ViewUsage with an explicit GeneralView reference](briefs/F05.md) | model | F01, F04 |
| [F07: Add initialized ConnectionUsage and InterfaceUsage construction](briefs/F07.md) | model | F06 |
| [F09: Implement ordered AST postprocessor dispatch](briefs/F09.md) | parser-dispatch | F08 |
| [F11: Construct ASTs for package and part declaration fixtures](briefs/F11.md) | parser-construction | F08, F10 |
| [F17: Route flat-graph orthogonal edges with explicit constraints](briefs/F17.md) | routing | F16 |
| [F18: Layout nested package containers](briefs/F18.md) | layout | F15, F16 |
| [F27: Persist and reload selected JSON documents transactionally](briefs/F27.md) | storage | F04, F25, F26 |
| [F40: Build and capture the isolated pinned SysON reference](briefs/F40.md) | reference-execution | F39 |

### Dependency wave 2

| Planned PR | Proposed lane | Future prerequisites |
| --- | --- | --- |
| [F12: Apply OperatorExpression postprocessing to bounded fixtures](briefs/F12.md) | parser-construction | F09, F11 |
| [F13: Link local ElementReference chains in package fixtures](briefs/F13.md) | parser-linking | F04, F11 |
| [F19: Integrate async layout, rotation and stale-result cancellation](briefs/F19.md) | canvas | F15, F17, F18 |
| [F22: Serve the two admitted REST read operations](briefs/F22.md) | api-read | F20, F21, F27 |
| [F23: Dispatch bounded editing commands with cancellation](briefs/F23.md) | api-commands | F21, F27 |
| [F28: Apply the two existing library-ID migrations in a versioned pipeline](briefs/F28.md) | migration | F27 |
| [F29: Implement input-addressed semantic undo and redo](briefs/F29.md) | history | F27 |
| [F33: Round-trip selected cross-resource EMF JSON references](briefs/F33.md) | interchange-json | F04, F05 |
| [F41: Compare the three strict seed behavior fixtures](briefs/F41.md) | differential-validation | F40 |
| [F46: Expose semantic owner and qualified names through the native adapter](briefs/F46.md) | model-operations | F03, F04 |

### Dependency wave 3

| Planned PR | Proposed lane | Future prerequisites |
| --- | --- | --- |
| [F14: Import package and part text into one staged model](briefs/F14.md) | text-import | F11, F13 |
| [F24: Implement bounded subscription session lifecycle](briefs/F24.md) | api-subscriptions | F23 |
| [F30: Integrate client history stacks with command responses](briefs/F30.md) | frontend-history | F23, F29 |
| [F32: Qualify success acknowledgement against durable commit](briefs/F32.md) | recovery-storage | F23, F27 |
| [F35: Read and write a selected project archive manifest](briefs/F35.md) | interchange-archive | F33 |
| [F37: Load projects with explicit cancel, error and retry states](briefs/F37.md) | frontend-projects | F22, F23 |

### Dependency wave 4

| Planned PR | Proposed lane | Future prerequisites |
| --- | --- | --- |
| [F34: Export package and part textual forms](briefs/F34.md) | interchange-text | F14 |
| [F42: Compare one complete Vehicle authoring journey](briefs/F42.md) | acceptance | F14, F19, F30, F35, F37, F41 |
| [F47: Validate one package/part containment and typing rule family](briefs/F47.md) | semantic-validation | F14, F46 |

## Scope remaining after these units

The 18 permanent area anchors remain unchanged and incomplete. These 47 units initiate or advance each area; they are not an assertion that all remaining features fit in 47 PRs. Full constructor families, the 70 operations, all grammar/AST obligations, every diagram/renderer, authenticated API surface, database migrations, complete interchange, collaboration, accessibility and installed-user acceptance require additional source-enumerated sibling PRs.

After each finite family, preserve unsupported dispositions and add the next source-backed family. Do not silently expand a PR into a whole parser, backend or graphical-editor rewrite. The authoritative execution-backlog.json and held leases remain untouched by this proposed handoff index.

## Verification

- All 683 changed files at the frozen source are reconstructed exactly, including modes and blob identities.
- All 20 published split tree hashes were independently recomputed from Git tree encoding.
- 102 Rust path dependency references and 68 literal included-file references exist in their respective split dependency closures.
- All 20 remote PRs were read back as open drafts with the expected head and base.
- The 47-unit future graph is acyclic, traces all 18 area anchors and has no proposed write-scope overlap between unordered units.
- No Rust, browser or upstream runtime test was rerun; no existing receipt is requalified.

[Structural result](verification.json) · [Remote PR verification](remote-verification.json) · [Exact split manifest](existing-split.json) · [Future units](future-prs.json)

Reproduce the local structural checks from a checkout containing the pinned Git objects:

```sh
python3 -B porting/pr-split-20261024/prepare.py
python3 -B porting/pr-split-20261024/future.py
python3 -B porting/pr-split-20261024/verify.py
python3 -B porting/pr-split-20261024/index.py
```

The scripts read the immutable pinned Git objects independently of the current HEAD. All outputs stay on the verified workspace; these scripts do not compile, launch services, alter Git refs/index or update leases.
