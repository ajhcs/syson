"""Inactive physical-root unsigned constructors; no catalogue is trusted by default."""
import copy
import hashlib
import json
import os
import sys
import types
import zlib
import weakref
from pathlib import Path
from . import derive as D
from . import guard as G

ROOT = Path(__file__).resolve().parents[2]
OUTPUT_ROOT = Path('/workspace/build/builderbob')
CATALOGUE = 'remaining-work/checkpoint-identity-partition-1.0.12/prior/catalogue.json'
CATALOGUE_SHA256 = 'PENDING_ROOT_FREEZE'
MIB = 1024**2
BOUNDS = {'completion-contract.lock.json': 256 * 1024, 'units.json': 5 * MIB,
          'snapshot.lock.json': 256 * 1024, 'manifest.json': 512 * 1024}
ADDITIONS = {D.FRAGMENT, D.AMENDMENT, 'tools/identity_partition_splits.py'}
ADDITIONS.update('composition/identity_partition_prepare/' + name for name in
    ['derive.py', 'guard.py', 'test_derive.py', 'test_guard.py', 'README.md', 'GUARD.md',
     'construct.py', 'test_construct.py', 'CONSTRUCT.md'])
FIXTURES = {'composition/identity_partition_prepare/fixtures/lane.json',
            'composition/identity_partition_prepare/fixtures/amendment.md'}
CAPTURED = {'completion-contract.json', 'acceptance-packet.schema.json', 'COMPLETION-CONTRACT.md',
            'tools/build_plan.py', 'tools/completion_contract.py', 'tools/source_next_splits.py',
            'plan.json', 'team-dispatch.json', 'completion-contract.lock.json',
            'remaining-work/snapshot.lock.json', 'remaining-work/units.json'}
_COLD = None


def cold():
    global _COLD
    Q, _, _, sources = D.provider()
    path = Path(Q.__file__).parent / 'construct.py'
    name = Q.__package__ + '.construct'
    if _COLD is None:
        D.require(name not in sys.modules, 'old constructor namespace collision')
        module = types.ModuleType(name)
        module.__file__, module.__package__ = str(path), Q.__package__
        sys.modules[name] = module
        exec(compile(sources[str(path)], str(path), 'exec'), module.__dict__)
        _COLD = module
    D.require(sys.modules.get(name) is _COLD and _COLD.ROOT == ROOT, 'actual constructor root changed')
    return _COLD


def physical(name, maximum, seen):
    cold().canonical(name)
    path = ROOT / name
    D.require(path.resolve() == path and path.is_file() and not path.is_symlink()
              and path.stat().st_size <= maximum, 'early canonical physical bound')
    with path.open('rb') as stream:
        raw = stream.read(maximum + 1)
    D.require(len(raw) <= maximum, 'physical source changed past bound')
    seen[name] = D.sha(raw)
    return raw


def parse(raw, maximum):
    def pairs(items):
        result = {}
        for key, value in items:
            D.require(key not in result, 'duplicate JSON key')
            result[key] = value
        return result
    value = json.loads(raw, object_pairs_hook=pairs)
    used = 0
    encoder = json.JSONEncoder(ensure_ascii=False, allow_nan=False, sort_keys=True, separators=(',', ':'))
    for piece in encoder.iterencode(value):
        used += len(piece.encode())
        D.require(used <= maximum, 'streamed JSON representation bound')
    return value


