# Diagram move recovery and SysON exchange

The Rust archive prototype first saved a rename to DriveShaft V4.
A separately authorized successor allowed one move of the existing
Drivetrain package. A keyboard preview stayed local to the browser.
Cancel restored the original position and sent no write. A mouse drag
moved Drivetrain from (9, 46) to (19, 50) in the parent coordinate frame.
Stage layout draft sent exactly one move request.

The service stopped before Save. The draft reopened at the same URL,
and another restart served the same published ZIP to desktop and
mobile. The final archive changed only the selected position and its
movedByUser flag. The model, manifest, and other layout records stayed
unchanged.

A fresh SysON v2026.9.3 project imported the exact moved ZIP and reopened
Drivetrain at its saved position. Its re-export matched the full manifest,
model, and diagram after mapping 452 generated IDs. A separate native
drag preserved the same core movement fields and also wrote four
selection-handle records. The Rust archive and its fresh, unselected
SysON re-export both retained nine node and nine label layout records.

One local run staged the move in 69 ms, opened the recovered draft in
34 ms, and saved in 103 ms. These are diagnostic observations.
The fixture covers one authorized package move. Touch editing,
multi-node moves, and diagram history and undo remain open. The parent
page still displays Saved revision alongside the active layout preview.

Run the offline replay from the repository root.

```sh
python3 -I -B porting/interop/evidence/archive-layout/verify.py
```
