# RR21 source preparation

Status: SOURCE_PREPARED_NOT_EXECUTED_BY_AUTHOR. Root owns formatting, all compiler/runtime evidence, exact source acceptance and publication.

## Scope and authority

Base: `3cc8784e00dc31715a30986ab97bcbfc68b57305`, the selected #59 source with the coordinator-recorded unchanged RR04 model/RR01 crate prerequisite blobs. Author lease: semantic-validation `src/lib.rs`, `tests/rr21_regression.rs` and this evidence directory only. The coordinator separately owns the direct local syson-json-value dependency and lockfile wiring. No model, initializer, reference, NameHelper, held factory or exporter source is changed.

MODEL-RR-007 identified pretty serialization and generic Value reparsing before finite validation. RR04 now already refuses canonical output above 4 MiB; the historical original finding is retained unchanged. RR21 addresses the remaining redundant copies below that ceiling and the full Document copies retained by the existing F46 Snapshot consumer.

## Deliberate selected-validator policy

Validation now counts the complete existing Document through RR04 `Document::json_size` before parser construction, element summaries or Snapshot creation. The selected-validator limit is **1,048,576 canonical pretty JSON bytes inclusive**, with RR04's unchanged **value-depth 128** authority, including otherwise ignored opaque envelope fields. Over-budget provisional or published candidates return `UnsupportedShape("selected document byte/depth budget")`. Typed RR01 cancellation maps to `Error::Cancelled`, without inspecting error strings.

The 1 MiB whole-document policy is a deliberate selected API tightening, not a claim about native SysON project capacity. Documents above 1 MiB that previously passed selected shape checks are now explicitly refused, even when opaque fields would be ignored semantically. Budget/depth refusal now precedes parse/shape/source-association errors when multiple inputs are invalid. Empty/unknown rule and initial cancellation ordering remains unchanged. The counting pass also walks the whole admitted Value before output counting, so the claim is bounded requested allocation before refusal, not constant time or a fixed cancellation deadline.

Root package shape is borrowed directly from `Document::value()`. Existing finite classifier/data checks, 2048-model-row cap, declaration ordering/ownership, explicit semantic-ID type references, actual F11/F14/F46 routes, selected checks, warning severity and source locations remain the same code. Snapshot still clones admitted documents; this change bounds those copies rather than introducing another semantic engine or claiming zero-copy success. No owned JSON serialization or generic Value reparse remains in validate.

## Regression preparation

Seven source regressions cover:

- An actual F14 document with an actual staged OwningMembership containing 20,000 null values under 32 arrays. Compact JSON is below 1 MiB; canonical pretty size is above 1 MiB but below RR04's 4 MiB. Refusal must avoid a proportional requested allocation and leave editor/history unchanged.
- Otherwise ignored root opaque text at 1 MiB + 1 byte, proving an implementation that only removes serialization but reaches Snapshot::new is insufficient.
- Independent serde pretty-byte construction at exactly 1 MiB, successful unchanged F14 selected checks, and exact +1-byte refusal.
- A within-budget opaque Unicode root object with an actual untyped F14 PartUsage: unchanged actual F46 qualified label, semantic identity, WARNING/PART result and byte/UTF16 spans across CRLF and supplementary characters.
- Late declaration-name/source mismatch, late parse failure and unchanged editor/history.
- An actual staged opaque depth beyond 128, refused before parsing deliberately bad source; transaction cancellation leaves publication/history untouched.
- Initial typed cancellation, empty/unknown rule errors and selected-rule deduplication.

The development allocator delegates original pointers/layouts to System unchanged. Constant thread-local counters track only requested allocation on the calling test thread during validate: largest request, net live and additional peak; they are not allocator overhead, RSS, C allocation, all-process memory or latency measurements. The guard remains largest <= 65,536, peak <= 131,072 and live == 0 for refusal. Setup, exact byte construction, document comparisons and logs occur outside the observation window. A cancellation Arc owner is retained outside observation, with its passed clone also created outside, so dropping validate's clone does not deallocate an uncounted Arc allocation.

## Baseline and qualification boundary

Coordinator executed the immutable first regression snapshot (SHA256 `e67abccf8ec78a22e81b9d5fa92c259df9b71fb910caca9649510b2672fecbad`) against baseline. Reported canonical bytes and largest allocation were both **1,723,812**. The original raw observer also printed peak 2,789,082 and live -24 because the sole pre-existing cancellation Arc was dropped inside observation. Those raw values remain retained in the original receipt, and are not valid net-live or precise peak claims. The original largest request is the reliable allocation counterexample. Final source fixes observer ownership; no baseline replay or historical evidence relabeling is requested.

Author ran no Cargo, formatter, compiler, test, runtime or network action. Root must review the final allocator/test code, format only leased Rust files, and run a fresh serialized offline semantic-validation suite including existing rules tests and all new regressions on final hashes. Existing selected consumers exercise actual F14 import/Editor and F46 names; no UI, native upstream execution, full AQL/validation corpus, large project, RSS or user-latency acceptance is claimed. Independent source review and coordinator receipts are separate acceptance gates.

Coordinator's first final-source run (`rr21-01`) passed six of seven new cases. The late-source fixture failed during actual F14 setup before validate: unquoted `first` is reserved in the pinned parser (RR07 is not this prerequisite). Final source changes only that ordinary usage name to `frontPart`; production is unchanged. The failed setup receipt stays retained. Coordinator separately observed corrected refusal largest request 64 bytes, additional peak 190 bytes, net live 0; final exact-source qualification remains a separate coordinator receipt. The existing rules target had not run after the failed new target and remains required.

Coordinator's corrected final-source run passed **7 new regressions plus 18 inherited rules = 25 tests**, with formatting and all-target Clippy passing. The 18 inherited cases include two #59 line-ending regressions beyond original F47's 16; earlier 16-case pending estimates are historical estimates, not executed counts. Production and corrected test hashes matched the accepted freeze. This paragraph records coordinator-reported results; author performed no execution. The coordinator's exact receipt and independent source review govern bounded acceptance.
