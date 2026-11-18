# Restore a saved package size as a new revision

The Rust prototype saved a Drivetrain resize to 210 × 101. A separate
authorization allowed restoring the previous size. Opening the restore
disclosure showed both sizes and sent no write. Stage restore draft
submitted only the restore operation. The draft survived a forced stop
before Save.

Desktop and mobile Chromium downloaded the original, previous-size,
resized, and restored archives after restart. Both earlier saved
versions kept their bytes and filesystem identities. The restore
published a new archive at a new URL with all three unpacked members
byte-identical to the earlier archive. The page links to both the
resized parent and the previous-size revision.

A fresh SysON v2026.9.3 project imported the exact restored ZIP and
rendered Drivetrain at (9, 46) with size 200 × 101 before and after
reload. The re-export matched the complete manifest, model, and diagram
after mapping 452 generated identities.

A separate native history run resized width to 213. Ctrl+Z restored 200
and Ctrl+Y reapplied 213 in the open page. The saved geometry persisted
on reload. Neither shortcut changed the diagram or sent a history
request after reload in that run. This Rust prototype covers an
immutable restore. General Undo/Redo, multi-item history, and production
history integration remain open.

One local browser run staged the restore in 73 ms, reopened the draft
in 40 ms, and saved in 100 ms. These are measurements for one fixture.

Run the offline replay from the repository root.

```sh
python3 -I -B porting/interop/evidence/archive-restore/verify.py
```
