# Captured native HTTP contract

This offline tool reads the actual Actuator mappings response (`mappings FILE`)
or the 22 fixed anonymous HTTP observations (`probes FILE`). `--probes` emits the
canonical request manifest. It sends no requests and implements no HTTP routes.

Mappings retain every supplied context/category, handler, predicate, condition,
filter and servlet entry, including unknown keys. Structured MVC conditions must
include all six condition families. The tool preserves the complete raw tree;
its counters describe admitted observations, not exhaustive deployment coverage.

Probe admission requires each canonical ID exactly once, exact method/path/body,
canonical header values plus only the explicit generated wire headers, distinct
loopback application/management origins and diagnostic-mappings context. Response
statuses, repeated header pairs, exact UTF8 or hex bodies, byte counts, elapsed
observations and additional context values survive without normalization. Source,
artifact and configuration identities are required metadata shapes; this reader
cannot authenticate those supplied identities or reconstruct a native build.

Inputs are bounded at 32 MiB before parsing, then 200,000 JSON values, 24 MiB of
aggregate key/string bytes, 64 contexts, 20,000 servlet handlers and exactly 22
probe observations. Auxiliary traversal growth is checked before retaining child
references. Response bodies are at most 2 MiB, with at most 256 header pairs per
request/response. Standard serde_json nesting limits also apply. The CLI requires
a regular nonsymlink file before opening it and streams compact JSON output.

The actual capture enables only the mappings endpoint on an additional loopback
management listener. Its diagnostic context differs from an ordinary deployment.
This does not qualify redirects, authentication, sessions, mutations, uploads,
streaming, WebSockets, business/model payloads, other deployments or Rust handler
parity. Official artifact label and target-source differences remain explicit;
parent C01 stays OPEN. Raw capture/provenance and staged verification are retained
in `porting/rust-parity-evidence/C01f/` after coordinator execution.
