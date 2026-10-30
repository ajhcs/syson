"""Fixed unsigned RR13 candidates; importing performs no file or application I/O."""
import copy
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import sys
import types

LIMIT = 64 * 1024**2
OUTPUT_ROOT = Path('/workspace/build/builderbob')
CHILDREN = ['RR13a', 'RR13b']
OMIT = {'completion_contract', 'source_fragment_sha256'}
BASE_PLAN = '2a30cac14037502dc18128ff9b57d477b6fd18b5d36c658ff7413d07dca324e3'
BASE_CONTRACT = '6771410b685f5b9c691705a3821a13202b61535a04e622e8399cfd592fc36e17'
BASE_UNITS = '0e635baca84625363aa0ad198b4f4f25c3339a7b5203c6ab3a3af99356f37ed1'
BASE_GRAPH = '860242d1203695c5d3986e51fddc035b37e0a16c49755a5de9e343bf08e966e5'
BASE_IDS = '48abf62395974509d1de83b9cb09acc9777bab4fd60e0d51aaee7c4a5d85ae66'
HISTORY = '082af527d8dd4140a8d4104f45c62a62b777d24c68837ed03554d14deec25321'
BASE_LOCK = '87ca5b8de8bd19420d6abf06dd49c87d4ad2f44fa915546ef5a3ab3ee616ad53'
BASE_SNAPSHOT = '0c006791f6952eef5fe360f5604976ff6a85eba2f933ab14cde55c8a0a329a47'
BASE_EXECUTION = 'a08e8b34a1708e9d2e82ef4254040f524fb9f46d4b4c15cf135b90651519ea0e'
NEW_CONTRACT = 'SYSON-RUST-COMPLETION/1.0.10'
PRESERVED = {
    'teams.json': '8654745923a5fe258fff0387cbfd11a19fd29e203cec54d69e2026193ba35421',
    'cross-lane-review.json': 'b044d26288b26c2f7fd7ae30b38cd4092e55ebeb0e23a7c9012bd9e5d1b17b97',
    'rereview-overlay.json': '35fb1c586a94b7f7cac7656e5e06533569dca1c5e2fe09fd7c862d6c31ba163e',
    'rereview-batch11-coordinator-ownership.json': '7cc5303b2d5c85e45f571b12e39404f6684a22607a8323a40e6eb8970599875a',
    'performance-policy.json': '62990fca393be5443eb80c6dd106692e721704f0cdb6df708dcbf6472f739a18',
    'TEAM-EXECUTION.md': '8af52e32ca87dc5b04883c03845064c8d91c850dfc010012b353b1cd27d7e64c'}
EXTRA = {'lanes/storage-source.json', 'storage-policy-source.json', 'execution.json',
         'amendments/1.0.10-storage-source.md', 'tools/storage_splits.py', 'tools/test_storage_splits.py',
         'composition/storage_split_guard.py', 'composition/registration/prepare.py',
         'composition/registration/test_prepare.py', 'composition/registration/README.md',
         'remaining-work/checkpoint-storage-1.0.10/prepare_storage_registration.py'}
_PROVIDERS = {}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def encode(value, canonical=False):
    text = json.dumps(value, sort_keys=canonical, ensure_ascii=False, allow_nan=False,
                      separators=(',', ':') if canonical else None, indent=None if canonical else 2)
    return (text + ('' if canonical else '\n')).encode()


def digest(value):
    return sha(encode(value, True))


def parse(raw):
    require(type(raw) is bytes and len(raw) <= LIMIT, 'finite byte input required')
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, 'duplicate JSON key')
            result[key] = value
        return result
    def nonfinite(value):
        raise ValueError('nonfinite JSON: ' + value)
    value = json.loads(raw, object_pairs_hook=pairs, parse_constant=nonfinite)
    encode(value, True)
    return value


def inputs(files):
    require(type(files) is dict and all(type(v) is bytes for v in files.values()), 'byte map required')
    used = sum(map(len, files.values()))
    require(used <= LIMIT, 'aggregate input limit')
    for name in files:
        require(type(name) is str, 'string input key required')
        path = PurePosixPath(name)
        require(type(name) is str and str(path) == name and not path.is_absolute() and
                all(part not in ('.', '..') for part in path.parts), 'canonical input key required')
        used += len(encode({name: '0' * 64}))
        require(used <= LIMIT, 'aggregate input provenance framing limit')
    return {name: sha(raw) for name, raw in files.items()}


def packet(files, artifacts):
    return {'status': 'UNSEALED_PROPOSAL_ONLY', 'input_sha256': inputs(files), 'artifacts': artifacts}


