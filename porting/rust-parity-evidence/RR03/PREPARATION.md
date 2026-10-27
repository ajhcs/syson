# RR03 source preparation

Base snapshot e234787259bb2a15c2ec5384b14f65b1a6e387de combines the exact C01g #85
head with five exact RR01 #90 crate blobs. Root owns manifests, lockfiles,
formatter/compiler, all Rust/CLI/native execution, independent review and final
publication. This document is preparation, not a PASS receipt.

The original phantom fixture was supplied before implementation as baseline-
compatible source SHA256 cee9198cf0fce3cc57f029c6ac207398f5b43729da77a4c09075bc5db5058f44.
It uses only plain String serialization as an independent oracle: content is
empty and no physical object has eClass. The node-shaped JSON exists solely as
String content under the literal private RawValue key. Root retained that source
for a real old-catalogue counterexample; execution remains root-owned.

## Approved integration policy

Both payload and manifest materialization use RR01. Exact source bytes remain in
Resource::json, and physical node, node-ID and semantic-ID indices retain their
separate domains. Actual eClass objects outside content remain physical under
C01g's explicit stored-format rule; encoded strings never become nodes.

Per-resource 4MiB / aggregate32MiB raw limits and every existing physical-node,
path, identity, pointer, index-string and traversal budget remain. New aggregate
literal values and keys are each capped at2million; scalar-copy charge is capped
at twice per-resource raw budget and twice aggregate raw budget, following RR01's
conservative copy policy. This measures requested scalar content, not allocator
or RSS overhead. These explicit limits replace old serde-specific numeric-wrapper
visit accounting. No hidden coupling to generic Value token reinterpretation is
retained. Root must verify the actual94resource/109673tuple corpus and measured
memory on final exact source, preserving original native captures unchanged.

RR01's hard depth256 counts all values; old C01g counted containers only. Root
approved explicit tightening solely at container256 + scalar257. Value-depth256
and the actual depth208 corpus remain required positive cases. The prior depth
fixture's deepest admitted scalar is adjusted accordingly. Two old serde prose
assertions change to honest typed JSON-syntax refusal text; malformed input still
refuses rather than fabricating an EOF/trailing message.

The additive with_cancel library entry point reaches raw token scans, private
materialization and resource/physical traversal checks; the existing entry point
delegates false. No partial catalogue escapes early or late cancellation. This
is not a CLI deadline, internally preemptible library call or UI latency claim.

## Planned root verification

Retain baseline red source/output separately. Run existing catalogue tests plus
new marker-position/object/string, encoded-identity, raw-number/escape, duplicate,
malformed, cancellation, actual depth and aggregate-value boundary regressions.
The aggregate test uses three sub-4MiB resources with exactly2million total
literal values, then one extra value; no fabricated reduced production limits.
Run actual CLI phantom/ordinary failures and the full unchanged94resource input
against retained109673 native physical tuples with independent raw-source counts.
Freeze exact prerequisites, production source, test fixtures, manifest/lock,
request/helper/input hashes; inspect before/after identity. No old PASS is reused
for changed source. C01, semantic loading, model/loader adoption, URI resolution,
UI consumers, PostgreSQL and whole-build source correspondence remain OPEN.
