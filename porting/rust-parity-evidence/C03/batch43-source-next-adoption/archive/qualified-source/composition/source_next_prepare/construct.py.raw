"""Fixed unsigned .11 lock/snapshot construction; never adopts or seals."""
import copy
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
from . import guard as G

LIMIT = 64 * 1024**2
ROOT = Path(__file__).resolve().parents[2]
OUTPUT_ROOT = Path('/workspace/build/builderbob')
BASE = 'remaining-work/checkpoint-source-next-1.0.11/'
PRIOR = BASE + 'prior/'
PRIOR_PINS = {
    'contract-lock.json': '5a214abc075f58cf582304781db6ed5d1e0dfcd96ea4f87504b6f16147e98558',
    'snapshot-lock.json': '474c6d7741e5f794e1bebc5a068c40b6ea0f27925a4fcc431bcdc152c81f7f69',
    'register.json': '1f46899f71c12c4a1e2397119ece727e70864430dcae64c88c074e7f397efb76'}
ADDITIONS = {'lanes/source-next.json', 'amendments/1.0.11-source-next.md', 'tools/source_next_splits.py'}
ADDITIONS.update('composition/source_next_prepare/' + n for n in
    ['derive.py', 'guard.py', 'test_derive.py', 'test_guard.py', 'README.md', 'GUARD.md',
     'construct.py', 'test_construct.py', 'CONSTRUCT.md'])
PROOFS = tuple(BASE + 'proof/' + n for n in [
    'derive/qualification.json', 'derive/archive/reviewer-source-02.json',
    'derive/archive/reviewer-checks-spec-02.json', 'derive/archive/reviewer-executed-02.json',
    'derive/archive/reviewer-candidate-data-02.json', 'derive/archive/candidate-normal-02/manifest.json',
    'derive/archive/candidate-two-origins.json', 'derive/archive/reviewer-lane-fidelity.json',
    'derive/archive/failure-01.json', 'guard/qualification.json',
    'guard/archive/reviewer-guard-source-02.json', 'guard/archive/reviewer-checks-spec-03.json',
    'guard/archive/reviewer-executed-03.json', 'guard/archive/source-freeze-03.json',
    'guard/archive/source-review-hold-01.json', 'guard/archive/reviewer-guard-source-findings-01.json',
    'guard/archive/postprocess-failure-01.json'])
PROOFS_SHA = 'dfd5b21e02a885c90b1afa89f4036a72148881eb6c38c9a5e5141f83c11e5bb1'
FIXTURES = {'composition/source_next_prepare/fixtures/' + n: h for n, h in [
    ('baseline-1.0.10.json.gz', G.D.CAPSULE_SHA), ('baseline-provenance.json', G.D.PROVENANCE_SHA),
    ('lane.json', G.D.LANE_SHA), ('amendment.md', G.D.AMENDMENT_SHA)]}
SOURCE_PINS = {'derive.py': 'fbb0352325dc948fdb1625348db69a4810dbfa61cc276a9c235716cecc8c9f17',
    'guard.py': 'eb80f827c8014cee147ed42c6efafa1f3856ff250fd2ea99a275cfd46aef12d8'}
BOUNDS = {'completion-contract.lock.json': 256 * 1024, 'units.json': 6 * 1024**2,
          'snapshot.lock.json': 256 * 1024, 'manifest.json': 512 * 1024}


def require(ok, message):
    if not ok:
        raise ValueError(message)


def encode(value):
    return (json.dumps(value, ensure_ascii=False, allow_nan=False, indent=2) + '\n').encode()


def parse(P, raw):
    return P.parse(raw)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(name):
    require(type(name) is str and len(name) <= 4096, 'bounded logical name')
    path = PurePosixPath(name)
    require(str(path) == name and path.parts and not path.is_absolute() and
        all(p not in ('.', '..') for p in path.parts) and
        not any(name == p or name.startswith(p + '/') for p in ['prior', 'physical', 'logical-origin']),
        'canonical nonreserved name required')


def charge(files):
    return sum(len(raw) + len(encode({name: '0' * 64})) for name, raw in files.items())


