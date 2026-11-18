# Commons Text effective model

Maven ModelBuilder 3.9.16 built the Commons Text 1.10.0 model at `VALIDATION_LEVEL_MAVEN_3_0` from four pinned POMs. The probe supplied the original Commons Text, Commons Parent 54, and Apache Parent 27 POMs from the #364 input roster, plus the JUnit BOM 5.9.1 captured in #373.

Commons Text sets `commons.junit.version` to 5.9.1. Maven used that value for the Commons Parent import, which defaults to 5.9.0. The effective model contained all 18 JUnit management entries from the imported BOM, filled in version 5.9.1 on the direct JUnit Jupiter `test` declaration, and retained Commons Lang3 3.12.0 as a direct `compile` declaration.

Withholding the BOM reproduced the #364 model blocker. Maven reported the missing import and the missing version for the JUnit Jupiter declaration. The withholding process received just the three parent-chain POMs and emitted no effective model. Both processes exited successfully after recording their respective model results.

The harness compiled with annotation processing disabled and invoked only Maven ModelBuilder, with plugin processing disabled. The guards denied socket creation and process-group changes. The controller gave each process up to 30 seconds to exit. On exit or timeout, it killed the owned group and kept the compiler lock until it confirmed teardown. The public observation preserves both model traces, the teardown outcomes, and the aggregate run receipt hash. Host paths, process identifiers, and individual teardown proof hashes stay in private custody.

Run both offline checks from this directory.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

This result describes the Commons Text model. Maven Resolver collection for the target, dependency mediation, binary acquisition and trust, cache installation, and the offline SysON native compile remain unverified by this replay.
