# Preserve the imported Namespace on native export

A fresh SysML text import should preserve SysON's unnamed root
`sysml:Namespace`, imported annotation, and ordered memberships when
producing a SysON-compatible document. Each package in this finite import
belongs under one `sysml:OwningMembership`. Keep this decision scoped to
fresh imported documents. Importing into an existing model needs its own
attachment and undo semantics. [Issue #249](https://github.com/ajhcs/syson/issues/249)

## What the structure changes

SysON adds `org.eclipse.syson.sysml.imported` to the resource's root
elements after text conversion. The resource service also treats
referenced libraries as imported. The Explorer's **Hide Root Namespaces**
filter shows a root Namespace's owned elements in its place. The Explorer
classifies an imported resource as a user library only if at least one
owned element of its root Namespaces is itself a Namespace and every such
element is a LibraryPackage. An ordinary package import stays in the model
tree.
[Text loader](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/application/syson-sysml-import/src/main/java/org/eclipse/syson/sysml/upload/SysMLExternalResourceLoaderService.java#L84-L91),
[Resource service](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/application/syson-application-configuration/src/main/java/org/eclipse/syson/application/services/SysONResourceService.java#L59-L65),
[Explorer filters](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/services/syson-tree-services/src/main/java/org/eclipse/syson/tree/explorer/services/SysONExplorerFilterService.java#L63-L125)

The read-only predicate checks standard-library status and the resource's
metadata adapter. The imported marker is separate. The Explorer can let a
user delete an imported resource whose contents are read-only. Publication
as a library clears the import marker and sets read-only metadata
separately. These permissions are findings from the pinned source. The
live probe below chose an editable upload and exercised the Explorer filter.
[Read-only predicate](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/application/syson-application-configuration/src/main/java/org/eclipse/syson/application/services/SysONReadOnlyObjectPredicateDelegate.java#L48-L68),
[Resource deletion](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/services/syson-tree-services/src/main/java/org/eclipse/syson/tree/explorer/services/SysONDefaultExplorerServices.java#L290-L302),
[Publication](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/application/syson-application-configuration/src/main/java/org/eclipse/syson/application/publication/SysONSysMLLibraryPublisher.java#L240-L252)

## What we verified

The off-tree prototype added the envelope to real Rust importer outputs
and reparsed the serialized JSON through Rust's document reader in memory. All three tested cases
matched the reference structure under the existing differential
comparator. We also checked the raw annotation, membership classes and
package order, which that comparator can omit. Reapplying the wrapper
and reusing a generated raw ID both failed.

| Case | Observed result |
| --- | --- |
| `s02-two-packages` | Packages A and B stayed in one document and in source order. The reference application preserved all nine semantic IDs and assigned nine new EMF object IDs. |
| `p01-basic-typed-part` | The reference application preserved all eight semantic IDs and assigned eight new EMF object IDs. Both FeatureTyping references resolved to the expected new object IDs in the re-export. |
| `s05-part-def-chain` | Rust serialized and reparsed the wrapped model in memory. The comparison against the captured reference agreed on the model structure. |

The live probe uploaded the first two prototype JSON bodies to separate
fresh projects through `uploadDocument` with `readOnly: false`, then
downloaded each project as a ZIP. The raw ZIPs passed CRC checks and
retained the annotation. In Chromium, the same A and B items appeared
under the document when the root was hidden and under the expandable
Namespace when it was shown. The receipt contains the Explorer DOM results
and SHA-256 hashes of the two screenshots retained in the build workspace.

The receipt records the pinned release and hashes for the exact synthetic
uploads and downloads. Runtime logs and fixture database state stayed
local. [Evidence](evidence.json)

## Reproduce

From the repository root, replay both exact ZIP comparisons without the
reference app.

```sh
probe=porting/oracle/results/imported-namespace-envelope
python3 -IB "$probe/verify.py"
```

If you have the original screenshots, add `--screenshots /path/to/captures`
to check their hashes. The archive comparison runs without those files.

To regenerate the two Rust upload bodies, use a fresh Cargo target
directory so each dependency resolves to one build artifact. Set
`probe_run` to an empty work directory you own.

```bash
set -euo pipefail
probe=porting/oracle/results/imported-namespace-envelope
probe_run=/absolute/work/directory/namespace-envelope
mkdir -m 700 "$probe_run"
export CARGO_TARGET_DIR="$probe_run/target"
cargo build --locked --example oracle_import \
  --manifest-path rust/crates/syson-text-import/Cargo.toml
deps="$CARGO_TARGET_DIR/debug/deps"
model=("$deps"/libsyson_model-*.rlib)
json=("$deps"/libserde_json-*.rlib)
test "${#model[@]}" -eq 1
test "${#json[@]}" -eq 1
rustc --edition=2024 -D warnings -L "dependency=$deps" \
  --extern "syson_model=${model[0]}" --extern "serde_json=${json[0]}" \
  "$probe/prototype.rs" -o "$probe_run/envelope"

replay() {
  local case="$1" source="$2"
  printf '%s' "$source" | "$CARGO_TARGET_DIR/debug/examples/oracle_import" |
    python3 -c 'import json,sys; print(json.dumps(json.load(sys.stdin)["document"]))' |
    "$probe_run/envelope" > "$probe_run/$case.json"
  cmp "$probe_run/$case.json" "$probe/$case-enveloped.json"
}

replay s02-two-packages 'package A { part def X; } package B { part def Y; }'
replay p01-basic-typed-part 'package Vehicle {part def Wheel;part wheel:Wheel;}'
```

For a live repeat, start the pinned reference from the [oracle runbook](../../README.md).
Upload each generated JSON file to a fresh editable project, open the
Explorer and toggle **Hide Root Namespaces**, then download the project.
Repeat captures have new EMF IDs and archive timestamps. Check the
annotation, semantic IDs, package order and resolved references using
the same comparisons as the offline replay.

The production Rust importer and differential harness still need to adopt
and verify the selected envelope. The prototype preserves existing
reference values. Native reference-ID emission is tracked in [#248](https://github.com/ajhcs/syson/issues/248).
