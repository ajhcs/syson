"""Coordinator launcher; owns the lock and exact scope through cleanup."""
from pathlib import Path
import fcntl, hashlib, importlib.util, json, os, pwd, signal, subprocess, sys, selectors, time


def require(ok, message):
    if not ok:
        raise ValueError(message)



LOG_BYTES = 960 * 1024


def capture(child, stdout, stderr, seconds=900):
    deadline, written = time.monotonic() + seconds, 0
    with selectors.DefaultSelector() as streams:
        streams.register(child.stdout, selectors.EVENT_READ, stdout)
        streams.register(child.stderr, selectors.EVENT_READ, stderr)
        while streams.get_map():
            require(time.monotonic() < deadline, 'controller timeout')
            for key, _ in streams.select(timeout=min(1, max(0, deadline - time.monotonic()))):
                raw = os.read(key.fd, min(65536, LOG_BYTES - written + 1))
                if not raw:
                    streams.unregister(key.fileobj)
                    key.fileobj.close()
                    continue
                available = LOG_BYTES - written
                key.data.write(raw[:available])
                written += min(len(raw), available)
                require(len(raw) <= available, 'controller log budget')
    child.wait(timeout=max(0.01, deadline - time.monotonic()))


def main():
    path, expected = Path(sys.argv[1]).resolve(), sys.argv[2]
    require(hashlib.sha256(path.read_bytes()).hexdigest() == expected, 'spec changed')
    spec = json.loads(path.read_bytes()); out = Path(spec['out'])
    require(hashlib.sha256(Path(__file__).read_bytes()).hexdigest() ==
            spec['input_sha256'][str(Path(__file__).resolve())], 'launcher differs from frozen admission')
    require(out.parent.resolve() == out.parent and out.is_relative_to('/workspace/build/builderbob')
            and not os.path.lexists(out), 'preserve existing/noncanonical output')
    home = pwd.getpwuid(os.getuid())
    require(home.pw_name == 'builderbob' and os.environ['HOME'] == home.pw_dir, 'operator changed')
    check = subprocess.run(['/usr/local/sbin/bob-admin', 'workspace-status'], capture_output=True, timeout=15)
    require(check.returncode == 0 and json.loads(check.stdout)['outcome'] == 'ok', 'Samsung unavailable')
    memory = int(next(x.split()[1] for x in Path('/proc/meminfo').read_text().splitlines() if x.startswith('MemAvailable:')))
    disk = os.statvfs(out.parent)
    require(memory >= 10 * 1024**2 and disk.f_bavail * disk.f_frsize >= 30 * 1024**3, 'fresh headroom unavailable')
    require(hashlib.sha256(Path(spec['ownership_helper']).read_bytes()).hexdigest() ==
            'ff18c5d2e6af6a8b21a4d59129401d9c9274f4d4727e2e448eae331c72f6b92d', 'ownership provider changed')
    loader = importlib.util.spec_from_file_location('owned', spec['ownership_helper'])
    owned = importlib.util.module_from_spec(loader); loader.loader.exec_module(owned)
    unit, cookie = owned.new_attempt('syson-b33-closure-')
    env = {'HOME': home.pw_dir, 'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8', 'LC_ALL': 'C.UTF-8',
           'XDG_RUNTIME_DIR': '/run/user/' + str(os.getuid()),
           'DBUS_SESSION_BUS_ADDRESS': 'unix:path=/run/user/' + str(os.getuid()) + '/bus',
           'SYSON_BOOTSTRAP_SOURCE_SHA256': spec['source_sha256']}
    require(owned.state(unit, env)['LoadState'] == 'not-found', 'preserve existing scope')
    out.mkdir(mode=0o700); child = None
    record = {'status': 'FAIL_PRESERVED', 'unit': unit, 'cookie': cookie, 'memory_kib': memory,
              'disk_bytes': disk.f_bavail * disk.f_frsize}
    def interrupted(signum, frame):
        raise RuntimeError('controller interrupted')
    signal.signal(signal.SIGTERM, interrupted)
    with open(spec['compiler_lock']) as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        try:
            args = ['/usr/bin/systemd-run', '--user', '--scope', '--quiet', '--collect', '--unit=' + unit,
                    '-p', 'CPUQuota=100%', '-p', 'MemoryHigh=512M', '-p', 'MemoryMax=1G',
                    '-p', 'MemorySwapMax=0', '-p', 'TasksMax=32', '-p', 'RuntimeMaxSec=1020s',
                    '/usr/bin/python3', '-I', '-B', spec['source'], str(path), expected, str(lock.fileno()), unit, cookie]
            with out.with_suffix('.controller.stdout').open('xb') as stdout, out.with_suffix('.controller.stderr').open('xb') as stderr:
                child = subprocess.Popen(args, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, pass_fds=(lock.fileno(),))
                capture(child, stdout, stderr)
            require(child.returncode == 0 and json.loads((out / 'receipt.json').read_bytes())['status'] ==
                    'PASS_PASSIVE_ACQUISITION_ONLY', 'passive body failed')
            record['status'] = 'PASS_PASSIVE_ACQUISITION_ONLY'
        except BaseException as error:
            record['error'] = repr(error)
        finally:
            if child is not None and child.poll() is None:
                child.terminate()
                try: child.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    child.kill(); child.wait(timeout=5)
            try:
                record['cleanup'] = owned.cleanup(unit, cookie, out / 'scope-proof.json', env)
            except BaseException as error:
                record.update(status='FAIL_PRESERVED', cleanup_error=repr(error))
            record['exit'] = None if child is None else child.returncode
            encoded = json.dumps(record, indent=2) + '\n'
            require(len(encoded.encode('utf-8')) <= 64 * 1024, 'controller metadata budget')
            out.with_suffix('.controller.json').write_text(encoded)
    require(record['status'] == 'PASS_PASSIVE_ACQUISITION_ONLY', 'passive controller failed')


if __name__ == '__main__':
    main()
