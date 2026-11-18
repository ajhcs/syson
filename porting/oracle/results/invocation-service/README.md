# Seeded invocation models across a Rust HTTP service restart

The existing standalone Rust draft service loaded two byte-exact invocation graphs from the preceding edit proof. It forked each saved revision, refused a stale-generation edit, renamed the actual target by its semantic element ID, and saved the new revision. A fresh service process at the same address returned both complete saved graphs. Across the restart, the usage and invocation typing links retained their resource fragments and semantic element IDs. [Input edit evidence](../invocation-edit/README.md)

| Seed | Service edit | Reopened result |
| --- | --- | --- |
| Concrete invocation in alpha | Rename `RenamedA` to `RestartedA` | One declared-name change, with both typed relationships retained |
| Abstract invocation in beta | Rename `RenamedA` to `RestartedA` | The same one-field change, with the abstract marker and links retained |
| Typed calculation in alpha | Read only | Exact original graph, including the invocation and return typing |

The controller installed the captured EMF JSON files in a disposable service root before launch. The service's existing text importer and its six-allocation fixture constrain the normal import route. Its definition selector is tied to a synthetic UUID. It displayed “No definition loaded” for these graphs while returning the complete model JSON. The controller supplied the captured semantic element ID directly to the existing rename handler. The capture covers the raw HTTP service workflow and restart. Product upload, interactive selection, browser acceptance, persisted editor history, and B06 database durability remain separate.

The retained evidence covers 15 HTTP exchanges before restart and five after. Both original seeds refused direct edits and kept their bytes. Both project revisions published and reopened under the same revision key, independently. The exact typed-calculation control remained unchanged. The connected loopback witness passed, and the supervisor proved quiescence for the witness and both service process groups before reaping their leaders. [Sanitized observations and saved graphs](observation.json)

## Verify the record

From a checkout with the prerequisite evidence, run both commands. The verifier reconstructs the expected document changes, checks every HTTP observation, and rejects six altered records.

```sh
python3 -IB porting/oracle/results/invocation-service/verify.py
python3 -OIB porting/oracle/results/invocation-service/verify.py
```
