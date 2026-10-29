# Shared leaf numeric repair

Original pinned Package/ViewFrame source methods evaluate `(8 + height) + 8`; the Rust shared helper combined the constants as `16 + height`. At the admitted finite height 2^56, actual original-method execution and both native/WASM Rust baseline captures expose a one-bit result difference. Restore the source operation order without changing the public API, default limits, ownership or other editor algorithms.

The regression fails before repair. All 13 original editor cases remain byte-identical; the new case brings the suite to 14. Formatting, Clippy and WASM checks pass with unchanged bound sources. Independent source/evidence review accepted. Original selected TypeScript bodies are type-erased with the existing Node24 builtin; their complete source/excerpt hashes and actual transformation are retained. This is source-method qualification on supplied facts, not a real DOM or deployed-module observation.

The baseline 35-case dataset has 30 size-comparable rows and five explicit Rust refusal controls. Two leaf differences belong to this shared repair; the independent Note signed-zero difference belongs to F15 and remains separate. Actual composed F15/browser measurement and full editor/layout/UI acceptance remain open. No native speed, GUI latency or general capacity claim is made.
