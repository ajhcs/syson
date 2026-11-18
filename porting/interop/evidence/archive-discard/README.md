# Discard an unpublished layout draft after restart

Chromium staged a Drivetrain resize from 200 × 101 to 210 × 101.
After the Rust prototype restarted, the same URL reopened the unpublished
draft. Discard closed that revision slot and retained the draft, its
authorization records, and the saved archive. The page labeled 210 × 101
as the discarded preview, and the parent still showed 200 × 101.
Another revision in this prototype requires separate authorization.

A stale Save returned 409. Repeating Discard kept the same marker and
filesystem identity. Replacing the checkpoint with identical bytes at a
new inode invalidated the old form. The refusal left the draft available
and created neither a saved archive nor a discard marker.

An injected directory-sync failure kept discard visibly pending through
a second request. A fresh process synced the marker and directory, then
confirmed the discard. A pending Save that had already linked an archive
refused discard and remained pending after restart. In an HTTP race
between two cooperating processes, one Save or Discard could complete.
The losing operation was refused. A pre-link discard failure left Save
available.

Desktop discard took 65 ms. Two cold confirmed opens took 53–58 ms.
The emulated-touch response in the injected fault path took 92 ms.
These are local observations on one fixture. Production integration,
slot reuse, physical touch, power loss, and comparison with a native
SysON durable-discard workflow remain open.

Run the offline receipt checks from the repository root.

```sh
python3 -I -B porting/interop/evidence/archive-discard/verify.py
```
