#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Reproduce the frozen documentary packet from the checked-out source."""
import importlib.util
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / 'porting/area-inventories/API/session-isolation'
spec = importlib.util.spec_from_file_location('contract_check', BASE / 'check_contract.py')
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
anchors = {
    'pom': ['<version>4.1.1</version>', '<sirius.web.version>2026.9.3</sirius.web.version>'],
    'application-pom': ['<artifactId>syson-starter</artifactId>'],
    'starter-pom': ['<artifactId>sirius-web-starter</artifactId>', '<version>${sirius.web.version}</version>'],
    'starter': ['Sirius Web components are configured by the', '@ComponentScan(basePackages = "org.eclipse.syson")'],
    'entry': ['@EnableAutoConfiguration'],
    'properties': ['sirius.components.cors.allowedOriginPatterns=*', 'sirius.components.cors.allowedCredentials=true'],
    'dev-properties': ['sirius.components.cors.allowedOriginPatterns=*', 'sirius.components.cors.allowedCredentials=true'],
    'object-delegate': ['implements IObjectRestServiceDelegate', 'return this.defaultObjectRestService.getElementById(editingContext, elementId);', 'emfEditingContext.getDomain().getResourceSet()'],
    'project-delegate': ['implements IProjectDataVersioningRestServiceDelegate', 'protected Optional<String> getProjectId(String editingContextId)', 'this.projectSemanticDataSearchService.findBySemanticDataId'],
    'read-only': ['implementation (used for the <i>Explorer</i> and <i>Details</i> views)', '.map(ResourceMetadataAdapter::isReadOnly)'],
    'frontend-entry': ['SiriusWebApplication,', 'httpOrigin={httpOrigin}', 'wsOrigin={wsOrigin}'],
    'frontend-url': ['const { VITE_HTTP_SERVER_PORT, VITE_WS_SERVER_PORT }', 'wsURL = `ws://${window.location.hostname}:${VITE_WS_SERVER_PORT}`;'],
    'frontend-package': ['"@eclipse-sirius/sirius-web-application": "2026.9.3"'],
    'compose': ['image: "${IMAGE_TAG:-eclipsesyson/syson:v2026.9.0}"', 'SIRIUS_COMPONENTS_CORS_ALLOWEDORIGINPATTERNS: "*"'],
    'application-compose': ['image: "${IMAGE_TAG:-eclipsesyson/syson:latest}"'],
    'dockerfile': ['COPY target/syson-application*[^sources].jar /syson-application.jar'],
    'security-manual': ['does not include built-in mechanisms for authentication or authorization'],
    'production-manual': ['all necessary security configurations', '== Secure the application', '--server.ssl.key-store='],
}
rows = []
data = {}
for id, path in m.SOURCES.items():
    raw = (ROOT / path).read_bytes()
    data[id] = raw.decode()
    lines = data[id].splitlines()
    captures = []
    for anchor in anchors[id]:
        indexes = [i for i, line in enumerate(lines) if anchor in line]
        if not indexes:
            raise ValueError((id, anchor))
        i = indexes[0]
        captures.append(dict(first_line=i+1, last_line=i+1, text=lines[i]))
    rows.append(dict(id=id, path=path, sha256=hashlib.sha256(raw).hexdigest(), anchors=captures))
manifest = dict(upstream_commit=m.UPSTREAM, evidence=rows)
gaps = [
 ('authentication', ['starter', 'security-manual'], 'The manual states no built-in authentication. Inherited request filters and external gateway policy are not bound to an executed artifact.', 'Exact inherited authentication/filter sources and reviewed deployment mode, then anonymous/authenticated request captures.'),
 ('authorization', ['object-delegate', 'security-manual'], 'The SysON delegate receives an editing context and forwards lookup. That method is not the HTTP authorization boundary.', 'Inherited route/method authorization plus project permission enforcement and caller-to-context binding.'),
 ('session-cookie', ['starter', 'frontend-entry'], 'Session identity, login/logout lifecycle and cookie attributes are inherited or deployment-specific; no runtime cookie receipt is supplied.', 'Resolved server/client session implementation, effective cookie configuration and login/expiry/logout captures.'),
 ('csrf', ['starter', 'entry'], 'Auto-configuration and a starter dependency identify a boundary, without supplying the effective CSRF filter chain or command entry policy.', 'Resolved effective CSRF configuration and state-changing HTTP tests for the selected session mode.'),
 ('cors', ['properties', 'dev-properties', 'compose'], 'Source properties declare wildcard origin patterns and credentials=true. The actual consuming configuration, profile/environment precedence and headers are unexecuted.', 'Inherited CORS consumer, effective configuration and preflight/credentialed response observations.'),
 ('websocket', ['frontend-entry', 'frontend-url'], 'The application passes HTTP/WS origins into SiriusWebApplication; credential forwarding, upgrade authorization, origin checks and revocation are inherited.', 'Resolved transport/upgrade/subscription implementation and cross-origin/revocation captures.'),
 ('project-context', ['project-delegate', 'object-delegate', 'read-only'], 'Semantic-data-to-project mapping and object/library read-only decisions are visible. Principal permission checks and request-to-editing-context selection are outside those local methods.', 'Resolved request/context lookup and principal/project checks; equal element IDs in separate synthetic projects.'),
 ('artifact-binding', ['pom', 'compose', 'application-compose', 'dockerfile', 'production-manual'], 'Source version2026.9.3 differs from the root compose default v2026.9.0; the other compose uses latest and Dockerfile copies a wildcard JAR. No supplied runtime digest binds them.', 'Exact JAR/image digest and dependency/source map, effective config/profile/gateway policy and corresponding runtime capture.'),
]
contract = dict(schema='SYSON-SESSION-ISOLATION-SOURCE/1', status=m.STATUS,
 source_projection=m.projection(data), host_adapter=None,
 claims=dict(runtime_executed=False, native_policy_qualified=False, deployed_correspondence=False),
 unresolved=[dict(id=id, status='UNRESOLVED', reason=why, source_ids=ids, required_evidence=need) for id, ids, why, need in gaps],
 artifact=dict(state='UNBOUND', jar_sha256=None, image_digest=None, effective_configuration_sha256=None, policy_source_sha256=None))
