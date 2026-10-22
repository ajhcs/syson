# Pinned upstream reference oracle: partial parser run and application plan

## Current observed state

A bounded secondary CLI parser reference ran on 2026-10-22 at actual UTC 21:35:48.458113 through 21:35:48.665293, using verified existing Node v24.21.0, the exact baseline embedded bundle, one synthetic fixture, `dump --validate --stdlib none`, closed stdin, cleared Node preload/cache settings and a 60-second timeout in the default network-restricted environment. Exit 0, no stderr, statistics followed by AST JSON. Exact hashes and limitations: [parser-oracle-01](runs/parser-oracle-01/README.md). This does not execute the primary Java import path, qualify Rust AST semantics or establish a full-application oracle.


The selected baseline is source SHA 6ba2ac64e6b16ee2b05752ad55395eebb87d4910, not the published v2026.9.0 image. Root package.json/pom.xml are 2026.9.3. Using the default docker-compose.yml would silently compare another release, so it has not been launched. No upstream application service, database, npm lifecycle, Java application or Maven plugin was executed. The bounded CLI run and the Rust preview browser checks are separate evidence.

Read-only command evidence on 2026-10-22:
- `command -v java mvn node npm docker`: java, node, npm, docker exist; mvn absent.
- `java -version`: Temurin 26.0.2.1; upstream workflow requests Java 21.
- `node --version` / `npm --version`: system v18.19.1 / 9.2.0; root package.json requests 24.20.0 / 11.19.0.
- Existing specialized Node 24.21.0 is documented at <existing-node24>/bin (local operator inventory); it is not the exact engine pin. Its verified absolute binary was subsequently selected for the bounded parser reference only. No global tools changed.
- `settings.xml` references existing environment USERNAME/PASSWORD for GitHub Maven packages; access with established authentication is unverified, not denied. No credentials have been requested, copied, printed or created.
- `.github/workflows/build.yml` builds frontend first, copies artifacts, then Maven clean verify; later Docker integration tests start containers and browsers and some branches publish. This workflow has NOT been run. Fork workflows API currently returns zero entries; this is a snapshot, not blanket permission to execute future workflows.
- `application.properties` and `syson-sysml-import` source embed syside-cli.js and invoke Node for parsing. This is a required semantic dependency, not generated loader glue. Rust parity requires replacing it.

## Separate unresolved gates

1. Toolchain availability: find existing exact runtime/Maven distributions through documented inventories first. If unavailable, prepare a reviewed scoped package/toolchain change; no apt or package baseline changes are covered by this port's initial source work.
2. Package access: check established package read permission through existing credential tooling without revealing plaintext. Unverified entitlement is not an authentication failure and does not justify new credentials.
3. Build script review: inventory lifecycle scripts and Maven plugins including dependent build scripts; capture hashes of approved scripts. Current inspection is partial. Pin dependency artifact digests or source revisions, especially SNAPSHOT artifacts. Root pom references sirius.emfjson.version=2.5.5-SNAPSHOT, so source pin alone is insufficient for reproducibility.
4. Isolation: prepare a standalone reference worktree, cache/build directories on the approved development volume, synthetic PostgreSQL state, loopback-only unclaimed ports, no production connections and no competing displays. Do not execute the upstream compose directly: fixed port 8080 and source/image mismatch are unresolved.
5. Action-time execution: present exact commands with expected downloads, filesystem paths, network/port effects and stop conditions before an unrecognized upstream build/runtime attempt requiring approval. Never bypass a denial; uncertain outcomes must be checked before retry.

## Proposed command shape, NOT executed or ready for execution

A reviewed isolated reference checkout of the selected source SHA would first use exact Java21, Node24.20.0/npm11.19.0 and Maven, with all package caches redirected to an approved development volume. Frontend install needs lifecycle review before npm ci, then frontend build and static artifact copy. Backend Maven verify must use a private, established credential path and a local repository on the approved development volume. A build must NOT run deploy, npm publish, Docker login, Docker push, restart-postgresql.sh or any service registration. The runtime requires synthetic isolated PostgreSQL and a newly reviewed loopback port rather than the upstream 8080 binding. Max total concurrent compiler jobs remains two including Rust work.

After a real oracle exists, record dependency hashes, commands, outputs, failures, wall time and environment. Capture API/data artifacts and screenshots through isolated browser tests. Until then unit tests from literal upstream cases establish only the listed local behavior, with runtime differential comparisons UNRUN.
