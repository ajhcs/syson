# SLF4J 1.7.36 historical JAR signature

C01 holds `org.slf4j:slf4j-api:1.7.36` outside the executable dependency set. The detached signature advertises SHA-1. C01 accepts SHA-256, SHA-384, or SHA-512 for detached artifact signatures, so the SHA-1 result stops at packet inspection. The packet carries a timestamp of February 8, 2022 at 13:33:50 UTC and claims issuer key ID `2C7B12F2A511E325` in an unhashed field. This observation leaves the issuer claim unverified.

Two retained copies of the 41,125-byte JAR matched the SHA-256 already recorded in the [passive Maven inventory](../batch33-passive-closure/requests.json). The public key and detached signature came from the [SLF4J publisher](https://www.slf4j.org/public-keys/ceki-public-key.pgp) and [Maven Central](https://repo.maven.apache.org/maven2/org/slf4j/slf4j-api/1.7.36/slf4j-api-1.7.36.jar.asc), respectively. Their lengths and SHA-256 digests identify the inspected inputs.

Offline packet inspection derived the primary key fingerprint and compared it with the publisher's documented key for releases before August 8, 2022. SLF4J 1.7.36 was released on February 8, 2022. The fingerprint match establishes the observed key identity. Verification of the signature against the JAR and verification of current key authority remain pending. [Publisher signing policy](https://github.com/qos-ch/slf4j/blob/master/SECURITY.md#before-2022-08-08) · [Release history](https://www.slf4j.org/news.html)

Run from the repository root:

```sh
python3 -I -B porting/rust-parity-evidence/C01/slf4j-historical-signature/verify.py
```

The check validates the committed metadata and the JAR inventory binding offline. Cache installation, dependency execution, native compilation, and runtime closure remain unqualified.
