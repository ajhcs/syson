# Physical standard-library catalogue

`Catalog::from_resources` consumes explicit classpath paths, observed resource URIs, and raw JSON. Its private result preserves every input byte and indexes serialized physical objects by resource-scoped `NodeId` and `SemanticId`. These typed domains never fall back to each other. Repeated IDs in different resources and equal UUIDs across domains remain distinct; duplicate paths, URIs, or IDs within one domain/resource refuse the whole candidate.

This is an index of stored `id`, `data.elementId`, `eClass`, optional `declaredName`, and JSON pointer fields. It is not EMF admission, metaclass validation, a namespace/type/proxy resolver, or an implementation of native `Resource.getEObject`. Physical objects outside `content` with an `eClass` member are also indexed; this is an explicit raw-format rule. Duplicate decoded JSON object keys refuse throughout this selected format. Opaque number/string tokens remain unchanged in `Resource::json()`.

The source URI binding is supplied explicitly; this crate does not reproduce Java MD5 UUID generation, authenticate an artifact, or prove that a path resolves to those bytes. C01 source/artifact/runtime binding is a separate gate. The root observer recorded native root-Namespace semantic-ID lookup refusal alongside successful LibraryPackage semantic-ID lookup; both stored identifiers remain available through this physical index.

Limits: 128 resources, 4 MiB each, 32 MiB aggregate, JSON value depth 256 (root and scalar leaves included), 2 million literal values and 2 million decoded keys across all resources, conservative scalar-copy charge at most 8 MiB per resource and 64 MiB aggregate, 200,000 physical nodes, 1,024-byte path/class labels, 4,096-byte declared names, 32,768-byte pointers, 128 MiB cumulative indexed string bytes and 256 MiB cumulative traversal pointer bytes. The two string budgets bound requested content, not allocator overhead or whole-process RSS. Scalar values do not enter the traversal stack. These limits accommodate the observed 94-file, 20,874,941-byte, depth-208 corpus; they do not change project limits or establish an elapsed-time SLA.

Run `syson-native-library-catalog ROOT RELATIVE_MANIFEST`. The manifest is an array of exact `{classpath_path,resource_uri,file}` objects. Files must be regular, nonsymlink, root-relative paths with no escaping components; ancestors are checked too. All metadata sizes are checked before payload reads, then reads and aggregate bytes are bounded again. Admission failure exits 1 with empty stdout. Success streams a compact physical-index JSON report and exits 0. An output I/O failure may leave a partial transport stream; no partial catalogue is returned by the API. Inputs must remain unchanged during reading; this is not a filesystem snapshot or hostile concurrent-writer boundary.

The full 94-file corpus and native 109,673-object index remain external. Two genuine shipped files and a compact native observation excerpt are committed for repeatable tests. Root's native observations also contain 12,162 errors and zero warnings: loaded object counts do not establish clean library loading or resolved-reference closure. C01, M15/M16, semantic loading, UI caller integration, persistence, and whole-build source binding remain open.

RR03 uses the pinned RR01 literal materializer for both resources and manifests.
The two private serde marker keys are ordinary literal object keys: a String
containing node-shaped JSON remains a String and never creates a physical node.
The original raw JSON remains authoritative and is never reconstructed from a
materialized Value. Explicit lexical map sorting preserves the prior index order
independently of downstream serde map feature unification.

The value budget now counts actual literal JSON values, without the old generic
serde decoder's synthetic arbitrary-precision-number wrapper visits. This is an
explicit corrected accounting policy. New key/scalar-copy budgets are enforced
before owned literal tree construction. Scalar charge is twice raw key-body bytes
plus raw value-string-body and numeric-token bytes, matching RR01; it is not an
allocator/RSS bound. Depth includes scalar leaves: the previous container-depth
256 plus scalar-depth257 edge is now refused, while value-depth256 is admitted.
No file, aggregate raw byte, physical-node, index-string, traversal, identity or
path budget is relaxed. The actual corpus and depth208 acceptance require fresh
root-owned verification; historical C01g PASS does not establish RR03 acceptance.

`from_resources_with_cancel(inputs, cancelled)` is an additive library API; the
existing entry point delegates a never-cancelled callback. Cancellation reaches
RR01 parsing/materialization, resource admission and physical traversal loops.
No partial catalogue is returned on cancellation. Bounded library calls and raw
byte copies are not internally preemptible; this is not a CLI process deadline,
UI latency promise or hostile concurrent-writer guarantee. Duplicate/depth errors
retain clear refusal labels; typed RR01 syntax errors now report `JSON syntax at
line:column` instead of serde's previous EOF/trailing prose. Invalid string-encoding
errors are positionless. No parser/model/loader/persistence/UI adoption is implied.
