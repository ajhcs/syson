# Unqualified library types across package scopes

SysON v2026.9.3 resolves unqualified `Integer` in a nested package that
inherits its parent's private membership import. It leaves the type
unresolved in a sibling package or when an owned package named `Integer`
shadows the imported type. [Issue #250](https://github.com/ajhcs/syson/issues/250)

| Fixture | Observed name lookup |
| --- | --- |
| `nested-private` | Walks from `Inner` to `Outer`, where the private import supplies the Integer DataType. |
| `public-sibling` | `Peer` has no matching membership. The public import in `Imports` remains in that package's scope. |
| `nested-shadow` | The first name match is the owned `Inner::Integer` package. It fails the type requirement, leaving the reference unresolved. |

The standalone Rust probe takes package ownership, type names, import
names and visibility from the syntax recognizer's CST spans. It composes
those facts with the existing domain model and registry. On the same
inputs, the landed one-segment raw search finds no Integer from each local
package. The wider search through all roots finds Integer for all three
cases, including both unresolved controls. The fixture's owner and
membership walk lives in the standalone probe. The production query
and importer stay as they are.
[Native lexical implementation](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/NamespaceImpl.java#L269-L287),
[Native local lookup](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/NamespaceImpl.java#L330-L359)

Each native text import supplies the document scaffold. The capture
removes `MembershipImport.importedMembership` and `FeatureTyping.type`
before applying the Rust wires. The already populated scaffold and a
duplicate role both fail offline controls. The native hierarchy, names,
local semantic IDs and visibility come from the scaffold. The two
wire roles use distinct semantic fragments, as verified in the
[library fragment proof](../library-fragment-identity/README.md).

After JSON upload, each downloaded project preserved all model fields
apart from remapped raw object IDs and the matching local
`typedFeature` alias. Editor-service queries confirmed that every
imported membership owns Integer. A boolean query confirmed the type is
absent in both unresolved cases and present in the nested-private case.
That case also returns Integer from the reloaded typing. The evidence
contains the unmodified six ZIPs and the observed editor responses.

The three fixtures exercise explicit membership imports. Production
library loading, whole-importer integration, wildcard and recursive
imports, ambiguity handling and broader visibility remain open.

## Verify the recorded run

Python 3 is sufficient for the six captured ZIPs and editor responses.

```sh
proof=porting/oracle/results/library-lexical-scope
python3 -IB "$proof/verify.py"
```

To regenerate the lexical decisions, use a new external build directory
and the exact `ScalarValues.json` extracted by the
[library fragment proof](../library-fragment-identity/README.md).

```bash
set -euo pipefail
proof=porting/oracle/results/library-lexical-scope
run=/absolute/work/directory/lexical-proof
scalar=/absolute/path/to/fragment-replay/ScalarValues.json
mkdir -m 700 "$run"
cargo build --locked --example oracle_import \
  --manifest-path rust/crates/syson-text-import/Cargo.toml --target-dir "$run/model"
cargo build --locked --manifest-path rust/crates/syson-syntax/Cargo.toml --target-dir "$run/syntax"
m="$run/model/debug/deps"
s="$run/syntax/debug/deps"
model=("$m"/libsyson_model-*.rlib)
domain=("$m"/libsyson_domain-*.rlib)
references=("$m"/libsyson_references-*.rlib)
metamodel=("$m"/libsyson_metamodel-*.rlib)
json=("$m"/libserde_json-*.rlib)
syntax=("$s"/libsyson_syntax-*.rlib)
rustc --edition=2024 -D warnings -L "dependency=$m" -L "dependency=$s" \
  --extern "syson_model=${model[0]}" --extern "syson_domain=${domain[0]}" \
  --extern "syson_references=${references[0]}" --extern "syson_metamodel=${metamodel[0]}" \
  --extern "serde_json=${json[0]}" --extern "syson_syntax=${syntax[0]}" \
  "$proof/probe.rs" -o "$run/probe"
"$run/probe" "$proof/cases.json" "$scalar" > "$run/rust-results.json"
python3 -IB "$proof/verify.py" --probe "$run/probe" --scalar "$scalar"
```

For a live repeat, start a fresh reference with the
[oracle runbook](../../README.md). Run the capture with that
oracle's state path and the regenerated Rust result. The capture
creates fresh fixture projects in the disposable reference database.
Stop the reference afterward through the runbook's teardown command.

```sh
python3 -IB "$proof/capture.py" --state /absolute/reference/current.json \
  --rust-results "$run/rust-results.json" --out "$run/native"
python3 -IB "$proof/verify.py" --capture "$run/native" \
  --rust-results "$run/rust-results.json"
```

The recorded execution used Rust source at `05a4be35d`, the pinned
release JAR and the reference helpers from `c4a9d152`. The generated
provenance records those pins and the artifact hashes. The replay
uses the library files in the external build directory under their
original LGPL terms. [Evidence provenance](evidence/provenance.json),
[Library license](../../../../LICENSE-LGPL), [Notices](../../../../NOTICE)
