# Linker declaration-budget audit fix

The F11 parser admits 256 real declarations by default. F13 also indexed the
anonymous synthetic Namespace root against its 256-scope budget, so its final
real declaration failed with `Limit` after parsing had succeeded.

The linker now reserves an extra scope only for an anonymous Namespace root.
A custom Package root counts as a real declaration. Four regressions exercise
actual F11 parsing and reference resolution at 255/256 declarations, keep the
257-declaration refusal with default or expanded parser budgets, and check
256/257 declarations when a custom real Package is the root.

The coordinator captured the original defect: `link-red-01` passed 17 tests and
failed the new 256-declaration regression. After the repair, `link-green-01`
passed all 19 integration tests, formatting and scoped all-target Clippy, offline
and locked with one compiler job. All captured source inputs were unchanged and
match the committed green source. The independent coordinator source review found
no blocker within this finite admission scope.

[VALIDATION.json](VALIDATION.json) is a compact summary, not a replacement for the
canonical capture receipts. Exact original red/green source, runners, receipts
and raw streams remain at the external Samsung paths named there. No original
failure is relabeled and the historical F13 evidence packet is unchanged.

This aligns local Rust limits; it does not qualify native parser limits or all
SysON behavior. The fix follows F13 #50. Land that prerequisite, retarget this PR
to the merged integration base, inspect and revalidate; never merge into the
prerequisite feature branch.
