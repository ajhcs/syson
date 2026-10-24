# MIGRATION-ID source inventory candidate

This candidate enumerates all fifteen Java sources directly in the pinned application migration package, its six direct integration test classes (eight declared tests), and the ID-map resource. [inventory.json](inventory.json) holds exact Git revision/path/hash references and eleven stable behavior IDs. The tests have not been executed. Package enumeration is not complete inherited/runtime behavior closure; `inventory_complete` remains false.

Existing Rust evidence qualifies only the pure library map, URI/diagram-target rules and persistence filter. It does not implement the migration service JSON traversal, the annotation/connector/exposedElement participants, diagram initialization hooks, transactional database changes, scheduler/version application or all errors/retries/interruption outcomes.

Two easily lost source details remain explicit: `isSysMLResource` returns after the first namespace entry; `collectElementsOfType` tests the parent JSON object's classifier before collecting each child object. Array traversal does not directly add each object member. A port must qualify these actual source contracts before choosing any intentional correction. No correction is authorized by this inventory.

Diagram-on-ViewUsage and one-diagram migrations combine post-load adapters with initialization hooks and first textual regex replacement. Both select only the first matching migration adapter per element in a hook pass. ViewUsage creation can return null while its caller proceeds; that is an unqualified failure path, not a newly invented success contract. No database/upstream execution occurred.

The next proposed implementation is a pure six-pair control-node description-ID replacement callback under the existing persistence crate. Its exact API, paths, cases and exclusions are in the JSON. It remains proposed pending independent source review and an explicit implementation lease. The source constants come from `StartDoneAndControlNodeMigrationParticipant`, not from handwritten UUID guesses.

This candidate needs distinct independent review before the inventory item can be accepted and the proposal becomes executable work. Full inherited services, fixture transformations, license/artifact correspondence, database transaction failure/retry/cancel and original integration acceptance remain separate open obligations.
