# PostgreSQL immediate-shutdown recovery

The test replayed the paired SQL write from #360 on a disposable, inode-distinct PostgreSQL 15.19 clone of the synthetic SysON release fixture. A second SQL session then changed only the document by appending two spaces and updating its test timestamp. An independent reader still saw the acknowledged pair.

Two observations matched the second writer's transaction ID and idle-in-transaction state. A checkpoint confirmed that PostgreSQL flushed WAL beyond the pending update. The harness sent SIGQUIT through the verified owned postmaster's PID file descriptor with the pending transaction still open. The writer never received a COMMIT. Its client exited successfully after stdin closed, which gives no commit acknowledgment. A fresh postmaster performed automatic WAL recovery, and a fresh session read the exact previously committed pair and migration ledger.

The first scope refused launch under the unchanged I/O guard. Its private input and pressure records were preserved before the successful retry. The original cluster retained its bytes and inode identities. The synthetic SQL does not exercise either application, test physical power loss, or establish an application save-acknowledgment contract.

From the repository root, verify the recorded projection against the exact #360 parent in both Python modes.

```sh
python3 -IB porting/persistence/evidence/b28-sql-wal-crash/verify.py
python3 -OIB porting/persistence/evidence/b28-sql-wal-crash/verify.py
```
