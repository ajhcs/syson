"""Read-only static contract checks. Behavioral acceptance remains RR16b/RR17 work."""
import hashlib
import json

CONTRACT_PATH = 'completion-contract.json'
LOCK_PATH = 'completion-contract.lock.json'


def require(ok, message):
    if not ok:
        raise ValueError(message)


def pointer(contract, family):
    return {'id': contract['contract_id'], 'path': CONTRACT_PATH,
            'document': contract['document'], 'packet_schema': contract['packet_schema'],
            'family': family}


def canonical_hash(value):
    encoded = json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False)
    return hashlib.sha256(encoded.encode()).hexdigest()


def validate_repair_splits(contract, plan=None):
    """Static RR08 scope preservation; never a runtime or acceptance decision."""
    splits = contract.get('repair_splits', {})
    require(splits or contract['version'] not in ('1.0.2', '1.0.3'), 'RR08 split policy missing')
    if not splits:
        return
    require(set(splits) == {'RR08'}, 'RR08 split policy coverage changed')
    policy = splits['RR08']
    children = ['RR08a', 'RR08b']
    require(policy['parent'] == 'RR08' and policy['children'] == children and
            policy['implementation_order'] == children + ['RR08'], 'RR08 split order changed')
    require(policy['count_basis'] == 'added_plus_deleted', 'RR08 split counting changed')
    for key, expected in [('production_lines_max', 400), ('authored_lines_max', 900)]:
        require(type(policy[key]) is int and policy[key] == expected, 'RR08 split cap changed')
    ids = contract['unit_ids']
    later_children = contract.get('layout_kernel_splits', {}).get('U10', {}).get('children', [])
    old_ids = sorted(uid for uid in ids if uid not in children + later_children)
    require(policy['original_unit_count'] == 778 and len(old_ids) == 778 and
            set(children) <= set(ids) and canonical_hash(old_ids) == policy['original_unit_ids_sha256'],
            'RR08 split original unit coverage changed')
    legacy = contract['legacy_prerequisite_budget_dispositions']
    require(not set(['RR08'] + children) & set(legacy), 'RR08 split added budget exception')
    require(set(children) <= set(contract['families']['repairs']), 'RR08 split family changed')
    gates = policy['whole_parent_gates']
    require(set(gates) == {'original_rr08_cases', 'final_ast_cases', 'final_adapter_cases',
            'current_f11_cases', 'original_f12_cases', 'workload_points', 'caller_and_failure_checks',
            'measurement_method', 'qualification', 'acceptance_boundary'}, 'RR08 split gate coverage changed')
    for key, expected in [('final_ast_cases', 37), ('final_adapter_cases', 27),
                          ('current_f11_cases', 15), ('original_f12_cases', 12)]:
        require(type(gates[key]) is int and gates[key] == expected, 'RR08 split suite gate changed')
    require(gates['workload_points'] == [100, 1000, 5000], 'RR08 split workload gate changed')
    cases = gates['original_rr08_cases']
    partition = policy['case_partition']
    require(set(partition) == set(children) and len(partition['RR08a']) == 2 and
            len(partition['RR08b']) == 7 and partition['RR08a'] + partition['RR08b'] == cases and
            len(cases) == len(set(cases)) == 9, 'RR08 split original case coverage changed')
    checks = gates['caller_and_failure_checks']
    require(len(checks) == len(set(checks)) == 10 and
            all(isinstance(value, str) and value.strip() for value in checks + cases) and
            all(isinstance(gates[key], str) and gates[key].strip() for key in
                ['measurement_method', 'qualification', 'acceptance_boundary']),
            'RR08 split whole-parent gate missing')
    require(policy['production_changed_estimates'] == {'RR08a': 166, 'RR08b': 334},
            'RR08 split estimate changed; review a new amendment')
    if plan is None:
        return
    byid = {unit['id']: unit for unit in plan['units']}
    require(set(['RR08'] + children) <= set(byid), 'RR08 split unit missing')
    parent = byid['RR08']
    require(parent['paths'] == policy['parent_paths'] == ['porting/rust-parity-evidence/RR08/'],
            'RR08 split parent implementation scope restored')
    require(parent.get('delegated_implementation') == {
                'children': children, 'original_paths': policy['original_paths'],
                'original_dependencies': policy['original_dependencies']}, 'RR08 split delegation changed')
    preserved = {key: parent.get(key) for key in policy['original_parent_keys']}
    preserved.update(paths=policy['original_paths'], dependencies=policy['original_dependencies'])
    require(canonical_hash(preserved) == policy['original_parent_sha256'],
            'RR08 split original parent obligation changed')
    require(parent.get('whole_parent_gates') == gates, 'RR08 split parent gates changed')
    require(parent.get('acceptance_children') == children, 'RR08 split acceptance children changed')
    stages = plan['readiness_graph']
    for uid, dependencies in [('RR08a', []), ('RR08b', ['RR08a']), ('RR08', ['RR08b'])]:
        unit = byid[uid]
        require(unit['dependencies'] == dependencies and
                unit.get('implementation_ready_dependencies', dependencies) == dependencies,
                'RR08 split source order changed: ' + uid)
        require(set(stages['implementation:' + uid]) == {'implementation:' + dep for dep in dependencies},
                'RR08 split implementation order changed: ' + uid)
        accepted = {'implementation:' + uid}
        if uid == 'RR08':
            accepted.update('accepted:' + child for child in children)
        require(set(stages['accepted:' + uid]) == accepted, 'RR08 split acceptance edges changed: ' + uid)
    for uid in children:
        unit = byid[uid]
        require(unit.get('split_parent') == 'RR08' and unit['paths'] == policy['child_paths'][uid] and
                unit['paths'] == policy['original_paths'][:2] + ['porting/rust-parity-evidence/' + uid + '/'],
                'RR08 split child scope changed: ' + uid)
        require(unit.get('ownership') == policy['lease_policy'] and
                unit.get('budget_counting') == policy['count_basis'], 'RR08 split lease/counting changed')
        for key in ['production_lines_max', 'authored_lines_max']:
            require(type(unit['budget'][key]) is int and unit['budget'][key] == policy[key],
                    'RR08 split child cap changed: ' + uid)
        estimate = unit['production_delta_estimate']
        require(all(type(estimate[key]) is int and estimate[key] >= 0 for key in ['added', 'deleted', 'changed']) and
                estimate['added'] + estimate['deleted'] == estimate['changed'] ==
                policy['production_changed_estimates'][uid], 'RR08 split child estimate changed')
        authored = unit['authored_or_touched_upper_estimate']
        require(type(authored) is int and estimate['changed'] <= authored <= policy['authored_lines_max'],
                'RR08 split child authored estimate invalid')
        require(unit.get('original_case_requirements') == (partition[uid] if uid == 'RR08a' else cases),
                'RR08 split child original cases changed: ' + uid)


