# Prepared bounded parser reference corpus

This is preparation for review, not an executed oracle corpus. Twelve new
public synthetic fixtures total 790 UTF-8 bytes; the largest is 124 bytes.
Every fixture is below 1 KiB. The manifest records exact byte hashes, source
provenance, stable case/assertion IDs and layer-specific assertions. All
observations are pending and all expected wire goldens are null. No upstream
software, compiler, browser, service or package operation was executed here.
All source rule dispositions remain UNPORTED; zero strict verified rows are
added.

| Case | Fixture | Source-based question; no observed answer supplied |
| --- | --- | --- |
| pc01 | raw-names-strings.sysml | Raw quoted names/strings, escapes and Unicode versus later name sanitation |
| pc02 | multiline-string.sysml | Literal LF versus backslash-n in STRING_VALUE |
| pc03 | number-spelling.sysml | Distinct integer, fraction and exponent raw spellings |
| pc04 | number-boundaries.sysml | Binary64 rounding, underflow, overflow and unary-minus zero |
| pc05 | malformed-number-value.sysml | Lexically accepted repeated exponent signs versus numeric conversion |
| pc06 | boolean-defaults.sysml | Matched `?=` presence versus mandatory boolean defaults |
| pc07 | comments.sysml | Hidden notes versus visible comments/documentation; CRLF and U+2028 |
| pc08 | datatype-reference.kerml | Reachable `typed by` datatype fragment and local reference construction |
| pc09 | unresolved-reference.sysml | Complete syntax versus unresolved-target semantic diagnostics |
| pc10 | actions-precedence.sysml | Operand actions, replacement, append order and empty expression |
| pc11 | invalid-suffix.sysml | Complete prefix plus invalid suffix and potential recovery |
| pc12 | unterminated-string.sysml | Lexical failure, parser recovery and partial-object availability |

`source_only_hypothesis` expresses a declaration-derived question, not a
measured result. In pc04, the minus belongs to a unary expression rather than
the NUMBER terminal; this fixture does not assume a primitive negative-zero
value. pc05 intentionally uses the source EXPONENT regex's repeated-sign
acceptance. Hidden WS/SL_NOTE/ML_NOTE are automatically active fixed lexer
terminals, not parser-reference reachable rules; the manifest records that
activation separately. All other cited rules are entry reachable.

## Exact proposed route and bounds

The reviewed command is one serial invocation per explicitly reviewed fixture:

```text
${VERIFIED_NODE24} ${PINNED_BUNDLE} dump --validate --stdlib none ${CORPUS_ROOT}/${case.path}
```

Use the exact pinned Node and bundle hashes in `manifest.json`; ambient Node
must not substitute. This is the existing Java wrapper's **secondary
diagnostic** route. It differs from the primary Java import dump route and its
standard library defaults. No primary-route execution is prepared or authorized
by these files. Imports/exposes, standard/local library modes, workspace roots,
plugins, server startup and additional command flags are excluded.

The reviewed parent runner is `porting/parser-oracle/run_reference.py`, SHA-256
`074425971218521895f8dc2d8468c9791d99c30de903b4fd1d52d62914e320c7`.
It fixes Node/bundle hashes, checks the caller-supplied fixture hash, suffix,
UTF-8 and 1 KiB bound, removes all inherited NODE_* settings, keeps HOME and
CODEX_HOME, closes stdin and applies a 60-second timeout. At most twelve
serial invocations imply a 720-second aggregate timeout allowance; this is not
a request to run them. Its own outer exit can be zero while the reference exit
is nonzero: consume the structured reference result.

The runner captures separate raw streams and emits zlib9/base64, actual UTC,
hashes and byte counts without writing files. The runner neither supplies nor
attests the managed network boundary. It also does not enforce this case
allowlist/no-import rule, immutable inputs or a byte/memory cap on PIPE output.
These remain caller conditions and explicit limits.

## Observation layers and recovery limit

Capture process exit/timeout, lexical diagnostics, parser diagnostics,
semantic diagnostics, serialized AST and later Rust comparison independently.
Statistics can precede JSON on stdout; preserve raw streams before any
separately hashed parsing/projection. Never call a nonempty stdout a successful
AST automatically. If diagnostic output cannot distinguish lexer/parser/model
errors, record the category as unclassified instead of guessing.

Bundle `extractDocument` at lines 114588–114602 filters severity-one
diagnostics and exits before returning to `evalAction.stringify`. Thus a
negative or unresolved-reference case may yield diagnostics and **no available
AST through this route**. That is not evidence that the parser constructed no
recovered AST. Recovery/suffix-loss construction can remain unavailable and
requires a separately reviewed route if later needed. No expected AST values
or invented goldens are included here.

## Preparation check

Run only our standard-library preparation checker:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 porting/validation/parser-corpus/check_preparation.py \
  --repository "${REPOSITORY}"
```

It checks exact regenerated fixture/manifest bytes, UTF-8, hashes and bounds,
unique IDs, source rule fragments/activation, the excluded import/expose
keywords and absence of fabricated observations/goldens. It does not invoke
any runtime parser. The executed preparation check passed for twelve fixtures,
48 assertions and 790 bytes. `prepare.py` can regenerate these preparation
artifacts from retained source data; it performs no oracle invocation.

[Execution effect review](EXECUTION-REVIEW.md) records the inspected code path,
remaining conditions and stop criteria. The coordinator must review the exact
corpus/runner/command/bounds before any action-time execution review. Source
presence and this preparation PASS do not establish execution authorization.
