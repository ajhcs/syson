# RR02 source preparation (execution pending)

This packet records authored source only. The parent coordinator owns all Cargo
manifests/locks, formatting, compiler jobs, test/runtime execution, evidence,
commits and publication. This worker ran no formatter, compiler, test, native,
browser, database or network command. The canonical backlog and all held factory,
raw-admission, initializer and prior-helper paths remain untouched.

## Parsing authority and caller inventory

The only generic Value deserialization in these model/bridge entry points was
Document::from_json. It now calls RR01 parse and literal materialization. The
selected bridge serializes its private converted Value and invokes Bundle::load,
which calls Document::from_json. The inverse bridge invokes Bundle::load first,
then calls Document::from_json for its converted result and validates through the
selected bridge. Thus both bridges consume the same authority without inserting
redundant reparses or another semantic engine. Their production algorithms remain
unchanged; comments document the call chain. Original node UUID, semantic element
UUID and caller resource key domains remain separate.

Root wiring adds the local syson-json-value dependency to syson-model. A direct
interchange dependency names the same typed limit error in its narrow Bundle
mapping. No registry version or common export is changed by this worker.

## Explicit policy and necessary publication guard

Model admission uses 4 MiB raw input, value-depth 128, two million values, two
million keys and 8 MiB scalar-copy charge. This is an explicit bounded-admission
tightening: the old model had no raw byte cap and an implicit serde nesting bound.
Decoded duplicate keys now refuse explicitly. Syntax and string-encoding failures
remain ModelError::Json; resource/duplicate/allocation admission errors retain the
RR01 type inside ModelError::JsonAdmission. Bundle maps resource limits back to
its existing Error::Limit, while other model failures remain Error::Model.

A raw-input limit alone can admit minified JSON whose canonical pretty save will
not reopen. The parent therefore approved one private read-only depth check and
bounded pretty-byte counter at Document::from_json and before changed Editor
publication. It creates no owned output. It runs once at load/changed commit,
not from shared validate or every transaction operation; unchanged commits retain
the existing fast path. Failure precedes document/history/revision publication.
RR04 should later promote/reuse this primitive rather than add a second counter.
The existing bridge 1 MiB budgets remain unchanged, and RR04/RR05 serializer/output
allocation repairs remain separate. This guard does not claim all staged or
caller-cloned provisional Document values have already passed publication.

Document is a canonical Value view: exact original whitespace, source key order
and escape spelling are not retained. Literal object meaning and numeric token
spelling survive canonical serialization; this is not native EMF wire-order or
complete resource loading acceptance. The existing synchronous entry point does
not add a cancellation API. These caps are not native-library/capacity, latency,
allocator-overhead or full parity acceptance.

## Prepared verification

The first baseline-compatible model regression source was frozen as
b44c0320502e546fc15693e2649b52dc29e8567b7596bec4ccca0d89bc1adbe8 for root's
red reproduction. It used only existing APIs and independent manually built
object expectations; root retains that original snapshot separately.

Final model tests cover both private marker keys in all member positions,
nested node-shaped strings, unrelated rename/save/reopen, exact numeric tokens,
decoded duplicate keys, existing syntax/shape families, depth 128 and one above,
exact canonical 4 MiB save/reopen, canonical/raw overflow, and failed rename
publication preserving document, IDs, history and revision. Normal nested package
creation also proves changed publication enforces reload depth.

Bridge tests run actual F14 construction, both identity conversions, an Editor
rename, canonical save to a fresh create-new workspace file, file reload, reverse
conversion and exact opaque/identity checks. Manually built marker objects are
the oracle; expected JSON never uses generic Value deserialization. Additional
cases retain numeric lexemes through both bridges, refuse duplicate keys, preserve
existing 1 MiB bridge budgets/immutability and map the dense pretty-expansion
resource refusal to Bundle::Error::Limit. Test file I/O is limited to fresh
TMPDIR-owned absolute /workspace directories with no parent traversal; artifacts
are retained, not deleted. No durable-store or power-loss claim is made.

Suggested root checks: formatting and full syson-model and syson-json-interchange
suites, then all-target Clippy on final source; no browser journey is implied by
these pure model/bridge edits. Root must inspect the owned-file test before
execution and bind exact executed inputs to committed blobs. This source packet
contains no execution PASS or feature acceptance claim.
