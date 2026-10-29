# SysON Rust port

This repository contains an independent, work-in-progress Rust port of [Eclipse SysON](https://github.com/eclipse-syson/syson), alongside the original project's sources. It is not an official Eclipse project or release. Eclipse SysON was initiated by [CEA](https://www.cea.fr/) and [Obeo](https://www.obeosoft.com/) to provide open-source, web-based SysML v2 modeling tools. Their work and the upstream contributors make this effort possible. The [Eclipse SysON website](https://mbse-syson.org/) has more about the upstream project.

The port aims to make more of the modeling toolchain available in Rust, with a path toward native and WebAssembly targets. The long-term goal is to support the workflows available in SysON; compatibility and performance are questions to verify, not benefits already established by this port.

## Why contribute

Working here is a chance to solve practical Rust problems in public APIs, error handling, serialization, and graph layout, and to help shape a modeling toolchain that could be embedded or deployed in different environments. SysML practitioners can contribute representative fixtures, describe expected behavior, and help assess results. You do not need to know every part of the project: small, well-tested changes and questions are useful.

## What works today

The Rust tree contains separate crates for model operations, syntax recognition, persistence helpers, layout and routing. The example below calls the public `syson-layout` library to place a small directed graph. Its tests exercise that limited library behavior; it is not a runnable SysML editor.

The port is incomplete. The syntax crate is a recognizer, not an accepted complete SysML parser, and the persistence helpers do not provide the complete storage system. Native and browser UI workflows, font measurement, worker integration, fractional routing, upstream equivalence, and full product parity remain unqualified. See the [syntax](porting/syntax/README.md), [persistence](porting/persistence/README.md), and [remaining-work guide](porting/rust-parity-plan-20261026/REMAINING-WORK.md) for details and evidence boundaries.

## Try the layout library

You need Git and a recent stable Rust toolchain capable of edition 2024, with Cargo, `rustfmt`, and Clippy. You can install Rust using [rustup](https://rustup.rs/); if you use rustup, `rustup component add rustfmt clippy` installs the two check tools. The selected library and its test dependencies are local Rust crates; this example does not require Java, Node.js, an account, or a database. The commands below use a Unix-like shell and `mktemp`. Clone the repository and run the checks from its root:

```sh
git clone https://github.com/ajhcs/syson.git
cd syson
cargo test --manifest-path rust/crates/syson-layout/Cargo.toml
cargo fmt --manifest-path rust/crates/syson-layout/Cargo.toml --check
cargo clippy --manifest-path rust/crates/syson-layout/Cargo.toml --all-targets -- -D warnings
```

To use the library from a separate Cargo project, run the following from the repository root. It creates a temporary project, adds the checked-out crate by path, and runs a small program:

```sh
REPO_ROOT="$(pwd -P)"
DEMO_DIR="$(mktemp -d)"
cargo init --bin --name syson-layout-demo "$DEMO_DIR"
cargo add --manifest-path "$DEMO_DIR/Cargo.toml" --path "$REPO_ROOT/rust/crates/syson-layout"
cat > "$DEMO_DIR/src/main.rs" <<'RS'
use syson_layout::{layout, DirectedEdge, Limits, MeasuredNode, Options};

fn node(id: &str) -> MeasuredNode<'_> {
    MeasuredNode {
        id,
        width: 30.0,
        height: 20.0,
        parent: None,
        port_count: 0,
        pinned: false,
    }
}

fn main() {
    let nodes = [node("a"), node("b")];
    let edges = [DirectedEdge { id: "ab", source: "a", target: "b" }];
    let result = layout(&nodes, &edges, Options::default(), Limits::default(), || false)
        .expect("layout should succeed");
    let a = result.nodes.iter().find(|p| p.id == "a").unwrap();
    let b = result.nodes.iter().find(|p| p.id == "b").unwrap();
    assert_eq!((a.rank, b.rank), (0, 1));
    assert!(a.x + a.width <= b.x);
    println!("Placed a before b: a=({}, {}), b=({}, {})", a.x, a.y, b.x, b.y);
}
RS
cargo run --manifest-path "$DEMO_DIR/Cargo.toml"
```

The program checks that `a` is ranked before `b` and their horizontal extents do not overlap, then prints `Placed a before b: a=(16, 16), b=(86, 16)`. The example uses flat, unpinned nodes without ports; it does not demonstrate edge routing or SysON/ELK layout equivalence. The temporary project remains at the path in `$DEMO_DIR` for inspection.

The layout checks and example passed on Linux x86_64 with `rustc 1.99.1` and `cargo 1.99.1`, a clean checkout, initially empty Cargo home and target directories, and offline Cargo. The run covered 56 layout tests and the example; other platforms have not been qualified by this run.

## Contributing

Contributions from people learning Rust, modeling, or the codebase are welcome. Browse the [open issues](https://github.com/ajhcs/syson/issues) and look for `good first issue`. Comment on an issue before starting so a maintainer can confirm its scope and whether someone is already working on the affected code. If you are unsure where to begin, open an issue describing your interests or ask on an existing issue.

The [contributor guide](CONTRIBUTING.md) explains the first steps, review expectations, and how to ask for help. Small documentation, test, and implementation changes are useful. Please keep evidence for the behavior you changed separate from claims about the whole port.

## Code and provenance

- [`rust/crates`](rust/crates/) contains the independent Rust crates; each has its own Cargo manifest.
- [`porting`](porting/) contains source maps, behavior notes, evidence, and the remaining-work plan.
- [`backend`](backend/), [`frontend`](frontend/), and [`doc`](doc/) retain the upstream application and documentation sources.

The repository retains the upstream [Eclipse Public License 2.0](LICENSE), [NOTICE](NOTICE), and [upstream contribution guide](CONTRIBUTING.adoc). Some bundled standard-library resources carry [LGPL-3.0-only](LICENSE-LGPL) terms; see the notices for details. [Rust third-party notices](rust/THIRD-PARTY-NOTICES.md) record additional provenance and unresolved notice work. SysML® is a trademark of OMG; see its [trademark guidelines](https://www.omg.org/legal/tm_guidelines.htm).
