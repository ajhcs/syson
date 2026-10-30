"""Sealed file-store additions; preserve the full 792-unit contract before reuse."""
import copy
import json
import preparation_splits as P

POLICY_SHA256 = '26415d63a468ef7376637ccb3ff905ca6bba79695f3a6d2f94f547603f458f66'
BASE_CONTRACT = '6771410b685f5b9c691705a3821a13202b61535a04e622e8399cfd592fc36e17'
BASE_UNITS = '0e635baca84625363aa0ad198b4f4f25c3339a7b5203c6ab3a3af99356f37ed1'
BASE_GRAPH = '860242d1203695c5d3986e51fddc035b37e0a16c49755a5de9e343bf08e966e5'
BASE_IDS = '48abf62395974509d1de83b9cb09acc9777bab4fd60e0d51aaee7c4a5d85ae66'
HISTORY = '082af527d8dd4140a8d4104f45c62a62b777d24c68837ed03554d14deec25321'
CHILDREN = {'RR13a', 'RR13b'}
require, digest = P.require, P.canonical_hash


def _policy(contract):
    require(type(contract) is dict and contract.get('version') == '1.0.10' and
            contract.get('contract_id') == 'SYSON-RUST-COMPLETION/1.0.10', 'storage current pointer')
    policy = contract.get('storage_source_splits')
    require(POLICY_SHA256 != 'PENDING_ROOT_FREEZE' and type(policy) is dict and
            digest(policy) == POLICY_SHA256, 'storage sealed policy')
    require(set(policy) == {'schema', 'amendment', 'base_contract_id', 'base_contract_sha256',
                           'original_unit_count', 'original_ids_sha256',
                           'original_normalized_units_sha256', 'original_readiness_graph_sha256',
                           'children', 'record_updates', 'graph_additions', 'preserved_inputs',
                           'historical_unit_count', 'historical_units_sha256'}, 'storage policy fields')
    require(type(policy['schema']) is int and policy['schema'] == 1 and
            policy['base_contract_id'] == 'SYSON-RUST-COMPLETION/1.0.9' and
            policy['base_contract_sha256'] == BASE_CONTRACT and
            type(policy['original_unit_count']) is int and policy['original_unit_count'] == 792 and
            policy['original_ids_sha256'] == BASE_IDS and
            policy['original_normalized_units_sha256'] == BASE_UNITS and
            policy['original_readiness_graph_sha256'] == BASE_GRAPH and
            type(policy['historical_unit_count']) is int and policy['historical_unit_count'] == 55 and
            policy['historical_units_sha256'] == HISTORY, 'storage baseline fingerprints')
    require(type(policy['children']) is dict and set(policy['children']) == CHILDREN,
            'storage exact child coverage')
    for child in policy['children'].values():
        require(type(child) is dict and set(child) ==
                {'family', 'team', 'dependencies', 'normalized_sha256'}, 'storage child policy')
    return policy


def storage_policy(contract):
    return copy.deepcopy(_policy(contract))


def previous_contract(contract):
    policy = _policy(contract)
    old = copy.deepcopy(contract)
    ids = old['unit_ids']
    require(type(ids) is list and len(ids) == len(set(ids)) == 794 and CHILDREN <= set(ids),
            'storage contract inventory')
    require(not CHILDREN.intersection(old['legacy_prerequisite_budget_dispositions']),
            'storage budget exception')
    for uid, child in policy['children'].items():
        require(sum(rows.count(uid) for rows in old['families'].values()) == 1 and
                uid in old['families'][child['family']], 'storage child family')
    old['unit_ids'] = [uid for uid in ids if uid not in CHILDREN]
    for family, rows in old['families'].items():
        old['families'][family] = [uid for uid in rows if uid not in CHILDREN]
    del old['storage_source_splits']
    old.update(version='1.0.9', contract_id=policy['base_contract_id'])
    require(digest(old) == BASE_CONTRACT, 'storage exact previous contract')
    P.validate_preparation_splits(old)
    return old


