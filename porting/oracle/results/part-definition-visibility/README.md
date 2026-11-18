# Visibility on package-owned part definitions

Eclipse SysON v2026.9.3 imported this model without warnings.

~~~sysml
package P { private part def A; public part def B; }
~~~

The release stores "private" on the owning membership of A. Public is the metamodel default, so the owning membership of B omits the field. Neither part definition carries a visibility field. The isolated Rust prototype reproduced the five-node Package model with the same containment and order. The released HTTP import also retains an annotated Namespace wrapper around the Package.

Three Rust snapshots reopened in fresh processes, including an explicit-versus-implicit public control. The default Rust importer still returns the same diagnostic for the exact source. All 260 incumbent outputs replayed byte for byte. Seventeen unsupported forms preserve their previous diagnostic and produce no snapshot. A later wrong-type reference refused after ten provisional identity requests and left the editor, undo history, and snapshot directory unchanged.

The prototype covers private and public on named part definitions directly owned by packages, with no nested declarations or extra modifiers. The supplementary release fixture also shows protected and private on feature memberships, and the prototype still refuses that broader fixture. Namespace imports, abstract definitions and default importer adoption remain open. This slice qualifies stored visibility. Cross-package lookup still resolves private members in the prototype. The Rust exercise used local snapshot persistence.

[Receipt](receipt.json) records the archived outputs and qualification scope. The public verifier checks those records. Live replay requires the isolated inputs identified by the receipt.

~~~sh
python3 -I -B porting/oracle/results/part-definition-visibility/verify.py
python3 -I -B -O porting/oracle/results/part-definition-visibility/verify.py
~~~
