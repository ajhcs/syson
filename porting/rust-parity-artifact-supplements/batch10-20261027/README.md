# Batch 10 retained HTTP log supplement

This later data-only supplement publishes 16 original retained `http.log` files that were named and hashed by the historical RR18b1/RR18b2 seals but absent from their Git trees. It does **not** make the original PR heads self-contained or alter their files, seals, receipts, failures, test attribution, or acceptance state.

`MANIFEST.json` maps each missing logical path from [PR #113](https://github.com/ajhcs/syson/pull/113) at `7f1ae8f4e6b618b75615f589f35ce25f79600efb` and [PR #114](https://github.com/ajhcs/syson/pull/114) at `592e918c892fcdef8439fea9056c41da3599013f` to its copied path here. It also identifies each exact historical `SHA256.json` Git blob and digest. All copied payloads match those historical seal entries byte for byte; none were reconstructed or redacted.

The retained payloads contain only status records for `GET /`, `/style.css`, `/loader.js`, and `/syson_web.wasm` using HTTP/1.1, status 200, and the original `-` size field. Every complete line passed that exact allowlist before copying. These records contain no authentication, cookie, query-string, or request-body fields; this describes the copied records, not an unlogged network-header audit.

The 16 copies retain their `.log` extensions. Existing ignore rules remain unchanged. Root owns the separate privacy review and explicit staging of only the manifest-listed log files, publication, and merge.

This supplement adds retrievable historical bytes on a later commit. It performs no Rust, browser, native, or runtime replay and closes no feature, usability, performance, or parity acceptance gate. Historical broad acceptance remains unchanged.
