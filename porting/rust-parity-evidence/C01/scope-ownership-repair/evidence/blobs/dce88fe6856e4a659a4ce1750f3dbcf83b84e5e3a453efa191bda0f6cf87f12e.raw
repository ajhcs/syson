from pathlib import Path
import fcntl, hashlib, json, os, signal, subprocess, time
base = Path(__file__).resolve().parent
helper = base.parent / 'scope-ownership-repair/scope_ownership.py'
helper_raw = helper.read_bytes()
if hashlib.sha256(helper_raw).hexdigest() != 'ff18c5d2e6af6a8b21a4d59129401d9c9274f4d4727e2e448eae331c72f6b92d':
    raise RuntimeError('reviewed scope ownership helper changed')
ownership = {'__file__': str(helper)}
exec(compile(helper_raw, str(helper), 'exec'), ownership)
outdir = base.parent / 'c01-version-resource-01'
outdir.mkdir(mode=0o700)
child = None
launch_attempted = False
unit, cookie = ownership['new_attempt']('syson-c01-versions-')
proof_path = outdir / 'ownership.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
files = {helper, Path(__file__), base / 'entry.py', base / 'check_versions.py', base / 'proposal.json',
         Path('/usr/bin/python3'), Path('/usr/bin/systemd-run'), Path('/usr/bin/systemctl')}
files.update(p for p in (base / 'configs').iterdir() if p.is_file())
files.update(p for p in (base / 'tests').rglob('*.py') if p.is_file())
snapshot = lambda: {str(p): sha(p) for p in sorted(files)}
record = {'status': 'PREPARING', 'exit': None, 'timed_out': False, 'scope_unit': unit, 'inputs_before': snapshot(), 'scope': 'Fixed version handlers only; no build, dependency, native application or UI acceptance.'}
(outdir / 'executed-wrapper.py').write_bytes(Path(__file__).read_bytes())
env = {'PATH': '/usr/bin:/bin', 'HOME': os.environ['HOME'], 'LANG': 'C.UTF-8', 'LC_ALL': 'C.UTF-8',
       'XDG_RUNTIME_DIR': os.environ.get('XDG_RUNTIME_DIR', '/run/user/' + str(os.getuid()))}
def interrupted(*args):
    raise RuntimeError('version wrapper interrupted')
signal.signal(signal.SIGTERM, interrupted)
def scope_state():
    return ownership['state'](unit, env)

def cleanup_scope():
    return ownership['cleanup'](unit, cookie, proof_path, env)
try:
    lock = open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock')
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    mount = subprocess.run(['/usr/local/sbin/bob-admin', 'workspace-status'], env=env, capture_output=True, timeout=15)
    (outdir / 'workspace-status.json').write_bytes(mount.stdout)
    if mount.returncode != 0 or json.loads(mount.stdout).get('outcome') != 'ok':
        raise RuntimeError('Samsung workspace identity not verified')
    mem = {row.split(':')[0]: int(row.split()[1]) for row in Path('/proc/meminfo').read_text().splitlines()}
    disk = os.statvfs('/workspace')
    record['headroom'] = {'available_memory_kib': mem['MemAvailable'], 'available_disk_bytes': disk.f_bavail * disk.f_frsize}
    if mem['MemAvailable'] < 10 * 1024**2 or disk.f_bavail * disk.f_frsize < 30 * 1024**3:
        raise RuntimeError('insufficient fresh headroom')
    argv = ['/usr/bin/systemd-run', '--user', '--scope', '--quiet', '--collect',
            '--unit=' + unit, '-p', 'CPUQuota=200%', '-p', 'MemoryHigh=4G',
            '-p', 'MemoryMax=6G', '-p', 'MemorySwapMax=0', '-p', 'TasksMax=128',
            '-p', 'RuntimeMaxSec=300s', '/usr/bin/python3', '-I', '-B', str(base / 'entry.py'),
            str(base / 'check_versions.py'), record['inputs_before'][str(base / 'check_versions.py')], str(lock.fileno()), unit, cookie, str(proof_path)]
    record['argv'] = argv
    start = time.monotonic()
    record['initial_scope'] = scope_state()
    if record['initial_scope']['LoadState'] != 'not-found':
        raise RuntimeError('version scope already exists; preserve its owner')
    launch_attempted = True
    child = subprocess.Popen(argv, env=env, pass_fds=(lock.fileno(),), stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
    try:
        stdout, stderr = child.communicate(timeout=240)
        record['timed_out'] = False
    except subprocess.TimeoutExpired:
        record['timed_out'] = True
        os.killpg(child.pid, signal.SIGTERM)
        try:
            stdout, stderr = child.communicate(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            stdout, stderr = child.communicate(timeout=10)
    (outdir / 'stdout.txt').write_bytes(stdout)
    (outdir / 'stderr.txt').write_bytes(stderr)
    record.update(exit=child.returncode, wall_seconds=time.monotonic() - start)
    if child.returncode != 0 or record['timed_out']:
        raise RuntimeError('version phase failed; retain attempt')
    record['status'] = 'PASS_VERSION_PHASE_ONLY;BUILD_NATIVE_UI_GATES_OPEN'
except BaseException as error:
    record.update(status='FAIL_PRESERVED', error=repr(error))
    raise
finally:
    try:
        if child is not None and child.poll() is None:
            os.killpg(child.pid, signal.SIGTERM)
            try:
                child.communicate(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid, signal.SIGKILL)
                child.communicate(timeout=10)
    except Exception as error:
        record.update(status='FAIL_PRESERVED', launcher_cleanup_error=repr(error))
    if launch_attempted:
        try:
            record['scope_cleanup'] = cleanup_scope()
        except Exception as error:
            record.update(status='FAIL_PRESERVED', scope_cleanup_error=repr(error))
    try:
        record['inputs_after'] = snapshot()
        record['inputs_unchanged'] = record['inputs_before'] == record['inputs_after']
    except OSError as error:
        record.update(status='FAIL_PRESERVED', inputs_unchanged=False, inputs_after_error=repr(error))
    if not record['inputs_unchanged']:
        record.update(status='FAIL_PRESERVED', error='selected input drift')
    (outdir / 'receipt.json').write_text(json.dumps(record, indent=2) + '\n')
    if not record['inputs_unchanged'] or record.get('scope_cleanup_error') or record.get('launcher_cleanup_error'):
        raise RuntimeError('version input or owned-scope teardown failure')
print(json.dumps({'status': record['status'], 'exit': record['exit'], 'inputs_unchanged': record['inputs_unchanged']}))