class Reader:
    """Exact-map byte claims or fixed actual-root reads; one cache and reserve."""
    def __init__(self, supplied, prior, reserve):
        require(supplied is None or type(supplied) is dict and len(supplied) <= 1300, 'finite input map')
        self.supplied, self.cache, self.reserve = supplied, {}, reserve
        self.prior_names = set(prior)
        for mapping in [prior] + ([] if supplied is None else [supplied]):
            for name, raw in mapping.items():
                canonical(name)
                require(type(raw) is bytes and len(raw) <= 8 * 1024**2, 'bounded byte input')
        require(supplied is None or not set(prior).intersection(supplied), 'prior role collision')
        require(charge(prior) + (charge(supplied) if supplied is not None else 0) + reserve <= LIMIT,
                'combined input/output reservation')
        self.cache.update(prior)

    def prime(self, names):
        require(self.supplied is None or set(self.supplied) == names, 'exact current input coverage')
        sizes = {}
        for name in sorted(names):
            canonical(name)
            if self.supplied is None:
                path = ROOT / name
                require(path.resolve() == path and path.is_file() and not path.is_symlink(), 'actual canonical source')
                sizes[name] = path.stat().st_size
            else:
                sizes[name] = len(self.supplied[name])
        require(all(0 <= size <= 8 * 1024**2 for size in sizes.values()), 'early per-file bound')
        total = charge(self.cache) + sum(size + len(encode({name: '0' * 64})) for name, size in sizes.items())
        require(total + self.reserve <= LIMIT, 'early combined input/output bound')
        for name in sorted(names):
            if self.supplied is None:
                with (ROOT / name).open('rb') as stream:
                    raw = stream.read(sizes[name] + 1)
                require(len(raw) == sizes[name], 'actual source changed while reading')
            else:
                raw = self.supplied[name]
            self.cache[name] = raw
        self.used = charge(self.cache)

    def __call__(self, name):
        require(name in self.cache, 'missing fixed current input: ' + str(name))
        return self.cache[name]


def prepare(current_files, prior, snapshot_phase):
    require(G.POLICY_SHA256 != 'PENDING_ROOT_FREEZE', 'constructor policy pending')
    for name, raw in prior.items():
        require(type(raw) is bytes and len(raw) <= 8 * 1024**2 and
                digest(raw) == PRIOR_PINS[name[len(PRIOR):]], 'fixed prior bytes changed')
    reserve = BOUNDS['manifest.json'] + BOUNDS['completion-contract.lock.json']
    if snapshot_phase:
        reserve = BOUNDS['manifest.json'] + BOUNDS['units.json'] + BOUNDS['snapshot.lock.json']
    reader = Reader(current_files, prior, reserve)
    P, _ = G.D.provider()
    old_contract_lock = parse(P, prior[PRIOR + 'contract-lock.json'])
    normative = set(old_contract_lock['sha256']) | ADDITIONS
    require(len(old_contract_lock['sha256']) == 48 and len(normative) == 60, 'fixed normative60 coverage')
    names = normative | {'plan.json', G.D.OWNER}
    old_snapshot = None
    if snapshot_phase:
        old_snapshot = parse(P, prior[PRIOR + 'snapshot-lock.json'])
        require(len(old_snapshot['sha256']) == 1167 and len(old_snapshot['progress_inputs']) == 318,
                'fixed old1167/318 coverage')
        names |= set(old_snapshot['sha256']) - {'remaining-work/units.json'}
        names |= set(PROOFS) | set(FIXTURES) | {'prs/' + uid + '.md' for uid in G.D.IDS}
    reader.prime(names)
    for name in ['derive.py', 'guard.py', 'construct.py']:
        role = 'composition/source_next_prepare/' + name
        path = ROOT / role
        require(path.resolve() == path and path.stat().st_size <= 65536, 'bounded actual constructor source')
        with path.open('rb') as stream:
            raw = stream.read(65537)
        require(raw == reader(role), 'actual source differs from supplied binding')
        normalized = raw
        if name == 'guard.py':
            assignment = rb"(?m)^POLICY_SHA256 = '(?:PENDING_ROOT_FREEZE|[0-9a-f]{64})'$"
            require(len(P.re.findall(assignment, raw)) == 1, 'single actual guard seal')
            normalized = P.re.sub(assignment, b"POLICY_SHA256 = 'PENDING_ROOT_FREEZE'", raw)
        require(name not in SOURCE_PINS or P.sha(normalized) == SOURCE_PINS[name], 'fixed actual source pin')
    contract, plan = parse(P, reader('completion-contract.json')), parse(P, reader('plan.json'))
    G.validate_source_next_splits(contract, plan, reader)
    require(P.sha(reader('execution.json')) == P.BASE_EXECUTION, 'full current execution required')
    if snapshot_phase:
        require(P.digest({n: P.sha(reader(n)) for n in PROOFS}) == PROOFS_SHA, 'fixed proof progress changed')
        require(all(P.sha(reader(n)) == h for n, h in FIXTURES.items()), 'fixed fixture progress changed')
        mutable = normative | {'README.md', 'REMAINING-WORK.md', 'plan.json', 'dispatch.json',
            'PR-INDEX.md', 'completion-contract.lock.json', 'team-dispatch.json', 'remaining-work/units.json'}
        mutable.update('prs/' + u['id'] + '.md' for u in plan['units'])
        require(all(name in mutable or P.sha(reader(name)) == pin
            for name, pin in old_snapshot['sha256'].items()), 'old snapshot input drift')
    return P, reader, normative, old_contract_lock, old_snapshot, contract, plan


