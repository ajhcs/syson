# Stage and reopen a corrected width after discard

The fixture CLI staged Drivetrain at 206 × 101 in a new correction
checkpoint. A fresh process reopened it at the same dimensions with the
same checkpoint bytes and inode. The earlier 210 × 101 draft stayed
discarded and the previously saved parent stayed at 200 × 101.

A separate ordinary resize produced the exact ZIP hash computed in
memory by correction staging. The independent comparison changed only
the permitted width and its user-resized flag. The correction slot
wrote a checkpoint without publishing an archive.

A different width in the same slot, an incomplete approval pair, and
approval record replacement after staging were refused. File-sync and
directory-sync faults withheld success until the expected recovery path
completed. The original correction checkpoint was never overwritten by
these operations. The run covers the CLI on the nine-node fixture in a
private Linux directory. Browser staging, Save, and native comparison
of a published corrected revision remain open.

Run the offline receipt checks from the repository root.

```sh
python3 -I -B porting/interop/evidence/archive-corrected-draft/verify.py
```
