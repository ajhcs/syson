# U10a canonical graph admission

This packet qualifies the finite std-only public admission API; placement and parent U10 native/application acceptance remain open.
Eleven actual API tests, formatting, Clippy and WASM checks pass on the frozen crate bytes.
The original F16 source, all 15 original placement tests/572 lines and source mapping are retained as historical inputs; those placement tests are assigned to U10b.
Resource results use the separate profiler landed in PR #168, whose exact publication tree and packet seal are referenced in summary.json.
Both default 1,024-node workloads refuse with Limit(Work); their latency is refusal latency, not successful admission or capacity.
The initial profiler controller failure and subsequent validation of the unchanged 72 samples remain separately attributed.
Archive entries retain exact original paths, hashes and raw bytes; SHA256.json seals the stored packet.
U10b must consume AdmittedGraph through its public entry and retain the original spent-work/cancellation guard.
