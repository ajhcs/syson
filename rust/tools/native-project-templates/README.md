# Native project-template observations

This development-only Rust reader consumes the coordinator's actual anonymous
capture of the ten read-only GraphQL requests in the frozen C01h query manifest.
It preserves the complete capture Value and exact raw body strings, including
unknown metadata, arbitrary-precision numbers, all arrays and PageInfo values.
Requests/IDs must match that manifest; successful payloads validate the selected
template, library-reference and page field types without reconstructing paging.

Only the PROJECT_BROWSER context interprets the exact IDs `upload-project` and
`browse-all-project-templates` as controls. Every other ID is an ordinary template,
including `blank-project`, empty labels and those two IDs in other contexts.
Unknown IDs refuse selection. Duplicate native IDs remain observed, while an
ambiguous selection refuses. No default template, URL normalization, provider
activation, capability policy, library resolution or instantiation is invented.

GraphQL and HTTP failures remain explicit observations with null choices; partial
GraphQL data does not become a successful empty catalogue. Successful empty pages
remain distinct and retain their original count/cursors. The native small-browser
page failures and empty later modal page are preserved rather than corrected.

Input and output are each capped to 8 MiB with bounded observations, rows, headers,
library references and copied IDs. Projection uses bounded counting serialization.
The CLI accepts a regular nonsymlink capture, optionally an observation ID and
choice ID, and produces no stdout on refusal. Input files must remain unchanged
while inspected; race resistance and independent capture provenance are not claimed.
Serde Value object-member semantics apply; raw GraphQL body strings remain exact.

Coordinator verification recipe (no compiler/native execution by the author):

```sh
SYSON_PROJECT_TEMPLATES_TEST_ROOT=/absolute/owned/scratch \
  cargo test --offline --locked --manifest-path rust/tools/native-project-templates/Cargo.toml
cargo fmt --manifest-path rust/tools/native-project-templates/Cargo.toml --check
cargo clippy --offline --locked --manifest-path rust/tools/native-project-templates/Cargo.toml --all-targets -- -D warnings
syson-native-project-templates /absolute/owned/native-templates.json
syson-native-project-templates /absolute/owned/native-templates.json browser-large upload-project
```

C01, B23 and U03 remain open. The capture provides a template-discovery/choice
contract, not working project creation, browser usability, representation registry,
UI extension activation, whole-source/build correspondence or feature acceptance.
