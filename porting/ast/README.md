# Transactional AST construction kernel

`rust/crates/syson-ast` implements generic construction operations as a standalone, std-only Rust crate. This is a bounded Stage 1 implementation of the source plan in `porting/source-map/syside-source/AST-VALUE-CONVERSION.md`. It does not consume a grammar or CST and does not change an existing recognition result's `SemanticStatus`.

The completed `Arena` is immutable through its public API. Node IDs are owner-qualified and remain stable, including after rollback: failed branches consume IDs and work budget, and cannot alias a later node. Nodes expose scalar, node, flat array and unresolved-reference fields. A missing field is `Slot::Absent`; explicit `Value::Null` remains a present field. Optional raw lexemes retain both UTF-8 byte and UTF-16 code-unit ranges. An absent lexeme permits genuinely synthetic values; it does not assert a source token existed. `Number(f64)` is an input value slot, not an implementation of JavaScript Number parsing or formatting.

Builder::new returns Result so owner-namespace exhaustion fails explicitly. Numeric NodeId::value() is arena-local; only the private owner-qualified handle can access nodes. See OWNERSHIP-FIX.md for the independent-review regressions and fixed-version evidence.

A builder supports nested transactions, node/datatype frames, `=`, `?=`, `+=`, action frame replacement, non-datatype subrule adoption, zero-length composite-CST assignment suppression, source-first array merge and caller-declared reflection defaults. Arrays default when their prior slot is not an array; booleans default only when absent, preserving explicit null. Default metadata is deduplicated by first property name, but inheritance calculation/type-index semantics remain the caller's obligation. Unknown types are rejected rather than invented.

`finish` derives reachable container/property/index links, rejects cycles and repeated containment, and prunes unreachable temporary/merged nodes. CST identity associations are explicit; the caller must identify retargeted nodes. This is not the source parser's automatic CST walk or child ordering algorithm. The arena has no fabricated `$meta`, resolved reference target, scope, semantic child order, validation result, CLI projection or SysON model import result.

Work, node and frame limits and a shared cancellation flag produce explicit errors. Transactions roll back mutable construction state on returned errors. Consumed work/IDs are deliberately retained to bound failed-branch work. Panics are not a supported rollback protocol. Caller-owned input allocation, transaction snapshot cost and callback work are not individually metered; callers must bound event/input sizes and do not gain a general sandbox from this API. Nested arrays are rejected because the declared kernel slots use flat property arrays; this is an explicit supported-shape restriction.

## Source traceability

| Operation | Exact behavioral source | Native evidence |
| --- | --- | --- |
| Assignment replacement/presence/append and raw quote retention | Pinned bundle Langium parser module at 40939, assignment around 41147; converter module at 41808 | replacement/append reset, presence raw evidence, quoted/astral offsets |
| Source-first merge and adopted type defaults | Pinned bundle merge around 41172; Syside override around 108999; candidate parser.ts:326 | source-first arrays, target scalar preservation, untyped adoption |
| Action frame replacement/capture | Pinned bundle action around 41093 | nested wrapper containment; no-feature untyped-frame replacement |
| Datatype concatenation | Pinned bundle consume/subrule construction near 41015–41145 | `typed` + `by` becomes `typedby` |
| Subrule adoption and empty-CST suppression | Same Langium parser module, performSubruleAssignment/consume/subrule methods | active frame adoption; zero-length assignment omission |
| Mandatory properties | Pinned bundle assignMandatoryProperties near 41126; candidate reflection.ts:108 | invalid-array reset, absent bool default, explicit-null preservation, first-name metadata |
| Arena IDs/transactions/final containment/bounds | Proposed Rust architecture, not an upstream API or exact parser rollback model | branch rollback, nonreused IDs, cycles/shared children, node/frame/work limits and cancellation |

Pinned bundle: SysON commit `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`, `backend/application/syson-sysml-import/src/main/resources/syside-cli.js`, SHA256 `47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd`.

