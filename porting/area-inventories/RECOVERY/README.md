# RECOVERY source inventory candidate

This candidate traces dispatch, payload, postprocessing, persistence, reload and disposal boundaries in the pinned inherited source. It explicitly separates source facts from unresolved crash/durability guarantees. Nine contract groups and a five-point proposed failure matrix are source preparation only. No process crash, DB, fault injection, backup, filesystem, service or runtime operation was performed.

The dispatcher runs preprocessors, handler, then postprocessors. Payload emission can occur in the handler stage. The surrounding event processor waits on Future.get, logs interruption/execution failures and returns the payload Mono. This source shape does not by itself establish whether a client-observed success follows durable commit when later work fails; the exact end-to-end boundary remains an acceptance obligation.

Resource conversion may return empty after caught serialization failure, while persistence retains successful conversions. The effect on previously stored documents needs exact runtime qualification. Transaction annotations, reload service source and disposal ordering likewise do not prove crash consistency, idempotent retry, recovery or backup/restore.

Native local snapshot behavior remains credited only to its documented MODEL/UI scope. It is not proof of inherited DB/recovery/history semantics. Proposed failure cases remain NOT_RUN and do not authorize fault injection or changes to held infrastructure. Independent source review, resolved artifacts/runtime and original/native recovery acceptance remain open; inventory_complete is false.
