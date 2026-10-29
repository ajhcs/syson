"""Fixed full-placement observation job; only the driver owns the compiler lock."""
from pathlib import Path
import contextlib
import hashlib
import json
import os
import pwd
import signal
import subprocess
import sys
import time

base = Path(__file__).resolve().parent
build = Path('/workspace/build/builderbob/syson-batch28-20261029')
output = build / 'u12b-profile-resource-01'
if build.resolve() != build or os.path.lexists(output):
    raise RuntimeError('preserve existing/noncanonical qualification output')
output.mkdir(mode=0o700)
helper = Path('/workspace/worktrees/builderbob/syson/batch26-scope-ownership/porting/rust-parity-evidence/C01/scope-ownership-repair/scope_ownership.py')
checker, entry, profile = base / 'run_profile.py', base / 'entry.py', base / 'profile.rs'
files = {Path(__file__), checker, entry, profile, helper, Path(sys.executable), Path('/usr/bin/python3'),
         Path('/usr/bin/systemctl'), Path('/usr/bin/systemd-run'), Path('/usr/local/sbin/bob-admin')}
files.update(base.glob('*.py'))
files.update([base.parents[1] / 'C09/batch26-routing-admission-profile/u11a_profile.rs',
              base.parents[1] / 'U12a/batch27-profile/u12a_profile.rs'])
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
snapshot = lambda: {str(path): sha(path) for path in sorted(files)}
record = {'status': 'FAIL_TERMINAL', 'exit': None, 'timed_out': False,
          'scope': 'Experimental2GiBhigh/3GiBmax placement observations only; native/UI/capacity OPEN'}
child, owner, unit, cookie, env = None, None, None, None, None
attempted = False
started = time.monotonic()


def interrupted(*unused):
    raise RuntimeError('U12b qualification interrupted')


