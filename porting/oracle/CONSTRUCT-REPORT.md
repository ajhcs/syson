# Group finite importer refusals by grammar construct

Run the Rust validation example on a completed differential report directory.
It reads `summary.json` and the matching `cases/*.json` files and prints a
separate Markdown report with the **Blocked by construct** table and one row
per Rust refusal. Write it next to `SUMMARY.md` so the case links work.

```sh
cargo run --offline --locked \
  --manifest-path rust/validation/syntax/Cargo.toml \
  --example classify_import_refusals -- "$REPORT_DIRECTORY" \
  > "$REPORT_DIRECTORY/CONSTRUCTS.md"
```

Set `REPORT_DIRECTORY` to the output of the [differential oracle](README.md).
This step uses the report's recorded outcomes and needs no running SysON
server. The report counts every input case. It treats the first refusal
as one observed blocker. A later construct in the same input may also
need support.

A classification requires the importer to report a byte offset and the
full syntax recognizer to return a concrete CST rule covering that offset.
The rule and lexer token must match a minimal inline case that also refused
at its matching rule and token in the same differential run. For example,
`AliasMember` with `alias` is backed by the existing `alias` case. The
report links to the actual case and carries the zero-based UTF-8 byte offset
and one-based line and UTF-16 column. These positions handle CRLF as
one newline. A typed part usage body requires a CST typing before the
body. An untyped part usage body reaches a different importer refusal
and remains unclassified here.

`Malformed` and `UnresolvedLink` retain the importer diagnostic category.
`ImporterCapped` records an importer limit. `RecognizerCapped` and
`SyntaxRejected` mean the recognizer could not produce a CST for attribution.
A recognized input with no supported mapping is `NoVerifiedRule`. Matching concrete rules with equally small spans and
different meanings are `AmbiguousRule`. A partial report that omits the
required repro leaves that candidate at `ReproUnverified`. The recognizer
provides one chosen derivation, so these labels cannot establish a unique
parse across all derivations or diagnose semantic linking.

Run the targeted controls with this command.

```sh
cargo test --offline --locked \
  --manifest-path rust/validation/syntax/Cargo.toml \
  --example classify_import_refusals
```

Automatic inclusion in the main differential summary remains part of
[#256](https://github.com/ajhcs/syson/issues/256).
