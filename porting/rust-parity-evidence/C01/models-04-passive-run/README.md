# Models-04 passive acquisition result

One scoped run used the exact request manifest in #350. The JUnit BOM 5.12.1 POM, three checksum sidecars and signature sidecar each returned HTTP 200 at their original URL. Independent review recomputed all three checksums against the POM bytes. The signature was retained privately without verifying a signer.

The primary POM is 5,649 bytes and declares no parent, profiles, direct dependencies or managed imports. It declares 18 managed dependencies, including `org.junit.jupiter:junit-jupiter-api` at `5.12.1`. The isolated getter retained 6,744 response body bytes. The reviewer checked all five saved bodies, 134 frozen input hashes, the actual resource controls and teardown of the owned scope.

The public projection records bounded response facts and the source model summary. Private response headers, cookies, invocation identities, local paths and the absolute input map are excluded.

## Verify the checked-in projection

Run both commands in this directory with Python 3.9 or newer.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

The verifier checks the projection against the exact parent manifest. The commands use the retained observations and make no network requests.

The baseline and CI selector observations are in #349. The capture supplies the BOM bytes for a separately reviewed offline model-builder run. Effective model construction, plugin mediation, the pinned native compile and Rust application acceptance remain open.
