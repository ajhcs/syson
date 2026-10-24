The local snapshot adapter previously had no deterministic way to exercise write, sync and publication failures. This change adds a trusted `SnapshotIo` seam while keeping `save_new` on the same shared implementation and standard-library operations. Existing errors and no-replace publication remain compatible.

Thirteen new real-filesystem tests preserve an unrelated original snapshot and inspect staged/final bytes through fresh stores. They cover partial writes, each operation boundary, existing-file collisions and errors after publication/removal. In particular, an `Io` error can accompany a readable published file; an after-removal error can leave no staging, and retry still cannot replace that file.

Validation: coordinator-run full default workflow formatting, 15 tests (13 new, 2 existing) and all-target Clippy with warnings denied passed offline/locked with one compiler/test thread. Captured source remained unchanged. Original commands, streams, runner hashes, pinned baseline and retained-fixture inventory are included in `porting/follow-ons/F31`.

This is local snapshot failure qualification, not PostgreSQL recovery or crash/power-loss/upstream parity. Candidate factory feature and non-Unix execution remain unrun. No dependencies, manifests, services, automatic cleanup or recovery policy change.
