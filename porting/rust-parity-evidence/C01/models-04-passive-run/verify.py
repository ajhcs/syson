"""Validate one passive JUnit BOM capture against the pinned models-04 roster."""
from hashlib import sha256
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
MANIFEST = (HERE.parents[2] / 'rust-parity-plan-20261026' / 'closure' /
            'core-compile' / 'requests' / 'models-04.json')
MANIFEST_SHA = 'e0d4b420cdcdcf23023731af35511fb7fb78402e9d492a87840800c661814289'
OBSERVATION_SHA = '030238fc5164905fae81e2683572b8d8441a034d134ba0ef6341c6e77d112de6'
SHA256 = re.compile(r'[0-9a-f]{64}')
CONTROLS = {
    'cpu.max': '100000 100000',
    'memory.high': '536870912',
    'memory.max': '1073741824',
    'memory.swap.max': '0',
    'pids.max': '32',
}


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def unique_object(pairs):
    values = {}
    for key, value in pairs:
        require(key not in values, 'duplicate JSON key')
        values[key] = value
    return values


def sealed_json(path, expected):
    raw = path.read_bytes()
    require(sha256(raw).hexdigest() == expected, 'evidence seal changed')
    return json.loads(raw, object_pairs_hook=unique_object)


def verify(manifest, observation):
    require(observation['schema'] == 'SYSON-C01-MODELS04-PASSIVE-RUN/1' and
            observation['status'] == 'PASS_PASSIVE_ACQUISITION_ONLY' and
            observation['epoch'] == manifest['epoch'] == 'models-04' and
            observation['manifest_sha256'] == MANIFEST_SHA,
            'wrong acquisition binding')
    requests = manifest['requests']
    rows = observation['request_rows']
    require(len(rows) == len(requests) == 5, 'request count changed')
    for row, request in zip(rows, requests):
        require(set(row) == {'relative_path', 'role', 'status', 'bytes', 'sha256'} and
                row['relative_path'] == request['relative_path'] and
                row['role'] == request['role'] and
                type(row['status']) is int and row['status'] == 200 and
                row['status'] in request['allowed_status'] and
                type(row['bytes']) is int and 0 <= row['bytes'] <= request['maximum_bytes'] and
                SHA256.fullmatch(row['sha256']), 'wrong response row')
    primary, = manifest['new_primaries']
    checksums = [{'algorithm': request['algorithm'], 'artifact': primary['relative_artifact']}
                 for request in requests if request['role'] == 'checksum']
    require([row['algorithm'] for row in checksums] == ['sha512', 'sha256', 'sha1'] and
            observation['verified_same_origin_checksums'] == checksums,
            'wrong checksum record')
    model = observation['primary_model']
    require(model == {
        'coordinate': primary['coordinate'],
        'packaging': 'pom',
        'bytes': rows[0]['bytes'],
        'sha256': rows[0]['sha256'],
        'parent': None,
        'profiles': [],
        'direct_dependency_count': 0,
        'managed_dependency_count': 18,
        'managed_import_count': 0,
        'managed_jupiter_api': {
            'groupId': 'org.junit.jupiter', 'artifactId': 'junit-jupiter-api', 'version': '5.12.1',
        },
    }, 'wrong source model projection')
    require(observation['controls'] == CONTROLS and
            observation['controller_exit'] == 0 and
            observation['owned_scope_cleanup'] == 'not-found' and
            observation['output_file_count'] == 12 and
            observation['frozen_input_count'] == 134 and
            observation['frozen_inputs_before_after_and_review_match'] is True,
            'wrong scope observation')
    limits = manifest['limits']
    budgets = (
        ('body_bytes', 'payload_and_sidecar_total_bytes'),
        ('getter_metadata_bytes', 'getter_metadata_proof_receipt_bytes'),
        ('controller_json_bytes', 'root_controller_JSON_bytes'),
        ('controller_log_bytes', 'root_child_stdout_stderr_bytes'),
    )
    require(all(type(observation[name]) is int and 0 <= observation[name] <= limits[cap]
                for name, cap in budgets) and
            observation['body_bytes'] == sum(row['bytes'] for row in rows) and
            sum(observation[name] for name, _ in budgets) <=
            limits['total_logical_owned_write_bound_bytes'], 'wrong acquisition byte accounting')
    require(all(observation[name] is False for name in (
        'signature_verified', 'closure_complete', 'offline_compile_verified')),
        'scope exceeds passive capture')


def main():
    manifest = sealed_json(MANIFEST, MANIFEST_SHA)
    observation = sealed_json(HERE / 'observation.json', OBSERVATION_SHA)
    verify(manifest, observation)
    print('PASS: five passive responses, one JUnit BOM, three checksums, owned scope absent')


if __name__ == '__main__':
    main()
