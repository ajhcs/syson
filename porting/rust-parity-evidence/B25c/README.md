# B25c: ViewUsage exposedElement migration

The module prepares the selected native participant version
`2025.6.0-202506110000` using the shared B25S helper. It removes only the exact
`exposedElement` feature from object-valued children of matching ViewUsage
parents. The native comment's `exposeElement` misspelling is not the serialized
feature name. Pinned Ecore declares no ViewUsage subclasses; PartUsage and
ViewDefinition are explicit nonmatching cases.

The first namespace entry in original JSON order decides resource eligibility.
Consumers must feed `Prepared::to_json()` to the next participant or resource
reader. The public `json` value is for inspection; serializing it directly sorts
namespace keys and can change the next participant's resource predicate.
Traversal intentionally includes arbitrary nested arrays and vendor objects.
It is not restricted to known EMF containment: a typed parent's `data` and any
other object-valued child are removal targets, while primitive parent features
and untyped array rows remain untouched. These source quirks are compatibility
contracts pending the coordinator's exact selected native comparison.

The function returns owned JSON, the exact selected participant version and the
resource match observation only on success. Source input remains immutable.
It does not record native migration history, publish SQL or run a resource loader.
B06 must atomically persist semantic/representation state and participant records;
B01 preparation alone does not perform that publication.

The tests cover exact spelling and identity, child-object traversal, unrelated
classes, namespace insertion order, no-op/replay/failure/limits, and actual
Document admission followed by B01 library-ID preparation. `native-fixtures.json`
contains only original raw inputs. `source-predictions.json` records prepared
outputs and expected namespace-access failure separately. The exact initial
packet remains preserved as `prepared-packet-01.json`.
Coordinator `b25c-01` passed 18 integration tests (9 new and 9 shared),
formatting and all-target Clippy with 51 unchanged source inputs. The committed
inputs were checked against every final capture hash. Thirteen exact-byte input
cases match the selected source-compiled Java participant for success/refusal,
version, resource predicate and JSON values. Native exception text/classes and
non-namespace object-key ordering are not asserted equal.

The shared parent retains the four exact pinned Java source classes, compilation
with annotation processing disabled, nine identified release dependency JARs,
728 class-predicate comparisons and four ordered participant compositions. These
are selected runtime observations; whole artifact/source correspondence, complete
application deserialization, durable SQL and GUI acceptance remain open.

`captures/b25c-01/` preserves the exact root runner, receipt, all six streams and
archived crate/fixture inputs. `observations/` retains this unit's raw Java and
Rust probe outputs. `verification.json` links the common B25S input/tool evidence;
source-derived predictions remain separate from actual observations. The original
prepared packet and pre-format test snapshot remain unchanged. The source author
ran no compiler or native runtime; all execution was coordinator-owned.
