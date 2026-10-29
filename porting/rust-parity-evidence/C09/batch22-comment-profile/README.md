# Comment-heavy finite parser API profiler

This development-only tool measures actual `Adapter::construct` on a named source candidate. It is not a production caller, native comparison, browser benchmark or capacity qualification. The recorded candidate includes the P01 scanner and native-observed hidden-only root-span repair; the original source remains pinned in the evidence closure.

Build a fresh owned Cargo package on `/workspace` with edition 2024, pinned `serde_json = "=1.0.151"` and an explicit path dependency on the candidate's `syson-ast-adapter`. Point its `p01-profile` bin at this source. Use a reviewed installed toolchain and offline/locked release build. The archived executed manifest and runner retain exact original paths and tool hashes; they are execution records, not a portable launcher. Use distinct baseline/candidate target directories.

Run `p01-profile 64|8192|65536|65537 [--note-only] [--cancel-before]` in a fresh process each time. For each size and mode retain two warmups and seven measured values; p95 requires at least 30 measured samples and a declared estimator. The default mode is one ordinary note followed by a Package; `--note-only` exercises the hidden-root text copy. Source recipes, limits, actual root bounds and expected typed results are explicit in JSON. Expected source-limit/cancellation controls are distinct from successful construction.

Only the public construct call is timed. Counting-System allocator traffic, absolute/incremental live peaks and whole-process VmHWM remain separate. Setup is outside the timer; counters precede `/proc`; error and JSON formatting follow memory observations. Precancellation is not proof of interior cancellation. Seven-sample API ranges do not establish user-visible latency, native equivalence or supported capacity.

The evidence retains all 72 raw samples, two precancellation and three invalid-CLI controls, exact source/binary hashes and independent review. Raw captures remain unchanged. Compiler/sysroot completeness, actual application/UI and full-language acceptance remain open.
