# Rust syntax milestone SYNTAX-01

`rust/crates/syson-syntax` is an independently buildable Rust lexer and generalized grammar/CST recognizer over the complete serialized candidate Syside grammars. It implements grammar syntax machinery rather than a handwritten selection of modeling features. It is **not a Syside conversion or accepted full parser port**: AST construction, postparse repairs, type/name linking, workspace semantics, validation, recovery, CLI JSON and SysON model import remain unimplemented. Every successful result carries `SemanticStatus::NotPerformed`.

Candidate grammar source is Syside `0.9.0`, commit `1326f234cae23b0e4777d5a48a069b17e9e332f1`, from `packages/syside-languageserver/src/generated/grammar.ts` and the five `.langium` inputs. Exact CLI bundle/source and matching notice provenance remain OPEN. The upstream parser/runtime was not executed in this work.

## Implemented behavior

- Both complete declaration sets compile into a Rust-owned BNF representation: KerML 270 parser rules plus 12 terminal rules; SysML 441 parser rules plus 12 terminal rules. All 735 source declarations remain explicitly `UNPORTED` for runtime parity and individually unqualified in `RULE-DISPOSITION.jsonl`.
- The standard-library lexer implements the fixed terminal contracts: four fragment components of NUMBER and eight non-fragment terminals. Grammar loading compares all terminal definitions/hidden/fragment flags against the pinned source contract and rejects mutations. It handles ASCII ID, unrestricted-name escapes, raw STRING_VALUE tokens, composed numbers, visible REGULAR_COMMENT, hidden ML_NOTE/SL_NOTE and ECMAScript whitespace. It performs no numeric/string value conversion.
- Grammar compilation handles RuleCall, Keyword, Group, Alternatives, cardinalities `?/*/+`, Assignment, Action and CrossReference. Assignments expose feature/operator markers, actions expose zero-width type/feature/operator markers, and references expose target-type markers in the CST. None executes AST creation, mutates model state or resolves a name. Unrecognized syntax constructs, argument/wildcard/hidden-token extensions, terminal changes or invalid metadata fail explicitly at grammar loading.
- An indexed Earley chart recognizes CFG membership, including recursive/nullable constructions. The result contains one deterministic discovered CST derivation and a merge counter. All input must be consumed. Missing syntax returns a source span, found token/EOF and sorted expected tokens. Unknown custom rules and declared custom entries not reachable from the real language entry are explicit errors.
- Tokens retain hidden source text and separate byte, absolute UTF-16, zero-based line and UTF-16 column positions. CRLF counts as one line break; CR and LF also count separately when not paired. U+2028/U+2029 are ECMAScript whitespace but do not introduce an LSP line break in this position contract. CST spans enclose visible tokens and child marker points; they are not qualified as upstream CST spans.
- Cooperative atomic cancellation and explicit input-byte, token, work-step, chart-state and constructed-CST-node budgets are checked. Waiting/completion indexes avoid unaccounted repeated whole-chart scans. No budget failure returns success or a partial accepted model. Results and repeated calls are deterministic for the same inputs/options.

Default limits are 1 MiB input, 100,000 tokens (including retained hidden tokens), 5,000,000 work steps, 250,000 chart states and 250,000 constructed CST nodes. These are bounded local API defaults, not upstream scaling equivalence. Large-model throughput and all standard-library parser cases are unqualified.

## Source fidelity and remaining parser differences

Langium's source `DefaultTokenBuilder.buildTokens/buildKeywordTokens` in the pinned embedded SysON bundle (approximately lines 41756-41810) derives keywords from **entry-reachable** parser rules, orders them by length and considers longer terminal alternatives. This milestone retains the full declared sets (146 KerML / 174 SysML keywords) while lexing with the entry-reachable sets (145 / 173). The unused ImportKind declaration's `::*::**` keyword must not swallow the actual `::*` then `::**` token sequence. A regression covers recursive namespace imports. Declared but unreachable custom entries return `UnsupportedEntry` instead of silently using the wrong lexer configuration.

REGULAR_COMMENT stays visible because the grammar uses it for comment/documentation AST syntax. ML_NOTE precedes SL_NOTE; when the multi-line note regex does not match a `//*` prefix, the valid line-note fallback is preserved. The NUMBER exponent definition permits repeated `+/-`; the raw token is preserved instead of normalized into ideal SysML numeric syntax. ECMAScript whitespace includes BOM/NBSP and excludes NEL/U+001C, unlike common Rust/Java whitespace helpers. ID remains ASCII; unrestricted names can carry Unicode. Source-derived tests preserve these distinctions.

