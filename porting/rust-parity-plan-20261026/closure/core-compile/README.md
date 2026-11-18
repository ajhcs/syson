# Fixed normal-core acquisition epochs

The first three epochs contain 35, 18, and five primary artifacts respectively.
The models-04 candidate adds one POM and four sidecar requests. The pinned
Maven ModelBuilder requested JUnit BOM 5.12.1 in both the baseline and
CI-property probes. The synthetic selector override remains outside the
allowlist. This draft prepares a fixed roster. Live acquisition requires
separate coordinator admission. [Selector evidence](https://github.com/ajhcs/syson/pull/349)
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

## Eclipse Filesystem child metadata proposal

The default Maven descriptor reader recorded 42 ranged edges from 14 of the 25
Filesystem releases. Each of those releases requested Equinox Common, Equinox
Registry, and Eclipse OSGi in `[3.2.0,4.0.0)`. The three metadata primaries
are absent from the earlier fixed rosters. An earlier capture holds the fixed
Eclipse OSGi 3.23.100 POM. [Descriptor evidence](../../../rust-parity-evidence/C01/eclipse-filesystem-default-reader/observation.json)

The proposal lists 15 fixed Maven Central requests. Each of the three
`maven-metadata.xml` bodies has a 256 KiB cap and its four sidecars have a
64 KiB cap each, for a total of 1.5 MiB across the roster. Live acquisition
requires separate coordinator admission. A later offline Maven replay can
measure the version offers before descriptor collection continues.

## JaCoCo binary capture proposal

The reported collector graph selects 13 exact artifacts. Ten had no matching
files in the 16 inspected trees, so this proposal adds 50 fixed requests for
those ten JARs. Each JAR has four fixed sidecar requests. The agent row
requires the `runtime` classifier. No provided dependency or version range
adds an artifact to the roster.

Reuse of the three observed coordinates needs separate data review. The plugin
and ASM have partial captures and retained passive receipts. The two SLF4J API
copies have identical bytes and matching local SHA-1 sidecars. All three
remain outside the new request roster and require separate trust admission
before installation or execution.

The isolated client uses the existing fixed Maven Central HTTPS transport
without redirects, proxy settings, archive extraction, installation or
execution. The aggregate body cap remains 16 MiB. Each JAR is capped at 1 MiB
and each sidecar at 64 KiB. Their request maxima total 12.5 MiB.
A missing or empty JAR, a malformed or mismatched checksum, or no matching
available checksum fails the run. Served checksum matches establish
same-origin byte consistency. Detached signatures require separate
authentication. A passive capture grants no code trust.

This draft creates no runnable scope. Live acquisition, review of downloaded
data for binary use, and the pinned native compile remain open.

Run both offline test modes from this directory with the same fixture output
environment described above. The tests feed synthetic responses and disallow
real sockets and subprocess launches.

```sh
python3 -IB -m unittest discover -s . -p test_jacoco_binaries.py -v
python3 -OIB -m unittest discover -s . -p test_jacoco_binaries.py -v
```
