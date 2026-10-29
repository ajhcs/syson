"""Verify complete retained distributions, then run only five fixed version handlers."""
import argparse
from datetime import datetime, timezone
import fcntl
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import pwd
import re
import resource
import signal
import stat
import subprocess
import sys
import time

BASE = Path(__file__).resolve().parent
PROPOSAL_SHA256 = 'ac7393022ef1bc89320bbd2d580f6d20a3c0bcf230b32cedffbcafc812c76ed9'
OUTPUT_CAP = 65536


class Refusal(RuntimeError):
    pass


def require(ok, message):
    if not ok:
        raise Refusal(message)


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def bound_inputs(proposal):
    for name, expected in proposal['bindings'].items():
        require(digest(name) == expected, 'bound public input changed: ' + name)
    require(not os.path.lexists('/etc/ld.so.preload'), 'unexpected loader preload file')
    return dict(proposal['bindings'])


def verify_tree(spec):
    root = Path(spec['root'])
    require(root.is_dir() and not root.is_symlink() and root.stat().st_uid == os.getuid(),
            'distribution root ownership/type changed')
    rows = json.loads(Path(spec['manifest']).read_bytes())
    require(isinstance(rows, list) and len(rows) == spec['entries'], 'manifest record count changed')
    expected = set()
    for row in rows:
        name = row['path']
        require(isinstance(name, str), 'manifest path must be text')
        relative = PurePosixPath(name)
        require(isinstance(name, str) and str(relative) == name and relative.parts and
                not relative.is_absolute() and '..' not in relative.parts and name not in expected,
                'manifest path malformed or repeated')
        expected.add(name)
        path = root / name
        observed = path.lstat()
        require(observed.st_uid == os.getuid() and stat.S_IMODE(observed.st_mode) == row['mode'] and
                not observed.st_mode & 0o7000, 'distribution ownership/mode changed')
        kind = row['kind']
        if kind == 'file':
            require(stat.S_ISREG(observed.st_mode) and observed.st_size == row['bytes'] and
                    digest(path) == row['sha256'], 'distribution file bytes changed')
        elif kind == 'directory':
            require(stat.S_ISDIR(observed.st_mode), 'distribution directory changed')
        elif kind == 'symlink':
            require(stat.S_ISLNK(observed.st_mode), 'distribution symlink type changed')
            target = os.readlink(path)
            require(stat.S_ISLNK(observed.st_mode) and target == row['target'] and
                    hashlib.sha256(os.fsencode(target)).hexdigest() == row['sha256'] and
                    row['target_exists'] is True and path.resolve(strict=True).is_relative_to(root.resolve()),
                    'distribution symlink changed or escaped')
        else:
            raise Refusal('unsupported manifest kind')
    actual = set()
    for directory, directories, files in os.walk(root, followlinks=False):
        for name in directories + files:
            actual.add(str((Path(directory) / name).relative_to(root)))
    require(actual == expected, 'distribution tree has missing or extra entries')
    return {'id': spec['id'], 'entries': len(rows), 'manifest_sha256': digest(spec['manifest']),
            'regular_bytes': sum(row.get('bytes', 0) for row in rows)}


def clean_environment(proposal, run):
    jdk, maven, node = (proposal['bundles'][i]['root'] for i in range(3))
    home = os.environ['HOME']
    require(home == pwd.getpwuid(os.getuid()).pw_dir, 'actual operator HOME changed')
    temporary = str(run / 'tmp')
    java = ['-Xmx256m', '-XX:ActiveProcessorCount=2', '-XX:-UsePerfData',
            '-Duser.home=' + str(run / 'java-home'), '-Djava.io.tmpdir=' + temporary]
    environment = {
        'HOME': home, 'PATH': jdk + '/bin:' + maven + '/bin:' + node + '/bin:/usr/bin:/bin',
        'LANG': 'C.UTF-8', 'LC_ALL': 'C.UTF-8', 'TMPDIR': temporary,
        'JAVA_HOME': jdk, 'MAVEN_SKIP_RC': '1', 'MAVEN_BASEDIR': str(run / 'cwd'),
        'MAVEN_OPTS': ' '.join(java + ['-Dstyle.color=never']),
        'NODE_DISABLE_COMPILE_CACHE': '1', 'UV_THREADPOOL_SIZE': '2', 'NO_COLOR': '1',
    }
    return environment, java


