# Next PRs: durable Rust snapshot storage

This queue follows contract 1.0.10. The [earlier queue](rereview-batch33-next-prs.md) remains historical; registration grants no source lease or feature acceptance. Recheck actual prerequisite merges and exclusive paths before dispatch.

| Scope | Deliverable and gate |
| --- | --- |
| [RR13a](prs/RR13a.md) | After this registration lands and RR04/ownership checks pass, adopt the complete regular SnapshotStore/SnapshotIo APIs and all 13 original cases. Retain the full donor; defer only the seven held raw-factory declarations. The planned 337 production/896 authored lines leave four authored lines: split necessary expansion before execution. |
| [RR13b](prs/RR13b.md) | After RR13a lands, adopt all 13 original fault cases. Run the full 26-case composition, actual permission controls and save/load probes with two warmups/seven raw samples. Report ranges; keep physical power-loss and native/UI durability open. |
| M06 integration | Preserve the entire original 166-line identity/SnapshotStore integration. Register the complete M06a2 proposal separately before dispatch; M06a1 and the parent native/library gates remain unchanged. |
| Reference and semantics | Continue admitted C01 dependency/build work and independently ready semantic scopes. B25d still needs a meaningful registered split; C01/C02-gated parser work gains no implementation permission here. |

The product milestone remains create/import → edit → save → reload → close/reopen → backend restart with semantic, representation, identity, reference and migration state intact. Keep #130, its 12 held registrations, existing owners and human issues #196–#201 preserved.