def _graph(units, joint):
    byid = {u['id']: u for u in units}
    require(len(byid) == len(units), 'duplicate unit')
    result = {}
    for uid, unit in byid.items():
        dependencies = unit.get('implementation_ready_dependencies', unit['dependencies'])
        require(len(dependencies) == len(set(dependencies)), 'duplicate dependency')
        result['implementation:' + uid] = ['implementation:' + d for d in dependencies]
        edges = ['implementation:' + uid] + ['accepted:' + d for d in
                 unit.get('acceptance_children', []) + unit.get('acceptance_gates', [])]
        edges += ['implementation:' + d for d in unit.get('acceptance_implementation_inputs', [])]
        if uid == 'C99':
            edges += ['accepted:' + d for d in byid if d != 'C99']
        result['accepted:' + uid] = sorted(set(edges))
    for uid in joint:
        pending, needed = list(byid[uid]['acceptance_implementation_inputs']), set()
        while pending:
            item = pending.pop()
            require(item in byid, 'unknown joint input')
            if item not in needed:
                needed.add(item)
                unit = byid[item]
                pending.extend(unit.get('implementation_ready_dependencies', unit['dependencies']))
                pending.extend(unit.get('acceptance_children', []))
        byid[uid]['expanded_implementation_inputs'] = sorted(needed)
        result['accepted:' + uid] = sorted(set(result['accepted:' + uid] +
                                              ['implementation:' + d for d in needed]))
    active, visited = set(), set()
    def visit(node):
        require(node in result and node not in active, 'unknown readiness node or typed cycle')
        if node in visited:
            return
        active.add(node)
        for dependency in result[node]:
            visit(dependency)
        active.remove(node)
        visited.add(node)
    for node in result:
        visit(node)
    return result


