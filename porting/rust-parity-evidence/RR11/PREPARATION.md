# RR11 source preparation

Prerequisite: #64, commit 6dc96e2cb4ef548656f4c9ac3a24f6e37ff9c194, with #53 inherited.

This source preparation is not execution evidence. Root owns every compiler,
formatter, SQLite, Python bridge and regression invocation. Two baseline-compatible
replacement tests were supplied first; root reports both reproduced the original
counterexample against an immutable #64 snapshot in rr11-red-01. Their source
SHA256 was 541fbe6d05c63313453940f26170926f2270f34227f77d15cd76d781afba85f8.

## Candidate format and epoch policy

The selected SQLite candidate format is v2. The existing application ID, document
and clock table definitions remain; a singleton generation table contains a
16-byte SQLite randomblob epoch. Format v1 is refused without migration or writes.
This PR introduces no migration facility or native PostgreSQL schema claim.

Initialization creates the epoch inside IMMEDIATE. Normal open performs a read
transaction and pins its epoch without rotation. All subsequent load and mutation
requests carry that pinned epoch. Schema, epoch and row reads are on the same
SQLite connection and transaction. Mutation validates the actual opened database
epoch inside IMMEDIATE before any document/clock effects. Per-row revisions,
project keys, handle-local observation tokens and SQL batch rollback remain.

An independently created replacement file has another epoch, so old handles
refuse loads, create-only batches, updates and deletes even when row revisions
coincide. This fences the database actually opened, rather than trusting a
pre-open pathname stat. Concurrent replacement while an already-open transaction
is writing remains unsupported: callers must quiesce writers before file swaps.

An offline caller-owned restore can copy an old epoch and revision. Therefore the
caller must use Store::open_restored after restoring: fixed renew_epoch rotates
the restored v2 epoch inside IMMEDIATE, invalidating every prior handle. It does
not copy files. Ordinary open never performs restore/rotation implicitly.
Unannounced same-epoch byte rewinds cannot be detected by this candidate; they
remain unsupported. Rotation collision refuses/rolls back rather than reporting
a fresh epoch. This policy does not claim power-loss or malicious-writer safety.

## Fresh verification sources

The dedicated RR11 regression source retains separate original update/delete
counterexamples, healthy normal reopen, foreign observation refusal, explicit
restore invalidating create-only batches and old-format refusal. Existing storage
tests continue to own row CAS, late SQL rollback, read-only and project isolation.

Three fixed dev-only wrappers execute the exact compiled-in production bridge:

1. Replace A with B after path checks and immediately before sqlite3.connect;
   the actual B connection must refuse A's epoch with unchanged B bytes.
2. Inject SQLite COMMIT refusal after renew_epoch has written its private change;
   rollback must preserve bytes/epoch and keep prior observations usable.
3. Under owned-fixture WAL, commit another connection's epoch/name change between
   load's epoch and row queries; the returned row and epoch must both belong to
   the original read snapshot, and later old-handle access must refuse.

Wrappers admit only fixed modes, absolute existing nonsymlink files under
/workspace, and a 10-second process alarm. Test SQL is fixed source only; no
production injection hook, arbitrary SQL API, service, package or registry change
is added. Root must freeze all action, path, wrapper and original bridge source
identities, retain before/after files and execute wrappers only on fresh owned
fixtures after exact source review. Runtime evidence and peer disposition are
pending. RR12 metadata/allocation repairs are intentionally separate.
