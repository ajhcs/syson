"""Fixed data-only U12 archive assembly; no subject, compiler or test execution."""
from pathlib import Path
import gzip
import hashlib
import json
import os
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
BASE = Path('/workspace/build/builderbob/syson-batch22-20261029')
ARCHIVER = Path('/workspace/build/builderbob/syson-batch15-20261029/archive_helpers.py')
ARCHIVER_SHA = 'b0f94b0d26a3e6d4665f78d42dae289eedf14d54f4646a31bd316d1ba26c4106'
PROFILE_COMMIT = '337433fb18a377e2f1a4c12fe5510b2d0798cfdb'
PROFILE_PREFIX = 'porting/rust-parity-evidence/C09/batch26-routing-admission-profile/evidence/'
PROFILE_SEAL = 'a594d9e8e93695ded8a4e741c869aca1dbf6b98e7e74a8486b8ccefac0d8fa58'
OWNER_COMMIT = '3f95399fc73a081a4486c4e52a983b16cd4ad1bd'
OWNER_PATH = 'porting/rust-parity-evidence/C01/scope-ownership-repair/scope_ownership.py'
OWNER_SHA = 'ff18c5d2e6af6a8b21a4d59129401d9c9274f4d4727e2e448eae331c72f6b92d'
CLIPPY_COMMIT = '3b3b46eda510587c69188d5dcd851932f8cec70b'
CLIPPY_PREFIX = 'porting/rust-parity-evidence/U11a/main-batch26/evidence/'
CLIPPY_SHA = '89b6cc8d0d988b00a3f65bc90c76fedbaedb6bcb442b34563aefe3621f26544b'
CLIPPY_PATH = CLIPPY_PREFIX + 'blobs/' + CLIPPY_SHA + '.gz'
ENV = {'PATH': '/usr/bin:/bin', 'HOME': os.environ['HOME'], 'LANG': 'C.UTF-8',
       'GIT_NO_REPLACE_OBJECTS': '1', 'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': '/dev/null'}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def require(ok, message):
    if not ok:
        raise RuntimeError(message)


def write(path, data):
    with path.open('x') as target:
        target.write(json.dumps(data, indent=2) + '\n')


def git_blob(commit, path):
    result = subprocess.run(['/usr/bin/git', '-C', str(ROOT), 'show', commit + ':' + path],
                            env=ENV, capture_output=True, timeout=30)
    require(result.returncode == 0, 'immutable blob lookup failed: ' + path)
    return result.stdout


