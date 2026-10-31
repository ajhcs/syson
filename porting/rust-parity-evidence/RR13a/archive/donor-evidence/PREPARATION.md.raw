# RR13 source preparation

Base0c2cfb9764337f3d3b3730743c912e646be09c9e is the exact #32 snapshot
plus10 exact RR04/model and RR01 crate blobs. Root owns Cargo/locks,
formatter/compiler, all FIFO/filesystem/test/runtime execution, captures,
independent review, commits and publication. This file is preparation, not PASS.

Baseline-compatible FIFO source was provided first at SHA256
a747bc727ef7fede0b5430a9e009299cc52974361bd49bbd5ca5f2489e2281d7.
Root reports the actual #32 API load blocked past the3-second child deadline,
then was killed and waited. That red source/output remains separate. Final child
harnesses additionally require running1test /1passed output, preventing an exact
filter accidentally selecting zero tests from passing.

## Root and opened-file policy

This selected local adapter requires Linux x86_64. Other targets explicitly
refuse. Roots must be absolute, have no parent components, and contain no root
or ancestor symlinks. Existing directories are checked before create_dir_all and
again afterwards. The admitted directory descriptor is retained to pin its inode;
path dev/ino must still match that descriptor on load/save. Missing owned roots
may be created; invalid/symlink ancestors refuse before creating descendants.

Inputs refuse final symlinks and nonregular files. Pre-open type/size metadata,
fixed Linux O_NONBLOCK|O_NOFOLLOW, actual descriptor fstat, and post-read
path/descriptor identity/length/mtime/ctime checks cover the owned FIFO, symlink
and replacement-regular-inode fixtures. A private no-op production seam supports
deterministic test swaps between lstat and open; there is no runtime hook.
The same opener and an actual-directory fstat protect FsSnapshotIo directory sync
against a root-to-FIFO fault after publication.

This is not descriptor-relative openat, a hostile-parent/writer snapshot, a
regular-disk-I/O deadline, arbitrary-device substitution safety, or physical
power-loss qualification. Caller-owned parent directories must stay stable.
An arbitrary device can still be substituted between lstat/open; no such safety
claim is made. Fixtures create only owned workspace files/FIFOs/sockets/symlinks,
without device creation/access or service/permission changes.

## Effective byte and publication policy

Read and output ceilings are4MiB, matching the already-admitted RR04 model cap.
The prior16MiB outer workflow guard no longer admitted>4MiB models on this exact
snapshot because model admission/serialization was already capped. Oversized
regular files now refuse TooLarge before allocation/open. Read reservation is
fallible and exact for the admitted length; fixed-size chunks refuse growth
before appending beyond that length. No oversized success or partial Document
escapes; raw UTF8 must decode and model admission must pass.

Save calls the actual RR04 to_json_bounded(4MiB,false) count/depth-before-output
API before any SnapshotIo effect. The complete bounded canonical output is sent
through exactly one write_staged hook, preserving the original injected-failure
sequence: create, write, file sync, no-replace hard link, directory sync, remove.
Errors after effects can still mean published output or retained staging; no
cleanup/retry inference changes. Root checks add no SnapshotIo operation and do
not overwrite an existing destination. This is local development persistence,
not native PostgreSQL or archive parity.

## Prepared verification

Eight integration cases cover the real FIFO API counterexample, save/reopen and
non-overwrite, root/ancestor/parent rejection before creation, pinned-root
replacement, directory/socket/symlink/invalid-UTF8 refusal, exact4MiB/+1 read,
provisional>4MiB serialization before any injected I/O, and root-to-FIFO directory
sync after publication. Three private unit cases swap regular input to FIFO,
symlink and new inode between admission/open. Every risky child has a3-second
deadline with kill/wait and exact one-case assertions. The socket case runs in an
exact child whose cwd is its owned directory and binds a short relative basename,
avoiding Unix sockaddr path-length failures without changing parent-process cwd.
Inherited write/sync/publication/removal faults remain source-unchanged and must
run against final source, together with meaningful normal callers. RR04 owns
serializer allocation proof; this adapter reuses it, not a second serializer.

All runtime/format/source/evidence gates remain root-owned and pending. Native,
UI/capacity/release acceptance, broad platform support and source/artifact closure
remain OPEN; no held lease or canonical dirty source is modified by this author.
