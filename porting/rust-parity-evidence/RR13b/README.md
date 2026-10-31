# RR13b snapshot fault and filesystem qualification

The complete 356-line donor adds 13 fault cases to RR13a's three admission races, eight regression cases and two connected cases: all 26 original cases remain. Historical F31 labels identify retained donor fixtures.
The suite checks ordered real filesystem callbacks, prior/candidate/staged bytes, partial writes, collisions and errors after publication or removal. An Io error can follow publication; inspect retained files before recovery. A visible snapshot does not prove durability.
Root supplies a canonical owned 0700 SYSON_TEST_DIR under /workspace, an owned CARGO_TARGET_DIR and unprivileged execution with zero capabilities and NoNewPrivs=1; fixtures remain for capture and scoped cleanup.
From the worktree, root runs `setpriv --no-new-privs cargo test --manifest-path rust/crates/syson-workflow/Cargo.toml --offline --locked --all-targets -- --test-threads=1 --nocapture` and the same command with `--release` before `--` to execute all 26 original cases and five additional real filesystem controls in both modes.
Build outside measurement: `cargo build --manifest-path rust/crates/syson-workflow/Cargo.toml --offline --locked --release --example snapshot_roundtrip_probe`.
Five genuine example tests exercise three actual permission controls and exact/over 4 MiB reads; permission modes must be restored and prior bytes verified. These controls are absent from the normal measurement binary.
Run `/usr/bin/time -v setpriv --no-new-privs "$CARGO_TARGET_DIR/release/examples/snapshot_roundtrip_probe" small-package` and repeat with `recovery-scalar-850000` for two warmups and seven raw samples per fixed workload.
Each workload records save, reopen, load and roundtrip separately; post-clock checks compare full documents, canonical bytes and fixed role identities. Saving warms the cache; process high-water memory includes setup and validation.
Root retains failures, raw samples, resource/source bindings and cleanup evidence; independent review and execution determine qualification. No hard timing veto, p95, native speedup or device-capacity claim follows from these samples.
Drop/reopen stays within one process. Writer-process recovery, application/backend restart, UI/native/SQL/HTTP parity and physical power-loss acceptance remain separate open gates; held initializer/raw-factory and human scopes remain reserved.
