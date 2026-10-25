The existing DB inventory retained schema XML but did not freeze a checked declaration
sequence. This adds a source-only planner for the captured candidate graph: 29 XML
documents, 28 ordered includes and 20 changeSet file/id/author identities. Operations,
attributes and character data are retained without computing an applied schema.

The planner rejects unsupported include/property/logical-identity forms, cycles,
missing/duplicate inputs and unclosed external payload/custom execution. DBMS,
precondition, rollback, transaction and checksum declarations remain unevaluated.
SHA-256 source pins are explicitly separate from Liquibase history/checksums.
A synthetic two-changeSet example and blocked separate-review command templates
prepare the next execution decision without inventing unresolved tool/artifact bindings.

Validation: 19 stdlib-only tests and four pinned integration Git inputs pass;
raw outputs, exact runners/input hashes and original pre-fix capture are preserved.
Root review requested XML tail preservation and nonempty-include rejection; both
have regressions. Candidate-to-resolved-artifact correspondence remains UNPROVEN,
including the EMF JSON SNAPSHOT. No database, migration, upstream lifecycle,
download, service, credential, shared backlog or dependency change was performed.

F26 has no future-unit prerequisites and targets rust/integration. This is a bounded
source-contract candidate, not resolved schema or whole DB parity.
