# Archive diagram zoom and pan

The Rust archive prototype reopened the nine-node vehicle diagram after
a forced stop and exposed Fit diagram, Actual size, Zoom in, and Zoom out.
The viewport panned without altering the stored diagram. Desktop Chromium
tabbed into the viewport, moved right with ArrowRight, and reached the
far-right Body package with the wheel. At 320 CSS pixels, trusted emulated
touch swipes reached the same package. Its label measured 3 CSS pixels at
Fit, 16 at Actual size, and 33 at 200 percent.

The browser restored the fitted view after zooming and after resizing to
568 × 300. A separate run with classic scrollbars at 1040 × 190 reproduced
two earlier Fit defects. The wide diagram left a 15-pixel unused strip,
and the two-node diagram kept 80 pixels of vertical overflow. The revised
Fit used the stable outer width and available height. SVG node and label
identities remained unchanged.

The four desktop and mobile downloads, taken before and after viewport
interaction, matched the exact ZIP previously round-tripped in live
SysON v2026.9.3. The archived native proof in the preceding packet applies
to these same bytes.

The measured button actions took 3–20 ms on desktop and 42–62 ms at the
emulated mobile viewport. These are one-host interaction samples. They
exclude the time spent panning and do not establish application capacity.
This fixture covers browser scrolling and buttons. Physical-device use,
pinch zoom, and full diagram-editor interactions remain open.

Run the offline check from the repository root.

```sh
python3 -I -B porting/interop/evidence/archive-zoom/verify.py
```
