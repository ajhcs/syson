# Save and reopen a corrected resize in the browser

On the nine-node VehicleAssembly fixture, an authorized user staged a
new 206 × 101 Drivetrain resize after discarding the earlier 210 × 101
draft. A fresh Rust process reopened the same correction URL. Save
published a new revision. After another forced restart, the browser
downloaded the same archive. The saved parent stayed at 200 × 101
and the old draft stayed discarded.

Desktop Chromium exercised the width chooser with real keyboard input.
Emulated touch exercised the 44 px chooser and trusted actions for
Stage, Save, Retry, Confirm, and download. Playwright selected the width
on touch because the native popup is outside the harness. The chooser
showed a 3 px keyboard focus outline. Both draft and saved pages
displayed the corrected size as text.

A failure before file publication left the draft available for Retry.
After a failure following publication, the pending page withheld the
archive until a fresh process confirmed it without relinking. A stale
Save was refused, and a repeat of the exact Stage retained the
checkpoint. Five injected failures passed the separate HTTP checks.

Stage took 121–186 ms, cold reopen 92–103 ms, and download 85–90 ms
across the two final local runs. Successful Save and Confirm took
166 ms and 191 ms, respectively. These observations cover one fixture.

The downloads are byte-identical to the separate ordinary 206 × 101
resize. The ZIP changes only the selected node's width and user-resized
flag compared with the saved parent. Production integration, physical
touch, power loss, and a fresh native SysON import remain open.

Run the offline archive and receipt comparison from the repository root.

```sh
python3 -I -B porting/interop/evidence/archive-corrected-publication/verify.py
```
