"""Inactive finite historical capture producer; every output remains unsigned."""
import hashlib
import os
import sys
import zlib
from pathlib import Path
from . import derive as D
from . import guard as G
from . import construct as C

ROOT = Path(__file__).resolve().parents[2]
OUTPUT_ROOT = Path('/workspace/build/builderbob')
TAILS = {'remaining-work/units.json', 'team-dispatch.json'}
PRIOR = C.CATALOGUE.rsplit('/', 1)[0] + '/'
MEMBER_MAX = 512 * 1024
RESERVE = 2 * C.MIB + 512 * 1024 + 256 * 1024


def read(root, name, maximum):
    C.cold().canonical(name)
    path = root / name
    D.require(root.resolve() == root and path.resolve() == path and not path.is_symlink()
              and path.is_file() and path.stat().st_size <= maximum, 'capture physical admission')
    with path.open('rb') as stream:
        raw = stream.read(maximum + 1)
    D.require(len(raw) <= maximum, 'capture physical source grew')
    return raw


def validate(files):
    D.require(type(files) is dict and len(files) == 66, 'exact66 historical roles')
    old = {name: raw for name, raw in files.items() if name not in TAILS}
    _, plan = D.baseline(old)
    _, _, P, _ = D.provider()
    snapshot = P.parse(old['remaining-work/snapshot.lock.json'])
    D.require(all(D.sha(files[name]) == snapshot['sha256'][name] for name in TAILS), 'historical tails changed')
    state = P.parse(old['execution.json'])
    register = P.parse(files['remaining-work/units.json'])
    D.require(register['unit_count'] == len(register['units']) == 797 and all(a == b for a, b in
              zip(register['units'], C.projected(plan, state, snapshot['sha256']))), 'whole797 historical register')
    D.require(register['snapshot_id'] == snapshot['snapshot_id'] and
              register['integrated_scope_at_snapshot'] == state['current_integration'] and
              set(register['copied_plan_fields']) == {key for unit in plan['units'] for key in unit},
              'historical register metadata')
    return files


def originals(root):
    raw = read(root, 'completion-contract.lock.json', 256 * 1024)
    D.require(D.sha(raw) == D.BASE_LOCK, 'exact .11 historical lock required')
    lock = C.parse(raw, 256 * 1024)
    names = set(lock['sha256']) | D.EXTRA | TAILS
    D.require(len(names) == 66 and C.CAPTURED <= names, 'finite original capture set')
    sizes = {}
    for name in sorted(names):
        C.cold().canonical(name)
        path = root / name
        D.require(path.resolve() == path and not path.is_symlink() and path.is_file(), 'canonical capture origin')
        sizes[name] = path.stat().st_size
    D.require(all(0 <= size <= (6 * C.MIB if name == 'remaining-work/units.json' else
              2 * C.MIB if name == 'team-dispatch.json' else 8 * C.MIB) for name, size in sizes.items()),
              'early capture role ceiling')
    charge = sum(size + 128 + 6 * len(name) for name, size in sizes.items())
    D.require(charge <= 24 * C.MIB and sum(size + 128 + 6 * len(name)
              for name, size in sizes.items() if name not in TAILS) <= 16 * C.MIB,
              'early historical phase ceiling')
    files = {name: read(root, name, size) for name, size in sizes.items()}
    D.require(all(len(files[name]) == size for name, size in sizes.items()), 'capture source size drift')
    return validate(files)


def historical(root):
    D.require(root == C.ROOT, 'historical reader root binding')
    lock = C.parse(read(root, 'completion-contract.lock.json', 256 * 1024), 256 * 1024)
    D.require(type(lock) is dict and type(lock.get('contract_id')) is str, 'historical lock object')
    if lock.get('contract_id') == 'SYSON-RUST-COMPLETION/1.0.11':
        return originals(root)
    D.require(lock.get('contract_id') == D.VERSION, 'unknown historical reader version')
    _, files, tails, _ = C.prior()
    D.require(not set(files).intersection(tails), 'historical role collision')
    return validate(dict(files, **tails))


def member(raw):
    D.require(type(raw) is bytes and len(raw) <= 8 * C.MIB, 'bounded raw member input')
    encoder = zlib.compressobj(9, zlib.DEFLATED, 16 + zlib.MAX_WBITS)
    result = bytearray()
    for at in range(0, len(raw), 65536):
        part = encoder.compress(raw[at:at + 65536])
        D.require(len(result) + len(part) <= MEMBER_MAX, 'bounded gzip member output')
        result.extend(part)
    part = encoder.flush()
    D.require(len(result) + len(part) <= MEMBER_MAX, 'bounded gzip member output')
    result.extend(part)
    return bytes(result)


