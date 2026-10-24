# HISTORY source inventory candidate

The source references cover the direct inherited undo package, editing-context change maps, diagram undo events, client stack code/SDL and one original SysON integration test. Exact source and read-byte identity are pinned; dependency artifact correspondence is still unproven. Nine stable contract groups are candidates, not complete behavior closure. Nothing here runs the original application, client, test, compiler or a new worker.

Backend undo and redo are addressed by original input UUID, with separate semantic and representation changes. Both use applyAndReverse on a found semantic change. A well-typed missing-ID request can still produce SuccessPayload and SEMANTIC_CHANGE. This must not silently become a conventional last-action stack/error contract. Representation-specific handlers and recorders remain separately unported.

Client stacks are another layer: qualifying outgoing mutations push input IDs and clear redo before their result; successful undo/redo responses move the then-current first entry between sessionStorage stacks. Component mounting resets stacks. Failed requests, in-flight concurrency, project changes, keyboard/focus and collaborative state require their own tests. No durable history or database restart guarantee follows from these maps/stacks.

The existing Rust model's local undo/redo is bounded MODEL evidence, not acceptance of inherited HISTORY. Full ignore/extension/recorder/handler closure, real input/event/serialization ordering, error/cancel/retry effects, retention/restart/recovery, isolation and original installed-client behavior remain open. Independent review and native implementation are pending; inventory_complete remains false.
