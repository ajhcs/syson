# SysON Rust port

This repository contains an independent, work-in-progress Rust port of [Eclipse SysON](https://github.com/eclipse-syson/syson), alongside the original project's sources. It is not an official Eclipse project or release. Eclipse SysON was initiated by [CEA](https://www.cea.fr/) and [Obeo](https://www.obeosoft.com/) to provide open-source, web-based SysML v2 modeling tools. Their work and the upstream contributors make this effort possible. The [Eclipse SysON website](https://mbse-syson.org/) has more about the upstream project.

## Goal

1. **Port SysON to Rust, completely.** Every workflow SysON supports (SysML v2 text import and export, model editing, persistence, diagrams, the APIs and the editor) should have a Rust implementation that behaves the same way, checked against the original.
2. **Build MBSECraft on it.** The Rust crates become the modeling engine for **MBSECraft** ([ajhcs/mbse-craft](https://github.com/ajhcs/mbse-craft)), a native, open-source systems-engineering workbench in the [ArtCraft](https://github.com/storytold/artcraft) family of tools. It aims to be an open alternative to commercial MBSE suites such as CATIA.

Rust gives one codebase for native desktop apps and the browser (through WebAssembly), with no Java runtime, application server or database required just to model. Whether that also brings speed or easier deployment is something to measure, not assume.

## How progress is measured

The original SysON app is the reference. The [differential harness](porting/oracle/README.md) feeds the same SysML text to SysON and to the Rust importer, normalizes generated identifiers, and diffs the resulting models. It reports structural matches and wire-format compatibility separately: building the same model isn't enough if SysON can't open what Rust writes.

First run, 2026-11-17 ([full results](porting/oracle/results/20261117T143026Z.md)): **10 of 32 inputs match**, 1 differs (an escaped-name case where SysON looks wrong) and Rust refuses 21, including 17 of the 19 SysML files bundled with SysON. The complete SysML grammar is already recognized in Rust; the main gap is turning it into model elements ([#246](https://github.com/ajhcs/syson/issues/246)).

## Roadmap

| Phase | Outcome | Status |
|---|---|---|
| 0 | Differential harness against the official SysON release | Done ([`porting/oracle`](porting/oracle/README.md)) |
| 1 | Rust imports all 19 bundled SysON examples with the same model, and SysON can open the result | **In progress:** [tracking issue #269](https://github.com/ajhcs/syson/issues/269) |
| 2 | Edits, save and reopen, existing SysON projects, diagrams | Planned ([#259](https://github.com/ajhcs/syson/issues/259)) |
| 3 | Rust APIs and editor (native and WebAssembly) with full SysON workflow coverage | Planned |
| 4 | MBSECraft uses the Rust engine for its SysML v2 models | Planned, in [ajhcs/mbse-craft](https://github.com/ajhcs/mbse-craft) |

## Why contribute

Working here is a chance to solve practical Rust problems in parsing, public APIs, error handling, serialization and graph layout, and to help build an open engineering toolchain people can embed or run anywhere. Every task has an objective check: the Rust result either matches SysON or it doesn't. SysML practitioners can contribute representative models, describe expected behavior and settle semantics questions. You don't need to know the whole project; small, well-tested changes and good questions are useful.

**New here?** Start with the [Phase 1 tracking issue](https://github.com/ajhcs/syson/issues/269), which lists beginner-friendly tasks, or browse [`good first issue`](https://github.com/ajhcs/syson/labels/good%20first%20issue).

## What works today

The Rust tree contains separate crates for the metamodel, model operations, the SysML grammar recognizer, a text importer for a small subset of SysML, persistence helpers, layout and routing. The example below calls the public `syson-layout` library to place a small directed graph. Its tests exercise that limited library behavior; it is not a runnable SysML editor.

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

Contributions from people learning Rust, modeling, or the codebase are welcome. The [Phase 1 tracking issue](https://github.com/ajhcs/syson/issues/269) lists the current work, grouped by area, with a short "start here" list. Issues labelled [`good first issue`](https://github.com/ajhcs/syson/labels/good%20first%20issue) need the least background. Each import issue includes minimal SysML examples and the model SysON builds from them, so you know exactly what "done" looks like. Comment on an issue before starting so a maintainer can confirm its scope and whether someone is already working on the affected code. If you are unsure where to begin, open an issue describing your interests or ask on an existing issue.

The [contributor guide](CONTRIBUTING.md) explains the first steps, review expectations, and how to ask for help. Small documentation, test, and implementation changes are useful. Please keep evidence for the behavior you changed separate from claims about the whole port.

## Code and provenance

- [`rust/crates`](rust/crates/) contains the independent Rust crates; each has its own Cargo manifest.
- [`porting/oracle`](porting/oracle/README.md) contains the differential harness that compares Rust with the reference SysON.
- [`porting`](porting/) also contains source maps, behavior notes, evidence, and the remaining-work plan.
- [`backend`](backend/), [`frontend`](frontend/), and [`doc`](doc/) retain the upstream application and documentation sources.

The repository retains the upstream [Eclipse Public License 2.0](LICENSE), [NOTICE](NOTICE), and [upstream contribution guide](CONTRIBUTING.adoc). Some bundled standard-library resources carry [LGPL-3.0-only](LICENSE-LGPL) terms; see the notices for details. [Rust third-party notices](rust/THIRD-PARTY-NOTICES.md) record additional provenance and unresolved notice work. SysML® is a trademark of OMG; see its [trademark guidelines](https://www.omg.org/legal/tm_guidelines.htm). CATIA is a trademark of Dassault Systèmes; it is named only for comparison, and this project is not affiliated with or endorsed by Dassault Systèmes, Eclipse, CEA or Obeo.
