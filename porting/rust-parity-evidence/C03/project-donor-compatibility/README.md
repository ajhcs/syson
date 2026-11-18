# Project preparation and wire compatibility rehearsal

The exact corrected RR22 donor compiled off-tree with 79 local dependency
files from pinned `main`. Those files match the migration fixture verified
in #378. The historical #81 and #82 snapshots identify the original stack.
The run used the corrected #104 snapshot and its exact included fixture.
[#81](https://github.com/ajhcs/syson/pull/81) · [#82](https://github.com/ajhcs/syson/pull/82) · [#104](https://github.com/ajhcs/syson/pull/104) · [#378](https://github.com/ajhcs/syson/pull/378)

Both original #104 Cargo lockfiles passed offline locked metadata and
remained byte-identical. Each root resolved the same 11 registry tuples.
All-target Clippy passed for both crates. Seventeen named preparation
tests and ten named wire tests passed, including literal input and replay
cases. The wire crate's compile-fail doctest separately confirmed that
encoding rejects unprepared input at compile time. The owned scope ended
before the compiler lock was released.

The run establishes compatibility for the pinned crates and dependency
inputs. It leaves production source adoption, connected persistence,
application behavior and UI qualification open.

Verify the sealed observation and its 12 altered-record controls offline.

```sh
python3 -IB verify.py --self-test
python3 -OIB verify.py --self-test
```
