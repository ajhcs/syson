# Release converter confirmation for typed [*]

This follows up on #332 with the exact source `package P { part def System; part systems : System [*]; }`. The SysON v2026.9.3 release converter used the retained bundled Syside AST and all 94 bundled standard library resources. It emitted no transformation messages.

The saved range contains one owning membership with an identity-only `LiteralInfinity`. The Java lower-bound getter returned a detached integer zero, and the upper-bound getter pointed to the stored infinity. The saved JSON contains no zero node. The range precedes the typing link from `systems` to `System`.

The ten-node Package projection matches the Rust stage and snapshot already saved and reopened in separate processes. The comparison retains the converter's original twelve-node tree and its unannotated Namespace. Native typing links use resource IDs. Rust uses semantic `elementId` references.

The successful run reused the pinned Java class, bundled AST and ordered release classpath. The earlier stopped JVM and application attempts remain part of the evidence. This result qualifies the in-memory converter and its EMF save. HTTP capture, application persistence and default Rust importer adoption remain open under #246 and #268.

## Verify the captured evidence

The verifier reads the exact files from the parent evidence branch. It checks both the original converter output and its explicit Package projection.

```sh
python3 -I -B porting/oracle/results/part-usage-unlimited-converter/verify.py
python3 -I -B -O porting/oracle/results/part-usage-unlimited-converter/verify.py
```
