from pathlib import Path
import fcntl
import hashlib
import json
import os
import shutil
import subprocess

b = Path(__file__).parent
original = Path('/workspace/worktrees/builderbob/syson/batch26-scope-ownership/porting/rust-parity-evidence/C01')
out = b / 'scope-repair-controls-01'
out.mkdir()
copy = out / 'source'
copy.mkdir()
fixtures = out / 'fixtures'
fixtures.mkdir()
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
inputs = {str(Path(__file__)): sha(Path(__file__)), '/usr/bin/python3': sha(Path('/usr/bin/python3'))}
for module in ['batch24-tool-versions', 'batch25-maven-bootstrap', 'scope-ownership-repair']:
    for path in sorted((original / module).rglob('*')):
        if path.is_file() and 'evidence' not in path.relative_to(original / module).parts:
            target = copy / path.relative_to(original)
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, target)
            inputs[str(path)] = sha(path)
            inputs[str(target)] = sha(target)
record = {'status': 'FAIL', 'inputs_before': inputs, 'checks': [], 'scope': 'Fake systemd and source controls only; no acquired executable or HTTP request'}
lock = open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock')
fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
env = {'PATH': '/usr/bin:/bin', 'HOME': os.environ['HOME'], 'LANG': 'C.UTF-8',
       'TMPDIR': str(fixtures), 'SYSON_C01_BOOTSTRAP_TEST_TMP': str(fixtures),
       'SYSON_C01_BOOTSTRAP_SOURCE_SHA256': sha(copy / 'batch25-maven-bootstrap/acquire_plugins.py')}
try:
    suites = [('versions', 'batch24-tool-versions/tests/test_version_admission.py', 13),
              ('maven', 'batch25-maven-bootstrap/tests/test_bootstrap.py', 12),
              ('ownership', 'scope-ownership-repair/test_scope_ownership.py', 14)]
    for mode, flags in [('normal', []), ('optimized', ['-O'])]:
        for name, source, count in suites:
            argv = ['/usr/bin/python3', '-I', '-B'] + flags + [str(copy / source), '-v']
            result = subprocess.run(argv, cwd=out, env=env, capture_output=True, timeout=120)
            label = mode + '-' + name
            (out / (label + '-stdout.txt')).write_bytes(result.stdout)
            (out / (label + '-stderr.txt')).write_bytes(result.stderr)
            record['checks'].append({'label': label, 'argv': argv, 'exit': result.returncode, 'required_tests': count})
            if result.returncode != 0 or ('Ran ' + str(count) + ' tests').encode() not in result.stderr:
                raise RuntimeError('control qualification failed: ' + label)
    record['status'] = 'PASS_39_CONTROL_METHODS_EACH_MODE;ACTUAL_SCOPE_SMOKE_PENDING'
except BaseException as error:
    record['error'] = repr(error)
    raise
finally:
    record['inputs_after'] = {name: sha(Path(name)) for name in inputs}
    if record['inputs_before'] != record['inputs_after']:
        record['status'] = 'FAIL_INPUT_DRIFT'
    (out / 'receipt.json').write_text(json.dumps(record, indent=2) + '\n')
    if record['status'] == 'FAIL_INPUT_DRIFT':
        raise RuntimeError('control input drift')
print(json.dumps({'status': record['status'], 'checks': len(record['checks'])}))
