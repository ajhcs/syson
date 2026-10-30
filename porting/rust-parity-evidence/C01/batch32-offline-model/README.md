# Frozen offline native reactor-model replay

This job repeats the native Maven Help 3.5.1 `effective-pom` query across the complete pinned 36-model SysON reactor. It uses the existing 309-file cache and verifies all 3,552 target source files. A successful result establishes frozen offline model replay. Compiling or running SysON, binding the complete inherited source/artifact closure, and C01 feature acceptance remain open.

The integration coordinator freezes the proposal and all input hashes before execution. The proposal supplies the target manifest and POSIX modes, exact JDK/Maven distribution manifests, cache file hashes, prior model/debug captures, compiler lock and a fresh output root on the workspace. The prior XML stays outside that output root as a bound read-only input. Existing attempts, target sources and original cache remain unchanged. The entrypoint imports only pinned, already published tool-verification and invocation-ownership providers.

Maven receives `-o`, explicit offline settings, the original Central mirror identity, a clean environment and the fixed model goal. No online retry or alternate registry follows a failure. This constrains Maven resolution; it does not provide kernel network or filesystem isolation. The model comparison preserves namespace-qualified tags, attributes, child order and meaningful text, including tails. Leaf text is preserved exactly. Structural-parent indentation is ignored only when the parent has no nonblank text or child tail and inherited `xml:space` permits it; `preserve` inheritance and `default` resets are honored. XML comments are ignored. All 36 identities and the ordered 13-entry Help classrealm must match the bound prior result.

The coordinator admits one owned scope with a 200% CPU quota, memory high 3 GiB/max 4 GiB, no swap and 128 tasks. The body reads those actual controls. One compiler lock remains held through proof-bound cleanup. Each child file has a 32-MiB limit; total output has a polled 64-MiB threshold whose overshoot is retained. Maven has 600 seconds, the outer wait 660 seconds and the scope 720 seconds. These are execution-containment limits, not product performance criteria. Failures retain raw streams and terminal receipts; unknown scope identity is preserved rather than killed.

After independent source review, the coordinator supplies `SYSON_OFFLINE_SOURCE_SHA256` and an existing admitted workspace directory in `SYSON_OFFLINE_TEST_TMP`, then runs:

```sh
C01_PACKET=porting/rust-parity-evidence/C01/batch32-offline-model
python3 -B -m unittest discover -s "$C01_PACKET/tests" -v
python3 -O -B -m unittest discover -s "$C01_PACKET/tests" -v
python3 -I -B "$C01_PACKET/replay.py" --run "$C01_PROPOSAL" "$C01_PROPOSAL_SHA256"
```

The unit controls use owned fixtures and fake Python processes. Hosted CI may supply its admitted temporary directory when `GITHUB_ACTIONS=true`; only the environment unit test supplies a mocked operator identity. Production workspace, identity and scope guards remain unchanged. Actual Maven/JVM execution belongs to the coordinator after fresh source, tool, cache and host-headroom admission. Record the native outcome, exact comparisons, input stability and cleanup separately; this source packet makes no unexecuted check a pass. Existing source holds, the frontend package entitlement blocker, full native build, database/UI workflows and release acceptance remain unchanged.
