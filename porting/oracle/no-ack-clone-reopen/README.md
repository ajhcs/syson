# Reopen the committed SysON edit from a PostgreSQL clone

Eclipse SysON v2026.9.3 reopened the full saved project, representation, and selection route in fresh Chromium. The Details panel displayed the Documentation value that PostgreSQL had committed before the original browser request was aborted. The recovery run used a verified copy of the stopped database and issued zero Documentation edit requests. [Commit-before-abort evidence](../no-ack-commit/README.md) · [Released SysON](https://github.com/eclipse-syson/syson/releases/tag/v2026.9.3)

The 1,055 files and directories in the sealed standby matched the source bytes and modes and had distinct inodes. The run moved the standby into its owned runtime, started PostgreSQL on the copy, and independently read the selected committed row before starting Java and after Chromium reopened the view. The row matched the historical observer under a distinct PostgreSQL start and stayed identical across the two clone observations. The source cluster stayed stopped and unchanged. [Recorded projection](observation.json)

## The one admitted layout refresh

The after-only gate forwarded exactly one `layoutDiagram` refresh tied to the latest delivered selected revision. Its `UNCHANGED` state matched one node, one label, and zero edges without a geometry delta. The response was HTTP 200 with the expected typed success payload. Chromium completed without browser errors, unclassified GraphQL operations, redirects, or transport faults.

Read-only checks matched the selected representation content bytes, content row and audit fields, and metadata across all four observations. The set of peer representations was empty throughout. The selected content row's PostgreSQL version first changed between the pre-Java observation and the observation immediately before forwarding the refresh. That interval included Java startup and initial browser loading. The version changed again during the refresh and stayed stable afterward. Both export ZIPs passed CRC checks. The semantic document and diagram representation members were byte-identical, and the manifests parsed to the same JSON.

This completes the selected official SysON **clone reopen**. The original in-place attempt in #386 remains partial. Rust behavior requires its own recovery run. The controller exited zero, verified its inputs afterward, and confirmed the owned scope was empty or absent.

## Verify the retained projection

Run from the repository root. The verifier checks the parent evidence, clone custody, exact request counts, layout row comparisons, export equivalence, and controller outcome. It rejects nine altered records. The public projection contains hashes and bounded comparisons for privately retained receipts, without database paths, row IDs, route strings, request payloads, or browser captures.

```sh
python3 -IB porting/oracle/no-ack-clone-reopen/verify.py
python3 -OIB porting/oracle/no-ack-clone-reopen/verify.py
```
