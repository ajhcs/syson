# AST and value conversion implementation plan

This is a source-grounded plan, not an implementation or a runtime parity result. The static correspondence checker proves equality of the two embedded grammar declaration objects only. It does not establish correspondence of the candidate service implementations to the embedded runtime. The present Rust recognizer's CST and `SemanticStatus::NotPerformed` boundary must remain explicit until these layers are implemented and independently tested.

All references below identify retained source data. Candidate sources are pinned at `1326f234cae23b0e4777d5a48a069b17e9e332f1`; SysON sources and embedded runtime are pinned at `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`. Full source texts and SHA256 values are in `semantic-source-documents.jsonl`, `syson-import-adapter-source.jsonl`, and `generated-grammar-documents.jsonl`. Embedded line references refer to `backend/application/syson-sysml-import/src/main/resources/syside-cli.js`, SHA256 `47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd`.

## Work order and acceptance boundary

| Stage | Concrete output | Evidence required before marking implemented |
| --- | --- | --- |
| 1. Construction contract | Rust AST arena, explicit absent/null values, parent slots and CST associations; per-construct disposition | Immutable source trace and native edge cases for assignments, actions, fragments, defaults and references |
| 2. Primitive conversion | Rule-specific conversion and raw lexeme retention | Source-derived edge cases and an authorized pinned runtime oracle for JavaScript number/escape behavior |
| 3. Syside construction hooks | Synthetic nodes, retyping, parenting and child order | Every registered hook and derived-type dispatch tested, including missing/default operands |
| 4. Reference and semantic model | Reference chains, scope/linking, metamodel initialization, naming and validation | Independent service/dependency inventory and pinned fixtures; unresolved references represented explicitly |
| 5. CLI serialization | Exact JSON field selection and value projection | Byte/structural comparisons against pinned `dump` outputs, with declared allowed differences |
| 6. SysON import adaptation | Classifier/attribute/reference translations into Rust model | Source-derived adapter cases plus authorized upstream import differentials |

Each stage needs cancellation and allocation/work budgets. A failure must retain its layer and source span. Syntax recognition, recovered/partial AST construction, linking, semantic validation and import success are distinct outcomes. A generic CST is insufficient evidence for any later stage. Do not change existing per-rule UNPORTED statuses merely because construction metadata or the obligation ledger exists.

`ast-construction-obligations.jsonl` retains all 783 Assignment, 61 Action and 4 CrossReference occurrences from `grammar-features.jsonl`: 848 declaration locations, all UNPORTED. These counts include declarations repeated across the KerML and SysML grammar roots; they are neither distinct runtime code paths nor tests nor coverage. `relative_path` points inside each containing rule declaration. To regenerate, filter the existing feature inventory for those three `kind` values, preserve each declaration verbatim, and add the same status/evidence fields. No source parser needs to run.

## Construction engine

The embedded Langium 1.2.1 parser module begins at bundle line 40939. Its construction and assignment behavior is visible around lines 41015–41190. Implement this contract before grammar-wide AST claims:

- Rule entry creates a frame with `$type`; datatype rules accumulate a separate string value. Tokens inserted by recovery are excluded from normal token assignment. Recovery and the caught-exception path can still produce a constructed partial object; success cannot be inferred solely from a nonempty AST.
- A keyword assignment uses raw token text. A terminal assignment invokes the value converter. An unassigned token in a datatype rule contributes converted text without hidden whitespace. Unassigned subrules contribute datatype strings or merge objects. A subrule assignment is performed only when its composite CST has nonzero length.
- `=` replaces a feature; `?=` sets true on matched presence; `+=` initializes an array if necessary and appends. Crossreferences build a reference when the assigned value is a string. A Rust reference must retain raw text, source range, target type and unresolved/resolved state, rather than immediately pretending to hold a model target.
- An action constructs or captures the previous object as necessary, creates the new type, replaces the frame, and optionally assigns the previous object through the action's feature/operator. No-op handling of actions would lose nesting/type changes.
- Mandatory arrays default to empty arrays and mandatory booleans default to false when undefined. Defaults follow reflection metadata and inheritance; they are not a global policy that every omitted field is null. Container links and CST association are established during construction.
- Object merge preserves source properties when target lacks them. If both contain arrays, source array entries precede target entries (`existingValue.push(...newValue)`). A conventional target-first merge changes source order.

The candidate overrides the base parser by a prototype patch in `packages/syside-languageserver/src/services/parser/parser.ts:326` (embedded equivalent starts near line 108999). Besides merge ordering, it assigns defaults and collects children when adopting a type into a previously untyped target. It retargets selected CST element associations for a nonfragment rule call from the old source object to the merged target. Preserve that ownership relationship using Rust arena identifiers; copying serialized objects cannot reproduce it. Candidate-to-bundle behavior beyond grammar declarations still requires a separate comparison/oracle.

