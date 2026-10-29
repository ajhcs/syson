"""Fixed data-only library packet; reuse qualified immutable tooling without executing it."""
from pathlib import Path
import gzip
import hashlib
import json
import os
import subprocess

ROOT = Path(__file__).resolve().parent
BUILD = Path('/workspace/build/builderbob/syson-batch22-20261029')
HELPER = Path('/workspace/build/builderbob/syson-batch15-20261029/archive_helpers.py')
HELPER_SHA = 'b0f94b0d26a3e6d4665f78d42dae289eedf14d54f4646a31bd316d1ba26c4106'
COMMIT = 'ca10950d8f4aa0fc6d002d8121b33afad2e0cbba'
PREFIX = 'porting/rust-parity-evidence/U12a/batch27-profile/evidence/'
SEAL = '3cccc83eefc56b9ee22a3232773472d9dffe5bb8ef9486dc4ff1ec6f3bf734ae'
CANDIDATE = '1b2b773141210e64d2c2d64bf8820dcba5f1552e'
DONOR = Path('/workspace/worktrees/builderbob/syson/batch14-history-summary-joint/rust/crates/syson-layout')
ENV = {'PATH': '/usr/bin:/bin', 'HOME': os.environ['HOME'], 'GIT_NO_REPLACE_OBJECTS': '1',
       'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': '/dev/null', 'LANG': 'C.UTF-8'}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def require(ok, message):
    if not ok:
        raise RuntimeError(message)


def blob(commit, path):
    result = subprocess.run(['/usr/bin/git', '-C', str(ROOT), 'show', commit + ':' + path],
                            env=ENV, capture_output=True, timeout=30)
    require(result.returncode == 0, 'immutable lookup failed: ' + path)
    return result.stdout


def write(path, value):
    with path.open('x') as stream:
        stream.write(json.dumps(value, indent=2) + '\n')


def main():
    evidence = ROOT / 'evidence'
    evidence.mkdir(mode=0o700)
    record = {'status': 'FAIL_PRESERVED', 'source_sha256': sha(Path(__file__).read_bytes())}
    try:
        budget = json.loads((ROOT / 'source-budget.json').read_bytes())
        require(772 + sum(len((ROOT / name).read_bytes().splitlines()) for name in budget['packet_sources'])
                == budget['authored_lines'] <= 900, 'authored budget changed')
        raw = HELPER.read_bytes()
        require(sha(raw) == HELPER_SHA, 'trusted archive helper changed')
        helper = {'__name__': 'data_archive', '__file__': str(HELPER)}
        exec(compile(raw, str(HELPER), 'exec'), helper)
        seals_raw = blob(COMMIT, PREFIX + 'SHA256.json')
        require(sha(seals_raw) == SEAL, 'published tooling seal differs')
        seals = json.loads(seals_raw)
        metadata = {name: blob(COMMIT, PREFIX + name) for name in ['archive-index.json', 'input-recovery.json']}
        require(all(sha(raw) == seals[name] for name, raw in metadata.items()), 'tool metadata differs')
        published = {row['raw_sha256']: row for row in json.loads(metadata['archive-index.json'])}
        closure = json.loads(metadata['input-recovery.json'])['rows']
        require(len(closure) == 1086, 'complete published phase closure differs')
        entries = [('assembly-helper', HELPER), ('base-proof', Path('/workspace/build/builderbob/syson-batch27-20261029/u12a-current-base-proof.json'))]
        entries.append(('source-budget.json', ROOT / 'source-budget.json'))
        entries.extend(('active-source/' + name, ROOT / name) for name in budget['packet_sources'])
        entries.extend(('preformat/' + str(path.relative_to(BUILD / 'u12a-before-format')), path)
                       for path in sorted((BUILD / 'u12a-before-format').rglob('*')) if path.is_file())
        for name, expected in [('src/containers.rs', '9d86a510de50586ffaacabb14ecb54415493d6888d9457ec87783b3399d1bec8'),
                               ('tests/containers.rs', 'e486c5c2e96b07d786f772ee322c83762db24ba3ef70bd6bc81c2fbaff393aea')]:
            require(sha((DONOR / name).read_bytes()) == expected, 'full donor changed')
            entries.append(('deferred-U12b-donor/' + name, DONOR / name))
        recovered, checked = [], set()
        for receipt_name in ['u12a-checks-01', 'u12a-profile-resource-01']:
            path = BUILD / receipt_name / 'receipt.json'
            receipt = json.loads(path.read_bytes())
            require(receipt['inputs_before'] == receipt['inputs_after'], 'qualification input drift')
            entries.append((receipt_name + '/receipt.json', path))
            for phase in ['inputs_before', 'inputs_after']:
                for name, digest in receipt[phase].items():
                    require(sha(Path(name).read_bytes()) == digest, 'current input drift: ' + name)
                    if '/rust/crates/syson-layout/' in name:
                        relative = 'rust/crates/syson-layout/' + name.split('/rust/crates/syson-layout/')[1]
                        require(sha(blob(CANDIDATE, relative)) == digest, 'candidate source differs')
                    item = published.get(digest)
                    recovery = ({'kind': 'immutable_git_blob', 'commit': COMMIT, 'path': PREFIX + item['stored'],
                                 'encoding': item['encoding'], 'stored_sha256': item['stored_sha256'], 'raw_bytes': item['raw_bytes']}
                                if item else next(row['recovery'] for row in closure if row['path'] == name and row['sha256'] == digest))
                    locator = (recovery['commit'], recovery['path'])
                    if locator not in checked:
                        stored = blob(*locator)
                        raw = gzip.decompress(stored) if recovery['encoding'] == 'gzip' else stored
                        require(sha(stored) == recovery['stored_sha256'] and sha(raw) == digest
                                and len(raw) == recovery['raw_bytes'], 'immutable recovery differs')
                        checked.add(locator)
                    recovered.append({'receipt': receipt_name, 'phase': phase, 'path': name, 'sha256': digest, 'recovery': recovery})
        require(len(recovered) == 52, 'selected recovery count differs')
        record['inputs_before'] = {str(path): sha(path.read_bytes()) for _, path in entries}
        rows = helper['archive'](evidence, entries)
        write(evidence / 'input-recovery.json', {'schema': 1, 'rows': recovered})
        write(evidence / 'dependencies.json', {'tool_pr': 190, 'commit': COMMIT, 'module_tree': '44382509b7952bc2219f909b1bee68eac91c8595',
              'prefix': PREFIX, 'seal_sha256': SEAL, 'metadata_sha256': {name: sha(raw) for name, raw in metadata.items()},
              'complete_phase_rows': 1086, 'candidate': CANDIDATE, 'scope': '37 cases = 26 earlier + 11 forest; resource 90/10/5; placement/native/UI/capacity OPEN'})
        record['inputs_after'] = {path: sha(Path(path).read_bytes()) for path in record['inputs_before']}
        require(record['inputs_before'] == record['inputs_after'], 'assembly input drift')
        record.update(status='PASS_DATA_ASSEMBLY_ONLY;INDEPENDENT_REVIEW_PENDING', archive_entries=len(rows), recovery_rows=len(recovered))
        write(evidence / 'assembly-bindings.json', record)
        record.update(seals=len(helper['seal'](evidence)), seal_sha256=sha((evidence / 'SHA256.json').read_bytes()))
    except BaseException as error:
        record.update(status='FAIL_PRESERVED', error=repr(error))
        raise
    finally:
        write(ROOT / 'assembly-receipt.json', record)


if __name__ == '__main__':
    main()
