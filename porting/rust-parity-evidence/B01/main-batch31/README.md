# B01 migration preparation on main

This change adopts the complete original B01 finite migration-preparation API and all eleven original tests against the current model, metamodel and persistence crates. The native SQL fixture, source provenance, prior results and original limitations remain preserved in the donor archive and inventory.

The added regression passes literal JSON through the current `Document::from_json` boundary, migrates semantic and diagram references, and checks opaque metadata-looking objects, exact numeric tokens, input immutability, JSON reopen and watermark replay. Representation input is already a typed JSON value; this API cannot recover data that a caller has previously collapsed while parsing.

This library prepares private output. Durable storage must commit semantic documents, representations and migration watermarks together; database transactions, backend restart/recovery, native runtime equivalence, UI and performance acceptance remain open. Existing ownership holds remain unchanged. Fresh checks and their exact source bindings are recorded separately by the coordinator.
