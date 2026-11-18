# Released B06 repository binding

This verifier reads the official SysON 2026.9.3 JAR without starting Java or
PostgreSQL. It binds 11 classes from the embedded Sirius modules, including
the semantic, representation metadata and representation content persistence
methods. All three selected methods carry default transactional annotations.
The respective caller references appear separately in the classfiles.
The inspection leaves the effective enclosing application transaction open.

The released changelog matches the earlier executed 20 changesets across
29 resources. The retained selected-row query joins project membership to
semantic data and its distinct document row. It checks document read-only
state, representation metadata identity and target, then binds content using
the compound key and both component IDs. The projection replaces fixture
values and retains the actual column and join conditions. [F26 changelog](https://github.com/ajhcs/syson/pull/346) · [Executed SQL](https://github.com/ajhcs/syson/pull/360)

The released representation content class stores two migration scalars.
The #387 rehearsal preserves six independent stamps and uses a synthetic
outer generation. Exact native mappings for the six stamps, a native
compare-and-swap token, a combined application transaction and exact
target-build activation remain open. The result informs B06 preparation
within its existing implementation gates. [Paired wire rehearsal](https://github.com/ajhcs/syson/pull/387)

From this directory, verify the sealed observation and 13 altered records.

```sh
python3 -IB verify.py --self-test
python3 -OIB verify.py --self-test
```

Add `--jar` and the exact official release archive to reconstruct the
classfile and changelog projections. The sibling F26 brief gives the
artifact acquisition instructions. Add `--observed-join` with the
digest-matched retained SQL copy when available to verify the sanitized
join projection. [Release acquisition](../f26-release-changelog/README.md)
