# Independent construction-kernel checks

These std-only tests exercise the public `syson-ast` operation contract,
including arena ownership, rollback, flat slot shapes and source-derived
construction rules. They execute no upstream code and do not consume a grammar
or qualify AST evaluation, linking, semantic services or CLI projection.

From a checkout containing the implementation:

```sh
CARGO_TARGET_DIR="${SYSON_BUILD_DIR}/validation-ast" \
  cargo test --offline --locked --jobs 1 \
  --manifest-path rust/validation/ast/Cargo.toml
```

Source-only preparation identified foreign numeric NodeId aliasing in the
original kernel and an Append/Array path that could create a nested property
array. Execution and exact reviewed fix dispositions belong in the separate
independent review receipt; this README does not claim these tests have run.
