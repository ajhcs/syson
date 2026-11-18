# Archive recovery by touch and keyboard

Chromium replayed the archive recovery cases on an updated Rust prototype.
The browser recorded 35 trusted touch activations at 320 and 390 CSS pixels
with device pixel ratio 3, plus 12 desktop keyboard actions. Every tested
control was at least 44 × 44 CSS pixels. The previous build failed the same
check at a 21-pixel-high name field.

The touch runs covered renaming, saving, reopening after a stale-tab
conflict, retrying a failed save, confirming a pending save, and downloading
the archive. Desktop controls showed a visible outline on Tab focus.
Recovery errors exposed alert semantics. A pending confirmation page
reflowed from 320 × 568 to 568 × 320 and back at the same URL without
horizontal overflow. All four runs preserved the expected archive bytes.

The touch runs observed 38–52 ms across 15 openings and 68–162 ms across
21 measured interactions. The desktop run observed 39–48 ms across six
openings and 29–89 ms across seven measured interactions. The interaction
timer covers target interaction and the response or navigation checkpoint.
These concurrent loopback samples describe the fixture and test host.

Run the offline replay from the repository root.

```sh
python3 -I -B porting/interop/evidence/archive-touch/verify.py
```

The replay checks the retained input events, target geometry, responsive
layout, and recovery responses against the saved archive evidence. The
browser runs used emulated touch and automated text entry. Physical
devices, mobile keyboard behavior, and screen-reader announcements
require separate checks.
