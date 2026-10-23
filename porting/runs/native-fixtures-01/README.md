# Exact native fixture replay revisions

These run records preserve the actual worker harness revision `e3d5b050fe5b49998982abc974f52be738e3c0a4` and product revision `6830df280ecf81542b51c76832d367b60e97ad4e` as recorded in each JSON report (the reports are authoritative for the full implementation SHA). Integration cherry-picks have different commit IDs even when the inspected source bytes agree. Do not silently replace an executed revision with a cherry-pick alias.

The exact reviewed harness is preserved on the fork branch `evidence/native-fixtures-01`. Fetch it explicitly alongside `rust/integration` before replaying or checking Git-blob hashes in a shallow/single-branch clone. The product revision is an ancestor of the integration branch. The branch contains public upstream source and portable validation artifacts only; it is an evidence ref, not a runnable product release.

```
git fetch origin evidence/native-fixtures-01 rust/integration
```

Use the source and harness revision strings from the JSON reports with `run_ledger_fixtures.py` and a fresh output directory on the approved build volume. All9native fixture cases passed. Upstream runtime records remain empty and all3behavior statuses remain unrun against upstream; no full parity claim is made.
