"""Fixed data-only recovery and sealing of the actual56-case qualification."""
from pathlib import Path
import gzip
import hashlib
import json
import os
import subprocess

ROOT = Path(__file__).resolve().parent
BUILD = Path('/workspace/build/builderbob/syson-batch28-20261029')
HELPER = Path('/workspace/build/builderbob/syson-batch15-20261029/archive_helpers.py')
COMMIT = 'ca10950d8f4aa0fc6d002d8121b33afad2e0cbba'
PREFIX = 'porting/rust-parity-evidence/U12a/batch27-profile/evidence/'
SEAL = '3cccc83eefc56b9ee22a3232773472d9dffe5bb8ef9486dc4ff1ec6f3bf734ae'
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


def write(path, data):
    with path.open('x') as stream:
        stream.write(json.dumps(data, indent=2) + '\n')


def main():
    evidence = ROOT / 'evidence'
    evidence.mkdir(mode=0o700)
    record = {'status': 'FAIL_PRESERVED', 'source_sha256': sha(Path(__file__).read_bytes())}
    try:
        budget = json.loads((ROOT / 'source-budget.json').read_bytes())
        require(sum(len((ROOT / row['path']).read_bytes().splitlines()) for row in budget['rows'])
                == budget['authored_lines'] <= 900 and budget['production_python_lines'] <= 400, 'source budget drift')
        require(all(sha((ROOT / row['path']).read_bytes()) == row['sha256'] for row in budget['rows']), 'source hashes differ')
        raw = HELPER.read_bytes()
        require(sha(raw) == 'b0f94b0d26a3e6d4665f78d42dae289eedf14d54f4646a31bd316d1ba26c4106', 'archive helper differs')
        archive = {'__file__': str(HELPER), '__name__': 'fixed_archive'}
        exec(compile(raw, str(HELPER), 'exec'), archive)
        raw = blob(COMMIT, PREFIX + 'SHA256.json')
        require(sha(raw) == SEAL, 'published tooling seal differs')
        seals = json.loads(raw)
        metadata = {name: blob(COMMIT, PREFIX + name) for name in ['archive-index.json', 'input-recovery.json']}
        require(all(sha(raw) == seals[name] for name, raw in metadata.items()), 'published metadata differs')
        published = {row['raw_sha256']: row for row in json.loads(metadata['archive-index.json'])}
        previous = {row['sha256']: row['recovery'] for row in json.loads(metadata['input-recovery.json'])['rows']}
        entries = [('archive-helper', HELPER), ('source-budget.json', ROOT / 'source-budget.json'),
                   ('previous-budget', ROOT / 'source-before-assembly/source-budget.json')]
        entries.extend(('active-source/' + row['path'], ROOT / row['path']) for row in budget['rows'])
        recovered, required, checked = [], {}, set()
        for name in ['u12b-checks-01', 'u12b-check-resource-01']:
            directory = BUILD / name
            receipt = json.loads((directory / 'receipt.json').read_bytes())
            require(receipt['status'].startswith('PASS_56_') and receipt['inputs_before'] == receipt['inputs_after'], 'qualification failed/drifted')
            entries.extend((name + '/' + str(path.relative_to(directory)), path)
                           for path in sorted(directory.rglob('*')) if path.is_file())
            for phase in ['inputs_before', 'inputs_after']:
                for path, digest in receipt[phase].items():
                    require(sha(Path(path).read_bytes()) == digest, 'current input drift: ' + path)
                    required[path] = digest
                    item = published.get(digest)
                    recovery = ({'kind': 'immutable_git_blob', 'commit': COMMIT, 'path': PREFIX + item['stored'],
                                 'encoding': item['encoding'], 'stored_sha256': item['stored_sha256'], 'raw_bytes': item['raw_bytes']}
                                if item else previous.get(digest))
                    if recovery and recovery['kind'] == 'immutable_git_blob':
                        locator = (recovery['commit'], recovery['path'])
                        if locator not in checked:
                            stored = blob(*locator)
                            raw = gzip.decompress(stored) if recovery['encoding'] == 'gzip' else stored
                            require(sha(stored) == recovery['stored_sha256'] and sha(raw) == digest
                                    and len(raw) == recovery['raw_bytes'], 'immutable recovery differs')
                            checked.add(locator)
                    else:
                        recovery = {'kind': 'local_archive', 'raw_sha256': digest}
                        if not any(str(source) == path for _, source in entries):
                            entries.append(('recorded-input/' + digest + '/' + Path(path).name, Path(path)))
                    recovered.append({'receipt': name, 'phase': phase, 'path': path, 'sha256': digest, 'recovery': recovery})
        require(len(recovered) == 74, 'selected phase rows differ')
        inputs = {str(path) for _, path in entries} | set(required)
        record['inputs_before'] = {path: sha(Path(path).read_bytes()) for path in sorted(inputs)}
        rows = archive['archive'](evidence, entries)
        require(all(row['sha256'] in {item['raw_sha256'] for item in rows}
                    for row in recovered if row['recovery']['kind'] == 'local_archive'), 'local recovery missing')
        write(evidence / 'input-recovery.json', {'schema': 1, 'rows': recovered})
        write(evidence / 'dependencies.json', {'tool_pr': 190, 'commit': COMMIT, 'prefix': PREFIX, 'seal_sha256': SEAL,
              'metadata_sha256': {name: sha(raw) for name, raw in metadata.items()}, 'scope': 'Exact byte recovery only; no acceptance transfer'})
        record['inputs_after'] = {path: sha(Path(path).read_bytes()) for path in record['inputs_before']}
        require(record['inputs_before'] == record['inputs_after'], 'assembly input drift')
        record.update(status='PASS_DATA_ASSEMBLY_ONLY;INDEPENDENT_REVIEW_PENDING', archive_entries=len(rows), recovery_rows=len(recovered))
        write(evidence / 'assembly-bindings.json', record)
        record.update(seals=len(archive['seal'](evidence)), seal_sha256=sha((evidence / 'SHA256.json').read_bytes()))
    except BaseException as error:
        record.update(status='FAIL_PRESERVED', error=repr(error))
        raise
    finally:
        write(ROOT / 'assembly-receipt.json', record)


if __name__ == '__main__':
    main()
