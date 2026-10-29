"""Fixed56-case U12b qualification; no native/UI/capacity acceptance."""
from pathlib import Path
import contextlib
import fcntl
import hashlib
import json
import os
import pwd
import re
import signal
import subprocess
import sys
import time

build = Path('/workspace/build/builderbob/syson-batch28-20261029')
out, target = build / 'u12b-checks-01', build / 'target-u12b-checks-01'
if os.path.lexists(out) or os.path.lexists(target):
    raise RuntimeError('preserve existing qualification output/target')
out.mkdir(mode=0o700)
record = {'status': 'FAIL_TERMINAL', 'commands': [],
          'scope': '56 finite layout/forest cases only; native/UI/capacity acceptance OPEN'}
crates = Path('/workspace/worktrees/builderbob/syson/batch27-forest-placement/rust/crates')
roots = [crates / name for name in ['syson-layout', 'syson-node-measurement', 'syson-editor']]
tool = Path('/workspace/cache/builderbob/global-tools/rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin')
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
child = None


def snapshot():
    files = {Path(__file__), Path(sys.executable), tool / 'cargo', tool / 'rustc',
             tool / 'rustfmt', tool / 'clippy-driver'}
    for root in roots:
        if not root.is_dir() or root.is_symlink():
            raise RuntimeError('missing/noncanonical dependency source root')
        files.update(path for path in root.rglob('*') if path.is_file() or path.is_symlink())
    if any(path.is_symlink() for path in files if path != Path(sys.executable)):
        raise RuntimeError('unexpected source/tool symlink')
    return {str(path): sha(path) for path in sorted(files)}


def interrupted(*unused):
    raise RuntimeError('U12b checks interrupted')


def stop():
    if child is not None:
        if child.poll() is None:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(child.pid, signal.SIGKILL)
        child.wait(timeout=10)


def run(label, args):
    global child
    argv = list(map(str, [tool / 'cargo', *args]))
    started, failure = time.monotonic(), None
    stdout, stderr = out / (label + '-stdout.txt'), out / (label + '-stderr.txt')
    with stdout.open('xb') as output, stderr.open('xb') as errors:
        child = subprocess.Popen(argv, cwd=out, env=env, stdout=output, stderr=errors, start_new_session=True)
        try:
            child.wait(timeout=180)
        except BaseException as error:
            failure = repr(error)
            stop()
    raw = stdout.read_bytes()
    row = {'label': label, 'argv': argv, 'exit': child.returncode, 'failure': failure,
           'wall_seconds': time.monotonic() - started, 'stdout_sha256': sha(stdout), 'stderr_sha256': sha(stderr),
           'test_cases_passed': sum(map(int, re.findall(rb'test result: ok\. (\d+) passed;', raw)))}
    record['commands'].append(row)
    child = None
    print(json.dumps(row), flush=True)
    if row['exit'] != 0 or failure is not None:
        raise RuntimeError('failed qualification phase: ' + label)
    return row


signal.signal(signal.SIGTERM, interrupted)
try:
    record['inputs_before'] = snapshot()
    account = pwd.getpwuid(os.getuid())
    if account.pw_name != 'builderbob' or os.environ.get('HOME') != account.pw_dir:
        raise RuntimeError('trusted builderbob HOME required unchanged')
    (out / 'tmp').mkdir(mode=0o700)
    (out / 'executed-checker.py').write_bytes(Path(__file__).read_bytes())
    env = {'HOME': account.pw_dir, 'PATH': str(tool) + ':/usr/bin:/bin', 'LANG': 'C.UTF-8',
           'LC_ALL': 'C.UTF-8', 'CARGO_HOME': '/workspace/cache/builderbob/global-tools/cargo',
           'RUSTUP_HOME': '/workspace/cache/builderbob/global-tools/rustup',
           'RUSTUP_TOOLCHAIN': '1.99.1-x86_64-unknown-linux-gnu', 'CARGO_BUILD_JOBS': '1',
           'RUST_TEST_THREADS': '1', 'CARGO_INCREMENTAL': '0',
           'CARGO_TARGET_DIR': str(target), 'TMPDIR': str(out / 'tmp')}
    lock = open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock')
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    manifest = roots[0] / 'Cargo.toml'
    run('format', ['fmt', '--manifest-path', manifest, '--', '--check'])
    tests = run('tests', ['test', '--offline', '--locked', '--jobs', '1', '--manifest-path', manifest,
                          '--', '--test-threads=1', '--nocapture'])
    if tests['test_cases_passed'] != 56:
        raise RuntimeError('expected exactly56 actual cases')
    run('clippy', ['clippy', '--offline', '--locked', '--jobs', '1', '--manifest-path', manifest,
                   '--all-targets', '--', '-D', 'warnings'])
    run('wasm', ['check', '--offline', '--locked', '--jobs', '1', '--manifest-path', manifest,
                 '--target', 'wasm32-unknown-unknown'])
    record['status'] = 'PASS_56_CASES_FMT_CLIPPY_WASM;NATIVE_UI_CAPACITY_OPEN'
except BaseException as error:
    record.update(status='FAIL_TERMINAL', error=repr(error))
finally:
    try:
        stop()
    except BaseException as error:
        record.update(status='FAIL_TERMINAL', cleanup_error=repr(error))
    try:
        record['inputs_after'] = snapshot()
        if record.get('inputs_before') != record['inputs_after']:
            raise RuntimeError('selected source/tool/path-set drift')
    except BaseException as error:
        record.update(status='FAIL_TERMINAL', binding_error=repr(error))
    (out / 'receipt.json').write_text(json.dumps(record, indent=2) + '\n')
if record['status'] != 'PASS_56_CASES_FMT_CLIPPY_WASM;NATIVE_UI_CAPACITY_OPEN':
    raise SystemExit(1)
