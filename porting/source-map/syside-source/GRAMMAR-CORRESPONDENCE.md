# Embedded declaration correspondence

The static checker establishes a specific correspondence: both complete serialized KerML and SysML Grammar objects embedded in the pinned SysON CLI bundle equal the retained Syside 0.9.0 candidate Grammar objects after static template decoding. No JavaScript is executed.

Inputs are fixed independently:

- SysON commit `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`, bundle `backend/application/syson-sysml-import/src/main/resources/syside-cli.js`, SHA256 `47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd`.
- Retained candidate source commit `1326f234cae23b0e4777d5a48a069b17e9e332f1`, `packages/syside-languageserver/src/generated/grammar.ts`, source-file SHA256 `fca1a64377f64c1aefb5476e89227cbc24f064ca40b7a58d6608646cae66149d`.

The bundle's esbuild call spelling is normalized from `loadGrammarFromJson)(` followed by a template delimiter to the direct call spelling understood by the existing static extractor. This changes no template contents or newlines. All three embedded grammar objects must be present exactly once: LangiumGrammar, KerML and SysML. Only KerML/SysML are the two correspondence subjects; the Langium grammar is not silently mistaken for one of them.

The checker validates root/rule/list shape, distinct rule names, local references, candidate source metadata and agreement between retained encoded-template, decoded-JSON and parsed-object forms. It then requires canonical object equality and pinned hashes. Canonical serialization is Python JSON with sorted keys, compact separators, ASCII escaping and UTF-8 encoding, without a trailing newline in the hashed content.

| Subject | Bundle source line | Canonical declaration SHA256 |
| --- | ---: | --- |
| KerML: 282 rules | 45569 | `2b439aeca16aa8f5247f74c541278f93dfefea44098ec8fe1b6f85ac31013829` |
| SysML: 453 rules | 60250 | `84e3deb1cbcc89c400b56e7b2a0a20eaa64ff6305146bf34edd982ab292898f8` |

`baseline-grammar-correspondence.json` is reproducible evidence of those two equal declarations. The checker rejects full-bundle hash drift, source metadata/shape drift, inconsistent retained representations, unresolved references, duplicate/missing grammars and canonical declaration hash drift. Tests include a re-encoded mutation that keeps the retained representations mutually consistent but still fails the pinned declaration hash.

Run from an existing checkout containing the pinned SysON Git object:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 porting/source-map/syside-source/check_baseline_grammar.py
PYTHONDONTWRITEBYTECODE=1 python3 porting/source-map/syside-source/test_baseline_grammar.py
```

`--write` regenerates the receipt in this subtree; the default checks the checked-in receipt. `--syson-reference` can select another existing Git checkout containing the same pinned object. No dependency download, parser process, source generator or build is invoked.

Eight static comparison/drift-rejection tests pass. This closes the earlier uncertainty about **these two serialized grammar declaration objects**. It does not prove whole-bundle reproducibility, exact source correspondence of parser patches/lexer/runtime/value converters/metamodel/linking/validation, or completeness of recovered dependency notices. It does not prove upstream-versus-Rust parse decisions, ASTs, values, errors, libraries, import conversion or semantic behavior. A separate reported single synthetic Node-oracle acceptance does not broaden this static result and is not generated or validated by this checker. Full artifact, notice and behavioral provenance remain OPEN; all 735 rule parity statuses remain UNPORTED.
