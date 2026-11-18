# Released HTTP handler bindings

This packet joins the Actuator capture from [#83](https://github.com/ajhcs/syson/pull/83) to the official SysON 2026.9.3 release JAR and to pinned Sirius Web Java sources. Each of the 25 observed Sirius MVC handler rows names an exact JVM method descriptor declared in one of ten packaged controller classes. The class names match Java source files at Sirius Web revision `3191e2b8990373afee3415f6b26f8c5039a28321`. The verifier checks method declaration names in the sources and the full erased descriptors in the class files.

The capture contains 54 dispatcher rows. The verifier records the packaged `FrontendRouterConfiguration` class that encloses the generated frontend fallback. Spring, OpenAPI, filters, servlets, and other rows outside the selected Sirius MVC handlers remain outside this descriptor comparison.

The target and released SysON source objects contain byte-identical root POM, ordinary and development properties, SysML external-resource loader, and frontend entry point. Both property files also match the corresponding resources inside the official JAR. The target-to-release diff spans 125 paths, including 104 metamodel and 17 view paths. The packaged Sirius JARs declare Maven version `2026.9.3`. Reproducible source-to-binary provenance, the exact target build, effective routes under other runtime configurations, and the Rust production transport remain open.

## Reproduce

Use Python 3.11 or newer, the official release JAR (SHA-256 `85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1`), a SysON Git clone with revisions `6ba2ac64e6b16ee2b05752ad55395eebb87d4910` and `b380dada415ce05155150f23cb3b74ae21cc45e4`, and a Sirius Web Git clone with the revision above. From the root of this PR checkout, pass those local paths to the verifier.

```sh
python3 -IB porting/rust-parity-evidence/C01f/release-handler-binding/verify.py \
  --jar "$SYSON_RELEASE_JAR" --syson "$SYSON_SOURCE_REPO" --sirius "$SIRIUS_SOURCE_REPO"
python3 -OIB porting/rust-parity-evidence/C01f/release-handler-binding/verify.py \
  --jar "$SYSON_RELEASE_JAR" --syson "$SYSON_SOURCE_REPO" --sirius "$SIRIUS_SOURCE_REPO"
```

Both modes rederive the checked-in JSON from the committed capture, release artifact, and Git objects. Six controls reject a changed capture, a truncated class, a missing method, a different descriptor for a known method, a missing ZIP member, and a duplicate ZIP member. The verifier reads archives in memory and performs no network requests or Java execution.
