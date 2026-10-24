# Inherited source reference audit: OPEN

This directory records read-only public source evidence for the SysON baseline `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`. Public source acquisition was authorized separately from builds. No downloaded script, migration, source generator, application, container or test from the reference projects was executed. No binary dependency artifacts were fetched or resolved by this audit.

## Verified source candidates

| Dependency | Verified public ref | Peeled source commit | Remaining artifact gap |
| --- | --- | --- | --- |
| Sirius Web/Components | `eclipse-sirius/sirius-web` tag `v2026.9.3`; annotated object `2c5f702579366a8a2d509524dc8fa88b10f74374` | `3191e2b8990373afee3415f6b26f8c5039a28321` | Matching source version; no proof that SysON's resolved Maven/npm artifacts were built from these bytes. |
| Sirius EMF JSON | `eclipse-sirius/sirius-emf-json` tag `v2.5.5`; annotated object `e81eb76d8f7c11057d5bd7ce11fb9df872efe8a0` | `1a0282fcfdaebae13ed52b64b6457e8218d9e4b1` | Candidate root POM still declares `2.5.5-SNAPSHOT`. Need actual timestamp/build number, JAR/source-JAR hashes and provenance. |
| AQL | `eclipse-acceleo/acceleo` tag `4.2.2`; annotated object `957f6d748e900be59f939a5866f65c1030bd2a83` | `4feb6200ae70938038503600fcde33953a639cda` | Candidate AQL manifest declares `8.1.2`. Need resolved AQL artifacts/source correspondence. |
| Syside | `sensmetry/sysml-2ls` lightweight tag `0.9.0` | `1326f234cae23b0e4777d5a48a069b17e9e332f1` | Candidate version matches embedded bundle; original source revision, local modifications, build inputs and output reproduction still unproven. |

`verified-source-candidates.jsonl` retains the actual public ref responses and capture time. `candidate-source-documents.jsonl` retains six exact source documents with commit-specific URLs and SHA256: EMF JSON POM, AQL manifest, Syside build/license helper scripts, CLI package declaration and root license. These are source evidence, not artifact attestation. A tag and a POM version are distinct claims.

The Sirius source checkout is detached at the exact candidate commit. The reference path was verified vacant before cloning with `--filter=blob:none --no-checkout --single-branch --branch v2026.9.3`; tracked `AGENTS.md`, `.agents` and skills paths were checked before checkout, with none found. The reference checkout remains clean. Its location is an execution input, not a public-document dependency.

## Sirius declaration inventory

`inventory_sirius.py --reference "${SIRIUS_REFERENCE}"` reads an existing, clean exact-commit Git reference and writes this directory. It uses Git blob reads and Python standard-library parsing only. It does not fetch, install or execute dependency code. It deliberately refuses an unexpected reference commit or a modified checkout.

| Artifact | Source evidence and limit |
| --- | --- |
| `sirius-source-files.jsonl` | Every one of the 10,683 tracked source locations, byte length and SHA256. Hashes do not prove runtime inclusion or behavior. |
| `sirius-modules.jsonl` | 188 Maven/npm manifest locations. They include examples, tests and tools; runtime dependency closure remains open. |
| `sirius-graphql-documents.jsonl` | All 43 tracked GraphQL SDL documents, exact source text/hashes. Schema composition, resolver wiring, nullability/error semantics, imported/generated declarations and actual deployment inclusion remain open. |
| `sirius-source-surfaces.jsonl` | 1,235 source-file candidates for GraphQL, HTTP/REST, serializer/deserializer and editing-context/collaboration behavior. Categories and HTTP annotation lines are search aids, not a complete parser or resolved routing map. |
| `sirius-tests.jsonl` | 897 tracked test-source/resource/spec locations. None run; number of files is not coverage or number of test cases. |
| `sirius-notice-documents.jsonl` | Supplied tracked license/notice texts and hashes. Preserve original notices; transitive library obligations remain unassessed. |
| `sirius-liquibase-*.jsonl` | Exact recursively reached migration XML, include edges/order and changeSet subtrees. This is declaration closure, not an applied DB schema or successful migration. |

## Persistence declaration closure

Master: `packages/sirius-web/backend/sirius-web-infrastructure/src/main/resources/db/db.changelog-master.xml` at Sirius commit `3191e2b8990373afee3415f6b26f8c5039a28321`.

Its version roots occur in this order: `2024.3`, `2024.5`, `2024.11`, `2025.1`, `2025.2`, `2025.4`, `2025.10`, `2026.1`. The recursion contains 29 XML documents (master, eight version roots and 20 changeSet documents), 28 include edges and 20 changeSets. No local include is unresolved in this pinned graph. Original IDs/authors, contexts/attributes, SQL, constraints, table/column declarations and ordering are retained as exact XML and parsed trees. Nothing was applied to a database.

