# JaCoCo model dependency selects the JUnit 5.12.1 BOM

Apache Maven ModelBuilder 3.9.16 on Adoptium JDK 21.0.12.1+1 opened the
`org.apache.maven.shared:file-management:3.2.0` dependency identified in
the retained JaCoCo `prepare-agent` metadata as the probe's root POM. It
read three pinned parents, including Apache 34 from #345, and requested `org.junit:junit-bom:5.12.1`.
The offline resolver withheld that POM. Maven reported an unresolved import
and a missing managed version for `junit-jupiter-api`.

| Scenario | Active parent 44 format profile | BOM requested |
| --- | --- | --- |
| No CI property | `format` | `5.12.1` |
| `env.CI=true` | `format-check` | `5.12.1` |
| Synthetic user-property override | `format` | `99.99.99-model-probe` |

All cases activated the JDK profiles `java11+` and `jdk9+`. The request
used the retained settings scope with no external profiles or toolchains,
disabled the explicit `sonar` and `verify-iplog` profiles, and used an
in-memory resolver with no relative-file lookup. A kernel guard denied
socket creation in a control process and applied the same restriction to
every JVM. The experiment ran only the Maven model-building API.

The exact Java probe and three sanitized traces accompany this report. The
first outer checker refused the trace because it expected a different
resolver overload and a single diagnostic. A separate read-only checker
validated the retained generic coordinate lookup and both diagnostics.

From this directory, check the retained projection and the exact harness
and parent artifact bindings:

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

These commands verify the recorded observation. BOM acquisition, a complete
effective model, plugin dependency mediation, and the native compile remain
open.