def main():
    evidence = ROOT / 'evidence'
    evidence.mkdir(mode=0o700)
    record = {'status': 'FAIL_PRESERVED', 'source_sha256': sha(Path(__file__).read_bytes())}
    try:
        budget = json.loads((ROOT / 'source-budget.json').read_bytes())
        actual_lines = sum(len((ROOT / row['path']).read_bytes().splitlines()) for row in budget['rows'])
        require(actual_lines == budget['authored_lines'] <= 900, 'active authored budget drift')
        require(all(sha((ROOT / row['path']).read_bytes()) == row['sha256'] for row in budget['rows']), 'active source hash drift')
        helper_raw = ARCHIVER.read_bytes()
        require(sha(helper_raw) == ARCHIVER_SHA, 'trusted archive helper drift')
        archive = {'__file__': str(ARCHIVER), '__name__': 'fixed_archive_helper'}
        exec(compile(helper_raw, str(ARCHIVER), 'exec'), archive)
        receipts = {name: json.loads((BASE / name / 'receipt.json').read_bytes())
                    for name in ['u12a-profile-01', 'u12a-profile-resource-01', 'u12a-checks-01']}
        inner, outer, checks = (receipts[name] for name in ['u12a-profile-01', 'u12a-profile-resource-01', 'u12a-checks-01'])
        require(inner['status'] == 'PASS_90_SAMPLES_10_CANCEL_5_INVALID;PARENT_ACCEPTANCE_OPEN', 'inner status differs')
        require(outer['status'] == 'PASS_FOREST_ADMISSION_RESOURCE_JOB_ONLY;PLACEMENT_NATIVE_UI_CAPACITY_OPEN', 'outer status differs')
        require(outer['exit'] == 0 and not outer['timed_out'] and outer['inputs_unchanged'], 'outer phase failed')
        require(inner['prepared_inputs_before'] == inner['inputs_after']
                and inner['compiled_inputs_before'] == inner['compiled_inputs_after'], 'inner binding drift')
        require(len(inner['commands']) == 108, 'unexpected command count')
        for command in inner['commands']:
            require(command['exit'] == command['expected_exit'] and command['interruption_or_timeout'] is None, 'recorded command failed')
            for stream in ['stdout', 'stderr']:
                path = BASE / 'u12a-profile-01' / (command['label'] + '-' + stream + '.txt')
                require(sha(path.read_bytes()) == command[stream + '_sha256'], 'raw command stream differs')
        require(checks['status'] == 'PASS_37_LAYOUT_FOREST_CASES_FMT_CLIPPY_WASM;RESOURCE_PARENT_ACCEPTANCE_OPEN', '37-case qualification status differs')
        require(checks['inputs_before'] == checks['inputs_after'] and len(checks['commands']) == 4, 'qualification binding/count differs')
        require(all(command['exit'] == 0 and command['failure'] is None for command in checks['commands']), 'qualification command failed')
        require(sum(command['test_cases_passed'] for command in checks['commands']) == 37, 'qualification case count differs')
        published_seal = git_blob(PROFILE_COMMIT, PROFILE_PREFIX + 'SHA256.json')
        require(sha(published_seal) == PROFILE_SEAL, 'immutable profile seal differs')
        seals = json.loads(published_seal)
        index_raw = git_blob(PROFILE_COMMIT, PROFILE_PREFIX + 'archive-index.json')
        require(sha(index_raw) == seals['archive-index.json'], 'immutable index differs')
        published = {row['raw_sha256']: row for row in json.loads(index_raw)}
        require(sha(git_blob(OWNER_COMMIT, OWNER_PATH)) == OWNER_SHA, 'immutable owner helper differs')
        clippy_stored = git_blob(CLIPPY_COMMIT, CLIPPY_PATH)
        clippy_raw = gzip.decompress(clippy_stored)
        require(sha(clippy_raw) == CLIPPY_SHA, 'immutable Clippy bytes differ')
        prior_path = ROOT / 'assembly-before-checks/epoch.json'
        prior = json.loads(prior_path.read_bytes())
        require(sha((prior_path.parent / 'evidence/SHA256.json').read_bytes()) == prior['seal_sha256'], 'prior assembly seal differs')
        entries = [('prior-assembly/epoch.json', prior_path)]
        for name in ['u12a-profile-01', 'u12a-profile-resource-01', 'u12a-profile-before-format', 'u12a-checks-01']:
            entries.extend((name + '/' + str(path.relative_to(BASE / name)), path)
                           for path in sorted((BASE / name).rglob('*')) if path.is_file())
        for path in sorted(ROOT.iterdir()):
            if path.suffix in ['.py', '.rs']:
                entries.append(('active-source/' + path.name, path))
        entries.append(('assembly-source/archive_helpers.py', ARCHIVER))
        local_paths = {str(path) for _, path in entries}
        recovered, checked, required = [], {}, {}
        for name, receipt in receipts.items():
            phases = ['inputs_before', 'prepared_inputs_before', 'inputs_after',
                      'compiled_inputs_before', 'compiled_inputs_after'] if name == 'u12a-profile-01' else ['inputs_before', 'inputs_after']
            for phase in phases:
                for path, digest in receipt[phase].items():
                    require(sha(Path(path).read_bytes()) == digest, 'current bound bytes differ: ' + path)
                    required[path] = digest
                    row = {'receipt': name, 'phase': phase, 'path': path, 'sha256': digest}
                    if digest in published:
                        item = published[digest]
                        stored_name = item['stored']
                        if stored_name not in checked:
                            stored = git_blob(PROFILE_COMMIT, PROFILE_PREFIX + stored_name)
                            require(sha(stored) == seals[stored_name] == item['stored_sha256'], 'published stored bytes differ')
                            raw = gzip.decompress(stored) if item['encoding'] == 'gzip' else stored
                            require(sha(raw) == digest and len(raw) == item['raw_bytes'], 'published raw bytes differ')
                            checked[stored_name] = True
                        row['recovery'] = {'kind': 'immutable_git_blob', 'commit': PROFILE_COMMIT,
                                           'path': PROFILE_PREFIX + stored_name, 'encoding': item['encoding'],
                                           'stored_sha256': item['stored_sha256'], 'raw_bytes': item['raw_bytes']}
                    elif digest == CLIPPY_SHA:
                        row['recovery'] = {'kind': 'immutable_git_blob', 'commit': CLIPPY_COMMIT,
                                           'path': CLIPPY_PATH, 'encoding': 'gzip',
                                           'stored_sha256': sha(clippy_stored), 'raw_bytes': len(clippy_raw)}
                    elif digest == OWNER_SHA:
                        row['recovery'] = {'kind': 'immutable_git_blob', 'commit': OWNER_COMMIT,
                                           'path': OWNER_PATH, 'encoding': 'identity',
                                           'stored_sha256': OWNER_SHA, 'raw_bytes': Path(path).stat().st_size}
                    else:
                        row['recovery'] = {'kind': 'local_archive', 'raw_sha256': digest}
                        if path not in local_paths:
                            entries.append(('recorded-input/' + digest + '/' + Path(path).name, Path(path)))
                            local_paths.add(path)
                    recovered.append(row)
        input_paths = {str(path) for _, path in entries} | set(required)
        record['inputs_before'] = {path: sha(Path(path).read_bytes()) for path in sorted(input_paths)}
        rows = archive['archive'](evidence, entries)
        local_hashes = {row['raw_sha256'] for row in rows}
        require(all(row['sha256'] in local_hashes for row in recovered if row['recovery']['kind'] == 'local_archive'), 'missing local recovery')
        for row in rows:
            stored = (evidence / row['stored']).read_bytes()
            raw = gzip.decompress(stored) if row['encoding'] == 'gzip' else stored
            require(sha(stored) == row['stored_sha256'] and sha(raw) == row['raw_sha256']
                    and len(raw) == row['raw_bytes'], 'assembled blob differs')
        write(evidence / 'input-recovery.json', {'schema': 1, 'rows': recovered})
        write(evidence / 'dependencies.json', {
            'routing_profile': {'pr': 183, 'commit': PROFILE_COMMIT, 'module_tree': '37a3e6c0d80b7cb89f8cee8a939bdd80a94131e7',
                                'evidence_prefix': PROFILE_PREFIX, 'seal_sha256': PROFILE_SEAL},
            'ownership_helper': {'pr': 184, 'commit': OWNER_COMMIT, 'path': OWNER_PATH, 'sha256': OWNER_SHA},
            'clippy_recovery': {'pr': 186, 'commit': CLIPPY_COMMIT, 'path': CLIPPY_PATH, 'sha256': CLIPPY_SHA},
            'prior_assembly': {'path': 'assembly-before-checks/epoch.json', 'sha256': sha(prior_path.read_bytes()),
                               'seal_sha256': prior['seal_sha256'], 'scope': 'Earlier executed profile-only assembly, superseded by this qualification-inclusive packet'},
            'archive_helper': {'original_path': str(ARCHIVER), 'sha256': ARCHIVER_SHA},
            'scope': 'Byte recovery only; no transfer of feature acceptance or another workload qualification'})
        record['inputs_after'] = {path: sha(Path(path).read_bytes()) for path in record['inputs_before']}
        require(record['inputs_before'] == record['inputs_after'], 'assembly input drift')
        record.update(status='PASS_DATA_ASSEMBLY_ONLY;INDEPENDENT_PACKET_REVIEW_PENDING',
                      archive_entries=len(rows), recovery_rows=len(recovered), distinct_bound_paths=len(required))
        write(evidence / 'assembly-bindings.json', record)
        sealed = archive['seal'](evidence)
        record.update(seals=len(sealed), seal_sha256=sha((evidence / 'SHA256.json').read_bytes()))
    except BaseException as error:
        record.update(status='FAIL_PRESERVED', error=repr(error))
        raise
    finally:
        write(ROOT / 'assembly-receipt.json', record)


if __name__ == '__main__':
    main()
