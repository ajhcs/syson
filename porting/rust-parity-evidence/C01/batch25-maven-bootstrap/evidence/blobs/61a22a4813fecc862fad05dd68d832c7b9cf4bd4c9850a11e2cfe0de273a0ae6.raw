from pathlib import Path
import fcntl
import hashlib
import json
import os
import subprocess

b = Path(__file__).parent
source = b / 'c01-maven-bootstrap'
out = b / 'c01-maven-bootstrap-controls-01'
out.mkdir()
fixtures = out / 'fixtures'
fixtures.mkdir()
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
inputs = {str(f): sha(f) for f in source.rglob('*') if f.is_file()}
inputs[str(Path(__file__))] = sha(Path(__file__))
record = {'status': 'FAIL', 'inputs_before': inputs, 'checks': [], 'network_or_acquired_execution': False}
lock = open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock')
fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
env = {'PATH': '/usr/bin:/bin', 'HOME': os.environ['HOME'], 'LANG': 'C.UTF-8',
       'SYSON_C01_BOOTSTRAP_SOURCE_SHA256': '7a171424c4fe46664bb4cace3e6c6695ad25c5b6a86f968cbd3dd1c54e6cb42d',
       'SYSON_C01_BOOTSTRAP_TEST_TMP': str(fixtures)}
try:
    for mode, flags in [('normal', []), ('optimized', ['-O'])]:
        argv = ['/usr/bin/python3', '-I', '-B'] + flags + [str(source / 'tests/test_bootstrap.py'), '-v']
        result = subprocess.run(argv, env=env, capture_output=True, timeout=120)
        (out / (mode + '-stdout.txt')).write_bytes(result.stdout)
        (out / (mode + '-stderr.txt')).write_bytes(result.stderr)
        record['checks'].append({'mode': mode, 'argv': argv, 'exit': result.returncode})
        if result.returncode != 0 or b'Ran 12 tests' not in result.stderr:
            raise RuntimeError('mocked controls failed: ' + mode)
    record['status'] = 'PASS_12_MOCKED_CONTROLS_EACH_MODE_ONLY'
except BaseException as error:
    record['error'] = repr(error)
    raise
finally:
    record['inputs_after'] = {name: sha(Path(name)) for name in inputs}
    if record['inputs_before'] != record['inputs_after']:
        record['status'] = 'FAIL_SOURCE_DRIFT'
    (out / 'receipt.json').write_text(json.dumps(record, indent=2) + '\n')
    if record['status'] == 'FAIL_SOURCE_DRIFT':
        raise RuntimeError('mocked-control input drift')
print(json.dumps({'status': record['status'], 'checks': len(record['checks'])}))
