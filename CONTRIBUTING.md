# Contributing to the SysON Rust port

Thank you for helping. This guide is for the independent Rust effort in this repository. The original [Eclipse SysON contribution guide](CONTRIBUTING.adoc) is preserved; if you want to contribute to Eclipse SysON itself, follow its upstream process and requirements.

## Find a small task

1. Read the [README](README.md) and try its layout example and checks.
2. Browse [open issues](https://github.com/ajhcs/syson/issues), especially those labelled `good first issue`. You can also propose a documentation fix, a focused test, or a bounded behavior improvement.
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

## Ask for help

Use the relevant issue or pull request for questions, ideas, and review. If you do not have a task yet, [open an issue](https://github.com/ajhcs/syson/issues/new) with the area you would like to explore. For mentoring, mention [@ajhcs](https://github.com/ajhcs) when asking on an issue. Maintainers can help find a small first step and work through any project-specific coordination.

Please follow the repository's [code of conduct](CODE_OF_CONDUCT). The preserved [security policy](SECURITY.md) describes Eclipse SysON's reporting process. Do not post vulnerability details in public issues or pull requests.