def validate_layout_splits(contract, plan=None, lane=None):
    """Static U10 prerequisite mapping; no native, UI or feature acceptance."""
    splits = contract.get('layout_kernel_splits', {})
    require(splits or contract['version'] != '1.0.3', 'U10 split policy missing')
    if not splits:
        return
    require(set(splits) == {'U10'}, 'U10 split policy coverage changed')
    policy = splits['U10']
    children = ['U10a', 'U10b']
    require(policy['parent'] == 'U10' and policy['children'] == children and
            policy['implementation_order'] == children, 'U10 split order changed')
    require(policy['count_basis'] == 'added_plus_deleted', 'U10 split counting changed')
    for key, expected in [('production_lines_max', 400), ('authored_lines_max', 900)]:
        require(type(policy[key]) is int and policy[key] == expected, 'U10 split cap changed')
    ids = contract['unit_ids']
    old_ids = sorted(uid for uid in ids if uid not in children)
    require(policy['original_unit_count'] == 780 and len(old_ids) == 780 and
            set(children) <= set(ids) and canonical_hash(old_ids) ==
            policy['original_unit_ids_sha256'] == 'a05da43ce998f9725d47447212b926205b6250831fc4503b8d9941476dbc1052', 'U10 split original unit coverage changed')
    require(set(children) <= set(contract['families']['views']) and not
            set(['U10'] + children) & set(contract['legacy_prerequisite_budget_dispositions']),
            'U10 split family or budget exception changed')
    require(policy['parent_paths'] == ['rust/crates/syson-web/src/u10/'] and
            policy['parent_dependencies'] == ['U02', 'U09'] and
            policy['parent_implementation_ready_dependencies'] == ['U02', 'U09', 'U10b'],
            'U10 split parent scope or prerequisites changed')
    expected_paths = {'U10a': ['rust/crates/syson-layout/src/types.rs', 'rust/crates/syson-layout/src/admission.rs', 'rust/crates/syson-layout/tests/admission.rs', 'porting/rust-parity-evidence/U10a/'], 'U10b': ['rust/crates/syson-layout/src/placement.rs', 'rust/crates/syson-layout/tests/layout.rs', 'porting/rust-parity-evidence/U10b/']}
    expected_wiring = ['rust/crates/syson-layout/src/lib.rs', 'rust/crates/syson-layout/Cargo.toml', 'rust/crates/syson-layout/Cargo.lock', 'rust/crates/syson-layout/NOTICE.md']
    require(policy['child_paths'] == expected_paths and
            policy['coordinator_wiring'] == expected_wiring, 'U10 split child scope changed')
    require(policy['child_dependencies'] == {'U10a': [], 'U10b': ['U10a']},
            'U10 split source order changed')
    require(policy['lease_policy'] == 'Separate exclusive module/test leases after amendment adoption and actual prerequisite admission; U10b follows landed U10a. Coordinator alone owns crate root exports/manifests/locks and actual UI assembly. This amendment grants no source or runtime lease.', 'U10 split lease policy changed')
    require(policy['production_changed_upper_estimates'] == {'U10a': 309, 'U10b': 295} and
            policy['authored_upper_estimates'] == {'U10a': 650, 'U10b': 890},
            'U10 split estimate changed; review another amendment')
    require(policy.get('measurement_interface') == {'revision': '1f090ddc3f7c20797e5740ab1e7bcfa3895ab7c1', 'path': 'rust/crates/syson-node-measurement/src/lib.rs', 'sha256': '3006fc02a1d34827c435e427e62532e1cd3ec872d14158b6f57cc7b190eb218d', 'scope': 'Existing f64 Size/IDs only. No dependency on or claim of complete UI measurement/provenance/affinity; actual conversion/consumer remains U10.'}, 'U10 split measurement interface changed')
    require(policy.get('kernel_dependency_policy') == 'std_only; no runtime or build dependencies; measurement is a source-interface pin, not a crate dependency', 'U10 split kernel dependency boundary changed')
    donor = policy['donor']
    require(donor['pull_request'] == 29 and donor['revision'] ==
            '6c198db81434ad4b1f318a6895fee8933a331d53' and
            donor['production_path'] == 'rust/crates/syson-layout/src/lib.rs' and
            donor['production_sha256'] ==
            'fd713fada8d59d9654783236d64709ef47329afc7029f8c06b65131c9615f1ad' and
            type(donor['production_lines']) is int and donor['production_lines'] == 529,
            'U10 split donor source changed')
    require(donor['tests_path'] == 'rust/crates/syson-layout/tests/layout.rs' and
            donor['tests_sha256'] ==
            'e7700f18f4f673e9a21fd7d1379234bc1014211f7722f48c456c91980ba5fe2a' and
            type(donor['test_lines']) is int and donor['test_lines'] == 572 and
            type(donor['test_count']) is int and donor['test_count'] == 15 and
            len(donor['original_cases']) == len(set(donor['original_cases'])) == 15 and
            canonical_hash(donor['original_cases']) == '367a0437bcdab74fe53f1457044fcba834209d7f3e26a079c3e51b9e290d307c', 'U10 split original tests changed')
    require(policy['original_parent_sha256'] == '833a96c882bd3a27087ab48cd7c1e867991084dd7e14201db79f5e2a36918cb5' and
            policy['preserved_normalized_parent_sha256'] == '2d98fbbdf23571bfb40dbf43a50d10f715d4620d73eae8e2b4397f601173a6aa',
            'U10 split original parent proof changed')
    gates = policy['whole_parent_gates']
    require(set(gates) == {'native_source_artifact', 'actual_caller', 'geometry_and_worker',
            'usability', 'resources', 'acceptance_boundary'} and
            canonical_hash(gates) == '3abe0ba55b379658a9b6b5f18676980f137511aaae5e1524485b8d582d7fdaf9', 'U10 split whole-parent gates changed')
    if lane is not None:
        raw = {unit['id']: unit for unit in lane['units']}['U10']
        preserved = {key: raw.get(key) for key in policy['original_parent_keys']}
        require(canonical_hash(preserved) == policy['original_parent_sha256'],
                'U10 split original lane parent changed')
    if plan is None:
        return
    byid = {unit['id']: unit for unit in plan['units']}
    require(set(['U10'] + children) <= set(byid), 'U10 split unit missing')
    parent = byid['U10']
    preserved = {key: parent.get(key) for key in policy['preserved_normalized_parent_keys']}
    require(canonical_hash(preserved) == policy['preserved_normalized_parent_sha256'],
            'U10 split original normalized parent changed')
    require(parent['dependencies'] == policy['parent_dependencies'] and
            parent['paths'] == policy['parent_paths'] and
            parent.get('implementation_ready_dependencies') ==
            policy['parent_implementation_ready_dependencies'] and
            parent.get('acceptance_children') == children and
            parent.get('kernel_prerequisite_children') == children and
            parent.get('kernel_parent_gates') == gates, 'U10 split parent gates changed')
    stages = plan['readiness_graph']
    require(set(stages['implementation:U10']) == {'implementation:' + uid for uid in
            policy['parent_implementation_ready_dependencies']} and
            set(stages['accepted:U10']) == {'implementation:U10', 'accepted:U10a', 'accepted:U10b'},
            'U10 split parent readiness or acceptance changed')
    lane_byid = {unit['id']: unit for unit in lane['units']} if lane is not None else None
    for uid in children:
        unit = byid[uid]
        dependencies = policy['child_dependencies'][uid]
        require(unit.get('split_parent') == 'U10' and unit['paths'] == expected_paths[uid] and
                unit.get('coordinator_wiring') == expected_wiring, 'U10 split child scope changed')
        require(unit['dependencies'] == dependencies and
                unit.get('implementation_ready_dependencies') == dependencies and
                set(stages['implementation:' + uid]) == {'implementation:' + dep for dep in dependencies} and
                set(stages['accepted:' + uid]) == {'implementation:' + uid},
                'U10 split child readiness changed')
        require(unit.get('kernel_dependency_policy') == policy['kernel_dependency_policy'],
                'U10 split child dependency boundary changed')
        require(unit.get('ownership') == policy['lease_policy'] and
                unit.get('budget_counting') == policy['count_basis'], 'U10 split lease/counting changed')
        for key in ['production_lines_max', 'authored_lines_max']:
            require(type(unit['budget'][key]) is int and unit['budget'][key] == policy[key],
                    'U10 split child cap changed')
        estimate = unit['production_delta_estimate']
        require(all(type(estimate[key]) is int and estimate[key] >= 0 for key in
                    ['added_upper', 'deleted_upper', 'changed_upper']) and
                estimate['added_upper'] + estimate['deleted_upper'] == estimate['changed_upper'] ==
                policy['production_changed_upper_estimates'][uid] and
                type(unit['authored_or_touched_upper_estimate']) is int and
                unit['authored_or_touched_upper_estimate'] == policy['authored_upper_estimates'][uid],
                'U10 split child estimate changed')
        expected_cases = donor['original_cases'] if uid == 'U10b' else []
        require(unit.get('original_case_requirements') == expected_cases, 'U10 split child tests changed')
        if uid == 'U10b':
            require(unit.get('retained_test_source') == {'path': donor['tests_path'],
                    'revision': donor['revision'], 'sha256': donor['tests_sha256'],
                    'line_count': 572, 'test_count': 15}, 'U10 split retained test source changed')
        if lane_byid is not None:
            require(all(unit.get(key) == value for key, value in lane_byid[uid].items()),
                    'U10 split child lane obligation changed: ' + uid)