def produce(output, proof_inputs):
    root, output_root = ROOT, OUTPUT_ROOT
    D.require(type(proof_inputs) is list and len(proof_inputs) <= 64, 'finite explicit proof list')
    for name in proof_inputs:
        D.require(type(name) is str and len(name) <= 4096 and name.startswith(PRIOR.rsplit('prior/', 1)[0] + 'proof/'),
                  'fixed proof namespace')
        C.cold().canonical(name)
    D.require(len(set(proof_inputs)) == len(proof_inputs), 'duplicate proof origin')
    proof_inputs = tuple(proof_inputs)
    files = originals(root)
    for name in proof_inputs:
        path = root / name
        D.require(path.resolve() == path and path.is_file() and not path.is_symlink(), 'canonical proof origin')
    proof_sizes = {name: (root / name).stat().st_size for name in proof_inputs}
    used = sum(len(raw) + 128 + 6 * len(name) for name, raw in files.items())
    used += sum(size + 128 + 6 * len(name) for name, size in proof_sizes.items())
    D.require(all(0 <= size <= 8 * C.MIB for size in proof_sizes.values()) and
              used + RESERVE + C.MIB <= D.LIMIT, 'early complete capture proof budget')
    proofs = {name: read(root, name, size) for name, size in proof_sizes.items()}
    D.require(all(len(proofs[name]) == size for name, size in proof_sizes.items()), 'proof source size drift')
    programs = {}
    for name in ['derive.py', 'guard.py', 'construct.py', 'capture.py']:
        programs[name] = read(Path(__file__).resolve().parents[2], 'composition/identity_partition_prepare/' + name, 65536)
    _, _, _, actual = D.provider()
    actual_root = Path(__file__).resolve().parents[2]
    for name in ['composition/storage_split_guard.py', 'tools/preparation_splits.py']:
        actual[str(actual_root / name)] = read(actual_root, name, 65536)
    program_pins = {str(Path(__file__).parent / name): D.sha(raw) for name, raw in programs.items()}
    program_pins.update({name: D.sha(raw) for name, raw in actual.items()})
    D.require(sum(len(raw) + 128 + 6 * len(name) for name, raw in programs.items()) +
              sum(len(raw) + 128 + 6 * len(name) for name, raw in actual.items()) <= C.MIB,
              'finite actual program input budget')
    catalogue, encoded = {'schema': 1, 'files': {}, 'proof_inputs': sorted(proof_inputs)}, {}
    for name, raw in sorted(files.items()):
        source = PRIOR + 'captures/' + name + '.gz' if name in C.CAPTURED else name
        stored = member(raw) if name in C.CAPTURED else raw
        if name in C.CAPTURED:
            encoded[source] = stored
        catalogue['files'][name] = {'source': source, 'encoding': 'gzip' if name in C.CAPTURED else 'raw',
            'bytes': len(raw), 'sha256': D.sha(raw), 'stored_bytes': len(stored), 'stored_sha256': D.sha(stored),
            'git_blob_sha1': hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()}
    encoded[C.CATALOGUE] = bytes(C.compact(catalogue, 128 * 1024))
    D.require(sum(map(len, encoded.values())) <= 2 * C.MIB, 'complete stored capture ceiling')
    pins = {name: D.sha(raw) for name, raw in files.items()}
    pins.update({name: D.sha(raw) for name, raw in proofs.items()})
    manifest = {'status': 'UNSEALED_PROPOSAL_ONLY', 'input_sha256': pins, 'actual_program_sha256': program_pins,
                'logical_input_ceiling': used + C.MIB, 'output_sha256': {n: D.sha(v) for n, v in encoded.items()},
                'runtime': {'python': sys.version, 'zlib': zlib.ZLIB_RUNTIME_VERSION}, 'raw_alias_count': 52}
    encoded['manifest.json'] = bytes(C.compact(manifest, 512 * 1024))
    outputs = tuple(sorted(encoded.items(), key=lambda item: (item[0] == 'manifest.json', item[0])))
    D.require(used + C.MIB + sum(len(raw) + 128 + 6 * len(name) for name, raw in outputs) <= D.LIMIT,
              'complete capture input output framing budget')
    D.require(ROOT == root and OUTPUT_ROOT == output_root, 'capture issuance root changed')
    for name, pin in pins.items():
        D.require(D.sha(read(root, name, len(files[name]) if name in files else proof_sizes[name])) == pin,
                  'capture physical source changed before output')
    for name, pin in program_pins.items():
        path = Path(name)
        D.require(D.sha(read(path.parent, path.name, 65536)) == pin, 'actual program changed before output')
    out = Path(output)
    D.require(out.is_absolute() and out.parent.resolve() == out.parent and not out.exists() and
              not out.is_symlink() and out.is_relative_to(output_root) and out.parent.stat().st_uid == os.getuid(),
              'fresh owned capture output')
    out.mkdir(mode=0o700)
    for name, raw in outputs:
        path = out / name
        path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        with path.open('xb') as stream:
            stream.write(raw)
        D.require(read(out, name, len(raw)) == raw, 'fresh capture readback changed')
    return manifest
