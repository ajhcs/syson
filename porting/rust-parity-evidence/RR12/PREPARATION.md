# RR12 source preparation

Base: RR11 #89, commit e441af7de9f66146231295ef71519016d8c2141d.
This document describes prepared source and planned verification, not execution
or acceptance. Root owns every formatter/compiler, SQLite/Python bridge/test
invocation, exact captures, independent review, commits and publication. The
source author has executed no reviewed code, compiler, DB, CLI or network action.

## Independent counterexamples

Baseline-compatible regression source was supplied before implementation at
SHA256 7b3ac1e893d4b25b988e4c3108d092c92f2e79cc8f7d7df03969c7db7f6de756.
Root reports the real immutable-baseline API counterexamples: a borrowed2MiB
update name was cloned before Limit (largest2097152 / peak2098260 tracked Rust
bytes), and a2MiB stored name was admitted through a fresh connection. Root must
retain original sources/failures separately from final qualification.

## Field and transaction contract

Borrowed Create/Update names are checked against4096 UTF8 bytes before protocol
buffering, serialization or any owned name copy. This metadata admission now
precedes document serialization; multi-invalid input error ordering is not a
compatibility guarantee. Complete bounded packets still precede bridge invocation.
The unchanged8MiB protocol and1MiB serialized-content limits remain.

The bridge requires UTF-8 database encoding inside the existing transaction.
Fresh stores use that default. Externally created UTF16 v2 files now refuse
unchanged, including open_restored; there is no format-version bump, migration,
encoding rewrite, package, driver replacement or service change. This explicit
candidate admission constraint ensures SQLite CAST-to-BLOB lengths equal Rust
UTF8-byte lengths rather than UTF16 storage lengths.

Per-connection SQLITE_LIMIT_LENGTH is1MiB+8192 before statements, allowing a
maximum valid document plus4096-byte name, both32-byte keys, integers and record
headers. SQLITE_TOOBIG maps to the existing limit refusal. This guards SQLite
value/record materialization; it is not a global SQLite parser or process-RSS cap.
No hostile database-file safety or statement-latency guarantee is added.

Schema introspection returns at most4 rows with name/type/SQL byte projections
capped64/16/1024; oversized fields become null and fail exact existing v2 schema
comparison. Generation SQL tests BLOB type and16-byte length before hex emission.
The generation is still validated against the actual connection/snapshot before
all row access and mutations. Normal reopen, explicit offline restore renewal,
CAS, project isolation, read-only policy and late-batch rollback stay with RR11.

Loads first select only typeof/byte-length/bounded-validity descriptors. Name and
content must be TEXT of at most4096 /1048576 UTF8 bytes, read_only integer0/1,
and revision integer1..i64MAX. Only then does the same snapshot fetch the text.
Worst JSON UTF8 escaping costs at most6 bytes per input byte;128 covers the
fixed result/generation/delimiter/integer envelope. Thus the pre-fetch proof is
6*(name_bytes+content_bytes)+128 <=6316160 <8388608, before json.dumps/encoding.
Every other bridge result is fixed small status/generation data. The existing
final protocol-size check remains a defensive check, not the admission boundary.
Private transaction fields receive matching byte/type checks before row writes.

## Root verification sources and scope

Eight dedicated tests cover the two original counterexamples; exact/one-extra
ASCII and Unicode names with atomic batch refusal; a valid maximum document/name
row; corrupt row types/byte lengths before payload SELECT; bounded schema and
generation results; worst JSON-escape response; and an external owned UTF16 copy.
All fixture paths are fresh caller-owned workspace files with10-second helper
alarms. Helpers expose no production SQL API or injection flag.

The dev-only response probe executes the exact included bridge source and forwards
setlimit to its actual SQLite connection. It records payload SELECT attempts,
largest returned Python String character count and tracemalloc peak. Compilation,
imports and fixture creation are outside tracing; SQLite C allocation and whole
process RSS are not measured by tracemalloc. Refusals must avoid payload SELECT,
keep responses<=128 bytes and whole-bridge Python traced peak<=9MiB. Tracing
includes the existing read(LIMIT+1) input reader's roughly8MiB preallocation even
for tiny requests. The first256KiB expectation incorrectly excluded that phase;
root retained its two failed assertions (observed8390218 bytes) while payload
SELECT count remained zero and largest returned String344 characters. The input
reader and production guards are unchanged. The worst-escape raw
NUL content is intentionally invalid model JSON: it tests bridge response bounds,
not successful Document admission. Its bounded positive response uses a32MiB
Python-allocation ceiling, not a general performance claim.

Root additionally owns the single setlimit forwarder in the inherited RR11 dev
Connection proxy, outside this author's lease. It delegates the actual guard;
none is bypassed. Full affected final-source tests and exact frozen inputs remain
required before a passing disposition. The previous95% RSS reduction stays
attributed only to #64's original64-document aggregate workload; it is not a new
RR12 measurement or replay claim. Native PostgreSQL, all-Rust host replacement,
full persistence/source/UI acceptance and unannounced rewind/live-swap support
remain OPEN under their existing units.
