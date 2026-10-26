# C03b: prepare a connected migrated project privately

Callers previously had to chain semantic cleanup, document admission, library-ID migrations and diagram description-ID migration themselves. Add one typed project preparation API that calls the real B25b/a/c, Document, B01 and B25d paths in explicit order, retains separate stamps and namespace ordering, and returns matching wire/typed snapshots only after complete success. Late failures preserve input and prior output; external resolver effects remain caller-owned.

Validation: ten integration tests, formatting, Clippy and release observer build pass. The prerequisite passes 36 composed tests. Fresh native-segment projects match whole JSON values and namespace eligibility, including foreign-first and intentional atomic refusal. Original Unicode capture failure and its explicit encoding repair remain separate.

Measured median preparation is 1.77/6.89/29.10 ms for 256/1024/4096 semantic nodes plus six diagram nodes, with ten measured samples after two warmups per size. Raw inputs/outputs/RSS/receipts/runners are retained. These synthetic measurements do not claim native capacity or UI responsiveness. Native B01 library/history/SQL and host/global acceptance remain open.

Base: `codex/syson-rp-four-migrations-base` at 9baa77f9ce8c27cb0b784bee121872aab0ffcef5. Land reviewed prerequisites, retarget and revalidate; do not merge into the temporary snapshot. Existing held model/factory/initializer paths remain untouched.
