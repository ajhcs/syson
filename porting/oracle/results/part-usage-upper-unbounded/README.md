# Upper-unbounded ranges on typed part usages

The Syside CLI bundled with SysON v2026.9.3 parsed `part w : W[1..*];` with integer `1` before the infinity token. The same release's `ASTTransformer.convertResource` saved two separate owning memberships under the multiplicity range. The first contains `LiteralInteger` with value 1 and the second contains `LiteralInfinity` with identity alone. The range membership precedes the typing link to `W`. The converter loaded all 94 bundled library resources and emitted no transformation messages.

The isolated Rust model matches all 14 nodes of the converter's Package projection after a fresh process reopen. The raw converter tree has 16 nodes because it retains an unannotated root Namespace and its owning membership. The evidence preserves both comparisons. The native typing link uses resource IDs and Rust uses semantic `elementId` references.

Two archived `[0..*]` refusals now reopen in the prototype, one nested usage and one top-level usage. Both store an explicit integer-bound node with zero represented by the metamodel default. These are local Rust controls. `[1..*]` is the sole converter fixture.

Nine unsupported forms preserve the predecessor's refusal bytes, including lower infinity, open bounds, out-of-domain numbers, and a range used as a feature value. Appending a wrongly typed usage after a valid one refused after 19 identity requests without saving a snapshot or changing the editor or undo history. All 224 unaffected output pairs match byte for byte.

The qualification uses the release converter in memory. HTTP upload, application persistence, and adoption in the default Rust importer remain open under #246 and #268. [receipt.json](receipt.json) records the exact input, artifact and campaign identities.

## Verify the captured evidence

```sh
python3 -I -B porting/oracle/results/part-usage-upper-unbounded/verify.py
python3 -I -B -O porting/oracle/results/part-usage-upper-unbounded/verify.py
```
