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
    'ordered_primary_sha256': '61db2cd50e4a9cfcac49801208c2060093bfe68fab141fcca4fc0caa690aaa42'},
    'models-03': {
        'frontier_sha256': 'cda4107a7e40d12941a2f6d6e696780a434039cd364cfecfe61f55f5d7f44723',
        'frontier_bytes': 7460, 'primaries': 5, 'requests': 25, 'parents': 4, 'conditional_imports': 1,
        'ordered_primary_sha256': 'c4f24591cad25f9aac6f8e2c6a6aa78e1f6eef396497af42c057546a7b602458'}}
ABSENT = 'ABSENT_FROM_ALL_SIX_DECLARED_INVENTORIES'
CONDITIONAL = 'CONDITIONAL_MODEL_EVIDENCE_ONLY_EFFECTIVE_INTERPOLATION_ACTIVATION_OPEN'
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


def model02_observations(frontier, fixed):
    require(frontier['schema'] == 'C01-NEXT-FINITE-MODEL-FRONTIER-PROPOSAL/2', 'fixed frontier schema')
    declared = frontier['next_finite_epoch']
    require(all(type(declared[k]) is int and declared[k] == value for k, value in
        [('primaries', fixed['primaries']), ('literal_parents', fixed['parents']),
         ('conditional_local_import_observations', fixed['conditional_imports']),
         ('all4sidecar_requests', fixed['requests'])]), 'strict frontier count declarations')
    observations = declared['rows']
    require(len(observations) == fixed['primaries'] and
            sum(row['need'] == 'IMPORT' for row in observations) == fixed['conditional_imports'] and
            sum(row['need'] == 'PARENT' for row in observations) == fixed['parents'], 'fixed18 observation roles')
    for row in observations:
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
        yield row, provenance


def model03_observations(frontier, fixed):
    require(frontier['schema'] == 'C01-MODELS-03-BOUNDED-FRONTIER/1', 'fixed frontier schema')
    observations = frontier['rows']
    require(len(observations) == fixed['primaries'] and
            sum(row['need'] == 'PARENT' for row in observations) == fixed['parents'] and
            sum(row['need'] == 'CONDITIONAL_IMPORT' for row in observations) == fixed['conditional_imports'],
            'fixed5 observation roles')
    deferred = frontier['deferred']
    require(deferred['GAV'] == 'org.junit:junit-bom:5.12.1' and
            deferred['need'] == 'CONDITIONAL_IMPORT' and deferred['source_disposition'] == CONDITIONAL and
            deferred['status'] == ABSENT and
            deferred['provenance']['exact_local_properties'] == {'versions.junit5': '5.12.1'} and
            all(row['GAV'] != deferred['GAV'] for row in observations), 'JaCoCo import must stay deferred')
    inventories = {'failed280', 'frozen309', 'models01', 'models02', 'new140', 'prior395'}
    for row in [*observations, deferred]:
        absence = row['absence_vector']
        require(row['status'] == ABSENT and set(absence) == inventories and
                all(type(n) is int and n == 0 for n in absence.values()), 'fixed absence evidence')
    for row in observations:
        evidence = row['provenance']
        raw = evidence['raw']
        if row['need'] == 'PARENT':
            require(row['source_disposition'] == 'LITERAL_PARENT_MODEL_OBLIGATION' and
                    evidence['xml_path'] == '/project/parent' and
                    row['GAV'] == ':'.join(raw[k] for k in ('groupId', 'artifactId', 'version')),
                    'literal parent observation changed')
        else:
            require(row['GAV'] == 'org.junit:junit-bom:5.14.1' and row['source_disposition'] == CONDITIONAL and
                    evidence['source_coordinate'] == 'org.codehaus.plexus:plexus:25' and
                    evidence['xml_path'] == '/project/dependencyManagement/dependencies/dependency' and
                    raw == {'groupId': 'org.junit', 'artifactId': 'junit-bom', 'version': '${junit5Version}',
                            'type': 'pom', 'scope': 'import'} and
                    evidence['exact_local_properties'] == {'junit5Version': '5.14.1'},
                    'conditional Plexus observation changed')
        yield row, [copy.deepcopy(evidence)]


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
    limits = template['limits']
    require(type(limits) is dict and set(limits) == set(PROFILE) and all(
        type(limits[k]) is int and limits[k] == value for k, value in PROFILE.items()), 'fixed resource profile')
    observations = (model03_observations(frontier, fixed) if epoch == 'models-03' else
                    model02_observations(frontier, fixed))
    primaries, requests = [], []
    for index, (row, provenance) in enumerate(observations):
        group, artifact, version = row['GAV'].split(':')
        require(all(re.fullmatch(r'[A-Za-z0-9_-]+', part) for part in group.split('.')) and
                all(re.fullmatch(r'[A-Za-z0-9_.-]+', part) and part not in ('.', '..')
                    for part in [artifact, version]), 'canonical fixed GAV')
        relative = group.replace('.', '/') + '/' + artifact + '/' + version + '/' + artifact + '-' + version + '.pom'
        require(row['kind'] == 'pom' and row['url'] == BASE + relative, 'fixed canonical POM URL')
        primaries.append({'relative_artifact': relative, 'url': BASE + relative, 'kind': 'pom',
            'coordinate': {'groupId': group, 'artifactId': artifact, 'version': version},
            'primary_index': index, 'maximum_bytes': 2 * MIB, 'allowed_status': [200],
            'categories': ['parent_model' if row['need'] == 'PARENT' else 'conditional_local_import_observation'],
            'provenance': provenance, 'disposition': row['status']})
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
    gates = copy.deepcopy(template['remaining_gates'])
    if epoch == 'models-03':
        gates[0] = gates[0].replace('Existing140URL getter does not admit these280URLs.',
                                    'The existing 140-URL getter does not admit this 25-request proposal.')
        next_actions = [
            'No active client entry. Separate source and epoch admission remains required.',
            'Inspect Apache 34 and any inherited inputs, then freeze the JaCoCo model context.',
            'Both BOM observations remain conditional. Effective import and range selection remain open.']
    else:
        next_actions = frontier['next_actions']
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
        'remaining_gates': gates + copy.deepcopy(next_actions),
        'authorization': {'network': False, 'acquired_code_execution': False, 'range_selection': False,
            'effective_import_selection': False, 'complete_closure': False, 'offline_compile': False}}
    if epoch == 'models-03':
        result['source_bindings'].extend(copy.deepcopy(frontier['evidence_bindings']))
        result['deferred_model_input'] = {'source_observation': copy.deepcopy(frontier['deferred']),
            'still_open': True,
            'next_evidence': 'Apache 34, its actual parent closure and frozen JaCoCo context.'}
    require(len(encoded(result)) <= 256 * 1024, 'proposal output byte limit')
    return result