Generated AST declarations provide field types but do not execute assignment/action semantics. Use the complete interface/type data, generated reflection metadata, and `src/model/types.ts` inheritance ordering. The model type index treats union/non-type and relationship cases specially; alphabetical flattening is not a justified substitute.

## Primitive values and lexemes

The embedded DefaultValueConverter begins at line 41808. It resolves a rule call (or the terminal of a crossreference), dispatches first by uppercase rule name, then by the rule's declared type:

| Dispatch | Source operation | Rust obligation |
| --- | --- | --- |
| INT | JavaScript `parseInt` | Implement only for reachable rules; preserve JS semantics if made reachable |
| STRING | Remove delimiters; scan backslash escapes b/f/n/r/t/v/0, otherwise retain escaped character | Do not substitute JSON string decoding |
| ID | Remove a leading caret | Do not strip arbitrary prefixes |
| REGEXLITERAL | Remove first and last character | Keep separate from regex evaluation |
| declared `number` | JavaScript `Number(input)` | Binary64 conversion, rounding, nonfinite values and signed zero require explicit policy/evidence |
| declared `string` or default | Original input | Preserve quotes, escape spelling and original bytes |
| declared boolean/bigint/date | Case-insensitive true / BigInt / Date in inherited helper | No explicit reachable rules of those return types are declared in these two grammars; helper presence is not grammar feature proof |

The complete declarations contain 31 parser string datatype rules (15 KerML, 16 SysML), six string terminals and two number terminals, counting both language roots. `STRING_VALUE` and `UNRESTRICTED_NAME` are string-typed terminals, not a rule named STRING; the converter therefore retains their quotes. Do not pre-unescape them in the lexer. `LiteralBoolean` expresses false by its default and true through `literal?='true'`; it is not evidence for a terminal boolean conversion. `TypedByToken` is a datatype fragment whose two keywords can accumulate `typedby`, without a space.

Retain raw token/CST text even after conversion. SysON chooses LiteralInteger versus LiteralRational from the source spelling (`.`, `E`, or `e`), so `1`, `1.0` and `1e0` cannot be collapsed permanently into the same number. Lexically accepted malformed exponent forms need an explicit conversion outcome; regex recognition does not establish a finite numeric value. JavaScript binary64 and JSON number behavior cannot be replaced silently by arbitrary precision Rust JSON parsing.

Names have another, later conversion layer: candidate `src/model/naming.ts:30` sanitizes quoted names by removing their outer apostrophes and unescaping b/t/n/f/r/quote/double-quote/backslash. Empty or undefined names become undefined. The Name object retains declared and sanitized forms; qualified naming consults semantic ownership and transparent elements. These rules differ from Java import cleaning and must not be applied globally to every string.

Candidate `src/model/KerML/expressions/literal-number.ts:59` also has semantic behavior beyond primitive conversion. Its return-type strings appear inverted relative to the `Number.isInteger` branch, and programmatic creation uses a truthiness guard that excludes zero/-zero/NaN. Preserve these as source observations requiring a pinned oracle; do not silently repair them under a parity claim.

## Construction hooks and reflection

Candidate `packages/syside-languageserver/src/services/parser/parser.ts` registers hooks through a CST builder map expanded to derived types. Every hook needs original and synthetic-node tests:

| Function (candidate line) | Required behavior |
| --- | --- |
| `fixOperatorExpression` (63) | Classification operators istype/hastype/@/as with no operands receive FeatureReferenceExpression, ReturnParameterMembership and Feature placeholders |
| `addLoopMember` (89) | WhileLoopActionUsage with no condition receives ParameterMembership and Usage |
| `finalizeImport` (104) | Select concrete namespace/membership Import or Expose type, retype reference, reparent filtered package imports, remove temporary isNamespace |
| `createEmptyParametersInTransitionUsage` (144) | Supply missing source/link parameter memberships and ReferenceUsage placeholders |
| `createMissingEndsInSuccessionAsUsage` (175) | Supply missing EndFeatureMembership/Feature ends with multiplicity-sensitive placement |
| `collectChildren` (259) | Rebuild children from streamed contents, sort by source range and assign child indexes after construction/default filling |

