# Explicit null feature value import evidence

This packet compares five minimal typed assignments in the official SysON v2026.9.3 release with an isolated Rust prototype based on `05a4be35d4842272d0249ab43b9e49295591ef1e`. The Rust models were saved and reopened in a fresh process before the live reference capture. The prototype remains outside the default importer. Adoption is gated by #246. [receipt.json](receipt.json) records the exact source and runtime fingerprints.

| Source form | Model observation |
| --- | --- |
| `attribute n : N = null;` | A `FeatureValue` owns a `NullExpression` with identity and no scalar field |
| `attribute n : N = ();` | The same saved model as keyword `null`, with a separate source span |
| `attribute n : N default := null;` | The null node is retained with both assignment flags |
| `attribute n : N;` | Typing is present and the `FeatureValue` is absent |
| `attribute n : N = "null";` | The value is a `LiteralString` containing four characters |

The retained Batmobile capture contains the same null model shape for the `driver = null` assignment. Its source bytes match both upstream pins. The full model produced conversion diagnostics. The five fresh comparisons use clean reduced models and preserve the known wire differences. SysON adds an import Namespace and uses resource IDs in links. The Rust snapshots use semantic `elementId` references.

A valid null assignment followed by `null + 1` refused in the Rust prototype without leaving a snapshot. SysON constructed a model with ordered null and integer operands. The evidence concerns model construction and persistence. It does not establish evaluation behavior.

## Check the captured models

```sh
python3 -I -B porting/oracle/results/explicit-null/verify.py
python3 -I -B -O porting/oracle/results/explicit-null/verify.py
```

The offline check recomputes structural and wire comparisons and checks source spans, identity roles, stored snapshot bytes and the refusal record. The receipt also records byte-identical results for 105 earlier accepted cases, 13 prior refusals and all 62 default importer cases. These replays used the exact existing binaries in the isolated campaign.