def prior():
    D.require(CATALOGUE_SHA256 != 'PENDING_ROOT_FREEZE' and
              G.POLICY_SHA256 != 'PENDING_ROOT_FREEZE', 'constructor catalogue or policy pending')
    seen = {}
    raw = physical(CATALOGUE, 128 * 1024, seen)
    D.require(D.sha(raw) == CATALOGUE_SHA256, 'fixed catalogue changed')
    catalogue = parse(raw, 128 * 1024)
    D.require(set(catalogue) == {'schema', 'files', 'proof_inputs'} and
              type(catalogue['schema']) is int and catalogue['schema'] == 1
              and type(catalogue['files']) is dict and len(catalogue['files']) == 66, 'fixed catalogue roles')
    proof = catalogue['proof_inputs']
    D.require(type(proof) is list and len(proof) <= 64, 'finite proof roles')
    for name in proof:
        cold().canonical(name)
        D.require(name.startswith('remaining-work/checkpoint-identity-partition-1.0.12/proof/'),
                  'explicit compact proof role')
    D.require(len(proof) == len(set(proof)), 'unique proof roles')
    used, stored_total = 0, 0
    for name, row in catalogue['files'].items():
        cold().canonical(name)
        D.require(type(row) is dict and set(row) == {'source', 'encoding', 'bytes', 'sha256',
                  'stored_bytes', 'stored_sha256', 'git_blob_sha1'}, 'exact source descriptor')
        cold().canonical(row['source'])
        D.require(all(type(row[k]) is int and 0 <= row[k] <= 8 * MIB
                      for k in ['bytes', 'stored_bytes']), 'source byte claims')
        D.require(all(type(row[k]) is str and len(row[k]) == size for k, size in
                      [('sha256', 64), ('stored_sha256', 64), ('git_blob_sha1', 40)]), 'digest claims')
        D.require(row['encoding'] == 'raw' and row['source'] == name or
                  row['encoding'] == 'gzip' and row['source'].startswith(CATALOGUE.rsplit('/', 1)[0] + '/')
                  and row['source'].endswith('.gz') and row['stored_bytes'] <= 512 * 1024, 'source origin')
        D.require(row['encoding'] != 'raw' or row['bytes'] == row['stored_bytes'] and
                  row['sha256'] == row['stored_sha256'], 'identical raw alias descriptor')
        cap = {'remaining-work/units.json': 6 * MIB, 'team-dispatch.json': 2 * MIB}.get(name, 8 * MIB)
        D.require(row['bytes'] <= cap, 'early prior role ceiling')
        used += row['bytes'] + 128 + 6 * len(name)
        stored_total += row['stored_bytes'] if row['encoding'] == 'gzip' else 0
    D.require(CAPTURED <= set(catalogue['files']) and all(
        row['encoding'] == ('gzip' if name in CAPTURED else 'raw')
        for name, row in catalogue['files'].items()), 'exact eleven capture origins')
    D.require(sum(row['bytes'] + 128 + 6 * len(name)
        for name, row in catalogue['files'].items() if name not in
        {'remaining-work/units.json', 'team-dispatch.json'}) <= 16 * MIB, 'early old64 ceiling')
    D.require(used <= 24 * MIB and stored_total + len(raw) <= 2 * MIB, 'early prior phase bound')
    files, tails = {}, {}
    for name, row in catalogue['files'].items():
        stored = physical(row['source'], row['stored_bytes'], seen)
        D.require(len(stored) == row['stored_bytes'] and D.sha(stored) == row['stored_sha256'],
                  'stored prior changed')
        value = stored
        if row['encoding'] == 'gzip':
            decoder = zlib.decompressobj(16 + zlib.MAX_WBITS)
            value = decoder.decompress(stored, row['bytes'] + 1)
            D.require(decoder.eof and not decoder.unused_data and not decoder.unconsumed_tail,
                      'single bounded gzip member required')
        D.require(len(value) == row['bytes'] and D.sha(value) == row['sha256'] and
                  hashlib.sha1(
                      b'blob ' + str(len(value)).encode() + b'\0' + value).hexdigest() == row['git_blob_sha1'],
                  'whole prior raw bytes changed')
        (tails if name in ['remaining-work/units.json', 'team-dispatch.json'] else files)[name] = value
    D.require(set(tails) == {'remaining-work/units.json', 'team-dispatch.json'}, 'exact prior tails')
    D.preflight(files)
    D.require(all(name in files and D.sha(files[name]) == pin for name, pin in
              [('completion-contract.lock.json', D.BASE_LOCK), ('plan.json', D.BASE_RAW_PLAN),
               ('remaining-work/snapshot.lock.json', D.BASE_SNAPSHOT)]), 'fixed prior roots')
    D.require(cold().charge(files) <= 16 * MIB and len(tails['remaining-work/units.json']) <= 6 * MIB
              and len(tails['team-dispatch.json']) <= 2 * MIB, 'separate prior role ceilings')
    return catalogue, files, tails, seen


