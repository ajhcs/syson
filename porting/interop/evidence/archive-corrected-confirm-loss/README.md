# Recover when Chromium misses Confirm

The Rust archive prototype linked a corrected ZIP, then an injected directory-sync error left the revision pending. Save returned HTTP 503. The correction route withheld the download before and after the original Rust process was killed. A fresh process served the pending page, and Chromium submitted Confirm using the keyboard. Confirm added only the confirmation record. The existing 206 × 101 archive retained its device, inode, modification time, length, and bytes.

In the withheld case, Playwright read one actual same-route HTTP 303 from Rust after confirmation existed. It then aborted that response before Chromium received an HTTP status. Chromium recorded a connection-closed error. A separate run delivered Confirm's response normally and showed the saved page.

After killing the Confirm process, a fresh Rust process and Chromium instance downloaded the exact [corrected ZIP](../archive-corrected-publication/rust-corrected.zip). Both runs preserved the saved parent and discarded resize. Repeating Confirm from an actual same-origin form returned HTTP 303 and kept both artifacts unchanged. Repeating Save returned HTTP 409, offered the saved revision, and also kept both artifacts unchanged. A Confirm request without Origin returned 403. Both private user scopes were removed.

From the repository root, verify the sanitized observation against the committed Save-loss evidence and archive.

```sh
python3 -IB porting/interop/evidence/archive-corrected-confirm-loss/verify.py
python3 -OIB porting/interop/evidence/archive-corrected-confirm-loss/verify.py
```

The verifier checks the recorded result and archive contents. It does not launch Chromium or inject a new fault. The response was withheld at the browser interception boundary after the upstream Confirm completed. This qualifies one filesystem prototype transition. Production SQL acknowledgment, application restart, and power-loss durability remain open.
