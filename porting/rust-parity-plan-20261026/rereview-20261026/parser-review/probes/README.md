# New bounded review counterexamples (source only)

No probe has been compiled or executed by the reviewer. Export each dependency
closure from the exact immutable head listed in proposal.json into its separate
exports directory. Never edit existing worktrees or reuse another PR's model
crate. The prepared manifests resolve only those isolated exported paths.

Root must review the measurement shim (explicit unsafe delegation to System),
create offline locks using existing version/checksum tuples, freeze all exported
inputs and tools, then serialize build/run work. Each executable prints one small
JSON line: input bytes, declared output cap, peak additional requested live layout
bytes, largest allocation requested during the API call, and outcome. It prints
no names, model payload or exported text. Use /usr/bin/time -v separately if RSS is
needed. Allocator bookkeeping is not RSS or a general capacity benchmark.

F33 uses roughly0.5MiB input with60nested objects and100000nulls, safely below the
1MiB upload gate. Its known pretty serializer can request over10MiB output before
rejecting the serialized size. F34 uses a2MiB ASCII name, as specifically bounded
by the coordinator, and measures only export after Document admission. Expected
outcome for both is Limit; the defect is budget checking after large allocation.
The probe is observational and does not impose an unreviewed future allocation
threshold. Future repair belongs to existing I09/P170 work plus caller gates.
