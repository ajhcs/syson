"""Inactive fixed identity partition proposals; imports perform no source reads."""
import copy
import hashlib
import json
import sys
import types
from pathlib import Path

LIMIT = 64 * 1024**2
NAME_LIMIT = 4096
VERSION = 'SYSON-RUST-COMPLETION/1.0.12'
IDS = ['M06a2a', 'M06a2b']
FRAGMENT = 'lanes/identity-partition.json'
AMENDMENT = 'amendments/1.0.12-identity-integration.md'
LANE_SHA = '765ab7a527f3489cb46e97a56a627d151c7a993aff151f0d2c6e3e8c2e7c0464'
AMENDMENT_SHA = '85b57839fa9ce620791dcfc98bd9df0c1959c8a3f1a510cbf19ed6d27675fc9d'
ROWS = ['770e01b204ff9237df86682ac4b65730881dd557ea209f8d1c436d8e0f6de852',
        '0029b9ce1126596b34893f87cacab5bbc479ec2e817c34886887f499fbbee65a']
BASE_CONTRACT = '502aa44f21663011d3f8030ed41d398b29b3659677f1f460fe0b6c4ee806d585'
BASE_PLAN = '2fc2e9fb8c3ac121cb397a8c075f8ad54238e2a42be8fcd84a843d095f036e11'
BASE_RAW_PLAN = '4b10b1df933a555debe3c5d2e473d354fabb7b4acf8cfffd3386a881a0bc5691'
BASE_LOCK = 'a36a19cdcbb24441df3afd8f67b9ddd3f8648e0051f09ab387d9f7668d05c70f'
BASE_SNAPSHOT = 'ae852f9d77ec55ccc0e0178b73858c9da0abee43991bcf3e46d9cf970499bbc0'
BASE_GENERATOR = '708982b09860061a0108328877e15f7b3a66c13e2adfd9ef9fe224384ce1be4c'
PROGRESS = '9c8fca380a2814ec7c90612b7e31f14ff2a17394ad0dc4b724fa26bbfea120d4'
BASE_SCHEMA = '44da955df9840636ca4b8f0ec540e9397418625f1785a18cb2be4c9da29f9914'
OWNER = 'rereview-batch11-coordinator-ownership.json'
EXTRA = {'plan.json', 'completion-contract.lock.json', 'remaining-work/snapshot.lock.json', OWNER}
SOURCE_PINS = {'derive': 'fbb0352325dc948fdb1625348db69a4810dbfa61cc276a9c235716cecc8c9f17',
    'guard': 'c26d1446bddccd6475a6b921f52e2db660b2d0857460434bd9ef45110c4f645b',
    'construct': '89d56e9bde5aa8627346feb7e50ea7447572f0e452d1e74e95d40dc148b682a9'}
_OLD = None


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def digest(value):
    return sha(json.dumps(value, sort_keys=True, ensure_ascii=False,
                          allow_nan=False, separators=(',', ':')).encode())


def provider():
    global _OLD
    directory = Path(__file__).resolve().parent.parent / 'source_next_prepare'
    sources = {}
    for name, expected in SOURCE_PINS.items():
        path = directory / (name + '.py')
        require(path.resolve() == path and path.is_file() and not path.is_symlink()
                and path.stat().st_size <= 65536, 'canonical bounded old provider')
        with path.open('rb') as stream:
            raw = stream.read(65537)
        require(len(raw) <= 65536 and sha(raw) == expected, 'whole old provider changed')
        sources[name] = (path, raw)
    package_name = '_syson_identity_partition_prior_11'
    if _OLD is None:
        require(not any(n == package_name or n.startswith(package_name + '.')
                        for n in sys.modules), 'old provider namespace collision')
        package = types.ModuleType(package_name)
        package.__path__ = [str(directory)]
        sys.modules[package_name] = package
        for name in ['derive', 'guard']:
            path, raw = sources[name]
            module = types.ModuleType(package_name + '.' + name)
            module.__file__, module.__package__ = str(path), package_name
            sys.modules[module.__name__] = module
            setattr(package, name, module)
            exec(compile(raw, str(path), 'exec'), module.__dict__)
        P, _ = package.derive.provider()
        _OLD = package.derive, package.guard, P
    require(all(sys.modules.get(package_name + '.' + name) is module
                for name, module in zip(['derive', 'guard'], _OLD[:2])), 'old module identity changed')
    _, registration = _OLD[0].provider()
    actual = {str(path): raw for path, raw in sources.values()}
    actual[str(directory.parents[1] / 'composition/registration/prepare.py')] = registration
    return (*_OLD, actual)


