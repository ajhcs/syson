"""Inactive .12 restoration guard over immutable whole-qualified .11 programs."""
import copy
from . import derive as D

POLICY_SHA256 = 'PENDING_ROOT_FREEZE'


def checked(contract, plan, read, lane, amendment, generator):
    D.require(POLICY_SHA256 != 'PENDING_ROOT_FREEZE', 'identity partition policy pending')
    D.require(callable(read), 'explicit prior byte reader required')
    Q, S, P, _ = D.provider()
    S.bound(contract, plan)
    D.require(type(contract) is dict and contract.get('version') == '1.0.12' and
              contract.get('contract_id') == D.VERSION, 'fixed current partition version')
    policy = contract.get('identity_partition_splits')
    D.require(type(policy) is dict and D.digest(policy) == POLICY_SHA256, 'sealed partition policy')
    D.require(type(plan) is dict and type(plan.get('units')) is list and
              plan.get('unit_count') == len(plan['units']) == 799, 'complete current unit count')
    D.require(all(type(u) is dict and type(u.get('id')) is str for u in plan['units']), 'unit ID types')
    ids = [u['id'] for u in plan['units']]
    D.require(ids == sorted(set(ids)) and set(D.IDS) <= set(ids), 'unique sorted complete units')
    old_contract = copy.deepcopy(contract)
    del old_contract['identity_partition_splits']
    for name in ['unit_ids']:
        D.require(type(old_contract.get(name)) is list, 'contract ID list')
        old_contract[name] = [uid for uid in old_contract[name] if uid not in D.IDS]
    D.require(type(old_contract.get('families')) is dict and
              type(old_contract['families'].get('model')) is list, 'model family list')
    old_contract['families']['model'] = [uid for uid in old_contract['families']['model'] if uid not in D.IDS]
    old_contract.update(version='1.0.11', contract_id='SYSON-RUST-COMPLETION/1.0.11')
    old = copy.deepcopy(plan)
    old['units'] = [u for u in old['units'] if u['id'] not in D.IDS]
    byid = {u['id']: u for u in old['units']}
    D.require('C99' in byid and len(byid) == 797, 'complete old unit inventory')
    D.require(type(policy.get('record_updates')) is dict, 'record update map')
    for uid, fields in policy['record_updates'].items():
        D.require(uid in byid and type(fields) is dict, 'old record update owner')
        for key, change in fields.items():
            D.require(type(change) is dict and set(change) == {'before', 'after'} and
                      key in byid[uid] and D.digest(byid[uid][key]) == D.digest(change['after']),
                      'exact after field')
            before = change['before']
            D.require(type(before) is dict and set(before) == {'present', 'value'} and
                      type(before['present']) is bool, 'old field presence')
            if before['present']:
                byid[uid][key] = copy.deepcopy(before['value'])
            else:
                D.require(before['value'] is None, 'absent old field')
                del byid[uid][key]
    for unit in old['units']:
        D.require(type(unit.get('completion_contract')) is dict, 'unit contract pointer')
        unit['completion_contract']['id'] = 'SYSON-RUST-COMPLETION/1.0.11'
    byid['C99']['source_fragment_sha256'] = D.BASE_GENERATOR
    old['unit_count'] = 797
    D.require(type(old.get('completion_contract')) is dict, 'plan contract pointer')
    old['completion_contract']['id'] = 'SYSON-RUST-COMPLETION/1.0.11'
    joint = read('cross-lane-review.json')
    D.require(type(joint) is bytes and D.sha(joint) == P.PRESERVED['cross-lane-review.json'],
              'prior joint bytes')
    old['readiness_graph'] = P._graph(old['units'], P.parse(joint)['joint_acceptance_gates'])
    metadata = Q.source_metadata(P, old['units'])
    for key in ['lane_counts', 'source_author_dispatch_waves', 'unordered_write_conflicts']:
        old[key] = metadata[key]
    D.require(D.digest(old_contract) == D.BASE_CONTRACT and D.digest(old) == D.BASE_PLAN,
              'whole previous contract or plan changed')
    S.validate_source_next_splits(old_contract, old, read)
    expected = D.build(old_contract, old, lane, amendment, generator, read)
    D.require(D.digest(contract) == D.digest(expected['completion-contract.json']) and
              D.digest(plan) == D.digest(expected['expected-plan.json']), 'exact complete current partition')
    return copy.deepcopy(policy), old_contract, old


def validate_partition(contract, plan, read, lane, amendment, generator):
    return checked(contract, plan, read, lane, amendment, generator)[0]
