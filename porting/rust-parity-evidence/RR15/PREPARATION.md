# RR15 source preparation (not execution)

Base: PR #12, ca80c1960620f87b940bd1e2040597efd7842052. Repairs PARSER-RR-05.
The baseline-compatible FIFO regression was frozen before implementation at
SHA256 a9ba1ccca4bec1bf95f45ad9352d02bb48eced86e895b1e2f04cbf017c758d6a;
root copied it independently before permitting extensions. This preparation
records authored source only; it is not a PASS or native parity attestation.

The CLI retains case-sensitive .sysml/.kerml selection before input admission,
its existing grammar calls, syntax/error observation fields and JSON formatting.
Linux x86_64 requires a final-component nonsymlink regular file <=1024 bytes
before open. O_NONBLOCK=0x800 and O_NOFOLLOW=0x20000 are the existing reviewed
RR14 pattern and match the installed target bits/fcntl-linux.h definitions.
The opened descriptor must match admitted dev/inode/length/mtime/ctime including
nanoseconds. Input capacity is reserved once for that admitted <=1024-byte size;
a 256-byte stack read and one extra bounded read refuse growth before copying it.
Descriptor and pathname metadata must match after reading, and UTF-8 is required.
Other targets refuse explicitly without guessing platform flags.

Final symlinks (including dangling links) refuse. Stable ancestor aliases are
permitted: actual opened identity, not lexical spelling, is authoritative.
Parents and contents must remain stable during admission/read. The policy is not
openat parent pinning, a hostile-writer coherent snapshot, general arbitrary
special-device substitution protection, or a regular-disk-I/O deadline. A device
can theoretically replace an admitted file between lstat and open; there is no
such fixture or device access in these tests. No schema/parser/fallback changes.

Five integration cases cover the original FIFO, .sysml/.kerml and lexical-error
byte-for-byte original observations, exactly1024 UTF-8 bytes including Unicode
and1025-byte refusal, invalid UTF-8/suffix/directory, final symlinks and a stable
ancestor alias. Expected bytes come from immutable #12 original observer stdout,
not the repaired program. Three input and three original output files are explicit
compile-time fixture dependencies in SOURCE-FREEZE.json. Historical captures are
output compatibility oracles only; no reference/native parser run is inferred.

One private source-only unit case launches its exact one-test child and checks
both running1test and1passed. Private callbacks default to no-ops in the CLI;
they deterministically replace the input after admission with FIFO, final link,
same-size different inode; replace after reading; grow after reading; and retarget
an ancestor alias. Renamed original files remain present so inode reuse cannot
make a replacement accidentally share the admitted identity. Test callbacks are
not a public operation, environment hook or production injection interface.

All real CLI and race children have three-second deadlines with kill+wait on
expiry. Timeout is a test failure, never a successful refusal. FIFO creation uses
fixed /usr/bin/mkfifo with bounded waiting; no shell, service or production path.
SYSON_TEST_DIR is mandatory and canonicalized under /workspace; fixtures are
fresh process-specific directories and retained for root evidence. Refusals must
exit unsuccessfully without observation stdout. No test has been run by author.

Root owns formatting, compiler/tests, immutable baseline red, independent review,
all action/evidence admission, commit/publication. Budgets are400production and
900authored lines; formatting counts remain to be measured by root. No acceptance
of native parser parity, UI, performance, power loss or hostile filesystem races.
