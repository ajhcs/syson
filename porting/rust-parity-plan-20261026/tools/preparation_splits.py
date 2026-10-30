"""Sealed 1.0.9 source-preparation additions; no acceptance or lease grant.

Legacy views are available only after validating the current contract/plan. Their
full original fingerprints preserve unknown fields, not selected-key summaries.
"""
import copy
import hashlib
import json

POLICY_SHA256 = 'a26293c00b782b770337c65f10470df3896c8b1064ebd0e48e62ce03726fbbd1'
BASE_CONTRACT = 'fc6f9343183bcc16824d64cfc5caf3510826ebb30a944a3a7dbde0fdc508d090'
BASE_UNITS = '6abb573cf979ef117e39ecc73df4f61fd23cc7c11d691052ebfaf0bc2c4872b8'
BASE_GRAPH = '48fd99ff59ee6528bea1aadb7a9206ad54a42dd0a2a129a76ca07b5dfa086045'
BASE_IDS = 'ab249647449cd5a19208a8c1dff90aca15bf26be73bcea4246ae808887d03dd0'
CHILDREN = {'B25S1', 'M06a1'}
OMIT = {'completion_contract', 'source_fragment_sha256'}


def require(ok, message):
    if not ok:
        raise ValueError(message)


def canonical_hash(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'),
                                    ensure_ascii=False, allow_nan=False).encode()).hexdigest()


def _policy(contract):
    require(type(contract) is dict and contract.get('version') == '1.0.9' and
            contract.get('contract_id') == 'SYSON-RUST-COMPLETION/1.0.9',
            'preparation split current version')
    policy = contract.get('source_preparation_splits')
    require(POLICY_SHA256 != 'PENDING_ROOT_FREEZE' and type(policy) is dict and
            canonical_hash(policy) == POLICY_SHA256, 'preparation split sealed policy')
    require(set(policy) == {'schema', 'amendment', 'base_contract_id', 'base_contract_sha256',
                           'original_unit_count', 'original_ids_sha256',
                           'original_normalized_units_sha256', 'original_readiness_graph_sha256',
                           'children', 'record_updates', 'graph_additions'},
            'preparation split policy fields')
    require(type(policy['schema']) is int and policy['schema'] == 1 and
            policy['base_contract_id'] == 'SYSON-RUST-COMPLETION/1.0.8' and
            policy['base_contract_sha256'] == BASE_CONTRACT and
            type(policy['original_unit_count']) is int and policy['original_unit_count'] == 790 and
            policy['original_ids_sha256'] == BASE_IDS and
            policy['original_normalized_units_sha256'] == BASE_UNITS and
            policy['original_readiness_graph_sha256'] == BASE_GRAPH,
            'preparation split original policy fingerprints')
    require(type(policy['children']) is dict and set(policy['children']) == CHILDREN,
            'preparation split child coverage')
    for child in policy['children'].values():
        require(type(child) is dict and set(child) ==
                {'family', 'team', 'dependencies', 'normalized_sha256'},
                'preparation split child policy fields')
    return policy


def _contract(contract, policy):
    result = copy.deepcopy(contract)
    ids = result['unit_ids']
    require(type(ids) is list and len(ids) == len(set(ids)) == 792 and
            CHILDREN <= set(ids), 'preparation split contract inventory')
    require(not CHILDREN.intersection(result['legacy_prerequisite_budget_dispositions']),
            'preparation split budget exception')
    for uid, child in policy['children'].items():
        require(sum(rows.count(uid) for rows in result['families'].values()) == 1 and
                uid in result['families'][child['family']], 'preparation split child family')
    result['unit_ids'] = [uid for uid in ids if uid not in CHILDREN]
    for family, rows in result['families'].items():
        result['families'][family] = [uid for uid in rows if uid not in CHILDREN]
    del result['source_preparation_splits']
    result['version'] = '1.0.8'
    result['contract_id'] = policy['base_contract_id']
    require(canonical_hash(result) == BASE_CONTRACT, 'preparation split original contract changed')
    return result


def _normalized(unit):
    return {key: value for key, value in unit.items() if key not in OMIT}


