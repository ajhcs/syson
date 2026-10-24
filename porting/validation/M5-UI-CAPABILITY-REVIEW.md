# Independent review: advertised scalar UI capabilities

Initial UI source: `a8dbc3dc6ee952d38fcb3916361e47c06ef45a27`. Corrected UI source: `d12a2388ba609827ad193363e294d76306379aa0`. Disposition: the two bounded action/backend inconsistencies below are resolved. Full UI, browser and upstream parity remain open.

The initial scalar inspector correctly used the shared model setter for typed properties, escaped stored/staged values and retained explicit missing/null display. Source review found two advertised actions exceeding their actual backing contracts:

1. P2: `can_rename` accepted every class with a writable declaredName, while the handler still called legacy `Transaction::rename`, restricted to Package/PartDefinition/PartUsage. Imported LiteralInteger advertised a rename form which always failed.
2. P2: parent choices used Namespace ancestry, while bounded construction still accepted only three exact parent classes. Imported ActionUsage appeared as a creation parent but the advertised package creation failed.

Independent tests in `rust/validation/tests/web_contract_regressions.rs` invoke the public native byte ABI, import/select synthetic documents, inspect actual Rust-rendered output and dispatch the advertised actions. Both tests accept either a correctly unavailable control or a successful backing action; they do not require unsupported engine expansion. Initial result: **0 passed, 2 failed**.

Owner correction `d12a2388ba609827ad193363e294d76306379aa0` routes rename through the generic declaredName setter with an explicit nonempty UI policy. Parent/definition options and handler guards agree with the then-current bounded constructor contract. Independent source diff review and the same ABI tests: **2 passed, 0 failed**. Fixed web library SHA256: `b8c145095f4264ddd3ea4d49c50f8d3f291ce3b5dd0b543e04d28b5aad68fca9`.

The native test copies the output pointer/length immediately on the same thread, within the documented ABI lifetime and before another initialize/dispatch call. No pointer is retained. This is native contract coverage; it is not a WASM/browser execution claim. No browser, display, server or implementation source was changed by this validator. Builds used inspected sources, cached dependencies, offline/locked Cargo and one compiler job.

A later registry-backed constructor capability extension must be reviewed at its own commit. These tests remain suitable when an additional parent becomes genuinely supported. They do not establish semantic validity of every accepted membership/context.

Source review also identified a separate multiline EString concern: single-line HTML text inputs cannot preserve arbitrary line breaks supplied by the model. The frontend owner acknowledged this and is preparing textarea/unchanged-line-ending handling. That fix and actual browser string preservation are pending and are **not** accepted by this report. Existing integer/finite-double/enum form parsing and required-field behavior also remain subject to the integration lane's browser checks.

Reproduce the bounded native checks after integration:

```
CARGO_TARGET_DIR="${SYSON_BUILD_DIR}/web-contract-target" cargo test --offline --locked --jobs 1 --manifest-path rust/validation/Cargo.toml --test web_contract_regressions
```

Upstream runtime run arrays remain empty and zero behavior rows are promoted to verified.