def read_contract(root, read=None):
    """Check the adopted input set without refreshing hashes or writing files."""
    read = read or (lambda path: (root / path).read_bytes())
    contract = json.loads(read(CONTRACT_PATH))
    lock = json.loads(read(LOCK_PATH))
    require(contract['contract_id'] == lock['contract_id'], 'Contract/lock version mismatch')
    require(contract['contract_id'] == 'SYSON-RUST-COMPLETION/' + contract['version'],
            'Contract ID/version mismatch')
    required = {CONTRACT_PATH, contract['document'], contract['packet_schema'],
                contract['performance_policy'], 'cross-lane-review.json', 'rereview-overlay.json'}
    disposition = contract.get('legacy_prerequisite_budget_dispositions')
    amendments = ([disposition['amendment']] if disposition else [])
    amendments += [policy['amendment'] for policy in contract.get('repair_splits', {}).values()]
    amendments += [policy['amendment'] for policy in contract.get('layout_kernel_splits', {}).values()]
    for amendment in amendments:
        require(amendment['path'].startswith('amendments/') and
                '..' not in amendment['path'].split('/'), 'Invalid amendment path')
        required.add(amendment['path'])
        require(hashlib.sha256(read(amendment['path'])).hexdigest() ==
                amendment['sha256'], 'Contract amendment hash mismatch')
    required.update(str(p.relative_to(root)) for p in (root / 'lanes').glob('*.json'))
    required.update(str(p.relative_to(root)) for p in (root / 'tools').glob('*.py'))
    require(set(lock['sha256']) == required, 'Contract lock input set mismatch')
    for path, expected in lock['sha256'].items():
        require(hashlib.sha256(read(path)).hexdigest() == expected,
                'Contract lock hash mismatch: ' + path)
    ids = contract['unit_ids']
    require(len(ids) == len(set(ids)), 'Duplicate contract unit ID')
    families = contract['families']
    owners = [uid for units in families.values() for uid in units]
    require(len(owners) == len(set(owners)) and set(owners) == set(ids),
            'Contract family coverage mismatch')
    schema = json.loads(read(contract['packet_schema']))
    require(set(schema['properties']['unit_id']['enum']) == set(ids),
            'Packet schema unit coverage mismatch')
    require(schema['properties']['contract_id']['const'] == contract['contract_id'],
            'Packet schema contract mismatch')
    require(schema['properties']['target_source']['const'] == contract['target_source'],
            'Packet schema target mismatch')
    validate_repair_splits(contract)
    validate_layout_splits(contract, lane=json.loads(read('lanes/ui.json')))
    return contract


def validate_bindings(contract, plan, root):
    require(plan['target_source'] == contract['target_source'], 'Contract target mismatch')
    require(plan.get('completion_contract') == {'id': contract['contract_id'],
            'path': CONTRACT_PATH, 'lock': LOCK_PATH}, 'Missing plan contract binding')
    require({u['id'] for u in plan['units']} == set(contract['unit_ids']),
            'Plan/contract unit coverage mismatch')
    validate_repair_splits(contract, plan)
    validate_layout_splits(contract, plan, json.loads((root / 'lanes/ui.json').read_text()))
    families = {uid: family for family, ids in contract['families'].items() for uid in ids}
    for unit in plan['units']:
        require(unit.get('completion_contract') == pointer(contract, families[unit['id']]),
                'Missing/changed unit contract binding: ' + unit['id'])
        brief = (root / 'prs' / (unit['id'] + '.md')).read_text()
        for expected in [contract['contract_id'], '../' + CONTRACT_PATH,
                         '../' + contract['document'], '../' + contract['packet_schema'],
                         'Primary acceptance family: ' + families[unit['id']]]:
            require(expected in brief, 'Missing brief contract binding: ' + unit['id'])
