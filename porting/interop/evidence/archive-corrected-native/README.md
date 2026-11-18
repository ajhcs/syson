# Reimport the corrected Rust archive into Eclipse SysON

SysON v2026.9.3 imported the exact Rust browser download from the
[corrected resize](../archive-corrected-publication). Its initial export
and its export after the first browser open both match the Rust
manifest, model, and diagram after mapping 452 generated identities.
Both native exports use the same native identities. They preserve 431
model objects and nine diagram nodes, including Drivetrain at (9, 46)
with a size of 206 × 101 and the user-resized flag set. A separate ZIP
without a manifest was refused and created zero projects.

The first native browser opened the diagram and downloaded its archive.
At the tested viewport, the diagram extended beyond the right pane.
Host memory pressure stopped the run as the second browser context
opened. Fresh-context reopen and native backend restart remain open.
The owned test scope and its listeners shut down cleanly.

Run the offline archive comparison from the repository root.

```sh
python3 -I -B porting/interop/evidence/archive-corrected-native/verify.py
```
