# Unlimited cardinality on typed part usages

The retained SysON v2026.9.3 Batmobile capture contains `part systems : System [*];`. Its part usage owns a multiplicity membership before the typing relationship to `System`. The range holds one `LiteralInfinity` through a second owning membership, with no numeric field. SysON's getter derives a lower bound of zero from that single node. It does not store a zero node.

The off-tree Rust prototype staged, saved, and reopened the same projected shape in three reduced fixtures. The receipts check the four paired identities and the exact `*` source span. The full Batmobile capture has conversion diagnostics. A clean direct comparison of the reduced fixture remains open. The first fresh reference attempt hit PostgreSQL's socket path limit. The short-path retry stopped on host CPU pressure before capturing the model. Both scopes shut down.

A separate late wrong-type control refused after 17 identity requests with no persisted snapshot, editor change, or undo entry. Signed star, mixed range, and star as a feature assignment retained their previous refusals. Binary multiplication was byte-identical, and all 206 earlier and default importer output pairs matched. The one intended change was a previously refused typed `[*]` part usage.

This draft records the scoped prototype and its retained reference projection. Default importer adoption and the rest of #268 remain open under #246. [receipt.json](receipt.json) identifies the exact pins and campaign artifacts.

## Verify the captured evidence

```sh
python3 -I -B porting/oracle/results/part-usage-unlimited/verify.py
python3 -I -B -O porting/oracle/results/part-usage-unlimited/verify.py
```
