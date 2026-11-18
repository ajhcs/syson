# Lower infinity on a typed part usage

Eclipse SysON v2026.9.3 converted the following model in memory.

~~~sysml
package P { part def W; part def C { part w : W[*..4]; } }
~~~

The release parser identifies the asterisk as the left operand and the integer as the right operand. The converter stores those operands as `LiteralInfinity` and `LiteralInteger(4)` in two distinct owning memberships. The lower and upper getters return the stored objects. The range membership precedes the typing link from `w` to `W`.

An isolated Rust prototype accepted the same model and reopened its snapshot in a fresh process. Its 14 nodes match the converter's Package subtree. The raw converter model adds a Namespace and an owning membership above that Package. The converter's typing link targets resource IDs. The Rust snapshot uses semantic element IDs.

The archived prototype originally refused this exact source. The current default importer returns the same refusal bytes as its own predecessor. All 236 other archived cases replayed byte for byte, including 62 on the default importer. Eighteen additional checks preserved earlier refusals for unsupported bounds, other owners, anonymous or untyped parts, specialization, ranges before the type, and feature values. The late wrong-type case refused after 19 identity requests and left the editor, undo history, and snapshot directory unchanged.

This evidence covers the exact release converter fixture and the named part syntax with the range after its type. Application HTTP import, database persistence, range evaluation, and adoption by the default Rust importer remain open under [#246](https://github.com/ajhcs/syson/issues/246) and [#268](https://github.com/ajhcs/syson/issues/268).

## Verify the captured evidence

~~~sh
python3 -I -B porting/oracle/results/part-usage-lower-unbounded/verify.py
python3 -I -B -O porting/oracle/results/part-usage-lower-unbounded/verify.py
~~~

The verifier reads the retained packet. Live Rust replay and native converter execution require the isolated qualification inputs referenced by [receipt.json](receipt.json).
