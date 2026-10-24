# Inspected offline dependency closure

Before execution: read normalized Cargo manifests, license metadata/files, and full build.rs for active dependencies.

| Crate | License | Build script effect | Manifest SHA256 |
| --- | --- | --- | --- |
| serde_json-1.0.151 | MIT OR Apache-2.0 | Reads target architecture; emits Cargo cfg | 00b920255c320a278bf25b962b491263a19d8c90e12e4a91c1d823db71eabb2f |
| serde_core-1.0.229 | MIT OR Apache-2.0 | Writes OUT_DIR/private.rs; rustc --version and Cargo cfg | c6049e170e4f7c2b48749fefab59db583bfac4e466cde6d0c53e4f49bc5044f7 |
| itoa-1.0.18 | MIT OR Apache-2.0 | No build script | 894c215e4e7e973b556b0e9627bf9b6a719a043fac9b5af9838660816392a09f |
| memchr-2.8.3 | Unlicense OR MIT | No build script | a14f68b8fa55f15fd03b15dced37bcf66a0a628f568eedc82817cdb7f9281951 |
| zmij-1.0.23 | MIT | rustc --version and Cargo cfg | b31420c7a806fae5b9ea2438dd4cbb99dcb3a22da67697fdf2776cc44a69527e |

Only serde_json is direct, exact-pinned =1.0.151; Cargo.lock pins the closure.
Default std + arbitrary_precision features. No derive proc macros, no optional
indexmap/no-panic/log, no dev dependencies of external crates, no network calls.
Cargo runs with explicit cached CARGO_HOME and --offline. No crate vendoring or
third-party source redistribution is performed by this adapter. Existing upstream
LICENSE/NOTICE files remain authoritative for the checked-in SysON fixtures.