class Proposal(dict):
    """Live constructor result; reloaded hash-only JSON is not an emission authority."""


def provenance(reader):
    pins = {n: digest(raw) for n, raw in reader.cache.items()}
    for name, raw in reader.cache.items():
        origin = 'FIXED_HASHED_PRIOR_ARGUMENT' if name in reader.prior_names else (
            'EXPLICIT_SUPPLIED_BYTE_CLAIM' if reader.supplied is not None else 'ACTUAL_CURRENT_ROOT')
        pins['logical-origin/' + name + '/via/' + origin] = digest(raw)
        if origin == 'ACTUAL_CURRENT_ROOT':
            pins['physical/' + name] = digest(raw)
    return pins


def packet(reader, artifacts):
    G.bound(artifacts)
    result = Proposal(status='UNSEALED_PROPOSAL_ONLY', input_sha256=provenance(reader), artifacts=artifacts)
    result.reader, result.artifact_sha = reader, G.digest(artifacts)
    return result


def contract_lock(current_files, prior_lock):
    require(G.POLICY_SHA256 != 'PENDING_ROOT_FREEZE', 'constructor policy pending')
    P, reader, required, old, _, _, _ = prepare(current_files, {PRIOR + 'contract-lock.json': prior_lock}, False)
    result = copy.deepcopy(old)
    result.update(contract_id=G.D.VERSION, sha256={n: P.sha(reader(n)) for n in sorted(required)})
    return packet(reader, {'completion-contract.lock.json': result})


