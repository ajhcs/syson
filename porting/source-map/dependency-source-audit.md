# Dependency source audit: OPEN

Baseline is `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`. This audit reads tracked source and lockfiles only. No dependency download, upstream script, parser/runtime CLI, container, build or service was executed. Counts below identify declaration locations and extraction evidence; they are neither unique runtime dependency counts nor test coverage.

## Declaration preservation

The improved `metamodel.jsonl` retains each full classifier declaration tree, including nested `eParameters`, `eLiterals`, `eAnnotations`/details, structural-feature metadata and operation content. Structural source locations use classifier and child ordinals with `@` containment segments. `ecore-documents.jsonl` also retains exact raw XML, hashes and entire parsed package roots: package namespace URIs/prefixes, root attributes and annotations no longer depend on a classifier-only projection. Raw XML preserves comments, formatting and prefix spellings that ElementTree's parsed model does not preserve. External Ecore reference resolution and executable semantic behavior remain open.

Four `.graphqls` documents are retained byte-exact in `graphql-documents.jsonl`. All 15 local frontend files containing `gql` tagged-template candidates are retained in full in `graphql-client-documents.jsonl`, with source hashes and candidate text. Whole-source retention preserves fragments, arguments, defaults, nullability notation and interpolated expressions even if candidate extraction is incomplete. This audit did not find an existing GraphQL parser in the checked Python import locations or common project/system module directories; no parser was installed. The candidate scanner is not a TypeScript/GraphQL parser. Actual semantic parsing, fragment/type resolution, imported documents, directive handling and inherited Sirius schema remain OPEN. `graphql-types.jsonl` and `graphql-operations.jsonl` remain syntactic search indexes only.

## Sirius/EMF source inputs

Root `pom.xml` declares `sirius.web.version=2026.9.3` and `sirius.emfjson.version=2.5.5-SNAPSHOT`. Maven repository declarations point to:

- `https://repo.maven.apache.org/maven2`
- `https://maven.pkg.github.com/eclipse-sirius/sirius-web`
- `https://maven.pkg.github.com/eclipse-sirius/sirius-emf-json`

`inherited-maven-inputs.jsonl` captures all 67 Sirius/EMF dependency declaration locations and expands locally available property expressions. It does not compute an effective Maven POM, dependency management, imported BOMs, profiles, classifiers, relocation, transitives or artifact resolution. A null expanded version is unresolved; it is not versionless or unnecessary. Declarations in test/plugin/managed contexts are not assumed to be runtime dependencies. The inherited source repository is not pinned by a Maven version property alone.

The EMF JSON SNAPSHOT is particularly unresolved: no timestamp/build number, artifact SHA, source-jar hash or source Git SHA is established. A later resolution of the same property can supply different bytes. Reference build closure requires preserving resolved artifact coordinates and checksums plus matching source inputs, not changing the version to a guess.

`npm-locked-inputs.jsonl` retains every package location from tracked npm v3 lockfiles, with declared version, resolved artifact URL, integrity and license metadata when present. There are 1,056 locked locations, including 22 `@eclipse-sirius/` package locations at declared version `2026.9.3`. These are lockfile claims, not independently fetched/verified artifacts. URL path hexadecimal identifiers are artifact registry identifiers; do not label them source Git commits. Lockfile integrity pins an expected archive, not the matching source revision or build instructions. Linked local packages have different metadata from fetched packages and remain explicitly marked as links.

The root lock pins `graphql` to `16.8.1` with npm integrity metadata, but the source audit does not install it. Layout/UI packages and their source mapping remain dependent-source audit items. Backend and frontend Sirius version equality does not establish byte-level or source-level closure.

## Embedded Syside provenance

`backend/application/syson-sysml-import/src/main/resources/syside-cli.js` is pinned by its Git blob and SHA256 in `syside-provenance.json`. It declares:

- CLI package `syside-cli`, version `0.9.0`, publisher/author Sensmetry, in the embedded package block at lines 113992-114044.
- Language-server version `0.9.0` at lines 113565-113572.
- Embedded repository URL `https://gitlab.com/sensmetry/public/sysml-2ls`, directory `packages/syside-cli`. This is the bundle's declared URL, not a verified current repository, source SHA or source availability statement.
- Source-map reference `index.js.map`; no corresponding tracked source map was found at the bundle location.

`syside-bundle-modules.jsonl` records 811 source-label locations and `syside-bundled-package-labels.jsonl` groups 23 pnpm package labels, including parser/LSP/CLI dependencies. These labels expose embedded source layout and version strings but are not an authoritative lockfile, source archive or transitive license manifest. They may omit inlined or unlabeled modules.

The bundle contains **zero** case-insensitive `copyright`, `license`/`licence`, or `SPDX` marker matches. This observation is limited to those literal searches; it does not prove that permissions are absent, that the code is unlicensed, or that no obligations apply. Root project EPL/LGPL declarations do not by themselves resolve every embedded dependency's obligations. Preserve bundled bytes and root notices. Obtain matching original source, build provenance and applicable complete notices before claiming distribution/source closure. Do not infer licenses from package names or substitute current releases for the bundled version.

## Exact next source-closure actions

1. In an approved dependency acquisition lane, resolve the pinned baseline Maven graph and SNAPSHOT with recorded source/artifact checksums and source repository revisions. Retain all effective profiles, BOMs and plugin/tool inputs separately from runtime dependencies.
2. Acquire the lockfile-identified Sirius npm artifacts, verify integrity, recover matching source/repository commits and preserve applicable dependency notices. Parse inherited schemas using the exact matching source/artifact inputs.
3. Identify the source revision and build inputs corresponding to the embedded Syside 0.9.0 bytes, including any local changes and generated artifacts. Recover complete bundled third-party notices; do not run the CLI as part of provenance inspection.
4. Run an existing approved GraphQL/TypeScript parser against the retained local documents, resolve interpolation/fragments and imported schema declarations, then check field/argument/nullability/error behavior against an approved oracle. No semantic closure is asserted until that evidence exists.
5. Keep feature/runtime parity OPEN independently of source extraction checks. Ecore declaration fidelity and lockfile hashes do not implement modeling semantics, authentication, persistence, UI interaction, cancellation or recovery.

## Observed extraction validation

Eight standard-library source-preservation tests passed. They compare exact Ecore XML and all classifier subtree tags/attributes/text/tails against pinned Git blobs, check ordinal containment references, preserve GraphQL whole-source bytes, reconcile every locked package location's version/resolution/integrity, and verify unresolved source/SNAPSHOT distinctions. The classifier trees contain 57 operation parameters, 19 enum literals and 309 annotations; there are no annotation `details` children in this baseline. The extraction tests preserve absence as well as presence. These eight checks are not application tests, a dependency-license clearance, a runtime oracle or a port-completion gate.