def previous_plan(contract, plan):
    policy = _policy(contract)
    previous_contract(contract)
    require(type(plan) is dict and type(plan.get('units')) is list and
            type(plan.get('unit_count')) is int and plan['unit_count'] == 794, 'storage plan count')
    byid = {unit['id']: unit for unit in plan['units']}
    require(len(byid) == len(plan['units']) == 794 and set(byid) == set(contract['unit_ids']),
            'storage exact plan inventory')
    require(plan.get('completion_contract') == {'id': contract['contract_id'],
            'path': 'completion-contract.json', 'lock': 'completion-contract.lock.json'},
            'storage current plan pointer')
    families = {uid: family for family, ids in contract['families'].items() for uid in ids}
    for uid, unit in byid.items():
        require(unit.get('completion_contract') == {'id': contract['contract_id'],
                'path': 'completion-contract.json', 'document': contract['document'],
                'packet_schema': contract['packet_schema'], 'family': families[uid]},
                'storage current unit pointer: ' + uid)
    for uid, child in policy['children'].items():
        unit = byid[uid]
        require(digest(P._normalized(unit)) == child['normalized_sha256'] and
                unit['dependencies'] == unit['implementation_ready_dependencies'] ==
                child['dependencies'], 'storage exact child record: ' + uid)
        require(all(type(unit['budget'][key]) is int and unit['budget'][key] == value
                    for key, value in [('production_lines_max', 400), ('authored_lines_max', 900)]),
                'storage ordinary caps')
    old = copy.deepcopy(plan)
    old['units'] = [unit for unit in old['units'] if unit['id'] not in CHILDREN]
    restored = {unit['id']: unit for unit in old['units']}
    for uid, fields in policy['record_updates'].items():
        require(uid in restored and type(fields) is dict, 'storage update owner')
        for field, change in fields.items():
            require(field not in P.OMIT | {'id'} and set(change) == {'before', 'after'} and
                    field in restored[uid] and digest(restored[uid][field]) == digest(change['after']),
                    'storage exact changed field')
            before = change['before']
            require(set(before) == {'present', 'value'} and type(before['present']) is bool,
                    'storage prior field presence')
            if before['present']:
                restored[uid][field] = copy.deepcopy(before['value'])
            else:
                require(before['value'] is None, 'storage absent prior field')
                del restored[uid][field]
    normalized = [P._normalized(unit) for unit in sorted(old['units'], key=lambda row: row['id'])]
    require(digest(normalized) == BASE_UNITS, 'storage all792 original records')
    graph = plan['readiness_graph']
    require(type(graph) is dict and set(graph) ==
            {stage + uid for stage in ['implementation:', 'accepted:'] for uid in byid},
            'storage typed graph inventory')
    for edges in graph.values():
        require(type(edges) is list and len(edges) == len(set(edges)) and
                all(type(edge) is str and edge in graph for edge in edges), 'storage graph edges')
    for uid, child in policy['children'].items():
        unit = byid[uid]
        accepted = {'implementation:' + uid}
        accepted.update('accepted:' + dep for dep in unit.get('acceptance_children', []) +
                        unit.get('acceptance_gates', []))
        require(not unit.get('acceptance_implementation_inputs'), 'storage undeclared graph expansion')
        require(graph['implementation:' + uid] == ['implementation:' + dep for dep in
                child['dependencies']] and graph['accepted:' + uid] == sorted(accepted),
                'storage child typed meaning')
    original_graph = copy.deepcopy(graph)
    for node, edges in policy['graph_additions'].items():
        require(node in graph and type(edges) is list and len(edges) == len(set(edges)) and
                set(edges) <= set(graph[node]), 'storage exact added edges')
        if node.split(':', 1)[1] in CHILDREN:
            require(graph[node] == edges, 'storage complete new node')
            del original_graph[node]
        else:
            original_graph[node] = [edge for edge in original_graph[node] if edge not in edges]
    require(len(original_graph) == 1584 and digest(original_graph) == BASE_GRAPH,
            'storage all1584 original typed nodes')
    active, visited = set(), set()
    def visit(node):
        require(node not in active, 'storage typed cycle')
        if node in visited:
            return
        active.add(node)
        for edge in graph[node]:
            visit(edge)
        active.remove(node)
        visited.add(node)
    for node in graph:
        visit(node)
    old['readiness_graph'] = original_graph
    old['unit_count'] = 792
    old['completion_contract']['id'] = policy['base_contract_id']
    for unit in old['units']:
        unit['completion_contract']['id'] = policy['base_contract_id']
    P.validate_preparation_splits(previous_contract(contract), old)
    return old


def validate_storage_splits(contract, plan=None, read=None):
    policy = _policy(contract)
    previous_contract(contract)
    if plan is not None:
        previous_plan(contract, plan)
    if read is not None:
        for path, sha256 in policy['preserved_inputs'].items():
            require(P.hashlib.sha256(read(path)).hexdigest() == sha256, 'storage preserved file hash mismatch: ' + path)
        history = json.loads(read('execution.json'))['units']
        require(len(history) == 55 and digest(history) == HISTORY, 'storage all55 historical records')
    return copy.deepcopy(policy)


def legacy_contract(contract):
    return P.legacy_contract(previous_contract(contract) if contract.get('version') == '1.0.10' else contract)


def legacy_plan(contract, plan):
    if contract.get('version') == '1.0.10':
        return P.legacy_plan(previous_contract(contract), previous_plan(contract, plan))
    return P.legacy_plan(contract, plan)


def validate_preparation_splits(contract, plan=None):
    if contract.get('version') != '1.0.10':
        return P.validate_preparation_splits(contract, plan)
    policy = validate_storage_splits(contract, plan)
    result = P.validate_preparation_splits(previous_contract(contract),
                                         previous_plan(contract, plan) if plan is not None else None)
    result['children'].update(policy['children'])
    return result
