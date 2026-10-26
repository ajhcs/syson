# C03c: transport authoritative prepared-project wire data losslessly

A prepared project's namespace order and independent migration stamps must survive transport. Add a local versioned codec whose encoder accepts only PreparedProject and reads its authoritative wire snapshot. Inline RawValue preserves payload token bytes without quoting/reformatting JSON; strict envelope metadata requires explicit nullable stamp keys and rejects duplicate, unknown or future-format fields. Decode returns untrusted ProjectInput for real C03b preparation.

Validation: nine actual preparation/codec/replay and refusal tests, one compile-fail API test, formatting and Clippy pass. The lock retains the same 11 registry tuples. Candidate byte/item limits and no-quoting-amplification behavior are tested; no native archive/capacity, timing SLA, SQL or global acceptance claim is made. Exact receipts/streams, independent review and the supplemental inherited-fixture binding are retained.

Depends on #81 at 7f111c24bb3f79c44c55cdef2d4fa3246e50560b. Land reviewed prerequisites, retarget and revalidate affected integration. Existing held paths remain untouched.
