"""Verify the sanitized JDT Core capture against the exact parent roster."""
from collections import Counter
from hashlib import sha256
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
CORE = HERE.parents[2] / 'rust-parity-plan-20261026' / 'closure' / 'core-compile'
EPOCH = 'eclipse-jdt-core-descriptors-01'
MANIFEST_SHA = '51e6eddd6081eb8ceff42dfd912ab9c58d77bba54e15eed65c7e5bdfce572c7c'
CLIENT_SHA = '0501427f01bee24fa590db811734a9604b65429f9bf1b695ee7d987c11ff06fb'
OBSERVATION_SHA = 'bb7e2818adf6fe8a62b43293e1d7f5e0ae1b2bc109dc0432a95696366dd22896'
SHA256 = re.compile(r'[0-9a-f]{64}')
CONTROLS = {
    'cpu.max': '100000 100000',
    'memory.high': '536870912',
    'memory.max': '1073741824',
    'memory.swap.max': '0',
    'pids.max': '32',
}
CUSTODY = {
    'frozen_spec': 'de05b3656e479045bda54a249efa67c626aa99bcd6c9f4c92cc09b0dfdabb6d6',
    'getter_receipt': 'abb95bd703de8f9cdcfc8165fecbf163891bd1b8949e0078031da65136e43f95',
    'controller_receipt': 'dbf9ea6ba5abedf62aa249d8d0dadfce3bacf00d8132a1a465c992653cbf792a',
    'scope_proof': '2346a55013cfdff7cf21f35cc7a5a662befec77e3bef58a40a29f86bda76f8df',
}
AUTHORITY_KEYS = {
    'publisher_signature_verified', 'cache_install_allowed', 'acquired_code_execution_allowed',
    'effective_maven_model_verified', 'final_target_versions_selected', 'closure_complete',
    'offline_native_compile_verified',
}
OBSERVATION_KEYS = {
    'schema', 'status', 'epoch', 'manifest_sha256', 'client_sha256', 'custody_sha256',
    'frozen_input_count', 'frozen_inputs_match_before_and_after', 'controls', 'controller_exit',
    'owned_scope_cleanup', 'output_file_count', 'all_final_urls_match_roster', 'accounting',
    'verified_same_origin_sha1_count', 'model_versions', 'request_rows', 'authority',
}
BUDGETS = {
    'body_bytes': 'payload_and_sidecar_total_bytes',
    'getter_metadata_bytes': 'getter_metadata_proof_receipt_bytes',
    'controller_json_bytes': 'root_controller_JSON_bytes',
    'controller_log_bytes': 'root_child_stdout_stderr_bytes',
}


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def sealed_bytes(path, expected):
    with path.open('rb') as stream:
        raw = stream.read(256 * 1024 + 1)
    require(len(raw) <= 256 * 1024 and sha256(raw).hexdigest() == expected,
            f'evidence seal changed: {path.name}')
    return raw


def verify(manifest, observation):
    require(set(observation) == OBSERVATION_KEYS and
            observation['schema'] == 'SYSON-C01-JDT-CORE-PASSIVE-RUN/1' and
            observation['status'] == 'PASS_PASSIVE_ACQUISITION_ONLY' and
            observation['epoch'] == manifest['epoch'] == EPOCH and
            observation['manifest_sha256'] == MANIFEST_SHA and
            observation['client_sha256'] == CLIENT_SHA and
            observation['custody_sha256'] == CUSTODY, 'capture binding changed')
    primaries = manifest['new_primaries']
    requests = manifest['requests']
    rows = observation['request_rows']
    require(len(primaries) == 22 and len(rows) == len(requests) == 110 and
            manifest['frontier']['observed_pom_count'] == 215 and
            manifest['frontier']['remaining_pom_count'] == 193, 'frontier or roster changed')
    statuses = Counter()
    sha1_coverage = set()
    for row, request in zip(rows, requests):
        require(type(row) is list and len(row) == 3, 'response row shape changed')
        status, length, digest = row
        require(type(status) is int and status in request['allowed_status'] and
                type(length) is int and 0 < length <= request['maximum_bytes'] and
                isinstance(digest, str) and SHA256.fullmatch(digest), 'response row changed')
        role = request.get('algorithm', request['role'])
        statuses[role, status] += 1
        if role == 'sha1' and status == 200:
            require(length == 40, 'SHA-1 sidecar length changed')
            sha1_coverage.add(request['primary_index'])
    require(statuses == {('pom', 200): 22, ('sha1', 200): 22, ('sha256', 404): 22,
                         ('sha512', 404): 22, ('signature', 200): 22} and
            sha1_coverage == set(range(len(primaries))) and
            type(observation['verified_same_origin_sha1_count']) is int and
            observation['verified_same_origin_sha1_count'] == len(sha1_coverage),
            'response statuses or reported checksum coverage changed')
    versions = [primary['coordinate']['version'] for primary in primaries]
    require(observation['model_versions'] == [
        [version, '4.0' if version == '3.31.0' else '4.0.0'] for version in versions
    ], 'observed modelVersion spelling changed')
    require(observation['all_final_urls_match_roster'] is True and
            observation['controls'] == CONTROLS and
            type(observation['controller_exit']) is int and observation['controller_exit'] == 0 and
            observation['owned_scope_cleanup'] == 'not-found' and observation['output_file_count'] == 222 and
            observation['frozen_input_count'] == 177 and
            observation['frozen_inputs_match_before_and_after'] is True, 'scope evidence changed')
    accounting = observation['accounting']
    limits = manifest['limits']
    require(accounting == {'body_bytes': 101783, 'getter_metadata_bytes': 495141,
                           'controller_json_bytes': 427, 'controller_log_bytes': 0} and
            all(type(accounting[name]) is int and 0 <= accounting[name] <= limits[cap]
                for name, cap in BUDGETS.items()) and
            accounting['body_bytes'] == sum(row[1] for row in rows) and
            sum(accounting.values()) <= limits['total_logical_owned_write_bound_bytes'],
            'capture byte accounting changed')
    authority = observation['authority']
    require(set(authority) == AUTHORITY_KEYS and all(value is False for value in authority.values()),
            'capture authority changed')


def main():
    sealed_bytes(CORE / 'acquire.py', CLIENT_SHA)
    manifest = json.loads(sealed_bytes(CORE / 'requests' / f'{EPOCH}.json', MANIFEST_SHA))
    observation = json.loads(sealed_bytes(HERE / 'observation.json', OBSERVATION_SHA))
    verify(manifest, observation)
    print('PASS: 110 passive responses, 22 POMs, 22 recorded SHA-1 matches, owned scope absent')


if __name__ == '__main__':
    main()
