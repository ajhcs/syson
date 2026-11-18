# Feature assignment marker import evidence

This packet compares six typed assignments from the official SysON v2026.9.3 release with an isolated Rust prototype based on `05a4be35d4842272d0249ab43b9e49295591ef1e`. The prototype remains outside the default importer. Adopting the full-grammar path awaits the design decision in #246. [receipt.json](receipt.json) identifies the release JAR and its source revisions.

| Assignment spelling | Fields present on `FeatureValue` |
| --- | --- |
| `= 0` | Neither assignment flag |
| `:= 0` and `:= 7` | `isInitial` |
| `default = 0` and `default = 10` | `isDefault` |
| `default := 0` | `isInitial` and `isDefault` |

The reference and Rust both omit the integer `value` field when the value is zero. The integer classifier and semantic zero are preserved. This changes the earlier Rust zero fixture by removing only the explicit zero field. The other 99 successful prototype outputs, 11 refusal outputs and all 62 default importer outputs remained byte-identical in the source campaign.

The retained upstream examples `convertAssignmentTest/assignment1.sysml`, `assignment2.sysml`, `assignment3.sysml`, and `convertBooleanTest/boolean.sysml` supplied the marker and zero observations. Their source bytes match both upstream pins. The Boolean example produced a native syntax warning. The fresh comparison uses six clean reduced models with local typing. Full library and action imports remain outside this evidence.

The duplicate marker produced native syntax recovery and an atomic Rust refusal. The separate late typing control saved and reopened the valid first assignment. Extending that source with a wrong-type second attribute produced a typing refusal after staging and left no snapshot.

## Check the captured models

```sh
python3 -I -B porting/oracle/results/assignment-markers/verify.py
python3 -I -B -O porting/oracle/results/assignment-markers/verify.py
```

The verifier checks [cases.jsonl](cases.jsonl), including source hashes and UTF-8 and UTF-16 spans, separate marker fields, semantic integer value, full structural and wire comparisons, and the retained refusals. It checks the saved valid-prefix snapshot. It uses captured data. The source campaign performed the save, fresh reopen, no-snapshot check, and default importer replay.
