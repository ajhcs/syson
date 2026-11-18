# Preserve unresolved sibling-package part typing

This records an isolated, opt-in Rust prototype against the four SysON 2026.9.3 converter cases in #344. When a plain PartUsage in one top-level package references a private PartDefinition in a sibling package, or the final simple name in that existing sibling package is missing, the prototype saves the FeatureTyping and its typedFeature endpoint. It leaves the type field absent and returns the converter's exact warning after the editor commits. The public and same-package private controls still bind the type and return no warnings.

Four snapshots reopened in distinct Rust processes with the exact saved bytes. Warnings travel with the commit result and stay outside the saved document. Nine fatal controls returned the prior diagnostic and created neither a snapshot nor a warning. A failure on a later wrong-type declaration discarded the already staged recovered edge and warning after 12 provisional identity requests. The baseline importer and prototype replayed 305 prior outputs byte for byte, including 65 default-import cases. Another 236 invocations checked that the opt-in mode preserved unrelated results.

The projection removes the converter's Namespace and its owning wrappers. Resource IDs used by native references and semantic IDs used by Rust become positional references. All retained package, relationship, field and endpoint comparisons remain exact.

From the repository root, verify the checked-in projection and rollback records with Python 3:

```sh
python3 -IB porting/oracle/results/qualified-part-type-recovery/verify.py
python3 -OIB porting/oracle/results/qualified-part-type-recovery/verify.py
```

These commands replay the recorded evidence. Runtime source and binaries remain in the isolated qualification. The default importer, general visibility enforcement, diagnostics UI, native HTTP and application storage remain outside this result.
