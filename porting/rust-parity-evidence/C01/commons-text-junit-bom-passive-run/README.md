# Commons Text JUnit BOM capture

The offline Maven trace for the SysON metamodel followed Commons Text 1.10.0 to a missing import of the JUnit BOM 5.9.1. This capture fetched the BOM and its four sidecars from the five exact Maven Central URLs in the [request roster](../../../rust-parity-plan-20261026/closure/core-compile/requests/commons-text-junit-bom-01.json).

All five requests returned HTTP 200 at their requested URLs and retained 6,715 body bytes. The SHA-512, SHA-256, and SHA-1 sidecars each matched the retained POM. These matches establish consistency with the responses from the same origin. The detached signature was retained for separate verification. Publisher authentication, effective Maven model replay, cache installation, dependency execution, and the offline native build remain open.

The observation records status, byte count, and SHA-256 in roster order. Custody hashes identify the sealed scope and the private receipts. The controller exited successfully, the 23 frozen input hashes agreed before and after capture, and the controller confirmed teardown of the owned scope.

Run both offline checks from this directory.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

The verifier checks the committed capture metadata against the exact client and request roster. Recomputing the same-origin checksum matches requires the retained POM and sidecars from private custody.
