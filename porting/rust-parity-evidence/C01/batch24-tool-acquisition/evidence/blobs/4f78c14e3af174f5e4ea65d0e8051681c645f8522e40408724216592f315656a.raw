from pathlib import Path
import fcntl, hashlib, json, os, signal, subprocess, time
b = Path(__file__).resolve().parents[1]
a = Path(__file__).parent
o = b / 'c01-resource-admission-02'
o.mkdir(mode=0o700)
child = None
def interrupted(signum, frame):
    raise RuntimeError('Wrapper interrupted by signal ' + str(signum))
signal.signal(signal.SIGTERM, interrupted)
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
inputs = [a / 'phase1-acquire-attempt02.py', a / 'proposal-attempt02.json', a / 'phase1-entry-attempt02.py',
          Path(__file__), Path('/usr/bin/python3'), Path('/usr/bin/systemd-run')]
record = {'status': 'PREPARING', 'inputs_before': {str(p): sha(p) for p in inputs},
          'scope': 'Pinned tool archives/metadata only; no acquired code, versions, hooks, build or native application.'}
(o / 'executed-wrapper.py').write_bytes(Path(__file__).read_bytes())
lockpath = Path('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock')
lock = lockpath.open()
fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
env = {'PATH': '/usr/bin:/bin', 'HOME': os.environ['HOME'], 'LANG': 'C.UTF-8', 'LC_ALL': 'C.UTF-8',
       'XDG_RUNTIME_DIR': os.environ.get('XDG_RUNTIME_DIR', '/run/user/' + str(os.getuid()))}
try:
    mount = subprocess.run(['/usr/local/sbin/bob-admin', 'workspace-status'], env=env,
                           capture_output=True, timeout=15)
    (o / 'workspace-status.json').write_bytes(mount.stdout)
    if mount.returncode != 0 or json.loads(mount.stdout).get('outcome') != 'ok':
        raise RuntimeError('Samsung workspace identity not verified')
    mem = {line.split(':')[0]: int(line.split()[1]) for line in Path('/proc/meminfo').read_text().splitlines()}
    disk = os.statvfs('/workspace')
    record['fresh_headroom'] = {'mem_available_kib': mem['MemAvailable'],
                               'swap_free_kib': mem['SwapFree'],
                               'workspace_available_bytes': disk.f_bavail * disk.f_frsize}
    if mem['MemAvailable'] < 10 * 1024**2 or disk.f_bavail * disk.f_frsize < 30 * 1024**3:
        raise RuntimeError('Insufficient fresh headroom')
    source = Path('/workspace/worktrees/builderbob/syson/c01-native-target-6ba2-01')
    source_manifest = json.loads((a / 'target-source-manifest.json').read_bytes())
    for item in source_manifest['files']:
        path = source / item['path']
        raw = os.fsencode(os.readlink(path)) if path.is_symlink() else path.read_bytes()
        if hashlib.sha256(raw).hexdigest() != item['sha256']:
            raise RuntimeError('Target source changed')
    revision = subprocess.check_output(['/usr/bin/git', 'rev-parse', 'HEAD'], cwd=source, env=env).decode().strip()
    status = subprocess.check_output(['/usr/bin/git', 'status', '--porcelain'], cwd=source, env=env)
    if revision != '6ba2ac64e6b16ee2b05752ad55395eebb87d4910' or status:
        raise RuntimeError('Target checkout identity/status changed')
    record['source_manifest_sha256'] = sha(a / 'target-source-manifest.json')
    record['source_files_verified'] = len(source_manifest['files'])
    argv = ['/usr/bin/systemd-run', '--user', '--scope', '--quiet', '--collect',
            '--unit=syson-c01-build-02-tools.scope', '-p', 'CPUQuota=200%', '-p', 'MemoryHigh=4G',
            '-p', 'MemoryMax=6G', '-p', 'MemorySwapMax=0',
            '-p', 'TasksMax=128', '-p', 'RuntimeMaxSec=3600s', '/usr/bin/python3', '-I', '-B',
            str(a / 'phase1-entry-attempt02.py'), str(lock.fileno())]
    record['argv'] = argv
    record['started_monotonic'] = time.monotonic()
    child = subprocess.Popen(argv, env=env, pass_fds=(lock.fileno(),), stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE, start_new_session=True)
    try:
        out, err = child.communicate(timeout=1200)
        record['timed_out'] = False
    except subprocess.TimeoutExpired:
        record['timed_out'] = True
        os.killpg(child.pid, signal.SIGTERM)
        try:
            out, err = child.communicate(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            out, err = child.communicate(timeout=10)
    (o / 'stdout.txt').write_bytes(out)
    (o / 'stderr.txt').write_bytes(err)
    record['exit'] = child.returncode
    record['wall_seconds'] = time.monotonic() - record['started_monotonic']
    if child.returncode != 0 or record['timed_out']:
        raise RuntimeError('Acquisition failed; preserve task and partial files')
    record['status'] = 'PASS_ARCHIVE_ACQUISITION_ONLY_NO_ACQUIRED_CODE_EXECUTED'
except BaseException as error:
    record.update(status='FAIL_PRESERVED', error_type=type(error).__name__, error=str(error))
    raise
finally:
    if child is not None and child.poll() is None:
        os.killpg(child.pid, signal.SIGTERM)
        try:
            child.communicate(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            child.communicate(timeout=10)
    record['inputs_after'] = {str(p): sha(p) for p in inputs}
    record['inputs_unchanged'] = record['inputs_before'] == record['inputs_after']
    (o / 'receipt.json').write_text(json.dumps(record, indent=2) + '\n')
    if not record['inputs_unchanged']:
        raise RuntimeError('Input drift')
