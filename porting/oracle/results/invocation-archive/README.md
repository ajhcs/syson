# Read a SysON bundle with a separate invocation document

An existing off-tree Rust archive reader opened a prepared two-document project bundle. Its first document and General View came from the v2026.9.3 re-export in #296. Its second document was the exact alpha revision saved and reopened by the Rust HTTP service in #381. The positive bundle adds one UUID-to-name entry to the manifest. The original diagram remains bound to the original document and ViewUsage. [Native archive re-export](https://github.com/ajhcs/syson/pull/296) · [Rust service evidence](../invocation-service/README.md)

The reader returned the full prepared ZIP and each of its four members byte for byte. The original native model and representation keep their exact member bytes. The added document keeps the complete saved Rust graph, including both typing relationships and their resource and semantic identities. The reader also reported the original diagram target and node links unchanged.

| Control | Reader result |
| --- | --- |
| Manifest declares the second document but its member is missing | Exit 2 with `manifest-declared member missing` and empty stdout |
| Existing representation points to its original ViewUsage fragment in the second document | Exit 2 with `diagram target EMF object absent` and empty stdout |

The native reference ZIP is included for offline byte comparison. The prepared ZIP is the reader's exact returned output. The evidence also preserves both reports, stream hashes, the kernel witness, and the two control diagnostics. [Native reference ZIP](native-reference.syson.zip) · [Prepared ZIP](combined.syson.zip) · [Observation](observation.json)

This is a bounded read-only archive check on staged bytes. It did not import a project, start an application service, repack an archive through the Rust writer, or change grants. Product upload, durable storage, arbitrary representation formats, and browser qualification remain separate.

## Verify the record

Both verifier modes reconstruct the member inventory and control fixtures, check the full report with distinct ID roles, and reject seven altered records.

```sh
python3 -IB porting/oracle/results/invocation-archive/verify.py
python3 -OIB porting/oracle/results/invocation-archive/verify.py
```