def commands(proposal, run):
    jdk, maven, node = (proposal['bundles'][i]['root'] for i in range(3))
    _, java = clean_environment(proposal, run)
    settings = str(BASE / 'configs/maven-public.xml')
    npm = node + '/lib/node_modules/npm/bin/npm-cli.js'
    return [
        ('java', [jdk + '/bin/java', *java, '-version']),
        ('javac', [jdk + '/bin/javac', *['-J' + option for option in java], '-version']),
        ('maven', [maven + '/bin/mvn', '--offline', '--settings', settings,
                   '--global-settings', settings, '--version']),
        ('node', [node + '/bin/node', '--version']),
        ('npm', [node + '/bin/node', '--max-old-space-size=256', npm,
                 '--userconfig=' + str(BASE / 'configs/npm-user.rc'),
                 '--globalconfig=' + str(BASE / 'configs/npm-global.rc'),
                 '--prefix=' + str(run / 'cwd'), '--cache=' + str(run / 'npm-cache'),
                 '--offline', '--ignore-scripts', '--update-notifier=false', '--audit=false',
                 '--fund=false', '--logs-max=0', '--timing=false', '--version']),
    ]


def check_version(name, out, err):
    text = (out + err).decode('utf-8', 'strict').strip()
    if name == 'java':
        require(re.search(r'^openjdk version "21\.0\.12\.1" ', text, re.MULTILINE) and
                'Temurin-21.0.12.1+1' in text and '(build 21.0.12.1+1-LTS)' in text,
                'unexpected Java runtime version')
    elif name == 'maven':
        require(re.search(r'^Apache Maven 3\.9\.16(?: |$)', text, re.MULTILINE) and
                re.search(r'^Java version: 21\.0\.12\.1,', text, re.MULTILINE),
                'unexpected Maven/Java version')
    else:
        exact = {'javac': 'javac 21.0.12.1', 'node': 'v24.20.0', 'npm': '11.19.0'}
        require(text == exact[name], 'unexpected exact tool version')


def child_limits():
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    resource.setrlimit(resource.RLIMIT_FSIZE, (OUTPUT_CAP, OUTPUT_CAP))
    resource.setrlimit(resource.RLIMIT_NOFILE, (128, 128))


def stop_group(pid):
    try:
        os.killpg(pid, signal.SIGKILL)
        return True
    except ProcessLookupError:
        return False


def run_one(name, argv, environment, run):
    if name == 'maven':
        environment = dict(environment, JDK_JAVA_OPTIONS=environment['MAVEN_OPTS'])
    started = time.monotonic()
    paths = [run / (name + '.stdout'), run / (name + '.stderr')]
    child = None
    with paths[0].open('xb') as out, paths[1].open('xb') as err:
        try:
            child = subprocess.Popen(argv, cwd=run / 'cwd', env=environment, stdin=subprocess.DEVNULL,
                                     stdout=out, stderr=err, close_fds=True, start_new_session=True,
                                     preexec_fn=child_limits)
            code = child.wait(timeout=20)
            require(not stop_group(child.pid), 'unexpected remaining version descendants')
        finally:
            if child is not None and child.poll() is None:
                stop_group(child.pid)
                child.wait(timeout=5)
    raw = [path.read_bytes() for path in paths]
    require(all(len(value) <= OUTPUT_CAP for value in raw), 'version output cap exceeded')
    require(code == 0, 'version handler returned nonzero: ' + name)
    check_version(name, *raw)
    return {'name': name, 'argv': argv, 'environment': environment, 'exit': code,
            'wall_seconds': time.monotonic() - started,
            'outputs': {str(path): {'bytes': len(value), 'sha256': hashlib.sha256(value).hexdigest()}
                        for path, value in zip(paths, raw)}}