def derive(files):
    inputs(files)
    require(sha(files['plan.json']) == BASE_PLAN and sha(files['completion-contract.lock.json']) == BASE_LOCK,
            'fixed complete baseline plan/lock changed')
    for name, pin in parse(files['completion-contract.lock.json'])['sha256'].items():
        require(sha(files[name]) == pin, 'old required source pin changed: ' + name)
    old = parse(files['plan.json'])
    require(sha(files['execution.json']) == BASE_EXECUTION, 'complete execution attribution changed')
    contract, state = parse(files['completion-contract.json']), parse(files['execution.json'])
    require(digest(contract) == BASE_CONTRACT and digest(state['units']) == HISTORY, 'baseline/history changed')
    normalized = [{k: v for k, v in u.items() if k not in OMIT} for u in sorted(old['units'], key=lambda u: u['id'])]
    require(len(normalized) == 792 and digest(normalized) == BASE_UNITS and
            digest(old['readiness_graph']) == BASE_GRAPH, 'all old records/edges required')
    for name, expected in PRESERVED.items():
        require(sha(files[name]) == expected, 'held policy changed: ' + name)
    policy = parse(files['storage-policy-source.json'])
    require(policy.pop('parent_intent') == {'RR13': CHILDREN} and policy['preserved_inputs'] == PRESERVED,
            'fixed RR13 intent/holds required')
    fixed = {'schema': 1, 'base_contract_id': contract['contract_id'], 'base_contract_sha256': BASE_CONTRACT,
        'original_unit_count': 792, 'original_ids_sha256': BASE_IDS, 'original_normalized_units_sha256': BASE_UNITS,
        'original_readiness_graph_sha256': BASE_GRAPH, 'historical_unit_count': 55, 'historical_units_sha256': HISTORY}
    require(set(policy) == set(fixed) | {'amendment', 'preserved_inputs', 'children', 'record_updates', 'graph_additions'}
            and all(type(policy[k]) is type(v) and policy[k] == v for k, v in fixed.items()), 'fixed policy header')
    require(not any(policy[k] for k in ['children', 'record_updates', 'graph_additions']), 'unsigned empty template required')
    require(policy['amendment']['path'] == 'amendments/1.0.10-storage-source.md' and
            sha(files[policy['amendment']['path']]) == policy['amendment']['sha256'], 'new amendment pin changed')
    units = copy.deepcopy(old['units'])
    joint = parse(files['cross-lane-review.json'])['joint_acceptance_gates']
    require(_graph(copy.deepcopy(units), joint) == old['readiness_graph'], 'old graph does not recompile')
    lane = parse(files['lanes/storage-source.json'])
    require(lane['lane'] == 'storage-source' and [u['id'] for u in lane['units']] == CHILDREN, 'fixed child coverage')
    for raw, deps, team in zip(lane['units'], [['RR04'], ['RR13a']], ['services-persistence', 'verification-release']):
        row = copy.deepcopy(raw)
        require(row['dependencies'] == row['implementation_ready_dependencies'] == deps and
                row['primary_team'] == team and row['tasks'] and row['verification'] and row['paths'] and
                row.get('source_freeze'), 'child contract')
        require(all(type(row['budget'][k]) is int and row['budget'][k] == v for k, v in
                    [('production_lines_max', 400), ('authored_lines_max', 900)]), 'ordinary child caps')
        row.update(lane=lane['lane'], source_fragment='lanes/storage-source.json',
                   source_fragment_sha256=sha(files['lanes/storage-source.json']))
        row['paths'] = [p.split(' (')[0] for p in row['paths']]
        row['external_pr_details'] = copy.deepcopy(row.get('external_prs', []))
        row['external_prs'] = [int(str(x.get('url') if isinstance(x, dict) else x).rstrip('/').split('/')[-1].replace('#', ''))
                              for x in row['external_pr_details']]
        units.append(row)
    byid = {u['id']: u for u in units}
    parent = byid['RR13']
    parent['implementation_ready_dependencies'] = sorted(set(parent.get('implementation_ready_dependencies', parent['dependencies']) + CHILDREN))
    parent['acceptance_children'] = sorted(set(parent.get('acceptance_children', []) + CHILDREN))
    parent['source_preparation_children'] = CHILDREN.copy()
    order = byid['C99']['dependencies'].copy()
    fragments = [byid[uid]['source_fragment'] for uid in order]
    require(fragments == sorted(fragments), 'old sorted lane order required')
    position = next((i for i, fragment in enumerate(fragments) if fragment > 'lanes/storage-source.json'), len(order))
    order[position:position] = CHILDREN
    byid['C99']['dependencies'] = order
    compiled = _graph(units, joint)
    changes = {}
    for before in old['units']:
        after = byid[before['id']]
        require(set(before) <= set(after), 'old field removed')
        fields = {k: {'before': {'present': k in before, 'value': before.get(k)}, 'after': v}
                  for k, v in after.items() if k not in OMIT and (k not in before or v != before[k])}
        if fields:
            require(before['id'] in {'RR13', 'C99'} | set(joint), 'unexpected parent rewrite')
            changes[before['id']] = fields
    for node, edges in old['readiness_graph'].items():
        require(set(edges) <= set(compiled[node]), 'old edge removed')
    policy['record_updates'] = changes
    policy['graph_additions'] = {n: [e for e in edges if e not in old['readiness_graph'].get(n, [])]
                                 for n, edges in compiled.items() if edges != old['readiness_graph'].get(n)}
    policy['children'] = {uid: {'family': 'repairs', 'team': byid[uid]['primary_team'],
        'dependencies': byid[uid]['dependencies'], 'normalized_sha256': digest({k: v for k, v in byid[uid].items() if k not in OMIT})} for uid in CHILDREN}
    contract = copy.deepcopy(contract)
    contract.update(version='1.0.10', contract_id=NEW_CONTRACT, storage_source_splits=policy)
    contract['unit_ids'] += CHILDREN
    contract['families']['repairs'] += CHILDREN
    for unit in units:
        family = 'repairs' if unit['id'] in CHILDREN else unit['completion_contract']['family']
        unit['completion_contract'] = {'id': NEW_CONTRACT, 'path': 'completion-contract.json',
            'document': contract['document'], 'packet_schema': contract['packet_schema'], 'family': family}
    expected = copy.deepcopy(old)
    expected.update(unit_count=794, units=sorted(units, key=lambda u: u['id']), readiness_graph=compiled)
    expected['completion_contract']['id'] = NEW_CONTRACT
    schema = parse(files['acceptance-packet.schema.json'])
    schema['properties']['unit_id']['enum'] = contract['unit_ids'].copy()
    schema['properties']['contract_id']['const'] = NEW_CONTRACT
    return packet(files, {'completion-contract.json': contract, 'acceptance-packet.schema.json': schema,
                          'expected-plan.json': expected, 'policy.json': policy})


