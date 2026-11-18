# JaCoCo binary capture

The scoped getter used the exact roster in #358. All ten JARs and all ten signature sidecars returned HTTP 200 at their requested URLs. Fourteen checksum responses matched the JAR bytes and covered every JAR. The remaining 16 checksum requests returned HTTP 404.

The run retained 1,660,526 response body bytes within the existing 16 MiB body cap. Independent review checked the saved responses, 161 frozen inputs, resource controls and teardown of the owned scope. The projection excludes raw bodies, signed bytes, headers, cookies, invocation identities, host paths and the absolute input map.

## Verify the checked-in projection

From this directory, run both Python modes with Python 3.9 or newer.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

The verifier binds the exact committed client and request roster and checks the retained observation offline.

The checksum results establish consistency with the same-origin responses. The capture retains unverified signature bytes privately. Signer verification, validation of the three pre-existing artifact candidates, cache installation, code trust and the pinned native build remain open.
