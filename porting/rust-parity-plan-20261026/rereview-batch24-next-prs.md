# Next PRs after landed graph and capture admission

Snapshot `SYSON-RUST-REMAINING/2026-10-29.13` uses main `6d9b151671692b3b569179d73e290d9e6efa5ffe`, with #171 U10a and #172 C09a landed. Contract 1.0.5 preserves all 784 prior units and adds C09b1/C09b2, for 786 review units. The [guide](REMAINING-WORK.md) and [checkpoint](remaining-work/checkpoint-batch24/status.json) distinguish landing, local qualification, rejection and acceptance.

| Separate PR | Readiness and work | Verification gate |
| --- | --- | --- |
| [U10b placement](prs/U10b.md) | U10a is landed. U10b has 26 locally qualified checks plus formatting, Clippy and WASM checks; publication is pending. | Preserve all 15 original tests (572 lines), continued guard, resource ranges and exact-main checks. Native/UI/worker/capacity remain OPEN. |
| [C09b1 kernel](prs/C09b1.md) | After contract 1.0.5 is adopted and a source lease is granted, use actual pinned C09a per raw-input call; compare every available native field and known absent name. | Both epochs, actual admission and compound/missing-only checks, independent review and 400/900 counts. Context projection private. |
| [C09b2 composed CLI](prs/C09b2.md) | After C09b1 has been reviewed, landed on main and locally qualified, pin exact kernel bytes; retain CLI, all 35 original plus ten controls and cloned root ranges. | Normal/optimized composition; the old 15-case epoch must produce two differences (exit 1); the final 25-case epoch must produce zero selected differences (exit 0) with six refusal/coordinate/getter limits OPEN. Inspect raw causality separately. |
| C01/reference/caller preparation | Existing C01/C02/RR15/parser/ownership prerequisites remain; acquire tools, inspect launchers, review dependencies/builds and admit native runtime as separate gates. | Exact target source/artifact closure, bounded owned lifecycle and matched native evidence; preparation is not parity acceptance. |

The 400-line C09b candidate is rejected: two reproduced false successes and a root-control schema failure remain immutable evidence. The original 686-line prototype remains unexecuted; no prior result is transferred to a new hash. Children retain ordinary 400 production/900 authored caps with all helpers/imports/controls/wiring counted; no exemption or missing-control shortcut applies.

Use disjoint source leases; root owns shared wiring and serialized subject execution. Performance uses workload-specific ranges, refusal latency distinct from successful capacity, and separate compute, UI and memory metrics, and at least 30 samples for p95. Full native/UI/usability/durability/Rust delivery and parent acceptance gates remain OPEN.
