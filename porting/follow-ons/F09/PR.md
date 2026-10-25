The Rust parser foundation had reflection tables but no ordered postprocessor
dispatch planner. This adds three distinct source-derived operations: first
truthy most-derived selection, ordered array-block merging and full-chain
collection/reversal. The parser's five pinned registrations are represented by
typed processor identifiers; no callbacks execute.

The immutable registry rejects foreign reflection handles and exposes explicit
falsey/default behavior. Empty arrays remain truthy, duplicate payloads survive,
and reverse block order preserves each block's internal order. Duplicate keys
and dynamic JavaScript object inputs are explicitly restricted. F08's ancestor
ordering remains a source-derived candidate, without JavaScript runtime parity.

Validation: fifteen tests, formatting and all-target Clippy passed offline and
locked. Three retained source documents and reviewed ranges pass hash checks;
exact commands, source hashes, original streams, notices and root review are
retained under F09. AST construction, callback application and cache side effects
remain unimplemented.

Stacked on F08 PR #23 (`codex/syson-f08` at
`7d1eb8a1a05c292c78ca27482158c6dfe5196c3b`). Merge the prerequisite first,
then retarget this child after inspecting its diff; do not merge into the parent
feature branch. No shared manifest, parent-source file, backlog or hold changes.
