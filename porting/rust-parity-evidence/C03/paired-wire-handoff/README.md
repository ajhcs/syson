# Prepared project wire and paired caller handoff

An isolated Rust caller prepared the exact #104 project fixture using the
dependency closure qualified in #382. It encoded the authoritative wire,
checked the paired semantic and representation row contexts, decoded and
reprepared the project, and compared the re-encoded bytes. Both lanes and
all six migration stamps survived. [#104](https://github.com/ajhcs/syson/pull/104) · [#382](https://github.com/ajhcs/syson/pull/382)

Eight named tests passed. They cover the complete pair, stale and mixed
generations, foreign project or revision, missing input, distinct semantic
and representation metadata identities, foreign or missing target, payload
mismatch and all six stamp mismatches. Offline locked metadata, Clippy and
the kernel witness also passed. The probe's new lock retained all 11
registry tuples. Input readback and owned scope teardown passed before the
compiler lock was released.

The in-memory pair uses synthetic UUIDs and generations. Its distinction
between a representation metadata UUID and the compound storage key follows
the B06 release observation. Native stamp mapping, SQL, transactions and
production source adoption remain open. [B06 evidence](https://github.com/ajhcs/syson/commit/8ce1ae50bc025214d0ad8e60691921c7fc61f6aa)

Verify the sealed observation and its 16 altered-record controls offline.

```sh
python3 -IB verify.py --self-test
python3 -OIB verify.py --self-test
```
