# C03a: Verify exact migration-source compositions in Rust

Open independent PRs need reproducible source assembly before integration. Add a
read-only Rust CLI that checks local base/source/candidate refs and commit/blob
identities, every selected regular file and fixture, declared ownership and exact
ordered common-export generation. Missing/undeclared source changes, conflicts,
stale heads, altered candidate bytes/modes, symlinks and escaped paths refuse.

Git calls use structured arguments without hooks, filters, replacement objects or
lazy fetching. Reads and selections are bounded. Source PR common files must be
base plus only their own declared exports; the candidate must be base plus all
ordered exports. The verifier never changes refs, index or user configuration.

Validation: eight isolated real-Git regressions, formatting and all-target Clippy
pass offline/locked. The corrected actual CLI verifies93selectedfiles+1generated
common file on immutable snapshot `f19c06915a664eb636f6d0782c6d5191f9a43cd3`.
The composed Rust migration library passes29distincttests with all97compiler
inputs checked against committed blobs and no source drift. Earlier7-case PASS,
peer owner-execute-bit finding and final8-case/runtime captures stay retained.
The final CLI runner was archived; the earlier inline runtime wrapper was not.

This qualifies selected source composition. Untouched inherited/untracked working
files, remote PR freshness, whole C03 landing, application/UI/SQL/native parity
and production acceptance remain separate. Parent C03 stays open. The coordinator
approved482production/784authored Rust lines to keep the Git safety boundary in
one reviewable devtool.

Base: `codex/syson-rp-migrations-composed-base`, exact
`f19c06915a664eb636f6d0782c6d5191f9a43cd3`, containing source from #72–#74 after
#71. Land prerequisites through review, then retarget, inspect and revalidate.
Never merge into the temporary composition branch. Two commits separate source
and regressions from exact execution evidence. No push or merge by the author.
