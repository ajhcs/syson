# F46: selected immutable Element operations

This adapter exposes semantic owner, ownedElement, effective primary/short names and the current qualified-name operation on explicit immutable resources. Its separate `syson-semantic-names` crate preserves the held initializer-graph source/tests. It targets the frozen F03/F04 snapshot; prerequisites must land before retargeting to integration and revalidating. The dependency snapshot is a review base and must not be merged.

The owner operation calls the existing public `ResourceSet::semantic_owner`. A related element's owner is the owning relationship's owningRelatedElement; a relationship's raw eContainer is not substituted for owner. Owned elements flatten each ownedRelationship's ownedRelatedElement list in containment order. Known classes/containment are admitted through the existing ResourceSet adapter before operations run.

Feature names follow the first owned Redefinition only when both declared name fields are null/absent. A declared short name or explicit empty primary name stops inheritance; the primary operation does not fall back to shortName. A self redefinition yields known absence. ConjugatedPortDefinition gets `~` plus its original PortDefinition's primary name; its short name retains the ordinary declared policy. Reached cycles and chains deeper than128 return explicit Rust refusals, rather than emulating native recursion/stack failure. This depth is a proposed adapter policy, not an upstream limit or a full memory/work bound.

References use the real F04 registry with explicit resource keys and fragment identity. Bare UUIDs address local JSON node IDs. Selected sysmllibrary/kermllibrary fragments use each resource's declared fragment policy; semantic IDs never become an implicit local fallback. Missing resources, malformed/proxy/unsupported forms and wrong metamodel types stay errors. Cross-resource inheritance changes only the name target; the semantic owner's resource stays local. Private handles carry an immutable snapshot token and cannot be reused in a different snapshot, even with identical document bytes.

Qualified names use the existing domain `printable_name(name,true)` helper and the current ElementImpl algorithm. An unnamed self has no qualified name. An unnamed parent contributes no prefix; a Membership owner is skipped once through its semantic owner. This uses the effective primary name, including inherited names and printable keyword/apostrophe escaping. The source method does not query imports: F03's frozen observation evaluator remains a prerequisite boundary and is not called artificially or treated as a native resource binding.

The11 pinned source blobs distinguish Java getter algorithms, existing containment operations and the selected F04 reference policy. Tests use checked Document values, real metamodel subtype checks and the actual registry/owner adapter. Synthetic expected names are source-derived; Java/EMF runtime, proxies, cache/inverse notifications, all derived Element operations and all70 metamodel operations remain unqualified. No source graph or caller Document is mutated. Rust strings exclude malformed Java UTF-16.

The coordinator capture `validation/f46-01` passes14 Rust integration tests, formatting and scoped all-target Clippy. Its65 captured inputs, exact runner and six raw streams are retained. The existing initializer dependency emits an atomic deprecation warning under the installed toolchain; that source is unchanged and scoped Clippy exits0. Parent source review found no blocking issue. Source inspection and these local tests do not establish native acceptance.

The source-pin checker is stdlib-only:

```sh
python3 -B porting/follow-ons/F46/check_sources.py
```

Compiler replay remains coordinator-controlled. The recorded command families are `cargo fmt -- --check`, `cargo test --offline --locked -- --test-threads=1` and `cargo clippy --offline --locked --all-targets -- -D warnings`, each with this crate's manifest and the captured one-job/one-test-thread environment. Full argument vectors, paths, timestamps and hashes are in the receipt.
