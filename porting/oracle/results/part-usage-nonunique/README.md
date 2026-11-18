# Part usage nonuniqueness import evidence

Two retained SysON v2026.9.3 imports of typed `part … nonunique : …;` carry `isUnique: false`. The ordinary typed part omits the field. SysON's metamodel defaults `Feature.isUnique` to true, and its importer inverts the grammar's `isNonunique` flag. The upstream `isUniqueFeature/model.sysml` fixture has identical source bytes at the two pins in [receipt.json](receipt.json).

An off-tree Rust prototype based on `05a4be35d4842272d0249ab43b9e49295591ef1e` staged, saved and reopened all three typed models in a fresh process. Every normalized model matched its retained SysON capture. SysON adds an import Namespace and encodes references by resource ID. The Rust snapshots encode those references by semantic `elementId`.

A separate valid prefix also persisted and reopened. Appending a second nonunique usage typed by an attribute definition caused the Rust transaction to refuse after requesting 11 identities, with no snapshot, editor mutation or undo entry. This last control verifies the Rust failure boundary.

The prototype is outside the default importer and this evidence leaves #246 and #268 open. The campaign used retained reference captures. It records 105 earlier successes, 13 earlier refusals, 10 explicit-null checks and 62 public default cases with byte-identical output. The regression counts describe that separate executable campaign. The offline verifier checks the included models and refusal metadata.

## Verify the saved evidence

```sh
python3 -I -B porting/oracle/results/part-usage-nonunique/verify.py
python3 -I -B -O porting/oracle/results/part-usage-nonunique/verify.py
```
