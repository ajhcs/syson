# B06 Rust lookup for an archived native row

A standalone Rust adapter resolved the writable document and representation
from retained SysON 2026.9.3 PostgreSQL observations. The committed and
restarted reads agreed on selected identity. The original SQL selector
supplied project identity, and the recorded joins established the repeated
membership and content keys used by the adapter.
[Paired SQL read](https://github.com/ajhcs/syson/pull/360) ·
[Released repository binding](https://github.com/ajhcs/syson/pull/389)

The two identity-only cases covered the archived row and the same row with
the native migration scalars absent. Twenty-two controls refused missing
or multiple rows, wrong caller identities, broken membership or content
keys, a read-only document and malformed input. Substituting the semantic
data ID for the distinct document ID refused at document validation.

The first attempt compiled and accepted the archived row, then stopped when
the checker expected the wrong refusal. Changing the central semantic ID
breaks membership before the semantic ID check. The retry corrected that
expectation and passed all 24 cases with the same Rust adapter.

The native migration scalars describe migration state. Rust prepared-wire
reopening still requires an independent generation token and six stamps.
This run consumed archived observations. A direct query of the clone, native
transaction behavior and application integration remain unverified here.
[Rust prepared-wire rehearsal](https://github.com/ajhcs/syson/pull/387)

From this directory, verify the sealed public record and stacked provenance.
The self-tests reject twelve deliberately altered records.

```sh
python3 -IB verify.py --self-test
python3 -OIB verify.py --self-test
```
