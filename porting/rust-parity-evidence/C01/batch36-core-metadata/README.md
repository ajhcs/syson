# Fixed core metadata frontier

Only the pinned 56-primary/280-request manifest is admitted: 51 POMs and five version metadata XMLs, each with four sidecars.
Whole manifest and independent ordered primary seals bind exact coordinates, roles, indices, paths, statuses and caps.
The unchanged, whole-hash-pinned 261-line transport and 99-line root launcher are reused; old policies and old 28/13 suites remain unchanged.
Complete checksum validation covers POMs and metadata XMLs, including bare digests and checked GNU filename records; signatures remain unverified.

The profile is 16 MiB aggregate body refusal, 63 MiB getter metadata including 1 MiB failure reserve, and 1 MiB root logs/controller: 80 MiB logical writes.
Each POM allows 2 MiB, metadata XML 256 KiB, sidecar 64 KiB; headers allow 64 pairs/16 KiB UTF-8, response framing 4 KiB, static framing 1 MiB.
Both escaped response copies count in the actual ledger; nominal framing is 64,028,672 bytes, leaving 2,031,616 bytes after an additional 1 MiB failure reserve.
Specs/manifests each stay within 256 KiB. Actual input maps, proof, controls and inherited lock descriptor are checked; offline controls cover the full pipeline and refusals.

Root must review and freeze source, execute old/new suites normally and optimized, qualify the exact spec, and own all network/runtime/Git/CI/archive actions.
Preparation grants no acquired code trust, extraction, classloading, Maven/JVM/compiler/native execution, cache installation, or completion promise.
Physical quota, polling, full RR16, parent-SIGKILL, parent/BOM/range closure, mediation, core compile and full native/UI/storage/Rust acceptance remain unproved; all holds remain.
