#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Qualify source evidence and unrun scenarios; does not evaluate authorization."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import subprocess
import xml.etree.ElementTree as ET

UPSTREAM = '6ba2ac64e6b16ee2b05752ad55395eebb87d4910'
STATUS = 'BLOCKED_SOURCE_PREPARATION_ONLY'
NS = {'m': 'http://maven.apache.org/POM/4.0.0'}
SOURCES = {
    'pom': 'pom.xml',
    'application-pom': 'backend/application/syson-application/pom.xml',
    'starter-pom': 'backend/application/syson-starter/pom.xml',
    'starter': 'backend/application/syson-starter/src/main/java/org/eclipse/syson/starter/SysONStarterConfiguration.java',
    'entry': 'backend/application/syson-application/src/main/java/org/eclipse/syson/SysONApplication.java',
    'properties': 'backend/application/syson-application/src/main/resources/application.properties',
    'dev-properties': 'backend/application/syson-application/src/main/resources/application-dev.properties',
    'object-delegate': 'backend/services/syson-sysml-rest-api-services/src/main/java/org/eclipse/syson/sysml/rest/api/SysONObjectRestService.java',
    'project-delegate': 'backend/services/syson-sysml-rest-api-services/src/main/java/org/eclipse/syson/sysml/rest/api/SysONProjectDataVersioningRestService.java',
    'read-only': 'backend/application/syson-application-configuration/src/main/java/org/eclipse/syson/application/services/SysONReadOnlyObjectPredicateDelegate.java',
    'frontend-entry': 'frontend/syson/src/index.tsx',
    'frontend-url': 'frontend/syson/src/core/URL.ts',
    'frontend-package': 'frontend/syson/package.json',
    'compose': 'docker-compose.yml',
    'application-compose': 'backend/application/syson-application/docker-compose.yml',
    'dockerfile': 'backend/application/syson-application/Dockerfile',
    'security-manual': 'doc/content/modules/user-manual/pages/features/security.adoc',
    'production-manual': 'doc/content/modules/installation-guide/pages/how-tos/install/production_deploy.adoc',
}
GAPS = {'authentication', 'authorization', 'session-cookie', 'csrf', 'cors',
        'websocket', 'project-context', 'artifact-binding'}
CASES = {'anonymous-read', 'forbidden-element', 'forbidden-relationships',
         'same-object-id-two-projects', 'forged-context', 'expired-session',
         'logout-subscription', 'cross-origin-preflight', 'websocket-origin',
         'csrf-command', 'cookie-attributes', 'artifact-mismatch'}


class Refusal(ValueError):
    pass


def require(value, message):
    if not value:
        raise Refusal(message)


def pairs(rows):
    result = {}
    for key, value in rows:
        require(key not in result, 'duplicate JSON key')
        result[key] = value
    return result


def read_json(path):
    return json.loads(path.read_text(), object_pairs_hook=pairs)


def keys(value, expected):
    require(isinstance(value, dict) and set(value) == set(expected), 'unexpected fields')


def verify_sources(root, manifest, reader=None):
    keys(manifest, ['upstream_commit', 'evidence'])
    require(manifest['upstream_commit'] == UPSTREAM, 'wrong upstream revision')
    rows = manifest['evidence']
    require(isinstance(rows, list) and len(rows) == len(SOURCES), 'source count')
    require({r.get('id') for r in rows} == set(SOURCES), 'source identities')
    data = {}
    for row in rows:
        keys(row, ['id', 'path', 'sha256', 'anchors'])
        relative = PurePosixPath(row['path'])
        require(not relative.is_absolute() and '..' not in relative.parts, 'unsafe source path')
        require(row['path'] == SOURCES[row['id']], 'source path mismatch')
        if reader is None:
            current = (root / row['path']).read_bytes()
            pinned = subprocess.check_output(['git', '-C', str(root), 'show', UPSTREAM + ':' + row['path']])
            require(current == pinned, 'working source differs from pinned upstream')
        else:
            current = reader(row['path'])
        require(hashlib.sha256(current).hexdigest() == row['sha256'], 'source digest mismatch')
        text = current.decode('utf-8')
        require(isinstance(row['anchors'], list) and bool(row['anchors']), 'anchors missing')
        for anchor in row['anchors']:
            keys(anchor, ['first_line', 'last_line', 'text'])
            lo, hi = anchor['first_line'], anchor['last_line']
            require(type(lo) is int and type(hi) is int and 1 <= lo <= hi, 'invalid anchor range')
            require('\n'.join(text.splitlines()[lo-1:hi]) == anchor['text'], 'anchor drift')
        data[row['id']] = text
    return data


def properties(text):
    values = {}
    for line in text.splitlines():
        line = line.strip()
        if line and not line.startswith('#') and '=' in line:
            key, value = map(str.strip, line.split('=', 1))
            require(key not in values, 'duplicate source property')
            values[key] = value
    return values