def storage_guard(files):
    root = Path(__file__).resolve().parents[2]
    preparation_path = root / 'tools/preparation_splits.py'
    guard_path = root / 'composition/storage_split_guard.py'
    preparation_raw, guard_raw = preparation_path.read_bytes(), guard_path.read_bytes()
    require(sha(preparation_raw) == '7ea701446b674ab0c4277c02e1c6e265630de3588b9675261216046d86149aa3',
            'fixed preparation provider changed')
    assignment = rb"(?m)^POLICY_SHA256 = '(?:PENDING_ROOT_FREEZE|[0-9a-f]{64})'$"
    require(len(re.findall(assignment, guard_raw)) == 1, 'single fixed guard seal assignment required')
    normalized = re.sub(assignment, b"POLICY_SHA256 = 'PENDING_ROOT_FREEZE'", guard_raw)
    require(sha(normalized) == '2672f0a27053abf8db807fa4356b16697a6252879fa08567ff2e065e66fb33d9',
            'fixed guard body changed')
    require(files['tools/preparation_splits.py'] == preparation_raw and
            files['composition/storage_split_guard.py'] == guard_raw, 'actual provider differs from supplied bindings')
    key = (sha(preparation_raw), sha(guard_raw))
    if key not in _PROVIDERS:
        preparation = types.ModuleType('preparation_splits')
        guard = types.ModuleType('storage_split_guard')
        preparation.__file__, guard.__file__ = str(preparation_path), str(guard_path)
        exec(compile(preparation_raw, str(preparation_path), 'exec'), preparation.__dict__)
        previous = sys.modules.get('preparation_splits')
        sys.modules['preparation_splits'] = preparation
        try:
            exec(compile(guard_raw, str(guard_path), 'exec'), guard.__dict__)
        finally:
            if previous is None:
                del sys.modules['preparation_splits']
            else:
                sys.modules['preparation_splits'] = previous
        _PROVIDERS[key] = guard
    return _PROVIDERS[key]


def contract_lock(files, before):
    require(type(files) is dict and all(type(name) is str for name in files), 'string-keyed byte map required')
    require(not any(name == 'prior' or name.startswith('prior/') for name in files), 'reserved prior input collision')
    inputs(dict(files, **{'prior/contract-lock.json': before}))
    require(sha(before) == BASE_LOCK, 'fixed old contract lock changed')
    result = parse(before)
    required = set(result['sha256']) | EXTRA
    require(required <= set(files) and parse(files['completion-contract.json'])['contract_id'] == NEW_CONTRACT,
            'missing current required input/contract')
    result.update(contract_id=NEW_CONTRACT, sha256={p: sha(files[p]) for p in sorted(required)})
    return packet(dict(files, **{'prior/contract-lock.json': before}), {'completion-contract.lock.json': result})


