# C03a: Explicit source composition verification

The Rust development CLI verifies a reviewed, same-base composition manifest
against local Git refs, immutable commit/blob identities and selected candidate
working-tree bytes. It uses structured read-only Git arguments with replacements,
lazy fetch, optional locks and external diff/text conversion disabled. No shell,
reference/index/config mutation or application runtime is invoked by verification.

The manifest supplies `schema:1`, base/candidate `{head_ref,commit}`, sources with
unit/head/base/ownership and exact selected file/blob lists, and generated common
files with base blobs and ordered `{unit,module,path}` exports. Every source delta
must be selected except a common file proven to equal its base plus only that
unit's declared exports. Candidate deltas must equal the complete selection plus
all declared generated files. Sources cannot silently omit changed fixtures.

The append recipe is base bytes ending in newline followed, per export, by
`\n#[path = "../b25a.rs"]\npub mod b25a;\n` (respective module/path for each row).
The verifier accepts only regular Git modes100644/100755 and checks the owner
execute bit on Unix. Symlinks, escapes, stale refs, altered candidate bytes,
unknown fields/exports and undeclared overlap refuse. Manifest/blob/raw reads,
selection counts and Git output are bounded. The coordinator supplies a process
timeout and the reviewed installed Git executable.

Run `cargo run --locked --offline --manifest-path rust/tools/verify-composition/Cargo.toml
-- <candidate-worktree> <manifest.json>` as one shell line. Tests require a fresh,
owned absolute `SYSON_COMPOSITION_TEST_ROOT` on the workspace; they retain unique
synthetic Git repositories and disable user hooks/config/attributes/signing.

This checks selected composition and committed delta identities. Untouched
inherited working-tree files and untracked files outside the selection are not
scanned. The coordinator separately verifies the full compiler-input closure
against candidate blobs before the composed build and checks it again afterward.
Remote PR state, complete C03 landing/retargeting, application/UI/native parity
and production release acceptance remain separate gates.

The coordinator approved482production/784authored Rust lines for this cohesive
read-only devtool; splitting the Git safety boundary would create a separate API
without improving review. No production application code changed.
