"""Fixed raw observations only; root owns execution, ranges and acceptance review."""
import contextlib
import fcntl
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tomllib

base = Path(__file__).resolve().parent
work = Path('/workspace/build/builderbob/syson-batch27-20261029')
output = work / 'u11b-search-profile-01'
target = work / 'target-u11b-search-profile-01'
source = base / 'profile.rs'
allocator_source = base.parents[1] / 'C09/batch26-routing-admission-profile/u11a_profile.rs'
workspace = Path('/workspace/worktrees/builderbob/syson/batch27-routing-search/rust/crates')
tool = Path('/workspace/cache/builderbob/global-tools/rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin')
cargo_home = Path('/workspace/cache/builderbob/global-tools/cargo')
if os.path.lexists(output) or os.path.lexists(target):
    raise RuntimeError('Preserve existing profile attempt or target')
os.umask(0o077)
output.mkdir(mode=0o700)
files = {Path(__file__), Path(sys.executable), source, allocator_source,
         tool / 'cargo', tool / 'rustc', tool / 'rustfmt'}
files.update(base.glob('*.py'))
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
snapshot = lambda: {str(path): sha(path) for path in sorted(files)}
record = {'status': 'RUNNING', 'commands': [], 'scope': 'Fixed integer search observations only; native/UI/capacity/p95 acceptance OPEN',
          'schedule': 'Ten default groups, each repetitions 0-1 warm and 2-8 measured; four separate controls and five CLI refusals'}
child, lock = None, None
def interrupted(*unused):
    raise RuntimeError('Profile job interrupted')
