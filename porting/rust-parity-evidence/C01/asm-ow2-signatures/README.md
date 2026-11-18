# ASM 9.9 detached signatures

GnuPG 2.4.4 verified the exact ASM, ASM Commons, and ASM Tree 9.9 JAR and detached-signature bytes against OW2's advertised primary key. Each verification reported one good RSA/SHA-512 signature and the same full primary and signing fingerprint. The verifier operated on private copies with an explicit keyring and an empty GnuPG home. A kernel guard rejected IPv4, IPv6, and Unix socket creation. [OW2 artifact signing policy](https://asm.ow2.io/) · [OW2 public key](https://asm.ow2.io/KEY.txt) · [ASM release history](https://asm.ow2.io/versions.html#versions)

The OW2 response matched the public-key packets pinned before capture. The 2011 UID certification and encryption-subkey binding in those packets use SHA-1. The three detached JAR signatures use SHA-512. This run establishes verification against the exact published key snapshot. It does not establish independent revocation history, grant executable trust, install the JARs, or run the pinned SysON compile.

Run both checks from this directory with Python 3.10 or newer.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

The replay checks the sealed observation and binds all three body and signature digests to inherited capture observations. Core ASM was captured separately before the Commons and Tree acquisition. The replay uses published metadata and makes no network requests. Raw keys, signature bytes, JARs, machine paths, and private logs remain outside Git. [Core capture observations](https://github.com/ajhcs/syson/tree/e80cb69f4dc35b1ad8a1193b7eecfc4553cb04b5/porting/rust-parity-evidence/C01/batch33-passive-closure/captures/network/downloads/org/ow2/asm/asm/9.9) · [Commons and Tree capture](https://github.com/ajhcs/syson/pull/361)