def _graph(graph, policy, byid):
    require(type(graph) is dict and set(graph) ==
            {stage + uid for stage in ['implementation:', 'accepted:'] for uid in byid},
            'preparation split typed graph coverage')
    for edges in graph.values():
        require(type(edges) is list and all(type(edge) is str for edge in edges) and
                len(edges) == len(set(edges)) and set(edges) <= set(graph),
                'preparation split typed graph edges')
    for uid, child in policy['children'].items():
        unit = byid[uid]
        pending = list(unit.get('acceptance_implementation_inputs', []))
        needed = set()
        while pending:
            dep = pending.pop()
            require(dep in byid, 'preparation split unknown child implementation input')
            if dep in needed:
                continue
            needed.add(dep)
            provider = byid[dep]
            pending.extend(provider.get('implementation_ready_dependencies', provider['dependencies']))
            pending.extend(provider.get('acceptance_children', []))
        accepted = {'implementation:' + uid}
        accepted.update('accepted:' + dep for dep in unit.get('acceptance_children', []) +
                        unit.get('acceptance_gates', []))
        accepted.update('implementation:' + dep for dep in needed)
        require(graph['implementation:' + uid] == ['implementation:' + dep for dep in
                child['dependencies']] and graph['accepted:' + uid] == sorted(accepted),
                'preparation split child stage semantics')
    original = copy.deepcopy(graph)
    additions = policy['graph_additions']
    require(type(additions) is dict and set(additions) <= set(graph),
            'preparation split graph additions')
    for node, edges in additions.items():
        require(type(edges) is list and len(edges) == len(set(edges)) and
                set(edges) <= set(graph[node]), 'preparation split missing added edge')
        if node.split(':', 1)[1] in CHILDREN:
            require(graph[node] == edges, 'preparation split new node edges')
            del original[node]
        else:
            original[node] = [edge for edge in original[node] if edge not in edges]
    require(len(original) == 1580 and canonical_hash(original) == BASE_GRAPH,
            'preparation split original typed graph changed')
    visited, active = set(), set()
    def visit(node):
        require(node not in active, 'preparation split cycle')
        if node in visited:
            return
        active.add(node)
        for edge in graph[node]:
            visit(edge)
        active.remove(node)
        visited.add(node)
    for node in graph:
        visit(node)
    return original


def _plan(contract, plan, policy):
    require(type(plan) is dict and type(plan.get('units')) is list and
            type(plan.get('unit_count')) is int and plan['unit_count'] == 792,
            'preparation split plan inventory')
    units = plan['units']
    require(all(type(unit) is dict and type(unit.get('id')) is str for unit in units),
            'preparation split unit records')
    byid = {unit['id']: unit for unit in units}
    require(len(byid) == len(units) == 792 and set(byid) == set(contract['unit_ids']),
            'preparation split duplicate/missing unit')
    families = {uid: family for family, ids in contract['families'].items() for uid in ids}
    require(plan.get('completion_contract') == {'id': contract['contract_id'],
            'path': 'completion-contract.json', 'lock': 'completion-contract.lock.json'},
            'preparation split plan pointer')
    for uid, unit in byid.items():
        require(unit.get('completion_contract') == {'id': contract['contract_id'],
                'path': 'completion-contract.json', 'document': contract['document'],
                'packet_schema': contract['packet_schema'], 'family': families[uid]},
                'preparation split current unit pointer: ' + uid)
    for uid, child in policy['children'].items():
        unit = byid[uid]
        require(canonical_hash(_normalized(unit)) == child['normalized_sha256'] and
                unit['dependencies'] == unit['implementation_ready_dependencies'] ==
                child['dependencies'], 'preparation split exact child record: ' + uid)
        require(all(type(unit['budget'][key]) is int and unit['budget'][key] == value
                    for key, value in [('production_lines_max', 400), ('authored_lines_max', 900)]),
                'preparation split child caps')
    result = copy.deepcopy(plan)
    result['units'] = [unit for unit in result['units'] if unit['id'] not in CHILDREN]
    restored = {unit['id']: unit for unit in result['units']}
    for uid, fields in policy['record_updates'].items():
        require(uid in restored and type(fields) is dict, 'preparation split original update owner')
        for field, change in fields.items():
            require(field not in OMIT | {'id'} and type(change) is dict and
                    set(change) == {'before', 'after'} and field in restored[uid] and
                    canonical_hash(restored[uid][field]) == canonical_hash(change['after']),
                    'preparation split exact added record field')
            before = change['before']
            require(type(before) is dict and set(before) == {'present', 'value'} and
                    type(before['present']) is bool, 'preparation split restoration presence')
            if before['present']:
                restored[uid][field] = copy.deepcopy(before['value'])
            else:
                require(before['value'] is None, 'preparation split absent original value')
                del restored[uid][field]
    normalized = [_normalized(unit) for unit in sorted(result['units'], key=lambda row: row['id'])]
    require(canonical_hash(normalized) == BASE_UNITS, 'preparation split all790 original records changed')
    result['readiness_graph'] = _graph(plan['readiness_graph'], policy, byid)
    result['unit_count'] = 790
    result['completion_contract']['id'] = policy['base_contract_id']
    for unit in result['units']:
        unit['completion_contract']['id'] = policy['base_contract_id']
    return result


def validate_preparation_splits(contract, plan=None):
    """Validate before exclusions; no old-version fallback or runtime acceptance."""
    policy = _policy(contract)
    _contract(contract, policy)
    if plan is not None:
        _plan(contract, plan, policy)
    return copy.deepcopy(policy)


def legacy_contract(contract):
    """Return the exact sealed 1.0.8 contract after current policy admission."""
    return _contract(contract, _policy(contract))


def legacy_plan(contract, plan):
    """Restore only sealed additions; callers still run their old validators."""
    policy = validate_preparation_splits(contract)
    return _plan(contract, plan, policy)
