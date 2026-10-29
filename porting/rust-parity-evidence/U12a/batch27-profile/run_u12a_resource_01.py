"""Fixed forest profile scope; only proof-bound shared-helper cleanup is allowed."""
from pathlib import Path
import contextlib, hashlib, json, os, signal, subprocess, sys, time
base = Path(__file__).resolve().parent
output = base / 'u12a-profile-resource-01'
output.mkdir(mode=0o700)
driver, profile = base / 'run_u12a_profile_01.py', base / 'u12a_profile.rs'
helper = Path('/workspace/worktrees/builderbob/syson/batch26-scope-ownership/porting/rust-parity-evidence/C01/scope-ownership-repair/scope_ownership.py')
files = {Path(__file__), base / 'u12a_profile_entry.py', helper, driver, profile,
         Path(sys.executable), Path('/usr/bin/python3'), Path('/usr/bin/systemctl'), Path('/usr/bin/systemd-run')}
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
snapshot = lambda: {str(path): sha(path) for path in sorted(files)}
record = {'status': 'FAIL_PRESERVED', 'exit': None, 'timed_out': False,
          'scope': 'Actual forest admission observations only; placement/native/UI/p95/capacity remain OPEN'}
child, ownership, unit, cookie, attempted = None, None, None, None, False
proof_path = output / 'ownership.json'
env = {'PATH': '/usr/bin:/bin', 'HOME': os.environ['HOME'], 'LANG': 'C.UTF-8', 'LC_ALL': 'C.UTF-8',
       'XDG_RUNTIME_DIR': os.environ.get('XDG_RUNTIME_DIR', '/run/user/' + str(os.getuid()))}
def interrupted(*unused):
    raise RuntimeError('Forest profile wrapper interrupted')
signal.signal(signal.SIGTERM, interrupted)
try:
    record['inputs_before'] = snapshot()
    raw = helper.read_bytes()
    if hashlib.sha256(raw).hexdigest() != 'ff18c5d2e6af6a8b21a4d59129401d9c9274f4d4727e2e448eae331c72f6b92d':
        raise RuntimeError('reviewed ownership helper changed')
    ownership = {'__file__': str(helper)}
    exec(compile(raw, str(helper), 'exec'), ownership)
    for path, digest in [(driver, 'f6e0c6d28bd6c6dfeffab6eb815a1c0a5b77d89e15227d7f6f671f5653859988'),
                         (profile, 'b8a8fb36b622398ba886172fdeb5b742aff3068c1efd26098d903be736550f95')]:
        if sha(path) != digest:
            raise RuntimeError('reviewed forest source changed: ' + str(path))
    (output / 'executed-wrapper.py').write_bytes(Path(__file__).read_bytes())
    mount = subprocess.run(['/usr/local/sbin/bob-admin', 'workspace-status'], env=env, capture_output=True, timeout=15)
    (output / 'workspace-status.json').write_bytes(mount.stdout)
    if mount.returncode != 0 or json.loads(mount.stdout).get('outcome') != 'ok':
        raise RuntimeError('Samsung workspace identity not verified')
    mem = {line.split(':')[0]: int(line.split()[1]) for line in Path('/proc/meminfo').read_text().splitlines()}
    disk = os.statvfs('/workspace')
    record['headroom'] = {'available_memory_kib': mem['MemAvailable'], 'available_disk_bytes': disk.f_bavail * disk.f_frsize}
    if mem['MemAvailable'] < 10 * 1024**2 or disk.f_bavail * disk.f_frsize < 30 * 1024**3:
        raise RuntimeError('insufficient fresh headroom')
    unit, cookie = ownership['new_attempt']('syson-u12a-profile-')
    record['unit'], record['cookie'] = unit, cookie
    record['initial_state'] = ownership['state'](unit, env)
    if record['initial_state']['LoadState'] != 'not-found':
        raise RuntimeError('fresh forest profile unit exists; preserve it')
    argv = ['/usr/bin/systemd-run', '--user', '--scope', '--collect', '--quiet', '--unit=' + unit,
            '-p', 'CPUQuota=200%', '-p', 'MemoryHigh=4G', '-p', 'MemoryMax=6G',
            '-p', 'MemorySwapMax=0', '-p', 'TasksMax=128', '-p', 'RuntimeMaxSec=720s',
            '/usr/bin/python3', '-I', '-B', str(base / 'u12a_profile_entry.py'), str(driver),
            record['inputs_before'][str(driver)], record['inputs_before'][str(profile)],
            unit, cookie, str(proof_path), str(output / 'actual-controls.json')]
    record['argv'] = argv
    start = time.monotonic()
    attempted = True
    child = subprocess.Popen(argv, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
    try:
        stdout, stderr = child.communicate(timeout=660)
    except subprocess.TimeoutExpired as error:
        record['timed_out'] = True
        (output / 'stdout.txt').write_bytes(error.output or b'')
        (output / 'stderr.txt').write_bytes(error.stderr or b'')
        raise RuntimeError('forest resource job exceeded 660 seconds') from error
    (output / 'stdout.txt').write_bytes(stdout)
    (output / 'stderr.txt').write_bytes(stderr)
    record.update(exit=child.returncode, wall_seconds=time.monotonic() - start)
    if child.returncode != 0:
        raise RuntimeError('forest profile phase failed; preserve attempt')
    record['status'] = 'PASS_FOREST_ADMISSION_RESOURCE_JOB_ONLY;PLACEMENT_NATIVE_UI_CAPACITY_OPEN'
except BaseException as error:
    record.update(status='FAIL_PRESERVED', error=repr(error))
    raise
finally:
    if attempted:
        try:
            record['scope_cleanup'] = ownership['cleanup'](unit, cookie, proof_path, env)
        except Exception as error:
            record.update(status='FAIL_PRESERVED', scope_cleanup_error=repr(error))
    try:
        if child is not None:
            if child.poll() is None:
                with contextlib.suppress(ProcessLookupError):
                    os.killpg(child.pid, signal.SIGKILL)
            stdout, stderr = child.communicate(timeout=10)
            (output / 'stdout.txt').write_bytes(stdout)
            (output / 'stderr.txt').write_bytes(stderr)
            record.update(exit=child.returncode, wall_seconds=time.monotonic() - start)
    except Exception as error:
        record.update(status='FAIL_PRESERVED', launcher_cleanup_error=repr(error))
    try:
        record['inputs_after'] = snapshot()
        record['inputs_unchanged'] = record.get('inputs_before') == record['inputs_after']
        if not record['inputs_unchanged']:
            raise RuntimeError('forest profile source or tool drift')
    except Exception as error:
        record.update(status='FAIL_PRESERVED', binding_error=repr(error))
    (output / 'receipt.json').write_text(json.dumps(record, indent=2) + '\n')
    if record['status'] == 'FAIL_PRESERVED':
        raise RuntimeError('forest resource job failed; inspect retained receipt')
print(json.dumps({'status': record['status'], 'exit': record['exit'], 'inputs_unchanged': record['inputs_unchanged']}))
