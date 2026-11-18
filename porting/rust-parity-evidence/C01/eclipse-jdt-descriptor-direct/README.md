# Eclipse JDT Core direct descriptor comparison

The guarded offline replay read the same original JDT Core POMs through two Maven 3.9.16 paths. The diagnostic path used ModelBuilder at `VALIDATION_LEVEL_MAVEN_3_0`. The native path used Maven's `DefaultArtifactDescriptorReader` with Resolver 1.9.27 and a POM-only adapter. The native reader configured ModelBuilder at validation level 0 and used the Maven session policy that ignores invalid and missing descriptors.

| JDT Core | Diagnostic path | Native reader |
| --- | --- | --- |
| 3.30.0 | 5 direct edges | 5 direct edges |
| 3.31.0 | Rejected `modelVersion` | 5 direct edges |
| 3.32.0 | 5 direct edges | 5 direct edges |
| 3.47.0 | 6 direct edges | 6 direct edges |

The diagnostic path reproduced the #371 rejection of literal `modelVersion` `4.0`. The native reader built the 3.31.0 descriptor and returned the five declared compile edges from the unchanged POM, including the optional Eclipse Team Core edge. The three accepted diagnostic controls matched the native reader's ordered edge data. The 3.47.0 descriptor names ECJ 3.46.100.

The separate diagnostic Codegen ancestor case built its root descriptor and reported seven direct declarations. The repository aggregation guard stopped that case before dependency traversal with `repository aggregation outside scope` and exit 1. The runner retained the pre-failure direct traces and stopped before the remaining twelve scheduled cases. Keep 3.31.0 in scope for graph replay. This run leaves graph collection, target version selection, mediation, binary resolution and the offline native compile open.

Run both public checks from this directory.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

The verifier checks the original POM identities against #369 and #371, reconstructs both stored direct traces, and checks the failure boundary. The socket-denial witness and controller confirmed the retained attempt ran offline and the owned scope was absent after cleanup. The full host-bound run records stay in private custody.
