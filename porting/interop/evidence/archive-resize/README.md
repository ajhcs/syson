# Package resize and native SysON exchange

The Rust archive prototype staged the same Drivetrain resize with a
desktop mouse and a trusted emulated touch gesture at 390 CSS pixels.
Each run first adjusted width and height in preview, then cancelled.
Cancel restored the original rectangle and sent no write. The parent
page labels the edit as a layout preview while it is active.

Both runs changed the saved width from 200 to 210 and retained height
101 and position (9, 46). The service stopped after staging and before
Save. The draft reopened at the same URL, and a later restart served an
identical published ZIP to desktop and mobile. The archive changed only
the selected width and resizedByUser flag. The model, manifest, and
other diagram records stayed unchanged.

A fresh SysON v2026.9.3 project reopened the exact browser download at
210 × 101. Its re-export matched the complete manifest, model, and
diagram after mapping 452 generated identities. A separate native handle
drag reached 213 × 101 and wrote four selection-handle layout records.
The re-import of the Rust output had all nine node and label records
with no selection-handle records.

The two local runs staged in 71–89 ms, reopened the draft in 40–57 ms,
and saved in 89–110 ms. These are diagnostic observations for one archive.
Physical touchscreen behavior, native height-resize interaction, and
general multi-node editing and undo remain open.

Run the offline replay from the repository root.

```sh
python3 -I -B porting/interop/evidence/archive-resize/verify.py
```
