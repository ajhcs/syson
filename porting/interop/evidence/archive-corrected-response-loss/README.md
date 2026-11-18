# Recover a saved resize when Chromium misses the response

The Rust archive prototype received one actual Save POST for the nine-node
resize fixture. The Playwright interceptor read the same-route HTTP 303 after
the ZIP and confirmation existed. It then aborted the browser request without
delivering HTTP headers or a status. Chromium recorded one connection-closed
failure. A separate control delivered the successful response normally.

The original Rust process was killed. A fresh Rust process and Chromium
instance opened the saved route and downloaded the same bytes as the parent
evidence archive. A replay of the same browser form returned HTTP 409 and
offered the saved revision. A request missing the Origin header returned 403.
Neither rewrote the ZIP or confirmation. The saved parent and discarded
revision were preserved. Both isolated scopes were removed.

The first two harness attempts reached the withheld response and fresh
download. The first retry lacked Origin. The second used scripted fetch, and
the harness observed no retry POST before browser closure. The final replay used
a native HTML form from the saved page with the original request body.

```sh
python3 -IB porting/interop/evidence/archive-corrected-response-loss/verify.py
python3 -OIB porting/interop/evidence/archive-corrected-response-loss/verify.py
```

The offline commands check the recorded observation against the parent ZIP.
The runtime experiment withheld a completed response through Playwright. It
covers one confirmed filesystem publication. Production SQL commit and
acknowledgment, application-wide Rust restart, and power-loss behavior remain
open.
