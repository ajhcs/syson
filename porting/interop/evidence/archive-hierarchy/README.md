# Nine-node archive recovery and SysON round trip

In SysON v2026.9.3, eight Explorer drags exposed eight vehicle subassemblies
through their owning memberships. SysON exported one root with eight
nested nodes, 431 model objects, and one node and label layout record for
every rendered node.

The Rust prototype opened that exact archive and changed DriveShaft V3
to V4, then to V5 in a second revision. Chromium staged V5 and stopped the
service before Save. A fresh process reopened the draft at the same route.
A second restart served the same final ZIP to desktop and mobile browsers.
The exact manifest and diagram bytes and all earlier ZIPs were preserved.

A fresh SysON project imported that final browser download. The native
browser reopened DriveShaft V5, its typed usage, and all nine nested
packages. The native re-export matched the full manifest, model, and
diagram after mapping 452 generated identities. The missing-manifest
control created no project.

The first Rust attempt rejected the 1,794-unit diagram width. The revised
prototype raised the coordinate bound for the explicitly admitted native
archive profile. The compact and flat fixture limits and the generic
inspection path were retained.

On a 390-pixel mobile viewport the canvas fit, but its label glyphs
were only four CSS pixels high. A touch-opened package outline showed
the same names and hierarchy at readable size. Desktop keyboard input
opened the same outline. One local run reopened the confirmed revision
in 34–39 ms across these two viewports. It provides no load or capacity
baseline. Native-style diagram zoom and pan still need qualification.

Run the offline replay from the repository root.

```sh
python3 -I -B porting/interop/evidence/archive-hierarchy/verify.py
```
