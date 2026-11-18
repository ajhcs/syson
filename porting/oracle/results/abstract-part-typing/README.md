# Part usage typed by an abstract part definition

The SysON v2026.9.3 parser linked `x` to `A` in both fixtures. A separate CLI validation invocation exited successfully for each. The released converter saved the typing relationship, and the derived `PartUsage.getPartDefinition()` returned `A` in both models.

```sysml
package P { abstract part def A; part x : A; }
package P { part def A; part x : A; }
```

The isolated Rust probe also accepted both fixtures. The complete six-node Package projections matched the released converter after removing the Namespace wrapper and translating the resource and semantic reference identities consistently. Only the abstract definition stored the Boolean `isAbstract: true`.

The wrong-metaclass control uses `attribute def A` and the same `part x : A` declaration. The parser still resolved the name, and the converter retained the reference to the attribute without a transformation warning. The derived part-definition list was empty. The independent CLI validation reported `PartUsage must be typed by Structures only`. The Rust probe returned a wrong-type error and produced no document.

The bare released AST dump and the separate `dump --validate --stdlib none` invocation match the two command contexts used by SysON text upload. The native converter ran directly against the captured ASTs in an isolated JVM. Upload-report rendering, editing-context AQL validation, constructor evaluation, and default importer adoption remain open. The definition-only fixture remains in #343.

Verify the stored comparison from the repository root.

```sh
python3 -IB porting/oracle/results/abstract-part-typing/verify.py
python3 -OIB porting/oracle/results/abstract-part-typing/verify.py
```
