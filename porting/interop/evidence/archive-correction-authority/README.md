# Authorize a correction after discarding a layout draft

After discarding a proposed Drivetrain resize from 200 × 101 to 210 × 101,
a local CLI issued a ticket and grant for one width-only correction.
The authorization ties the new slot to the saved parent archive, exact
discarded draft and marker, selected node, and original revision
directory. The allowed widths are 188–212 excluding 200 and 210.
The old draft URL stayed closed and the saved parent stayed intact.

The issuer refused an unconfirmed discard while directory sync
continued to fail. A fresh issuer synced the same marker before creating
authorization. It recovered each tested partial ticket or grant
publication without replacing existing files. It also refused a linked
Save, an inconsistent Save confirmation, and stale or foreign inputs.

The evidence covers the local fixture CLI and cooperative processes
using one private Linux directory on ext4. Starting, editing, or saving
the corrected draft through the browser remains open. This run does
not establish physical power-loss behavior or a native SysON equivalent.

Run the offline receipt checks from the repository root.

```sh
python3 -I -B porting/interop/evidence/archive-correction-authority/verify.py
```