This is a **generalized CFG recognizer**, not a reproduction of Chevrotain/ALL(*) lookahead, greedy/ordered decisions or recovery. Source alternatives, groups and markers are retained, but the first discovered CST can differ from the upstream chosen AST/CST. `ambiguous_state_merges` counts merged states with different retained construction histories, including nullable construction duplicates; it is not an exact number of language ambiguities. Generalized membership can differ from upstream runtime acceptance in cases needing implementation-specific decisions. There is no live differential evidence.

Lexing is fail-fast: it does not implement upstream skipped-error recovery, aggregate lexer errors or exact diagnostic wording. Quoted-token failures currently identify the quote's start point. Value conversion, including JavaScript numeric precision/NaN and escape decoding, is deliberately absent. An upstream semantically invalid AttributeUsage typing fixture is accepted only as syntax; `SemanticStatus::NotPerformed` prevents interpreting that as semantic validation success. No textual import API, model mutation or server endpoint is provided.

## Data generation, notices and dependencies

`python3 porting/syntax/generate_data.py --check` verifies all Rust grammar data and the rule disposition ledger against the retained source audit. No JavaScript, TypeScript, Langium, parser generator or external program implements product behavior. Generation uses Python only as an inspected development tool; the Rust library loads checked-in JSON data and uses Rust logic at runtime. `data/PROVENANCE.json` hashes the supplied grammar/contracts/license bytes.

Original candidate source license/copyright material is retained in `data/LICENSE-SYSIDE` and the source audit; see crate `NOTICE`. Rust translation headers retain Sensmetry attribution for terminal-derived work. Root SysON LICENSE/NOTICE and standard-library LGPL obligations remain unchanged. The crate does not embed the 94 textual standard libraries or claim distribution/license closure for the original CLI bundle.

Only `serde_json = 1.0.151` with the already inspected `arbitrary_precision` feature closure is added. The closure and build-script effects match `porting/domain/MODEL-DEPENDENCIES.md` (active serde_json/serde_core/itoa/memchr/zmij; MIT/Apache/Unlicense obligations as declared there). No regex crate, derive macros, network download, new service or non-Rust parser runtime is used. Cargo.lock pins the offline dependency graph; inactive lock entries do not imply runtime inclusion. Native/WASM compilation does not prove browser instantiation, ABI suitability, memory limits or UI behavior.

## Observed verification

The final run captured actual UTC `2026-10-22T21:32:27.202208Z` through `2026-10-22T21:32:27.900253Z`, independently of Git commit time. `VALIDATION.json` records exact source/data/test/manifest/generator/runner SHA256, command arguments, per-command UTC start/end, full-output hashes and test result lines. Complete logs remain in the selected build evidence directory. The initial evidence-runner filename error occurred before Cargo checks; the corrected run used a new evidence directory and is the only final receipt.

The final checks passed: deterministic data/ledger check; rustfmt check; **15 native tests**; clippy all targets with warnings denied; and `wasm32-unknown-unknown` check. Tests cover exact upstream namespace/camera inputs, a caret-removed incomplete upstream completion input, syntax-only acceptance of a semantic-error fixture, explicit action/assignment/reference markers, whitespace/quotes/numbers/comments, byte/UTF-16/CRLF positions, prefix/recursive-import priority, empty/repeated parsing, CST containment, all guard errors, no trailing garbage and rejected unknown grammar/terminal constructs. `FIXTURES.json` records the frozen upstream-derived fixture provenance. Source-derived cases are not captured upstream runtime goldens; 15 tests do not qualify 735 declarations.

Run checks with a vacant evidence destination and the approved build directory:

```sh
python3 porting/syntax/run_checks.py \
  --target-dir "${SYSON_BUILD_DIR}/syntax" \
  --evidence-dir "${SYSON_BUILD_DIR}/syntax/evidence/new-run"
```

The runner uses one compile job, offline/locked Cargo commands, and writes source/output hashes and UTC capture. It never runs the upstream parser or scripts. Independent review, per-rule syntax qualification, exact upstream acceptance/diagnostics, full semantic conversion, realistic model import/roundtrips and browser workflows remain OPEN.
