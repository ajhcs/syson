"""Check the sanitized models-03 observation against the exact request manifest."""
from collections import Counter
import hashlib
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
MANIFEST = (HERE.parents[2] / 'rust-parity-plan-20261026' / 'closure' /
            'core-compile' / 'requests' / 'models-03.json')
MANIFEST_SHA = '7eafb6e5436dd04692e43a1f3d87a204dc206ed3fe55a80f9cae2d128f1be428'
OBSERVATION_SHA = '2a3db744883bc3ee1f7933713a174bb70cce755080826dc1927d3bb54e2615ae'
SHA256 = re.compile(r'[0-9a-f]{64}')
MIB = 1024 ** 2
CONTROLS = {'cpu.max': '100000 100000', 'memory.high': '536870912',
            'memory.max': '1073741824', 'memory.swap.max': '0', 'pids.max': '32'}


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON key')
        result[key] = value
    return result


def sealed_json(path, expected_sha):
    raw = path.read_bytes()
    require(hashlib.sha256(raw).hexdigest() == expected_sha, 'evidence seal changed')
    return json.loads(raw, object_pairs_hook=unique_object)


def verify(manifest, evidence):
    require(evidence['schema'] == 'SYSON-C01-MODELS03-PASSIVE-RUN/1' and
            evidence['status'] == 'PASS_PASSIVE_ACQUISITION_ONLY' and
            evidence['epoch'] == manifest['epoch'] == 'models-03' and
            evidence['manifest_sha256'] == MANIFEST_SHA, 'wrong acquisition binding')
    rows = evidence['request_rows']
    require(len(rows) == len(manifest['requests']) == 25, 'request count changed')
    totals, primaries, checksums = Counter(), [], []
    for index, (row, request) in enumerate(zip(rows, manifest['requests'])):
        require(set(row) == {'relative_path', 'role', 'status', 'bytes', 'sha256'} and
                row['relative_path'] == request['relative_path'] and row['role'] == request['role'] and
                type(row['status']) is int and row['status'] in request['allowed_status'] and
                type(row['bytes']) is int and 0 <= row['bytes'] <= request['maximum_bytes'] and
                SHA256.fullmatch(row['sha256']), f'invalid response row {index}')
        totals[(row['role'], row['status'])] += 1
        primary = manifest['new_primaries'][request['primary_index']]
        if row['role'] == 'pom':
            primaries.append((primary, row))
        if row['role'] == 'checksum' and row['status'] == 200:
            checksums.append({'artifact': primary['relative_artifact'], 'algorithm': request['algorithm']})
    require(totals == {('pom', 200): 5, ('checksum', 200): 7, ('checksum', 404): 8, ('signature', 200): 5}
            and checksums == evidence['verified_same_origin_checksums'], 'HTTP or checksum result changed')
    observations = evidence['primary_model_observations']
    require(len(observations) == len(primaries) == 5, 'POM source count changed')
    for model, (primary, row) in zip(observations, primaries):
        require(model['coordinate'] == primary['coordinate'] and model['bytes'] == row['bytes'] and
                model['sha256'] == row['sha256'] and model['parent'] is None and
                model['packaging'] == 'pom' and model['managed_import_count'] == 0,
                'POM source projection differs')
    require([model['managed_dependency_count'] for model in observations] == [0, 0, 0, 0, 18] and
            observations[1]['profile_ids'] == ['apache-release', 'jdk9+', 'only-eclipse',
                                               'use-apache-snapshots'], 'raw model observation differs')
    require(evidence['controls'] == CONTROLS and evidence['controller_exit'] == 0 and
            evidence['owned_scope_cleanup'] == 'not-found' and evidence['output_file_count'] == 52 and
            evidence['frozen_input_count'] == 114 and
            evidence['frozen_inputs_before_after_and_review_match'] is True, 'run controls differ')
    sizes = [evidence[name] for name in ('body_bytes', 'getter_metadata_bytes', 'controller_json_bytes',
                                         'controller_log_bytes')]
    require(all(type(size) is int and 0 <= size <= cap
                for size, cap in zip(sizes, (16*MIB, 63*MIB, 64*1024, 960*1024))) and
            sizes[0] == sum(row['bytes'] for row in rows) and sum(sizes) <= 80*MIB,
            'logical byte accounting differs')
    require(all(evidence[name] is False for name in ('signature_verified', 'closure_complete',
                                                      'offline_compile_verified')), 'passive scope exceeded')


def main():
    verify(sealed_json(MANIFEST, MANIFEST_SHA), sealed_json(HERE / 'observation.json', OBSERVATION_SHA))
    print('PASS: 25 passive responses, five source POMs, seven checksums, owned scope absent')


if __name__ == '__main__':
    main()
