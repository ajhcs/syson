# Browser-edited archive reopened in SysON v2026.9.3

A disposable Rust service opened the authorized synthetic archive,
renamed the selected definition from Tire to Rim, and published it at
a new revision URL. Fresh desktop and mobile Chromium sessions reopened
both revisions after a Rust process restart. The original archive stayed
byte-identical. The saved archive retained the original manifest and
diagram bytes and changed only the selected model name.

The native run uploaded the exact Rust browser download. SysON
re-exported it with new archive identities. The comparator resolved 17
generated IDs and matched the structure, typing, and diagram layout.
After a page reload, 1440 × 960 Chromium rendered Rim, wheel, and one
package node in the SysON project. The retained browser observation
records both visits.

Run `python3 -I -B porting/interop/evidence/archive-edited-reopen-v2026.9.3/verify.py`
from the repository root to verify the three retained ZIPs and replay
the structural comparison. The original archive proof still covers the
default Wheel fixture. This packet covers one rename and archive exchange.

A follow-up Chromium run confirmed the rename and then stopped the Rust
service before final publication. A fresh process reopened the unpublished
draft at the same revision URL. The next browser submitted only Save, and
the published ZIP matched the one retained here byte for byte. The
[restart observation](rust-draft-restart.json) binds that result to this
packet. The native upload observation remains the original run.
