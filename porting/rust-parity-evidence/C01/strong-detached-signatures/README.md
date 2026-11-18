# C01 detached signatures for JaCoCo, Maven File Management, and Commons IO

Offline GPGv verified six retained binary RSA/SHA-512 signatures against three pinned OpenPGP v4 keys. The results cover JaCoCo 0.8.14’s Maven plugin, runtime agent, core, and report JARs, Maven File Management 3.2.0, and Commons IO 2.19.0. The verifier used an explicit keyring and an empty private GnuPG home for each group. A kernel probe confirmed that IPv4, IPv6, and Unix sockets were denied.

- File Management’s key came from the [Apache Maven key catalog](https://downloads.apache.org/maven/KEYS). The [ASF committer index](https://home.apache.org/keys/committer/) and [Apache Trusted Releases](https://releases.apache.org/report/maven/3.9.16/apache-maven-3.9.16-src.zip.asc) corroborate the primary fingerprint.
- Commons IO’s key came from the [Apache Commons key catalog](https://downloads.apache.org/commons/KEYS). The [ASF committer index](https://home.apache.org/keys/committer/) corroborates the primary fingerprint.
- The JaCoCo key came from [Apache Groovy’s pinned downstream keyring](https://github.com/apache/groovy/commit/8e0871db4b6e853ea0b5245200f90965269fc95e). JaCoCo’s [signed v0.8.14 release tag](https://github.com/jacoco/jacoco/releases/tag/v0.8.14) corroborates the signer’s key ID.

The historical JaCoCo and Commons IO key self-certifications use SHA-1. Each detached JAR signature uses SHA-512. The result establishes a match to the pinned key packets. Current key authority, legacy SHA-1 artifact signatures, SLF4J, cache installation, execution, and native compilation remain outside this result.

From the repository root, run:

```sh
python3 -I -B porting/rust-parity-evidence/C01/strong-detached-signatures/verify.py
```

The command checks the observation and binds all six JAR and signature digests to the [passive capture](../jacoco-binaries-passive-run/observation.json) and the [projected direct-core receipt](../batch34-direct-core/captures/network-projected/receipt.json). It validates committed metadata offline. Replaying GPGv requires the original JARs, signatures, and key packets.
