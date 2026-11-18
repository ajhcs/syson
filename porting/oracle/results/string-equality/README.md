# String equality import evidence

This packet records how the official SysON v2026.9.3 release and an isolated Rust prototype represented typed SysML string comparisons. The prototype was based on commit `05a4be35d4842272d0249ab43b9e49295591ef1e`. Its importer path remains outside the default Rust application and subject to the design decision in #246. The reference JAR and its upstream source revision are identified in [receipt.json](receipt.json). The release and the pinned grammar source have different revisions.

| Case | What matched after reopening the Rust model |
| --- | --- |
| Plain and empty strings | Ordered `LiteralString` operands for `==` and `!=`, including the explicit empty default value |
| Unicode | Composed and decomposed operands stay distinct in the model |
| Backslashes | Original source spellings stay distinct and both lowered values match SysON |
| Concatenation | The ordered `+` operand remains nested under equality |
| Logical composition | The right-hand comparison stays inside the deferred `and` operand |
| Conditional operand | The string-valued conditional retains both deferred branches under inequality |

The seven fresh cases and the retained `"a" == "a"` case are included in [cases.jsonl](cases.jsonl). The retained case uses the native capture from the earlier string concatenation comparison. All eight comparisons preserve the existing wire findings. SysON wraps the import in a Namespace and uses resource IDs for references. The Rust snapshot uses semantic `elementId` references.

The captured Rust prototype refused mixed scalar equality, string ordering, identity, subtraction, an invalid typing target and a deeper nesting control. The source campaign separately observed byte-identical outputs for 87 earlier prototype cases, five earlier refusals and 61 default importer cases. These broader observations are recorded in the receipt. The packet contains neither the prototype source nor its executable.

## Check the captured models

Run the offline verifier from the repository root with Python 3.

```sh
python3 -I -B porting/oracle/results/string-equality/verify.py
python3 -I -B -O porting/oracle/results/string-equality/verify.py
```

The verifier checks source and snapshot hashes, verifies UTF-8 and UTF-16 source spans, and recomputes the structural and wire comparisons with the existing oracle normalizer. It also checks the bytes and shape of the retained refusal records. Fresh-process save and reopen, test runs and default importer replay were performed in the source campaign. The offline check uses its captured output. It does not run or evaluate SysML expressions.
