# F02: source-pinned namespace import contract

This PR freezes the native metamodel import/exclusion/conflict API and implements
a stdlib-only contract checker with 19 ordered synthetic fixture cases and
17 checker tests. The dependency map gives F02 no future prerequisites. F03's
import evaluator is a separate change.

The new contract is in `porting/area-inventories/DOMAIN-QUERY/imports/`; the
canonical backlog, held leases, Rust source, compiler allocation and package
baseline are unchanged. The lease covers only that new directory and this F02
handoff directory. The source pin and all unsupported outcomes are explicit.

`validation/f02-01/` preserves the first captured pass: six source pins,
18 initial fixture cases and 17 checker tests. The independently requested
positive recursive fixture adds a nineteenth case; its final run is in
`validation/f02-02/`. Initial exploratory CLI output passed before capture;
no failed check has been discarded or relabeled. No upstream runtime ran.

Run `python3 -B porting/follow-ons/F02/run_checks.py <fresh-run-id>` to retain
new input hashes and original stdout/stderr. Existing run directories are never
overwritten. Final receipt source hashes must match the committed contract files;
the capture runner's executed bytes are recorded in the review note.
