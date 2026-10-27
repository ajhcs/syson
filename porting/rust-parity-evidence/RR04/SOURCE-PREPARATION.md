# RR04 source preparation (execution pending)

This worker authored source only and ran no formatter, compiler, test, runtime,
network, commit or publication command. The parent owns all execution, shared
wiring and evidence. No held factory/raw-admission/initializer region changed.
The only production file is the leased serializer/error region of model lib.rs.

## Shared authority

The existing private RR02 pretty-byte counter is promoted into Document::json_size,
Document::write_json and Document::to_json_bounded. They use one private bounded,
cancellable writer over serde_json::to_writer_pretty. There is no independent JSON
formatter, semantic engine or generic Value reparse. The private publication guard
now delegates to json_size; Document::to_json delegates to the bounded owned API.
The existing Editor publication call site remains unchanged.

All public byte limits must be at most the existing 4 MiB publication ceiling;
invalid configuration refuses before writing. Counting first checks all Value
nesting to the existing depth128 ceiling, then counts canonical serialized bytes
with an allocation-free sink. This covers provisional staged documents too,
without claiming provisional semantic validity. Owned output performs a fallible
exact-size reservation only after complete admission and uses the same serializer
under the exact admitted count. An owned partial String never escapes a failure.
Normal-publication limits are not widened.

## External transport and cancellation contract

write_json does not touch its external writer until the complete count/depth pass
succeeds. Once emission starts, external I/O failure or cancellation may leave a
prefix, or complete bytes if cancellation is observed after the last write. The
caller owns transactional transport/rollback. No flush or durability is promised.
JsonOutputIo contains the error kind and acknowledged written bytes: this is not
proof of actual side effects by a writer that changes state before returning Err.
Partial writes are handled; zero/oversized acknowledgements refuse explicitly.
Interrupted retries re-enter the wrapper and poll cancellation, rather than being
hidden inside an uninterruptible custom write_all loop.

Cancellation is checked during depth visits, each serializer write step, before
owned reservation and after serialization. Bounded serializer and caller writer
calls are not internally preemptible; this is not a wall-clock or UI latency SLA.
The extra depth/count passes are linear in the value/output size. Requested
allocation counters are neither allocator overhead nor RSS measurements.

## Prepared regressions

The baseline-compatible first dense test was frozen as
cfabe3d4c84e247d973c0f67b96946de6ac6bc28996bf3b34222fe21bf7a54d1 for root's
immutable #91 red reproduction. It uses actual Transaction attachment to create
an explicitly provisional dense unknown payload: compact input under1MiB with
100000 nulls nested60 levels. No model validation bypass or production test hook
is added. Old to_json grows a roughly16MiB owned output; the new API should refuse
before that allocation. Test inputs are constructed before the allocation observer.

The final twelve tests additionally cover exact manual legacy canonical bytes,
one-less refusals, UTF8/escape growth, private marker objects and numeric lexemes,
exact4MiB count/owned requests, invalid configuration, immediate/pre-reserve/late
cancellation, short writes and Interrupted retries, acknowledged partial failure,
complete-output cancellation after the final write, zero/oversized writer counts, side-effect-before-Err honesty, and over-depth
provisional refusal before external output. Document/editor state stays unchanged.
The development-only allocator delegates identical pointer/layout arguments to
System and uses nonallocating constant TLS; root must review it before execution.

Root should format only the leased files without recursive module traversal,
freeze final hashes, run the applicable model suites/all-target Clippy, preserve
original failures, independently review and bind executed inputs to committed
blobs. This packet claims no execution PASS, downstream adoption, native/UI,
capacity or full feature acceptance. RR05 and other consumers remain separate.
