# Native bounded syntax observation

Eight independent tests and the native CST exporter executed against published
revision `f1890270ef6d11110c5e94aa5b1134f4565e1cfa`. See `receipt.json` for actual
UTC intervals, exact source/runner hashes, commands and stream hashes.
`observation.stdout.json` contains the selected CST for the exact 37-byte
Oracle fixture. Syntax is recognized; AST and semantics are NotPerformed.

`tests.*` and `observation.*` are captured streams. Only selected build and
repository prefixes are replaced in portable outputs; raw and portable hashes
are separately recorded. Raw bytes were captured in memory; these public files
are the portable copies. This runner executed native Cargo commands only.

The separately captured bundled reference observation is in
`../parser-oracle-01`. Agreement on this fixture's syntax acceptance does not
establish semantic AST equality or promote strict behavior-ledger rows. All
735 source rule dispositions remain UNPORTED.
