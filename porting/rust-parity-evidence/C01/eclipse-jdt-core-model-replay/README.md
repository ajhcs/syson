# Eclipse JDT Core effective models

Maven ModelBuilder 3.9.16, invoked directly with `VALIDATION_LEVEL_MAVEN_3_0`, accepted 21 of the 22 original JDT Core POMs captured in #369. It rejected 3.31.0 with the error `'modelVersion' must be one of [4.0.0] but is '4.0'.` Maven read `4.0` from the original bytes. The probe made no substitutions and produced no effective model for that release. Keep 3.31.0 in the range replay inputs until a separate run exercises normal Maven Resolver descriptor handling and range collection.

The accepted models yielded 120 direct dependencies with `jar` type and `compile` scope. Twenty-one were optional, 25 retained literal version ranges, and 95 used fixed versions. JDT Core 3.47.0 declares ECJ 3.46.100. The rejected POM accounts for the remaining five raw declarations, including one optional dependency. All five carry version ranges. The raw and Maven counts therefore describe two different stages of the same 22-input replay.

The probe compiled only the included Java harness under JDK 21.0.12.1+1, with annotation processing disabled. It passed the SHA-256-checked POM bytes directly to Maven, disabled plugin processing, and refused model lookups. Each case recorded and ignored the Central repository declaration inherited from Maven's super POM. The kernel witness confirmed socket creation was denied. A successful process exit means the probe recorded its results, including the expected model rejection.

Run both offline checks from this directory.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

The verifier binds the stored Maven trace to the input identities in #369 and checks the model diagnostic and dependency counts. The exact harness is included for inspection. Original POMs, the host-bound run manifest, and process records remain in private custody. Dependency selection and mediation, binary trust, and the pinned downstream native build still need separate evidence.
