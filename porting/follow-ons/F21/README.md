# F21 source preparation handoff

The session/isolation packet is under [API/session-isolation](../../area-inventories/API/session-isolation/README.md). It records 18 pinned source files, eight unresolved boundaries and twelve unrun negative scenarios before any Rust host adapter is chosen.

The immutable upstream source is `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`; this independent PR targets Rust base `ad814c232ee24109639c1d52239cbeec6f4a3e92`. It has no future-unit prerequisite and changes no existing implementation or held lease. `create_packet.py` reproduces the three documentary JSON files from pinned local source. `run_checks.py` captures the stdlib source/claim checker and its regressions. No network, credentials, dependency installation or upstream application runtime is involved.

`validation/f21-02/` records the final execution: 21 tests pass; the source checker verifies 18 pins and 12 scenario records. Qualification remains BLOCKED_SOURCE_PREPARATION_ONLY. The tests establish source packet integrity and refusal of unsupported acceptance claims; they do not test authentication, cookies, project authorization or a deployed artifact. The coordinator independently reviewed the source packet, all24 final consumed input hashes and original streams/runner; no blocking findings were reported. The source lease is released after the scoped commit.

Original capture `validation/f21-01/` retains its executed runner, source-pin check and raw streams with the six-file packet-input freeze. Final capture02 expands the frozen input closure to all18 consumed native files plus the six packet inputs, without changing checker, fixtures or tests.
