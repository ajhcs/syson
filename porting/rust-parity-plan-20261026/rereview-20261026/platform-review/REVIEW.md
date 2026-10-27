# Platform and persistence re-review — 2026-10-26

This is an independent static re-review at the immutable heads in the 87-PR inventory. No reviewed code, compiler, test, application, browser or network operation was executed; no production source or plan was changed. The findings are future repair work. Existing PASS receipts retain their original bounded scope.

Six P2 findings are supported by concrete control flow. Runtime reproduction remains a verification task, so none is presented as a newly executed failure.

| Finding | Exact location | Trigger and consequence | Existing plan ownership |
| --- | --- | --- | --- |
| PLAT-01 — Bind observations to the actual database generation | #64 `6dc96e2cb4ef` `rust/crates/syson-storage/src/lib.rs:82` | Create two independent valid-format stores A and B with equal key and first revision. Load an observation from A, replace A pathname with B, then update/delete using the original A Store and observation. Store retains only pathname and Arc token. Every bridge invocation reopens the pathname; require() only checks Arc identity and SQL checks only key/revision. The replacement B row passes when revisions coincide. Existing foreign-handle and same-DB delete/recreate tests do not cover a replaced file. | B06 |
| PLAT-02 — Apply storage metadata bounds before read allocation and update cloning | #64 `6dc96e2cb4ef` `rust/crates/syson-storage/src/lib.rs:268` | Two independent cases: supply an oversized borrowed update name; or open an otherwise accepted-format candidate DB containing an oversized name inserted by an external writer (the exact DDL permits it), then load that row. Update copies the entire name into next.name before the 4096-byte check at 303. On reads the DDL does not constrain name length; fetchone materializes the full row, then json.dumps(...).encode() allocates the full output before the 8MiB comparison. The #64 request streaming repair remains effective for its measured document-batch workload, but these metadata paths bypass pre-allocation bounds. | B05, B06 |
| PLAT-03 — Refuse nonregular snapshot inputs before opening them | #32 `762217dd56e6` `rust/crates/syson-workflow/src/lib.rs:71` | In the caller-owned snapshot directory, an existing valid-key .emf.json path is a FIFO with no writer, or a symlink to another file. File::open executes before file-type admission. A FIFO blocks before take(LIMIT+1) can bound anything; symlink targets are followed. The key syntax prevents lexical traversal but does not establish the actual filesystem object type. | B06, C03 |
| PLAT-04 — Admit a regular composition manifest before opening it | #77 `9d244f04e76a` `rust/tools/verify-composition/src/lib.rs:20` | Invoke the composition CLI with an existing FIFO as its manifest path, without a writer. The public read_file helper opens the path immediately. CLI main.rs 5–7 calls it directly, so the selected-candidate regular-file guards never apply to the manifest. A byte cap cannot bound blocking File::open. | C03a |
| PLAT-05 — Make the general integration receipt bind the executed complete source closure | #19 `af4587008df8` `porting/run_integration_checks.py:21` | An input changes after the initial source hash scan but before/during a Cargo command; or a new crate/tool/include_str resource outside rust/crates and rust/validation is changed. The runner hashes rust/crates and rust/validation once before execution, never rechecks them, and sets PASS solely from command exit codes. Its fixed matrix covers five crates, validation and one WASM build; newer standalone tools and included porting resources are absent. The same runner bytes remain at integration ad814. New per-unit root captures contain stronger freezes, but do not repair this published general runner. | C02, C03 |
| PLAT-06 — Add a repository CI gate for the composed Rust source and tools | #19 `af4587008df8` `.github/workflows/build.yml:55` | Open or update a Rust-only parity PR: the repository workflow offers no Cargo/Rust verification of the changed implementation. The integration base workflow sets up Node/Java and runs npm/Maven; no Rust setup or Cargo stage exists in the checked workflow set, and none of the 87 inventoried PRs changes .github paths. The manually retained unit PASS receipts provide historical evidence only, without automatic regression protection for subsequent composition/retargeting. | C03, C04 |

The #64 aggregate serialization fix remains valid for its measured near-limit document batch: this review identifies separate metadata ingress/copy paths. PLAT-01 is distinct from the existing per-row stale-revision protection: two separately created stores can share a first revision while representing different database instances. PLAT-06 is an inherited repository integration gap, not a workflow defect introduced by #19.

## Future PR execution

Each repair should retain two focused commits: implementation/regressions, then exact verification evidence and independent disposition. Stay within 400 production / 900 authored lines; split by named boundary if needed. These are additions to existing plan units, not a new generic validation framework.

- PLAT-01 database identity, PLAT-03 snapshot admission and PLAT-04 composition manifest admission may proceed independently in disjoint source scopes.
- Serialize PLAT-02 against PLAT-01 because both touch storage; metadata bounds can be a separate small PR stacked after identity or coordinated as a single narrow boundary if budgets permit.
- PLAT-05 establishes the composed manifest/resource verification matrix and post-run freeze; PLAT-06 consumes that matrix in repository CI.
- Run fresh meaningful regressions under owned fixtures and external process/resource deadlines. Passed historical captures must not be replayed or relabeled as the new regression result.

The complete tasks, dependencies, trigger/expected/actual statements and verification gates are in `findings.json`.

## Product and parity boundaries

The newer tools mostly preserve honest boundaries. They are useful evidence and preparation, but do not implement the consuming product features:

