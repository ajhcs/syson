# JaCoCo plugin effective model

The guarded offline probe built the Maven model for `org.jacoco:jacoco-maven-plugin:0.8.14` in baseline and CI-property runs. Both runs imported ASM BOM 9.9 through the JaCoCo parent and read the BOM's OW2 1.5.1 parent. Each model contains five ASM 9.9 managed dependencies and nine direct declarations. The JaCoCo agent declaration has classifier `runtime` and scope `compile`.

Withholding ASM BOM 9.9 caused the model to fail. Overriding its selector caused Maven to request only the synthetic version and fail. The retained traces record the exact request order and errors for both controls. All four fresh JVM invocations exited successfully after reporting their model outcomes. A kernel self-test confirmed that socket creation was denied.

The file-management evidence in #352 verifies the model of one direct dependency. This packet supplies the plugin's direct declarations as inputs for an offline transitive dependency collection check. Dependency mediation, plugin execution, and the pinned native compile remain open.

Run these checks from this directory with Python 3.9 or newer.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

The verifier checks the committed traces, input identities, probe source, and file-management evidence. It runs without Java, Maven, or network access. The probe source is included for inspection.
