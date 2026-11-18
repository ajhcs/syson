# Finite ranges on typed part usages

The Syside CLI bundled with SysON v2026.9.3 parsed `part w : W[1..4];` into an operator with two ordered numeric operands. The exact release's `ASTTransformer.convertResource` converted that AST in a plain EMF resource set with all 94 bundled standard libraries. It saved two separate owning memberships under `MultiplicityRange`, one for integer 1 and one for integer 4. The range membership precedes the typing link to `W`. The converter emitted no transformation messages.

The off-tree Rust prototype saved and reopened all 14 nodes in the same Package projection. The converter's original output keeps an unannotated root Namespace and its owning membership, for 16 nodes. The verifier preserves the raw comparison and checks the exact two-node projection separately. SysON encodes the typing references by resource ID and Rust by `elementId`.

Local controls preserve the source order in `[4..1]` and store two bounds for `[0..0]`. Those controls establish Rust construction behavior. Nine unsupported forms preserve the predecessor's refusal bytes, including mixed, open, signed, and out-of-domain bounds. A valid range followed by a wrongly typed part refused after 19 identity requests without a persisted snapshot, editor change, or undo entry. All 212 previously admitted comparison outputs match byte for byte. The prior `[1..4]` refusal is the intended acceptance change.

The official HTTP attempt stopped on host CPU pressure before capture. The reference result covers the release converter and its EMF save. HTTP import, persistence, and activation in the default Rust importer remain open under #246 and #268. [receipt.json](receipt.json) records the exact inputs and outcomes.

## Verify the captured evidence

```sh
python3 -I -B porting/oracle/results/part-usage-finite-range/verify.py
python3 -I -B -O porting/oracle/results/part-usage-finite-range/verify.py
```
