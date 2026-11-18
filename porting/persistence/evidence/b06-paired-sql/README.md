# Paired writes on the release PostgreSQL schema

PostgreSQL 15.19 listened on a private Unix socket in a fresh clone of one
synthetic VehicleAssembly project stored by SysON 2026.9.3. The parent
changelog projection records 20 executed changesets for the same release.

The first attempt refused before either write because it compared the
archive representation UUID to the compound storage key. A separate
read-only clone query found the archive UUID in the metadata UUID field.
It verified that the storage key consists of the semantic-data UUID, a
`#`, and the metadata UUID. The retry required both exact identities
and the target object ID.

The retry first updated the document and triggered a late `NOT NULL`
violation on the representation. Rolling back to the savepoint restored
all observed fields. A separate transaction committed both payloads and
both representation migration markers. Each payload gained one trailing
space and retained its parsed JSON. The marker values were synthetic.

Repeating both writes with the original compare-and-swap observations
affected zero rows. A fresh read-only session and another after a clean
postmaster restart observed the same committed state. Both postmasters
exited cleanly. The original source cluster’s byte and identity manifest
was unchanged.

From this directory run both offline verifier modes.

```sh
python3 -IB verify.py
python3 -OIB verify.py
```

The verifier reads the projection without opening a database. The SQL
probe exercises the release schema directly. Application transaction
behavior, valid migration values, Rust integration and crash recovery
remain open.
