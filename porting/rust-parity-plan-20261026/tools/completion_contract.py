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
    return contract


def validate_bindings(contract, plan, root):
    require(plan['target_source'] == contract['target_source'], 'Contract target mismatch')
    require(plan.get('completion_contract') == {'id': contract['contract_id'],
            'path': CONTRACT_PATH, 'lock': LOCK_PATH}, 'Missing plan contract binding')
    require({u['id'] for u in plan['units']} == set(contract['unit_ids']),
            'Plan/contract unit coverage mismatch')
    families = {uid: family for family, ids in contract['families'].items() for uid in ids}
    for unit in plan['units']:
        require(unit.get('completion_contract') == pointer(contract, families[unit['id']]),
                'Missing/changed unit contract binding: ' + unit['id'])
        brief = (root / 'prs' / (unit['id'] + '.md')).read_text()
        for expected in [contract['contract_id'], '../' + CONTRACT_PATH,
                         '../' + contract['document'], '../' + contract['packet_schema'],
                         'Primary acceptance family: ' + families[unit['id']]]:
            require(expected in brief, 'Missing brief contract binding: ' + unit['id'])