Candidate service source: Syside `1326f234cae23b0e4777d5a48a069b17e9e332f1`, `packages/syside-languageserver/src/services/parser/parser.ts` SHA256 `4cba11ff49975d47bcd6c5aba6d9cb55c5a06d84993dc6ac2effffa902a67d61`, `src/services/sysml-ast-reflection.ts` SHA256 `9f43aba6051f1ef68b36730212e5ebdbb906816fac746a644be7cd6e89c9607b`. Retained full texts are in `semantic-source-documents.jsonl`. Candidate service-to-bundle correspondence beyond the grammar declarations remains unproven. The crate NOTICE and UPSTREAM-NOTICES.json preserve the exact retained TypeFox/Langium MIT section, Syside source header and complete candidate source license separately from the new Rust EPL license; embedded artifact/dependency notice closure remains OPEN.

## CST/evaluation adapter proposal

An adapter must supply a qualified construction event stream, rather than walk the first available Earley derivation. Its input contract needs exact grammar identity/rule paths, chosen alternatives, source token and composite-CST spans, recovery-inserted token markers, fragment/datatype flags, assignment/action placement, and reference target-type metadata. The event stream must distinguish explicit tokens from synthetic nodes and carry an independent selection/recovery evidence status.

A proposed adapter sequence is rule entry, terminal/keyword conversion, nested subrule completion, action or assignment, rule completion, and final freeze. Non-datatype unassigned subrules call `adopt_subrule`; assigned subrules use the actual composite-CST length. Terminal/crossreference conversion must follow the exact reachable rule dispatch. STRING_VALUE and UNRESTRICTED_NAME remain raw quoted strings at this layer. Numeric conversion/JavaScript stringification remain unsupported until explicitly implemented and tested. Recovery-inserted tokens require omission by the adapter; the kernel receives no lexer/parser tokens from which to infer it.

Transactions isolate a branch's state changes, but do not qualify which branch Langium/Chevrotain would select. Ambiguous choices, unsupported recovered constructs, unknown reflection, implicit-node hooks or unavailable primitive conversion must return an explicit adapter error. `Builder::unsupported` is available for a required semantic construct the adapter cannot implement; it must not be interpreted as permission to skip it. Automatic hook dispatch, CST subtree retarget walking, collectChildren/range sorting, inherited reflection generation, all scope/linker/model services and CLI/import serialization remain outside this milestone. A successful operation sequence is construction evidence only, even if syntax recognition also succeeded.

All 848 per-declaration AST obligations and all 735 rule parity statuses remain UNPORTED. These native tests exercise generic source-derived operations and Rust safety boundaries; they do not constitute runtime differential testing or grammar coverage. No upstream code was executed.

## Checks and captured evidence

Run with an existing Rust toolchain, no dependency downloads:

```sh
CARGO_TARGET_DIR="${SYSON_BUILD_DIR}/ast" CARGO_BUILD_JOBS=1 cargo test --offline --manifest-path rust/crates/syson-ast/Cargo.toml
CARGO_TARGET_DIR="${SYSON_BUILD_DIR}/ast" CARGO_BUILD_JOBS=1 cargo clippy --offline --manifest-path rust/crates/syson-ast/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path rust/crates/syson-ast/Cargo.toml -- --check
```

The original `native-run.json` records actual UTC intervals, exact code/input SHA256 values, tool versions, commands, exit codes and output hashes for the final 19-test run and checks. Committed outputs trim trailing empty lines and replace checkout/build path prefixes with `${SYSON_CHECKOUT}` and `${SYSON_BUILD_DIR}`; raw capture hashes remain separately recorded. The initial 14-test passing run is retained separately and is not the final source snapshot. No historical timing is inferred from Git commit time.

The fixed-version execution is recorded separately in `ownership-fix-run.json`; earlier 19-test evidence remains historical and unchanged.