`src/services/sysml-ast-reflection.ts:40` selects the final target type by reference-chain kind (Type, Classifier, Metaclass, Membership, Namespace, Feature, ConjugatedPortDefinition); earlier chain links target Element. Inherited mandatory property metadata is cached with first name winning, `$children` is added, and `$childIndex` defaults to zero. Synthetic nodes clone CST metadata and mark it implicit. Parent insertion updates container/property/index links and shifted array indexes. The source's child insertion/push behavior must be checked against an oracle before claiming exact child order, especially for synthetic nodes sharing CST locations.

A proposed Rust representation uses stable arena IDs and typed scalar/array/reference slots; construction frames store the active node and datatype accumulator. Keep source spans in UTF-8 bytes and UTF-16 positions with explicit conversion. Evaluate immutable parse derivations through transactional construction so failed alternatives cannot leak mutations or reference IDs. An ambiguous generic recognizer derivation must not silently choose different construction semantics from Langium/Chevrotain lookahead, rule precedence or recovery. Either establish that choice with source/oracle evidence, or report unsupported/ambiguous construction. This is a proposed Rust design, not an upstream API.

## CLI wire format and import behavior

Candidate `src/utils/common.ts:129` defines `JSONreplacer`; `stringify` at line 223 calls JSON.stringify with indent 2. The embedded common module starts at line 85046. The replacer invokes custom toJSON for JSONConvertible objects, keeps `$type`, projects Element `$meta` to elementId/qualifiedName, simplifies `$cstNode` to text and a short rule stack, drops other `$`/`_` keys, and projects ElementReference text/resolved qualified name/reference parts. Non-Element metamodel values follow a different path. Testing helpers can patch toJSON differently; their debug objects are not automatically CLI output fixtures.

The serializer needs explicit missing versus null semantics, undefined array handling, nonfinite number handling, escape/UTF-16 behavior, ordering where consumers depend on it, and cycle/identity policy. A normal derive(Serialize) of the arena is not a reproduction of this projection. Reference resolution and `$meta` contents also require semantic services, builtins and linking; AST-only implementation cannot manufacture them.

Pinned SysON `backend/application/syson-sysml-import/src/main/java/org/eclipse/syson/sysml/parser/translation/EClassifierTranslator.java` maps reference aliases and renamed flow classifiers and uses `/$cstNode/text` to select integer/rational. `EAttributeTranslator.java:52` remaps attributes and values: Membership declaredName to memberName, literal expression literal to value, isReference/isNonunique to inverse composite/unique, importsAll to isImportAll, and keyword flags/transition/requirement kinds to their target values. Its `asCleanedText` at line 239 removes apostrophes throughout strings; LiteralString separately removes double quotes and collapses doubled backslashes. Body/language handling uses its own quote/comment removal and Java trim. These are separate from Syside name sanitation and JSON decoding. Target Ecore datatype conversion must also be accounted for rather than replacing every value with a string.

`backend/application/syson-sysml-import/src/main/java/org/eclipse/syson/sysml/SysmlToAst.java:85` imports stdout after a successful `dump` process. Its optional validation report at line 158 runs separately with `--validate --stdlib none` and does not itself decide AST import. The comments explicitly allow syntax errors where the parser dropped a suffix. Tests must record exit status, lexer/parser diagnostics, recovered AST, validation and imported model independently. A strict Rust parse rejection is a defensible product policy only if declared as a behavioral difference.

## Next executable milestones

1. Freeze a small immutable AST construction API and retain all grammar obligations as UNPORTED. Add native tests for source-first array merge, reassignment, boolean defaults, action nesting, crossreference nonresolution, fragment datatype concatenation and synthetic parents. These are source-derived tests, not differential evidence.
2. Acquire authorized baseline oracle outputs for matched valid/invalid fixtures before broadening parity claims. Cover unrestricted Unicode names, quoted/backslashed strings, hidden versus visible comments, signed/nonfinite/rounded numeric boundaries, number spelling, multiplicity-dependent synthetic ends, operator placeholders, import retyping, ambiguous alternatives and recovery suffix loss. Capture exact source/bundle hashes, input bytes, arguments, UTC intervals, stdout/stderr/exit, environment and fixture provenance. No oracle execution is performed by this plan.
3. Implement complete construction semantics incrementally by explicit obligation dispositions, then validate AST shape and primitive values independently of the semantic model. Expected-error fixtures count as tests only when actually run and reported.
4. Port scoped linking/metamodel/builtin/validation services and CLI projection before making semantic or import correspondence claims. Preserve source licenses and newly discovered dependency provenance; full bundle/artifact/notice closure remains OPEN.
5. Require independent review of each semantic implementation and its pinned native/oracle evidence. A reported single synthetic package/part oracle success establishes only that individual acceptance case; it does not close any grammar-wide, AST, value or semantic obligation in this plan.
