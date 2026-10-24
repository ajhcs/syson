# Bounded embedded parser reference runner

`run_reference.py` invokes the exact pinned embedded CLI in its secondary diagnostic mode. It is a reference measurement tool, not part of the Rust product and not an execution-approval mechanism. Review the selected fixture corpus and effects before invocation. Execute in the default network-restricted boundary; escalation is unsandboxed and must not be described as network-restricted.

The runner checks fixed existing Node/bundle SHA256 values and the supplied fixture hash before invocation. A fixture must be UTF-8, at most 1 KiB, with `.sysml` or `.kerml` suffix. The Node identity is the verified existing v24.21.0 binary from M7; it is not the application's exact Node24.20.0 engine pin. It removes inherited `NODE_*` variables, sets `FORCE_COLOR=0`, leaves other environment including HOME/CODEX_HOME unchanged, closes stdin and applies a 60-second subprocess timeout. No installation, download, port, service or persistent state is configured by the runner.

The exact child arguments are:

```
<verified-node24>/bin/node <pinned-bundle> dump --validate --stdlib none <approved-fixture>
```

The runner itself writes no files. It captures actual UTC, exit/timeout and separate raw stdout/stderr; JSON output contains zlib-compressed base64 streams, byte counts and SHA256 values. Compression preserves bytes and avoids dumping large AST text into the conversational transcript. A separately reviewed persistence step decodes the report, verifies every raw hash, retains raw evidence in the approved build directory and generates explicitly transformed portable public evidence. Do not truncate streams or infer success from the wrapper's exit: the child exit is in the report. A child timeout or uncertain failure requires inspection before any retry.

Read-only review of the original scoped path found no expected filesystem writes, network requests, child processes or listening service. That source conclusion is not runtime effect instrumentation. `stdlib none` is not a general filesystem/network guard. This runner imposes no whole-process memory or output-size guarantee; corpus and command bounds are separate from the sandbox restrictions. It does not enforce a case allowlist or prohibit imports itself. Pin and keep the reviewed corpus immutable through execution, reject truncated/undecodable reports, and stop on timeout or unexpected effects. The wrapper exits zero after reporting an expected/nonzero child outcome; inspect the recorded child exit.

With `--validate`, the embedded CLI can exit 1 on severity-one diagnostics before it serializes the AST. Missing stdout AST therefore means unavailable through this route, not proof that no recovered parser AST existed. Keep lexer/parser errors, semantic diagnostics, process outcome, emitted AST and Rust syntax/AST states separate. The primary Java import command and recovered-AST inspection need their own reviewed scope.

Example shape, after source/effect review and action-time authorization:

```
python3 porting/parser-oracle/run_reference.py \
  --node <verified-absolute-node24> --bundle <verified-absolute-bundle> \
  --fixture <approved-absolute-fixture> --fixture-sha256 <exact-sha256>
```

This preparation does not run the upstream CLI. M7's earlier one-fixture execution used the separately documented controlled parent and is not retroactively attributed to this runner. All grammar/AST/semantic obligations remain unported until their own implementation and evidence qualify them.