- C01d captures 15 tables, 87 columns, 26 constraints, 16 indexes and 20 migration rows. B05/B06 still need a reviewed Rust PostgreSQL driver and actual atomic semantic/representation/watermark persistence. The SQLite candidate still invokes Python.
- C01e builds a selected seven-class / 114-SVG derived comparison artifact. It does not independently source-build the inherited dependency/application/frontend closure. Parent C01/C04 remain open.
- C01f mappings/probes and C01h template discovery are observations. B07/B09/B23/U03 still own the real host, context/security, template instantiation and connected UI journey. The actual C01h fixture retains four GraphQL errors in ten observations.
- C03a verifies selected/generated changed files. The root retained a separate 97-input composed compile freeze; that does not turn the selected-file verifier into a full compiler-input attestation.
- C02a compares supplied ordered JSON trees, with explicit incomplete MISMATCH at the reporting cap. It does not supply model/runtime observations or establish feature acceptance.

Native receipts were read at their committed heads. The audit confirmed preserved failure distinctions: C01d native-01 overall PARTIAL; C01c initial failure then later capture; C01e original-provider failures and 14-case remainder distinct from 26 derived cases; C03b failed first observation retained; C01f failed initial checks retained; C02a pointer-fix failure retained. No failure was promoted to a passing runtime claim here.

## Coverage and limits

`coverage.json` records 31 assigned PRs and 1,453 file identities (SHA256 from immutable Git blobs), excluding nested archived original-input trees. Hashing is explicitly not semantic audit coverage. Per-file depth distinguishes complete boundary reads, targeted reads, test-function scans, retained-receipt inspection and identity-only inventory.

| PR | Immutable head | Independent review depth / remaining limit |
| --- | --- | --- |
| #2 | `9246ef6e1382` | claim-boundary read; targeted source/provenance projection read; Source strategy/projection inspected; inherited/source scanner and every generated corpus row not independently reconstructed. |
| #3 | `fc920f3dacf0` | dependency/notices boundary read |
| #16 | `81b4cd04e4e0` | full snapshot adapter review |
| #18 | `81fc74e8572e` | targeted input/schema/source evidence admission review; Execution-backlog admission inspected; behavior/metamodel validators and all ledger rows require separate full consistency review. |
| #19 | `af4587008df8` | full capture runner review |
| #20 | `f3c584cb9255` | Inventory / delegated only; Historical inventories/checkpoints hashed; detailed claim and source-corpus cross-check not completed in this platform pass. Current acceptance gaps mapped from actual code and native receipts instead. |
| #21 | `b5be325bb288` | Inventory / delegated only; Historical inventories/checkpoints hashed; detailed claim and source-corpus cross-check not completed in this platform pass. Current acceptance gaps mapped from actual code and native receipts instead. |
| #24 | `b454d4991d25` | superseded baseline; final algorithm reviewed at #60 |
| #28 | `ae763874c419` | full supplied-bundle verification boundary review |
| #30 | `53dffb79ede9` | targeted DTO/raw-wire/controller/source-observation boundary review |
| #31 | `7200b3c7583e` | Inventory / delegated only; Domain/identity/import algorithms delegated to model/parser owners; platform records source boundary only. |
| #32 | `762217dd56e6` | full I/O publication and failure-semantics review; full load and save delegation review |
| #34 | `15501a5ff245` | Inventory / delegated only; Domain/identity/import algorithms delegated to model/parser owners; platform records source boundary only. |
| #38 | `fc15b4bdd436` | targeted XML include/control/data preservation review |
| #42 | `fa85317782c5` | targeted preparation-only source/command admission review |
| #46 | `ebc14e80f773` | source admission and unresolved policy boundary review |
| #53 | `b540273b901f` | full bridge schema/transaction/read review; full original store review; final superseding #64 also read |
| #60 | `a85366ae8252` | full identity/ordering/index/effects review |
| #64 | `6dc96e2cb4ef` | full final store and aggregate budget review; targeted transaction rollback/revision/foreign-handle/path/bounds tests review |
| #76 | `d592fe733249` | core policy review at superseding #78 |
| #77 | `9d244f04e76a` | full Git/source delta/generated exports/file/mode/bounds review; full manifest ingestion review |
| #78 | `a12d115f67d0` | full file/Git/hash/classpath inspection review; full pure policy core review |
| #79 | `beac96785d12` | targeted descriptor identity/FK/output bound checks; targeted selected axes and read-only query review |
| #80 | `604e6b6d929e` | targeted type/wrapper/member/root/error selected contract review |
| #81 | `7f111c24bb3f` | targeted preflight/private staging/namespace transport/independent versions/typed admission review |
| #82 | `606b169def89` | full RawValue/strict metadata/opaque payload/bounded envelope review |
| #83 | `9cb99468ca2d` | full mappings/fixed probe/wire preservation/admission review |
| #84 | `dfab0235208c` | targeted bounded trusted-observation planner scope and final MR-resource guard review |
| #85 | `df588dc53ca5` | full manifest/file/output review; full physical index identity domains/unique-key/depth/aggregate/index/walk bounds review |
| #86 | `e91c25633859` | targeted ordered tree/capped diff/pointer/refusal review |
| #87 | `a77137bb8c7d` | full observation/errors/pageInfo/context-bound control/selection/output/file review |

This is not a complete line-by-line audit of all 1,453 identified files. In particular, historical source scanner generated corpora (#2), every ledger consistency validator (#18), checkpoint/inventory claim cross-checks (#20/#21), and domain/import algorithms (#31/#34 delegated to their lane owners) are not independently closed by this platform pass. Test bodies, raw streams and historical source copies not individually marked reviewed remain inventory-only. The user’s whole-product audit must combine this with the model/parser/UI reviews and actual rendered/native verification.

No plan status, whole-artifact binding, full database parity, native performance, UI usability or production acceptance is granted by this review.
