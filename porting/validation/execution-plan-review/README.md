# Independent execution-plan gate review

`PLAN-VALIDATE-01` is complete for the bounded plan-checking contract. The
adapted validator passes 90 synthetic format tests normally and the same 90
under Python `-O`. The current 18-area, 72-item plan passes both modes. A
separate transition check against the explicitly supplied **unaccepted** prior
draft passes across five formerly held leases. This is not parity acceptance
or an execution authorization service.

## Frozen inputs and actual evidence

[The final receipt](final-run/receipt.json) records exact source hashes, Python
version, command arguments, actual UTC intervals, exit codes and raw/portable
stream hashes. The executed files were working-tree snapshots, subsequently
committed unchanged; the receipt does not relabel the old Git base as the
executed source revision.

Validator SHA-256:
`94605e837738ef1ef079026a21167d59104e4f0f9adfa1088497a990feac6418`.
Test file SHA-256:
`da19dadb72d986a51ddad87453bde4b9e2b28f917a34b49b70f7bb733c7998ad`.
[The checker-accepted input](final-run/accepted-plan.json) SHA-256 is
`bf5b82b7d34f968dd0bacbd43527ff8868134899716d38694f3b5afe9eb367c9`.
That filename means accepted by this format/consistency check, not published
or accepted by the coordinator. The root-owned plan was read, never edited.
Its bytes were checked unchanged throughout capture.

The normal test interval was `2026-10-22T23:24:35.700657+00:00` through
`2026-10-22T23:24:36.117274+00:00`; optimized tests ran through
`2026-10-22T23:24:36.641651+00:00`. Current-plan, optimized-current-plan and
explicit-draft transition checks then completed through
`2026-10-22T23:24:36.927364+00:00`. All five commands exited zero.

The checked plan has 2 ACTIVE, 38 BLOCKED, 14 DONE and 18 READY items, three
held writable leases, project compiler allocation zero, inventory incomplete
and zero parity-verified areas. DONE describes bounded item output; it does
not promote an area to parity or establish a feature-completion percentage.
The eighteen anchors are mixed-scope permanent work areas, not equal-sized
features.

`initial-schema-failure.txt` retains the actually observed rejection of added
metadata by the initial schema sketch. `initial-observations.json` records the
older observed 50-test result with its missing execution-source hash/UTC
explicitly null. The preserved partial checker and initial tests describe
that unfinished stage, not the final implementation. The old result is not
substituted for the fresh 90-test captures. The earlier 78/88 intermediate
checks likewise do not replace these final results.

## Checked contract

The gate requires all eighteen original anchor IDs in the area map,
denominator and pinned ESTABLISH event. It checks nonempty inventory/behavior,
source, test, integration, real-user and provenance plans; explicit exclusions
and unclosed scope; qualified status/evidence shapes; area/item references;
both dependency DAGs; and matching-area milestone/ready pointers. READY,
ACTIVE and DONE item dependencies must be DONE. Blockers identify action,
target, reason and one of the recognized pending/denied/uncertain/unavailable
states. Original IDs cannot silently disappear or be reused as a smaller
count. APPEND/SPLIT/RETIRE events deliberately fail closed until their future
schema/protocol extension receives review.

Leases bind owner, canonical repository-relative boundaries and explicit
acknowledgement. They have no automatic expiry. A BLOCKED item can retain its
ACTIVE lease. Different owners cannot hold overlapping file/directory paths;
text prefixes such as `foo` and `foobar` are distinguished. ACTIVE requires a
held lease; DONE requires a released record and bounded pinned completion
evidence. A schema string is not proof that an actual person granted or
released ownership.

The resource gate distinguishes global capacity from the project's current
allocation, accepts allocation zero, checks one job per process and rejects
compiler claims exceeding allocation or lacking an active matching item. It
creates no compiler process or resource entitlement. Unsupported scheduling
claims/workarounds cannot silently replace the recorded continuity limit.

Historical source evidence resolves to its exact Git blob even if today's
source evolves or disappears. Immutable execution/review/acceptance reports
must still exist inside the repository and match their declared published Git
bytes. Missing, mutable, empty or unpinned recorded reports fail. Pending and
NOT_RUN artifacts remain plans and cannot satisfy DONE or qualification.
BOUNDED implementation/Rust states require their corresponding recorded source
and passed native evidence. COMPLETE requires scoped inventory closure; parity
YES additionally requires closed inventory, no blockers and qualified source,
Rust, upstream, independent review, integration, real-user and license reports.

The structured proof fields are checked for internal consistency. They are
not independently authenticated runtime observations or a proof of complete
source inventory/full differential case equivalence. Generic schema tests use
public synthetic metadata, not product or oracle results. Actual qualification
still requires source assessment, real runs, scope matching and independent
acceptance through the existing evidence process. This gate does not grant
software, network, package, credential, service, browser or production access.

## Lease history and its limits

The CLI compares against a committed HEAD version of the plan when present.
For the initial unpublished plan there is none, and the captured default
checks explicitly report that gap. They do not invent an accepted predecessor.
The separately preserved interrupted draft is unaccepted; its explicit use
checks only lease transitions, not its earlier schema or execution authority.

`--previous-plan` permits an explicit prior snapshot. Held items cannot vanish,
lose their lease, change owner or change scope silently. A release requires a
new acknowledgement and nondecreasing parsed UTC. Reassignment follows a
recorded release snapshot. An unchanged blocked hold remains protected. A
single snapshot cannot establish whether an older ownership record was
removed; the prior-snapshot input and authentic acknowledgements remain
necessary. The comparison checks consistency, not the truth of asserted
human acknowledgements.

## Reproduction

From an integrated checkout, using the existing Python and a selected build
directory for synthetic test state:

```sh
SYSON_BUILD_DIR="${SYSON_BUILD_DIR}" PYTHONDONTWRITEBYTECODE=1 \
  python3 -m unittest discover -s rust/validation/tests -p test_execution_backlog.py
SYSON_BUILD_DIR="${SYSON_BUILD_DIR}" PYTHONDONTWRITEBYTECODE=1 \
  python3 -O -m unittest discover -s rust/validation/tests -p test_execution_backlog.py
PYTHONDONTWRITEBYTECODE=1 python3 rust/validation/scripts/check_execution_backlog.py
PYTHONDONTWRITEBYTECODE=1 python3 rust/validation/scripts/check_execution_backlog.py \
  --previous-plan "${PREVIOUS_PLAN}"
```

The validator contains zero authority-bearing `assert` statements. Negative
tests exercise malformed/duplicate JSON, denominator drift, cycles, unmet
dependencies, missing plans/references, expiry, held overlap, silent historical
lease deletion/reassignment, report tampering, source evolution, unsupported
completion/parity, scheduling claims and zero-allocation resource conflicts.
Checks remain active under `-O`. Only standard-library Python and read-only
Git inspection were used; no Cargo/compiler, upstream application, GUI,
network, administration or automation operation ran in this validation.
