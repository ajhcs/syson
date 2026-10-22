# Behavior-ledger validation contract

`porting/behavior-ledger.json` starts with three bounded behaviors and nine planned, hashed fixtures. All three statuses are `unrun`; zero behaviors are verified. The existing `PARITY-LEDGER.json` remains a separate incomplete milestone overlay. This file does not enumerate the whole application or close inherited dependencies.

IDs `SYSON-BEH-NNNNNN` and `SYSON-CASE-NNNNNN` are permanent. Keep an ID when evidence or implementation evolves; never reuse deleted IDs for different behavior. Each behavior declares its owner, dependency IDs, pinned source locations/hashes, expected state/result, implementation revision/files/authors, positive/negative/failure cases, hashed fixtures, upstream/Rust run records and independent review disposition. Empty run arrays and null review mean missing evidence. A planned fixture is not an executed test.

Use Python 3.9 or newer and Git; no third-party Python modules or schema download are required. The checked-in schema uses a small explicit JSON Schema vocabulary which the gate implements directly. No JavaScript, Java, network request, artifact command, upstream script or product runtime is executed by the gate.

```
python3 rust/validation/scripts/check_behavior_ledger.py
python3 rust/validation/scripts/check_behavior_ledger.py --require-complete
SYSON_BUILD_DIR="${SYSON_BUILD_DIR}" PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s rust/validation/tests -p 'test_behavior_ledger.py' -v
python3 rust/validation/scripts/check_metamodel_inventory.py
```

Set `SYSON_BUILD_DIR` to an existing approved build directory for temporary synthetic test repositories. The completion command is expected to **fail** until inventory closure, dependency closure, closure evidence and every required verified behavior are recorded. Pass `--root CHECKOUT` for another checkout and `--dependency-root DEPENDENCY_ID=CHECKOUT` for an independently pinned external source checkout. No dependency is fetched. Source/implementation hashes are checked against Git blobs at the specified full revisions; fixture and artifact paths must be safe relative paths and may not resolve outside the checkout.

A `verified` row additionally needs all three case groups, hashed fixtures for each case, and passing executed upstream and Rust records covering every case. Run report JSON must conform to the schema's `run_report` definition, identify the exact behavior/revision/side, list command arguments and valid UTC start/finish timestamps, record exit/result, case IDs and fixture hashes, and reference a nonempty hashed transcript. The gate reads these files but never executes the recorded command. Report fixtures and transcripts must be collected from the real runs before a product row can claim verification; synthetic unit-test reports exercise the format only and are never product evidence.

Independent review requires an accepted disposition, an identity outside the declared implementation authors and a hashed JSON report conforming to `review_report`. Its revision, identity, behavior ID, independence and case coverage must agree with the row. Revisions changing later require updated run/review evidence; historical passing tests do not transfer automatically.

These checks establish internal consistency, pinned-byte integrity and recorded coverage. They cannot authenticate a fabricated transcript, prove that a declared reviewer is a separate person, infer an omitted feature, establish semantic completeness from a closure flag, or independently prove real execution. Reviewers must assess that provenance and source/dependency closure. The gate is an auditable rejection condition, not independent certification of the supplied records.

The separate metamodel check recursively compares every classifier's complete parsed XML subtree with the pinned source, including node order, attributes, text, tail and source location. It protects nested `eParameters`, `eAnnotations` and `eLiterals` against inventory truncation. Passing it does not establish typechecking, parser semantics, inherited operations or product behavior parity.
