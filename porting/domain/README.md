# Rust domain lane: bounded compatibility foundation

Baseline: eclipse-syson/syson `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`.
Owned paths: `rust/crates/syson-domain` and `porting/domain`.

This crate is an initial independently buildable domain slice, not complete SysON
or SysML compatibility. It has no Java, TypeScript, network, database, or runtime
library dependency, and no build script. It uses Rust standard library only.
Existing root LICENSE, LICENSE-LGPL, and NOTICE are retained unchanged. Translated
files and adapted upstream tests retain Obeo copyright, EPL-2.0 license, and
contributor attribution. The handwritten SHA-1 primitive is solely UUIDv5 identity
construction; no security or authentication use is implied.

## Source and test traceability

All Java paths below are under `backend/`. `services` denotes
`services/syson-sysml-metamodel-services/src/main/java/org/eclipse/syson/sysml/`.
`metamodel` denotes `metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/`.
The exact upstream tests are in the analogous `src/test/java` paths.

| Source/method | Rust API | Validation | Status / boundary |
| --- | --- | --- | --- |
| services/metamodel/helper/NameHelper.parseQualifiedName | names::parse_qualified_name | Java split edge fixtures | Translated, deliberately not a quoted-name parser |
| NameHelper.unescapeString | names::unescape_string | null, quote, escape fixtures | Translated; only strips paired outer quotes |
| NameHelper.toPrintableName | names::printable_name | all 117 keywords, ASCII/name edges, 4 exact NameHelperTest outputs | Translated |
| NameHelper.escapeString | names::escape_string | Java UTF-16 controls and surrogate-pair fixture | Source-grounded Commons escapeJava translation; live oracle unrun |
| services/metamodel/services/textual/SysMLv2Keywords.KEYWORDS | names::KEYWORDS | complete keyword iteration | All 117 word constants; operators not exposed |
| services/metamodel/helper/NameConflictingFilter | names::NameConflictingFilter | partial state retention, short circuit, alias identity, fill tests | Translated; alias reference distinction must be supplied explicitly |
| metamodel/impl/ElementImpl.effectiveName/effectiveShortName/escapedName | model::Element | short-name fixture | Base Element behavior only; subclass overrides unported |
| ElementImpl.getQualifiedName | model::Model::qualified_name | unnamed root, membership owner, reserved identifier fixtures | Actual Java implementation projected; not commented specification |
| services/metamodel/util/ElementUtil.nameMatches/equalsConsideringOptionalQuotes | names::name_matches | optional quotes and Java whitespace fixtures | Java Character.isWhitespace, not Rust trim semantics |
| ElementUtil.findByNameAndType and traversal helpers | model::Model::find_by_name_and_type | root, short names, types, imports, shadowing, membership fixtures | Partial ownership projection only (non-Membership relationships must be preflattened); adapter class predicate replaces Java isInstance |
| ElementUtil.generateUUIDv5 | uuid::uuid_v5 | 7 exact upstream ElementUtilTest UUID goldens; Python stdlib long and Unicode UUIDs | Translated deterministic primitive |
| ElementUtil.isStandardLibraryResource | uuid::is_standard_library_resource | prefix/case/null fixtures | Preserves permissive startsWith behavior |
| ElementUtil.generateUUID standard LibraryPackage branch | uuid::standard_library_uuid | upstream Base golden | Partial; resource presence/name checks and random fallback are adapter responsibility |

## Behavioral findings that must survive integration

* `getQualifiedName` uses declared/effective name only, not short-name fallback.
  An unnamed owner stops prefix construction; it does not null the named child.
  The Java source contains a commented specification implementation that is NOT
  the pinned executable behavior.
* `getOwner` is derived through the owning Relationship. The Rust `owner` is that
  semantic projection, not a promise of EMF eContainer equivalence. Membership
  wrapper nodes are optional projection inputs; if supplied, qualified name skips
  a membership owner and lookup flattens one membership wrapper level.
* Qualified lookup picks the first matching owned name before applying the final
  type check. A wrong-type owned match hides same-name later owned/imported
  elements. Unqualified lookup does apply type before accepting each match.
* Public imported membership is a fallback for namespaces only. Unqualified
  lookup walks containment rather than importing external elements.
* The Java name filter compares alias/name object REFERENCES. Adapter callers
  provide `alias_is_distinct`; inferring it from string value equality is wrong.
* Name filter state is incremental. A short-name conflict leaves a previously
  inserted long name reserved. Repeating the same name as long and short fails.
* Qualified splitting is literal `::`, discards trailing empties, and splits
  quoted names containing `::`. This is compatibility, not grammar validation.
* Java strip excludes U+00A0/U+0085/U+2007/U+202F and includes U+001C..U+001F.

## Dependency and integration contract

Native server and future WASM editor may depend on this pure crate by path.
Its public IDs (`ElementId(usize)`) are process-local lookup handles, NOT stable
SysML IDs, persistence IDs, REST IDs, or serialized schema. `Uuid` is a 16-byte
value with lowercase canonical Display. The persistence/API adapters must preserve
upstream stable UUID and null conventions without serializing index handles.
No serde, HTTP framework, browser toolkit, database library, UUID dependency, or
root Cargo workspace is selected by this lane. A later shared contract must pin
these explicitly; this crate does not pick for other lanes.

`Model` is a query projection, not a transactional editing model. Its constructor
requires existing parents, so ownership is acyclic. Missing owners/import targets
produce explicit errors. Insertion is local scaffolding, not a translated mutation
service. Mutable EMF opposite relations, notifications, proxies, resource ordering,
subtyping and subclass-derived names need an adapter or the complete metamodel.
An adapter must provide semantic owned-element order; only membership wrappers
are flattened here. Other Relationship kinds are not currently projected.

## Validation and parity limits

Run with Samsung build outputs and one job:

```
CARGO_TARGET_DIR=${SYSON_BUILD_DIR}/domain CARGO_BUILD_JOBS=1 cargo test --offline --manifest-path rust/crates/syson-domain/Cargo.toml
```

No upstream Java runtime oracle has been run by this lane. Golden assertions
are transcribed source fixtures, not evidence that Java and Rust applications
execute equivalently. No server, HTTP API, browser, screenshot, WASM, database,
full model roundtrip, load/performance, accessibility, authentication, or deployment
acceptance is claimed. See UNPORTED.md for remaining domains.
