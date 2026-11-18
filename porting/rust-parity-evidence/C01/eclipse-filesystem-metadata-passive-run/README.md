# Eclipse Filesystem metadata capture

The passive getter followed the five fixed requests in #380. Maven Central returned the Filesystem version metadata and all three checksum sidecars with HTTP 200. The detached signature URL returned HTTP 404. The SHA-512, SHA-256, and SHA-1 responses each matched the retained metadata bytes. Those matches establish consistency with the responses from the same origin. Publisher authentication remains open.

The metadata lists 26 version declarations in its original order. Its `latest` and `release` fields both say `1.11.500`. The public observation records what the XML declares. A later offline Resolver run must determine which offers belong to `[1.7.0,2.0.0)` and how the collection proceeds.

The getter retained 1,974 response body bytes, including the 404 response body. All 39 frozen input hashes agreed before and after the run. The controller exited successfully and confirmed teardown of the owned scope. Raw bodies, HTTP headers, and host-specific records remain in private custody.

Run both offline checks from this directory.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

The verifier checks the committed projection, exact request roster, and client identity. Rechecking the physical checksums and rederiving the XML declarations require the private retained bodies. This capture leaves Maven offers, descriptor collection, final version selection, binary trust, cache installation, and native compilation open.
