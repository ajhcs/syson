"""Verify the JDT Core proposal against the exact offline observation."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

CORE = Path(__file__).resolve().parent
MANIFEST = CORE / 'requests/eclipse-jdt-core-descriptors-01.json'
OBSERVATION_COMMIT = '960fd31bc91d8fd37b51493d2dfbadef034ad40b'
OBSERVATION_PATH = 'porting/rust-parity-evidence/C01/eclipse-range-descriptor-frontier/observation.json'
OBSERVATION_SHA256 = '459861665f2c3798672748b10ba2470c76f8c8931230d19eaaaae10dc7475dbe'
CLIENT_COMMIT = 'a6758b8df743ddae19e75cf217553d6b54c5e441'
JDT = 'org.eclipse.jdt:org.eclipse.jdt.core'
RANGES = (
    (JDT, 22),
    ('org.eclipse.jdt:org.eclipse.jdt.launching', 40),
    ('org.eclipse.platform:org.eclipse.core.resources', 35),
    ('org.eclipse.platform:org.eclipse.core.runtime', 33),
    ('org.eclipse.platform:org.eclipse.text', 29),
    ('org.eclipse.platform:org.eclipse.debug.core', 34),
)
COLLECTORS = ('codegen-ancestor', 'ecore-ancestor', 'target')
TARGET_MODEL_REQUEST = [
    ['org.apache.commons:commons-text:1.10.0', 'coordinate', 'org.junit:junit-bom:5.9.1'],
]
MIB = 1024**2


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def seal(rows):
    return sha(json.dumps(rows, separators=(',', ':')).encode())


def pom_path(gav):
    group, artifact, version = gav.split(':')
    return f'{group.replace(".", "/")}/{artifact}/{version}/{artifact}-{version}.pom'


def verify(observation_bytes, manifest):
    require(len(observation_bytes) <= 65536 and sha(observation_bytes) == OBSERVATION_SHA256,
            'exact #364 observation required')
    obs = json.loads(observation_bytes)
    require(obs['schema'] == 'c01-eclipse-range-descriptor-frontier-v1', 'unknown observation schema')
    require(manifest['source_bindings'] == {
        'pinned_client_commit': CLIENT_COMMIT,
        'observation_commit': OBSERVATION_COMMIT,
        'observation_path': OBSERVATION_PATH,
        'observation_sha256': OBSERVATION_SHA256,
        'offline_scope_sha256': obs['scope_sha256'],
        'offline_receipt_sha256': obs['receipt_sha256'],
        'target_revision': obs['target_revision'],
    }, 'source bindings changed')
    require(manifest['authorization'] == {k: False for k in obs['authority']}
            and all(value is False for value in manifest['authorization'].values())
            and manifest['status'] == 'PROPOSED_FINITE_DESCRIPTOR_EVIDENCE_EPOCH_NO_EXECUTION_AUTHORITY',
            'authority must remain closed')

    ranges = obs['missing_range_descriptor_gavs']
    cases = obs['cases']
    for name in COLLECTORS:
        case = cases[name]
        require(case['attempted_range_gavs_sha256'] == seal(ranges)
                and case['terminal'] == ['partial', 'no-binary-resolution'], 'collector evidence changed')
    withheld = cases['withheld-metadata']
    require(withheld['offered_version_count'] == withheld['descriptor_request_rows'] == 0,
            'JDT Core withholding control changed')
    target = cases['target']
    require(target['missing_model_requests'] == TARGET_MODEL_REQUEST, 'target model import changed')

    cohorts = []
    all_paths = []

    def cohort(identifier, role, gavs, count):
        paths = sorted(map(pom_path, gavs))
        require(len(paths) == len(set(paths)) == count, f'{identifier} count or uniqueness changed')
        cohorts.append({'identifier': identifier, 'role': role, 'pom_count': count, 'paths_sha256': seal(paths)})
        all_paths.extend(paths)
        return paths

    families = obs['metadata_families']
    require(len(families) == len(RANGES), 'range family count changed')
    for family, (ga, count) in zip(families, RANGES, strict=True):
        require(family['ga'] == ga, 'range family changed')
        gavs = [gav for gav in ranges if gav.rsplit(':', 1)[0] == ga]
        require(gavs == sorted(f'{ga}:{v}' for v in family['offered_versions']), 'range offer drift')
        paths = cohort(ga, 'range_candidate_descriptor', gavs, count)
        if ga == JDT:
            selected = paths
            jdt = family
    require(len(all_paths) == len(ranges) == 193, 'range candidate partition incomplete')
    cohort('target-fixed-descriptors', 'fixed_descriptor', target['other_missing_descriptor_gavs'], 21)
    cohort('target-model-import', 'model_import', [TARGET_MODEL_REQUEST[0][2]], 1)
    require(len(all_paths) == len(set(all_paths)) == 215, 'full frontier changed')
    require(manifest['frontier'] == {
        'cohorts': cohorts, 'observed_pom_count': len(all_paths),
        'observed_paths_sha256': seal(sorted(all_paths)), 'proposed_cohort': JDT,
        'remaining_pom_count': len(all_paths) - len(selected),
        'range_interval': jdt['interval'], 'range_metadata_sha256': jdt['metadata_sha256'],
        'case_trace_sha256': {name: row['stdout_sha256'] for name, row in cases.items()},
    }, 'partition or provenance changed')
    require([row['relative_artifact'] for row in manifest['new_primaries']] == selected
            and all(row['kind'] == 'pom' for row in manifest['new_primaries']), 'JDT Core proposal drift')

    limits = manifest['limits']
    requests = manifest['requests']
    body_note = manifest['body_bound_note']
    response_copies = len(requests) * 2 * (6 * 16384 + 5120 + 4096)
    metadata = manifest['nominal_metadata_charge']
    require(
        len(requests) == limits['exact_requests'] == 110
        and limits['exact_primaries'] == len(selected)
        and body_note['sum_individual_caps_bytes'] == sum(row['maximum_bytes'] for row in requests) == 51904512
        and body_note['aggregate_refusal_cap_bytes'] == limits['payload_and_sidecar_total_bytes'] == 16 * MIB
        and body_note['completion_within_aggregate_promised'] is False
        and metadata['requests'] == len(requests) and metadata['two_escaped_response_copies_bytes'] == response_copies
        and metadata['getter_nominal_bytes'] == response_copies + 2 * MIB == 25751552
        and metadata['getter_nominal_bytes'] <= metadata['getter_limit'] == 63 * MIB
        and limits['total_logical_owned_write_bound_bytes'] == 80 * MIB,
        'bounded response or payload profile changed',
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('observation', help='Exact #364 observation file, or - to read it on stdin')
    args = parser.parse_args()
    if args.observation == '-':
        raw = sys.stdin.buffer.read(65537)
    else:
        with Path(args.observation).open('rb') as stream:
            raw = stream.read(65537)
    with MANIFEST.open('rb') as stream:
        proposed = stream.read(256 * 1024 + 1)
    require(len(proposed) <= 256 * 1024, 'manifest exceeds bound')
    verify(raw, json.loads(proposed))
    spec = importlib.util.spec_from_file_location('fixed_core_client', CORE / 'acquire.py')
    client = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(client)
    client.requests('eclipse-jdt-core-descriptors-01', proposed)
    print('PASS: 215 observed POM paths in 8 cohorts; 22 JDT Core POMs proposed, 193 POMs outside this proposal')


if __name__ == '__main__':
    main()
