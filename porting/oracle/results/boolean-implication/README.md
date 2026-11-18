# Boolean implication import evidence

This packet compares five typed SysML assignments using `implies` in the official SysON v2026.9.3 release and an isolated Rust prototype based on `05a4be35d4842272d0249ab43b9e49295591ef1e`. The prototype remains outside the default importer. Adopting the full-grammar path awaits the design decision in #246. The reference JAR, release source, and separate pinned grammar source are identified in [receipt.json](receipt.json).

| Source expression | What the saved model preserves |
| --- | --- |
| `false implies true` | A direct left operand and an ordered deferred right operand |
| `true implies false or true` | The entire `or` expression stays inside the deferred right operand |
| `true or false implies true` | The entire `or` expression stays in the direct left operand |
| `true implies false implies true` | The first implication is the left operand of the second |
| `not (1 < 2 implies "a" != "b")` | Comparison and string inequality trees remain operands under `not` |

The stored snapshots preserve the deferred `FeatureReferenceExpression` and `FeatureMembership` around each implication's right operand. The comparison reports the reference's import Namespace and resource-ID references separately from the Rust snapshots, which use semantic `elementId` references.

SysON's importer constructs models with numeric implication operands. The prototype checks for Boolean operands and refused both numeric orders without saving. The captured missing-operand input triggered syntax recovery in SysON and an atomic refusal in the prototype. A wrong typing target and an eight-operator chain also refused. A seven-operator chain reopened in the source campaign.

## Check the captured models

Run the offline verifier from the repository root with Python 3.

```sh
python3 -I -B porting/oracle/results/boolean-implication/verify.py
python3 -I -B -O porting/oracle/results/boolean-implication/verify.py
```

The verifier checks [cases.jsonl](cases.jsonl), including source hashes and UTF-8 and UTF-16 spans, saved snapshot hashes, full structural and wire comparisons, and refusal stdout. The source campaign performed the fresh-process save and reopen. The receipt also records byte-identical results for 95 earlier prototype positives, eight earlier refusals and 61 default importer cases. The offline verifier uses captured data and does not run either importer or evaluate implications.
