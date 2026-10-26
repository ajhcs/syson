# Remaining Rust parity PR index

The counts below are review units, not completed features. Prerequisites here describe source preparation; each linked brief and plan.json separately defines implementation readiness, child acceptance and joint gates. Read [execution and acceptance](README.md) before dispatch.

| ID | PR scope | Source prerequisites | Existing PRs |
|---|---|---|---|
| [B01](prs/B01.md) | Versioned standard-library semantic and diagram migration preparation |  | #64 |
| [B02](prs/B02.md) | Rust HTTP host, request lifecycle and wire error envelope | C01 |  |
| [B03](prs/B03.md) | GraphQL root execution, coercion and dispatch foundation | B02, C01 |  |
| [B04](prs/B04.md) | Session, CSRF, CORS and WebSocket admission | B02, C01 |  |
| [B05](prs/B05.md) | Effective PostgreSQL schema and Rust connection foundation | C01 |  |
| [B06](prs/B06.md) | Atomic semantic, representation and migration persistence | B01, B05 |  |
| [B07](prs/B07.md) | Rust local-store replacement and SQLite import | B06 |  |
| [B08](prs/B08.md) | Project list, create, rename and delete service | B03, B06 |  |
| [B09](prs/B09.md) | REST project and element read endpoints | B02, B06 |  |
| [B10](prs/B10.md) | REST branch and commit versioning | B09 |  |
| [B11](prs/B11.md) | Object/document/project download archives | B09 |  |
| [B12](prs/B12.md) | Project/document archive upload and import | B03, B06, B11 |  |
| [B13](prs/B13.md) | Semantic command dispatcher and transaction boundary | B03, B06 |  |
| [B14](prs/B14.md) | Textual expression and SysML insertion endpoints | B13 |  |
| [B15](prs/B15.md) | Editing-context actor lifecycle and collaboration ordering | B13, B04 |  |
| [B16](prs/B16.md) | GraphQL WebSocket protocol and subscriptions | B15, B04 |  |
| [B17](prs/B17.md) | Semantic history, canUndo/canRedo and durable revision boundaries | B15 |  |
| [B18](prs/B18.md) | Diagram layout history and cross-view undo events | B17 |  |
| [B19](prs/B19.md) | Explorer, selection dialog and object search services | B16 |  |
| [B20](prs/B20.md) | Diagram runtime query, mutation and refresh transport | B16 |  |
| [B21](prs/B21.md) | Forms, details and widget interaction services | B16 |  |
| [B22](prs/B22.md) | Requirements table, table/grid and validation subscriptions | B16 |  |
| [B23](prs/B23.md) | Template catalog and project instantiation | B08, B12, C01h |  |
| [B24](prs/B24.md) | Library publication, versions, update and impact analysis | B08, B15 |  |
| [B25S](prs/B25S.md) | Shared bounded JSON migration helper and namespace-safe composition | B01 | #70 |
| [B25a](prs/B25a.md) | Annotation annotating-element migration | B01, B25S |  |
| [B25b](prs/B25b.md) | Connector source/target migration | B01, B25S |  |
| [B25c](prs/B25c.md) | ViewUsage exposed-element migration | B01, B25S |  |
| [B25d](prs/B25d.md) | Action control start/done migration | B01, B25S |  |
| [B26a](prs/B26a.md) | Diagram-on-ViewUsage migration and hook | B25a, B25b, B25c, B25d, B20 |  |
| [B26b](prs/B26b.md) | Merged diagram-description migration and hook | B25a, B25b, B25c, B25d, B20 |  |
| [B27](prs/B27.md) | Project snapshot, backup and restore format | B06, B11 |  |
| [B28](prs/B28.md) | Crash/restart and failure recovery qualification | B15, B27 |  |
| [B29](prs/B29.md) | Extension/provider invocation and module composition | B03, B15 |  |
| [B30](prs/B30.md) | Images, localization and active ancillary HTTP APIs | B02, B04 |  |
| [B31](prs/B31.md) | Connected project author/save/close/reopen workflow | B08, B11, B12, B17, B20, B21, B24, B27 |  |
| [B32](prs/B32.md) | Rust native/WASM builds, packaging and process closure | C03, B07, B31 |  |
| [B33](prs/B33.md) | Notices, source distribution, upgrade and rollback | B32, B27 |  |
| [B34](prs/B34.md) | Strict seed and Vehicle runtime differentials | C02, B31, B32 |  |
| [B35](prs/B35.md) | Inherited API and active-module closure gate | C01, B03, B34, B36, B37, B38, B39, B40, B41 |  |
| [B36](prs/B36.md) | Model browser representation API | B16, B29 |  |
| [B37](prs/B37.md) | Charts and hierarchy representation API | B16, B29 |  |
| [B38](prs/B38.md) | Deck representation API | B16, B29 |  |
| [B39](prs/B39.md) | Portal representation API | B16, B29 |  |
| [B40](prs/B40.md) | Form description editor API | B16, B29 |  |
| [B41](prs/B41.md) | Studio/view fork and representation descriptions | B16, B29 |  |
| [C01](prs/C01.md) | Close pinned source, artifact and effective capability inventory |  | #39, #66 |
| [C01a](prs/C01a.md) | Pure fail-closed selected-reference evidence policy |  | #66 |
| [C01b](prs/C01b.md) | Read-only file and Git inspector for selected-reference policy | C01a | #76 |
| [C01c](prs/C01c.md) | Capture and validate the actual native GraphQL introspection contract |  | #66 |
| [C01d](prs/C01d.md) | Capture the native PostgreSQL catalog and applied migration contract |  | #66 |
| [C01e](prs/C01e.md) | Build a selected-source-delta comparison artifact with exact replacement planning | C01b | #78 |
| [C01f](prs/C01f.md) | Capture actual native HTTP handler and wire-observation contracts | C01c | #80 |
| [C01g](prs/C01g.md) | Index actual standard-library payload and native identity contracts | C01e | #84 |
| [C01h](prs/C01h.md) | Capture native project-template discovery and typed choices | C01c, C01f | #80, #83 |
| [C02](prs/C02.md) | Build reproducible differential and connected-workflow harnesses | C01 | #41 |
| [C02a](prs/C02a.md) | Compare complete JSON trees without hiding identity or ordering differences | C01c, C01d, C01f | #79, #80, #83 |
| [C03](prs/C03.md) | Compose reviewed branches and serialize integration ownership |  | #22, #66 |
| [C03a](prs/C03a.md) | Verify explicit committed source-blob compositions | B25S, B25a, B25b, B25c | #71, #72, #73, #74 |
| [C03b](prs/C03b.md) | Compose private project migration preparation through actual callers | B01, B25S, B25a, B25b, B25c, B25d, C03a | #70, #71, #72, #73, #74, #75, #77 |
| [C03c](prs/C03c.md) | Losslessly transport authoritative prepared-project wire payloads | C03b | #81 |
| [C04](prs/C04.md) | Verify Rust production artifact and process closure | C01, C02 |  |
| [C05](prs/C05.md) | Compare AST, CST and diagnostic observations | C01, C02 |  |
| [C06](prs/C06.md) | Compare model identities, references and query outputs | C01, C02 |  |
| [C07](prs/C07.md) | Compare API, database and migration observations | C01, C02 |  |
| [C08](prs/C08.md) | Compare geometry and browser task observations | C01, C02 |  |
| [C09](prs/C09.md) | Run isolated differential runtime lifecycle | C01, C02 |  |
| [C99](prs/C99.md) | Accept complete matched SysON parity and Rust release closure | C01c, C01d, C03b, C01e, C01f, C03c, C01g, C01h, C02a, C01a, C01b, C03a, B25S, I01, M01, M02, M03, M04, M05, M06, M07, M08, M09, M10, M11, M12, M13, M14, M15, M16, M17, M18, M19, M20, M21, M22, M23, M24, M25, M26, M27, M28, M29, M30, M31, M32, M33, M34, M35, M36, M37, M38, M39, M40, I02, I03, I04, I05, I06, I07, I08, I09, M41, M42, M43, M44, M45, M46, M47, M48, M50, M51, M52, M53, M54, M55, M56, M57, M58, M59, M60, M61, M62, M63, M64, M65, M66, M67, M68, M69, M70, M71, M72, M73, M74, M75, M76, M77, M78, M79, M80, M81, M82, M83, M84, M85, M86, M87, M88, M89, M90, M91, M92, M93, M94, M95, M96, M97, M98, M99, M100, M101, M102, M103, M104, M105, M106, M107, M108, M109, M110, M111, M112, M113, M114, M115, M116, M117, M118, M119, M120, M121, M122, M123, M124, M125, M126, M127, M128, M129, M130, M131, M132, M133, M134, M135, M136, M137, M138, M139, M140, M141, M142, M143, M144, M145, M146, M147, M148, M149, M150, M151, M152, M153, M154, M155, M156, M157, M158, M159, M160, M161, M162, M163, M164, M165, M166, M167, M168, M169, M170, M171, M172, M173, M174, M175, M176, M177, M178, M179, M180, M181, M182, M183, M184, M185, M186, M187, M188, M189, M190, M191, M192, M193, M194, M195, M196, M197, M198, M199, M200, M201, M202, M203, M204, M205, M206, M207, M208, M209, M210, M211, M212, M213, M214, M215, M216, M217, M218, M219, M220, M221, M222, M223, M224, M225, M226, M227, M228, M229, M230, M231, M232, M233, M234, M235, M236, M237, M238, M239, M240, M241, M242, M243, M244, M245, M246, M247, M248, M249, M250, M251, M252, M253, M254, M255, M256, M257, M258, M259, M260, M261, M262, M263, M264, M265, M266, M267, M268, M269, M270, M271, M272, M273, M274, M275, M276, M277, M278, M279, M280, M281, M282, M283, M284, M285, M286, M287, M288, M289, M290, M291, M292, M293, M294, M295, M296, M297, M298, M299, M300, M301, M302, M303, M304, M305, M306, M307, M308, M309, M310, M311, M312, M313, M314, M315, M316, M317, M318, M319, M320, M321, M322, M323, M324, M325, M326, M327, M328, M329, M330, M331, M332, M333, M334, M335, M336, M337, M338, M339, M340, M341, M342, M343, M344, M345, M346, M347, M348, M349, M350, M351, M352, M353, M354, M355, M356, M357, M358, M359, M360, M361, M362, M363, M364, M365, M366, M367, M368, M369, M370, M371, M372, M373, M374, M375, M376, M377, M378, M379, M380, M381, M382, M383, M384, M385, M386, M387, M388, M389, M390, M391, M392, M393, M394, M395, M396, M397, M398, M399, M400, P01, P02, P03, P04, P05, P06, P07, P08, P09, P10, P11, P12, P13, P14, P15, P16, P17, P18, P19, P20, P21, P22, P23, P24, P25, P26, P27, P28, P29, P30, P31, P32, P33, P34, P35, P36, P37, P38, P39, P40, P41, P42, P43, P44, P45, P46, P47, P48, P49, P50, P51, P52, P53, P54, P55, P56, P57, P58, P59, P60, P61, P62, P63, P64, P65, P66, P67, P68, P69, P70, P71, P72, P73, P74, P75, P76, P77, P78, P79, P80, P81, P82, P83, P84, P85, P86, P87, P88, P89, P90, P91, P92, P93, P94, P95, P96, P97, P98, P99, P100, P101, P102, P103, P104, P105, P106, P107, P108, P109, P110, P111, P112, P113, P114, P115, P116, P117, P118, P119, P120, P121, P122, P123, P124, P125, P126, P127, P128, P129, P130, P131, P132, P133, P134, P135, P136, P137, P138, P139, P140, P141, P142, P143, P144, P145, P146, P147, P148, P149, P150, P151, P152, P153, P154, P155, P156, P157, P158, P159, P160, P161, P162, P163, P164, P165, P166, P167, P168, P169, P170, P171, P172, P173, P174, P175, P176, P177, P178, P179, P180, P181, P182, P183, P184, B01, B02, B03, B04, B05, B06, B07, B08, B09, B10, B11, B12, B13, B14, B15, B16, B17, B18, B19, B20, B21, B22, B23, B24, B25a, B25b, B25c, B25d, B26a, B26b, B27, B28, B29, B30, B31, B32, B33, B34, B35, B36, B37, B38, B39, B40, B41, C01, C02, C03, C04, C05, C06, C07, C08, C09, U01, U02, U03, U04, U05, U06, U07, U08, U09, U10, U11, U12, U13, U14, U15, U16, U17, U18, U19, U20, U21, U22, U23, U24, U25, U26, U27, U28, U29, V001, V002, V003, V004, V005, V006, V007, V008, V009, V010, V011, V012, V013, V014, V015, V016, V017, V018, V019, V020, V021, V022, V023, V024, V025, V026, V027, V028, V029, V030, V031, V032, V033, V034, V035, V036, V037, V038, V039, V040, V041, V042, V043, V044 | #66 |
| [I01](prs/I01.md) | Reverse the finite local node-ID bridge into editable semantic-ID documents |  | #63 |
| [I02](prs/I02.md) | Extend explicit identity bridges to port/view and external library typing | I01, M07, M08, M15 | #55, #63 |
| [I03](prs/I03.md) | Generate full native EMF JSON feature/schema adapters | M01, M04, M05, M06, M08 |  |
| [I04](prs/I04.md) | Qualify identity-service reference serialization policy | I03, M06, M15 |  |
| [I05](prs/I05.md) | Implement capability-scoped external resource loading | M08, M09, M15 |  |
| [I06](prs/I06.md) | Close native-edit-save-reload application journeys for every schema family | I03, I04, M38, M39 |  |
| [I07](prs/I07.md) | Port native library/reference/document identity migrations | I04, M06, M39 |  |
| [I08](prs/I08.md) | Integrate text-linking and external resource references across entry points | M09, M14, I04, M38 |  |
| [I09](prs/I09.md) | Qualify large model/library resource bounds and streaming import | I03, M07, M15, M21 |  |
| [M01](prs/M01.md) | Freeze complete model schema and effective Java behavior assignment | C01 |  |
| [M02](prs/M02.md) | Independently qualify held raw factory and admission candidates | C01, C02 | #1 |
| [M03](prs/M03.md) | Define the fully Rust typed graph and owned transaction boundary | M01 |  |
| [M04](prs/M04.md) | Implement Element/Relationship containment and inverse ownership | M03 |  |
| [M05](prs/M05.md) | Implement scalar defaults, unset/null state and enum families | M03, M01 |  |
| [M06](prs/M06.md) | Bind semantic/resource identities to the Rust resource lifecycle | M03, M05 | #31 |
| [M07](prs/M07.md) | Index immutable resource lookup and exact scoped reference handles | M03, M06 | #47 |
| [M08](prs/M08.md) | Normalize ordinary and relative EMF resource addresses explicitly | M07, C01 |  |
| [M09](prs/M09.md) | Resolve lazy syson-import and cross-resource proxies without fallback | M08, M14 |  |
| [M10](prs/M10.md) | Integrate owner/effective/qualified names on live Rust snapshots | M04, M06, M07 | #49 |
| [M100](prs/M100.md) | Disjoining: disjoiningType, typeDisjoined feature contract | C01, C02, M03, M04, M05 |  |
| [M101](prs/M101.md) | Disjoining: owningType feature contract | C01, C02, M03, M04, M05 |  |
| [M102](prs/M102.md) | Documentation: documentedElement feature contract | C01, C02, M03, M04, M05 |  |
| [M103](prs/M103.md) | Element: aliasIds, declaredName, declaredShortName, isImpliedIncluded feature contract | C01, C02, M03, M04, M05 |  |
| [M104](prs/M104.md) | Element: elementId feature contract | C01, C02, M03, M04, M05 |  |
| [M105](prs/M105.md) | Element: isLibraryElement, name, qualifiedName, shortName feature contract | C01, C02, M03, M04, M05 |  |
| [M106](prs/M106.md) | Element: documentation, ownedAnnotation, ownedElement, owner feature contract | C01, C02, M03, M04, M05 |  |
| [M107](prs/M107.md) | Element: owningMembership, owningNamespace, textualRepresentation feature contract | C01, C02, M03, M04, M05 |  |
| [M108](prs/M108.md) | Element: ownedRelationship, owningRelationship feature contract | C01, C02, M03, M04, M05 |  |
| [M109](prs/M109.md) | ElementFilterMembership: condition feature contract | C01, C02, M03, M04, M05 |  |
| [M11](prs/M11.md) | Evaluate public namespace imports from actual resource graph facts | M04, M10, M07 | #34, #37 |
| [M110](prs/M110.md) | EnumerationDefinition: enumeratedValue feature contract | C01, C02, M03, M04, M05 |  |
| [M111](prs/M111.md) | EnumerationUsage: enumerationDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M112](prs/M112.md) | EventOccurrenceUsage: eventOccurrence feature contract | C01, C02, M03, M04, M05 |  |
| [M113](prs/M113.md) | ExhibitStateUsage: exhibitedState feature contract | C01, C02, M03, M04, M05 |  |
| [M114](prs/M114.md) | Expression: isModelLevelEvaluable feature contract | C01, C02, M03, M04, M05 |  |
| [M115](prs/M115.md) | Expression: function, result feature contract | C01, C02, M03, M04, M05 |  |
| [M116](prs/M116.md) | Feature: direction, isComposite, isConstant, isDerived feature contract | C01, C02, M03, M04, M05 |  |
| [M117](prs/M117.md) | Feature: isEnd, isOrdered, isPortion, isUnique feature contract | C01, C02, M03, M04, M05 |  |
| [M118](prs/M118.md) | Feature: isVariable feature contract | C01, C02, M03, M04, M05 |  |
| [M119](prs/M119.md) | Feature: isNonunique feature contract | C01, C02, M03, M04, M05 |  |
| [M12](prs/M12.md) | Complete private/protected/importAll namespace visibility | M11 |  |
| [M120](prs/M120.md) | Feature: chainingFeature, crossFeature, featureTarget, featuringType feature contract | C01, C02, M03, M04, M05 |  |
| [M121](prs/M121.md) | Feature: ownedRedefinition, type feature contract | C01, C02, M03, M04, M05 |  |
| [M122](prs/M122.md) | Feature: endOwningType, ownedCrossSubsetting, ownedFeatureChaining, ownedFeatureInverting feature contract | C01, C02, M03, M04, M05 |  |
| [M123](prs/M123.md) | Feature: ownedReferenceSubsetting, ownedSubsetting, ownedTypeFeaturing, ownedTyping feature contract | C01, C02, M03, M04, M05 |  |
| [M124](prs/M124.md) | Feature: owningFeatureMembership, owningType feature contract | C01, C02, M03, M04, M05 |  |
| [M125](prs/M125.md) | FeatureChainExpression: targetFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M126](prs/M126.md) | FeatureChaining: chainingFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M127](prs/M127.md) | FeatureChaining: featureChained feature contract | C01, C02, M03, M04, M05 |  |
| [M128](prs/M128.md) | FeatureInverting: featureInverted, invertingFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M129](prs/M129.md) | FeatureInverting: owningFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M13](prs/M13.md) | Implement Type inheritance and conjugated membership visibility | M12, M17 |  |
| [M130](prs/M130.md) | FeatureMembership: ownedMemberFeature, owningType feature contract | C01, C02, M03, M04, M05 |  |
| [M131](prs/M131.md) | FeatureReferenceExpression: referent feature contract | C01, C02, M03, M04, M05 |  |
| [M132](prs/M132.md) | FeatureTyping: owningFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M133](prs/M133.md) | FeatureTyping: type, typedFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M134](prs/M134.md) | FeatureValue: isDefault, isInitial feature contract | C01, C02, M03, M04, M05 |  |
| [M135](prs/M135.md) | FeatureValue: featureWithValue, value feature contract | C01, C02, M03, M04, M05 |  |
| [M136](prs/M136.md) | Flow: flowEnd, interaction, payloadFeature, payloadType feature contract | C01, C02, M03, M04, M05 |  |
| [M137](prs/M137.md) | Flow: sourceOutputFeature, targetInputFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M138](prs/M138.md) | FlowDefinition: flowEnd feature contract | C01, C02, M03, M04, M05 |  |
| [M139](prs/M139.md) | FlowUsage: flowDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M14](prs/M14.md) | Complete root/global/relative qualified-name and alias resolution | M10, M11, M08 |  |
| [M140](prs/M140.md) | ForLoopActionUsage: loopVariable, seqArgument feature contract | C01, C02, M03, M04, M05 |  |
| [M141](prs/M141.md) | FramedConcernMembership: ownedConcern, referencedConcern feature contract | C01, C02, M03, M04, M05 |  |
| [M142](prs/M142.md) | Function: isModelLevelEvaluable feature contract | C01, C02, M03, M04, M05 |  |
| [M143](prs/M143.md) | Function: expression, result feature contract | C01, C02, M03, M04, M05 |  |
| [M144](prs/M144.md) | IfActionUsage: elseAction, ifArgument, thenAction feature contract | C01, C02, M03, M04, M05 |  |
| [M145](prs/M145.md) | Import: isImportAll, isRecursive, visibility feature contract | C01, C02, M03, M04, M05 |  |
| [M146](prs/M146.md) | Import: importedElement feature contract | C01, C02, M03, M04, M05 |  |
| [M147](prs/M147.md) | Import: importOwningNamespace feature contract | C01, C02, M03, M04, M05 |  |
| [M148](prs/M148.md) | IncludeUseCaseUsage: useCaseIncluded feature contract | C01, C02, M03, M04, M05 |  |
| [M149](prs/M149.md) | InstantiationExpression: argument, instantiatedType feature contract | C01, C02, M03, M04, M05 |  |
| [M15](prs/M15.md) | Load and version pinned SysML/KerML standard libraries in Rust | M06, M08, M11, C01g |  |
| [M150](prs/M150.md) | InterfaceDefinition: interfaceEnd feature contract | C01, C02, M03, M04, M05 |  |
| [M151](prs/M151.md) | InterfaceUsage: interfaceDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M152](prs/M152.md) | Intersecting: intersectingType feature contract | C01, C02, M03, M04, M05 |  |
| [M153](prs/M153.md) | Intersecting: typeIntersected feature contract | C01, C02, M03, M04, M05 |  |
| [M154](prs/M154.md) | Invariant: isNegated feature contract | C01, C02, M03, M04, M05 |  |
| [M155](prs/M155.md) | ItemUsage: itemDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M156](prs/M156.md) | LibraryPackage: isStandard feature contract | C01, C02, M03, M04, M05 |  |
| [M157](prs/M157.md) | LiteralBoolean: value feature contract | C01, C02, M03, M04, M05 |  |
| [M158](prs/M158.md) | LiteralInteger: value feature contract | C01, C02, M03, M04, M05 |  |
| [M159](prs/M159.md) | LiteralRational: value feature contract | C01, C02, M03, M04, M05 |  |
| [M16](prs/M16.md) | Materialize implicit standard-library specializations | M15, M17, M18 |  |
| [M160](prs/M160.md) | LiteralString: value feature contract | C01, C02, M03, M04, M05 |  |
| [M161](prs/M161.md) | LoopActionUsage: bodyAction feature contract | C01, C02, M03, M04, M05 |  |
| [M162](prs/M162.md) | Membership: memberElementId feature contract | C01, C02, M03, M04, M05 |  |
| [M163](prs/M163.md) | Membership: memberName, memberShortName, visibility feature contract | C01, C02, M03, M04, M05 |  |
| [M164](prs/M164.md) | Membership: memberElement feature contract | C01, C02, M03, M04, M05 |  |
| [M165](prs/M165.md) | Membership: membershipOwningNamespace feature contract | C01, C02, M03, M04, M05 |  |
| [M166](prs/M166.md) | MembershipImport: importedMembership feature contract | C01, C02, M03, M04, M05 |  |
| [M167](prs/M167.md) | MetadataAccessExpression: referencedElement feature contract | C01, C02, M03, M04, M05 |  |
| [M168](prs/M168.md) | MetadataFeature: metaclass feature contract | C01, C02, M03, M04, M05 |  |
| [M169](prs/M169.md) | MetadataUsage: metadataDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M17](prs/M17.md) | Implement Type/Classifier operation families | M04, M06 |  |
| [M170](prs/M170.md) | MultiplicityRange: bound, lowerBound, upperBound feature contract | C01, C02, M03, M04, M05 |  |
| [M171](prs/M171.md) | Namespace: importedMembership, member, membership feature contract | C01, C02, M03, M04, M05 |  |
| [M172](prs/M172.md) | Namespace: ownedImport, ownedMember, ownedMembership feature contract | C01, C02, M03, M04, M05 |  |
| [M173](prs/M173.md) | NamespaceImport: importedNamespace feature contract | C01, C02, M03, M04, M05 |  |
| [M174](prs/M174.md) | ObjectiveMembership: ownedObjectiveRequirement feature contract | C01, C02, M03, M04, M05 |  |
| [M175](prs/M175.md) | OccurrenceDefinition: isIndividual feature contract | C01, C02, M03, M04, M05 |  |
| [M176](prs/M176.md) | OccurrenceUsage: isIndividual, portionKind feature contract | C01, C02, M03, M04, M05 |  |
| [M177](prs/M177.md) | OccurrenceUsage: individualDefinition, occurrenceDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M178](prs/M178.md) | OperatorExpression: operator feature contract | C01, C02, M03, M04, M05 |  |
| [M179](prs/M179.md) | OwningMembership: ownedMemberElementId, ownedMemberName, ownedMemberShortName feature contract | C01, C02, M03, M04, M05 |  |
| [M18](prs/M18.md) | Implement Feature typing/chaining/subsetting/redefinition operations | M17, M04 |  |
| [M180](prs/M180.md) | OwningMembership: ownedMemberElement feature contract | C01, C02, M03, M04, M05 |  |
| [M181](prs/M181.md) | Package: filterCondition feature contract | C01, C02, M03, M04, M05 |  |
| [M182](prs/M182.md) | ParameterMembership: ownedMemberParameter feature contract | C01, C02, M03, M04, M05 |  |
| [M183](prs/M183.md) | PartUsage: partDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M184](prs/M184.md) | PerformActionUsage: performedAction feature contract | C01, C02, M03, M04, M05 |  |
| [M185](prs/M185.md) | PortConjugation: conjugatedPortDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M186](prs/M186.md) | PortConjugation: originalPortDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M187](prs/M187.md) | PortDefinition: conjugatedPortDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M188](prs/M188.md) | PortUsage: portDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M189](prs/M189.md) | Redefinition: redefinedFeature, redefiningFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M19](prs/M19.md) | Qualify full accepted Unicode/name/string domains | C01, M05, M10 | #26 |
| [M190](prs/M190.md) | ReferenceSubsetting: referencedFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M191](prs/M191.md) | ReferenceSubsetting: referencingFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M192](prs/M192.md) | Relationship: isImplied feature contract | C01, C02, M03, M04, M05 |  |
| [M193](prs/M193.md) | Relationship: ownedRelatedElement, owningRelatedElement feature contract | C01, C02, M03, M04, M05 |  |
| [M194](prs/M194.md) | Relationship: relatedElement feature contract | C01, C02, M03, M04, M05 |  |
| [M195](prs/M195.md) | Relationship: source, target feature contract | C01, C02, M03, M04, M05 |  |
| [M196](prs/M196.md) | RenderingDefinition: rendering feature contract | C01, C02, M03, M04, M05 |  |
| [M197](prs/M197.md) | RenderingUsage: renderingDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M198](prs/M198.md) | RequirementConstraintMembership: kind feature contract | C01, C02, M03, M04, M05 |  |
| [M199](prs/M199.md) | RequirementConstraintMembership: ownedConstraint, referencedConstraint feature contract | C01, C02, M03, M04, M05 |  |
| [M20](prs/M20.md) | Implement ownership-sensitive delete/move/clone commands | M04, M05, M06, M18 |  |
| [M200](prs/M200.md) | RequirementDefinition: reqId feature contract | C01, C02, M03, M04, M05 |  |
| [M201](prs/M201.md) | RequirementDefinition: text feature contract | C01, C02, M03, M04, M05 |  |
| [M202](prs/M202.md) | RequirementDefinition: actorParameter, assumedConstraint, framedConcern, requiredConstraint feature contract | C01, C02, M03, M04, M05 |  |
| [M203](prs/M203.md) | RequirementDefinition: stakeholderParameter, subjectParameter feature contract | C01, C02, M03, M04, M05 |  |
| [M204](prs/M204.md) | RequirementUsage: reqId feature contract | C01, C02, M03, M04, M05 |  |
| [M205](prs/M205.md) | RequirementUsage: text feature contract | C01, C02, M03, M04, M05 |  |
| [M206](prs/M206.md) | RequirementUsage: actorParameter, assumedConstraint, framedConcern, requiredConstraint feature contract | C01, C02, M03, M04, M05 |  |
| [M207](prs/M207.md) | RequirementUsage: requirementDefinition, stakeholderParameter, subjectParameter feature contract | C01, C02, M03, M04, M05 |  |
| [M208](prs/M208.md) | RequirementVerificationMembership: ownedRequirement, verifiedRequirement feature contract | C01, C02, M03, M04, M05 |  |
| [M209](prs/M209.md) | ResultExpressionMembership: ownedResultExpression feature contract | C01, C02, M03, M04, M05 |  |
| [M21](prs/M21.md) | Implement dependency-indexed derived caches and invalidation | M17, M18, M20 |  |
| [M210](prs/M210.md) | SatisfyRequirementUsage: satisfiedRequirement, satisfyingFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M211](prs/M211.md) | SendActionUsage: payloadArgument, receiverArgument, senderArgument feature contract | C01, C02, M03, M04, M05 |  |
| [M212](prs/M212.md) | Specialization: general, specific feature contract | C01, C02, M03, M04, M05 |  |
| [M213](prs/M213.md) | Specialization: owningType feature contract | C01, C02, M03, M04, M05 |  |
| [M214](prs/M214.md) | StakeholderMembership: ownedStakeholderParameter feature contract | C01, C02, M03, M04, M05 |  |
| [M215](prs/M215.md) | StateDefinition: isParallel feature contract | C01, C02, M03, M04, M05 |  |
| [M216](prs/M216.md) | StateDefinition: doAction, entryAction, exitAction, state feature contract | C01, C02, M03, M04, M05 |  |
| [M217](prs/M217.md) | StateSubactionMembership: kind feature contract | C01, C02, M03, M04, M05 |  |
| [M218](prs/M218.md) | StateSubactionMembership: action feature contract | C01, C02, M03, M04, M05 |  |
| [M219](prs/M219.md) | StateUsage: isParallel feature contract | C01, C02, M03, M04, M05 |  |
| [M22](prs/M22.md) | Implement model notification and transaction event semantics | M04, M05, M20, M21 |  |
| [M220](prs/M220.md) | StateUsage: doAction, entryAction, exitAction, stateDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M221](prs/M221.md) | Step: behavior, parameter feature contract | C01, C02, M03, M04, M05 |  |
| [M222](prs/M222.md) | Subclassification: owningClassifier feature contract | C01, C02, M03, M04, M05 |  |
| [M223](prs/M223.md) | Subclassification: subclassifier, superclassifier feature contract | C01, C02, M03, M04, M05 |  |
| [M224](prs/M224.md) | SubjectMembership: ownedSubjectParameter feature contract | C01, C02, M03, M04, M05 |  |
| [M225](prs/M225.md) | Subsetting: owningFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M226](prs/M226.md) | Subsetting: subsettedFeature, subsettingFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M227](prs/M227.md) | TerminateActionUsage: terminatedOccurrenceArgument feature contract | C01, C02, M03, M04, M05 |  |
| [M228](prs/M228.md) | TextualRepresentation: body, language feature contract | C01, C02, M03, M04, M05 |  |
| [M229](prs/M229.md) | TextualRepresentation: representedElement feature contract | C01, C02, M03, M04, M05 |  |
| [M23](prs/M23.md) | Integrate qualified generic construction and initializer dispatch | M02, M03, M04, M05, M06 |  |
| [M230](prs/M230.md) | TransitionFeatureMembership: kind feature contract | C01, C02, M03, M04, M05 |  |
| [M231](prs/M231.md) | TransitionFeatureMembership: transitionFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M232](prs/M232.md) | TransitionUsage: effectAction, guardExpression, source, succession feature contract | C01, C02, M03, M04, M05 |  |
| [M233](prs/M233.md) | TransitionUsage: target, triggerAction feature contract | C01, C02, M03, M04, M05 |  |
| [M234](prs/M234.md) | TriggerInvocationExpression: kind feature contract | C01, C02, M03, M04, M05 |  |
| [M235](prs/M235.md) | Type: isAbstract, isSufficient feature contract | C01, C02, M03, M04, M05 |  |
| [M236](prs/M236.md) | Type: isConjugated feature contract | C01, C02, M03, M04, M05 |  |
| [M237](prs/M237.md) | Type: differencingType, directedFeature, endFeature, feature feature contract | C01, C02, M03, M04, M05 |  |
| [M238](prs/M238.md) | Type: featureMembership, inheritedFeature, inheritedMembership, input feature contract | C01, C02, M03, M04, M05 |  |
| [M239](prs/M239.md) | Type: intersectingType, multiplicity, output, unioningType feature contract | C01, C02, M03, M04, M05 |  |
| [M24](prs/M24.md) | Finish ConnectionUsage/InterfaceUsage and connector initializer schedules | M23 |  |
| [M240](prs/M240.md) | Type: ownedConjugator, ownedDifferencing, ownedDisjoining, ownedEndFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M241](prs/M241.md) | Type: ownedFeature, ownedFeatureMembership, ownedIntersecting, ownedSpecialization feature contract | C01, C02, M03, M04, M05 |  |
| [M242](prs/M242.md) | Type: ownedUnioning feature contract | C01, C02, M03, M04, M05 |  |
| [M243](prs/M243.md) | TypeFeaturing: featureOfType, featuringType feature contract | C01, C02, M03, M04, M05 |  |
| [M244](prs/M244.md) | TypeFeaturing: owningFeatureOfType feature contract | C01, C02, M03, M04, M05 |  |
| [M245](prs/M245.md) | Unioning: typeUnioned feature contract | C01, C02, M03, M04, M05 |  |
| [M246](prs/M246.md) | Unioning: unioningType feature contract | C01, C02, M03, M04, M05 |  |
| [M247](prs/M247.md) | Usage: isReference, mayTimeVary feature contract | C01, C02, M03, M04, M05 |  |
| [M248](prs/M248.md) | Usage: isVariation feature contract | C01, C02, M03, M04, M05 |  |
| [M249](prs/M249.md) | Usage: definition, directedUsage, nestedAction, nestedAllocation feature contract | C01, C02, M03, M04, M05 |  |
| [M25](prs/M25.md) | Complete core package/annotation/definition initializer families | M23 |  |
| [M250](prs/M250.md) | Usage: nestedAnalysisCase, nestedAttribute, nestedCalculation, nestedCase feature contract | C01, C02, M03, M04, M05 |  |
| [M251](prs/M251.md) | Usage: nestedConcern, nestedConnection, nestedConstraint, nestedEnumeration feature contract | C01, C02, M03, M04, M05 |  |
| [M252](prs/M252.md) | Usage: nestedFlow, nestedInterface, nestedItem, nestedMetadata feature contract | C01, C02, M03, M04, M05 |  |
| [M253](prs/M253.md) | Usage: nestedOccurrence, nestedPart, nestedPort, nestedReference feature contract | C01, C02, M03, M04, M05 |  |
| [M254](prs/M254.md) | Usage: nestedRendering, nestedRequirement, nestedState, nestedTransition feature contract | C01, C02, M03, M04, M05 |  |
| [M255](prs/M255.md) | Usage: nestedUseCase, nestedVerificationCase, nestedView, nestedViewpoint feature contract | C01, C02, M03, M04, M05 |  |
| [M256](prs/M256.md) | Usage: usage, variant, variantMembership feature contract | C01, C02, M03, M04, M05 |  |
| [M257](prs/M257.md) | Usage: nestedUsage, owningDefinition, owningUsage feature contract | C01, C02, M03, M04, M05 |  |
| [M258](prs/M258.md) | UseCaseDefinition: includedUseCase feature contract | C01, C02, M03, M04, M05 |  |
| [M259](prs/M259.md) | UseCaseUsage: includedUseCase, useCaseDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M26](prs/M26.md) | Complete part/item/attribute/occurrence/port initializer families | M23, M15 |  |
| [M260](prs/M260.md) | VariantMembership: ownedVariantUsage feature contract | C01, C02, M03, M04, M05 |  |
| [M261](prs/M261.md) | VerificationCaseDefinition: verifiedRequirement feature contract | C01, C02, M03, M04, M05 |  |
| [M262](prs/M262.md) | VerificationCaseUsage: verificationCaseDefinition, verifiedRequirement feature contract | C01, C02, M03, M04, M05 |  |
| [M263](prs/M263.md) | ViewDefinition: satisfiedViewpoint, view, viewCondition, viewRendering feature contract | C01, C02, M03, M04, M05 |  |
| [M264](prs/M264.md) | ViewpointDefinition: viewpointStakeholder feature contract | C01, C02, M03, M04, M05 |  |
| [M265](prs/M265.md) | ViewpointUsage: viewpointDefinition, viewpointStakeholder feature contract | C01, C02, M03, M04, M05 |  |
| [M266](prs/M266.md) | ViewRenderingMembership: ownedRendering, referencedRendering feature contract | C01, C02, M03, M04, M05 |  |
| [M267](prs/M267.md) | ViewUsage: exposedElement, satisfiedViewpoint, viewCondition, viewDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M268](prs/M268.md) | ViewUsage: viewRendering feature contract | C01, C02, M03, M04, M05 |  |
| [M269](prs/M269.md) | WhileLoopActionUsage: untilArgument, whileArgument feature contract | C01, C02, M03, M04, M05 |  |
| [M27](prs/M27.md) | Complete action/flow/control initializer families | M23, M18 |  |
| [M270](prs/M270.md) | SysMLPackageNodeStyleDescription: background feature contract | C01, C02, M03, M04, M05 |  |
| [M271](prs/M271.md) | SysMLNoteNodeStyleDescription: background feature contract | C01, C02, M03, M04, M05 |  |
| [M272](prs/M272.md) | SysMLImportedPackageNodeStyleDescription: background feature contract | C01, C02, M03, M04, M05 |  |
| [M273](prs/M273.md) | SysMLViewFrameNodeStyleDescription: background feature contract | C01, C02, M03, M04, M05 |  |
| [M274](prs/M274.md) | AcceptActionUsage: isTriggerAction operation contract | C01, C02, M03, M04, M05 |  |
| [M275](prs/M275.md) | ActionUsage: argument, inputParameter operation contract | C01, C02, M03, M04, M05 |  |
| [M276](prs/M276.md) | ActionUsage: inputParameters, isSubactionUsage operation contract | C01, C02, M03, M04, M05 |  |
| [M277](prs/M277.md) | ControlNode: multiplicityHasBounds operation contract | C01, C02, M03, M04, M05 |  |
| [M278](prs/M278.md) | Element: effectiveName, effectiveShortName operation contract | C01, C02, M03, M04, M05 |  |
| [M279](prs/M279.md) | Element: escapedName, path operation contract | C01, C02, M03, M04, M05 |  |
| [M28](prs/M28.md) | Complete state/transition initializer families | M23, M18 |  |
| [M280](prs/M280.md) | Element: libraryNamespace operation contract | C01, C02, M03, M04, M05 |  |
| [M281](prs/M281.md) | Expression: checkCondition, evaluate operation contract | C01, C02, M03, M04, M05 |  |
| [M282](prs/M282.md) | Expression: modelLevelEvaluable operation contract | C01, C02, M03, M04, M05 |  |
| [M283](prs/M283.md) | Feature: allRedefinedFeatures, asCartesianProduct operation contract | C01, C02, M03, M04, M05 |  |
| [M284](prs/M284.md) | Feature: canAccess, directionFor operation contract | C01, C02, M03, M04, M05 |  |
| [M285](prs/M285.md) | Feature: isCartesianProduct, isFeaturedWithin operation contract | C01, C02, M03, M04, M05 |  |
| [M286](prs/M286.md) | Feature: isFeaturingType, isOwnedCrossFeature operation contract | C01, C02, M03, M04, M05 |  |
| [M287](prs/M287.md) | Feature: namingFeature, ownedCrossFeature operation contract | C01, C02, M03, M04, M05 |  |
| [M288](prs/M288.md) | Feature: redefines, redefinesFromLibrary operation contract | C01, C02, M03, M04, M05 |  |
| [M289](prs/M289.md) | Feature: subsetsChain, typingFeatures operation contract | C01, C02, M03, M04, M05 |  |
| [M29](prs/M29.md) | Complete requirement/case/objective initializer families | M23, M18 |  |
| [M290](prs/M290.md) | FeatureChainExpression: sourceTargetFeature operation contract | C01, C02, M03, M04, M05 |  |
| [M291](prs/M291.md) | Import: importedMemberships operation contract | C01, C02, M03, M04, M05 |  |
| [M292](prs/M292.md) | InstantiationExpression: instantiatedType operation contract | C01, C02, M03, M04, M05 |  |
| [M293](prs/M293.md) | Membership: isDistinguishableFrom operation contract | C01, C02, M03, M04, M05 |  |
| [M294](prs/M294.md) | MetadataAccessExpression: metaclassFeature operation contract | C01, C02, M03, M04, M05 |  |
| [M295](prs/M295.md) | MetadataFeature: evaluateFeature, isSemantic operation contract | C01, C02, M03, M04, M05 |  |
| [M296](prs/M296.md) | MetadataFeature: isSyntactic, syntaxElement operation contract | C01, C02, M03, M04, M05 |  |
| [M297](prs/M297.md) | MultiplicityRange: hasBounds, valueOf operation contract | C01, C02, M03, M04, M05 |  |
| [M298](prs/M298.md) | Namespace: importedMemberships, membershipsOfVisibility operation contract | C01, C02, M03, M04, M05 |  |
| [M299](prs/M299.md) | Namespace: visibleMemberships operation contract | C01, C02, M03, M04, M05 |  |
| [M30](prs/M30.md) | Complete view/rendering/exposure initializer families | M23, M15, M14 | #48, #54 |
| [M300](prs/M300.md) | Namespace: namesOf, qualificationOf operation contract | C01, C02, M03, M04, M05 |  |
| [M301](prs/M301.md) | Namespace: resolve, resolveGlobal operation contract | C01, C02, M03, M04, M05 |  |
| [M302](prs/M302.md) | Namespace: resolveLocal, resolveVisible operation contract | C01, C02, M03, M04, M05 |  |
| [M303](prs/M303.md) | Namespace: unqualifiedNameOf, visibilityOf operation contract | C01, C02, M03, M04, M05 |  |
| [M304](prs/M304.md) | Package: includeAsMember operation contract | C01, C02, M03, M04, M05 |  |
| [M305](prs/M305.md) | ParameterMembership: parameterDirection operation contract | C01, C02, M03, M04, M05 |  |
| [M306](prs/M306.md) | StateUsage: isSubstateUsage operation contract | C01, C02, M03, M04, M05 |  |
| [M307](prs/M307.md) | TransitionUsage: sourceFeature, triggerPayloadParameter operation contract | C01, C02, M03, M04, M05 |  |
| [M308](prs/M308.md) | Type: allRedefinedFeaturesOf, allSupertypes operation contract | C01, C02, M03, M04, M05 |  |
| [M309](prs/M309.md) | Type: directionOf, directionOfExcluding operation contract | C01, C02, M03, M04, M05 |  |
| [M31](prs/M31.md) | Complete parser-owned expression/literal/metadata construction | M23, M18 |  |
| [M310](prs/M310.md) | Type: isCompatibleWith, multiplicities operation contract | C01, C02, M03, M04, M05 |  |
| [M311](prs/M311.md) | Type: specializes, specializesFromLibrary operation contract | C01, C02, M03, M04, M05 |  |
| [M312](prs/M312.md) | Type: supertypes operation contract | C01, C02, M03, M04, M05 |  |
| [M313](prs/M313.md) | Type: inheritableMemberships, inheritedMemberships operation contract | C01, C02, M03, M04, M05 |  |
| [M314](prs/M314.md) | Type: nonPrivateMemberships, removeRedefinedFeatures operation contract | C01, C02, M03, M04, M05 |  |
| [M315](prs/M315.md) | Usage: referencedFeatureTarget operation contract | C01, C02, M03, M04, M05 |  |
| [M316](prs/M316.md) | ViewUsage: includeAsExposed operation contract | C01, C02, M03, M04, M05 |  |
| [M317](prs/M317.md) | AcceptActionUsage: exact caseAcceptActionUsage initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M318](prs/M318.md) | AttributeUsage: exact caseAttributeUsage initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M319](prs/M319.md) | BindingConnectorAsUsage: exact caseBindingConnectorAsUsage initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M32](prs/M32.md) | Port foundational namespace/membership/containment rule families | M04, M10, M11, M22 | #57 |
| [M320](prs/M320.md) | Comment: exact caseComment initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M321](prs/M321.md) | ConnectionDefinition: exact caseConnectionDefinition initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M322](prs/M322.md) | InterfaceDefinition: exact caseInterfaceDefinition initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M323](prs/M323.md) | TextualRepresentation: exact caseTextualRepresentation initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M324](prs/M324.md) | Dependency: exact caseDependency initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M325](prs/M325.md) | Definition: exact caseDefinition initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M326](prs/M326.md) | Documentation: exact caseDocumentation initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M327](prs/M327.md) | Element: exact caseElement initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M328](prs/M328.md) | EnumerationDefinition: exact caseEnumerationDefinition initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M329](prs/M329.md) | FeatureTyping: exact caseFeatureTyping initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M33](prs/M33.md) | Port type/feature/typing constraint families | M32, M17, M18 |  |
| [M330](prs/M330.md) | FlowUsage: exact caseFlowUsage initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M331](prs/M331.md) | Expose: exact caseExpose initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M332](prs/M332.md) | Package: exact casePackage initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M333](prs/M333.md) | PartUsage: exact casePartUsage initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M334](prs/M334.md) | PerformActionUsage: exact casePerformActionUsage initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M335](prs/M335.md) | PortDefinition: exact casePortDefinition initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M336](prs/M336.md) | Redefinition: exact caseRedefinition initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M337](prs/M337.md) | ReferenceSubsetting: exact caseReferenceSubsetting initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M338](prs/M338.md) | ReferenceUsage: exact caseReferenceUsage initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M339](prs/M339.md) | RequirementUsage: exact caseRequirementUsage initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M34](prs/M34.md) | Port structural usage/part/port/connection rule families | M33, M26, M24 |  |
| [M340](prs/M340.md) | Specialization: exact caseSpecialization initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M341](prs/M341.md) | Subclassification: exact caseSubclassification initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M342](prs/M342.md) | Subsetting: exact caseSubsetting initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M343](prs/M343.md) | SuccessionAsUsage: exact caseSuccessionAsUsage initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M344](prs/M344.md) | TransitionUsage: exact caseTransitionUsage initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M345](prs/M345.md) | Usage: exact caseUsage initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M346](prs/M346.md) | ViewUsage: exact caseViewUsage initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M347](prs/M347.md) | FlowEnd: exact caseFlowEnd initializer schedule | C01, C02, M02, M03, M04, M05 |  |
| [M348](prs/M348.md) | Construction admission: AcceptActionUsage, ActionDefinition, ActionUsage, AssignmentActionUsage | C01, C02, M02, M23 |  |
| [M349](prs/M349.md) | Construction admission: DecisionNode, ForkNode, ForLoopActionUsage, IfActionUsage | C01, C02, M02, M23 |  |
| [M35](prs/M35.md) | Port action/state/flow constraint families | M33, M27, M28 |  |
| [M350](prs/M350.md) | Construction admission: JoinNode, MergeNode, PerformActionUsage, SendActionUsage | C01, C02, M02, M23 |  |
| [M351](prs/M351.md) | Construction admission: TerminateActionUsage, WhileLoopActionUsage | C01, C02, M02, M23 |  |
| [M352](prs/M352.md) | Construction admission: ActorMembership, AnalysisCaseDefinition, AnalysisCaseUsage, CaseDefinition | C01, C02, M02, M23 |  |
| [M353](prs/M353.md) | Construction admission: CaseUsage, ConcernDefinition, ConcernUsage, FramedConcernMembership | C01, C02, M02, M23 |  |
| [M354](prs/M354.md) | Construction admission: IncludeUseCaseUsage, ObjectiveMembership, RequirementConstraintMembership, RequirementDefinition | C01, C02, M02, M23 |  |
| [M355](prs/M355.md) | Construction admission: RequirementUsage, RequirementVerificationMembership, SatisfyRequirementUsage, StakeholderMembership | C01, C02, M02, M23 |  |
| [M356](prs/M356.md) | Construction admission: SubjectMembership, UseCaseDefinition, UseCaseUsage, VerificationCaseDefinition | C01, C02, M02, M23 |  |
| [M357](prs/M357.md) | Construction admission: VerificationCaseUsage | C01, C02, M02, M23 |  |
| [M358](prs/M358.md) | Construction admission: AllocationDefinition, AllocationUsage, Association, AssociationStructure | C01, C02, M02, M23 |  |
| [M359](prs/M359.md) | Construction admission: BindingConnector, BindingConnectorAsUsage, ConjugatedPortDefinition, ConjugatedPortTyping | C01, C02, M02, M23 |  |
| [M36](prs/M36.md) | Port requirement/case/view/rendering constraint families | M33, M29, M30 |  |
| [M360](prs/M360.md) | Construction admission: ConnectionDefinition, ConnectionUsage, Connector, Flow | C01, C02, M02, M23 |  |
| [M361](prs/M361.md) | Construction admission: FlowDefinition, FlowEnd, FlowUsage, InterfaceDefinition | C01, C02, M02, M23 |  |
| [M362](prs/M362.md) | Construction admission: InterfaceUsage, PortConjugation, PortDefinition, PortUsage | C01, C02, M02, M23 |  |
| [M363](prs/M363.md) | Construction admission: Succession, SuccessionAsUsage, SuccessionFlow, SuccessionFlowUsage | C01, C02, M02, M23 |  |
| [M364](prs/M364.md) | Construction admission: AnnotatingElement, AssertConstraintUsage, Behavior, CalculationDefinition | C01, C02, M02, M23 |  |
| [M365](prs/M365.md) | Construction admission: CalculationUsage, Class, Classifier, Comment | C01, C02, M02, M23 |  |
| [M366](prs/M366.md) | Construction admission: ConstraintDefinition, ConstraintUsage, DataType, Definition | C01, C02, M02, M23 |  |
| [M367](prs/M367.md) | Construction admission: Documentation, Feature, Function, Interaction | C01, C02, M02, M23 |  |
| [M368](prs/M368.md) | Construction admission: Invariant, LiteralBoolean, LiteralInfinity, LiteralInteger | C01, C02, M02, M23 |  |
| [M369](prs/M369.md) | Construction admission: LiteralRational, LiteralString, Metaclass, Namespace | C01, C02, M02, M23 |  |
| [M37](prs/M37.md) | Port expression/multiplicity/metadata constraint families | M33, M31 |  |
| [M370](prs/M370.md) | Construction admission: Package, PayloadFeature, Predicate, Step | C01, C02, M02, M23 |  |
| [M371](prs/M371.md) | Construction admission: Structure, TextualRepresentation, Type, Usage | C01, C02, M02, M23 |  |
| [M372](prs/M372.md) | Construction admission: Annotation, Conjugation, CrossSubsetting, Dependency | C01, C02, M02, M23 |  |
| [M373](prs/M373.md) | Construction admission: Differencing, Disjoining, ElementFilterMembership, EndFeatureMembership | C01, C02, M02, M23 |  |
| [M374](prs/M374.md) | Construction admission: FeatureChaining, FeatureInverting, FeatureMembership, FeatureTyping | C01, C02, M02, M23 |  |
| [M375](prs/M375.md) | Construction admission: Intersecting, Membership, OwningMembership, ParameterMembership | C01, C02, M02, M23 |  |
| [M376](prs/M376.md) | Construction admission: Redefinition, ReferenceSubsetting, ReturnParameterMembership, Specialization | C01, C02, M02, M23 |  |
| [M377](prs/M377.md) | Construction admission: Subclassification, Subsetting, TypeFeaturing, Unioning | C01, C02, M02, M23 |  |
| [M378](prs/M378.md) | Construction admission: VariantMembership | C01, C02, M02, M23 |  |
| [M379](prs/M379.md) | Construction admission: AttributeDefinition, AttributeUsage, EnumerationDefinition, EnumerationUsage | C01, C02, M02, M23 |  |
| [M38](prs/M38.md) | Integrate metamodel query/mutation service APIs with actual callers | M20, M22, M23, M14 |  |
| [M380](prs/M380.md) | Construction admission: EventOccurrenceUsage, ItemDefinition, ItemUsage, MetadataDefinition | C01, C02, M02, M23 |  |
| [M381](prs/M381.md) | Construction admission: MetadataFeature, MetadataUsage, OccurrenceDefinition, OccurrenceUsage | C01, C02, M02, M23 |  |
| [M382](prs/M382.md) | Construction admission: PartDefinition, PartUsage, ReferenceUsage | C01, C02, M02, M23 |  |
| [M383](prs/M383.md) | Construction admission: BooleanExpression, CollectExpression, ConstructorExpression, Expression | C01, C02, M02, M23 |  |
| [M384](prs/M384.md) | Construction admission: FeatureChainExpression, FeatureReferenceExpression, FeatureValue, IndexExpression | C01, C02, M02, M23 |  |
| [M385](prs/M385.md) | Construction admission: InvocationExpression, LiteralExpression, MetadataAccessExpression, Multiplicity | C01, C02, M02, M23 |  |
| [M386](prs/M386.md) | Construction admission: MultiplicityRange, NullExpression, OperatorExpression, ResultExpressionMembership | C01, C02, M02, M23 |  |
| [M387](prs/M387.md) | Construction admission: SelectExpression, TriggerInvocationExpression | C01, C02, M02, M23 |  |
| [M388](prs/M388.md) | Construction admission: ConnectorAsUsage | C01, C02, M02, M23 |  |
| [M389](prs/M389.md) | Construction admission: ControlNode, LoopActionUsage | C01, C02, M02, M23 |  |
| [M39](prs/M39.md) | Qualify library reload and cross-resource cache lifecycle | M15, M21, M22, M09 |  |
| [M390](prs/M390.md) | Construction admission: Element, Relationship | C01, C02, M02, M23 |  |
| [M391](prs/M391.md) | Construction admission: ExhibitStateUsage, StateDefinition, StateSubactionMembership, StateUsage | C01, C02, M02, M23 |  |
| [M392](prs/M392.md) | Construction admission: TransitionFeatureMembership, TransitionUsage | C01, C02, M02, M23 |  |
| [M393](prs/M393.md) | Construction admission: Expose | C01, C02, M02, M23 |  |
| [M394](prs/M394.md) | Construction admission: Import | C01, C02, M02, M23 |  |
| [M395](prs/M395.md) | Construction admission: InstantiationExpression | C01, C02, M02, M23 |  |
| [M396](prs/M396.md) | Construction admission: LibraryPackage, MembershipImport, NamespaceImport | C01, C02, M02, M23 |  |
| [M397](prs/M397.md) | Construction admission: MembershipExpose, NamespaceExpose, RenderingDefinition, RenderingUsage | C01, C02, M02, M23 |  |
| [M398](prs/M398.md) | Construction admission: ViewDefinition, ViewpointDefinition, ViewpointUsage, ViewRenderingMembership | C01, C02, M02, M23 |  |
| [M399](prs/M399.md) | Construction admission: ViewUsage | C01, C02, M02, M23 |  |
| [M40](prs/M40.md) | Expose custom-node schema identities to representation services | M01, M05, C01 |  |
| [M400](prs/M400.md) | Construction admission: SysMLPackageNodeStyleDescription, SysMLNoteNodeStyleDescription, SysMLImportedPackageNodeStyleDescription, SysMLViewFrameNodeStyleDescription | C01, C02, M40 |  |
| [M41](prs/M41.md) | Relationship and membership derived getter acceptance | C01, C02, M03, M04, M05 |  |
| [M42](prs/M42.md) | Structural, definition and occurrence derived getter acceptance | C01, C02, M03, M04, M05 |  |
| [M43](prs/M43.md) | Connection, port and flow derived getter acceptance | C01, C02, M03, M04, M05 |  |
| [M44](prs/M44.md) | Action and control derived getter and operation acceptance | C01, C02, M03, M04, M05 |  |
| [M45](prs/M45.md) | State and transition derived getter and operation acceptance | C01, C02, M03, M04, M05 |  |
| [M46](prs/M46.md) | Requirement, case and objective derived getter acceptance | C01, C02, M03, M04, M05 |  |
| [M47](prs/M47.md) | View, rendering and exposure derived getter acceptance | C01, C02, M03, M04, M05 |  |
| [M48](prs/M48.md) | Expression, value, multiplicity and metadata getter and operation acceptance | C01, C02, M03, M04, M05 |  |
| [M50](prs/M50.md) | AcceptActionUsage: payloadArgument, payloadParameter, receiverArgument feature contract | C01, C02, M03, M04, M05 |  |
| [M51](prs/M51.md) | ActionDefinition: action feature contract | C01, C02, M03, M04, M05 |  |
| [M52](prs/M52.md) | ActionUsage: actionDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M53](prs/M53.md) | ActorMembership: ownedActorParameter feature contract | C01, C02, M03, M04, M05 |  |
| [M54](prs/M54.md) | AllocationDefinition: allocation feature contract | C01, C02, M03, M04, M05 |  |
| [M55](prs/M55.md) | AllocationUsage: allocationDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M56](prs/M56.md) | AnalysisCaseDefinition: resultExpression feature contract | C01, C02, M03, M04, M05 |  |
| [M57](prs/M57.md) | AnalysisCaseUsage: analysisCaseDefinition, resultExpression feature contract | C01, C02, M03, M04, M05 |  |
| [M58](prs/M58.md) | AnnotatingElement: annotatedElement feature contract | C01, C02, M03, M04, M05 |  |
| [M59](prs/M59.md) | AnnotatingElement: annotation, ownedAnnotatingRelationship, owningAnnotatingRelationship feature contract | C01, C02, M03, M04, M05 |  |
| [M60](prs/M60.md) | Annotation: annotatedElement feature contract | C01, C02, M03, M04, M05 |  |
| [M61](prs/M61.md) | Annotation: annotatingElement, ownedAnnotatingElement, owningAnnotatedElement, owningAnnotatingElement feature contract | C01, C02, M03, M04, M05 |  |
| [M62](prs/M62.md) | AssertConstraintUsage: assertedConstraint feature contract | C01, C02, M03, M04, M05 |  |
| [M63](prs/M63.md) | AssignmentActionUsage: referent, targetArgument, valueExpression feature contract | C01, C02, M03, M04, M05 |  |
| [M64](prs/M64.md) | Association: associationEnd, relatedType, sourceType, targetType feature contract | C01, C02, M03, M04, M05 |  |
| [M65](prs/M65.md) | AttributeUsage: attributeDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M66](prs/M66.md) | Behavior: parameter, step feature contract | C01, C02, M03, M04, M05 |  |
| [M67](prs/M67.md) | BooleanExpression: predicate feature contract | C01, C02, M03, M04, M05 |  |
| [M68](prs/M68.md) | CalculationDefinition: calculation feature contract | C01, C02, M03, M04, M05 |  |
| [M69](prs/M69.md) | CalculationUsage: calculationDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M70](prs/M70.md) | CaseDefinition: actorParameter, objectiveRequirement, subjectParameter feature contract | C01, C02, M03, M04, M05 |  |
| [M71](prs/M71.md) | CaseUsage: actorParameter, caseDefinition, objectiveRequirement, subjectParameter feature contract | C01, C02, M03, M04, M05 |  |
| [M72](prs/M72.md) | Classifier: ownedSubclassification feature contract | C01, C02, M03, M04, M05 |  |
| [M73](prs/M73.md) | Comment: body, locale feature contract | C01, C02, M03, M04, M05 |  |
| [M74](prs/M74.md) | ConcernUsage: concernDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M75](prs/M75.md) | ConjugatedPortDefinition: originalPortDefinition, ownedPortConjugator feature contract | C01, C02, M03, M04, M05 |  |
| [M76](prs/M76.md) | ConjugatedPortTyping: conjugatedPortDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M77](prs/M77.md) | ConjugatedPortTyping: portDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M78](prs/M78.md) | Conjugation: conjugatedType, originalType feature contract | C01, C02, M03, M04, M05 |  |
| [M79](prs/M79.md) | Conjugation: owningType feature contract | C01, C02, M03, M04, M05 |  |
| [M80](prs/M80.md) | ConnectionDefinition: connectionEnd feature contract | C01, C02, M03, M04, M05 |  |
| [M81](prs/M81.md) | ConnectionUsage: connectionDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M82](prs/M82.md) | Connector: association, connectorEnd, defaultFeaturingType, relatedFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M83](prs/M83.md) | Connector: sourceFeature, targetFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M84](prs/M84.md) | ConstraintUsage: constraintDefinition feature contract | C01, C02, M03, M04, M05 |  |
| [M85](prs/M85.md) | CrossSubsetting: crossedFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M86](prs/M86.md) | CrossSubsetting: crossingFeature feature contract | C01, C02, M03, M04, M05 |  |
| [M87](prs/M87.md) | Definition: isVariation feature contract | C01, C02, M03, M04, M05 |  |
| [M88](prs/M88.md) | Definition: directedUsage, ownedAction, ownedAllocation, ownedAnalysisCase feature contract | C01, C02, M03, M04, M05 |  |
| [M89](prs/M89.md) | Definition: ownedAttribute, ownedCalculation, ownedCase, ownedConcern feature contract | C01, C02, M03, M04, M05 |  |
| [M90](prs/M90.md) | Definition: ownedConnection, ownedConstraint, ownedEnumeration, ownedFlow feature contract | C01, C02, M03, M04, M05 |  |
| [M91](prs/M91.md) | Definition: ownedInterface, ownedItem, ownedMetadata, ownedOccurrence feature contract | C01, C02, M03, M04, M05 |  |
| [M92](prs/M92.md) | Definition: ownedPart, ownedPort, ownedReference, ownedRendering feature contract | C01, C02, M03, M04, M05 |  |
| [M93](prs/M93.md) | Definition: ownedRequirement, ownedState, ownedTransition, ownedUseCase feature contract | C01, C02, M03, M04, M05 |  |
| [M94](prs/M94.md) | Definition: ownedVerificationCase, ownedView, ownedViewpoint, usage feature contract | C01, C02, M03, M04, M05 |  |
| [M95](prs/M95.md) | Definition: variant, variantMembership feature contract | C01, C02, M03, M04, M05 |  |
| [M96](prs/M96.md) | Definition: ownedUsage feature contract | C01, C02, M03, M04, M05 |  |
| [M97](prs/M97.md) | Dependency: client, supplier feature contract | C01, C02, M03, M04, M05 |  |
| [M98](prs/M98.md) | Differencing: differencingType feature contract | C01, C02, M03, M04, M05 |  |
| [M99](prs/M99.md) | Differencing: typeDifferenced feature contract | C01, C02, M03, M04, M05 |  |
| [P01](prs/P01.md) | Support hidden single-line // notes without rewriting source |  | #40, #25 |
| [P02](prs/P02.md) | Add hidden //* multiline-note precedence as a distinct lexical family | C01, C02, P01 | #40, #25 |
| [P03](prs/P03.md) | Qualify native whitespace and regular/unrestricted-name token boundaries | C01, C02 | #40, #25 |
| [P04](prs/P04.md) | Complete numeric and quoted-string terminal scanning/conversion boundaries | C01, C02 | #40, #25 |
| [P05](prs/P05.md) | Tokenize regular block comments as semantic input, never hidden trivia | C01, C02 | #40, #25 |
| [P06](prs/P06.md) | Compile pinned keyword/terminal priority tables in bounded review batches | P01, P02, P03, P04, P05 | #23, #25, #35, #40, #41 |
| [P07](prs/P07.md) | Preserve CST token/trivia ranges and UTF-16 source mapping | P06 | #23, #25, #35, #40, #41 |
| [P08](prs/P08.md) | Add bounded token checkpoints and deterministic parse failure state | P07 | #23, #25, #35, #40, #41 |
| [P09](prs/P09.md) | Implement sequence and ordered-alternative CST execution | P08 | #23, #25, #35, #40, #41 |
| [P10](prs/P10.md) | Implement cardinality and repetition with non-progress guards | P09 | #23, #25, #35, #40, #41 |
| [P100](prs/P100.md) | KerML features typing multiplicity branch batch 3: References, Crosses, Redefinitions and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P101](prs/P101.md) | KerML features typing multiplicity branch batch 4: Subsetting, OwnedSubsetting, OwnedReferenceSubsetting and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P102](prs/P102.md) | KerML features typing multiplicity branch batch 5: Redefinition, OwnedRedefinition, FeatureValue and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P103](prs/P103.md) | KerML features typing multiplicity branch batch 6: MultiplicityBounds, OwnedCrossMultiplicityMember, OwnedCrossMultiplicity and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P104](prs/P104.md) | KerML features typing multiplicity branch batch 7: RedefinesToken, FeatureDirectionKind, MultiplicityProperties and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P105](prs/P105.md) | KerML features typing multiplicity branch batch 8: OwnedFeatureTyping | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P106](prs/P106.md) | SysML features typing multiplicity branch batch 1: ImportedReference, FeatureDeclaration, Typings and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P107](prs/P107.md) | SysML features typing multiplicity branch batch 2: Redefines, FeatureTyping, OwnedSubsetting and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P108](prs/P108.md) | SysML features typing multiplicity branch batch 3: Reference, OwnedCrossingFeatureMember, OwnedCrossingFeature and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P109](prs/P109.md) | SysML features typing multiplicity branch batch 4: FeatureBinding, ParameterRedefinition, SubsetsToken and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P11](prs/P11.md) | Implement rule/fragment calls and parameterized lookahead gates | P10 | #23, #25, #35, #40, #41 |
| [P110](prs/P110.md) | SysML features typing multiplicity branch batch 5: QualifiedReferenceChain, ElementReference, NamespaceReference and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P111](prs/P111.md) | KerML kernel types relationships branch batch 1: TypedByToken, ConjugatesToken, TypePrefix and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P112](prs/P112.md) | KerML kernel types relationships branch batch 2: TypeFeatureMember, Specialization, OwnedSpecialization and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P113](prs/P113.md) | KerML kernel types relationships branch batch 3: OwnedConjugation, Disjoining, OwnedDisjoining and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P114](prs/P114.md) | KerML kernel types relationships branch batch 4: Differencing, Classifier, ClassifierDeclaration and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P115](prs/P115.md) | KerML kernel types relationships branch batch 5: TypeFeaturing, OwnedTypeFeaturing, FeatureConjugation and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P116](prs/P116.md) | KerML kernel types relationships branch batch 6: Behavior, Step, StepDeclaration and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P117](prs/P117.md) | SysML kernel types relationships branch batch 1: OwnedSubclassification, FeatureSpecialization, TypedBy and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P118](prs/P118.md) | KerML names prefixes modifiers branch batch 1: Succession, SuccessionDeclaration, FunctionBody and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P119](prs/P119.md) | KerML names prefixes modifiers branch batch 2: SpecializesToken, VisibilityKind, Abstract and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P12](prs/P12.md) | Bridge CST assignment events into the existing AST operation kernel | P11 | #23, #25, #35, #40, #41 |
| [P120](prs/P120.md) | KerML names prefixes modifiers branch batch 3: Standard | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P121](prs/P121.md) | SysML names prefixes modifiers branch batch 1: DefinedByToken, RefPrefix, EmptySuccession and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P122](prs/P122.md) | SysML names prefixes modifiers branch batch 2: VisibilityKind, Abstract, Visibility and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P123](prs/P123.md) | KerML namespaces packages imports branch batch 1: RelationshipBody, RelationshipOwnedElements, RelationshipOwnedElement and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P124](prs/P124.md) | KerML namespaces packages imports branch batch 2: NamespaceBodyElement, RelationshipMember, AliasMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P125](prs/P125.md) | KerML namespaces packages imports branch batch 3: Package, LibraryPackage, PackageBody and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P126](prs/P126.md) | SysML namespaces packages imports branch batch 1: Dependency, RelationshipBody, RelationshipBodyElements and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P127](prs/P127.md) | SysML namespaces packages imports branch batch 2: PackageBodyElement, RelationshipMember, ElementFilterMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P128](prs/P128.md) | SysML namespaces packages imports branch batch 3: FilterPackage, FilterPackageMember, InitialNodeMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P129](prs/P129.md) | SysML requirements cases concerns branch batch 1: RequirementConstraintKind, RequirementDefinition, RequirementBody and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P13](prs/P13.md) | Bridge AST actions, unassigned subrules and datatype returns | P12 | #23, #25, #35, #40, #41 |
| [P130](prs/P130.md) | SysML requirements cases concerns branch batch 2: RequirementConstraintUsage, FramedConcernMember, FramedConcernUsage and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P131](prs/P131.md) | SysML requirements cases concerns branch batch 3: SatisfyRequirementUsage, SatisfactionSubjectMember, SatisfactionParameter and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P132](prs/P132.md) | SysML requirements cases concerns branch batch 4: CaseBody, CaseBodyItems, CaseBodyItem and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P133](prs/P133.md) | SysML requirements cases concerns branch batch 5: VerificationCaseDefinition, VerificationCaseUsage, RequirementVerificationMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P134](prs/P134.md) | SysML states transitions branch batch 1: StateDefinition, StateBody, StateBodyPart | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P135](prs/P135.md) | SysML states transitions branch batch 2: StateBodyItem | C01, C02, P12, P13, P15, P134 | #40, #41, #50, #52 |
| [P136](prs/P136.md) | SysML states transitions branch batch 3: StateBodyItem | C01, C02, P12, P13, P15, P135 | #40, #41, #50, #52 |
| [P137](prs/P137.md) | SysML states transitions branch batch 4: EntryActionMember, DoActionMember, ExitActionMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P138](prs/P138.md) | SysML states transitions branch batch 5: StateActionUsage_5, StateActionUsage, TransitionUsageMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P139](prs/P139.md) | SysML states transitions branch batch 6: TargetTransitionUsage, TransitionSourceMember | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P14](prs/P14.md) | Qualify native mandatory defaults, reflection ordering and child-cache projection | P13 | #23, #25, #35, #40, #41 |
| [P140](prs/P140.md) | SysML states transitions branch batch 7: TriggerActionMember, TriggerAction, GuardExpressionMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P141](prs/P141.md) | SysML states transitions branch batch 8: EffectBehaviorUsage_4, EffectBehaviorUsage_5, EffectBehaviorUsage and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P142](prs/P142.md) | SysML views viewpoints rendering branch batch 1: ViewDefinition, ViewDefinitionBody, ViewDefinitionBodyItems and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P143](prs/P143.md) | SysML views viewpoints rendering branch batch 2: ViewBodyItems, ViewBodyItem, ExposePrefix and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P144](prs/P144.md) | Complete reference target-kind dispatch beyond Type/Element | C01, C02, P15 | #37, #47, #49, #50 |
| [P145](prs/P145.md) | Compute declared/short/alias scope names and local visibility | C01, C02, P15 | #37, #47, #49, #50 |
| [P146](prs/P146.md) | Resolve private/protected membership access | C01, C02, P15 | #37, #47, #49, #50 |
| [P147](prs/P147.md) | Resolve wildcard and recursive imports with exclusions/reexports | C01, C02, P15 | #37, #47, #49, #50 |
| [P148](prs/P148.md) | Resolve inherited/redefined and specialized member scopes | C01, C02, P15 | #37, #47, #49, #50 |
| [P149](prs/P149.md) | Link chained features and invocation/return-result references | C01, C02, P15 | #37, #47, #49, #50 |
| [P15](prs/P15.md) | Expose KerML/SysML root parsing through an immutable family registry | P14 | #23, #25, #35, #40, #41 |
| [P150](prs/P150.md) | Link across explicit document/resource snapshots | C01, C02, P15 | #37, #47, #49, #50 |
| [P151](prs/P151.md) | Freeze complete AST-to-EMF translator classifier coverage and emit bounded import children | C01, C02, P25 | #52, #55, #64 |
| [P152](prs/P152.md) | Publish complete text-import deltas atomically | C01, C02, P151, P144, P145, P146, P147, P148, P149, P150 | #52, #55, #64 |
| [P153](prs/P153.md) | Implement expression create/edit with original text and diagnostics | C01, C02, P151, P144, P145, P146, P147, P148, P149, P150 | #52, #55, #64 |
| [P154](prs/P154.md) | Qualify text→model→native JSON→reload interoperability | C01, C02, P151, P144, P145, P146, P147, P148, P149, P150 | #52, #55, #64 |
| [P155](prs/P155.md) | Text export dispatch batch 1: caseAcceptActionUsage, caseActionDefinition | C01, C02 | #56, #52, #49 |
| [P156](prs/P156.md) | Text export dispatch batch 2: caseAllocationUsage, caseAssertConstraintUsage | C01, C02 | #56, #52, #49 |
| [P157](prs/P157.md) | Text export dispatch batch 3: caseCollectExpression, caseComment | C01, C02 | #56, #52, #49 |
| [P158](prs/P158.md) | Text export dispatch batch 4: caseDocumentation, caseEnumerationDefinition | C01, C02 | #56, #52, #49 |
| [P159](prs/P159.md) | Text export dispatch batch 5: caseFeatureReferenceExpression, caseFlowUsage | C01, C02 | #56, #52, #49 |
| [P16](prs/P16.md) | Add expected-token diagnostics without recovery success claims | P15 | #23, #25, #35, #40, #41 |
| [P160](prs/P160.md) | Text export dispatch batch 6: caseItemDefinition, caseItemUsage | C01, C02 | #56, #52, #49 |
| [P161](prs/P161.md) | Text export dispatch batch 7: caseLiteralExpression, caseLiteralInfinity | C01, C02 | #56, #52, #49 |
| [P162](prs/P162.md) | Text export dispatch batch 8: caseMetadataAccessExpression, caseMetadataDefinition | C01, C02 | #56, #52, #49 |
| [P163](prs/P163.md) | Text export dispatch batch 9: caseOccurrenceUsage, caseOperatorExpression | C01, C02 | #56, #52, #49 |
| [P164](prs/P164.md) | Text export dispatch batch 10: casePerformActionUsage, casePortDefinition | C01, C02 | #56, #52, #49 |
| [P165](prs/P165.md) | Text export dispatch batch 11: caseRequirementDefinition, caseRequirementUsage | C01, C02 | #56, #52, #49 |
| [P166](prs/P166.md) | Text export dispatch batch 12: caseStakeholderMembership, caseStateDefinition | C01, C02 | #56, #52, #49 |
| [P167](prs/P167.md) | Text export dispatch batch 13: caseTextualRepresentation, caseTransitionFeatureMembership | C01, C02 | #56, #52, #49 |
| [P168](prs/P168.md) | Text export dispatch batch 14: caseUseCaseUsage, caseVerificationCaseUsage | C01, C02 | #56, #52, #49 |
| [P169](prs/P169.md) | Complete native textual name deresolution and escaping | C01, C02 | #56 |
| [P17](prs/P17.md) | Recover missing delimiters and resynchronize bounded incomplete input | P16 | #23, #25, #35, #40, #41 |
| [P170](prs/P170.md) | Close non-dispatch textual helpers and residual exporters | C01, C02 | #56 |
| [P171](prs/P171.md) | Qualify complete textual import/export corpus and residual differences | C01, C02, P155, P156, P157, P158, P159, P160, P161, P162, P163, P164, P165, P166, P167, P168 | #56 |
| [P172](prs/P172.md) | Freeze all standard-library packages, source/artifact bytes and dependencies | C01, C02, P18 | #37, #47, #49 |
| [P173](prs/P173.md) | Load selected libraries into explicit immutable resource registries | C01, C02, P18, P172 | #37, #47, #49 |
| [P174](prs/P174.md) | Resolve implicit base features and standard-library names | C01, C02, P18, P173 | #37, #47, #49 |
| [P175](prs/P175.md) | Rebuild library/workspace indexes after document changes | C01, C02, P18, P174 | #37, #47, #49 |
| [P176](prs/P176.md) | Publish versioned text diagnostics and recovered editor states | C01, C02, P18, P175 | #51, #52, #56, #57 |
| [P177](prs/P177.md) | Complete names, members and import-context suggestions | C01, C02, P18, P175 | #51, #52, #56, #57 |
| [P178](prs/P178.md) | Show scoped semantic hover and documentation | C01, C02, P18, P175 | #51, #52, #56, #57 |
| [P179](prs/P179.md) | Navigate to declaration and enumerate scoped references | C01, C02, P18, P175 | #51, #52, #56, #57 |
| [P18](prs/P18.md) | Version incremental document edits and invalidate stale CST/AST snapshots | P07, P17 | #23, #25, #35, #40, #41 |
| [P180](prs/P180.md) | Apply atomic multi-reference symbol renames | C01, C02, P18, P175 | #51, #52, #56, #57 |
| [P181](prs/P181.md) | Produce native-compatible semantic token classifications | C01, C02, P18, P175 | #51, #52, #56, #57 |
| [P182](prs/P182.md) | Format admitted syntax without semantic or comment loss | C01, C02, P18, P175 | #51, #52, #56, #57 |
| [P183](prs/P183.md) | Bind text import/export/expression edits to actual UI state | C01, C02, P18, P175 | #51, #52, #56, #57 |
| [P184](prs/P184.md) | Close every grammar/AST/native-caller obligation with an evidence matrix | C01, C02, C03, P01, P02, P03, P04, P05, P06, P07, P08, P09, P10, P11, P12, P13, P14, P15, P16, P17, P18, P19, P20, P21, P22, P23, P24, P25, P26, P27, P28, P29, P30, P31, P32, P33, P34, P35, P36, P37, P38, P39, P40, P41, P42, P43, P44, P45, P46, P47, P48, P49, P50, P51, P52, P53, P54, P55, P56, P57, P58, P59, P60, P61, P62, P63, P64, P65, P66, P67, P68, P69, P70, P71, P72, P73, P74, P75, P76, P77, P78, P79, P80, P81, P82, P83, P84, P85, P86, P87, P88, P89, P90, P91, P92, P93, P94, P95, P96, P97, P98, P99, P100, P101, P102, P103, P104, P105, P106, P107, P108, P109, P110, P111, P112, P113, P114, P115, P116, P117, P118, P119, P120, P121, P122, P123, P124, P125, P126, P127, P128, P129, P130, P131, P132, P133, P134, P135, P136, P137, P138, P139, P140, P141, P142, P143, P144, P145, P146, P147, P148, P149, P150, P151, P152, P153, P154, P155, P156, P157, P158, P159, P160, P161, P162, P163, P164, P165, P166, P167, P168, P169, P170, P171, P172, P173, P174, P175, P176, P177, P178, P179, P180, P181, P182, P183 |  |
| [P19](prs/P19.md) | Attach hidden trivia and semantic Comment CST to user-visible source maps | P02, P05, P07, P15 | #23, #25, #35, #40, #41 |
| [P20](prs/P20.md) | Expand the finite caller to untyped usages and usage bodies | C01, C02 | #23, #25, #35, #40, #41 |
| [P21](prs/P21.md) | Run the actual loop-member postprocessor through parsed loop fixtures | P15 | #23, #25, #35, #40, #41 |
| [P22](prs/P22.md) | Finalize parsed imports through the source postprocessor | P15 | #23, #25, #35, #40, #41 |
| [P23](prs/P23.md) | Create missing transition parameters with exact implicit-node order | P15 | #23, #25, #35, #40, #41 |
| [P24](prs/P24.md) | Create missing succession ends with exact implicit-node order | P15 | #23, #25, #35, #40, #41 |
| [P25](prs/P25.md) | Reconcile generated AST declarations and operation coverage against embedded source | C01, C02, P14, P15 | #23, #25, #35, #40, #41 |
| [P26](prs/P26.md) | SysML actions control flow branch batch 1: EmptySuccessionMember, ActionDefinition, ActionBody and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P27](prs/P27.md) | SysML actions control flow branch batch 2: ActionBodyItem | C01, C02, P12, P13, P15, P26 | #40, #41, #50, #52 |
| [P28](prs/P28.md) | SysML actions control flow branch batch 3: ActionBodyItem | C01, C02, P12, P13, P15, P27 | #40, #41, #50, #52 |
| [P29](prs/P29.md) | SysML actions control flow branch batch 4: ActionNodeMember, TargetSuccessionMember, GuardedSuccessionMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P30](prs/P30.md) | SysML actions control flow branch batch 5: ActionNodeUsageDeclaration, ActionNodePrefix, AcceptNode and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P31](prs/P31.md) | SysML actions control flow branch batch 6: TriggerFeatureValue, TriggerExpression, SendNode and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P32](prs/P32.md) | SysML actions control flow branch batch 7: TerminateNode, IfNode, ActionBodyParameterMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P33](prs/P33.md) | SysML actions control flow branch batch 8: ForVariableDeclarationMember, ForVariableDeclaration, ControlNode and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P34](prs/P34.md) | SysML actions control flow branch batch 9: ActionNodeBody, ActionNodeItems, ActionTargetSuccession and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P35](prs/P35.md) | KerML annotations metadata comments branch batch 1: OwnedAnnotation, AnnotatingElement, AnnotatingMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P36](prs/P36.md) | KerML annotations metadata comments branch batch 2: MetadataFeature, MetadataFeatureDeclaration, MetadataBody and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P37](prs/P37.md) | KerML annotations metadata comments branch batch 3: TextualAnnotatingElementBody, Comment, Documentation and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P38](prs/P38.md) | SysML annotations metadata comments branch batch 1: OwnedAnnotation, AnnotatingMember, AnnotatingElement and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P39](prs/P39.md) | SysML annotations metadata comments branch batch 2: MetadataUsage, MetadataUsageDeclaration, MetadataBody and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P40](prs/P40.md) | SysML annotations metadata comments branch batch 3: Annotation, TextualAnnotatingElementBody, Comment and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P41](prs/P41.md) | SysML annotations metadata comments branch batch 4: MetadataAccessExpression | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P42](prs/P42.md) | KerML calculations constraints branch batch 1: Predicate, BooleanExpression, Invariant | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P43](prs/P43.md) | SysML calculations constraints branch batch 1: CalculationDefinition, CalculationBody, CalculationBodyPart and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P44](prs/P44.md) | SysML calculations constraints branch batch 2: ConstraintUsageDeclaration | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P45](prs/P45.md) | KerML connections interfaces flows branch batch 1: Connector, ConnectorDeclaration, BinaryConnectorDeclaration and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P46](prs/P46.md) | KerML connections interfaces flows branch batch 2: ItemFlow, SuccessionItemFlow, ItemFlowDeclaration and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P47](prs/P47.md) | KerML connections interfaces flows branch batch 3: ItemFlowEnd, ItemFlowEndSubsetting, ItemFlowFeatureMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P48](prs/P48.md) | SysML connections interfaces flows branch batch 1: ConnectorEndMember, ConnectorEnd, BindingConnectorAsUsage and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P49](prs/P49.md) | SysML connections interfaces flows branch batch 2: FlowConnectionDefinition, Message, MessageDeclaration and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P50](prs/P50.md) | SysML connections interfaces flows branch batch 3: ItemFeatureMember, ItemFeature, PayloadFeature and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P51](prs/P51.md) | SysML connections interfaces flows branch batch 4: FlowFeature, FlowRedefinition, InterfaceDefinition and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P52](prs/P52.md) | SysML connections interfaces flows branch batch 5: InterfaceOccurrenceUsageMember, InterfaceOccurrenceUsageElement, DefaultInterfaceEnd and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P53](prs/P53.md) | SysML connections interfaces flows branch batch 6: InterfaceEndMember, InterfaceEnd, AllocationDefinition and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P54](prs/P54.md) | KerML definitions usages structures branch batch 1: SpecializationPart, ConjugationPart, TypeRelationshipPart and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P55](prs/P55.md) | KerML definitions usages structures branch batch 2: SuperclassingPart, ClassifierConjugationPart, EndFeatureDeclarationPart and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P56](prs/P56.md) | KerML definitions usages structures branch batch 3: FeatureSpecializationPart, MultiplicityPart, FeatureConjugationPart and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P57](prs/P57.md) | SysML definitions usages structures branch batch 1: PortionKind, Variation, Individual and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P58](prs/P58.md) | SysML definitions usages structures branch batch 2: FeatureSpecializationPart, MultiplicityPart, BasicDefinitionPrefix and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P59](prs/P59.md) | SysML definitions usages structures branch batch 3: DefinitionBodyItems, DefinitionBodyItem, DefinitionMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P60](prs/P60.md) | SysML definitions usages structures branch batch 4: OccurrenceUsageMember, StructureUsageMember, BehaviorUsageMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P61](prs/P61.md) | SysML definitions usages structures branch batch 5: UsageSuffix, UsageDeclaration, UsageCompletion and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P62](prs/P62.md) | SysML definitions usages structures branch batch 6: NonOccurrenceUsageElement, OccurrenceUsageElement, StructureUsageElement and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P63](prs/P63.md) | SysML definitions usages structures branch batch 7: AttributeUsage, EnumerationDefinition, EnumerationBody and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P64](prs/P64.md) | SysML definitions usages structures branch batch 8: OccurrenceDefinition, IndividualDefinition, BasicOccurrenceUsagePrefix and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P65](prs/P65.md) | SysML definitions usages structures branch batch 9: ItemDefinition, ItemUsage, PartDefinition and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P66](prs/P66.md) | SysML definitions usages structures branch batch 10: SuccessionAsUsage | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P67](prs/P67.md) | KerML expressions literals operators branch batch 1: MultiplicityExpressionMember, ResultExpressionMember, Expression and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P68](prs/P68.md) | KerML expressions literals operators branch batch 2: ConcreteConditionalExpression, ConditionalExpression, NullCoalescingExpression and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P69](prs/P69.md) | KerML expressions literals operators branch batch 3: OrExpressionReference, OrExpressionMember, OrExpression and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P70](prs/P70.md) | KerML expressions literals operators branch batch 4: AndExpression, EqualityExpressionReference, EqualityExpressionMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P71](prs/P71.md) | KerML expressions literals operators branch batch 5: ClassificationExpression | C01, C02, P12, P13, P15, P70 | #40, #41, #50, #52 |
| [P72](prs/P72.md) | KerML expressions literals operators branch batch 6: ClassificationExpression | C01, C02, P12, P13, P15, P71 | #40, #41, #50, #52 |
| [P73](prs/P73.md) | KerML expressions literals operators branch batch 7: RelationalExpression, RangeExpression, AdditiveExpression and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P74](prs/P74.md) | KerML expressions literals operators branch batch 8: ExponentiationExpression, UnaryExpression, ExtentExpression | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P75](prs/P75.md) | KerML expressions literals operators branch batch 9: PrimaryExpression | C01, C02, P12, P13, P15, P74 | #40, #41, #50, #52 |
| [P76](prs/P76.md) | KerML expressions literals operators branch batch 10: PrimaryExpression | C01, C02, P12, P13, P15, P75 | #40, #41, #50, #52 |
| [P77](prs/P77.md) | KerML expressions literals operators branch batch 11: BaseExpression, BodyExpression, ExpressionBodyMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P78](prs/P78.md) | KerML expressions literals operators branch batch 12: Argument, NamedArgumentList, NamedArgumentMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P79](prs/P79.md) | KerML expressions literals operators branch batch 13: LiteralInfinity, LiteralExpression, NullExpression and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P80](prs/P80.md) | KerML expressions literals operators branch batch 14: TypeResultMember, TypeReferenceFeature, ReferenceTyping and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P81](prs/P81.md) | KerML expressions literals operators branch batch 15: FeatureChain, FeatureChainPrefix, OwnedFeatureChaining and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P82](prs/P82.md) | SysML expressions literals operators branch batch 1: MultiplicityExpressionMember, ArgumentExpressionMember, ArgumentExpression and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P83](prs/P83.md) | SysML expressions literals operators branch batch 2: OwnedExpression, OwnedExpressionReference, ConcreteConditionalExpression and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P84](prs/P84.md) | SysML expressions literals operators branch batch 3: ImpliesExpression, OrExpressionReference, OrExpressionMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P85](prs/P85.md) | SysML expressions literals operators branch batch 4: XorExpression, AndExpression, EqualityExpressionReference and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P86](prs/P86.md) | SysML expressions literals operators branch batch 5: EqualityExpression | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P87](prs/P87.md) | SysML expressions literals operators branch batch 6: ClassificationExpression | C01, C02, P12, P13, P15, P86 | #40, #41, #50, #52 |
| [P88](prs/P88.md) | SysML expressions literals operators branch batch 7: ClassificationExpression | C01, C02, P12, P13, P15, P87 | #40, #41, #50, #52 |
| [P89](prs/P89.md) | SysML expressions literals operators branch batch 8: RelationalExpression, RangeExpression, AdditiveExpression and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P90](prs/P90.md) | SysML expressions literals operators branch batch 9: ExponentiationExpression, UnaryExpression, ExtentExpression | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P91](prs/P91.md) | SysML expressions literals operators branch batch 10: PrimaryExpression | C01, C02, P12, P13, P15, P90 | #40, #41, #50, #52 |
| [P92](prs/P92.md) | SysML expressions literals operators branch batch 11: PrimaryExpression | C01, C02, P12, P13, P15, P91 | #40, #41, #50, #52 |
| [P93](prs/P93.md) | SysML expressions literals operators branch batch 12: BaseExpression, BodyExpression, ExpressionBodyMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P94](prs/P94.md) | SysML expressions literals operators branch batch 13: Argument, NamedArgumentList, NamedArgumentMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P95](prs/P95.md) | SysML expressions literals operators branch batch 14: LiteralInfinity, LiteralExpression, NullExpression and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P96](prs/P96.md) | SysML expressions literals operators branch batch 15: TypeResultMember, TypeReferenceFeature, ReferenceTyping and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P97](prs/P97.md) | SysML expressions literals operators branch batch 16: FeatureChain, FeatureChainPrefix, OwnedFeatureChaining and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P98](prs/P98.md) | KerML features typing multiplicity branch batch 1: NonFeatureMember, NamespaceFeatureMember, ImportedReference and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [P99](prs/P99.md) | KerML features typing multiplicity branch batch 2: FeaturePrefix, Feature, OwnedCrossingFeatureMember and siblings | C01, C02, P12, P13, P15 | #40, #41, #50, #52 |
| [U01](prs/U01.md) | Move DOM binding and browser event dispatch into Rust/WASM | C03 | #65 |
| [U02](prs/U02.md) | Move canvas scheduling, measurements and viewport state into Rust | U01 | #65 |
| [U03](prs/U03.md) | Project home, templates and application routing | U01, B08, B23 |  |
| [U04](prs/U04.md) | Explorer navigation, document actions and search | U03, B19 |  |
| [U05](prs/U05.md) | Details forms, typed references and feature help | U04, B21 | #61, #62 |
| [U06](prs/U06.md) | Connected text import and positioned validation | U05 | #52, #59, #63 |
| [U07](prs/U07.md) | Real file download/upload and archive reload | U06, B11, B12 | #55, #56, #63 |
| [U08](prs/U08.md) | Client history and remote command result integration | U03, B13, B17, B18 |  |
| [U09](prs/U09.md) | Diagram representation lifecycle and selection | U02, U04, B20 |  |
| [U10](prs/U10.md) | Rust layered layout execution and worker lifecycle | U02, U09 |  |
| [U11](prs/U11.md) | Ports, edge routing and reconnect interactions | U10 |  |
| [U12](prs/U12.md) | Rust package custom node appearance and interactions | U09, U10 |  |
| [U13](prs/U13.md) | Rust note custom node appearance and interactions | U09, U10 |  |
| [U14](prs/U14.md) | Rust view frame custom node appearance and interactions | U09, U10 |  |
| [U15](prs/U15.md) | Rust imported package custom node appearance and interactions | U09, U10 |  |
| [U16](prs/U16.md) | Direct-edit labels and expression modal | U05, U09, B14 |  |
| [U17](prs/U17.md) | General View node/edge/tool registry and source assignment | U09, U11 | #48, #54 |
| [U18](prs/U18.md) | Interconnection View and connection endpoint workflows | U17, V001, V002, V003, V004, V005, V006, V007, V008, V009, V010, V011, V012, V013, V014, V015, V016, V017, V018, V019, V020, V021, V022, V023, V024, V025, V026, V027, V028, V029, V030, V031, V032, V033, V034, V035, V036, V037, V038, V039, V040, V041, V042, V043, V044 |  |
| [U19](prs/U19.md) | Action Flow View and control-flow tools | U17, V001, V002, V003, V004, V005, V006, V007, V008, V009, V010, V011, V012, V013, V014, V015, V016, V017, V018, V019, V020, V021, V022, V023, V024, V025, V026, V027, V028, V029, V030, V031, V032, V033, V034, V035, V036, V037, V038, V039, V040, V041, V042, V043, V044 |  |
| [U20](prs/U20.md) | State Transition View and transition tools | U17, U16, V001, V002, V003, V004, V005, V006, V007, V008, V009, V010, V011, V012, V013, V014, V015, V016, V017, V018, V019, V020, V021, V022, V023, V024, V025, V026, V027, V028, V029, V030, V031, V032, V033, V034, V035, V036, V037, V038, V039, V040, V041, V042, V043, V044 |  |
| [U21](prs/U21.md) | Requirements table and requirement authoring | U05, U09, B22 |  |
| [U22](prs/U22.md) | Validation panel and positioned problem navigation | U06, U09, B22 |  |
| [U23](prs/U23.md) | Query, related elements and cross-project search | U04, U09 |  |
| [U24](prs/U24.md) | Library import, version selection and publication UX | U03, U07, B24 |  |
| [U25](prs/U25.md) | Omnibox, keyboard shortcuts, menus and extension composition | U03, U09 |  |
| [U26](prs/U26.md) | Collaboration, reconnect and multi-client editor states | U08, U09, B15, B16 |  |
| [U27](prs/U27.md) | Accessible responsive application qualification | U04, U05, U07, U08, U18, U19, U20, U21, U22, U23, U24, U25, U26 |  |
| [U28](prs/U28.md) | Cross-view performance and lifecycle qualification | U27, U10 |  |
| [U29](prs/U29.md) | Qualify native documented capacity and remove prototype-only limits | U28 |  |
| [V001](prs/V001.md) | Port concrete diagram providers: AbstractAllocateEdgeDescriptionProvider, AbstractBindingConnectorAsUsageEdgeDescriptionProvider, AbstractDefinitionOwnedUsageEdgeDescriptionProvider, AbstractDependencyEdgeDescriptionProvider | U17 |  |
| [V002](prs/V002.md) | Port concrete diagram providers: AbstractEdgeDescriptionProvider, AbstractFeatureTypingEdgeDescriptionProvider, AbstractFeatureValueEdgeDescriptionProvider, AbstractFlowUsageEdgeDescriptionProvider | U17 |  |
| [V003](prs/V003.md) | Port concrete diagram providers: AbstractInterfaceUsageEdgeDescriptionProvider, AbstractRedefinitionEdgeDescriptionProvider, AbstractSubclassificationEdgeDescriptionProvider, AbstractSubsettingEdgeDescriptionProvider | U17 |  |
| [V004](prs/V004.md) | Port concrete diagram providers: AbstractSuccessionEdgeDescriptionProvider, AbstractTransitionEdgeDescriptionProvider, AbstractUsageNestedUsageEdgeDescriptionProvider, AnnotationEdgeDescriptionProvider | U17 |  |
| [V005](prs/V005.md) | Port concrete diagram providers: AbstractCompartmentNodeDescriptionProvider, AbstractControlNodeActionNodeDescriptionProvider, AbstractDefinitionNodeDescriptionProvider, AbstractFakeNodeDescriptionProvider | U17 |  |
| [V006](prs/V006.md) | Port concrete diagram providers: AbstractItemUsageBorderNodeDescriptionProvider, AbstractNodeDescriptionProvider, AbstractPackageNodeDescriptionProvider, AbstractPortUsageBorderNodeDescriptionProvider | U17 |  |
| [V007](prs/V007.md) | Port concrete diagram providers: AbstractReferenceUsageBorderNodeDescriptionProvider, AbstractUsageNodeDescriptionProvider, ActionFlowCompartmentNodeDescriptionProvider, AnnotatingNodeDescriptionProvider | U17 |  |
| [V008](prs/V008.md) | Port concrete diagram providers: AssumeConstraintCompartmentItemNodeDescription, AssumeConstraintCompartmentNodeDescription, CompartmentItemNodeDescriptionProvider, DecisionActionNodeDescriptionProvider | U17 |  |
| [V009](prs/V009.md) | Port concrete diagram providers: DoneActionNodeDescriptionProvider, DoneStateNodeDescriptionProvider, ExhibitStateInheritedCompartmentItemNodeDescriptionProvider, ForkActionNodeDescriptionProvider | U17 |  |
| [V010](prs/V010.md) | Port concrete diagram providers: FramedConcernCompartmentItemNodeDescription, FramedConcernCompartmentNodeDescription, ImportedPackageNodeDescriptionProvider, InheritedCompartmentItemNodeDescriptionProvider | U17 |  |
| [V011](prs/V011.md) | Port concrete diagram providers: InheritedItemUsageBorderNodeDescriptionProvider, InterconnectionCompartmentNodeDescriptionProvider, ItemUsageBorderNodeDescriptionProvider, JoinActionNodeDescriptionProvider | U17 |  |
| [V012](prs/V012.md) | Port concrete diagram providers: MergeActionNodeDescriptionProvider, RequireConstraintCompartmentItemNodeDescription, RequireConstraintCompartmentNodeDescription, SatisfyRequirementCompartmentItemNodeDescription | U17 |  |
| [V013](prs/V013.md) | Port concrete diagram providers: SatisfyRequirementCompartmentNodeDescription, SatisfyRequirementInheritedCompartmentItemNodeDescription, StartActionNodeDescriptionProvider, StartStateNodeDescriptionProvider | U17 |  |
| [V014](prs/V014.md) | Port concrete diagram providers: StateTransitionCompartmentNodeDescriptionProvider, StatesCompartmentItemNodeDescriptionProvider, StatesCompartmentNodeDescriptionProvider, AbstractViewNodeToolSectionSwitch | U17 |  |
| [V015](prs/V015.md) | Port concrete diagram providers: AbstractViewNodeToolsWithoutSectionSwitch, ViewEdgeToolService, ViewEdgeToolSwitch, NodeToolSectionNameSwitch | U17 |  |
| [V016](prs/V016.md) | Port concrete diagram providers: AbstractCompartmentNodeToolProvider, AbstractFreeFormCompartmentNodeToolProvider, AcceptActionNodeToolProvider, AcceptActionPayloadNodeToolProvider | U17 |  |
| [V017](prs/V017.md) | Port concrete diagram providers: ActionFlowCompartmentNodeToolProvider, ActorCompartmentNodeToolProvider, AnnotatingElementOnRelationshipNodeToolProvider, AssignmentActionNodeToolProvider | U17 |  |
| [V018](prs/V018.md) | Port concrete diagram providers: AssumeConstraintNodeToolProvider, CompartmentNodeToolProvider, ConnectionDefinitionEndCompartmentNodeToolProvider, DecisionActionNodeToolProvider | U17 |  |
| [V019](prs/V019.md) | Port concrete diagram providers: DoneActionNodeToolProvider, DoneStateNodeToolProvider, ExhibitStateNodeToolProvider, FlowNodeToolProvider | U17 |  |
| [V020](prs/V020.md) | Port concrete diagram providers: ForkActionNodeToolProvider, FramedConcernNodeToolProvider, InterfaceDefinitionEndCompartmentNodeToolProvider, JoinActionNodeToolProvider | U17 |  |
| [V021](prs/V021.md) | Port concrete diagram providers: MergeActionNodeToolProvider, NamespaceImportNodeToolProvider, ObjectiveDocumentationNodeToolProvider, ObjectiveRequirementCompartmentNodeToolProvider | U17 |  |
| [V022](prs/V022.md) | Port concrete diagram providers: OccurrenceUsageSnapshotNodeToolProvider, OccurrenceUsageTimesliceNodeToolProvider, ParameterCompartmentNodeToolProvider, PartUsageBindingConnectorAsUsageNodeToolProvider | U17 |  |
| [V023](prs/V023.md) | Port concrete diagram providers: PartUsageFeatureTypingNodeToolProvider, PartUsageFlowConnectionNodeToolProvider, PartUsageInterfaceNodeToolProvider, PartUsageSubsettingNodeToolProvider | U17 |  |
| [V024](prs/V024.md) | Port concrete diagram providers: PerformActionNodeToolProvider, ReferenceUsageCompartmentNodeToolProvider, ReferenceUsageNodeToolProvider, RequireConstraintNodeToolProvider | U17 |  |
| [V025](prs/V025.md) | Port concrete diagram providers: SatisfyNodeToolProvider, SatisfyRequirementNodeToolProvider, SetAsCompositeToolProvider, SetAsRefToolProvider | U17 |  |
| [V026](prs/V026.md) | Port concrete diagram providers: SetAsViewToolProvider, StakeholdersCompartmentNodeToolProvider, StartActionNodeToolProvider, StartStateNodeToolProvider | U17 |  |
| [V027](prs/V027.md) | Port concrete diagram providers: StateSubactionNodeToolProvider, StateTransitionCompartmentNodeToolProvider, SubjectCompartmentNodeToolProvider, ViewNodeAsToolProvider | U17 |  |
| [V028](prs/V028.md) | Port concrete diagram providers: AllocateEdgeDescriptionProvider, BindingConnectorAsUsageEdgeDescriptionProvider, ConnectionUsageEdgeDescriptionProvider, DefinitionOwnedUsageEdgeDescriptionProvider | U17 |  |
| [V029](prs/V029.md) | Port concrete diagram providers: DependencyEdgeDescriptionProvider, FeatureTypingEdgeDescriptionProvider, FeatureValueEdgeDescriptionProvider, FlowUsageEdgeDescriptionProvider | U17 |  |
| [V030](prs/V030.md) | Port concrete diagram providers: FrameConcernEdgeDescriptionProvider, InterfaceUsageEdgeDescriptionProvider, NestedActorEdgeDescriptionProvider, NestedStakeholderEdgeDescriptionProvider | U17 |  |
| [V031](prs/V031.md) | Port concrete diagram providers: NestedSubjectEdgeDescriptionProvider, RedefinitionEdgeDescriptionProvider, ReferenceSubsettingEdgeDescriptionProvider, RequirementConstraintEdgeDescriptionProvider | U17 |  |
| [V032](prs/V032.md) | Port concrete diagram providers: RequirementDerivationEdgeDescriptionProvider, SatisfyRequirementEdgeDescriptionProvider, SubclassificationEdgeDescriptionProvider, SubsettingEdgeDescriptionProvider | U17 |  |
| [V033](prs/V033.md) | Port concrete diagram providers: SuccessionEdgeDescriptionProvider, TransitionEdgeDescriptionProvider, UsageNestedUsageEdgeDescriptionProvider, ActionDefinitionParametersCompartmentNodeDescriptionProvider | U17 |  |
| [V034](prs/V034.md) | Port concrete diagram providers: ActionItemNodeDescriptionProvider, ActionUsageParametersCompartmentNodeDescriptionProvider, ActorNodeDescriptionProvider, ActorsCompartmentItemNodeDescriptionProvider | U17 |  |
| [V035](prs/V035.md) | Port concrete diagram providers: AllocationDefinitionEndsCompartmentItemNodeDescriptionProvider, AllocationDefinitionEndsCompartmentNodeDescriptionProvider, CaseDefinitionActorsCompartmentNodeDescriptionProvider, CaseDefinitionObjectiveRequirementCompartmentNodeDescriptionProvider | U17 |  |
| [V036](prs/V036.md) | Port concrete diagram providers: CaseDefinitionSubjectCompartmentNodeDescriptionProvider, CaseUsageActorsCompartmentNodeDescriptionProvider, CaseUsageObjectiveRequirementCompartmentNodeDescriptionProvider, CaseUsageSubjectCompartmentNodeDescriptionProvider | U17 |  |
| [V037](prs/V037.md) | Port concrete diagram providers: CompartmentNodeDescriptionProvider, ConnectionDefinitionEndsCompartmentItemNodeDescriptionProvider, ConnectionDefinitionEndsCompartmentNodeDescriptionProvider, DefinitionNodeDescriptionProvider | U17 |  |
| [V038](prs/V038.md) | Port concrete diagram providers: FakeNodeDescriptionProvider, InheritedPortUsageBorderNodeDescriptionProvider, InheritedReferenceUsageBorderNodeDescriptionProvider, InterfaceDefinitionEndsCompartmentItemNodeDescriptionProvider | U17 |  |
| [V039](prs/V039.md) | Port concrete diagram providers: InterfaceDefinitionEndsCompartmentNodeDescriptionProvider, ObjectiveDocumentationCompartmentItemNodeDescription, PackageNodeDescriptionProvider, PerformActionsCompartmentItemNodeDescriptionProvider | U17 |  |
| [V040](prs/V040.md) | Port concrete diagram providers: PerformActionsCompartmentNodeDescriptionProvider, PerformActionsInheritedCompartmentItemNodeDescriptionProvider, PortDefinitionOwnedItemCompartmentItemNodeDescriptionProvider, PortUsageBorderNodeDescriptionProvider | U17 |  |
| [V041](prs/V041.md) | Port concrete diagram providers: PortUsageNestedItemCompartmentItemNodeDescriptionProvider, ReferenceUsageBorderNodeDescriptionProvider, RequirementDefinitionActorsCompartmentNodeDescriptionProvider, RequirementDefinitionStakeholdersCompartmentNodeDescriptionProvider | U17 |  |
| [V042](prs/V042.md) | Port concrete diagram providers: RequirementDefinitionSubjectCompartmentNodeDescriptionProvider, RequirementUsageActorsCompartmentNodeDescriptionProvider, RequirementUsageStakeholdersCompartmentNodeDescriptionProvider, RequirementUsageSubjectCompartmentNodeDescriptionProvider | U17 |  |
| [V043](prs/V043.md) | Port concrete diagram providers: SDVNodeDescriptionProviderSwitch, StakeholderNodeDescriptionProvider, SubjectNodeDescriptionProvider, UsageNodeDescriptionProvider | U17 |  |
| [V044](prs/V044.md) | Port concrete diagram providers: ViewUsageNodeDescriptionProvider, SDVNodeToolSectionSwitch, SDVNodeToolsWithoutSectionSwitch | U17 |  |
