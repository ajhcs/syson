"""Fixed raw-data assembly only; no compiler, tests, subject or network execution."""
from pathlib import Path
import gzip
import hashlib
import json
import os
import subprocess

ROOT = Path(__file__).resolve().parent
BASE = Path('/workspace/build/builderbob/syson-batch27-20261029')
HELPER = Path('/workspace/build/builderbob/syson-batch15-20261029/archive_helpers.py')
ANCHORS = [
    (183, '337433fb18a377e2f1a4c12fe5510b2d0798cfdb', 'porting/rust-parity-evidence/C09/batch26-routing-admission-profile/evidence/', 'a594d9e8e93695ded8a4e741c869aca1dbf6b98e7e74a8486b8ccefac0d8fa58'),
    (184, '3f95399fc73a081a4486c4e52a983b16cd4ad1bd', 'porting/rust-parity-evidence/C01/scope-ownership-repair/evidence/', 'c90bab5df725617573e408652149a3ce0467c3c535e9df3594ff9b1a3ab2b52f'),
    (186, '3b3b46eda510587c69188d5dcd851932f8cec70b', 'porting/rust-parity-evidence/U11a/main-batch26/evidence/', '9e132468f3eb892393ad3b6029127e9dfaacb69e53608b8dd3efb7a68dac36cb'),
    (190, 'ca10950d8f4aa0fc6d002d8121b33afad2e0cbba', 'porting/rust-parity-evidence/U12a/batch27-profile/evidence/', '3cccc83eefc56b9ee22a3232773472d9dffe5bb8ef9486dc4ff1ec6f3bf734ae'),
]
ENV = {'PATH': '/usr/bin:/bin', 'HOME': os.environ['HOME'], 'LANG': 'C.UTF-8',
       'GIT_NO_REPLACE_OBJECTS': '1', 'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': '/dev/null'}
sha = lambda raw: hashlib.sha256(raw).hexdigest()
read_git = lambda commit, path: subprocess.check_output(
    ['/usr/bin/git', '-C', str(ROOT), 'show', commit + ':' + path], env=ENV, timeout=30)
