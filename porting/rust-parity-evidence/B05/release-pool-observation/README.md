# SysON release pool observation for B05

At application readiness, we held ten simultaneous Hikari checkouts to ten
distinct PostgreSQL backends. We queried the held sessions in borrow order.
The first reported `public, "$user", public` with source `session`.
The remaining nine reported `"$user", public` with source `default`.
All ten reported effective schemas `{pg_catalog,public}`, read committed
isolation, and read/write sessions. JDBC auto-commit was enabled on every
checkout. No application write or rollback was exercised.

The subject was the official SysON 2026.9.3 release JAR with Spring Boot 4.1.1,
HikariCP 7.0.2, PGJDBC 42.7.13, and PostgreSQL 15.19. The JVM ran with
`-XX:ActiveProcessorCount=1`. The observer held all checkouts before running its sequential
read-only queries of PostgreSQL settings. Both processes shut down cleanly
and the original cluster stayed byte-identical.

The stopped source cluster had applied all 20 migrations at first boot. Its
project list was empty before and after a missing-manifest rejection and
contained one synthetic Rust-generated project after a successful archive
import. This capture booted a private copy of that cluster. During the
observed boot, Liquibase applied zero changesets and reported 20 previously
applied. The observation does not establish what changed the first session's
settings, whether every startup yields this distribution, or how a clean
first boot behaves. The release artifact identity is separate from the
plan's pinned-source build.

## Replay the published observation

From the repository root, with Python 3.9 or newer:

```sh
python3 -IB porting/rust-parity-evidence/B05/release-pool-observation/verify_observation.py
python3 -OIB porting/rust-parity-evidence/B05/release-pool-observation/verify_observation.py
```

Both invocations verify the published fixture digest and recompute the same
two search-path counts from its ten rows. The fixture omits the database,
endpoint and role details, backend IDs, process and host details, and raw logs.
These commands replay the checked-in data. A new application or database run
is a separate experiment.

A native follow-up should record three fixture conditions independently:
`clean-first-boot` with a new database, `clean-unseeded-restart` after its clean
stop, and `synthetic-archive-restart` after a known archive import. Pin the
release artifact, database and JDBC versions, processor count, application
settings, and fixture bytes. Hold ten simultaneous connections for each
capture and compare the setting, source, reset value, and effective schemas.
This packet supplies only the seeded-restart capture. It leaves B05's
pinned-source applicability, Rust implementation and differential acceptance
open.
