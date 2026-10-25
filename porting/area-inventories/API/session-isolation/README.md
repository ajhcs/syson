# F21 session and project-isolation source contract

This packet traces the pinned source boundary needed before selecting a Rust HTTP/session adapter. Its checker passes while qualification remains **BLOCKED_SOURCE_PREPARATION_ONLY**. Authentication, authorization, session/cookie, CSRF, CORS, WebSocket, project-context and deployed-artifact outcomes remain unresolved. The twelve scenarios are proposals for a later disposable runtime qualification; all native expected outcomes and receipts remain unresolved/null.

Source baseline is SysON `6ba2ac64e6b16ee2b05752ad55395eebb87d4910` (2026.9.3), within the Rust source base `ad814c232ee24109639c1d52239cbeec6f4a3e92`. The [manifest](source-manifest.json) pins 18 whole-file SHA256 values and exact line anchors. The checker compares each current file with the corresponding immutable Git blob, then checks the selected version/CORS projection against XML, properties and package JSON. This is a source declaration check, with no effective dependency resolution or application execution.

| Boundary | Direct source observation | Evidence still required |
|---|---|---|
| Authentication / authorization | The security manual states that SysON has no built-in authentication/authorization. The executable enables auto-configuration, imports the SysON starter, and that starter declares `sirius-web-starter` through `${sirius.web.version}` (2026.9.3). | Resolved inherited request filters, route/method/project checks, external gateway policy and the deployment's chosen anonymous/authenticated mode. The manual is documentary evidence, not an executed filter-chain receipt. |
| Session / cookie / CSRF | `SiriusWebApplication` receives HTTP and WS origins. Client transport/session behavior and server auto-configuration cross the inherited dependency boundary. | Exact inherited server/client sources, effective cookie/session/CSRF settings, expiry/logout/command captures and reviewed deployment policy. |
| Origin / transport | Both checked-in application property files declare `allowedOriginPatterns=*` and `allowedCredentials=true`. Root compose also declares an origin-pattern override. `URL.ts` derives default HTTP/WS origins from browser location; an explicit WS port takes the `ws://` branch. | Property consumers, effective profile/environment/proxy overrides, credential forwarding, actual response headers and WebSocket upgrade/subscription checks. These source declarations alone do not establish accepted origins or exploitability. |
| Project / editing context | The object REST delegate accepts `IEditingContext` and forwards element lookup to `IDefaultObjectRestService`. Project-data-versioning maps a parsed editing-context semantic-data ID through `IProjectSemanticDataSearchService`. | Request-to-context selection and principal/project permission enforcement in the inherited entry path. A resolved project mapping establishes identity, with authorization still unqualified. |
| Object read-only | `SysONReadOnlyObjectPredicateDelegate` applies standard-library and resource metadata decisions to objects; its source documents Explorer/Details consumers. | Evidence for authorization and HTTP/command enforcement remains separate. This object/library rule does not close the principal/project boundary. |
| Source / deployed artifact | Root compose defaults to `v2026.9.0`, application compose to `latest`, source version is 2026.9.3, and Dockerfile copies a wildcard matching JAR. | Exact JAR/image digests, dependency/source correspondence, effective configuration and policy hashes for the artifact used by future observations. No deployed instance was inspected. |

The production guide's TLS instructions and broad security wording do not supply the missing principal/session/project implementation. The security manual, inherited declarations and project delegates are retained together so downstream work can see precisely which claim each source supports. This work performs no vulnerability scan or operational security certification, and makes no inference that controls are absent merely because a local text search did not find them.

## Negative qualification scenarios

[negative-scenarios.json](negative-scenarios.json) specifies synthetic setup, probe, source links, unresolved native outcome and an explicitly proposed candidate goal for:

1. An anonymous element read under the selected deployment mode.
2. A foreign project's element read.
3. A foreign project's relationships read.
4. Equal element IDs in two projects with distinct marker data.
5. Forged or mismatched project/editing-context identifiers.
6. An expired session on reads and commands.
7. Revocation/logout during a subscription and reconnect.
8. Cross-origin preflight and credentialed HTTP requests.
9. Disallowed-origin or expired-identity WebSocket upgrades.
10. CSRF evidence on a state-changing command under the reviewed session mode.
11. Cookie attributes under a bound HTTPS/proxy configuration.
12. Source/artifact/configuration mismatch before attributing any runtime result.

Candidate goals require explicit host-mode/policy decisions; native 401/403/404 values, token types, cookies and anonymous access behavior are intentionally unassigned. No HTTP request, session, credential, database, production service or host adapter is created by the packet. Later runtime work must bind the inherited implementation and reviewed deployment policy before translating these scenarios into executable requests.

## Checks

`check_contract.py` validates this frozen preparation packet and source correspondence. It rejects missing/duplicated source identities, changed source bytes/anchors, stale projected declarations, removed boundary/scenario families, invented source citations, selected host adapters and attempts to relabel unresolved native/runtime/artifact claims as qualified. It does not execute the scenarios or decide an authorization policy from their prose. New native evidence requires a separately reviewed contract revision; this checker has no promotion path.

From the worktree root, with existing Python 3 only:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 porting/follow-ons/F21/run_checks.py f21-01
```

Use a fresh capture ID for another execution. The runner retains exact input hashes, original stdout/stderr, its own executed bytes and unchanged-input receipt under `porting/follow-ons/F21/validation/`. The packet generator is deterministic for this pinned source. It emits documentary declarations and scenarios, and runs no Maven/npm lifecycle or dependency acquisition.
