# Syside parser source audit: OPEN

Candidate source: `sensmetry/sysml-2ls` tag `0.9.0`, commit `1326f234cae23b0e4777d5a48a069b17e9e332f1`. This is the intended candidate parser version; it is **not proof** that SysON's embedded CLI was built from this exact source. The reference destination was vacant, the download used a filtered/no-checkout clone, tracked agent/skill instructions were inspected (none found), and the checkout is detached and clean. No upstream script, build, parser, test, generator, language server or extension was executed.

## Retained declarations and source

| Evidence | Coverage and limits |
| --- | --- |
| `source-files.jsonl` | All 525 tracked candidate source paths, sizes and SHA256. |
| `grammar-source-documents.jsonl` | All five exact Langium source files: KerML, SysML, their interface declarations and shared expression grammar. |
| `generated-grammar-documents.jsonl` | Both complete serialized Langium Grammar JSON documents embedded in `src/generated/grammar.ts`, including original encoded template text, decoded JSON and full declaration trees. Static template decoding does not execute JS, Langium or Chevrotain. |
| `grammar-rules.jsonl` | All 735 generated per-language rule declarations with complete nested contents, rule/terminal dependencies, structural source fragment, original `.langium` declaration candidates and explicit `UNPORTED` status. No rule lacks an original source candidate. Multiple candidate paths do not prove import/override precedence. |
| `grammar-ast-declarations.jsonl` | All 303 per-language interface/type declarations from the serialized grammars, retaining attributes, inheritance and reference declarations. These overlap between the two grammars; they are not 303 unique SysML metamodel classes. |
| `grammar-features.jsonl` | Nested source index for actions, assignments, crossreferences, rule calls, regex tokens and keywords. Full constructs, cardinalities and operators remain in the rule/document trees. |
| `grammar-references.jsonl` | All 2,720 serialized local grammar references and resolved target names/kinds; no unresolved local reference. This is declaration resolution, not model name linking. |
| `semantic-source-documents.jsonl` | 349 exact source documents for parser services, semantic metamodels, scope/linking/name providers, workspace builders, validation, utilities, generated AST types, CLI and standard-library configuration. Includes source notices. |
| `validation-decorators.jsonl` | 143 syntactic validation decorator candidates with exact argument text and source line. Decorator bounds/defaults, inheritance, dynamic registration and execution remain unresolved; candidate count is not validation coverage. |
| `tests-and-fixtures.jsonl`, `test-fixture-documents.jsonl` | All 83 indexed test/testing/fixture source locations, full text and hashes. Expectations are source-derived, not captured running outputs. None run. |
| `syson-import-adapter-source.jsonl` | 37 exact pinned SysON import/translation/diagnostics/schema source documents. This adaptation layer remains essential even after parsing succeeds. |

Generated declaration counts by language are:

| Grammar | Parser rules | Terminal rules |
| --- | ---: | ---: |
| KerML | 270 | 12 |
| SysML | 441 | 12 |

These include fragments and shared/imported declarations per language. They are not unique syntax features or implemented Rust behaviors. The rule trees contain 783 assignment nodes, 61 AST action nodes and four crossreference occurrences. Original groups, alternatives, parser/terminal calls, keywords, regular-expression text, cardinalities, return/inferred types, hidden-token flags and AST interfaces are retained. Serialized regex text still requires the exact Langium loader/regex semantics before use in a Rust lexer; it must not be assumed to be a ready-to-compile Rust regex.

`inventory_syside.py` reads an existing exact clean reference and public release ZIP data. It requires explicit `--reference`, `--assets` and `--syson-reference` paths; use portable environment variables such as `${SYSIDE_REFERENCE}`, `${SYSIDE_RELEASE_ASSETS}` and `${SYSON_REFERENCE}`. It never downloads or runs upstream code. Source acquisition and generation are separate operations.

## Release notice/provenance recovery

Public release metadata for `0.9.0` lists two assets: `sensmetry.sysml-2ls-0.9.0.vsix` and `syside-languageserver.zip`. Their original metadata is retained in `release-metadata.json`; upstream asset digest fields were null at capture. The acquired archive hashes are recorded in `release-archives.jsonl`. These are hashes of acquired bytes, not signed build attestations.

Archives were inspected using bounded ZIP reads only; nothing was extracted or executed. `release-members.jsonl` hashes every file member. The language-server JS and generated `LICENSE` in the standalone ZIP are byte-identical to the node language-server JS and node license in the VSIX. Six exact notice documents are retained in `release-notice-documents.jsonl`, including generated node/browser dependency notices, root license text, `sysml.library/LICENSE.GPLv3` and `LICENSE.LGPLv3`. Original source notices are also retained in `source-notice-documents.jsonl`.

**No release member matches SysON's embedded CLI bundle SHA256** `47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd`. A language-server release asset is a different entry-point artifact. The recovered generated server notices are candidate provenance evidence; their coverage of the exact CLI bundle is OPEN. Do not substitute the release license file as a claimed exact matching CLI dependency manifest. Matching original build inputs, transformations, local modifications, CLI entry-point dependencies and generated notices still need evidence.

## Standard libraries and licensing

The VSIX contains 94 textual `.sysml`/`.kerml` library files. Exact source and hashes are retained in `release-library-documents.jsonl`; GPLv3/LGPLv3 texts from the same library directory are retained with their original paths. This preserves supplied material and does not assert legal clearance for all possible distributions.

Candidate `packages/syside-base/src/stdlib.ts` declares version `2024-12` and lists exactly the same 94 library paths. Its tree/raw URLs use the mutable `fixes` branch of `daumantas-kavolis-sensmetry/SysML-v2-Release`; a label `2024-12` does not pin those external bytes. `library-provenance.json` records this gap. The release archive pins acquired library bytes but does not establish their original Git commit, their relationship to SysON's bundled JSON libraries, or conversion semantics. No external library updater/download script ran.

The candidate source has EPL/secondary-license declarations; exact original texts and headers remain authoritative. Do not collapse those, generated third-party notices, or the library LGPL/GPL obligations into a credit line or infer licenses solely from file extensions/package names.

## Observed checks

Ten offline extraction/data checks passed: clean exact reference and full tracked index, exact five grammar sources/two JSON documents, strict static decoder rejection of dynamic code, complete rule/interface/action/crossreference retention, all local grammar reference targets, semantic/fixture source bytes, archive/notice integrity, all 94 library names/bytes with explicit mutable-source gap, and server sibling artifact equality with baseline CLI mismatch. These checks execute only this audit's Python code and Git/ZIP reads. They do not run the Syside parser or prove language correctness, diagnostics, runtime equivalence, license closure or a Rust implementation.

See `rust-parser-replacement.md` for the complete replacement obligations. Full source/artifact/behavior parity remains OPEN.
