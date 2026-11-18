# Draft PR triage, 2026-11-18

On 2026-11-18, 192 of the 195 open pull requests were closed without merging, following the proposal in [#454](https://github.com/ajhcs/syson/issues/454). **No branch was deleted.** Each PR's branch and head SHA are listed in [`index.json`](index.json), so its content stays reachable with `git fetch origin <branch>` or by SHA. Each closed PR has a comment naming its successor on `main` or the issue that now carries the work.

| Disposition | Count | Meaning |
|---|---:|---|
| landed | 26 | Content is on `main` through the linked successor PR |
| partly landed | 4 | Part is on `main`; the rest is prior art for the linked issue |
| fix on main | 2 | Worth re-landing; tracked as #456 and #457 |
| prior art | 45 | Code that never reached `main`; the linked issue references it |
| historical | 27 | Process, contract and closure records |
| evidence | 88 | Verification records from #307–#395 |
| hold | 3 | Left open: #77, #96, #287 (source handoff pending) |

## Results cited in the 2026-11-18 maintainer handoff

These evidence PRs back the results the handoff describes. Their records stay on their branches:

| PR | Result | Branch | Head |
|---|---|---|---|
| #375 | Verify Rust snapshot readback for released SysON invocation models | `codex/syson-invocation-rust-readback-268` | `4a32c1bcadd0` |
| #376 | Verify Rust edits preserve captured SysON invocation links | `codex/syson-invocation-edit-reopen-268` | `32f801c81f3a` |
| #381 | Verify seeded invocation edits across Rust HTTP service restart | `codex/syson-invocation-seeded-service-268` | `f6ad3d6861a9` |
| #385 | Verify a read-only SysON bundle with a separate invocation document | `codex/syson-invocation-multi-document-archive-268` | `72685155d699` |
| #386 | Record official SysON commit before browser response loss | `codex/syson-reference-no-ack-commit-abort` | `aa08dbfc986c` |
| #387 | Record paired prepared-project wire handoff | `codex/syson-c03-paired-handoff-evidence` | `bad0efa31317` |
| #392 | Record official SysON recovery from a stopped database clone | `codex/syson-reference-no-ack-clone-reopen` | `9a68544f5998` |
| #393 | Record Rust lookup of an archived native B06 row | `codex/syson-b06-archived-row-resolution` | `7345aa3549ea` |
| #394 | Record the Eclipse Filesystem default descriptor reader | `codex/syson-c01-eclipse-filesystem-default-reader` | `9034a7af1fb6` |
| #395 | Stage the Eclipse Filesystem child metadata roster | `codex/syson-c01-filesystem-child-metadata-roster` | `042b419a5a88` |

These records describe runs on the maintainer's machine. They are evidence for the specific claims in their titles, not application-level parity. See [`porting/PARITY-GAP.md`](../PARITY-GAP.md).
