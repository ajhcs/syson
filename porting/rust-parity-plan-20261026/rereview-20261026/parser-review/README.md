# Parser/interchange/migration re-review

Five concrete P2 findings are recorded in [findings.json](findings.json). Two have
new coordinator-executed counterexamples; three are high-confidence source/control
flow proofs. No implementation fix, compiler, native/runtime, browser or network
execution was performed by this reviewer. Root's bounded new probes are separately
attributed; existing passed suites were not replayed.

| Finding | PR/head | Result | Existing plan target |
|---|---|---|---|
| PARSER-RR-01 | #55 92aaccaf (also #63/#69) | 500481 raw bytes cause a 16777216-byte single allocation before the 1MiB canonical-size refusal; additional requested live-byte peak 21011477 | I09, I03/I06 |
| PARSER-RR-02 | #56 82af0cea | A 2MiB name causes a 4194320-byte single allocation before the 65536-byte export refusal; additional requested live-byte peak 4194672 | P170, P171/I09 |
| PARSER-RR-03 | #56 82af0cea | `package 'class';` imports, but export prints unquoted `class` and its own real importer gate refuses it | P170/P171/P15 |
| PARSER-RR-04 | #11 511b0b1c, #41 25acfaa6 | Whole-state snapshots cause quadratic copied work for growing small transactions; snapshot work is charged as one unit | P12/P14 |
| PARSER-RR-05 | #12 ca80c196 | A FIFO with the admitted suffix/size reaches blocking open before the byte-limited read | P08/C09 |

Allocation metrics count requested layouts during the API call, not whole-process
RSS, native speed or a capacity SLA. The first root counterexample script's overall
FAIL came after the two confirmed allocation observations, at an unrelated model
probe compile error; no model runtime ran in that failed step. Its corrected second
step confirms the separately owned opaque-JSON private-token reification finding.
The original failed receipt remains unchanged. [Probe proposals](probes/proposal.json)
list exact immutable dependency exports, fixed small inputs and measurement limits.

[Coverage](coverage.json) inventories 28 assigned immutable PRs and 128 current
file entries, including 33 test files. Coverage is source/invariant/caller focused;
test case inventories and targeted assertions do not mean every historical source
snapshot or every generated grammar/table tuple received exhaustive semantic
qualification. Base components and later branches overlap and are not separate
feature-completion counts. F47 coverage is limited to spans/source association;
model/rule semantics belong to the model lane.

The known F13 synthetic-root boundary fix (#58), F47 CR/LF fix (#59), and explicit
identity bridges (#63/#69) are recorded as existing repairs rather than new bugs.
The model lane owns the shared serde private-key data-loss finding; materialized
F33/bridge/migration/preparation paths are cross-referenced without a duplicate fix.
C03c RawValue-only transport retains tokens, while later preparation can still
encounter the shared Value materialization defect.

Future work augments existing I09/P170/P12/P08 and acceptance tasks. The findings
provide exact files, prerequisites, focused implementation/verification tasks,
review budgets and separate code/evidence commits. Export budget and reserved-name
repairs share a production file, so the naming PR follows the budget PR. Existing
I06/C03/I07/P14 units receive concrete composed-caller and native qualification
checks; no new generic instrumentation or independent semantic engine is proposed.
All feature/native/application acceptance remains open until its actual gates pass.
