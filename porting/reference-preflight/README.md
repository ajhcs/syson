# F39: immutable upstream reference preparation

This packet prepares a separate F40 build/launch review. **It does not authorize,
run or qualify an upstream build, dependency installation, application, database
or browser journey.** The checker can pass while the action remains `BLOCKED`.

The source target is SysON commit `6ba2ac64e6b16ee2b05752ad55395eebb87d4910`,
version 2026.9.3. Thirteen Git object/SHA-256 pins cover the POM, npm manifests and
lock, Turbo/Vite/lifecycle declarations, entry point and existing dependency
inventories. The first ten are upstream objects; the three prepared inventories
are pinned separately at port baseline `ad814c232ee24109639c1d52239cbeec6f4a3e92`.
This is a finite input inventory, not an effective build/dependency closure.

## Existing tools and observed access

`tool-inventory.json` records bounded read-only observations. Java and Node version
commands used installed executables only; npm was not invoked. Exact command
arrays, binary SHA-256 values, actual timestamps and raw streams are retained in
`porting/follow-ons/F39/validation/tool-inventory-*`. Executable hashes identify
those files, not every JDK/Node library or a fully pinned distribution. The
inventory recorder was an inline tool call; no standalone executed recorder
script was captured. The actual per-tool argv/output evidence is retained.

| Requirement from source | Observation | Disposition |
| --- | --- | --- |
| Java 21 | Selected Java 26.0.2.1; bounded visible JDK release lookup found 26.0.2.1+1. | Exact admitted Java 21 not established. |
| Node 24.20.0 | PATH Node 18.19.1; installed alternates 22.23.2, 22.23.3, 24.21.0 and 24.22.0 were checked. | No silent substitution of a different version. |
| npm 11.19.0 | PATH package metadata reports 9.2.0; alternate runtime npm metadata is retained where present. | No npm invocation or complete matching-toolchain qualification. |
| Maven | Not found in PATH or the bounded conventional file locations. | Version/binary binding unresolved; not a host-wide absence claim. |

This does not exhaustively search the host. An existing compatible candidate may
be proposed later with exact bytes/version/source review. Read-only cache-directory
visibility is recorded separately from artifact-byte verification. No Maven/npm
registry request, package entitlement probe, settings.xml/.npmrc read, environment
dump or credential collection occurred. `NOT_TESTED` registry entitlement and a
missing/unobserved local path are not a remote access denial. The Maven SNAPSHOT
has no established timestamp/build, artifact/source digest or source correspondence.

## Source lifecycle and immutable action boundary

The POM declares Java 21, Sirius 2026.9.3 and EMF JSON 2.5.5-SNAPSHOT. Root package
engines declare Node 24.20.0/npm 11.19.0. The source CI runs npm ci/build and Maven
`-U ... clean verify`; the update flag permits mutable SNAPSHOT resolution. Those
are inspected source facts, not executed actions or immutable commands approved
here. Turbo build also runs format checks/dependency builds. Vite's configuration
spreads process.env and calls loadEnv: a later build needs an explicitly reviewed
allowlisted environment/configuration. No current environment values are captured.

`preflight.json` freezes structured review-only command templates. The local
checkout command has a known commit/path; build/tool/config/artifact bindings
remain null or explicit placeholders. The project-dependency-installation stage uses npm ci offline with lifecycle
scripts disabled. It still installs/materializes project dependencies in node_modules;
it is not read-only cache validation. It remains review-only and NOT_RUN, distinct
from forbidden system tool/package-baseline installation and from the subsequent
frontend build, which executes lifecycle code. The backend template omits mutable -U and fixes one Maven thread.
No template is passed to a subprocess by the checker. Even a build command can
execute plugins/code and requires the later action review and resource allocation.

The proposed application and DB endpoints are loopback-only 127.0.0.1:18439 and
127.0.0.1:15439. Their availability is **unchecked**. New source/capture/cache/DB
paths are under /workspace; freshness/ownership has not been established. No
existing source checkout, cache, database or port is selected for mutation here.
A synthetic DB, extension/history baseline, isolated config and credential delivery
remain unresolved bindings. No production URL or secret is supplied.

Stop/retention rules require exact tool/source/dependency hashes, known owned
process groups, a bounded allocation, sufficient workspace capacity and unmodified
original outputs. Unexpected network resolution, external writes, unrelated
processes or non-loopback targets stop the future action. No automatic cleanup,
service registration, system tool/package-baseline installation, deployment or
rollback is prepared. The separate project-dependency installation is a future
review-only action, not an executed or authorized installation.
Only an explicitly identified owned process group may be stopped after a future
execution; no PID/port-based blanket kill is proposed.

## Reproduce the source/preflight checks

```sh
python3 -B porting/reference-preflight/check.py
python3 -B -m unittest discover -s porting/reference-preflight -v
```

The 21 synthetic tests check source facts, original version streams, blocking
status, version/access uncertainty, workspace paths, loopback ports, stage order,
immutable Maven/project-dependency actions, argument encoding, bindings, ownership/resource
rules and retention. These tests are not Maven/npm resolution, software builds,
security-policy acceptance, server behavior or feature parity. The captured
source and outputs remain unchanged across `f39-02`; raw receipts include actual
command results rather than a fabricated successful build. F40 is still blocked
on the exact compatible toolchain, resolved artifacts/SNAPSHOT/source, curated
environment, synthetic DB/history, fresh destinations and separate action review.
