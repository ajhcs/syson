from pathlib import Path
import fcntl, hashlib, json, os, statistics, subprocess, time
from types import ModuleType
b = Path(__file__).parent
o = b / 'c09b1-checks-01'
o.mkdir()
(o / 'tmp').mkdir()
w = Path('/workspace/worktrees/builderbob/syson/batch24-comment-projection')
module = w / 'porting/rust-parity-evidence/C09/batch22-comment-comparison'
constructor = w / 'porting/rust-parity-evidence/C09/batch21-constructor-tools/compare_construction.py'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
config = json.loads((b / 'c09a-checks-03/fixtures.json').read_bytes())
config.update(constructor_comparator=str(constructor), temporary_root=str(o / 'tmp'),
              capture_admission=str(module / 'capture_p01.py'), projection_sha256=sha(module / 'projection_p01.py'))
manifest = o / 'fixtures.json'
manifest.write_text(json.dumps(config, indent=2) + '\n')
(o / 'executed-runner.py').write_bytes(Path(__file__).read_bytes())
files = {Path('/usr/bin/python3'), Path(__file__), manifest, constructor,
         module / 'projection_p01.py', module / 'capture_p01.py',
         module / 'tests/test_capture_p01.py', module / 'tests/test_projection_p01.py'}
files.update(Path(p) for epoch in config['epochs'].values() for p in epoch.values())
snapshot = lambda: {str(p): sha(p) for p in sorted(files)}
record = {'status': 'RUNNING', 'inputs_before': snapshot(), 'commands': [], 'api_samples': [],
          'scope': 'Finite P01 projection through fresh admission; API compute time only, no UI/native causality or parent acceptance.'}
env = {'PATH': '/usr/bin:/bin', 'HOME': os.environ['HOME'], 'LANG': 'C.UTF-8', 'LC_ALL': 'C.UTF-8',
       'TMPDIR': str(o / 'tmp'), 'PYTHONDONTWRITEBYTECODE': '1',
       'SYSON_C09A_FIXTURES': str(manifest), 'SYSON_C09B1_FIXTURES': str(manifest)}
lock = open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock')
fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
try:
    for optimized in (False, True):
        for name, expected in [('capture', 19), ('projection', 20)]:
            label = ('optimized-' if optimized else 'normal-') + name
            argv = ['/usr/bin/python3', '-B'] + (['-O'] if optimized else []) + [str(module / ('tests/test_' + name + '_p01.py')), '-v']
            started = time.monotonic()
            r = subprocess.run(argv, cwd=o, env=env, capture_output=True, timeout=120)
            (o / (label + '-stdout.txt')).write_bytes(r.stdout)
            (o / (label + '-stderr.txt')).write_bytes(r.stderr)
            record['commands'].append({'label': label, 'argv': argv, 'exit': r.returncode, 'wall_seconds': time.monotonic() - started})
            if r.returncode != 0 or ('Ran ' + str(expected) + ' tests').encode() not in r.stderr:
                raise RuntimeError('actual named tests failed: ' + label)
    raw = (module / 'projection_p01.py').read_bytes()
    if hashlib.sha256(raw).hexdigest() != config['projection_sha256']:
        raise RuntimeError('projection drift before profile')
    subject = ModuleType('reviewed_p01_projection')
    subject.__file__ = str(module / 'projection_p01.py')
    exec(compile(raw, subject.__file__, 'exec'), subject.__dict__)
    for epoch, expected in [('15', 2), ('25', 0)]:
        blobs = [Path(config['epochs'][epoch][n]).read_bytes() for n in ['corpus', 'native', 'rust', 'receipt']]
        for index in range(9):
            start = time.perf_counter()
            result = subject.compare(*blobs, constructor, module / 'capture_p01.py')
            elapsed = time.perf_counter() - start
            if len(result['cases']) != int(epoch) or result['difference_count'] != expected or result['full_parity_claimed']:
                raise RuntimeError('profile outcome changed')
            record['api_samples'].append({'epoch': epoch, 'warmup': index < 2, 'milliseconds': elapsed * 1000})
        (o / ('epoch-' + epoch + '-report.json')).write_text(json.dumps(result, indent=2) + '\n')
    record['ranges_ms'] = {epoch: {'min': min(values), 'median': statistics.median(values), 'max': max(values)}
                           for epoch in ['15', '25']
                           for values in [[s['milliseconds'] for s in record['api_samples'] if s['epoch'] == epoch and not s['warmup']]]}
    record['status'] = 'PASS_19_CAPTURE_AND_20_PROJECTION_TESTS_NORMAL_OPTIMIZED_AND_18_API_SAMPLES'
except BaseException as error:
    record.update(status='FAIL_TERMINAL', error=repr(error))
    raise
finally:
    try:
        record['inputs_after'] = snapshot()
        record['source_unchanged'] = record['inputs_before'] == record['inputs_after']
    except OSError as error:
        record.update(status='FAIL_TERMINAL', source_unchanged=False, inputs_after_error=repr(error))
    if not record['source_unchanged']:
        record.update(status='FAIL_TERMINAL', error='selected input drift')
    (o / 'receipt.json').write_text(json.dumps(record, indent=2) + '\n')
    if not record['source_unchanged']:
        raise RuntimeError('selected input drift')
print(json.dumps({k: record[k] for k in ['status', 'ranges_ms', 'source_unchanged']}))
