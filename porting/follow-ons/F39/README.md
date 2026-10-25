# F39 qualification packet

The implementation is `porting/reference-preflight/`. This is a source/toolchain
preparation candidate, not an upstream runtime qualification.

- `validation/tool-inventory-01` retains the initial installed Java/Node command
  outputs and metadata-only npm/PATH-Maven observations.
- `validation/tool-inventory-02` preserves that capture's hash and checks bounded
  installed Node alternatives, retaining all original version streams.
- `validation/f39-01` captures the source checker and 20 Python synthetic tests.
  Its exact five inputs and runner remain preserved before the review correction.
- `validation/f39-02` captures 21 tests after renaming npm ci as project dependency
  installation, with system tool/baseline installation separately forbidden.
  Both captures report PASS for checks and BLOCKED for execution.

Before that frozen check capture, two source/test smoke commands passed 18 tests;
their output exists in the tool transcript only. Two extra manifest-shape/closure
negative tests were then added; no failed attempt is relabeled or invented.
No lifecycle, network/package probe, build, server, database or production action
was performed. Installed tool version commands do not qualify a future build.
