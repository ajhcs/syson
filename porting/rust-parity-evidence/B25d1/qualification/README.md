# B25d1 action/control migration qualification

This packet qualifies the literal-safe Rust participant API on its recorded finite inputs.
The full corrected 200-line API and six original direct cases are retained byte for byte.
Four additional literal, opaque-value, duplicate-key, boundary and late-refusal cases pass.
Actual B01 handoff preserves separate participant and representation-ID watermarks.
All 48 migration tests pass in debug and release; fmt, all-target Clippy and WASM checks pass.
Reproduce library checks with `cargo test --locked --offline --manifest-path rust/crates/syson-migrations/Cargo.toml --all-targets`.
Use the same command with `--release` for the release test mode; the receipt records exact tools and environment.
The release probe runs small and 1,024-control workloads with two warmups and seven raw samples each.
Three fresh probe processes retain all raw values and min/median/max; no p95 is inferred.
Prepare timing includes internal output validation; setup and later serialization are outside that interval.
GNU time RSS covers each whole process, including both workloads, setup, checks and serialization.
These measurements establish neither allocation high-water marks nor native/UI speed or capacity.
The first unexecuted check specification lacked a direct launcher pin; its HOLD and correction remain preserved.
Source captures with `.raw` suffixes are passive provenance, not execution entry points.
The original seventh case and entire 19-row corpus remain mandatory B25d2 work, retained here only as provenance.
Native framework history, SQL/HTTP/UI/backend restart and full B25d/RR24 acceptance remain open.
The earlier main-CI timeout is retained in main evidence; merged repair #236 passed all 16 required checks.
