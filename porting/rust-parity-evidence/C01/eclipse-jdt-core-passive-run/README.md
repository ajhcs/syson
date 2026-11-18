# Eclipse JDT Core descriptor capture

The bounded getter used the exact JDT Core roster in #365, derived from the 215 observed POM paths in the offline replay in #364. All 22 JDT Core POMs at versions 3.26.0 through 3.47.0 returned HTTP 200 at their requested Maven Central URLs. The other 193 POM paths remain outside this capture.

Each POM had a matching same-origin SHA-1 response. All 44 SHA-256 and SHA-512 requests returned HTTP 404, and all 22 signature sidecars returned HTTP 200. Signature verification and publisher authentication remain open.

The raw XML for JDT Core 3.31.0 spells `modelVersion` as `4.0`. The other 21 POMs spell it `4.0.0`. The recorded hashes bind the original bytes and the observation preserves the two spellings. Effective Maven model acceptance, final version selection, cache installation, execution and the offline native build remain open.

The getter retained 101,783 response body bytes within the 16 MiB aggregate cap. Independent review checked the retained responses, the 177 frozen input hashes, the observed CPU, memory and task controls, and teardown of the owned scope. The public observation records hashes and status in exact request order. Original POMs and sidecars, response headers and host-specific custody details stay in private custody.

## Verify the observation

From this directory, run both Python modes.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

The offline verifier binds the parent client, request roster and checked-in observation. Each response row records HTTP status, body size in bytes and the SHA-256 of the retained body. Custody hashes identify the private records independently checked for this capture.
