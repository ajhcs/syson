"""Verify the sanitized JaCoCo binary capture against its exact parent roster."""
from collections import Counter
from hashlib import sha256
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
CORE = HERE.parents[2] / 'rust-parity-plan-20261026' / 'closure' / 'core-compile'
MANIFEST_SHA = '9ec6ebc4b9805b4117ad1b13534f620316bdab5758b5a8a562d23da1561910e6'
CLIENT_SHA = '23ec4392f701f8e9ef5b8162514029429a18095015348c4a1ada30bbe61d2b54'
OBSERVATION_SHA = '68528f34cdda6789e00cdeb653a2eb4f49cd8c4f28cf11c3c3bd9e0e12251b00'
SHA256 = re.compile(r'[0-9a-f]{64}')
CONTROLS = {
    'cpu.max': '100000 100000',
    'memory.high': '536870912',
    'memory.max': '1073741824',
    'memory.swap.max': '0',
    'pids.max': '32',
}
AUTHORITY_KEYS = {
    'publisher_signature_verified', 'cache_install_allowed',
    'jar_member_extraction_allowed', 'acquired_code_execution_allowed',
    'closure_complete', 'offline_native_compile_verified',
}
OBSERVATION_KEYS = {
    'schema', 'status', 'epoch', 'manifest_sha256', 'client_sha256',
    'frozen_input_count', 'frozen_inputs_match_before_and_after', 'controls',
    'controller_exit', 'owned_scope_cleanup', 'output_file_count',
    'all_final_urls_match_roster', 'accounting', 'verified_same_origin_checksums',
    'request_rows', 'authority',
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


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON key')
        result[key] = value
    return result


def sealed_bytes(path, expected):
    raw = path.read_bytes()
    require(sha256(raw).hexdigest() == expected, 'evidence seal changed')
    return raw


def sealed_json(path, expected):
    return json.loads(sealed_bytes(path, expected), object_pairs_hook=unique_object)


def verify(manifest, observation):
    require(set(observation) == OBSERVATION_KEYS and
            observation['schema'] == 'SYSON-C01-JACOCO-BINARIES-PASSIVE-RUN/1' and
            observation['status'] == 'PASS_PASSIVE_ACQUISITION_ONLY' and
            observation['epoch'] == manifest['epoch'] == 'jacoco-binaries-01' and
            observation['manifest_sha256'] == MANIFEST_SHA and
            observation['client_sha256'] == CLIENT_SHA, 'wrong capture binding')
    primaries = manifest['new_primaries']
    requests = manifest['requests']
    rows = observation['request_rows']
    require(len(primaries) == 10 and len(rows) == len(requests) == 50 and
            len(manifest['excluded_existing_data_candidates']) == 3, 'wrong roster size')
    states = Counter()
    checksums = []
    for row, request in zip(rows, requests):
        require(set(row) == {'relative_path', 'role', 'status', 'bytes', 'sha256'} and
                row['relative_path'] == request['relative_path'] and
                row['role'] == request['role'] and
                type(row['status']) is int and row['status'] in request['allowed_status'] and
                type(row['bytes']) is int and 0 <= row['bytes'] <= request['maximum_bytes'] and
                isinstance(row['sha256'], str) and SHA256.fullmatch(row['sha256']),
                'wrong response row')
        states[row['role'], row['status']] += 1
        if row['status'] == 200:
            require(row['bytes'] > 0, 'empty successful response')
            if row['role'] == 'checksum':
                checksums.append({'artifact': primaries[request['primary_index']]['relative_artifact'],
                                  'algorithm': request['algorithm']})
    require(states == {('jar', 200): 10, ('checksum', 200): 14, ('checksum', 404): 16,
                       ('signature', 200): 10}, 'response status summary changed')
    require(observation['verified_same_origin_checksums'] == checksums and
            Counter(row['algorithm'] for row in checksums) == {'sha1': 10, 'sha256': 2, 'sha512': 2} and
            {row['artifact'] for row in checksums} == {row['relative_artifact'] for row in primaries},
            'wrong checksum coverage')
    require(observation['all_final_urls_match_roster'] is True and
            observation['controls'] == CONTROLS and
            type(observation['controller_exit']) is int and observation['controller_exit'] == 0 and
            observation['owned_scope_cleanup'] == 'not-found' and observation['output_file_count'] == 102 and
            observation['frozen_input_count'] == 161 and
            observation['frozen_inputs_match_before_and_after'] is True, 'wrong scope observation')
    accounting = observation['accounting']
    limits = manifest['limits']
    require(set(accounting) == set(BUDGETS) and all(
        type(accounting[name]) is int and 0 <= accounting[name] <= limits[cap]
        for name, cap in BUDGETS.items()) and
        accounting['body_bytes'] == sum(row['bytes'] for row in rows) and
        accounting['body_bytes'] <= sum(request['maximum_bytes'] for request in requests) and
        sum(accounting.values()) <= limits['total_logical_owned_write_bound_bytes'],
        'wrong capture byte accounting')
    authority = observation['authority']
    require(set(authority) == AUTHORITY_KEYS and all(value is False for value in authority.values()),
            'scope exceeds passive capture')


def main():
    sealed_bytes(CORE / 'acquire.py', CLIENT_SHA)
    manifest = sealed_json(CORE / 'requests' / 'jacoco-binaries-01.json', MANIFEST_SHA)
    observation = sealed_json(HERE / 'observation.json', OBSERVATION_SHA)
    verify(manifest, observation)
    print('PASS: 50 passive responses, ten JARs, 14 matching checksums, owned scope absent')


if __name__ == '__main__':
    main()
