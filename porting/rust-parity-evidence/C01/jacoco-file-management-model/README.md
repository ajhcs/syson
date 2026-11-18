# Build the isolated file-management model with JUnit BOM 5.12.1

Apache Maven ModelBuilder 3.9.16 built the effective
`org.apache.maven.shared:file-management:3.2.0` model under JDK 21.0.12.1+1
in both the baseline and CI-profile cases. Both cases loaded three pinned
parent POMs and the exact JUnit BOM 5.12.1 from the preceding passive capture.
The previously missing `junit-jupiter-api` version resolved to 5.12.1 in
dependency management and in the direct test dependency.

Withholding the BOM recreated the missing import and dependency version
errors. An independent property override requested the synthetic
`99.99.99-model-probe` version and failed closed with the same error classes.
Each case ran in a fresh JVM. The runner compiled the included Java probe
with annotation processing disabled, disabled Maven plugin processing,
provided an in-memory POM-only resolver, and verified kernel denial of
IPv4, IPv6, and Unix sockets before launching the JVMs. The retained output
contains no host paths or invocation identities.

From this directory, verify the exact recorded traces, probe source, and
parent acquisition observation using Python 3.9 or newer.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

The verifier reads the checked-in evidence. It does not run Java or retrieve
dependencies. This observation covers the file-management model and the
recorded resolver context. The JaCoCo plugin model and dependency mediation,
plugin execution, and the pinned native compile remain open.
