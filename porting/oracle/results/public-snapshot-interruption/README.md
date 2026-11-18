# Public SnapshotStore after writer interruption

A Rust writer imported the six-object model from the [process-reopen proof](../public-snapshot-reopen/README.md), renamed the part definition, and called public `SnapshotStore::save_new_with_io`. An experiment-only observer delegated all I/O to public `FsSnapshotIo` and paused after one successful operation. The supervisor verified the identity of its direct child, sent SIGKILL through a PID descriptor, and reaped the writer before starting a fresh reader.

| Last completed operation | State after the writer exited | Fresh public load | First operation refused on the same-key retry |
| --- | --- | --- | --- |
| Staged file sync | Complete stage. No final name. | NotFound | Create stage. The retained stage stayed unchanged. |
| Final hard link | Both names pointed to the same complete inode with two links. | Complete model | Create stage. Both names and their bytes stayed unchanged. |
| Directory sync | Both names pointed to the same complete inode with two links after fsync returned success. | Complete model | Create stage. Both names and their bytes stayed unchanged. |
| Stage unlink | Only the final name remained. The complete inode had one link. | Complete model | Publish. The retry retained a separate complete alternate stage and preserved the final inode and bytes. |

All 32 children were reaped. The four interrupted writers had matching kernel SIGKILL wait records. A different PID descriptor failed each admission check. The normal writer completed all six operations, removed staging, and published the same model. Fresh loads before and after retries checked all six identity pairs, both typed endpoints, and the independently constructed document. The distinct-key baseline's bytes and inode stayed unchanged in each store.

The published snapshot matched the 2,202 bytes from the previous experiment. [The run summary](evidence.json) records the original receipt hash, stage and final device and inode identifiers, link counts, load results, and exact retry traces. The raw process streams and test store files stay in the run directory.

These observations cover process interruption after successful I/O operations on a local Linux ext4 filesystem with the kernel and storage still running. M06, full application restart, power-loss recovery, and archive behavior require separate evidence. In particular, the [archive writer prototype](https://github.com/ajhcs/syson/issues/271) syncs its directory after staging removal and uses different staging names on retry. SnapshotStore syncs its directory before removing the fixed per-key stage.

## Reproduce

Use a Linux x86-64 host with `/proc`, PID descriptors, cgroup v2, a running systemd user manager, Python 3 with `os.pidfd_open` and `signal.pidfd_send_signal`, Git, tar, GNU coreutils and findutils, `setpriv`, and a Rust toolchain with enough cached dependencies for an offline build. The recorded run used Rust and Cargo 1.99.1. Pick a fresh absolute path under a caller-owned local directory with hard-link and fsync support.

Run from the root of a checkout with the history for the two commits in [evidence.json](evidence.json).

```bash
bash porting/oracle/results/public-snapshot-interruption/replay.sh /absolute/path/to/fresh-snapshot-interruption
```

The launcher archives the pinned Git revisions, verifies all 114 landed source paths, and builds serially to a fresh target directory. It runs the process probe with one CPU of quota, 512 MiB maximum memory, a 256 MiB memory high threshold, zero swap, and a 48-task ceiling. The supervisor bounds each wait and writes the raw results and independent model bytes under `capture/runs/`. The launcher compares the stable outcomes with the original run and checks that the user scope has exited. PID, timing and inode values vary across runs.
