# Indexed document reconciliation prerequisite

This adoption brings the complete finite planner from [PR #60](https://github.com/ajhcs/syson/pull/60), head `a85366ae8252596b417384d02e0ded6784d93eed`, onto the current Rust foundation. The original production API and all 16 tests are preserved. The planner distinguishes reused observations from rebuilt documents, preserves caller order, and reports required timestamp/event effects. The consuming service must execute the resulting plan. This is a prerequisite for the storage work that uses `syson-document-reconcile`.

The [donor inventory](donor-inventory.json) records the retained source and evidence. Historical F25 and indexing-repair captures include the original failures and benchmark; their `.raw` files preserve those observations without rerunning their scripts. Independent source/build correspondence with the deployed native application remains unresolved.

The current release probe calls the real planner for 1,000, 5,000 and 10,000 unique unchanged documents. It builds each immutable fixture before timing, performs two warmups, then emits all seven elapsed nanosecond values in execution order and their min/median/max. Every call asserts an unchanged plan. Results describe this API workload on the recorded host; seven samples do not establish p95, allocation or RSS, UI latency, native speedup or application capacity. Timing observations inform review without a hard timing cutoff.

From the repository root, use the following commands with a coordinator-admitted Cargo target directory on the workspace:

```sh
cargo fmt --manifest-path rust/crates/syson-document-reconcile/Cargo.toml --check
cargo test --offline --locked --manifest-path rust/crates/syson-document-reconcile/Cargo.toml -- --test-threads=1
cargo clippy --offline --locked --manifest-path rust/crates/syson-document-reconcile/Cargo.toml --all-targets -- -D warnings
cargo run --offline --locked --release --manifest-path rust/crates/syson-document-reconcile/Cargo.toml --example reconcile_bench
```

[qualify.py](qualify.py) accepts a trusted, coordinator-reviewed JSON command/input specification under the owned launcher; scope lifetime and cleanup remain the launcher's responsibility.
It checks declared source hashes before/after and named command/test outcomes, without establishing full RR16 closure. F13/B01 reuse these exact helper bytes after the F25 adoption PR lands.

Current execution results belong to the coordinator's source-bound receipts. This README records no unexecuted check as passing. Durable commit, PostgreSQL behavior, restart/reopen, real storage callers, UI/native comparison and full feature acceptance remain open. Existing source holds and dependency gates are unchanged.
