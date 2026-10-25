#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Validate frozen source/preflight data, never run the proposed build or launch."""
import hashlib
import json
from pathlib import Path, PurePosixPath
import subprocess
import xml.etree.ElementTree as ET

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
UPSTREAM = '6ba2ac64e6b16ee2b05752ad55395eebb87d4910'
BASE = 'ad814c232ee24109639c1d52239cbeec6f4a3e92'


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def read_json(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, 'duplicate JSON key')
            result[key] = value
        return result
    return json.loads(raw, object_pairs_hook=pairs)


def workspace_path(value, prefix):
    require(isinstance(value, str), 'workspace path type')
    path = PurePosixPath(value)
    require(path.is_absolute() and '..' not in path.parts and str(path) == value,
            'noncanonical workspace path')
    require(value.startswith(prefix) and len(value) > len(prefix), 'outside admitted workspace prefix')
    return value


def verify_sources():
    manifest = read_json((HERE / 'source-manifest.json').read_text())
    seen = set()
    for source in manifest['inputs']:
        path = source['path']
        require(path not in seen and not PurePosixPath(path).is_absolute()
                and '..' not in PurePosixPath(path).parts, 'invalid/duplicate source path')
        seen.add(path)
        require(source['revision'] in {UPSTREAM, BASE}, 'unexpected source pin')
        spec = source['revision'] + ':' + path
        raw = subprocess.check_output(['git', '-C', str(ROOT), 'show', spec])
        blob = subprocess.check_output(['git', '-C', str(ROOT), 'rev-parse', spec], text=True).strip()
        require(source['sha256'] == sha(raw) and source['blob'] == blob, 'source object drift')
        require((ROOT / path).read_bytes() == raw, 'worktree source drift')
    require(len(seen) == 13, 'source closure shape')
    return len(seen)


def verify_facts(plan):
    parent = read_json((ROOT / 'package.json').read_text())
    java = ET.parse(ROOT / 'pom.xml').getroot().find(
        '{http://maven.apache.org/POM/4.0.0}properties/{http://maven.apache.org/POM/4.0.0}java.version').text
    require(plan['requirements'] == {'java_major': java, **parent['engines'],
            'maven': 'VERSION_NOT_PINNED_BY_INSPECTED_SOURCE'}, 'source tool requirement drift')
    require(set(plan['source_scripts']) == {'package.json', 'frontend/syson/package.json',
                                          'frontend/syson-components/package.json'}, 'script scope')
    for name, scripts in plan['source_scripts'].items():
        require(scripts == read_json((ROOT / name).read_text())['scripts'], 'source lifecycle drift')
    turbo = read_json((ROOT / 'turbo.json').read_text())['tasks']
    facts = plan['source_observations']
    require(facts['turbo_build_depends_on'] == turbo['build']['dependsOn']
            and facts['turbo_start_persistent'] == turbo['start']['persistent'], 'Turbo fact drift')
    vite = (ROOT / 'frontend/syson/vite.config.js').read_text()
    require(facts['vite_env_spread_and_loadEnv'] and '...process.env' in vite and 'loadEnv(' in vite,
            'environment input boundary missing')
    workflow = (ROOT / '.github/workflows/build.yml').read_text()
    require(facts['ci_maven_uses_mutable_snapshot_update_flag']
            and all(command in workflow for command in facts['ci_commands']), 'CI source fact drift')


def verify_tool_capture(inventory):
    require(inventory['credential_files_read'] is False and inventory['network_probe'] == 'NOT_RUN'
            and inventory['registry_entitlement'] == 'NOT_TESTED'
            and inventory['lifecycle_execution'] == 'NOT_RUN', 'inventory scope promotion')
    for record in inventory['commands']:
        if 'argv' not in record:
            require(record['invocation'] == 'NOT_RUN', 'unobserved tool invocation')
            continue
        name = record['name']
        require(name in {'java', 'node'} or name.startswith('node-alternate-'), 'unknown version probe')
        require(record['argv'][1:] == (['-version'] if name == 'java' else ['--version']),
                'non-version command in inventory')
        folder = record['capture_directory']
        require(folder in {'tool-inventory-01', 'tool-inventory-02'}, 'capture path')
        stem = 'node-alt-' + name.rsplit('-', 1)[1] if name.startswith('node-alternate-') else name
        directory = ROOT / 'porting/follow-ons/F39/validation' / folder
        stdout = (directory / (stem + '-stdout.txt')).read_bytes()
        stderr = (directory / (stem + '-stderr.txt')).read_bytes()
        require(sha(stdout) == record['stdout_sha256'] and sha(stderr) == record['stderr_sha256'],
                'version output drift')
        require(record['exit_code'] == 0 and record['observed_version'] in (stdout + stderr).decode(),
                'version observation mismatch')
        require(record['whole_distribution_pin'] == 'NOT_ESTABLISHED', 'executable hash is not distribution closure')
    require(all(not cache['artifact_bytes_verified'] for cache in inventory['caches']),
            'directory visibility is not artifact closure')


