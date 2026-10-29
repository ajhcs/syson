"""Verify this reviewed execution snapshot without refreshing its lock.

This is a documentation-integrity check, not behavioral feature acceptance.
Run from any directory: python3 /absolute/path/to/this/file.py --self-test
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path


def digest(data):
    return hashlib.sha256(data).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def validate(root, read=None):
    read = read or (lambda name: (root / name).read_bytes())
    load = lambda name: json.loads(read(name))
    lock = load('remaining-work/snapshot.lock.json')
    plan = load('plan.json')
    state = load('execution.json')
    register = load('remaining-work/units.json')
    contract = load('completion-contract.json')
    contract_lock = load('completion-contract.lock.json')
    require(lock['snapshot_id'] == register['snapshot_id'], 'snapshot identity')
    require(lock['contract_id'] == register['contract_id'] == contract['contract_id'],
            'contract identity')
    require(lock['base_main'] == register['base_main'], 'base identity')
    required = {'README.md', 'REMAINING-WORK.md', 'plan.json', 'execution.json',
                'dispatch.json', 'PR-INDEX.md', 'completion-contract.lock.json',
                'remaining-work/units.json', 'remaining-work/verify_snapshot.py', 'team-dispatch.json'}
    required.update(lock.get('progress_inputs', []))
    required.update(contract_lock['sha256'])
    required.update('prs/' + unit['id'] + '.md' for unit in plan['units'])
    require(set(lock['sha256']) == required, 'locked input coverage')
    for name, expected in lock['sha256'].items():
        require(digest(read(name)) == expected, 'locked input drift: ' + name)
    for name, expected in contract_lock['sha256'].items():
        require(digest(read(name)) == expected, 'contract input drift: ' + name)
    require(set(register['copied_plan_fields']) ==
            {key for unit in plan['units'] for key in unit}, 'projected field coverage')
    rows = register['units']
    require(register['unit_count'] == plan['unit_count'] == len(rows), 'unit count')
    require([u['id'] for u in rows] == [u['id'] for u in plan['units']],
            'unit identity/order')
    require(set(contract['unit_ids']) == {u['id'] for u in rows}, 'contract coverage')
    for source, row in zip(plan['units'], rows):
        expected = {key: source[key] for key in register['copied_plan_fields']
                    if key in source}
        expected.update(
            primary_family=source['completion_contract']['family'],
            brief='../prs/' + source['id'] + '.md',
            brief_sha256=digest(read('prs/' + source['id'] + '.md')),
            complete_plan_unit_sha256=digest(json.dumps(
                source, sort_keys=True, separators=(',', ':'),
                ensure_ascii=False).encode()),
            feature_acceptance=state['units'].get(source['id'], {}).get(
                'feature_acceptance', 'OPEN_NO_ACCEPTANCE_RECORD'))
        require(row == expected, 'unit projection drift: ' + source['id'])
    require(register['integrated_scope_at_snapshot'] == state['current_integration'],
            'integration attribution')
    return {'result': 'PASS_DOCUMENTATION_SNAPSHOT_INTEGRITY_ONLY',
            'snapshot_id': register['snapshot_id'], 'unit_count': len(rows),
            'locked_inputs': len(required),
            'primary_family_counts': dict(Counter(u['primary_family'] for u in rows)),
            'feature_acceptance_counts': dict(Counter(u['feature_acceptance'] for u in rows)),
            'full_parity_claimed': False}


def self_test(root):
    original = (root / 'remaining-work/units.json').read_bytes()
    for kind in ('omit_unit', 'change_task', 'drop_dependency', 'claim_accepted'):
        data = json.loads(original)
        if kind == 'omit_unit':
            data['units'].pop()
        elif kind == 'change_task':
            data['units'][0]['tasks'][0] = 'Invented task'
        elif kind == 'drop_dependency':
            row = next(u for u in data['units'] if u.get('dependencies'))
            row['dependencies'] = []
        else:
            data['units'][0]['feature_acceptance'] = 'FEATURE_ACCEPTED'
        changed = json.dumps(data).encode()
        # Rehash the mutated projection to ensure semantic comparison also refuses it.
        lock = json.loads((root / 'remaining-work/snapshot.lock.json').read_bytes())
        lock['sha256']['remaining-work/units.json'] = digest(changed)
        def read(name):
            if name == 'remaining-work/units.json':
                return changed
            if name == 'remaining-work/snapshot.lock.json':
                return json.dumps(lock).encode()
            return (root / name).read_bytes()
        try:
            validate(root, read)
        except ValueError:
            continue
        raise AssertionError('Snapshot accepted ' + kind)
    for target in ('REMAINING-WORK.md', 'lanes/model.json', 'prs/RR04.md'):
        def read(name):
            data = (root / name).read_bytes()
            return data + b'\n' if name == target else data
        try:
            validate(root, read)
        except ValueError:
            continue
        raise AssertionError('Snapshot accepted input drift: ' + target)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    result = validate(root)
    if args.self_test:
        self_test(root)
        result['negative_controls'] = 'PASS_7_REFUSAL_CASES'
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
