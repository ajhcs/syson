# Eclipse Filesystem direct Maven models

Maven ModelBuilder 3.9.16 received all 25 original Filesystem POMs from #390 at `VALIDATION_LEVEL_MAVEN_3_0` (level 30). It built 24 models and rejected version 1.9.500 with `'modelVersion' must be one of [4.0.0] but is '4.0'.` The probe passed the original bytes directly to Maven. A separate wrong-hash control exited 41 before invoking ModelBuilder.

The 24 accepted models returned 90 direct dependencies in declaration order. They were all compile-scope JARs, with 51 fixed versions and 39 ranges. The three Equinox and OSGi ranges in the rejected POM appear only as raw declarations. The observation preserves every returned tuple, including the exact optional field and exclusion count.

The guarded run compiled only the included probe with annotation processing disabled. It disabled plugin processing and refused model lookups. The kernel denied socket creation and `io_uring_setup`. The controller confirmed teardown. These checks cover direct ModelBuilder behavior. Maven's default descriptor-reader policy, graph collection, final version selection, mediation, binary trust, and native compilation remain open.

Run both checks from this evidence directory.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

The verifier ties each input to #390, reconstructs the two stdout traces, and checks the model counts, rejection and wrong-hash control. Raw POMs, host fields, and controller logs remain private.
