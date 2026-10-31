from pathlib import Path
import fcntl, hashlib, importlib.util, json, os, re, subprocess, sys, time
spec = json.loads(Path(sys.argv[1]).read_text())
out = Path(spec['out'])
out.mkdir()
env = spec['env']
loader = importlib.util.spec_from_file_location('owned_scope', spec['ownership_helper'])
owned = importlib.util.module_from_spec(loader)
loader.loader.exec_module(owned)
owned.attest(spec['unit'], spec['cookie'], spec['prefix'], out / 'scope-proof.json', env)
def snapshot():
    return {p: hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in spec['inputs']}
record = {'status': 'FAIL_TERMINAL', 'inputs_before': snapshot(), 'commands': []}
try:
    with open(spec['compiler_lock']) as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        for step in spec['commands']:
            start = time.monotonic()
            with (out / (step['name'] + '.stdout')).open('wb') as stdout, (out / (step['name'] + '.stderr')).open('wb') as stderr:
                result = subprocess.run(step['argv'], cwd=spec['crate'], env=env, stdout=stdout, stderr=stderr, timeout=180)
            raw = (out / (step['name'] + '.stdout')).read_bytes()
            count = sum(map(int, re.findall(rb'test result: ok\. (\d+) passed;', raw)))
            row = dict(step, exit=result.returncode, wall_seconds=time.monotonic() - start, passed=count)
            record['commands'].append(row)
            print(json.dumps(row), flush=True)
            if result.returncode or ('expected_tests' in step and count != step['expected_tests']):
                raise RuntimeError('Failed ' + step['name'])
        record['status'] = 'PASS_NAMED_LOCAL_CHECKS_ONLY'
finally:
    record['inputs_after'] = snapshot()
    if record['inputs_before'] != record['inputs_after']:
        record['status'] = 'FAIL_SOURCE_DRIFT'
    (out / 'receipt.json').write_text(json.dumps(record, indent=2) + '\n')
    if record['status'] != 'PASS_NAMED_LOCAL_CHECKS_ONLY':
        raise RuntimeError(record['status'])
