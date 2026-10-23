# Independent bounded syntax checks

These native tests exercise `syson-syntax` without running upstream software.
They check exhaustive small CFG languages independently of the recognizer,
actual grammar examples, terminal spelling/positions, grammar rejection,
resource budgets and cooperative cancellation. The receipt example emits the
selected CST for the same synthetic input used by the separately recorded
reference observation. It emits no semantic AST and performs no linking or
validation.

From a checkout containing the implementation:

```sh
CARGO_TARGET_DIR="${SYSON_BUILD_DIR}/validation-syntax" \
  cargo test --offline --locked --jobs 1 \
  --manifest-path rust/validation/syntax/Cargo.toml
CARGO_TARGET_DIR="${SYSON_BUILD_DIR}/validation-syntax" \
  cargo run --offline --locked --jobs 1 --quiet \
  --manifest-path rust/validation/syntax/Cargo.toml --example oracle_receipt
```

`porting/validation/fixtures/syntax-terminal-base.json` derives its twelve
terminal definitions from the candidate Syside grammar at
`1326f234cae23b0e4777d5a48a069b17e9e332f1`, preserving their contents while
remapping local rule indices. The original source and EPL-2.0 notice are
retained in `rust/crates/syson-syntax/data/LICENSE-SYSIDE`; that provenance does
not establish equivalence to the bundled reference parser. The small custom
parser rules in the tests provide separate language oracles for the CFG
engine. They are not SysML fixtures or semantic tests.

All source rule dispositions remain UNPORTED. These checks do not establish
upstream parity, AST equality, error wording, recovery, semantic validation,
SysON import or product completion.
