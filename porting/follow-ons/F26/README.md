# F26 validation packet

The source-only schema contract is in [schema-contract](../../persistence/schema-contract/README.md).
Its frozen candidate plan preserves retained XML declarations; artifact correspondence
and database execution remain blocked/unrun. Synthetic source-plan checks do not
qualify an applied schema, DBMS compatibility, rollback or the Liquibase checksum algorithm.

`run_checks.py f26-02` produced retained raw commands, stdout/stderr, timestamps
and five input hashes in `validation/f26-02/receipt.json`. Nineteen test functions
pass. Preliminary successful checks existed in the tool transcript; the captures
record fresh executions rather than invented metadata for those earlier calls.
The original f26-01 capture (17 passing tests) predates root-reviewed XML fidelity
fixes. Its streams, exact executed runner and all five old source inputs remain
under `validation/f26-01/`; it is not relabeled as the later source.
No failed test attempts occurred. The runner never invokes a migration tool or DBMS.

`VALIDATION.json` summarizes measured evidence and unresolved boundaries.
`REVIEW.md` records the separate source-review gate. No shared backlog, dependency,
credential, package, existing lease or production state was changed.
