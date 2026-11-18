# Rust edit and reopen of captured SysON invocations

The existing Rust editor renamed `PartDefinition A` to `RenamedA` in the concrete and abstract `A()` models retained by the preceding Rust readback. The committed edit changed exactly one `declaredName` in each graph. The usage and invocation `FeatureTyping` relationships kept their identities and resolved both endpoints after the edit. The typed `F()` calculation served as an unchanged control, including the invocation target and the typing of return parameter `r` by `N`. [Original Rust readback](../invocation-rust/README.md)

A canceled rename left the document and history untouched. A staged rename stayed isolated until commit. Undo restored the captured document and redo restored the candidate. The editor exercised this history in the stage process. The saved files contain the semantic document.

The snapshot store published the originals and candidates under separate keys. A same-key save attempt for each edited graph returned `AlreadyExists`, kept the visible original byte for byte, and retained a staged file whose bytes matched the candidate. A second process reopened all five final snapshots and reproduced the graph and reference results. The originals matched all three snapshots from the preceding run byte for byte. [Captured snapshots and verification record](observation.json)

The offline build, stage, and reopen passed. The kernel step checked the narrow anonymous Rust subprocess handshake and denials for external socket operations and process-group changes. Each step recorded two empty owned-group observations before the supervisor reaped its leader. This evidence establishes scalar edit and file-level reopen of the captured models. Source import, application routes, persisted history and database durability remain separate.

## Verify the retained evidence

Run from the repository root. The verifier reconstructs the one-field changes and checks six altered records.

```sh
python3 -IB porting/oracle/results/invocation-edit/verify.py
python3 -OIB porting/oracle/results/invocation-edit/verify.py
```

The portable probe uses the same assertions and adapts the fixture and snapshot paths for a repository checkout. The recorded run used Rust 1.99.1 with a pinned private launcher. To replay the portable source, use Linux x86_64, a populated Cargo cache, and a new owned snapshot directory.

```sh
cargo build --offline --locked --release --manifest-path porting/oracle/results/invocation-edit/probe/Cargo.toml
probe=porting/oracle/results/invocation-edit/probe/target/release/syson-invocation-edit-probe
"$probe" stage /absolute/path/to/new-snapshots
"$probe" reopen /absolute/path/to/new-snapshots
```
