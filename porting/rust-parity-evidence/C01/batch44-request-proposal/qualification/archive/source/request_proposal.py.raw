"""Pure finite model proposals; no filesystem, acquisition or version selection."""
import copy
import hashlib
import json
import re

MIB = 1024**2
BASE = 'https://repo.maven.apache.org/maven2/'
TEMPLATE_SHA = 'd4c51a84aa264eaa057d42f4d208744573520085dca116103b874790c8fb445d'
EPOCHS = {'models-02': {
    'frontier_sha256': 'cfe1c1f36f4f9d438d7840e9798b15dc21bbafa3da3c9af50a0ec68215de769f',
    'frontier_bytes': 28714, 'primaries': 18, 'requests': 90, 'parents': 12, 'conditional_imports': 6,
    'ordered_primary_sha256': '61db2cd50e4a9cfcac49801208c2060093bfe68fab141fcca4fc0caa690aaa42'}}
PROFILE = {'exact_primaries': 56, 'exact_requests': 280,
    'payload_and_sidecar_total_bytes': 16 * MIB, 'per_POM_bytes': 2 * MIB,
    'per_version_metadata_bytes': 256 * 1024, 'per_sidecar_bytes': 65536,
    'per_response_header_pairs': 64, 'per_response_header_UTF8_bytes': 16384,
    'getter_metadata_proof_receipt_bytes': 63 * MIB,
    'getter_final_failure_receipt_reserve_bytes': MIB,
    'root_child_stdout_stderr_bytes': 960 * 1024, 'root_controller_JSON_bytes': 64 * 1024,
    'all_metadata_and_log_bytes': 64 * MIB, 'total_logical_owned_write_bound_bytes': 80 * MIB,
    'manifest_and_spec_each_maximum_bytes': 256 * 1024,
    'whole_phase_subject_seconds': 900, 'scope_RuntimeMaxSec': 1020}


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def encoded(value):
    return (json.dumps(value, indent=2, allow_nan=False) + '\n').encode()


def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON key')
        result[key] = value
    return result


