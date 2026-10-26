Add a bounded physical catalogue for shipped SysML/KerML library JSON, preserving raw bytes and exposing separate resource-scoped node-ID and semantic-ID lookups. The consuming CLI admits only bounded regular files under an explicit root and exports an identity index without fallback or a second semantic engine.

Validation: nine integration tests, formatting, Clippy and both builds passed. The release CLI matched all 109,673 stored identity/class/name tuples across the actual 94-resource native corpus; exact runners, receipts, raw streams and external capture identities are retained. One process observed 0.34s wall/150,928 KiB RSS including a 49 MB index export, without an SLA claim.

Native loading also recorded 12,162 package-not-found diagnostics. This qualifies physical catalogue behavior, not clean reference resolution, native fragment aliases, semantic loading, UI integration or full SysON parity. C01 and M15/M16 remain open.
