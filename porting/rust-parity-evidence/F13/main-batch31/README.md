# Finite linker adoption on current main

This adopts the complete F13 API and all 15 original tests from PR #50 (`aac4d2fd46d6f1c96358fa67bc017a4bcba85c5c`). It links finite local Package/PartDefinition/PartUsage reference chains over an immutable AST snapshot, with caller-observed aliases and explicit JSON-node identity bindings. It does not parse aliases, load resources or infer semantic identities.

The existing main provides the AST, finite Adapter, model, metamodel and reference APIs required by the donor. A new consuming-caller regression sends ordinary P01 notes, mixed newlines and a supplementary Unicode character through the actual Adapter, then checks qualified lookup, original byte/UTF-16 locations and unchanged syntax. Source review alone does not establish that this candidate passes runtime checks; the coordinator records fresh results after source freeze.

All original F13 source mappings, source-check scripts, review notes and captured execution streams remain verbatim historical `.raw` data under `archive/`; archived scripts are not executed by this adoption. Any failure is retained rather than rewritten as a pass. Original donor tests are preserved byte-for-byte before the appended regression.

This is an existing-provider dependency adoption for the subsequent atomic text importer. It does not complete that importer, the durable application journey, C01/C02, P02 or full language acceptance. Existing factory/raw/initializer/validation and UI ownership holds remain unchanged. Native artifact correspondence, consuming import/UI workflows and application performance/capacity remain open.