def projected(plan, state, pins):
    for unit in plan['units']:
        row = copy.deepcopy(unit)
        row.update(primary_family=unit['completion_contract']['family'], brief='../prs/' + unit['id'] + '.md',
            brief_sha256=pins['prs/' + unit['id'] + '.md'], complete_plan_unit_sha256=fingerprint(unit, True),
            feature_acceptance=state['units'].get(unit['id'], {}).get('feature_acceptance', 'OPEN_NO_ACCEPTANCE_RECORD'))
        yield row


def contract_only(contract, old, lane, amendment, generator):
    D.require(G.POLICY_SHA256 != 'PENDING_ROOT_FREEZE', 'partition policy pending')
    D.require(type(contract) is dict, 'exact builtin contract dictionary')
    _, S, _, _ = D.provider()
    S.bound(contract)
    previous, plan = D.baseline(old)
    expected = D.build(previous, plan, lane, amendment, generator, old.__getitem__)
    D.require(D.digest(expected['policy.json']) == G.POLICY_SHA256 and
              D.digest(contract) == D.digest(expected['completion-contract.json']), 'whole current contract')
    return previous, plan, expected['acceptance-packet.schema.json']


def early(with_plan):
    catalogue, old, tails, seen = prior()
    _, _, P, _ = D.provider()
    old_lock = P.parse(old['completion-contract.lock.json'])
    names = set(old_lock['sha256']) | ADDITIONS
    D.require(len(names) == 72 and sum((ROOT / n).stat().st_size + 128 + 6 * len(n)
              for n in names) <= 8 * MIB, 'early actual normative phase bound')
    reader = cold().Reader(None, {}, 0)
    reader.prime(names)
    seen.update({name: D.sha(raw) for name, raw in reader.cache.items()})
    contract = parse(reader('completion-contract.json'), MIB)
    D.require(reader('execution.json') == old['execution.json'], 'current history source drift')
    args = tuple(reader(name) for name in [D.FRAGMENT, D.AMENDMENT, 'tools/build_plan.py'])
    _, previous, schema = contract_only(contract, old, *args)
    D.require(parse(reader('acceptance-packet.schema.json'), 256 * 1024) == schema, 'whole current schema')
    old_snapshot, state = P.parse(old['remaining-work/snapshot.lock.json']), P.parse(old['execution.json'])
    D.require(D.sha(tails['remaining-work/units.json']) == old_snapshot['sha256']['remaining-work/units.json']
              and D.sha(tails['team-dispatch.json']) == old_snapshot['sha256']['team-dispatch.json'], 'old tails pins')
    register = P.parse(tails['remaining-work/units.json'])
    D.require(register['unit_count'] == 797 and len(register['units']) == 797 and all(
        a == b for a, b in zip(register['units'], projected(previous, state, old_snapshot['sha256']))),
        'complete797 prior register')
    D.require(register['integrated_scope_at_snapshot'] == state['current_integration'] and
              register['snapshot_id'] == old_snapshot['snapshot_id'] and
              set(register['copied_plan_fields']) == {k for u in previous['units'] for k in u}, 'prior metadata')
    header = {key: value for key, value in register.items() if key != 'units'}
    plan = None
    if with_plan:
        plan = parse(physical('plan.json', 8 * MIB, seen), 8 * MIB)
        G.checked(contract, plan, old.__getitem__, *args)
    progress = set(old_snapshot['progress_inputs']) | set(catalogue['proof_inputs']) | FIXTURES | {CATALOGUE, D.OWNER}
    progress.update(row['source'] for row in catalogue['files'].values() if row['encoding'] == 'gzip')
    lock = copy.deepcopy(old_lock)
    lock.update(contract_id=D.VERSION, sha256={name: D.sha(reader(name)) for name in sorted(names)})
    old.clear()
    tails.clear()
    reader.cache.clear()
    return lock, old_snapshot, state, header, plan, progress, seen


class Proposal(dict):
    """Mutable diagnostics are checked against closure-owned immutable emission authority."""


def pieces(value, canonical=False):
    encoder = json.JSONEncoder(ensure_ascii=False, allow_nan=False, sort_keys=canonical,
                               separators=(',', ':'))
    for piece in encoder.iterencode(value):
        D.require(len(piece) <= 16384, 'bounded output token')
        yield piece.encode()
    if not canonical:
        yield b'\n'