def preflight(files):
    require(type(files) is dict and len(files) == 64, 'exact64 baseline byte map required')
    used = 0
    for name, raw in files.items():
        require(type(name) is str and type(raw) is bytes, 'exact byte map key/value types')
        require(0 < len(name) <= NAME_LIMIT, 'input name length limit')
        require(not any(0xD800 <= ord(c) <= 0xDFFF for c in name), 'input name surrogate')
        used += len(raw)
        require(used <= LIMIT, 'aggregate input limit')
        used += 128 + 6 * len(name)
        require(used <= LIMIT, 'aggregate input name framing limit')


def baseline(files):
    preflight(files)
    Q, S, P, _ = provider()
    P.inputs(files)
    for name, expected in [('completion-contract.lock.json', BASE_LOCK),
            ('plan.json', BASE_RAW_PLAN), ('remaining-work/snapshot.lock.json', BASE_SNAPSHOT)]:
        require(name in files and sha(files[name]) == expected, 'whole fixed baseline changed')
    lock = P.parse(files['completion-contract.lock.json'])['sha256']
    require(len(lock) == 60 and set(files) == set(lock) | EXTRA, 'exact sixty plus baseline roles')
    require(all(sha(files[n]) == h for n, h in lock.items()), 'old normative source changed')
    contract, plan = P.parse(files['completion-contract.json']), P.parse(files['plan.json'])
    S.bound(contract, plan)
    require(digest(contract) == BASE_CONTRACT and digest(plan) == BASE_PLAN, 'complete old objects')
    snapshot = P.parse(files['remaining-work/snapshot.lock.json'])
    require(len(snapshot['progress_inputs']) == 342 and digest({n: snapshot['sha256'][n]
            for n in snapshot['progress_inputs']}) == PROGRESS, 'all342 prior progress roles')
    S.validate_source_next_splits(contract, plan, files.__getitem__)
    return contract, plan


