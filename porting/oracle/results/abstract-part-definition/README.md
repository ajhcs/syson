# Abstract part definitions owned by packages

Eclipse SysON v2026.9.3 imported this fixture without warnings.

~~~sysml
package P { abstract part def A; }
~~~

The released model stores Boolean `isAbstract: true` on A. The owning membership and the enclosing Package carry no abstract flag. The isolated Rust probe translated the exact `abstract` keyword from the grammar’s raw string assignment to the stored Boolean. The three-node Package subtree matched SysON. The released HTTP import also preserves an annotated Namespace wrapper around that subtree.

Removing the modifier produced the same Rust containment, ownership, order and identities, with the abstract field omitted. Three snapshots reopened in fresh processes. The probe preserved 283 archived outputs, including 64 from the default importer, and 16 unsupported forms kept their previous diagnostics and produced no snapshots. A later wrong-type reference failed after eight provisional identity requests and left the editor, undo history and snapshot directory unchanged. A separate cancellation control requested no identities.

The initial probe expected a Boolean in the syntax tree and refused the fixture. The receipt preserves that failed output. The comparison covers the exact semicolon fixture and the stored flag on a named package-owned part definition. The probe also accepts a PartUsage typed by that abstract definition. The corresponding SysON type and instantiation rules, broader syntax, default importer adoption and the integrated application journey remain open. The default importer continues to refuse the exact fixture.

The public verifier checks the archived results in normal and optimized Python. Runtime comparison used the pinned release capture and a separately built off-tree Rust probe.

~~~sh
python3 -IB porting/oracle/results/abstract-part-definition/verify.py
python3 -OIB porting/oracle/results/abstract-part-definition/verify.py
~~~
