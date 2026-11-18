# Eclipse Filesystem POM capture

The getter captured the 25 Filesystem POMs in #388, in the order Maven offered them in #384. All 25 POM requests returned HTTP 200 at their requested Maven Central URLs. The getter retained 108,147 response body bytes across all 125 requests.

Each POM matched its same-origin SHA-1 sidecar. The 25 SHA-512 and 25 SHA-256 requests returned HTTP 404. All 25 detached signature requests returned HTTP 200. The signature bytes remain in private custody and have yet to be verified.

The retained POMs declare the expected Maven coordinates. Version 1.9.500 spells `modelVersion` as `4.0`. The other 24 use `4.0.0`. The observation records those raw declarations and keeps the original POMs in private custody. ModelBuilder and descriptor-reader behavior for these POMs still require an offline replay.

All 66 frozen input hashes agreed before and after capture. The controller exited successfully and confirmed teardown of the owned scope. Raw response bodies, headers, and host-specific records remain private.

Run both offline checks from this evidence directory.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

The verifier binds the public response data to the fixed client, request roster, and offer order. It checks the recorded checksum and signature dispositions and rejects alterations to the roster, response data, and declared authority. Independent custody review rechecks the physical XML and checksum bytes. Graph effects, final selection, binary trust, installation, and native compilation remain open.
