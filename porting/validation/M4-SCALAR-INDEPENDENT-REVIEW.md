# Independent review: registry-backed scalar adapter

Reviewed source commit: `4df1f01d7f7bcd0c4c691254e958fc7250240389`. Upstream baseline: `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`. Disposition: accepted within the explicit `MODEL-SCALARS.md` contract. No implementation edits by this validator. No upstream runtime was executed; zero upstream-verified behavior claims are added.

Reviewed files and exact SHA256:

| File | SHA256 |
| --- | --- |
| `rust/crates/syson-model/src/lib.rs` | `beabae83600f90612a5aa6a74c68ce44e27c93d4a78be9cef185e7e8268a827d` |
| `rust/crates/syson-model/src/scalar.rs` | `cab38bc0eff8d1087ac45905a0020bc7762a7de75419578331d25a2a0e1404ec` |
| `rust/crates/syson-metamodel/src/lib.rs` | `f5755e0e1b7910389cce955280f1424cdc75b9e33e0dde99a7cb6386613cf61c` |
| `rust/crates/syson-metamodel/src/generated.rs` | `02d68b123460a89590d4c3d62a6421e94d0b8d933ad9a80cc67fbcd816158aab` |

## Source interpretation

Scalar feature lookup preserves actual inherited declarations and rejects unresolved external branches, ambiguous candidates, references and containments before writing. Descriptor blocks distinguish derived/nonchangeable/many-valued/identity/unsupported types. A descriptor's stored optional value distinguishes absent from explicit JSON null; inspection does not materialize defaults.

The setter matches its documented strict JSON subset: optional EString permits null, primitives reject null/coercion, EInt requires integer spelling in signed 32-bit range, EDouble normalizes finite inputs to f64, and EEnums require exact literal strings while all null-reset semantics remain unsupported. Normalization is completed before staged JSON mutation. Existing transaction cloning/poisoning/commit validation provides error atomicity and history preservation.

Pinned upstream `impl/LiteralIntegerImpl.java` takes Java int and casts eSet input to Integer; `impl/LiteralRationalImpl.java` takes double and casts eSet input to Double. Ecore types and enum spellings provide source evidence for adapter descriptors. The adapter's finite-number and JSON coercion rules are explicit local restrictions; they are not claimed as complete Jackson/eSet behavior. In particular Java enum setters' different null/default/ESet behavior is not replaced by a generic raw-null write.

Projection now classifies namespaces through local registry ancestry. Only known `sysml:` domain EClasses can produce positive typed matches; unqualified/foreign-prefix/unknown/enum-as-object identities remain unresolved, with original serialized class strings preserved. Relationship projection, effective-name overrides, imported scope and complete external semantics retain their documented boundaries. Existing bounded create/rename/typing methods retain their separate policies.

## Executed independent tests

`rust/validation/tests/scalar_regressions.rs`: **6 tests passed**, including:

- EInt endpoints and negative integer zero accepted; out-of-range, fractional/exponent spelling, strings, booleans, null, arrays and objects rejected.
- EDouble maximum finite value, minimum positive subnormal, positive/negative zero, signed underflow and rounding above 2^53 normalized as documented; overflow and other JSON types rejected. Normalized values roundtrip.
- Optional EString absent versus explicit null remains distinguishable. Required strings accept empty/Unicode/control-character values, reject null, and roundtrip through JSON.
- Every advertised writable descriptor accepted a valid typed value: **1,519 assignments across all 175 domain EClasses and all seven enum types**. Unrelated vendor fields and separate resource/stable IDs remained unchanged, and each result roundtripped. This checks consistency between introspection advertised to consumers and the setter; it does not establish complete construction or semantic validity of all classes.
- Enum reset, external-branch, containment and identity failures after a successful staged scalar change preserved the last staged value and rejected whole-transaction commit without altering committed history.
- An unnamed AttributeDefinition root was correctly treated as a Namespace ancestor during qualified lookup. Unknown prefixes, unknown class names and enum declarations used as object classes did not become positive typed matches.

Exact model/registry source files were snapshotted from the reviewed commit into an isolated build tree. One offline compiler job was used. An initial `--locked` preflight rejected the prior lockfile's missing local model-to-registry dependency edge; the isolated lockfile was refreshed offline without adding an external dependency, then the suite ran successfully with `--locked`. The updated validator lockfile is committed.

Reproduce after integration:

```
CARGO_TARGET_DIR="${SYSON_BUILD_DIR}/scalar-target" cargo test --offline --locked --jobs 1 --manifest-path rust/validation/Cargo.toml --test scalar_regressions -- --nocapture
```

No unresolved correctness finding was found within the promised subset. External schema resolution, notifications, unset flags, enum reset/default materialization, cross-feature constraints, opposite/proxy changes, semantic operations and full API/storage/runtime behavior remain open. UI integration is separately reviewed and tested.