def validate_plan(plan):
    require(plan['schema'] == 1 and plan['source_revision'] == UPSTREAM, 'preflight source pin')
    require(plan['execution'] == 'NOT_RUN' and plan['requires_separate_action_review'] is True,
            'execution authority cannot be inferred')
    require(plan['status'] == 'BLOCKED_PREPARATION_ONLY', 'preparation status promotion')
    require(plan['registry_access'] == {'authorization': 'NOT_TESTED', 'network_probe': 'NOT_RUN',
            'missing_or_unobserved_is_not_denial': True}, 'registry denial/access claim without probe')
    paths = plan['workspace']
    workspace_path(paths['source_checkout'], '/workspace/worktrees/builderbob/syson/reference-')
    for name in ['capture_directory', 'database_directory']:
        workspace_path(paths[name], '/workspace/artifacts/builderbob/syson/')
    for name in ['maven_repository', 'npm_cache']:
        workspace_path(paths[name], '/workspace/cache/builderbob/')
    require(paths['maven_repository'] != paths['npm_cache'], 'cache roots collide')
    network = plan['network']
    require(network['bind_address'] == '127.0.0.1' and network['public_exposure'] == 'FORBIDDEN',
            'non-loopback target')
    ports = [network['application_port'], network['database_port']]
    require(all(type(port) is int and 1024 <= port <= 65535 for port in ports)
            and ports[0] != ports[1], 'synthetic port collision/range')
    require(network['port_availability'] == 'NOT_CHECKED' and paths['destinations_verified_fresh'] is False,
            'unprobed destination claim')
    limits = plan['limits']
    require(limits['compiler_jobs'] == 1 and limits['owned_process_group_only'] is True,
            'resource/ownership boundary')
    require(all(limits[key] is False for key in ['automatic_cleanup', 'service_registration', 'system_tool_installation']),
            'unadmitted state change')
    require('new_installations' not in limits
            and limits['project_dependency_installation'] == 'REVIEW_ONLY_NOT_RUN',
            'project installation is distinct from forbidden system baseline installation')
    steps = plan['command_templates']
    require([step['stage'] for step in steps] == ['checkout', 'project-dependency-installation',
            'frontend-build', 'backend-build', 'launch'], 'command stage order')
    for step in steps:
        require(all(isinstance(arg, str) and arg and '\n' not in arg and '\r' not in arg
                    for arg in step['argv']), 'command argument encoding')
    require(steps[0]['argv'] == ['git', 'worktree', 'add', '--detach', paths['source_checkout'], UPSTREAM],
            'checkout is not frozen local source')
    require(steps[1]['argv'] == ['<NPM11_BINARY>', 'ci', '--offline', '--ignore-scripts', '--cache', paths['npm_cache']],
            'unreviewed dependency/lifecycle action')
    require(steps[2]['argv'] == ['<NPM11_BINARY>', 'run', 'build'], 'frontend action changed')
    require(steps[3]['argv'] == ['<MAVEN_BINARY>', '--offline', '-B', '-e', '-T1',
            '-Dmaven.repo.local=' + paths['maven_repository'], '--settings', '<REVIEWED_SETTINGS_FILE>',
            'clean', 'verify'], 'mutable/changed Maven action')
    require(steps[4]['argv'] == ['<JAVA21_BINARY>', '-jar', '<BUILT_APPLICATION_JAR>',
            '--server.address=127.0.0.1', '--server.port=' + str(ports[0]),
            '--spring.config.additional-location=<SYNTHETIC_APPLICATION_CONFIG>'], 'launch target changed')
    require(set(plan['bindings']) == {'JAVA21_BINARY', 'NODE24_BINARY', 'NPM11_BINARY', 'MAVEN_BINARY',
            'CURATED_BUILD_ENVIRONMENT', 'REVIEWED_SETTINGS_FILE', 'SYNTHETIC_APPLICATION_CONFIG',
            'BUILT_APPLICATION_JAR', 'JDBC_DATABASE_BASELINE'}, 'binding manifest shape')
    require(all(value is None for value in plan['bindings'].values()), 'unreviewed executable/config binding')
    require(len(plan['stop_conditions']) >= 6 and len(plan['retention']) >= 4, 'missing stop/retention conditions')
    snapshot = plan['dependency_closure']['snapshot']
    require(snapshot['coordinate'] == 'org.eclipse.sirius:org.eclipse.sirius.emfjson:2.5.5-SNAPSHOT'
            and all(snapshot[k] is None for k in ['timestamp', 'build_number', 'artifact_sha256', 'source_sha256']),
            'unresolved SNAPSHOT promoted')
    require(plan['dependency_closure']['artifact_source_correspondence'] == 'UNPROVEN'
            and plan['dependency_closure']['maven_effective_graph'] == 'UNRESOLVED'
            and plan['dependency_closure']['npm_archive_integrity'] == 'NOT_INDEPENDENTLY_VERIFIED',
            'source-to-artifact/dependency promotion')
    return {'status': 'BLOCKED', 'execution': 'NOT_RUN', 'checker': 'PASS',
            'blockers': ['exact compatible toolchain/whole-distribution pins',
                'effective Maven/plugin graph and immutable SNAPSHOT/artifact/source correspondence',
                'independent npm archive integrity and untested registry entitlement',
                'reviewed allowlisted build environment/configuration (Vite process.env/loadEnv)',
                'fresh workspace/ports and isolated synthetic database/history/extension baseline',
                'separate exact-action and resource allocation review']}


def main():
    count = verify_sources()
    plan = read_json((HERE / 'preflight.json').read_text())
    raw = (HERE / 'tool-inventory.json').read_bytes()
    require(sha(raw) == plan['tool_inventory_sha256'], 'tool inventory drift')
    verify_tool_capture(read_json(raw))
    verify_facts(plan)
    result = validate_plan(plan)
    result['pinned_sources'] = count
    print(json.dumps(result))


if __name__ == '__main__':
    main()
