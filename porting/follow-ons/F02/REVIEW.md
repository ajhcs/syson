# F02 review and execution attribution

Root independently inspected NamespaceImpl, MembershipComputer and
NameConflictingFilter against the fixture expectations. Review accepted the
nineteen source-derived cases, the generated-null disposition, Java alias
reference-identity observation, exclusion append order, filter side effects and
cycle traces. Review requested the positive recursive Namespace walk; it was
added before final run f02-02. No behavior changes followed that run.

The 19-case fixture set is a contract candidate. Graph expectations were reviewed
manually; passing the checker does not derive or validate graph traversal against
an upstream executable. The local filter transition checker and graph identity/
visited-state checks establish only their documented consistency properties.

The final receipt freezes all five contract-directory files, including docs.
Its before/after hashes match each other and the files committed with this PR.
All raw stdout/stderr hashes were checked against each receipt before commit.
The executed capture runner is retained at `validation/executed-runner.py`, SHA-256
`81e5c778ea926cb3320f6e8c2eea9381d4911b97462a30ee80c5eb5231e88d32`.
It is byte-identical to run_checks.py executed for both captures. f02-01 qualifies
its earlier 18-case inputs; f02-02 qualifies the final 19-case input set. Neither
capture is relabeled as an upstream oracle or a Rust implementation test.

NameConflictingFilter's source-derived transition rules originate in SysON's
Obeo implementation (Copyright (c) 2024, 2026 Obeo), distributed under EPL-2.0.
All six upstream original notices remain available in the pinned Git blobs.
