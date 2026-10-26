# Selected reference delta planning

This Rust development tool derives concrete source/resource/class-family actions
from an actual coordinator-captured Git/ZIP/compiler packet. It targets one pinned
artifact and the explicit label-to-target source delta: 7 Java sources, 114 SVGs
and 4 documentation files. It validates ownership and coverage, package/binary
paths, complete selected member queries, old resource bytes, whole class families,
compiled output identities, archive collisions, multi-release ambiguity and
signature/digest state. It emits the resource/family/document action fields used
by the coordinator's independent reassembly recipe.

The planner does not open ZIP archives or execute Git, compilers, Python, the
reference application or a database. The owned staging/oracle scripts inspect
actual local bytes, verify Git blobs and immutable artifact/source hashes, compile
seven explicit sources with annotation processing disabled, and retain the exact
packet plus receipts. Packet provenance and the truth of its source/member hashes
remain coordinator gates. A valid plan is neither a completed repack nor a native
runtime PASS, independent build or whole-source attestation.

The one admitted non-signature marker is the exact pinned artifact's empty
`META-INF/BOOT.SF`, with zero bytes and the SHA256 of empty content. It must remain
unchanged. This exception does not admit another empty `.SF`, a nonempty marker,
cryptographic signature files or manifest digest sections. Original precompile
observations and the explicit later marker classification remain distinct.

Input is capped to 8 MiB, path components to a candidate 1024 bytes, and inventory
counts to this selected slice. Class paths, source paths and required object keys
are checked; source/hash provenance is not inferred from those checks. Unknown
fields refuse. The CLI reads one regular nonsymlink packet and prints a recipe only
after complete planning. Caller-owned inputs must remain unchanged during reads;
filesystem race resistance is not claimed.

The separate repack gate preserves all unselected uncompressed payloads and
declared ZIP metadata, including CRC/size, flags, methods, timestamps, extras,
comments and attributes. Physical offsets and compressed lengths are separate
representation observations. Nested Spring Boot JAR entries must remain STORED.
No compressed-stream identity or full Maven/frontend rebuild is claimed.

Root verification recipe (not executed by the source author):

```sh
SYSON_REFERENCE_DELTA_TEST_ROOT=/absolute/owned/scratch \
  cargo test --offline --locked --manifest-path rust/tools/reference-delta-plan/Cargo.toml
cargo fmt --manifest-path rust/tools/reference-delta-plan/Cargo.toml --check
cargo clippy --offline --locked --manifest-path rust/tools/reference-delta-plan/Cargo.toml --all-targets -- -D warnings
syson-reference-delta-plan /absolute/owned/reference-inputs/packet-final.json
```

Parent C01, independent source/build binding, all inherited dependency provenance,
application behavior and UI/UX/performance acceptance remain open. The native
comparison artifact is development evidence, with no Java/Python production
fallback added to the Rust application.
