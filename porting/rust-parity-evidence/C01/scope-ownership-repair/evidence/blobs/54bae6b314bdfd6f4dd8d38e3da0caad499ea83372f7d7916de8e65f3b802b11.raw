"""One fresh proof-bound live cleanup; no acquired code or foreign unit cleanup."""
from pathlib import Path
import contextlib
import fcntl
import hashlib
import json
import os
import pwd
import signal
import subprocess
import sys
import time

base = Path(__file__).resolve().parent
output = base / 'scope-ownership-live-smoke01'
output.mkdir(mode=0o700)
helper = Path('/workspace/worktrees/builderbob/syson/batch26-scope-ownership/porting/rust-parity-evidence/C01/scope-ownership-repair/scope_ownership.py')
entry = base / 'scope_ownership_smoke_entry.py'
files = [Path(__file__), entry, helper, Path(sys.executable), Path('/usr/bin/systemctl'), Path('/usr/bin/systemd-run')]
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
snapshot = lambda: {str(path): sha(path) for path in files}
record = {'status': 'FAIL', 'scope': 'One reviewed Python helper live owned-cleanup smoke only; no acquired tools/native/UI acceptance'}
child, ownership, unit, cookie, attempted = None, None, None, None, False
proof_path, ready_path = output / 'ownership.json', output / 'ready.json'
env = {'PATH': '/usr/bin:/bin', 'HOME': pwd.getpwuid(os.getuid()).pw_dir, 'LANG': 'C.UTF-8',
       'LC_ALL': 'C.UTF-8', 'XDG_RUNTIME_DIR': '/run/user/' + str(os.getuid())}
def interrupted(*unused):
    raise RuntimeError('live smoke interrupted')
signal.signal(signal.SIGTERM, interrupted)
try:
    if pwd.getpwuid(os.getuid()).pw_name != 'builderbob':
        raise RuntimeError('trusted builderbob identity required')
    lock = Path('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock').open()
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    record['inputs_before'] = snapshot()
    raw = helper.read_bytes()
    if hashlib.sha256(raw).hexdigest() != 'ff18c5d2e6af6a8b21a4d59129401d9c9274f4d4727e2e448eae331c72f6b92d':
        raise RuntimeError('reviewed ownership helper changed')
    ownership = {'__file__': str(helper)}
    exec(compile(raw, str(helper), 'exec'), ownership)
    unit, cookie = ownership['new_attempt']('syson-c01-ownership-smoke-')
    record['unit'], record['cookie'] = unit, cookie
    record['initial_state'] = ownership['state'](unit, env)
    if record['initial_state']['LoadState'] != 'not-found':
        raise RuntimeError('fresh smoke unit exists; preserve it')
    argv = ['/usr/bin/systemd-run', '--user', '--scope', '--collect', '--quiet', '--unit=' + unit,
            '-p', 'CPUQuota=200%', '-p', 'MemoryHigh=4G', '-p', 'MemoryMax=6G',
            '-p', 'MemorySwapMax=0', '-p', 'TasksMax=128', '-p', 'RuntimeMaxSec=60s',
            '/usr/bin/python3', '-I', '-B', str(entry), str(helper), unit, cookie, str(proof_path), str(ready_path)]
    record['argv'] = argv
    attempted = True
    child = subprocess.Popen(argv, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
    deadline = time.monotonic() + 10
    while not ready_path.exists() and child.poll() is None and time.monotonic() < deadline:
        time.sleep(0.05)
    if not ready_path.is_file() or child.poll() is not None:
        raise RuntimeError('smoke entry failed or was not live within ten seconds')
    record['ready'] = json.loads(ready_path.read_bytes())
    record['proof'] = json.loads(proof_path.read_bytes())
    observed = ownership['state'](unit, env)
    ownership['owned'](observed, record['proof'], unit, cookie)
    expected = {'cpu.max': '200000 100000', 'memory.high': '4294967296',
                'memory.max': '6442450944', 'memory.swap.max': '0', 'pids.max': '128'}
    if record['ready'] != {'proof': record['proof'], 'controls': expected, 'entry_pid': record['proof']['entry_pid']}:
        raise RuntimeError('ready controls or proof differ')
    record['verified_live_state'] = observed
    record['positive_cleanup'] = ownership['cleanup'](unit, cookie, proof_path, env)
    observations = record['positive_cleanup']['observations']
    if not record['positive_cleanup']['confirmed_empty_or_absent'] or not any(row.get('kill_exit') == 0 for row in observations):
        raise RuntimeError('positive owned SIGKILL and empty/absent result were not observed')
    stdout, stderr = child.communicate(timeout=10)
    (output / 'stdout.txt').write_bytes(stdout)
    (output / 'stderr.txt').write_bytes(stderr)
    record['returncode'] = child.returncode
    if child.returncode == 0:
        raise RuntimeError('controlled live child unexpectedly exited zero')
    record['status'] = 'PASS_ONE_LIVE_PROOF_BOUND_CLEANUP_ONLY'
except BaseException as error:
    record.update(status='FAIL', error=repr(error))
    raise
finally:
    if attempted:
        try:
            record['final_cleanup'] = ownership['cleanup'](unit, cookie, proof_path, env)
        except Exception as error:
            record.update(status='FAIL', scope_cleanup_error=repr(error))
    try:
        if child is not None:
            if child.poll() is None:
                with contextlib.suppress(ProcessLookupError):
                    os.killpg(child.pid, signal.SIGKILL)
            stdout, stderr = child.communicate(timeout=10)
            (output / 'stdout.txt').write_bytes(stdout)
            (output / 'stderr.txt').write_bytes(stderr)
            record['returncode'] = child.returncode
    except Exception as error:
        record.update(status='FAIL', launcher_cleanup_error=repr(error))
    try:
        record['inputs_after'] = snapshot()
        if record.get('inputs_before') != record['inputs_after']:
            raise RuntimeError('smoke source or tool drift')
    except Exception as error:
        record.update(status='FAIL', binding_error=repr(error))
    (output / 'receipt.json').write_text(json.dumps(record, indent=2) + '\n')
    if record['status'] == 'FAIL':
        raise RuntimeError('live smoke failed; retain attempt and inspect receipt')
