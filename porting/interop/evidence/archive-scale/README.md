# Archive recovery at two project sizes

The same disposable Rust build ran the archive workflow with a 12-object
fixture and a 423-object vehicle assembly. The larger fixture contains
80 typed part pairs and nine archived package nodes. Each fixture ran
two warmups and seven measured desktop Chromium repetitions in serial.
Each run saved a first rename, authorized a second, stopped before Save,
reopened the draft, and downloaded it after another restart. Both
renames preserved the exact manifest and diagram bytes.

| Browser operation | 12 objects, ms | 423 objects, ms |
| --- | ---: | ---: |
| Reopen draft after process stop | 21–28 | 32–35 |
| Save recovered draft | 86–104 | 100–113 |
| Reopen confirmed successor | 20–21 | 35–47 |
| Download and hash archive | 12–17 | 25–30 |

Peak sampled Rust process RSS was 3,864 and 10,660 KiB, respectively.
These local measurements exclude process and browser startup and cover
one viewport. Production capacity still needs separate qualification.

## Native SysON exchange

SysON v2026.9.3 uploaded the exact Rust download. The native model
preserved all 423 stable objects and 81 typing relationships after ID
mapping. The browser showed one of nine archived package nodes after
reload. Its re-export had no saved node or label positions.

The streaming style decoder reads the parent node type before processing
its style. An otherwise identical upload with type placed before style
preserved all nine node and label position records. It still rendered
only the root package. The source ViewUsage exposes only the root. Native
synchronized views require semantic exposure of child packages and nest
them under their visible parent. In a fresh native project, dragging
Drivetrain from the Explorer added one MembershipExpose to its owning
membership. The re-export nested Drivetrain under VehicleAssembly with
matching node and label layout, and the browser preserved it on reload.
Exchange of a native-compatible diagram containing all nine nodes remains open.

Run the offline checks from the repository root.

```sh
python3 -I -B porting/interop/evidence/archive-scale/verify.py
python3 -I -B porting/interop/evidence/archive-scale/check_native.py
```
