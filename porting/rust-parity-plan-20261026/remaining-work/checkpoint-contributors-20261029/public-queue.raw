# Contributor tasks and the existing PR queue

Start with [CONTRIBUTING.md](CONTRIBUTING.md). Six small tasks are reserved for human contributors; no agent is implementing them. Comment on an issue and wait for @ajhcs to confirm its exclusive file scope. Existing implementation holds and shared-file ownership still apply.

| Issue | Outcome | Reserved new file |
| --- | --- | --- |
| [#196](https://github.com/ajhcs/syson/issues/196) | Runnable literal JSON preservation example | `rust/crates/syson-json-value/examples/literal_carrier.rs` |
| [#197](https://github.com/ajhcs/syson/issues/197) | Supplementary Unicode escape regression cases | `rust/crates/syson-json-value/tests/unicode_pairs.rs` |
| [#198](https://github.com/ajhcs/syson/issues/198) | Small bounded routing API example | `rust/crates/syson-routing/examples/two_nodes.rs` |
| [#199](https://github.com/ajhcs/syson/issues/199) | Synthetic measurement-host contract example | `rust/crates/syson-node-measurement/examples/fake_host.rs` |
| [#200](https://github.com/ajhcs/syson/issues/200) | Worked pinned-source evidence walkthrough | `doc/contributing/checking-source-evidence.md` |
| [#201](https://github.com/ajhcs/syson/issues/201) | Independent clean-checkout setup report | `doc/contributing/reproducibility/standalone-rust.md` |

The issue bodies give prerequisites, commands and acceptance criteria. The examples and new tests have not been implemented or executed. Existing baseline checks passed; they do not stand in for a contributor's new deliverable. Issues #200/#201 collect documentation and observations, not native-reference implementation or environment provisioning.

## How to read the older draft PRs

The [per-PR disposition register](porting/rust-parity-plan-20261026/remaining-work/checkpoint-contributors-20261029/pr-dispositions.json) gives every audited PR a maintainer contact, recorded prerequisite checkout and review status. Unverified implementation owners and additional prerequisites remain explicitly unverified.

The [pinned queue audit](porting/rust-parity-plan-20261026/remaining-work/checkpoint-contributors-20261029/pr-queue-audit.json) observed 110 open PRs at main `19aaf5db26a8ba43b64fced1bb7091baa221ca2b`: all were drafts and had no labels. Two targeted `main`, 26 targeted the old integration branch and 82 targeted other feature or snapshot branches. This is a dated inventory, not a live count or a review of every implementation.

Each PR's changed paths were compared with main by Git mode, object type and object ID. The [classification](porting/rust-parity-plan-20261026/remaining-work/checkpoint-contributors-20261029/queue-classification.json) accounts for all 110: 85 have every changed path absent from main, 23 have mixed path states and two have every changed path identical. A missing path does not establish readiness; identical paths do not establish feature acceptance.

| PR | Disposition and next action |
| --- | --- |
| [#35](https://github.com/ajhcs/syson/pull/35), [#90](https://github.com/ajhcs/syson/pull/90) | All 23/36 changed paths respectively match main. Closed as superseded by [#152](https://github.com/ajhcs/syson/pull/152)/[#128](https://github.com/ajhcs/syson/pull/128), with [#35 replacement comment](https://github.com/ajhcs/syson/pull/35#issuecomment-6509910746) and [#90 replacement comment](https://github.com/ajhcs/syson/pull/90#issuecomment-6509913869). Branches and evidence are retained; the [closure receipt](porting/rust-parity-plan-20261026/remaining-work/checkpoint-contributors-20261029/superseded-closeout.json) is separate from the original audit. |
| [#27](https://github.com/ajhcs/syson/pull/27), [#29](https://github.com/ajhcs/syson/pull/29), [#36](https://github.com/ajhcs/syson/pull/36), [#39](https://github.com/ajhcs/syson/pull/39) | Reconcile separately with the landed measurement/layout/routing/forest slices. Preserve unique donor source, tests and evidence. Their full PRs are not established as superseded. |
| [#130](https://github.com/ajhcs/syson/pull/130) | Frozen UI/history composition. Preserve its 12 registration lines and original evidence; actual-main prerequisites, owner handoffs and composition checks still block adoption. |
| [#148](https://github.com/ajhcs/syson/pull/148) | Diagnostic native/parser captures with unresolved differences. Its older body references #147; do not interpret it as current parser or native acceptance. |
| Other audited drafts | Retain for scope, dependency, provenance and owner review. No blanket closure, merge eligibility or implementation lease follows from this audit. |

The two targeted closures leave 108 drafts from that historical census; new contributor-preparation PRs are outside it. For an older PR, ask @ajhcs to identify the current unit, its dependencies, unique retained work, exact writable paths and relevant checks before changing it. Do not choose a draft solely because it has no assignee or label. The [current delivery queue](porting/rust-parity-plan-20261026/rereview-batch29-next-prs.md) and [remaining-work guide](porting/rust-parity-plan-20261026/REMAINING-WORK.md) distinguish merged finite libraries from the connected application and native/UI acceptance still to deliver.
