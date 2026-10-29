from pathlib import Path
import contextlib
import fcntl
import hashlib
import json
import os
import re
import signal
import subprocess
import sys
import time

b = Path(__file__).parent
out = b / 'u11a-main-checks-01'
target = b / 'target-u11a-main-checked-01'
if os.path.lexists(out) or os.path.lexists(target):
    raise RuntimeError('Preserve existing forest output or target')
out.mkdir()
(out / 'tmp').mkdir()
crate = Path('/workspace/worktrees/builderbob/syson/batch25-routing-admission/rust/crates/syson-routing')
tool = Path('/workspace/cache/builderbob/global-tools/rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin')
files = {Path(sys.executable), Path(__file__), tool / 'cargo', tool / 'rustc', tool / 'rustfmt', tool / 'clippy-driver'}
files.update(path for path in crate.rglob('*') if path.is_file())
files.update(path for path in (crate.parent / 'syson-layout').rglob('*') if path.is_file())
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
snapshot = lambda: {str(path): sha(path) for path in sorted(files)}
record = {'status': 'RUNNING', 'inputs_before': snapshot(), 'commands': [],
          'scope': '13 actual routing admission cases against current main; search/native/UI/capacity acceptance open'}
(out / 'executed-runner.py').write_bytes(Path(__file__).read_bytes())
child = None

def interrupted(*unused):
    raise RuntimeError('Routing checks interrupted')
signal.signal(signal.SIGTERM, interrupted)
env = {'HOME': os.environ['HOME'], 'PATH': str(tool) + ':/usr/bin:/bin', 'LANG': 'C.UTF-8',
       'LC_ALL': 'C.UTF-8', 'CARGO_HOME': '/workspace/cache/builderbob/global-tools/cargo',
       'RUSTUP_HOME': '/workspace/cache/builderbob/global-tools/rustup',
       'RUSTUP_TOOLCHAIN': '1.99.1-x86_64-unknown-linux-gnu', 'CARGO_BUILD_JOBS': '1',
       'RUST_TEST_THREADS': '1', 'CARGO_INCREMENTAL': '0',
       'CARGO_TARGET_DIR': str(target), 'TMPDIR': str(out / 'tmp')}

def run(label, args):
    global child
    argv = list(map(str, [tool / 'cargo', *args]))
    start = time.monotonic()
    child = subprocess.Popen(argv, cwd=out, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
    failure = None
    try:
        stdout, stderr = child.communicate(timeout=180)
    except BaseException as error:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(child.pid, signal.SIGKILL)
        stdout, stderr = child.communicate(timeout=10)
        failure = repr(error)
    (out / (label + '-stdout.txt')).write_bytes(stdout)
    (out / (label + '-stderr.txt')).write_bytes(stderr)
    row = {'label': label, 'argv': argv, 'exit': child.returncode, 'failure': failure,
           'wall_seconds': time.monotonic() - start,
           'test_cases_passed': sum(map(int, re.findall(rb'test result: ok\. (\d+) passed;', stdout)))}
    record['commands'].append(row)
    child = None
    print(json.dumps(row), flush=True)
    if row['exit'] != 0 or failure is not None:
        raise RuntimeError('failed ' + label)
    return row
try:
    lock = open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock')
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    manifest = crate / 'Cargo.toml'
    run('format', ['fmt', '--manifest-path', manifest, '--', '--check'])
    row = run('tests', ['test', '--offline', '--locked', '--jobs', '1', '--manifest-path', manifest,
                        '--', '--test-threads=1', '--nocapture'])
    if row['test_cases_passed'] != 13:
        raise RuntimeError('Expected 13 actual cases')
    run('clippy', ['clippy', '--offline', '--locked', '--jobs', '1', '--manifest-path', manifest,
                   '--all-targets', '--', '-D', 'warnings'])
    run('wasm', ['check', '--offline', '--locked', '--jobs', '1', '--manifest-path', manifest,
                 '--target', 'wasm32-unknown-unknown'])
    record['status'] = 'PASS_13_CURRENT_MAIN_ROUTING_ADMISSION_CASES_FMT_CLIPPY_WASM;PARENT_ACCEPTANCE_OPEN'
except BaseException as error:
    record.update(status='FAIL_TERMINAL', error=repr(error))
    raise
finally:
    try:
        if child is not None and child.poll() is None:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(child.pid, signal.SIGKILL)
            child.communicate(timeout=10)
        record['inputs_after'] = snapshot()
        if record['inputs_before'] != record['inputs_after']:
            raise RuntimeError('selected input drift')
    except BaseException as error:
        record.update(status='FAIL_TERMINAL', cleanup_or_binding_error=repr(error))
    (out / 'receipt.json').write_text(json.dumps(record, indent=2) + '\n')
    if record.get('cleanup_or_binding_error'):
        raise RuntimeError('routing check cleanup or binding failed')