def snapshot(current_files, prior_register, prior_snapshot_lock, prior_contract_lock, snapshot_id, base_main):
    require(G.POLICY_SHA256 != 'PENDING_ROOT_FREEZE', 'constructor policy pending')
    require(type(snapshot_id) is str and snapshot_id.startswith('SYSON-RUST-REMAINING/') and
            len(snapshot_id) <= 128 and type(base_main) is str and len(base_main) == 40 and
            all(c in '0123456789abcdef' for c in base_main), 'explicit snapshot/base identity')
    prior = {PRIOR + 'register.json': prior_register, PRIOR + 'snapshot-lock.json': prior_snapshot_lock,
             PRIOR + 'contract-lock.json': prior_contract_lock}
    P, reader, required, old_contract, old_lock, contract, plan = prepare(current_files, prior, True)
    require(snapshot_id != old_lock['snapshot_id'], 'new snapshot identity required')
    register, state = parse(P, prior_register), parse(P, reader('execution.json'))
    G.bound(register, state)
    expected_lock = copy.deepcopy(old_contract)
    expected_lock.update(contract_id=G.D.VERSION, sha256={n: P.sha(reader(n)) for n in sorted(required)})
    require(parse(P, reader('completion-contract.lock.json')) == expected_lock, 'current normative lock changed')
    rows = []
    for unit in plan['units']:
        row = copy.deepcopy(unit)
        row.update(primary_family=unit['completion_contract']['family'], brief='../prs/' + unit['id'] + '.md',
            brief_sha256=P.sha(reader('prs/' + unit['id'] + '.md')), complete_plan_unit_sha256=P.digest(unit),
            feature_acceptance=state['units'].get(unit['id'], {}).get('feature_acceptance', 'OPEN_NO_ACCEPTANCE_RECORD'))
        rows.append(row)
    register.update(snapshot_id=snapshot_id, contract_id=G.D.VERSION, base_main=base_main, unit_count=797,
        copied_plan_fields=sorted({key for unit in plan['units'] for key in unit}), units=rows,
        integrated_scope_at_snapshot=copy.deepcopy(state['current_integration']))
    progress = set(old_lock['progress_inputs']) | set(PROOFS) | set(FIXTURES) | set(prior)
    required |= set(old_lock['sha256']) | progress | {'prs/' + u['id'] + '.md' for u in plan['units']}
    pins = {n: P.sha(reader(n)) for n in sorted(required - {'remaining-work/units.json'})}
    G.bound(register)
    require(len(encode(register)) <= BOUNDS['units.json'], 'register output profile')
    pins['remaining-work/units.json'] = P.sha(encode(register))
    result = copy.deepcopy(old_lock)
    result.update(snapshot_id=snapshot_id, contract_id=G.D.VERSION, base_main=base_main,
                  sha256=pins, progress_inputs=sorted(progress))
    return packet(reader, {'units.json': register, 'snapshot.lock.json': result})


def emit_fresh(output, unsigned_proposal):
    require(G.POLICY_SHA256 != 'PENDING_ROOT_FREEZE', 'constructor policy pending')
    require(type(unsigned_proposal) is Proposal, 'live constructor result required; no hash-only replay')
    reader = unsigned_proposal.reader
    require(reader.used + reader.reserve <= LIMIT, 'combined input/output reservation')
    G.bound(dict(unsigned_proposal))
    require(set(unsigned_proposal) == {'status', 'input_sha256', 'artifacts'}
            and unsigned_proposal['status'] == 'UNSEALED_PROPOSAL_ONLY', 'fixed unsigned proposal')
    artifacts = unsigned_proposal['artifacts']
    require(G.digest(artifacts) == unsigned_proposal.artifact_sha and
            unsigned_proposal['input_sha256'] == provenance(reader), 'constructor result changed')
    require(set(artifacts) in [{'completion-contract.lock.json'}, {'units.json', 'snapshot.lock.json'}],
            'fixed constructor output set')
    pins = unsigned_proposal['input_sha256']
    require(type(pins) is dict and all(type(v) is str and len(v) == 64 and
            all(c in '0123456789abcdef' for c in v) for v in pins.values()), 'complete input provenance')
    encoded = {n: encode(v) for n, v in artifacts.items()}
    require(all(len(raw) <= BOUNDS[n] for n, raw in encoded.items()), 'fixed output byte profile')
    manifest = {k: copy.deepcopy(v) for k, v in unsigned_proposal.items() if k != 'artifacts'}
    manifest.update(artifacts={n: digest(raw) for n, raw in encoded.items()},
                    logical_input_bytes=reader.used)
    encoded['manifest.json'] = encode(manifest)
    require(len(encoded['manifest.json']) <= BOUNDS['manifest.json'], 'manifest output profile')
    require(reader.used + sum(map(len, encoded.values())) <= LIMIT, 'combined input/output byte limit')
    out = Path(output)
    require(out.is_absolute() and out.parent.resolve() == out.parent and not out.exists() and
            not out.is_symlink() and out.is_relative_to(OUTPUT_ROOT) and
            out.parent.stat().st_uid == os.getuid(), 'fresh owned canonical output required')
    out.mkdir(mode=0o700)
    for name, raw in encoded.items():
        with (out / name).open('xb') as stream:
            stream.write(raw)
    return manifest
