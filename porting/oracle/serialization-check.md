# Check whether SysON can reopen Rust JSON

The differential oracle compares the models built from the same SysML text.
This standalone check asks what happens when the pinned SysON release receives
the Rust model itself. It uploads the Rust document as JSON into an isolated
blank project and checks the downloaded document.

Start the [reference oracle](README.md) in an owned run directory. After
running the ordinary differential command, pass its cases directory to the
serialization check.

~~~sh
python3 -I porting/oracle/serialization_check.py \
  --cases /path/to/differential-run/cases \
  --out /path/to/fresh-wire-report \
  --controls
~~~

The check uses the same isolated reference state as the differential run. It
creates and deletes its own project for each upload. Run the reference
shutdown command from the oracle runbook when done. The output directory
contains a JSON summary and one file per case with the original document,
its uploaded byte digest, the downloaded document and the verdict.

- **yes** means the current finite model's containment, scalar data, semantic
  IDs and typed references survive. The check allows SysON to assign fresh EMF
  object IDs and requires the reexported typing links to resolve to those IDs.
- **no** gives the first import refusal or model discrepancy.
- **n/a** covers a case that Rust did not import, input outside the finite
  checker or an inconclusive reference call. An attempted but inconclusive
  upload exits with status 2 and preserves the diagnostics.

The check supports the classes and attributes emitted by the finite
Package and Part importer. It checks the two local FeatureTyping references.
A model with other classes or fields needs a wider verifier. Issue
[#251](https://github.com/ajhcs/syson/issues/251) still tracks the column in the
main differential report.

## Release observation

The [v2026.9.3 run](results/json-reopen-v2026.9.3.json) returned **yes** for
all 12 documents the Rust importer produced. Rust refused the other 49
corpus inputs, which the wire check reported as **n/a**.

All three positive controls reopened: an empty package, a typed part with
elementId references, and the same typed part with EMF object-id
references. SysON assigned fresh object IDs and reexported both typed
links using the new IDs. It preserved the semantic elementId values.
The negative control pointed the type at an unknown ID. The upload
returned a document, but the reexported type failed to resolve, so the
check returned **no**.

That observation narrows the claim in
[#248](https://github.com/ajhcs/syson/issues/248). The release's document
upload path handled both encodings on these cases. Direct archive writes,
other reference families and the broader serialization issue remain open.
