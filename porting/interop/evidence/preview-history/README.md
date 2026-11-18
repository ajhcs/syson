# Undo and redo a layout preview before saving

On the VehicleAssembly fixture, Chromium exercised mouse, keyboard, and
emulated touch controls for moving and resizing Drivetrain. A completed
drag produced one undo step. Undo returned to the original geometry
and Redo restored the proposed geometry. A new edit after Undo cleared
Redo. A no-op gesture and a cancelled touch preserved valid history.

Stage, Undo, and Redo were disabled while a drag was active. Requesting
form submission directly during the drag sent no POST. Cancel and
reload reset the preview. In a separate browser run, both controls
retained 32 undo steps after more than 32 effective edits.

After staging and saving, the move and restore downloads matched the
exact ZIPs previously reimported in SysON v2026.9.3. The
[move evidence](../archive-layout) and [restore evidence](../archive-restore)
contain those native comparisons. This packet covers the Rust archive
prototype on one diagram with 431 model objects and nine rendered nodes.
Production editor integration, multi-object history, and physical
touch-device verification remain open.

The measured stage requests took 66–82 ms across three local runs
with this fixture. The range describes those observations. It
does not establish a production performance budget.

Run the offline receipt and archive comparison from the repository root.

```sh
python3 -I -B porting/interop/evidence/preview-history/verify.py
```
