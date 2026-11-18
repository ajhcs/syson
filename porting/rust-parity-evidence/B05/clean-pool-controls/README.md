# Clean database pool controls for B05

The SysON 2026.9.3 release showed the same observed checkout behavior in a
clean database at first boot, an unseeded restart, and the [seeded restart
capture](../release-pool-observation/README.md). Each run held ten simultaneous
checkouts backed by ten distinct PostgreSQL sessions. One checkout had source
`session` and `search_path` set to `public, "$user", public`. Nine had source
`default` and path `"$user", public`. All ten used effective schemas
`{pg_catalog,public}`, read committed isolation, and read/write sessions
with JDBC auto-commit enabled.

| Fixture | New / previously applied changesets | Project rows after startup |
| --- | --- | ---: |
| Clean first boot | 20 / 0 | 0 |
| Unseeded clone restart | 0 / 20 | 0 |

The clean first boot used a fresh PostgreSQL 15.19 cluster. We stopped both
processes cleanly, verified the stopped cluster, and restarted a separate
inode-disjoint copy. The restart left the original first-boot cluster
unchanged. After each application stop, an independent read-only SQL query
against the still-running database counted 20 applied changesets, zero
projects, and zero held changelog locks. The read-only setting in that query
belongs to the checker's SQL session. Boot and pool settings, read-only
session observer, and JVM option `-XX:ActiveProcessorCount=1` match the
seeded run. Each fixture condition contributes one capture. The cause and
distribution under other conditions remain open.

## Replay the comparison

Run from the repository root with Python 3.9 or newer and the seeded
observation present:

```sh
python3 -IB porting/rust-parity-evidence/B05/clean-pool-controls/verify_controls.py
python3 -OIB porting/rust-parity-evidence/B05/clean-pool-controls/verify_controls.py
```

Both commands verify the fixture digests and compare the ten ordered rows
and pool settings for each clean fixture with the seeded capture. They also
check the recorded migration, unseeded database and custody summaries.
The files contain an allowlisted projection. The databases, logs, endpoints,
roles and connection IDs stay in the private capture. The replay reads only
the checked-in fixtures. B05 still needs pinned-source applicability, a Rust
database implementation and a differential comparison.
