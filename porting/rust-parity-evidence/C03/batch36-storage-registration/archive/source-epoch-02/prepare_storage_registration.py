"""Fixed unsigned storage registration candidates; default import is passive."""
import base64, gzip, hashlib, io, sys, types
from pathlib import Path, PurePosixPath

HELPER_SHA = '61f8deab0d64a62a092b93bdf7c5d82cd3d10f9dfc2358eccff62d0a19e6bf59'
BASE = 'remaining-work/checkpoint-storage-1.0.10/baseline/'
PROVENANCE_SHA = '110f172594d01ef73e2ed22a8b60d80ed5216ffe7843d5974d4910d745df9a4c'
FIXTURES_SHA = '5f56e611e9f962959e3d0096a5b33fd852c1bf8eaf555b4bf4b1bb188e01d045'
BASELINE = {BASE + n for n in ['provenance.json', 'contract-lock.json', 'snapshot-lock.json',
                              'register.json', 'normative-inputs.json.gz']}


def provider(root):
    path = root / 'composition/registration/prepare.py'
    if path.resolve() != path or path.is_symlink() or path.stat().st_size != 22249:
        raise ValueError('fixed helper path/size changed')
    with path.open('rb') as stream:
        raw = stream.read(22250)
    if len(raw) != 22249 or hashlib.sha256(raw).hexdigest() != HELPER_SHA:
        raise ValueError('fixed helper whole-source pin changed')
    module = types.ModuleType('storage_registration_provider')
    module.__file__ = str(path)
    exec(compile(raw, str(path), 'exec'), module.__dict__)
    return module


def reader(P, root):
    actual, left = {}, [P.LIMIT]
    def read(name):
        P.require(type(name) is str and not any(name == prefix or name.startswith(prefix + '/')
                  for prefix in ['physical', 'logical-origin']), 'reserved provenance namespace')
        path = PurePosixPath(name)
        P.require(type(name) is str and str(path) == name and path.parts and not path.is_absolute()
                  and all(p not in ('.', '..') for p in path.parts), 'canonical relative source required')
        if name not in actual:
            source = root / name
            P.require(source.resolve() == source and source.is_file(), 'actual canonical file required')
            frame = len(P.encode({name: '0' * 64}))
            available = left[0] - frame
            P.require(0 <= source.stat().st_size <= available, 'early physical input bound')
            with source.open('rb') as stream:
                raw = stream.read(available + 1)
            P.require(len(raw) <= available, 'physical input changed past bound')
            actual[name], left[0] = raw, available - len(raw)
        return actual[name]
    return read, actual


def captured(P, raw, row):
    blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    P.require(len(raw) == row['bytes'] and P.sha(raw) == row['sha256']
              and blob == row['git_blob'], 'captured row changed')
    return raw


def baseline(P, read, derive=False):
    raw = read(BASE + 'provenance.json')
    P.require(P.sha(raw) == PROVENANCE_SHA, 'fixed captured provenance changed')
    provenance, prior = P.parse(raw), {}
    for name, row in provenance['files'].items():
        raw = captured(P, read(BASE + name), row)
        prior[row['original_path']] = raw
    cap = provenance['normative_capsule']
    raw = read(BASE + cap['path'])
    P.require(len(raw) == cap['compressed_bytes'] and P.sha(raw) == cap['compressed_sha256'],
              'compressed baseline pin changed')
    overrides, origins = {}, {}
    if derive:
        with gzip.GzipFile(fileobj=io.BytesIO(raw)) as stream:
            decoded = stream.read(cap['uncompressed_bytes'] + 1)
        P.require(len(decoded) == cap['uncompressed_bytes'] and P.sha(decoded) == cap['uncompressed_sha256'],
                  'uncompressed baseline pin changed')
        rows = P.parse(decoded)['files']
        lock = P.parse(prior['completion-contract.lock.json'])['sha256']
        P.require(set(rows) == set(lock) and len(rows) == 37, 'exact old normative coverage required')
        for name, row in rows.items():
            value = captured(P, base64.b64decode(row['base64'], validate=True), row)
            P.require(P.sha(value) == lock[name], 'captured old contract pin changed')
            overrides[name], origins[name] = value, BASE + cap['path']
        raw = read('composition/fixtures/baseline-provenance.json')
        P.require(P.sha(raw) == FIXTURES_SHA, 'full baseline fixture provenance changed')
        for name, row in P.parse(raw)['files'].items():
            path = 'composition/fixtures/' + name
            value = captured(P, read(path), row)
            overrides[row['original_path']], origins[row['original_path']] = value, path
        overrides['completion-contract.lock.json'] = prior['completion-contract.lock.json']
        origins['completion-contract.lock.json'] = BASE + 'contract-lock.json'
    return overrides, prior, origins


def main():
    phase, output, *arguments = sys.argv[1:]
    root = Path(__file__).resolve().parents[2]
    P = provider(root)
    P.require(phase in ['derive', 'contract-lock', 'snapshot'] and
              (len(arguments) >= 2 if phase == 'snapshot' else not arguments), 'fixed phase arguments required')
    read, actual = reader(P, root)
    overrides, prior, origins = baseline(P, read, phase == 'derive')
    old_lock = prior['completion-contract.lock.json']
    names = set(P.parse(old_lock)['sha256']) | P.EXTRA
    progress = sorted(BASELINE | set(arguments[2:]))
    if phase == 'derive':
        names.update(['completion-contract.lock.json', 'plan.json', 'execution.json'])
    if phase == 'snapshot':
        P.require(len(arguments[2:]) == len(set(arguments[2:])), 'duplicate progress input')
        names.update(P.parse(prior['remaining-work/snapshot.lock.json'])['sha256'])
        names.update(P.parse(read('completion-contract.lock.json'))['sha256'])
        names.update(progress)
        names.update('prs/' + u['id'] + '.md' for u in P.parse(read('plan.json'))['units'])
    files = {name: read(name) for name in sorted(names)}
    files.update(overrides)
    if phase == 'derive':
        proposal = P.derive(files)
    elif phase == 'contract-lock':
        proposal = P.contract_lock(files, old_lock)
    else:
        proposal = P.snapshot(files, prior['remaining-work/units.json'], prior['remaining-work/snapshot.lock.json'],
                              old_lock, arguments[0], arguments[1], progress)
    proposal['input_sha256'].update({'physical/' + name: P.sha(raw) for name, raw in actual.items()})
    proposal['input_sha256'].update({'logical-origin/' + name + '/via/' + origins[name]: P.sha(files[name])
                                    for name in origins})
    P.emit(output, proposal)


if __name__ == '__main__':
    main()
