# Contributing to the SysON Rust port

Thank you for helping. This guide is for the independent Rust effort in this repository. The original [Eclipse SysON contribution guide](CONTRIBUTING.adoc) is preserved; if you want to contribute to Eclipse SysON itself, follow its upstream process and requirements.

## What we're building

The goal is a complete Rust port of SysON that behaves the same as the original, and then **MBSECraft** ([ajhcs/mbse-craft](https://github.com/ajhcs/mbse-craft)): a native, open-source systems-engineering workbench in the [ArtCraft](https://github.com/storytold/artcraft) family, built on these Rust crates. The [README](README.md#roadmap) has the roadmap, and the pinned [roadmap issue #282](https://github.com/ajhcs/syson/issues/282) links each phase's tracking issue. The current milestone is **Phase 1: parser coverage and interoperability**, tracked in [#269](https://github.com/ajhcs/syson/issues/269). Phases 2–4 ([#279](https://github.com/ajhcs/syson/issues/279), [#280](https://github.com/ajhcs/syson/issues/280), [#281](https://github.com/ajhcs/syson/issues/281)) already have tasks that can start now, mostly inventories and decisions that need no Rust.

## Find a task

1. Read the [README](README.md) and try its layout example and checks.
2. Open the [roadmap issue](https://github.com/ajhcs/syson/issues/282), which suggests tasks by skill, or the [Phase 1 tracking issue](https://github.com/ajhcs/syson/issues/269), whose "Start here" list names tasks that need little background. You can also filter issues by label:
   - [`good first issue`](https://github.com/ajhcs/syson/labels/good%20first%20issue): smallest scope, most guidance
   - [`area: import`](https://github.com/ajhcs/syson/labels/area%3A%20import): SysML text to model (Rust)
   - [`area: interop`](https://github.com/ajhcs/syson/labels/area%3A%20interop): whether SysON can use what Rust writes
   - [`area: oracle`](https://github.com/ajhcs/syson/labels/area%3A%20oracle): the differential harness (mostly Python)
   - [`area: persistence`](https://github.com/ajhcs/syson/labels/area%3A%20persistence), [`area: api`](https://github.com/ajhcs/syson/labels/area%3A%20api), [`area: views`](https://github.com/ajhcs/syson/labels/area%3A%20views), [`area: ci`](https://github.com/ajhcs/syson/labels/area%3A%20ci): saving and project files, server APIs, explorer and diagrams, continuous integration
   - [`needs decision`](https://github.com/ajhcs/syson/labels/needs%20decision): behavior questions where SysML knowledge helps more than code

   You can also propose a documentation fix, a focused test, or a bounded behavior improvement.
3. Comment on the issue with what you would like to do. A maintainer can confirm the scope, whether it is available, and any dependencies or files that need coordination. If you need help choosing or narrowing a task, ask on the issue.
4. Fork [ajhcs/syson](https://github.com/ajhcs/syson) on GitHub. If you cloned the repository from the README, add your fork as a remote with `git remote add my-fork https://github.com/YOUR_USERNAME/syson.git`, replacing `YOUR_USERNAME` with your GitHub username. Alternatively, clone your fork directly and use its existing `origin` remote.
5. Create a branch with `git switch -c my-change`, then stage and commit your changes. Push it with `git push -u my-fork my-change`, or use `git push -u origin my-change` if you cloned your fork directly. On GitHub, open a pull request from your fork's branch to `ajhcs/syson`'s `main` branch. A draft pull request is welcome if you want feedback sooner.

There is no need to understand the entire remaining-work plan before making a small contribution. Maintainers will help identify the relevant source, current ownership, and review or evidence requirements for the issue.

## Make and check your change

Each Rust crate under [`rust/crates`](rust/crates/) has its own `Cargo.toml`; there is no root Cargo workspace. For a layout change, for example, run these from the repository root:

```sh
cargo test --manifest-path rust/crates/syson-layout/Cargo.toml
cargo fmt --manifest-path rust/crates/syson-layout/Cargo.toml --check
cargo clippy --manifest-path rust/crates/syson-layout/Cargo.toml --all-targets -- -D warnings
```

For another crate, use its manifest and run the checks relevant to the change. Add a focused test for changed behavior where appropriate, including an error or boundary case when it matters. Tell reviewers which commands passed and what you could not run. A crate test does not establish browser behavior, complete parser or storage support, upstream equivalence, or whole-product parity.

Keep a pull request focused on one reviewable outcome. Describe the problem, your change, and how to reproduce or check it. If you compare against upstream behavior, identify the source or fixture and any differences or unanswered questions. Preserve attribution, license notices, and existing evidence; ask a maintainer before changing shared manifests, generated data, or held source areas.

## Check behavior against SysON

"Correct" in this project means "builds the same model as SysON, and SysON can open it". The [differential harness](porting/oracle/README.md) checks both:

- **Without Java.** Each import issue includes minimal SysML examples and the model SysON builds from them, in a normalized form (`n0`, `n1`, … label generated identifiers; `ref(nK)` points to node `nK`). Turn these into Rust tests. Once [#254](https://github.com/ajhcs/syson/issues/254) lands, `python3 porting/oracle/differential.py --golden` will compare against committed reference outputs with only Rust and Python installed.
- **With the reference app.** `porting/oracle/oracle.py up` starts the official SysON release in isolation, and `differential.py` runs the corpus through both implementations. It currently expects the maintainer's tool paths; [#255](https://github.com/ajhcs/syson/issues/255) makes it portable. Maintainers can run it for your pull request if you can't.

In your pull request, list the cases whose verdict changed (for example `alias: RUST_REFUSES → MATCH`). A change that turns one case into a match must not break another.

When Rust and SysON disagree, don't assume SysON is right. If the original looks wrong (for example [#253](https://github.com/ajhcs/syson/issues/253)), raise it on the issue with a reference to the SysML v2 specification, and we'll record a deliberate divergence instead of copying a bug.

## Working on SysML import

- Import work goes through the full-grammar recognizer in `syson-syntax` ([#246](https://github.com/ajhcs/syson/issues/246)). Please don't add new constructs to the hand-written recognizer in `syson-ast-adapter`; it is being replaced.
- Upstream SysON maps its syntax tree to the model generically ([`AstTreeParser`](backend/application/syson-sysml-import/src/main/java/org/eclipse/syson/sysml/parser/AstTreeParser.java)). Prefer general mapping fixes over code that only handles one construct.
- Keep two questions separate: does Rust build the same model (semantic equivalence), and can SysON read it (serialization compatibility, [#251](https://github.com/ajhcs/syson/issues/251))? A change can pass the first and fail the second.
- The SysML standard library data in this repository is LGPL-3.0-only. Load it from where it lives rather than copying it into Rust source, and keep its notices.

## Ask for help

Use the relevant issue or pull request for questions, ideas, and review. If you do not have a task yet, [open an issue](https://github.com/ajhcs/syson/issues/new) with the area you would like to explore. For mentoring, mention [@ajhcs](https://github.com/ajhcs) when asking on an issue. Maintainers can help find a small first step and work through any project-specific coordination.

Please follow the repository's [code of conduct](CODE_OF_CONDUCT). The preserved [security policy](SECURITY.md) describes Eclipse SysON's reporting process. Do not post vulnerability details in public issues or pull requests.
