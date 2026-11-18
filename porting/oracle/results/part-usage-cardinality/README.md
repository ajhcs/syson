# Finite cardinality on typed part usages

The retained SysON v2026.9.3 import of `part w : W[4];` contains one `MultiplicityRange` before the feature typing relationship. The range contains one `LiteralInteger` with value 4. Each uses its own `OwningMembership`, for four new nodes in all. The retained Batmobile model has the same shape for `part wheels : Wheel [4];`. Its source matches at both upstream pins. The full Batmobile capture has conversion diagnostics, so only the wheels projection is compared here.

The isolated Rust prototype produced a 12-node normalized match for the reduced source after save and fresh-process reopen. SysON adds an import Namespace and links by resource ID. Rust links by semantic `elementId`.

Four additional Rust controls reopened the zero bound, largest nonnegative EInt, omitted bound, and a valid prefix used in the atomicity check. Zero uses the integer literal's default wire value. The prototype refused a signed negative, overflow, rational, arithmetic, a range, and an unlimited bound. Adding a usage typed by an attribute definition after a valid bounded usage refused after 17 identity requests without a snapshot, editor change, or undo entry. The local controls establish Rust behavior for this slice. The retained reference comparison covers `[4]`.

The evidence covers the off-tree prototype. Default importer adoption and the remaining multiplicity forms stay open in #246 and #268. The prior and default importer replay checked 195 outputs byte for byte. [receipt.json](receipt.json) records the exact fingerprints and counts.

## Verify the captured evidence

```sh
python3 -I -B porting/oracle/results/part-usage-cardinality/verify.py
python3 -I -B -O porting/oracle/results/part-usage-cardinality/verify.py
```