def snapshot(files, before_register, before_lock, before_contract_lock, snapshot_id, base_main, progress_inputs=()):
    prior = {'prior/register.json': before_register, 'prior/snapshot-lock.json': before_lock,
             'prior/contract-lock.json': before_contract_lock}
    require(type(files) is dict and all(type(name) is str for name in files), 'string-keyed byte map required')
    require(not any(name == 'prior' or name.startswith('prior/') for name in files), 'reserved prior input collision')
    inputs(dict(files, **prior))
    require(sha(before_contract_lock) == BASE_LOCK, 'fixed old contract lock changed')
    require(sha(before_lock) == BASE_SNAPSHOT, 'fixed old snapshot lock changed')
    register, lock = parse(before_register), parse(before_lock)
    require(sha(before_register) == lock['sha256']['remaining-work/units.json'], 'old register pin changed')
    require(type(snapshot_id) is str and snapshot_id.startswith('SYSON-RUST-REMAINING/') and
            snapshot_id != lock['snapshot_id'] and re.fullmatch('[0-9a-f]{40}', base_main), 'new explicit snapshot identity')
    require(sha(files['execution.json']) == BASE_EXECUTION, 'complete execution attribution changed')
    plan, state = parse(files['plan.json']), parse(files['execution.json'])
    contract, current_lock = parse(files['completion-contract.json']), parse(files['completion-contract.lock.json'])
    require(plan['unit_count'] == len(plan['units']) == 794 and contract['contract_id'] == NEW_CONTRACT and
            set(contract['unit_ids']) == {u['id'] for u in plan['units']} and digest(state['units']) == HISTORY,
            'current complete plan/history required')
    require(current_lock == contract_lock(files, before_contract_lock)['artifacts']['completion-contract.lock.json'],
            'current contract lock set/pin changed')
    require(len({u['id'] for u in plan['units']}) == 794 and
            digest(sorted(u['id'] for u in plan['units'] if u['id'] not in CHILDREN)) == BASE_IDS, 'all prior unit identities required')
    for unit in plan['units']:
        require(unit['completion_contract'] == {'id': NEW_CONTRACT, 'path': 'completion-contract.json',
                'document': contract['document'], 'packet_schema': contract['packet_schema'],
                'family': next(f for f, ids in contract['families'].items() if unit['id'] in ids)}, 'current unit pointer changed')
    require(_graph(copy.deepcopy(plan['units']), parse(files['cross-lane-review.json'])['joint_acceptance_gates']) ==
            plan['readiness_graph'], 'current graph does not recompile')
    require(plan['completion_contract'] == {'id': NEW_CONTRACT, 'path': 'completion-contract.json',
            'lock': 'completion-contract.lock.json'}, 'current plan pointer changed')
    owners = [uid for rows in contract['families'].values() for uid in rows]
    require(len(owners) == len(set(owners)) == 794 and set(owners) == set(contract['unit_ids']),
            'current unique family coverage required')
    require(contract['storage_source_splits']['preserved_inputs'] == PRESERVED and
            all(sha(files[name]) == pin for name, pin in PRESERVED.items()), 'fixed held policies changed')
    guard = storage_guard(files)
    guard.validate_storage_splits(contract, plan, read=files.__getitem__)
    fields = sorted({key for unit in plan['units'] for key in unit})
    rows = []
    for unit in plan['units']:
        row = copy.deepcopy(unit)
        row.update(primary_family=unit['completion_contract']['family'], brief='../prs/' + unit['id'] + '.md',
                   brief_sha256=sha(files['prs/' + unit['id'] + '.md']), complete_plan_unit_sha256=digest(unit),
                   feature_acceptance=state['units'].get(unit['id'], {}).get('feature_acceptance', 'OPEN_NO_ACCEPTANCE_RECORD'))
        rows.append(row)
    register.update(snapshot_id=snapshot_id, contract_id=NEW_CONTRACT, base_main=base_main, unit_count=794,
                    copied_plan_fields=fields, units=rows, integrated_scope_at_snapshot=copy.deepcopy(state['current_integration']))
    required = {'README.md', 'REMAINING-WORK.md', 'plan.json', 'execution.json', 'dispatch.json', 'PR-INDEX.md',
        'completion-contract.lock.json', 'remaining-work/units.json', 'remaining-work/verify_snapshot.py', 'team-dispatch.json'}
    require(type(progress_inputs) in (list, tuple) and len(progress_inputs) == len(set(progress_inputs)),
            'explicit unique added progress inputs required')
    lock['progress_inputs'] = sorted(set(lock.get('progress_inputs', [])) | set(progress_inputs))
    required.update(lock['progress_inputs'])
    required.update(current_lock['sha256'])
    required.update('prs/' + unit['id'] + '.md' for unit in plan['units'])
    current = dict(files, **{'remaining-work/units.json': encode(register)})
    require(required <= set(current), 'missing snapshot required input')
    lock.update(snapshot_id=snapshot_id, contract_id=NEW_CONTRACT, base_main=base_main,
                sha256={name: sha(current[name]) for name in sorted(required)})
    return packet(dict(files, **prior),
                  {'units.json': register, 'snapshot.lock.json': lock})


def emit(out, proposal):
    out = Path(out)
    allowed = {'completion-contract.json', 'acceptance-packet.schema.json', 'expected-plan.json', 'policy.json',
               'completion-contract.lock.json', 'units.json', 'snapshot.lock.json'}
    require(set(proposal) == {'status', 'input_sha256', 'artifacts'} and type(proposal['input_sha256']) is dict and
            all(type(pin) is str and re.fullmatch('[0-9a-f]{64}', pin) for pin in proposal['input_sha256'].values()),
            'fixed complete provenance fields required')
    require(proposal['status'] == 'UNSEALED_PROPOSAL_ONLY' and proposal['artifacts'] and set(proposal['artifacts']) <= allowed,
            'unsigned fixed artifact set required')
    require(out.parent.resolve() == out.parent and out.is_relative_to(OUTPUT_ROOT) and
            not out.exists() and not out.is_symlink() and out.parent.stat().st_uid == os.getuid(), 'fresh owned candidate output required')
    artifacts = {name: encode(value) for name, value in proposal['artifacts'].items()}
    manifest = {k: copy.deepcopy(v) for k, v in proposal.items() if k != 'artifacts'}
    manifest.update(artifacts={name: sha(raw) for name, raw in artifacts.items()}, source_sha256=sha(Path(__file__).read_bytes()))
    artifacts['manifest.json'] = encode(manifest)
    require(sum(map(len, artifacts.values())) <= LIMIT, 'aggregate output limit')
    out.mkdir(mode=0o700)
    for name, raw in artifacts.items():
        with (out / name).open('xb') as output:
            output.write(raw)