scenarios = [
 ('anonymous-read', ['authentication'], ['starter', 'security-manual', 'object-delegate'], 'Two synthetic projects; request has no established principal.', 'Read an element through the actual admitted REST entry point.', 'Record the selected anonymous-access policy and show its enforcement before choosing an authenticated Rust mode.'),
 ('forbidden-element', ['authorization', 'project-context'], ['object-delegate', 'project-delegate'], 'Synthetic principal A has access only to project A; element B belongs to project B.', 'Ask the admitted element operation for B while authenticated as A.', 'For a proposed isolated host mode, reject disclosure across the reviewed project permission boundary; exact native HTTP outcome remains unqualified.'),
 ('forbidden-relationships', ['authorization', 'project-context'], ['object-delegate'], 'Synthetic A is restricted to project A; related element B belongs to project B.', 'Query the relationships operation using B and both directions.', 'Check denial/no foreign relationship data, rather than accepting an empty-data fixture as evidence of authorization.'),
 ('same-object-id-two-projects', ['project-context'], ['object-delegate', 'project-delegate'], 'Projects A and B contain the same element ID but different marker data.', 'Read by ID in each allowed context and attempt the other project context.', 'Require identity to retain project context and detect accidental ID-only lookup; principal policy must be reviewed first.'),
 ('forged-context', ['project-context', 'authorization'], ['project-delegate'], 'Synthetic principal A; request pairs project A with semantic-data/editing-context ID B.', 'Send the mismatched identifiers through real request/context resolution.', 'Reject a foreign context substitution in the proposed isolated host mode; mapping existence alone is not authorization.'),
 ('expired-session', ['session-cookie', 'authentication'], ['starter', 'frontend-entry'], 'A disposable test session has expired under the bound effective configuration.', 'Repeat read and command requests with the expired session identifier.', 'Record expiry enforcement and prove no cached identity is accepted without revalidation.'),
 ('logout-subscription', ['session-cookie', 'websocket'], ['frontend-entry', 'frontend-url'], 'A synthetic subscribed session is revoked using the selected logout mechanism.', 'Observe the existing subscription and attempt reconnect after revocation.', 'Define and verify revocation semantics for active and reconnecting transports; no native lifecycle is presumed.'),
 ('cross-origin-preflight', ['cors'], ['properties', 'dev-properties', 'compose'], 'Disposable origins allowed.invalid and denied.invalid under a reviewed effective policy.', 'Send preflight and credentialed requests from each origin.', 'Capture actual allow-origin/credentials headers and browser result; source property declarations do not decide runtime policy.'),
 ('websocket-origin', ['websocket', 'cors'], ['frontend-entry', 'frontend-url'], 'Synthetic upgrade request supplies a disallowed origin and separately an expired identity.', 'Attempt the actual inherited WebSocket upgrade and subscription operation.', 'Check upgrade and subscription gates separately from HTTP CORS; record the chosen origin/identity policy.'),
 ('csrf-command', ['csrf', 'session-cookie'], ['starter', 'entry'], 'A synthetic cookie session and a state-changing command; only disposable model data.', 'Compare missing/invalid/valid anti-CSRF evidence through the actual command entry.', 'Determine whether CSRF applies to the selected authentication mode, then verify the effective protection or an explicitly reviewed alternative.'),
 ('cookie-attributes', ['session-cookie'], ['starter', 'production-manual'], 'Bound HTTPS/proxy configuration and a synthetic login/session establishment.', 'Inspect actual Set-Cookie attributes and subsequent requests without storing session secrets.', 'Qualify Secure/HttpOnly/SameSite/domain/path/lifetime against explicit deployment policy; TLS documentation alone supplies no cookie proof.'),
 ('artifact-mismatch', ['artifact-binding'], ['pom', 'compose', 'application-compose', 'dockerfile'], 'A source checkout and a candidate JAR/image whose digest has not been reconciled.', 'Compare the exact dependency/source/config receipt with the artifact used for all observations.', 'Refuse attribution of runtime results to source until the artifact and effective policy are bound.'),
]
negative = dict(schema='SYSON-SESSION-ISOLATION-SCENARIOS/1', execution='NOT_RUN', cases=[dict(id=id, boundaries=b, source_ids=s, synthetic_setup=setup, probe=probe, candidate_goal=goal, native_expected='UNRESOLVED', receipt=None) for id,b,s,setup,probe,goal in scenarios])
for name, value in [('source-manifest.json', manifest), ('contract.json', contract), ('negative-scenarios.json', negative)]:
    (BASE / name).write_text(json.dumps(value, indent=2) + '\n')
print('Created documentary packet:', len(rows), 'source pins;', len(scenarios), 'unrun scenarios')
