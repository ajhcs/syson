# Models-03 passive acquisition result

The coordinator admitted one run against the fixed #341 manifest. All five Maven Central POMs returned HTTP 200. Seven checksum responses matched the corresponding POM bytes, eight optional checksum requests returned HTTP 404, and all five signature requests returned HTTP 200. The signature bytes were retained privately without verifying a signer. The independent audit checked all 25 saved responses, 114 frozen input hashes, actual resource controls, and teardown of the owned scope.

The four Apache parent POMs are parentless. Apache 34 has no direct dependencies, managed dependencies, or JUnit selector. Its four raw profiles contain no BOM imports or local `versions.junit5` property. JUnit BOM 5.14.1 declares 18 managed dependencies and no BOM imports at its project root. The attached projection records the actual length and SHA-256 of each POM and the HTTP status, length, and digest of each response body. The private capture retains raw responses; the checked-in projection omits headers, cookies, invocation identities, local paths, and the absolute input map.

## Verify the checked-in projection

From this directory run both commands with Python 3.9 or newer.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

These commands bind the exact committed manifest and check the projection. They do not repeat the acquisition or recover the private POM bodies from their hashes.

## Remaining JaCoCo model input

The existing JaCoCo `prepare-agent` descriptor records the `initialize` phase and runtime dependency resolution. Its file-management parent chain reaches `org.apache.maven:maven-parent:44`, whose raw dependency management imports `org.junit:junit-bom` using `${versions.junit5}`. The local property is `5.12.1`. The fetched Apache 34 POM closes the literal parent chain. No inspected child or parent profile overrides this selector.

The effective import for the normal compile still requires a frozen Maven model context, including tool and JDK bytes, settings, explicit profiles, properties, and extensions. The historical settings and inspected model profiles show no override for the selector. The `5.12.1` BOM bytes remain absent from the inspected inventories. An isolated Maven model-only resolver could record the effective import request against frozen POM inputs before any separate acquisition. The run grants no effective dependency selection, acquired-code execution, or native compile acceptance.