def projection(data):
    pom = ET.fromstring(data['pom'])
    starter = ET.fromstring(data['starter-pom'])
    dependencies = starter.findall('m:dependencies/m:dependency', NS)
    matching = [d for d in dependencies if d.findtext('m:groupId', namespaces=NS) == 'org.eclipse.sirius'
                and d.findtext('m:artifactId', namespaces=NS) == 'sirius-web-starter']
    require(len(matching) == 1, 'inherited starter declaration ambiguous')
    result = {
        'source_version': pom.findtext('m:version', namespaces=NS),
        'spring_boot_parent': pom.findtext('m:parent/m:version', namespaces=NS),
        'sirius_web_property': pom.findtext('m:properties/m:sirius.web.version', namespaces=NS),
        'starter_version_expression': matching[0].findtext('m:version', namespaces=NS),
        'frontend_application_version': json.loads(data['frontend-package'])['dependencies']['@eclipse-sirius/sirius-web-application'],
    }
    for name in ['properties', 'dev-properties']:
        values = properties(data[name])
        result[name] = {key: values[key] for key in [
            'sirius.components.cors.allowedOriginPatterns', 'sirius.components.cors.allowedCredentials']}
    return result


def validate_contract(contract, scenarios, source_data):
    keys(contract, ['schema', 'status', 'source_projection', 'host_adapter', 'claims', 'unresolved', 'artifact'])
    require(contract['schema'] == 'SYSON-SESSION-ISOLATION-SOURCE/1', 'schema')
    require(contract['status'] == STATUS, 'source preparation cannot grant acceptance')
    require(contract['host_adapter'] is None, 'host adapter requires later policy review')
    require(contract['source_projection'] == projection(source_data), 'source projection mismatch')
    keys(contract['claims'], ['runtime_executed', 'native_policy_qualified', 'deployed_correspondence'])
    require(all(v is False for v in contract['claims'].values()), 'source-only claim escalation')
    gaps = contract['unresolved']
    require(isinstance(gaps, list) and len(gaps) == len(GAPS), 'missing boundary')
    require({g.get('id') for g in gaps} == GAPS, 'boundary identities')
    for gap in gaps:
        keys(gap, ['id', 'status', 'reason', 'source_ids', 'required_evidence'])
        require(gap['status'] == 'UNRESOLVED', 'unexecuted native boundary promoted')
        require(isinstance(gap['source_ids'], list) and bool(gap['source_ids'])
                and set(gap['source_ids']) <= set(SOURCES), 'boundary source links')
        require(all(isinstance(gap[k], str) and gap[k].strip() for k in ['reason', 'required_evidence']), 'boundary explanation')
    keys(contract['artifact'], ['state', 'jar_sha256', 'image_digest', 'effective_configuration_sha256', 'policy_source_sha256'])
    require(contract['artifact']['state'] == 'UNBOUND', 'deployment binding unproven')
    require(all(v is None for k, v in contract['artifact'].items() if k != 'state'), 'unverified artifact receipt')
    keys(scenarios, ['schema', 'execution', 'cases'])
    require(scenarios['schema'] == 'SYSON-SESSION-ISOLATION-SCENARIOS/1', 'scenario schema')
    require(scenarios['execution'] == 'NOT_RUN', 'scenario execution unqualified')
    cases = scenarios['cases']
    require(isinstance(cases, list) and len(cases) == len(CASES), 'scenario count')
    require({c.get('id') for c in cases} == CASES, 'scenario identities')
    covered = set()
    for case in cases:
        keys(case, ['id', 'boundaries', 'source_ids', 'synthetic_setup', 'probe', 'candidate_goal', 'native_expected', 'receipt'])
        require(isinstance(case['boundaries'], list) and bool(case['boundaries'])
                and set(case['boundaries']) <= GAPS, 'scenario boundaries')
        covered.update(case['boundaries'])
        require(isinstance(case['source_ids'], list) and bool(case['source_ids'])
                and set(case['source_ids']) <= set(SOURCES), 'scenario source links')
        require(case['native_expected'] == 'UNRESOLVED' and case['receipt'] is None, 'unknown native result promoted')
        require(all(isinstance(case[k], str) and case[k].strip() for k in ['synthetic_setup', 'probe', 'candidate_goal']), 'scenario detail')
    require(covered == GAPS, 'uncovered boundary')
    return {'checker': 'PASS', 'qualification': STATUS, 'source_pins': len(SOURCES),
            'negative_scenarios': len(CASES), 'native_runtime': 'NOT_RUN', 'host_adapter': None}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[4])
    args = parser.parse_args()
    folder = args.root / 'porting/area-inventories/API/session-isolation'
    sources = verify_sources(args.root, read_json(folder / 'source-manifest.json'))
    result = validate_contract(read_json(folder / 'contract.json'), read_json(folder / 'negative-scenarios.json'), sources)
    print(json.dumps(result, sort_keys=True))


if __name__ == '__main__':
    main()