evidence = ROOT / 'evidence'
evidence.mkdir(mode=0o700)
record = {'status': 'FAIL_PRESERVED', 'scope': 'Raw archival and byte recovery only; independent qualification remains required'}
try:
    helper_raw = HELPER.read_bytes()
    if sha(helper_raw) != 'b0f94b0d26a3e6d4665f78d42dae289eedf14d54f4646a31bd316d1ba26c4106':
        raise RuntimeError('trusted archive helper drift')
    api = {'__file__': str(HELPER), '__name__': 'fixed_archive_api'}
    exec(compile(helper_raw, str(HELPER), 'exec'), api)
    required = ['u11b-search-profile-01', 'u11b-search-resource-01', 'u11b-search-resource-02',
                'u11b-search-resource-04', 'u11b-checks-01', 'u11b-checks-02',
                'u11b-profile-before-format', 'u11b-profile-before-reuse',
                'u11b-profile-preflight-01-epoch', 'u11b-profile-preflight-02-epoch']
    entries = [('assembly-helper/archive_helpers.py', HELPER)]
    for name in required:
        if not (BASE / name).is_dir():
            raise RuntimeError('missing required retained epoch: ' + name)
        entries.extend((name + '/' + str(path.relative_to(BASE / name)), path)
                       for path in sorted((BASE / name).rglob('*')) if path.is_file())
    unexecuted = Path('/workspace/build/builderbob/syson-batch28-20261029/u11b-unexecuted-03-epoch')
    if not unexecuted.is_dir():
        raise RuntimeError('missing unexecuted attempt-03 source epoch')
    entries.extend(('u11b-unexecuted-03-epoch/' + str(path.relative_to(unexecuted)), path)
                   for path in sorted(unexecuted.rglob('*')) if path.is_file())
    entries.extend(('current-source/' + path.name, path) for path in sorted(ROOT.iterdir())
                   if path.is_file() and path.name != 'assembly-receipt.json')
    receipts = [(name, json.loads((BASE / name / 'receipt.json').read_bytes())) for name in required[:6]]
    published = {}
    for pr, commit, prefix, expected in ANCHORS:
        seal_raw = read_git(commit, prefix + 'SHA256.json')
        seals = json.loads(seal_raw)
        index_raw = read_git(commit, prefix + 'archive-index.json')
        if sha(seal_raw) != expected or sha(index_raw) != seals['archive-index.json']:
            raise RuntimeError('immutable anchor seal/index differs: ' + str(pr))
        for item in json.loads(index_raw):
            published[item['raw_sha256']] = (pr, commit, prefix, item, seals[item['stored']])
    local = {sha(path.read_bytes()) for _, path in entries}
    recovered, checked = [], set()
    for name, receipt in receipts:
        for phase in ['inputs_before', 'prepared_inputs_before', 'inputs_after',
                      'compiled_inputs_before', 'compiled_inputs_after']:
            for path, digest in receipt.get(phase, {}).items():
                row = {'receipt': name, 'phase': phase, 'original_path': path, 'raw_sha256': digest}
                if digest in local:
                    row['recovery'] = {'kind': 'local_archive', 'raw_sha256': digest}
                elif digest in published:
                    pr, commit, prefix, item, stored_sha = published[digest]
                    if digest not in checked:
                        stored = read_git(commit, prefix + item['stored'])
                        raw = gzip.decompress(stored) if item['encoding'] == 'gzip' else stored
                        if sha(stored) != stored_sha or stored_sha != item['stored_sha256'] or sha(raw) != digest or len(raw) != item['raw_bytes']:
                            raise RuntimeError('immutable byte recovery differs: ' + path)
                        checked.add(digest)
                    row['recovery'] = {'kind': 'immutable_git_blob', 'pr': pr, 'commit': commit,
                        'path': prefix + item['stored'], 'stored_sha256': stored_sha,
                        'encoding': item['encoding'], 'raw_bytes': item['raw_bytes']}
                else:
                    source = Path(path)
                    if sha(source.read_bytes()) != digest:
                        raise RuntimeError('recorded input is not recoverable: ' + path)
                    entries.append(('recorded-input/' + digest + '/' + source.name, source))
                    local.add(digest)
                    row['recovery'] = {'kind': 'local_archive', 'raw_sha256': digest}
                recovered.append(row)
    inputs = {str(path): sha(path.read_bytes()) for _, path in entries}
    record['inputs_before'] = inputs
    rows = api['archive'](evidence, entries)
    archived = {row['raw_sha256'] for row in rows}
    if any(row['raw_sha256'] not in archived for row in recovered if row['recovery']['kind'] == 'local_archive'):
        raise RuntimeError('local input recovery omitted bytes')
    bindings = {'anchors': [{'pr': pr, 'commit': commit, 'prefix': prefix, 'seal_sha256': seal}
                           for pr, commit, prefix, seal in ANCHORS], 'rows': recovered}
    (evidence / 'input-recovery.json').write_text(json.dumps(bindings, indent=2) + '\n')
    record['inputs_after'] = {path: sha(Path(path).read_bytes()) for path in inputs}
    if inputs != record['inputs_after']:
        raise RuntimeError('assembly inputs changed')
    record.update(status='ASSEMBLED_RAW_DATA_ONLY;INDEPENDENT_REVIEW_PENDING',
                  archive_entries=len(rows), recovery_rows=len(recovered))
    (evidence / 'assembly-bindings.json').write_text(json.dumps(record, indent=2) + '\n')
    sealed = api['seal'](evidence)
    record.update(seals=len(sealed), seal_sha256=sha((evidence / 'SHA256.json').read_bytes()))
except BaseException as error:
    record.update(status='FAIL_PRESERVED', error=repr(error))
    raise
finally:
    (ROOT / 'assembly-receipt.json').write_text(json.dumps(record, indent=2) + '\n')