signal.signal(signal.SIGTERM, interrupted)
def run(label, argv, expected=0):
    global child
    argv = list(map(str, argv))
    row = {'label': label, 'argv': argv, 'exit': None, 'expected_exit': expected}
    record['commands'].append(row)
    child = subprocess.Popen(argv, cwd=output, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
    failure = None
    try:
        stdout, stderr = child.communicate(timeout=180)
    except BaseException as error:
        (output / (label + '-stdout.txt')).write_bytes(getattr(error, 'output', None) or b'')
        (output / (label + '-stderr.txt')).write_bytes(getattr(error, 'stderr', None) or b'')
        with contextlib.suppress(ProcessLookupError):
            os.killpg(child.pid, signal.SIGKILL)
        stdout, stderr = child.communicate(timeout=10)
        failure = repr(error)
    (output / (label + '-stdout.txt')).write_bytes(stdout)
    (output / (label + '-stderr.txt')).write_bytes(stderr)
    row.update({'exit': child.returncode,
                               'interruption_or_timeout': failure,
                               'stdout_sha256': hashlib.sha256(stdout).hexdigest(), 'stderr_sha256': hashlib.sha256(stderr).hexdigest()})
    child = None
    if failure is not None or record['commands'][-1]['exit'] != expected:
        raise RuntimeError('Preserve failed ' + label)
    return stdout
try:
    files.update(path for name in ['syson-routing', 'syson-layout'] for path in (workspace / name).rglob('*') if path.is_file())
    env = {'HOME': os.environ['HOME'], 'PATH': str(tool) + ':/usr/bin:/bin', 'LANG': 'C.UTF-8', 'LC_ALL': 'C.UTF-8',
           'CARGO_HOME': str(cargo_home), 'RUSTUP_HOME': '/workspace/cache/builderbob/global-tools/rustup',
           'RUSTUP_TOOLCHAIN': '1.99.1-x86_64-unknown-linux-gnu', 'CARGO_BUILD_JOBS': '1',
           'CARGO_INCREMENTAL': '0', 'CARGO_TARGET_DIR': str(target), 'TMPDIR': str(output)}
    lock = Path('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock').open()
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    record['inputs_before'] = snapshot()
    if len(sys.argv) != 2 or sha(source) != sys.argv[1]:
        raise RuntimeError('Profile source is not the reviewed final bytes')
    allocator_raw = allocator_source.read_bytes()
    allocator_blob = hashlib.sha1(b'blob ' + str(len(allocator_raw)).encode() + b'\0' + allocator_raw).hexdigest()
    if sha(allocator_source) != '36127a02052ca203cead039dc2dae82e5e77118686ccf1d34c6d6f33bb1a0dda' or allocator_blob != '2f38d0e92afb48815ab813b8c887d760b887856d':
        raise RuntimeError('published allocator source/blob changed before compilation')
    block = b''.join(allocator_raw.splitlines(keepends=True)[11:95])
    block_sha = '1f927e4d0a780666081a13b4bdc4d2f3ffb94f220ac9bf39cdea8162734e3eb3'
    if len(block.splitlines()) != 84 or hashlib.sha256(block).hexdigest() != block_sha:
        raise RuntimeError('published allocator region changed before compilation')
    generated = output / 'counting_allocator.rs'
    with generated.open('xb') as stream:
        stream.write(block)
    files.add(generated)
    env['SYSON_COUNTING_ALLOCATOR_INCLUDE'] = str(generated)
    record['allocator_dependency'] = {'published_commit': '337433fb18a377e2f1a4c12fe5510b2d0798cfdb',
        'source_path': str(allocator_source), 'source_sha256': sha(allocator_source), 'git_blob': allocator_blob,
        'source_region_1_based_inclusive': [12, 95], 'region_sha256': block_sha,
        'generated_path': str(generated), 'generated_sha256': sha(generated)}
    manifest = output / 'Cargo.toml'
    manifest.write_text('[package]\nname="u11b-search-observer"\nversion="0.0.0"\nedition="2024"\n[lib]\npath="observer.rs"\n[dependencies]\nserde_json="=1.0.151"\nsyson-routing={path=' + json.dumps(str(workspace / 'syson-routing')) + '}\nsyson-layout={path=' + json.dumps(str(workspace / 'syson-layout')) + '}\n[workspace]\n')
    (output / 'observer.rs').write_bytes(b'')
    run('observer-lock', [tool / 'cargo', 'generate-lockfile', '--offline', '--manifest-path', manifest])
    files.update([manifest, output / 'Cargo.lock', output / 'observer.rs'])
    for package in tomllib.loads((output / 'Cargo.lock').read_text())['package']:
        if package.get('source', '').startswith('registry+'):
            roots = list((cargo_home / 'registry/src').glob('*/' + package['name'] + '-' + package['version']))
            if len(roots) != 1:
                raise RuntimeError('Offline dependency source selection is ambiguous')
            files.update(path for path in roots[0].rglob('*') if path.is_file())
    record['prepared_inputs_before'] = snapshot()
    run('actual-crates-release-build', [tool / 'cargo', 'build', '--release', '--offline', '--locked', '--manifest-path', manifest])
    libraries = {}
    for name in ['syson_routing', 'syson_layout', 'serde_json']:
        candidates = list((target / 'release/deps').glob('lib' + name + '-*.rlib'))
        if len(candidates) != 1:
            raise RuntimeError('Compiled dependency selection is ambiguous: ' + name)
        libraries[name] = candidates[0]
    record['compiled_inputs_before'] = {str(path): sha(path) for path in [*libraries.values(), allocator_source, generated]}
    executable = output / 'u11b-search-profile'
    argv = [tool / 'rustc', '--edition=2024', '-C', 'opt-level=3', source, '-L', 'dependency=' + str(target / 'release/deps'), '-o', executable]
    run('profile-build', argv + [part for name, path in libraries.items() for part in ['--extern', name + '=' + str(path)]])
    record['compiled_inputs_before'][str(executable)] = sha(executable)
    for count in [32, 64, 128, 256, 1024]:
        for density in [1, 2]:
            for repetition in range(9):
                data = json.loads(run(f'sample-{count}-{density}-{repetition}', [executable, count, density]))
                if data.get('expected_outcome') is not True:
                    raise RuntimeError('Unexpected search/admission observation')
    controls = [('pre-cancel', 32, '--cancel-before', 'SEARCH_CANCELLED'),
                ('late-cancel', 32, '--cancel-late', 'SEARCH_CANCELLED'),
                ('continued-quota', 32, '--continued-quota', 'SEARCH_WORK_REFUSAL'),
                ('default-grid', 64, '--grid', 'SEARCH_GRID_REFUSAL')]
    for label, count, flag, status in controls:
        data = json.loads(run(label, [executable, count, 1, flag]))
        if data.get('expected_outcome') is not True or data['status'] != status or data['search_called'] is not True:
            raise RuntimeError('Unexpected controlled search observation: ' + label)
    for index, values in enumerate([[], [31, 1], [64, 3], [64, 1, '--unknown'], [32, 1, '--cancel-before', '--cancel-before']]):
        if json.loads(run('invalid-' + str(index), [executable, *values], 2))['status'] != 'INVALID_ARGUMENTS':
            raise RuntimeError('CLI refusal mismatch')
    record['status'] = 'PASS_90_DEFAULT_SAMPLES_4_CONTROLS_5_INVALID;PARENT_ACCEPTANCE_OPEN'
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
        record['source_unchanged'] = (record['inputs_after'] == record.get('prepared_inputs_before', record.get('inputs_before', {}))
                                      and all(record['inputs_after'].get(key) == value for key, value in record.get('inputs_before', {}).items()))
        record['compiled_inputs_after'] = {path: sha(Path(path)) for path in record.get('compiled_inputs_before', {})}
        if not record['source_unchanged'] or record['compiled_inputs_after'] != record.get('compiled_inputs_before', {}):
            raise RuntimeError('Selected source/tool/manifest/library/binary drift')
    except BaseException as error:
        record.update(status='FAIL_TERMINAL', binding_or_cleanup_error=repr(error))
    (output / 'receipt.json').write_text(json.dumps(record, indent=2) + '\n')
    if record.get('binding_or_cleanup_error'):
        raise RuntimeError('Selected binding or cleanup failure')