def fingerprint(value, canonical=False):
    result = hashlib.sha256()
    for raw in pieces(value, canonical):
        result.update(raw)
    return result.hexdigest()


def compact(value, maximum):
    result = bytearray()
    for raw in pieces(value):
        D.require(len(result) + len(raw) <= maximum, 'bounded output stream')
        result.extend(raw)
    return result


def _constructors():
    pending = {}
    def packet(reader, artifacts, historical):
        D.require(len(pending) < 8, 'bounded live proposal registry')
        _, S, _, _ = D.provider()
        S.bound(artifacts)
        encoded = {name: compact(value, BOUNDS[name]) for name, value in artifacts.items()}
        D.require(all(len(raw) <= BOUNDS[name] for name, raw in encoded.items()), 'artifact output ceiling')
        manifest = {'status': 'UNSEALED_PROPOSAL_ONLY', 'input_sha256': cold().provenance(reader),
                    'historical_progress': historical, 'logical_input_bytes': reader.used,
                    'artifacts': {name: D.sha(raw) for name, raw in encoded.items()}}
        encoded['manifest.json'] = compact(manifest, BOUNDS['manifest.json'])
        D.require(len(encoded['manifest.json']) <= BOUNDS['manifest.json'] and
                  reader.used + sum(map(len, encoded.values())) <= D.LIMIT, 'complete input output bound')
        result = Proposal(status='UNSEALED_PROPOSAL_ONLY', artifacts=artifacts, manifest=manifest)
        pins = tuple(sorted((name, D.sha(raw)) for name, raw in reader.cache.items()))
        bindings = CATALOGUE_SHA256, G.POLICY_SHA256, str(ROOT), str(OUTPUT_ROOT)
        seal = fingerprint(dict(result), True)
        result.reader, result.encoded, result.seal = reader, encoded, seal
        result.inputs = dict(pins)
        result.bindings = bindings
        key = id(result)
        pending[key] = (weakref.ref(result, lambda reference: pending.pop(key, None)),
                        pins, reader.used, reader.reserve, bindings, seal, tuple(encoded))
        reader.cache.clear()
        return result


    def contract_lock():
        lock, _, _, _, _, _, seen = early(False)
        reserve = BOUNDS['completion-contract.lock.json'] + BOUNDS['manifest.json'] + 256 * 1024
        reader = cold().Reader(None, {}, reserve)
        reader.prime(set(seen))
        D.require(all(D.sha(reader(name)) == pin for name, pin in seen.items()), 'phase source drift')
        return packet(reader, {'completion-contract.lock.json': lock}, {})


    def snapshot(snapshot_id, base_main):
        D.require(type(snapshot_id) is str and snapshot_id.startswith('SYSON-RUST-REMAINING/') and
                  len(snapshot_id) <= 128 and type(base_main) is str and len(base_main) == 40 and
                  all(c in '0123456789abcdef' for c in base_main), 'explicit snapshot and source identity')
        lock, old, state, header, plan, progress, seen = early(True)
        D.require(snapshot_id != old['snapshot_id'], 'fresh snapshot identity')
        names = set(old['sha256']) | set(lock['sha256']) | progress | set(seen)
        names.update('prs/' + unit['id'] + '.md' for unit in plan['units'])
        names.discard('remaining-work/units.json')
        D.require(len(names) <= 1300, 'finite physical snapshot roles')
        reserve = BOUNDS['units.json'] + BOUNDS['snapshot.lock.json'] + BOUNDS['manifest.json'] + 256 * 1024
        reader = cold().Reader(None, {}, reserve)
        reader.prime(names)
        D.require(all(D.sha(reader(name)) == pin for name, pin in seen.items()), 'phase source drift')
        D.require(reader('completion-contract.lock.json') == compact(lock, BOUNDS['completion-contract.lock.json']),
                  'actual normative lock drift')
        observed = {name: old['sha256'][name] if name == 'team-dispatch.json' else D.sha(reader(name))
                    for name in old['progress_inputs']}
        D.require(observed == {n: old['sha256'][n] for n in old['progress_inputs']} and
                  fingerprint(observed, True) == D.PROGRESS, 'all342 historical progress bindings')
        pins = {name: D.sha(reader(name)) for name in sorted(names)}
        header.update(snapshot_id=snapshot_id, contract_id=D.VERSION, base_main=base_main, unit_count=799,
            copied_plan_fields=sorted({key for unit in plan['units'] for key in unit}),
            units=list(projected(plan, state, pins)),
            integrated_scope_at_snapshot=copy.deepcopy(state['current_integration']))
        pins['remaining-work/units.json'] = fingerprint(header)
        result = copy.deepcopy(old)
        history = {'prior_snapshot_sha256': D.BASE_SNAPSHOT, 'bindings_sha256': D.PROGRESS,
                   'catalogue': CATALOGUE, 'roles': 342}
        result.update(snapshot_id=snapshot_id, contract_id=D.VERSION, base_main=base_main,
                      sha256=pins, progress_inputs=sorted(progress), historical_progress=history)
        return packet(reader, {'units.json': header, 'snapshot.lock.json': result}, history)


    def emit_fresh(output, proposal):
        D.require(CATALOGUE_SHA256 != 'PENDING_ROOT_FREEZE' and
                  G.POLICY_SHA256 != 'PENDING_ROOT_FREEZE', 'emission binding pending')
        state = pending.get(id(proposal))
        D.require(type(proposal) is Proposal and state is not None and state[0]() is proposal,
                  'registered live proposal required')
        D.require(len(proposal) == 3 and set(proposal) == {'status', 'artifacts', 'manifest'}, 'fixed proposal body')
        _, pins, used, reserve, bindings, seal, names = state
        D.require(bindings == (CATALOGUE_SHA256, G.POLICY_SHA256, str(ROOT), str(OUTPUT_ROOT)),
                  'issuance bindings changed')
        D.require(type(proposal.inputs) is dict and type(proposal.reader) is cold().Reader and
                  type(proposal.bindings) is tuple and type(proposal.seal) is str, 'immutable diagnostic types')
        _, S, _, _ = D.provider()
        S.bound(dict(proposal), proposal.inputs, [proposal.reader.used, proposal.reader.reserve],
                list(proposal.bindings))
        D.require(proposal.inputs == dict(pins) and
                  (proposal.reader.used, proposal.reader.reserve) == (used, reserve) and
                  proposal.bindings == bindings and proposal.seal == seal, 'immutable emission metadata changed')
        encoded = proposal.encoded
        D.require(type(encoded) is dict and len(encoded) == len(names), 'fixed encoded container')
        D.require(all(type(name) is str and name in names for name in encoded), 'fixed encoded names')
        D.require(all(type(encoded[name]) in (bytes, bytearray) and len(encoded[name]) <= BOUNDS[name]
                      for name in names), 'encoded body ceiling')
        D.require(used + reserve <= D.LIMIT and sum(len(encoded[name]) for name in names) <= reserve - 256 * 1024,
                  'captured input output reservation changed')
        outputs = tuple((name, bytes(encoded[name])) for name in names)
        D.require(fingerprint(dict(proposal), True) == seal, 'unchanged live proposal body required')
        D.require(all(D.sha(raw) == fingerprint(proposal['manifest'] if name == 'manifest.json'
                      else proposal['artifacts'][name]) for name, raw in outputs), 'live output bytes changed')
        for name, pin in pins:
            D.require(D.sha(physical(name, 8 * MIB, {})) == pin, 'physical source changed before emission')
        out = Path(output)
        D.require(out.is_absolute() and out.parent.resolve() == out.parent and not out.exists()
                  and not out.is_symlink() and out.is_relative_to(OUTPUT_ROOT)
                  and out.parent.stat().st_uid == os.getuid(), 'fresh owned Samsung output')
        out.mkdir(mode=0o700)
        for name, raw in outputs:
            with (out / name).open('xb') as stream:
                stream.write(raw)
        pending.pop(id(proposal))
        return copy.deepcopy(proposal['manifest'])

    return contract_lock, snapshot, emit_fresh


contract_lock, snapshot, emit_fresh = _constructors()
