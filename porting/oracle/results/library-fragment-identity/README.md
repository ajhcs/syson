# Standard-library fragments in SysON references

The Rust registry resolves these two typed references from Eclipse SysON
v2026.9.3 when the caller supplies the matching resource and selects
semantic `elementId` fragments. The emitted fragments differ from the
stored EMF object IDs. [Issue #250](https://github.com/ajhcs/syson/issues/250)

| Target | Emitted `elementId` | Stored EMF `id` |
| --- | --- | --- |
| `ScalarValues::Integer` | `f2350199-2ab1-5258-8514-58812ef25dc6` | `949c0bd9-6f87-44c5-93ae-91cd99f2da3b` |
| `StandardViewDefinitions::GeneralView` | `03904fdf-d6f2-57b1-92d5-95d36b8208dc` | `d7955624-057b-45d2-98f9-671776192420` |

The Integer wire came from the release's `assignment1` text import. The
GeneralView wire is byte-identical in the native project download, its
native re-export, and two outputs of the off-tree Rust archive writer.
The archive writer preserves the existing string. The separate Rust
probe resolves each typed string and reconstructs it exactly from its
resource URI, classifier and semantic ID.
[Golden capture](https://github.com/ajhcs/syson/pull/288),
[Archive control](https://github.com/ajhcs/syson/pull/296),
[Evidence](evidence.json)

The two library files in the pinned nested configuration JAR match
release source `b380dada` and port source pin `6ba2ac64`. The resource
UUIDs match the loader's `UUID.nameUUIDFromBytes` result over each
ASCII classpath path. The prior native catalogue recorded the same
file hashes and URIs, along with two load diagnostics for ScalarValues
and one for StandardViewDefinitions. Broader library loading remains open.
[Default libraries](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/application/syson-application-configuration/src/main/java/org/eclipse/syson/application/configuration/SysONDefaultLibrariesConfiguration.java#L43-L58),
[URI derivation](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/application/syson-application-configuration/src/main/java/org/eclipse/syson/application/libraries/SysONLibraryLoader.java#L78-L89)

The controls check the wrong fragment policy, wrong URI scheme, missing
resource, wrong caller classifier, wrong wire classifier, and foreign
registry snapshot. The registry also resolves the stored EMF ID when
that policy is selected explicitly. The native catalogue observed
different fragment behavior for root Namespaces. Qualify each broader
fragment policy before adoption.

The production importer currently registers its staged project document
and accepts local linker bindings. Qualified and imported library-name
resolution and production reference emission remain open under #250.

## Reproduce

Use Python 3.11 or later, the pinned v2026.9.3 release JAR and a new
writable build directory outside the checkout. The replay verifies the
exact inputs and compares the regenerated probe output with the recorded
result. The compiled probe hash identifies the original execution.
[Reference runbook](../../README.md)

Run from the repository root. Set the two absolute paths and fetch the
linked PR heads if the pinned input commits are absent locally.

```bash
set -euo pipefail
probe=porting/oracle/results/library-fragment-identity
proof_run=/absolute/work/directory/library-fragment-proof
reference_jar=/absolute/path/to/syson-application-2026.9.3.jar
mkdir -m 700 "$proof_run"
export CARGO_TARGET_DIR="$proof_run/target"

git show a118e74688b3f3af7c153b9ff1a1e677e2647003:porting/interop/fixtures/archive-demo.syson.zip \
  > "$proof_run/original.zip"
git show e8260c93afc8dc9d134d5db63b86d692077f5cef:porting/interop/evidence/archive-demo-upload-v2026.9.3/archive-demo-reexport.syson.zip \
  > "$proof_run/reexport.zip"
git show c4a9d152e710974068d0d54efaf1d643df505dea:porting/oracle/golden/convertAssignmentTest-assignment1.json \
  > "$proof_run/assignment.json"

cargo build --locked --example oracle_import \
  --manifest-path rust/crates/syson-text-import/Cargo.toml
deps="$CARGO_TARGET_DIR/debug/deps"
model=("$deps"/libsyson_model-*.rlib)
references=("$deps"/libsyson_references-*.rlib)
metamodel=("$deps"/libsyson_metamodel-*.rlib)
json=("$deps"/libserde_json-*.rlib)
rustc --edition=2024 -D warnings -L "dependency=$deps" \
  --extern "syson_model=${model[0]}" --extern "syson_references=${references[0]}" \
  --extern "syson_metamodel=${metamodel[0]}" --extern "serde_json=${json[0]}" \
  "$probe/probe.rs" -o "$proof_run/probe"

python3 -IB "$probe/replay.py" --jar "$reference_jar" \
  --assignment "$proof_run/assignment.json" --original "$proof_run/original.zip" \
  --reexport "$proof_run/reexport.zip" --probe "$proof_run/probe" \
  --output "$proof_run/replay"
```

The original live assignment receipt can replace the golden input. Pass
`--writer-outputs EDITED OPAQUE` to verify the two exact retained writer
ZIPs. The replay extracts the two library files only into the external
output directory. Retain their attribution and LGPL terms.
[Library license](../../../../LICENSE-LGPL), [Notices](../../../../NOTICE)
