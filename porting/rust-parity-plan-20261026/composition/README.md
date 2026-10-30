# Fail-closed storage-split preservation guard

This C03 library validates proposed contract changes. Its default seal is `PENDING_ROOT_FREEZE`, so it refuses proposals. It provides no constructor, writer, installer, generator, subprocess or acceptance command. Current contract 1.0.9 does not import it; existing contracts, locks, units, holds and application behavior remain unchanged.

The module preserves the complete reviewed 191-line guard: exact prior 792 unit records, 1584 typed nodes, full contract and 55 historical records must survive before the unchanged earlier preparation guard verifies its 790-unit view. A future real seal or tools shim requires a separately reviewed adoption; this preparatory library grants neither.

Standalone tests use three verbatim captured baseline JSON files totaling 6,355,854 bytes. [Their provenance](fixtures/baseline-provenance.json) records commit `c1577a91f219b283d12a7b1131f7996f2b7737c1`, Git blobs, byte lengths and SHA-256. These are active machine-generated test inputs, never hidden executable code.

The two tiny added records are explicitly **SYNTHETIC UNSEALED** controls. They contain no adopted future work. Only a scoped test fixture patches the policy seal; the default API refuses this synthetic proposal before reading its callback. All nine original controls remain, including unknown original fields, source pins, caps, historical records, typed edges and an independent cycle defense.

Run `python3 porting/rust-parity-plan-20261026/composition/test_storage_split_guard.py` and repeat with `python3 -O`. Both modes must execute the same controls; production guard checks use exceptions rather than removable assertions. The tests explicitly import the unchanged existing `tools/preparation_splits.py`.

This is source preparation, not RR13/M06 registration or storage/native/UI acceptance. The complete RR13a/b draft and original 166-line identity/SnapshotStore obligation remain required. Root owns independent qualification, fixture/lock byte checks, CI, future seals/shims, runtime and publication. No ownership hold or human reservation is released.
