# RR05 source preparation (execution pending)

The worker authored source only. Root owns all compilation, formatting, tests,
allocation probes, runtime/network actions, evidence, commits and publication.
No model, held factory/raw-admission/initializer file, Cargo manifest, lock or
shared module export was changed. All implementation lives in the three leased
interchange source files and the new regression file.

## Separate original and remaining defects

The historical 500481-byte/depth60 original audit case produced a 16777216-byte
allocation before Error::Limit. RR02/RR04 now reject that case at the model 4 MiB
canonical guard. It is retained as an inherited refusal, not a falsely attributed
new red counterexample or unique RR05 repair.

On prerequisite #94 the remaining problem is lower: a compact input at 1 MiB can
have canonical output above 1 MiB but below 4 MiB. Bundle and bridge guards still call
model.to_json (its 4 MiB owned-output bound) and compare length afterward. Thus they
can own an over-interchange-budget output before returning their existing Limit.
The baseline-compatible two-test source was frozen as
22d980ec162e6201fc9b288d2a95c4b22515b425e52357d3bbb7ded8e4a50e42
for root's immutable baseline run. Its borrowed-bridge test asserts the forward
measurement before invoking reverse, so a forward red assertion skips the old
reverse measurement. Final fixed execution must reach both; no three-measurement
baseline claim follows from that two-case snapshot.

## Bounded authority and conversion proof

Bundle load now calls RR04 json_size(1MiB) without owning pretty output. Save
calls to_json_bounded(1MiB). The forward bridge counts before index/cloning; the
inverse bridge requests bounded owned output for its normal Bundle validation.
Typed model resource-limit errors map to each API's existing Limit family,
including provisional data that exceeds the model 4 MiB/depth128 publication cap.
Other model errors retain the existing Model family. Resource count, total node,
per-resource byte and identity-domain policies are unchanged.

Only FeatureTyping type/typedFeature strings change during conversion. Selected
index validation checks every node/semantic UUID with NodeIds (exact 36 ASCII
bytes). Native conversion follows Bundle validation of every ID, and requires
Decoded::LocalNode; the pinned decoder admits that form only for an untyped bare
UUID. Wrong-domain, typed, malformed or unresolved forms refuse before conversion.
Therefore every accepted replacement is exactly 36-byte string to 36-byte string:
canonical size is unchanged and opaque values/keys are untouched.

The already-counted canonical size also bounds compact converted JSON. One shared
private transport helper reserves exactly that admitted size, calls the existing
serde_json compact writer and refuses before appending past the bound. This
prevents amortized String growth beyond the limit after a large fragment plus
suffix. It introduces no independent formatter, counting or semantic authority.
Reservation failure is typed through model admission, UTF8 conversion reuses the
owned bytes and no partial String escapes. The helper's private unit case refuses
a deliberately broken size proof and invalid limit. Public bridge tests assert
canonical equality and allocation bounds at/near the actual 1 MiB boundary.

## Prepared verification and limits

The dev-only allocator delegates original pointer/layouts to System and keeps only
largest requested allocation in constant TLS; it owns no allocator containers.
Fixtures/inputs are built before tracking. By-value Bundle destruction may free
pre-observer inputs, so this observer makes no live-delta or RSS claim. The
remaining Bundle refusal allows one admitted scalar allocation up to 1 MiB; borrowed
bridge refusals allow at most 64 KiB scratch and no proportional serialized copy.

Eight integration cases plus one private transport unit case are prepared:
remaining bundle/borrowed-entry allocation gaps; exact/near 1 MiB actual F14 bridges,
save and reload with large-fragment/suffix allocation checks; one-byte-over
refusals; manual private-object/numeric-token oracles; wrong identity domains;
the already inherited original500481 case; and actual attach-created provisional
model-ceiling refusal. No model test hook or parser/identity authority is bypassed.
Both conversions, save/reload, immutable originals and existing canonical budgets
remain required. All predicates are synchronous as before; this does not add a
cancellation API, native oracle, UI/durability/capacity claim or new registry tuple.

Root should inspect the allocator, freeze exact final source, use scoped
nonrecursive formatting, run applicable interchange/model suites and all-target
Clippy, preserve all original failures and independently bind executed inputs to
committed blobs. This packet contains no execution PASS or acceptance claim.