def build(contract, plan, lane, amendment, generator, read):
    Q, S, P, _ = provider()
    S.bound(contract, plan)
    require(digest(contract) == BASE_CONTRACT and digest(plan) == BASE_PLAN, 'fixed whole old objects')
    require(type(lane) is bytes and sha(lane) == LANE_SHA and
            type(amendment) is bytes and sha(amendment) == AMENDMENT_SHA, 'fixed partition inputs')
    require(type(generator) is bytes and 0 < len(generator) <= 262144, 'bounded proposed generator')
    proposed = P.parse(lane)
    require(set(proposed) == {'lane', 'units'} and proposed['lane'] == 'identity-partition'
            and [u['id'] for u in proposed['units']] == IDS, 'fixed two child rows')
    units = copy.deepcopy(plan['units'])
    for row, expected, deps in zip(proposed['units'], ROWS, [['M06a1', 'RR13a'], ['M06a2a']]):
        require(digest(row) == expected and
                row['dependencies'] == row['implementation_ready_dependencies'] == deps
                and row['primary_family'] == 'model' and row['primary_team'] == 'services-persistence',
                'whole fixed child or dependency changed')
        require(row['budget']['production_lines_max'] == 400 and
                row['budget']['authored_lines_max'] == 900, 'ordinary child caps')
        child = copy.deepcopy(row)
        child.update(lane='identity-partition', source_fragment=FRAGMENT,
                     source_fragment_sha256=sha(lane), external_pr_details=copy.deepcopy(row['external_prs']))
        units.append(child)
    byid = {u['id']: u for u in units}
    require(len(byid) == len(units) == 799, 'unique complete799 units')
    parent = byid['M06a2']
    for field in ['dependencies', 'implementation_ready_dependencies', 'acceptance_children',
                  'source_preparation_children']:
        parent[field] = sorted(set(parent.get(field, []) + IDS))
    order = byid['C99']['dependencies']
    at = next((i for i, uid in enumerate(order) if byid[uid]['source_fragment'] > FRAGMENT), len(order))
    order[at:at] = IDS
    byid['C99']['source_fragment_sha256'] = sha(generator)
    joint_raw = read('cross-lane-review.json')
    require(type(joint_raw) is bytes and sha(joint_raw) == P.PRESERVED['cross-lane-review.json'],
            'fixed joint input')
    joint = P.parse(joint_raw)['joint_acceptance_gates']
    graph = P._graph(units, joint)
    require(len(graph) == 1598 and all(set(edges) <= set(graph[node])
            for node, edges in plan['readiness_graph'].items()), 'all original typed edges')
    changes = {}
    for before in plan['units']:
        after = byid[before['id']]
        fields = {k: {'before': {'present': k in before, 'value': before.get(k)}, 'after': value}
                  for k, value in after.items()
                  if k not in P.OMIT and (k not in before or value != before[k])}
        if fields:
            require(before['id'] in {'M06a2', 'C99'} | set(joint), 'unexpected old record change')
            changes[before['id']] = fields
    policy = {'schema': 1, 'amendment': {'path': AMENDMENT, 'sha256': sha(amendment)},
              'base_contract_sha256': BASE_CONTRACT, 'base_plan_sha256': BASE_PLAN,
              'base_snapshot_sha256': BASE_SNAPSHOT, 'prior_progress_count': 342,
              'prior_progress_sha256': PROGRESS, 'historical_unit_count': 55,
              'historical_units_sha256': P.HISTORY, 'preserved_inputs': copy.deepcopy(P.PRESERVED),
              'children': {uid: digest({k: v for k, v in byid[uid].items()
                  if k not in P.OMIT}) for uid in IDS},
              'record_updates': changes}
    current_contract = copy.deepcopy(contract)
    current_contract.update(version='1.0.12', contract_id=VERSION, identity_partition_splits=policy)
    current_contract['unit_ids'] += IDS
    current_contract['families']['model'] += IDS
    for unit in units:
        unit['completion_contract'] = copy.deepcopy(byid['M06a2']['completion_contract']
            if unit['id'] in IDS else unit['completion_contract'])
        unit['completion_contract']['id'] = VERSION
    metadata = Q.source_metadata(P, units)
    current = copy.deepcopy(plan)
    current.update(unit_count=799, units=sorted(units, key=lambda u: u['id']), readiness_graph=graph,
        **{key: metadata[key] for key in
           ['lane_counts', 'source_author_dispatch_waves', 'unordered_write_conflicts']})
    current['completion_contract']['id'] = VERSION
    schema_raw = read('acceptance-packet.schema.json')
    require(type(schema_raw) is bytes and sha(schema_raw) == BASE_SCHEMA, 'whole prior schema changed')
    schema = P.parse(schema_raw)
    schema['properties']['unit_id']['enum'] = current_contract['unit_ids'].copy()
    schema['properties']['contract_id']['const'] = VERSION
    return {'policy.json': policy, 'completion-contract.json': current_contract,
            'acceptance-packet.schema.json': schema, 'expected-plan.json': current,
            'source-metadata.json': metadata}


def derive(files, lane, amendment, generator):
    contract, plan = baseline(files)
    _, S, P, sources = provider()
    artifacts = build(contract, plan, lane, amendment, generator, files.__getitem__)
    inputs = {'supplied-baseline/' + n: raw for n, raw in files.items()}
    inputs.update({'proposed/lane.json': lane, 'proposed/amendment.md': amendment,
                   'proposed/current-generator.py': generator})
    inputs.update({'actual-provider/' + Path(name).name: raw for name, raw in sources.items()})
    for name in ['derive.py', 'guard.py']:
        path = Path(__file__).resolve().parent / name
        require(path.resolve() == path and path.is_file() and not path.is_symlink()
                and path.stat().st_size <= 65536, 'canonical bounded actual entry source')
        with path.open('rb') as stream:
            raw = stream.read(65537)
        require(len(raw) <= 65536, 'actual entry changed past bound')
        inputs['actual-entry/' + name] = raw
    S.bound(artifacts)
    result = P.packet(inputs, artifacts)
    require(Q_charge(P, inputs) + len(P.encode(result)) <= LIMIT, 'combined unsigned proposal bound')
    return result


def Q_charge(P, files):
    return sum(len(raw) + len(P.encode({name: '0' * 64})) for name, raw in files.items())