signal.signal(signal.SIGTERM, interrupted)
try:
    record['inputs_before'] = snapshot()
    account = pwd.getpwuid(os.getuid())
    if account.pw_name != 'builderbob' or os.environ.get('HOME') != account.pw_dir:
        raise RuntimeError('trusted builderbob HOME required unchanged')
    env = {'PATH': '/usr/bin:/bin', 'HOME': account.pw_dir, 'LANG': 'C.UTF-8', 'LC_ALL': 'C.UTF-8',
           'XDG_RUNTIME_DIR': os.environ.get('XDG_RUNTIME_DIR', '/run/user/' + str(os.getuid()))}
    raw = helper.read_bytes()
    if hashlib.sha256(raw).hexdigest() != 'ff18c5d2e6af6a8b21a4d59129401d9c9274f4d4727e2e448eae331c72f6b92d':
        raise RuntimeError('reviewed ownership helper changed')
    owner = {'__file__': str(helper)}
    exec(compile(raw, str(helper), 'exec'), owner)
    if sha(checker) != 'e2473fbfa7cbed318b60a4534236b79c37667fc6cebb05c52d79b7232f0bf1fd' or sha(entry) != 'c43f92077e1c8e7e796160121d6f8253088fe6db764887b1b6b1e97c5743f550' or sha(profile) != '69a362fc873716312f77a9231e43a1255e887b00dd2683af368ab5fe0d3c420b':
        raise RuntimeError('reviewed qualification source changed')
    (output / 'executed-wrapper.py').write_bytes(Path(__file__).read_bytes())
    mount = subprocess.run(['/usr/local/sbin/bob-admin', 'workspace-status'], env=env, capture_output=True, timeout=15)
    (output / 'workspace-status.json').write_bytes(mount.stdout)
    (output / 'workspace-status-stderr.txt').write_bytes(mount.stderr)
    if mount.returncode != 0 or json.loads(mount.stdout).get('outcome') != 'ok':
        raise RuntimeError('Samsung workspace not verified')
    memory = {line.split(':')[0]: int(line.split()[1]) for line in Path('/proc/meminfo').read_text().splitlines()}
    disk = os.statvfs('/workspace')
    record['headroom'] = {'memory_kib': memory['MemAvailable'], 'disk_bytes': disk.f_bavail * disk.f_frsize}
    if memory['MemAvailable'] < 7 * 1024**2 or disk.f_bavail * disk.f_frsize < 30 * 1024**3:
        raise RuntimeError('insufficient fresh full-profile headroom')
    unit, cookie = owner['new_attempt']('syson-u12b-placement-profile-')
    record.update(unit=unit, cookie=cookie, initial_scope=owner['state'](unit, env))
    if record['initial_scope']['LoadState'] != 'not-found':
        raise RuntimeError('preserve existing qualification scope')
    argv = ['/usr/bin/systemd-run', '--user', '--scope', '--collect', '--quiet', '--unit=' + unit,
            '-p', 'CPUQuota=200%', '-p', 'MemoryHigh=2G', '-p', 'MemoryMax=3G',
            '-p', 'MemorySwapMax=0', '-p', 'TasksMax=128', '-p', 'RuntimeMaxSec=720s',
            '/usr/bin/python3', '-I', '-B', str(entry), record['inputs_before'][str(checker)],
            record['inputs_before'][str(profile)], unit, cookie, str(output)]
    record['argv'] = argv
    attempted = True
    with (output / 'stdout.txt').open('xb') as stdout, (output / 'stderr.txt').open('xb') as stderr:
        child = subprocess.Popen(argv, env=env, stdout=stdout, stderr=stderr, start_new_session=True)
        child.wait(timeout=660)
    record['exit'] = child.returncode
    inner = json.loads((build / 'u12b-profile-01/receipt.json').read_bytes())
    if child.returncode != 0 or inner['status'] != 'PASS_90_PLACEMENT_SAMPLES_10_PRE_CANCEL_4_ADMISSION_8_CONTROLS_5_INVALID;PARENT_ACCEPTANCE_OPEN':
        raise RuntimeError('placement observations failed; preserve attempt')
    record['status'] = 'PASS_FULL_PLACEMENT_PROFILE_RESOURCE_JOB;NATIVE_UI_CAPACITY_OPEN'
except BaseException as error:
    record.update(status='FAIL_TERMINAL', error=repr(error), timed_out=isinstance(error, subprocess.TimeoutExpired))
finally:
    if attempted:
        try:
            record['scope_cleanup'] = owner['cleanup'](unit, cookie, output / 'ownership.json', env)
        except BaseException as error:
            record.update(status='FAIL_TERMINAL', scope_cleanup_error=repr(error))
    try:
        if child is not None:
            if child.poll() is None:
                with contextlib.suppress(ProcessLookupError):
                    os.killpg(child.pid, signal.SIGKILL)
            child.wait(timeout=10)
            record['exit'] = child.returncode
    except BaseException as error:
        record.update(status='FAIL_TERMINAL', launcher_cleanup_error=repr(error))
    try:
        record['inputs_after'] = snapshot()
        if record.get('inputs_before') != record['inputs_after']:
            raise RuntimeError('qualification wrapper/input drift')
    except BaseException as error:
        record.update(status='FAIL_TERMINAL', binding_error=repr(error))
    record['wall_seconds'] = time.monotonic() - started
    try:
        record['retained'] = {str(path.relative_to(output)): {'bytes': path.stat().st_size, 'sha256': sha(path)}
                              for path in sorted(output.rglob('*')) if path.is_file()}
    except BaseException as error:
        record.update(status='FAIL_TERMINAL', retention_error=repr(error))
    (output / 'receipt.json').write_text(json.dumps(record, indent=2) + '\n')
if record['status'] != 'PASS_FULL_PLACEMENT_PROFILE_RESOURCE_JOB;NATIVE_UI_CAPACITY_OPEN':
    raise SystemExit(1)
print(json.dumps({'status': record['status'], 'unit': unit, 'exit': record['exit']}))