The initial schema includes PostgreSQL `pgcrypto` extension creation and UUID defaults; that is source evidence only and does not authorize extension installation. Later declarations split representation metadata/content, move project-to-semantic-data relations, add library support and make representation identity semantic-data aware. Evaluated preconditions, DBMS differences, Liquibase checksums/history, transactional effects, rollback/recovery, current DB state, application serialization and SysON migration hooks still need runtime evidence. There is no schema-compatibility PASS.

## API and serializer review entry points

All paths below are relative to the pinned Sirius reference and are indexed by hash:

- REST controllers: `packages/sirius-web/backend/sirius-web-application/src/main/java/org/eclipse/sirius/web/application/project/controllers/ProjectRestController.java`, `project/data/versioning/controllers/CommitRestController.java`, `project/data/versioning/controllers/BranchRestController.java`, and `object/controllers/ObjectRestController.java`. Download controllers are separate; Spring injection, delegates, status/error mapping and SysON overrides require reconciliation.
- REST serialization: the same application module's `object/services/EObjectRestSerializer.java`, `EObjectRestDeserializer.java` and corresponding API interfaces. EMF JSON resource serialization is a separate source/artifact dependency; serializer name matching does not close it.
- GraphQL schemas: application `src/main/resources/schema/` includes project/document/library/query, capabilities, undo/redo, custom-node, search, related-view and representation surfaces; component package schemas add diagram/forms/table and other widgets. The full 43 exact documents are retained, including potential modules not deployed by SysON.
- Tests: `packages/sirius-web/backend/sirius-web/src/test/java/org/eclipse/sirius/web/application/controllers/projects/{ProjectRestControllerIntegrationTests,BranchRestControllerIntegrationTests,CommitRestControllerIntegrationTests}.java`, `.../objects/ObjectRestControllerIntegrationTests.java`, `.../rest/RestControllerIntegrationTests.java`, `.../services/undoredo/UndoRedoSemanticChangeTests.java`, and `.../controllers/diagrams/undo/`. Paths/tests are available evidence candidates; all executions remain NOT RUN.

A material source distinction: `DefaultProjectDataVersiongRestService.java` lines 92-158 documents a single current-state commit, then `createCommit` applies an amend operation by deleting objects, creating objects by `@type`, deserializing changes and attaching unattached new roots, returning a project-ID commit. Thus SysON documentation about no *additional* commits does not imply the mutation endpoint does nothing. Candidate `createBranch` returns null; default branch/commit lists synthesize one entry. SysON delegates and overrides must be combined with this source and the actual resolved artifact before asserting behavior. The existing source-map boundary document must be read with this refinement; no executable oracle was run.

## AQL and Syside provenance refinement

Sirius `packages/emf/backend/sirius-components-emf/pom.xml` references `sirius-components-interpreter:2026.9.3`; that interpreter's POM references `org.eclipse.acceleo:aql:8.1.2`. The Acceleo candidate's `query/plugins/org.eclipse.acceleo.query/META-INF/MANIFEST.MF` independently declares Bundle-Version `8.1.2`. This verifies a plausible source chain, not jar identity, full dependency closure or AQL query behavior.

The last change to SysON's tracked embedded `syside-cli.js` before the pinned baseline is public commit `1c531d7d12aaddb64988cd47f28487a9208c685f`, subject `[1385] Update SysIDE dependency to 0.9.0`. `syside-baseline-history.json` records that path-history observation. It does not establish a reproducible build relationship.

The candidate Syside `scripts/build.mjs` defines `LicenseBundler`, runs a license collector over esbuild metafile inputs, calls `getLicenseText(dependencies)` and writes a separate `LICENSE` beside the output. Therefore the absence of notice markers inside the JS bundle is compatible with separate notice generation; it cannot establish that the matching generated notice was included in SysON. The retained script header declares `EPL-2.0 OR GPL-2.0 WITH Classpath-exception-2.0`; this source declaration is not blanket clearance for every bundled dependency. The exact matching generated bundle license, dependency inputs and source modifications remain OPEN. Do not execute the build script to investigate without a separate approved build scope.

## Remaining acceptance gates

1. Resolve and verify actual SysON Maven/npm artifact bytes, SNAPSHOT metadata and matching source provenance.
2. Reconcile inherited surfaces with SysON overrides, deployment module inclusion and generated/third-party schemas; source search candidates are not a runtime call graph.
3. Recover exact matching Syside generated notices and original build/source inputs.
4. Build an approved reference oracle and execute mapped positive/error/cancellation/recovery/API/data/UI tests. Declaration extraction checks cannot substitute for that evidence.

Full dependency source, artifact and behavior closure all remain OPEN.

## Observed extraction checks

Six offline source checks passed: exact reference commit/clean state, complete tracked path enumeration, all SDL byte preservation, recursive migration include order and full changeSet subtree fidelity, unproven-artifact source-candidate distinctions, and separate Syside license-generation evidence with explicit unrun tests/migrations. Run `PYTHONDONTWRITEBYTECODE=1 python3 porting/source-map/inherited/test_inherited_audit.py --reference "${SIRIUS_REFERENCE}"` against the same exact clean source reference. These six source checks are not product tests or proof of feature coverage.
