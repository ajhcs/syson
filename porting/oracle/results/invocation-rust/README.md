# Rust readback of released SysON invocation models

The existing Rust model, snapshot, and reference APIs read the three EMF JSON graphs captured from Eclipse SysON v2026.9.3. The diagnostic saved one snapshot per fixture and reopened all three in a separate process. Each reopened JSON value matched the original captured graph. Both ends of each `FeatureTyping` resolved to the same resource fragment and semantic element ID before save and after reopen. [Released converter evidence](../invocation-emf/README.md)

| Captured fixture | Resolved target | Additional check |
| --- | --- | --- |
| Concrete `A()` | `PartDefinition A` | Abstract flag is false |
| Abstract `A()` | `PartDefinition A` | Abstract flag is true |
| Typed calculation `F()` | `CalculationDefinition F` | Return membership owns parameter `r` with direction `out` and type `AttributeDefinition N` |

The diagnostic rejected six reference controls per fixture in each process. A semantic ID in a resource-fragment slot, an unknown fragment, and an unknown resource remained unresolved. The two endpoints failed when read as the other's metaclass. A reference target obtained from one registry was stale in a second registry with the same document.

The first offline attempt stopped before rustc started. Cargo reported `Operation not permitted`, with exit status 101. The installed Rust source and glibc implementation point to a local process handshake as the likely cause. The separately admitted retry allowed the exact anonymous Unix pair and receive shape. Its kernel witness checked the handshake and the denied socket operations before Cargo ran. The build, save, and reopen then each exited successfully. [Recorded output and receipts](observation.json)

This uses captured JSON as input to the existing Rust APIs. Text parsing and linking, Java derived getters, application upload, database durability, process restart of the application, and feature parity remain separate acceptance work.

## Verify the record

Both commands verify the retained snapshots against the released converter graphs and reconstruct the endpoint identities. They also check both process captures, the failed attempt, and six altered records.

```sh
python3 -IB porting/oracle/results/invocation-rust/verify.py
python3 -OIB porting/oracle/results/invocation-rust/verify.py
```

The portable diagnostic source uses the same graph and reference assertions as the recorded probe. Its path to the converter observation and its acceptance of any caller-owned absolute snapshot directory are adapted for a repository checkout. The recorded run used the pinned private launcher and Rust 1.99.1. To replay the portable source, use Linux x86_64, a populated Cargo cache, and a new snapshot directory.

```sh
cargo build --offline --locked --release --manifest-path porting/oracle/results/invocation-rust/probe/Cargo.toml
probe=porting/oracle/results/invocation-rust/probe/target/release/syson-invocation-readback-probe
"$probe" stage /absolute/path/to/new-snapshots
"$probe" reopen /absolute/path/to/new-snapshots
```
