# Fixed normal-core acquisition epochs

The client recognizes models-01 (35 primaries/175 requests), models-02
(18 POMs/90 requests), and the models-03 candidate (five POMs/25 requests).
The latter includes four Apache parents and a conditional JUnit 5.14.1
observation. JUnit 5.12.1 remains deferred pending the Apache 34 parent model
and JaCoCo context. The draft authorizes no live acquisition.
The registry binds raw manifest bytes, independent ordered-primary seal, roles and limits.
Invocation retains the existing fixed launcher's five arguments: spec, SHA, lock FD, unit, cookie.
The frozen spec must name an admitted epoch and its exact registry-bound sibling manifest.
Its input map includes this client, manifest, immutable metadata/transport helpers and owner.
Whole pinned helpers supply transport, owned proof, checksum grammar and receipt accounting.
Bodies remain opaque raw bytes; every served SHA512/SHA256/SHA1 is checked, including metadata.
GNU filenames are untrusted metadata; signatures are retained without authentication claims.
The fixed logical profile is 16 MiB bodies, 63 MiB getter metadata and 1 MiB root logs.
Per-response headers and framing, aggregate static framing and a failure reserve remain bounded.
These logical write caps establish neither a physical quota nor release-capacity acceptance.
Offline unit tests require SYSON_BOOTSTRAP_TEST_ROOT on Samsung locally or RUNNER_TEMP in CI.
Tests use new-client scoped fake I/O; immutable old policies, manifests and globals stay unchanged.
No URL override, crawler, range selection, cache installation or automatic epoch extension exists.
Root separately admits every later model/binary epoch, code trust and actual runtime scope.
This prerequisite remains open through complete closure and offline unmodified 378-source compile.
Full native, library, SQL, UI, storage and fresh process/backend acceptance remains required.

From this directory, set `SYSON_BOOTSTRAP_TEST_ROOT` to an existing private
directory under `/workspace/build/builderbob` locally or `RUNNER_TEMP` in CI.
Both commands exercise the client with fake responses:

```sh
python3 -IB -m unittest discover -s . -p test_acquire.py -v
python3 -OIB -m unittest discover -s . -p test_acquire.py -v
```
