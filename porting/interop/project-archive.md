# SysON project download archive

[Issue #271](https://github.com/ajhcs/syson/issues/271) parts 1 and 2 record
what a project download contains and where the format is implemented. This
fixture captures one synthetic project from the official Eclipse SysON
v2026.9.3 release. Rust archive support and a round trip through SysON project
upload remain open.

## Capture the same project

1. Start the isolated reference using the [oracle runbook](../oracle/README.md).
   The capture used the release JAR SHA-256
   `85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1`
   from the [published release image](../audits/20261026/native-reference.json).
   The image labels identify source revision
   `b380dada415ce05155150f23cb3b74ae21cc45e4`.
2. Create a blank project named `ArchiveDemo` with no selected libraries and
   import `ArchiveDemoSource.sysml` as an editable document. The capture used
   the same GraphQL create and document-upload paths as the reference harness.

   ```sysml
   package ArchiveDemo {
       part def Wheel;
       part wheel : Wheel;
   }
   ```

3. Open the project editor. Show root namespaces in the Explorer, expand
   the imported document and choose **New representation** from the menu on
   its root **Namespace**. Name it **Archive overview**, select **General View**
   and click **Create**. Drag the **ArchiveDemo** package from the Explorer
   onto the diagram.
4. With the diagram open, choose **Download** from the project toolbar menu.
   The browser received HTTP 200 from `GET /api/projects/{projectId}` with
   an `application/zip` attachment named `ArchiveDemo.zip`. The committed fixture is the raw browser
   download with a descriptive repository filename. SysON generates IDs and
   ZIP timestamps, so a repeat capture has different bytes.

The model text was written for this fixture. The generated archive has no
imported user data. The project, document, representation and model IDs are
from the isolated synthetic run. See the repository [license](../../LICENSE).

## Every ZIP entry

[Raw browser download](fixtures/archive-demo.syson.zip) is 3,210 bytes.
Its SHA-256 is `96846d20a44dee2cf87e3643a7c6804ce0db0a13731aee013a1abaf40ab28012`.

| Entry | Uncompressed bytes | CRC-32 | Contents |
| --- | ---: | --- | --- |
| `ArchiveDemo/representations/0fede535-5237-4f7e-b03d-d1087431c764.json` | 2837 | `07829baf` | The `Archive overview` General View. Contains its diagram kind and description, one package node, label and position data, and no edges. |
| `ArchiveDemo/documents/f7819cef-db84-4b86-959c-c3bae4f13353.json` | 2504 | `c91689b1` | EMF JSON for `ArchiveDemoSource.sysml`. Contains the imported Namespace, the package and typed part, and the ViewUsage created by the editor. |
| `ArchiveDemo/manifest.json` | 711 | `9f03a381` | Maps document UUIDs to original filenames and representation UUIDs to their kind, target URI, description URI and migration metadata. Lists metamodels, project natures and dependencies. |

The manifest's `documentIdsToName` maps the document filename UUID to
`ArchiveDemoSource.sysml`. The representation's `targetObjectURI` has the
form `sirius:///<document UUID>#<ViewUsage object ID>`. It resolves to the
new `Archive overview` ViewUsage inside that document. The rendered node
has a separate layout ID and points to the package's EMF object `id`.
Those IDs differ from the model elements' `elementId` values.

Both `natures` and `dependencies` are empty. The ViewUsage still has a type
reference to a bundled SysML `ViewDefinition` through a `sysmllibrary:///`
URI. The archive carries the reference and the SysML metamodel URI. Its three
entries contain no library or metamodel files.

To verify the captured bytes from the repository root, run these commands
with Python 3.

```sh
python3 -m zipfile -t porting/interop/fixtures/archive-demo.syson.zip
python3 -m zipfile -l porting/interop/fixtures/archive-demo.syson.zip
sha256sum porting/interop/fixtures/archive-demo.syson.zip
```

## Pinned reader and writer source

SysON's [release POM](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/pom.xml#L33-L37)
declares Sirius Web `2026.9.3`. The release JAR includes
`sirius-web-application-2026.9.3.jar`. The links below use the matching
[v2026.9.3 Sirius source commit](https://github.com/eclipse-sirius/sirius-web/tree/3191e2b8990373afee3415f6b26f8c5039a28321).
That version match identifies the source to inspect. Artifact-to-source
reproduction is still open in the [inherited source audit](../source-map/inherited/README.md).

| Path | Behavior |
| --- | --- |
| SysON General View creation | Creates a ViewUsage for the new representation and assigns its type. [CreateRepresentationInputProcessor](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/views/syson-tree-explorer-view/src/main/java/org/eclipse/syson/tree/explorer/view/menu/context/CreateRepresentationInputProcessor.java#L64-L93), [ViewUsage typing](https://github.com/eclipse-syson/syson/blob/b380dada415ce05155150f23cb3b74ae21cc45e4/backend/views/syson-tree-explorer-view/src/main/java/org/eclipse/syson/tree/explorer/view/menu/context/CreateRepresentationInputProcessor.java#L135-L148) |
| Editor download action | Builds the download URL from the project ID and includes the optional name when supplied. [DownloadProjectMenuItem](https://github.com/eclipse-sirius/sirius-web/blob/3191e2b8990373afee3415f6b26f8c5039a28321/packages/sirius-web/frontend/sirius-web-application/src/views/edit-project/navbar/context-menu/DownloadProjectMenuItem.tsx#L23-L35) |
| Download endpoint | Resolves the editing context, checks download capability and sends an `application/zip` attachment. [ProjectDownloadController](https://github.com/eclipse-sirius/sirius-web/blob/3191e2b8990373afee3415f6b26f8c5039a28321/packages/sirius-web/backend/sirius-web-application/src/main/java/org/eclipse/sirius/web/application/project/controllers/ProjectDownloadController.java#L69-L105) |
| ZIP orchestration and manifest | Collects participant output and writes the manifest under the project name. [ProjectExportService](https://github.com/eclipse-sirius/sirius-web/blob/3191e2b8990373afee3415f6b26f8c5039a28321/packages/sirius-web/backend/sirius-web-application/src/main/java/org/eclipse/sirius/web/application/project/services/ProjectExportService.java#L55-L91) |
| Semantic model and dependency writer | Writes persistable model resources as EMF JSON, records document names, natures and metamodel URIs, and lists project dependencies. [ProjectSemanticDataExportParticipant](https://github.com/eclipse-sirius/sirius-web/blob/3191e2b8990373afee3415f6b26f8c5039a28321/packages/sirius-web/backend/sirius-web-application/src/main/java/org/eclipse/sirius/web/application/project/services/ProjectSemanticDataExportParticipant.java#L87-L146), [ProjectDependencyDataExportParticipant](https://github.com/eclipse-sirius/sirius-web/blob/3191e2b8990373afee3415f6b26f8c5039a28321/packages/sirius-web/backend/sirius-web-application/src/main/java/org/eclipse/sirius/web/application/project/services/ProjectDependencyDataExportParticipant.java#L42-L56) |
| Representation writer | Writes representation payloads plus target and description URIs, and migration metadata. [ProjectRepresentationDataExportParticipant](https://github.com/eclipse-sirius/sirius-web/blob/3191e2b8990373afee3415f6b26f8c5039a28321/packages/sirius-web/backend/sirius-web-application/src/main/java/org/eclipse/sirius/web/application/project/services/ProjectRepresentationDataExportParticipant.java#L95-L163) |
| Archive upload and validation | Reads ZIP entries and the manifest, checks that listed dependency IDs exist, then initializes a new project. [MutationUploadProjectDataFetcher](https://github.com/eclipse-sirius/sirius-web/blob/3191e2b8990373afee3415f6b26f8c5039a28321/packages/sirius-web/backend/sirius-web-application/src/main/java/org/eclipse/sirius/web/application/project/controllers/MutationUploadProjectDataFetcher.java#L32-L67), [ProjectUploadApplicationService](https://github.com/eclipse-sirius/sirius-web/blob/3191e2b8990373afee3415f6b26f8c5039a28321/packages/sirius-web/backend/sirius-web-application/src/main/java/org/eclipse/sirius/web/application/project/services/ProjectUploadApplicationService.java#L66-L101), [ProjectZipContentProvider](https://github.com/eclipse-sirius/sirius-web/blob/3191e2b8990373afee3415f6b26f8c5039a28321/packages/sirius-web/backend/sirius-web-application/src/main/java/org/eclipse/sirius/web/application/project/services/ProjectZipContentProvider.java#L54-L126), [ProjectZipContentValidator](https://github.com/eclipse-sirius/sirius-web/blob/3191e2b8990373afee3415f6b26f8c5039a28321/packages/sirius-web/backend/sirius-web-application/src/main/java/org/eclipse/sirius/web/application/project/services/ProjectZipContentValidator.java#L47-L71) |
| Semantic and representation readers | Loads model resources, maps document and semantic IDs and recreates representations using that mapping. [SemanticDataProjectContentImportParticipant](https://github.com/eclipse-sirius/sirius-web/blob/3191e2b8990373afee3415f6b26f8c5039a28321/packages/sirius-web/backend/sirius-web-application/src/main/java/org/eclipse/sirius/web/application/project/services/SemanticDataProjectContentImportParticipant.java#L134-L212), [RepresentationProjectContentImportParticipant](https://github.com/eclipse-sirius/sirius-web/blob/3191e2b8990373afee3415f6b26f8c5039a28321/packages/sirius-web/backend/sirius-web-application/src/main/java/org/eclipse/sirius/web/application/project/services/RepresentationProjectContentImportParticipant.java#L86-L216) |
| Diagram content reader | Dispatches diagram content updates for the newly created representation. [DiagramImporterUpdateService](https://github.com/eclipse-sirius/sirius-web/blob/3191e2b8990373afee3415f6b26f8c5039a28321/packages/sirius-web/backend/sirius-web-application/src/main/java/org/eclipse/sirius/web/application/project/services/DiagramImporterUpdateService.java#L111-L153) |

Keep the archive entry bytes and metadata available for any future Rust
adapter. Start the write and upload work after the wire-format dependencies in
[#248](https://github.com/ajhcs/syson/issues/248),
[#249](https://github.com/ajhcs/syson/issues/249),
[#250](https://github.com/ajhcs/syson/issues/250) and
[#251](https://github.com/ajhcs/syson/issues/251) are resolved. This fixture
covers a single document and one General View. Multi-document projects,
library-linked projects and other representation types need separate
captures.