def propose(epoch, frontier_bytes, template_bytes):
    require(type(epoch) is str and epoch in EPOCHS, 'unadmitted finite proposal epoch')
    fixed = EPOCHS[epoch]
    require(all(type(fixed[k]) is int and fixed[k] > 0 for k in
                ['frontier_bytes', 'primaries', 'requests', 'parents', 'conditional_imports']),
            'strict finite epoch counts')
    require(type(frontier_bytes) is bytes and len(frontier_bytes) <= 65536 and
            len(frontier_bytes) == fixed['frontier_bytes'] and
            sha(frontier_bytes) == fixed['frontier_sha256'], 'fixed frontier bytes/size/hash changed')
    require(type(template_bytes) is bytes and len(template_bytes) <= 256 * 1024 and
            len(template_bytes) == 203892 and sha(template_bytes) == TEMPLATE_SHA,
            'fixed main template bytes/size/hash changed')
    frontier = json.loads(frontier_bytes, object_pairs_hook=unique)
    template = json.loads(template_bytes, object_pairs_hook=unique)
    require(frontier['schema'] == 'C01-NEXT-FINITE-MODEL-FRONTIER-PROPOSAL/2', 'fixed frontier schema')
    limits = template['limits']
    require(type(limits) is dict and set(limits) == set(PROFILE) and all(
        type(limits[k]) is int and limits[k] == value for k, value in PROFILE.items()), 'fixed resource profile')
    declared = frontier['next_finite_epoch']
    require(all(type(declared[k]) is int and declared[k] == value for k, value in
        [('primaries', fixed['primaries']), ('literal_parents', fixed['parents']),
         ('conditional_local_import_observations', fixed['conditional_imports']),
         ('all4sidecar_requests', fixed['requests'])]), 'strict frontier count declarations')
    observations = declared['rows']
    require(len(observations) == fixed['primaries'] and
            sum(row['need'] == 'IMPORT' for row in observations) == fixed['conditional_imports'] and
            sum(row['need'] == 'PARENT' for row in observations) == fixed['parents'], 'fixed18 observation roles')
    primaries, requests = [], []
    for index, row in enumerate(observations):
        group, artifact, version = row['GAV'].split(':')
        require(all(re.fullmatch(r'[A-Za-z0-9_-]+', part) for part in group.split('.')) and
                all(re.fullmatch(r'[A-Za-z0-9_.-]+', part) and part not in ('.', '..')
                    for part in [artifact, version]), 'canonical fixed GAV')
        relative = group.replace('.', '/') + '/' + artifact + '/' + version + '/' + artifact + '-' + version + '.pom'
        require(row['kind'] == 'pom' and row['url'] == BASE + relative, 'fixed canonical POM URL')
        expected_status = ('MISSING_LITERAL_PARENT' if row['need'] == 'PARENT' else
                           'LOCAL_IMPORT_CANDIDATE_EFFECTIVE_PROPERTY_PROFILE_PROOF_OPEN')
        require(row['status'] == expected_status, 'conditional observation status changed')
        provenance = []
        for source, selector, raw_version, properties, contexts in row['evidence']:
            model = frontier['source_index']['models'][source]
            provenance.append({'source_id': source, 'binding': copy.deepcopy(model), 'xml_path': selector,
                'raw_version': raw_version, 'exact_local_properties': copy.deepcopy(properties),
                'contexts': [frontier['source_index']['contexts'][i] for i in contexts],
                'disposition': expected_status})
        primaries.append({'relative_artifact': relative, 'url': BASE + relative, 'kind': 'pom',
            'coordinate': {'groupId': group, 'artifactId': artifact, 'version': version},
            'primary_index': index, 'maximum_bytes': 2 * MIB, 'allowed_status': [200],
            'categories': ['parent_model' if row['need'] == 'PARENT' else 'conditional_local_import_observation'],
            'provenance': provenance, 'disposition': expected_status})
        for suffix in ['', '.sha512', '.sha256', '.sha1', '.asc']:
            request = {'url': BASE + relative + suffix, 'relative_path': 'downloads/' + relative + suffix + '.partial',
                'maximum_bytes': 65536 if suffix else 2 * MIB,
                'role': 'pom' if not suffix else 'signature' if suffix == '.asc' else 'checksum',
                'allowed_status': [200, 404] if suffix else [200], 'primary_index': index}
            if suffix in ('.sha512', '.sha256', '.sha1'):
                request['algorithm'] = suffix[1:]
            require(len(encoded(request)) <= 4096, 'request framing limit')
            requests.append(request)
    projection = [(p['relative_artifact'], p['kind']) for p in primaries]
    require(projection == sorted(projection) and sha(json.dumps(projection, separators=(',', ':')).encode())
            == fixed['ordered_primary_sha256'], 'independent fixed ordered primary set changed')
    require(len(requests) == fixed['requests'] and len({r['url'] for r in requests}) == len(requests)
            and len({r['relative_path'] for r in requests}) == len(requests), 'fixed ordered request coverage')
    limits = copy.deepcopy(limits)
    limits.update(exact_primaries=fixed['primaries'], exact_requests=fixed['requests'])
    copies = fixed['requests'] * 2 * (6 * limits['per_response_header_UTF8_bytes'] + 5120 + 4096)
    total = copies + 2 * MIB
    require(total <= limits['getter_metadata_proof_receipt_bytes'], 'serialized metadata profile does not fit')
    result = {'schema': 'C01-NORMAL-CORE-CLOSURE-REQUESTS/1', 'epoch': epoch,
        'status': 'PROPOSED_FINITE_MODEL_EVIDENCE_EPOCH_NO_EXECUTION_AUTHORITY',
        'target': copy.deepcopy(template['target']), 'new_primaries': primaries, 'requests': requests,
        'limits': limits, 'source_bindings': [{'role': 'frozen_frontier', 'sha256': sha(frontier_bytes),
            'bytes': len(frontier_bytes)}, {'role': 'unchanged_main_template', 'sha256': sha(template_bytes),
            'bytes': len(template_bytes)}], 'observed_frontier': copy.deepcopy(frontier),
        'nominal_metadata_charge': {'requests': len(requests), 'two_escaped_response_copies_bytes': copies,
            'header_structure_allowance_per_copy': 5120, 'nonheader_row_framing_per_copy': 4096,
            'static_and_failure_reserve': 2 * MIB, 'getter_nominal_bytes': total,
            'getter_limit': limits['getter_metadata_proof_receipt_bytes'],
            'meaning': 'Logical bound only; no physical quota, signature authentication or RR16 claim'},
        'body_bound_note': {'sum_individual_caps_bytes': sum(r['maximum_bytes'] for r in requests),
            'aggregate_refusal_cap_bytes': limits['payload_and_sidecar_total_bytes'],
            'completion_within_aggregate_promised': False, 'behavior': template['body_bound_note']['behavior']},
        'remaining_gates': copy.deepcopy(template['remaining_gates']) + copy.deepcopy(frontier['next_actions']),
        'authorization': {'network': False, 'acquired_code_execution': False, 'range_selection': False,
            'effective_import_selection': False, 'complete_closure': False, 'offline_compile': False}}
    require(len(encoded(result)) <= 256 * 1024, 'proposal output byte limit')
    return result
