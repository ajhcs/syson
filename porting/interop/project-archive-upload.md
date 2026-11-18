# SysON project archive upload control

Eclipse SysON v2026.9.3 accepted the [captured ArchiveDemo project](fixtures/archive-demo.syson.zip)
on a fresh database and returned a new project. Its re-export preserved all 12
SysML `elementId` values and generated new project, document, representation,
EMF object, diagram node and label IDs. The source [archive inventory](project-archive.md)
describes the input fixture and the matching Sirius Web reader and writer.

The control called the application's multipart `uploadProject` GraphQL
mutation, then downloaded the new project through `GET /api/projects/{projectId}`.
The multipart endpoint responded with HTTP 201. The download responded with
HTTP 200 and an `application/zip` attachment named `ArchiveDemo.zip`.

| Check | Observed result |
| --- | --- |
| Archive manifest | Maps the new document UUID to `ArchiveDemoSource.sysml` and the new representation UUID to the original General View description and migration metadata. The target URI resolves to the new EMF object ID of the same ViewUsage. |
| Model | Same containment, attributes and stable semantic IDs. The part `wheel` still resolves to its `Wheel` definition through `FeatureTyping`. The ViewUsage still points to the bundled ViewDefinition by the same URI and exposes the membership that owns the Package. |
| Diagram | `Archive overview` retains its package node, its label, placement, dimensions and styles, and has zero edges. Node and label layout entries point to the remapped node and label. |
| Missing-manifest control | Removing only `ArchiveDemo/manifest.json` returned a GraphQL `DataFetchingException` with `data: null` and left the project list empty. The reference log shows manifest deserialization failure followed by a null dereference in `ProjectUploadApplicationService.getNatures`. The original archive uploaded successfully immediately afterward. |

The [re-export](evidence/archive-demo-upload-v2026.9.3/archive-demo-reexport.syson.zip)
is the raw 3,206-byte HTTP response body. The [capture receipt](evidence/archive-demo-upload-v2026.9.3/capture.json)
records the original and re-export hashes, the new project UUID and the failed
control's GraphQL response. The [comparison](evidence/archive-demo-upload-v2026.9.3/comparison.json)
records all six raw member hashes and the complete identity mapping. The contents
of the re-export are limited to the synthetic ArchiveDemo model and diagram.

## Reproduce

An offline comparison uses Python 3 and the two checked-in ZIPs.

```sh
receipt=porting/interop/evidence/archive-demo-upload-v2026.9.3
python3 -IB porting/interop/archive-upload-compare.py \
  porting/interop/fixtures/archive-demo.syson.zip \
  "$receipt/archive-demo-reexport.syson.zip" "$receipt/capture.json" |
  diff -u "$receipt/comparison.json" -
```

For a live repeat, provision the pinned v2026.9.3 reference assets as in the
[oracle runbook](../oracle/README.md). Set `SYSON_ORACLE_RUNS` to a short path in
a new workspace directory so PostgreSQL's Unix socket stays within its pathname
limit. Start a fresh database and use that directory for the capture.

```bash
set -euo pipefail
reference_run=/workspace/build/builderbob/archive-replay
mkdir -m 700 "$reference_run"
export SYSON_ORACLE_RUNS="$reference_run/runs"
trap 'python3 -IB porting/oracle/oracle.py down' EXIT
python3 -IB porting/oracle/oracle.py up
fixture=porting/interop/fixtures/archive-demo.syson.zip
capture="$reference_run/capture"
python3 -IB porting/interop/archive-upload-control.py \
  "$reference_run/current.json" "$fixture" "$capture"
python3 -IB porting/interop/archive-upload-compare.py "$fixture" \
  "$capture/archive-demo-reexport.syson.zip" "$capture/capture.json"
```

A fresh run generates different UUIDs and ZIP bytes. The comparison verifies
the capture's hashes, archive CRCs, each identity binding and full JSON equality
after replacing the mapped identities. The two [issue #271](https://github.com/ajhcs/syson/issues/271)
fixtures cover a single-document, single-General-View round trip through the
reference application. Rust archive read/write, cross-project dependencies
and backend restart remain open.