def preflight(proposal, descriptor):
    require(pwd.getpwuid(os.getuid()).pw_name == 'builderbob', 'trusted builderbob identity required')
    lock = Path(proposal['lock']).stat()
    inherited = os.fstat(descriptor)
    require((lock.st_dev, lock.st_ino) == (inherited.st_dev, inherited.st_ino), 'wrong inherited lock FD')
    fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
    line = Path('/proc/self/cgroup').read_text().strip()
    require(line.startswith('0::/') and line.endswith('/syson-c01-versions-01.scope'), 'wrong version scope')
    group = Path('/sys/fs/cgroup') / line.split(':', 2)[2].lstrip('/')
    expected = {'cpu.max': '200000 100000', 'memory.high': '4294967296',
                'memory.max': '6442450944', 'memory.swap.max': '0', 'pids.max': '128'}
    observed = {name: (group / name).read_text().strip() for name in expected}
    require(observed == expected, 'actual version cgroup controls mismatch')
    return {'cgroup': line, 'controls': observed, 'network_namespace': os.readlink('/proc/self/ns/net')}


def interrupted(signum, frame):
    raise Refusal('interrupted by signal ' + str(signum))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--lock-fd', type=int, required=True)
    args = parser.parse_args()
    signal.signal(signal.SIGTERM, interrupted)
    signal.signal(signal.SIGINT, interrupted)
    raw = (BASE / 'proposal.json').read_bytes()
    require(hashlib.sha256(raw).hexdigest() == PROPOSAL_SHA256, 'version proposal changed')
    proposal = json.loads(raw)
    controls = preflight(proposal, args.lock_fd)
    run = Path(proposal['fresh_run_root'])
    require(run.parent.resolve().is_relative_to('/workspace') and
            run.parent.stat().st_dev == Path('/workspace').stat().st_dev and not os.path.lexists(run),
            'fresh Samsung version output destination required')
    run.mkdir(mode=0o700)
    state = {'status': 'FAILED_VERSION_ONLY_ATTEMPT_PRESERVED', 'commands': [], 'controls': controls,
             'proposal_sha256': PROPOSAL_SHA256, 'runner_sha256': digest(Path(__file__)),
             'started_utc': datetime.now(timezone.utc).isoformat(), 'scope': proposal['claim_boundary']}
    code = 1
    try:
        state['inputs_before'] = bound_inputs(proposal)
        state['trees_before'] = [verify_tree(spec) for spec in proposal['bundles']]
        for name in ['cwd', 'tmp', 'java-home', 'npm-cache']:
            (run / name).mkdir(mode=0o700)
        environment, _ = clean_environment(proposal, run)
        state['environment'] = environment
        for name, argv in commands(proposal, run):
            state['current_command'] = {'name': name, 'argv': argv}
            state['commands'].append(run_one(name, argv, environment, run))
        state['inputs_after'] = bound_inputs(proposal)
        state['trees_after'] = [verify_tree(spec) for spec in proposal['bundles']]
        require(state['trees_before'] == state['trees_after'], 'distribution verification drift')
        require(not any((run / 'cwd').iterdir()), 'version handler changed neutral cwd')
        code = 0
        state['status'] = 'PASS_FIXED_VERSION_HANDLERS_ONLY;BUILD_NATIVE_UI_GATES_OPEN'
    except Exception as error:
        state['error_type'] = type(error).__name__
        state['error'] = str(error) if isinstance(error, Refusal) else 'Operational failure; preserve raw outputs.'
    finally:
        state['finished_utc'] = datetime.now(timezone.utc).isoformat()
        with (run / 'receipt.json').open('x') as target:
            json.dump(state, target, indent=2, allow_nan=False)
            target.write('\n')
    print(json.dumps({'status': state['status'], 'receipt': str(run / 'receipt.json')}))
    return code


if __name__ == '__main__':
    sys.exit(main())
