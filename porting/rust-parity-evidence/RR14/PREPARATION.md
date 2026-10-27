# RR14 source preparation (execution pending)

This author ran no compiler, formatter, test, CLI, FIFO creation, Git fixture,
network, commit or publication. Root owns every execution and its exact admission.

## Bounded regular-file policy

The public read_file entry point first rejects final symlinks, nonregular types
and declared lengths above 2 MiB. Linux x86_64 opens use fixed NONBLOCK/NOFOLLOW
flags, independently matched to the installed target fcntl-linux.h definitions
04000/0400000. Actual descriptor metadata must be regular, bounded and match
admitted device/inode/length/mtime/ctime. Bounded reads retain the 2 MiB byte cap;
post-read descriptor and path metadata must remain the same admitted regular file.
No unsafe code or registry dependency is added. Candidate Git blob/mode/global
comparison algorithms are unchanged; their selected-file reads use the same gate.

Only Linux x86_64 is admitted for these fixed numeric flags. Other targets return
an explicit unsupported-platform refusal rather than a weaker blocking fallback.
Final symlinks refuse. Ancestor symlinks may alias the actual regular target;
its opened identity is checked. Stable metadata checks detect the tested races,
but they are not a hostile concurrent-writer snapshot or a guarantee that regular
disk I/O meets a wall-clock deadline. Inputs must remain stable while read.

A private callback seam exists only inside the fixed read helper; production
always supplies a no-op and cannot select another opener or expose a runtime
hook. cfg(test) unit cases replace the admitted regular path with FIFO, symlink
or another regular inode immediately before the fixed open. Each case runs in
an exact self-test child with a two-second deadline, kill and wait, so even a
regression of NONBLOCK cannot strand the root's test process. No product flag or
public test interface is introduced. Children must report one executed case.

## Prepared verification

The original baseline-compatible one-case integration source was frozen as
4c01b8b81ca747c74d7a39ea0fc3259cb2c2cb68b65fcc0a1d9e047c369c59aa
and acknowledged in root's separate red-test snapshot. It invokes the existing
CLI against an owned FIFO, kills/waits after two seconds, then fails if blocked.
This source packet does not relabel that expected red execution as a PASS.

The final four integration cases cover FIFO/directory/final symlink/missing input,
fixed /dev/null nonregular metadata refusal, exact/over-2 MiB regular payloads,
explicitly allowed ancestor aliases, and a real local Git source/candidate CLI
composition with unchanged manifest/index/config/refs. Additional assertions retain
wrong blob, modified candidate bytes, owner executable mode and undeclared delta
refusals. Original composition tests remain unchanged. Fresh fixture roots must
be existing absolute /workspace paths supplied by SYSON_COMPOSITION_TEST_ROOT.
Only fixed /usr/bin/mkfifo and /usr/bin/git run, with controlled flags, no hooks,
empty Git configuration, no fetch, no production repository mutation and no shell.
The /dev/null case reaches only the metadata gate in the corrected library and
changes no device state. Root must review that fixed read-only target explicitly.

FIFO fixtures are special files: preserve type/path metadata, never hash them by
reading their contents. Existing fixture outputs and original failing receipts
must remain retained. Root must freeze source/tool/command/input identities,
execute the red baseline and final source suites serially, independently review,
and bind results to committed bytes. No native application, browser, full source
composition/build acceptance or feature parity is implied by this preparation.
