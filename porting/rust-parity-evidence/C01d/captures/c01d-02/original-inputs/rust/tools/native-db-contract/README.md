# Native PostgreSQL contract projection

This read-only Rust development tool projects an actual coordinator-captured
PostgreSQL catalog into deterministic B05-owned comparison obligations. It does
not connect to a database, execute SQL, infer SQL type semantics, or establish
that supplied capture metadata is true. Native provenance requires the retained
SQL template, bindings, tool identities, runner, database/application startup and
unaltered raw output from the coordinator.

The input describes selected ordinary/partitioned tables, their columns, table
constraints, indexes and nonempty Liquibase applied history from one read-only,
repeatable-read snapshot. All descriptor fields are preserved, including exact
PostgreSQL type/default/constraint/index expressions and nullable migration data.
Each obligation has an unambiguous JSON-array name key and stable B05 ID. Numeric
OIDs/attribute numbers support integrity checks; they are not portable schema
identities. `migration_order` retains capture positions, independently of the
canonical named obligation order. Duplicate migration execution-order numbers
are permitted; duplicate ID/author/filename tuples or position gaps are refused.

Cross-reference checks include parent relations, local/referenced attributes,
backing indexes, foreign-key target-index ownership and index expression/INCLUDE
references. Expression attribute zero is admitted for indexes and exclusion
constraints. Invalid/not-ready indexes and unvalidated constraints remain
observations, rather than being promoted to a successful Rust comparison.

Input and output are each bounded to 8 MiB, with separate collection limits.
The CLI accepts one regular nonsymlink JSON input file and writes the complete
contract only after all validation succeeds; refusals produce no stdout.
Serde JSON object-member behavior applies; the native producer uses PostgreSQL
JSONB objects with unique member names. Input files must remain unchanged during
inspection; filesystem race resistance is not claimed.

The selected contract excludes sequences, views, functions, triggers, policies,
privileges, extension/type definitions, transaction behavior and Rust schema
comparison. Parent C01 and whole-build source correspondence remain open.

Coordinator verification recipe (not executed by the source author):

```sh
SYSON_DB_CONTRACT_TEST_ROOT=/absolute/owned/scratch \
  cargo test --offline --locked --manifest-path rust/tools/native-db-contract/Cargo.toml
cargo fmt --manifest-path rust/tools/native-db-contract/Cargo.toml --check
cargo clippy --offline --locked --manifest-path rust/tools/native-db-contract/Cargo.toml --all-targets -- -D warnings
syson-native-db-contract /absolute/owned/native-catalog.json
```

The fixture root must already exist; the CLI regression retains a process-specific
subdirectory and its synthetic inputs. Synthetic regression descriptors are not
native observations. Native SQL/template, execution captures and claim boundaries
belong in `porting/rust-parity-evidence/C01d/`.
