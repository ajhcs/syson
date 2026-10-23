# Local Feature naming query candidate

Source and tests NOT_RUN. Published nine-test graph baseline: bad9b9ea9af590e4ebdb488a47058d3d87b56ec2. Its executed freeze is preserved unchanged, with original source available at that commit. This later candidate changes the current graph library; it does not relabel the earlier run.

Pinned SysON baseline: 6ba2ac64e6b16ee2b05752ad55395eebb87d4910. FeatureImpl.getOwnedRedefinition filters direct ownedRelationship in list order (line746), namingFeature selects only the first (line1058), effectiveName/effectiveShortName share the declared-name-or-short-name branch (lines1331/1352). RedefinitionImpl.getRedefinedFeature resolves a cached reference/proxy; only explicitly selected local semantic-ID string references are supported here.

ReferenceEncoding defaults to Disabled. Callers may explicitly select SemanticElementIds for this bounded Rust JSON adapter convention; this does not establish general EMF JSON reference encoding. Resource IDs never become semantic lookup fallbacks. Missing local targets are Unresolved, unsupported wire/proxy shapes and non-Feature targets are Unsupported. No external resolution is attempted. Missing/null first reference and self-reference yield known null. Traversal preserves containment order; cycles are refused rather than executing unbounded Java recursion.

Document admission validates envelope/identities/two containment features, not every attribute type. The candidate rejects non-string declaredName or declaredShortName before a known naming result. The published earlier absence branch's treatment of malformed short names is an unqualified negative-input concern addressed by a new explicit-refusal fixture, NOT an independently runtime-validated finding.

Nine new regressions are prepared alongside the nine original graph regressions: raw list order vs lexical identity, chained naming with scalar prefix visibility, first-null/no-later-fallback, missing/self reference, paired long/short/empty name rules, unresolved foreign/resource IDs, unsupported non-Feature/proxy shapes, naming cycles, and malformed name attributes. No Cargo, upstream, GUI or independent qualification has run for this candidate.

Factory allocation/reference mutation/inverse notifications, library/GeneralView lookup, non-Feature effective-name implementation dispatch, generic serializer/proxy/UTF16 behavior, independent factory acceptance and full MODEL parity remain open. Existing model and held review paths are unchanged.

The naming implementation is covered by the separate repaired combined31-test candidate. The original33-file standalone freeze at7effa0c71 remains NOT_RUN. See porting/domain/containment-adapter/QUALIFICATION.json. Owner evidence does not establish independent or upstream acceptance.
