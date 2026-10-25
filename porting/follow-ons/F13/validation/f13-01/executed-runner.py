#!/usr/bin/env python3
"""Coordinator-only frozen offline checks for batch-seven adapters and property inspector."""
from datetime import datetime, timezone
import fcntl
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tomllib

unit, run = sys.argv[1:]
configs = {'F13': ('f13-local-linker', 'syson-linker'),
           'F46': ('f46-semantic-names', 'syson-semantic-names'),
           'F36': ('f36-inspector-accessibility', 'syson-web')}
folder, name = configs[unit]
if not run.startswith(unit.lower() + '-') or '/' in run:
    raise SystemExit('Invalid capture ID')
root = Path('/workspace/worktrees/builderbob/syson') / folder
crate = root / 'rust/crates' / name
base = {'F13':'19cc940e7a0ea0c909f140fc0bc79d19220cddd2','F46':'78aa06f55abc10d6f60280a9d9d301f0ad6b66ef','F36':'797b1c194b1cd46f0152b257d337e6cf2983041c'}[unit]
coordinator = Path(__file__).parent

def sha(data):
    return hashlib.sha256(data).hexdigest()

def require(value, reason):
    if not value:
        raise RuntimeError(reason)

def git(*args):
    return subprocess.check_output(['git', '-C', str(root), *args])

def now():
    return datetime.now(timezone.utc).isoformat()

seal = json.loads((coordinator / 'manifest-seals.json').read_text())[unit]
for relative, digest in seal.items():
    require(sha((root / relative).read_bytes()) == digest, 'Manifest/lock seal changed: ' + relative)
sources = []
def closure(source):
    if source in sources:
        return
    sources.append(source)
    manifest = tomllib.loads((source / 'Cargo.toml').read_text())
    require(not (source / 'build.rs').exists() and not manifest.get('package', {}).get('build'), 'Unreviewed build script')
    if source != crate:
        require((source / 'Cargo.toml').read_bytes() == git('show', base + ':' + str((source / 'Cargo.toml').relative_to(root))), 'Parent manifest changed')
    for section in ['dependencies', 'dev-dependencies', 'build-dependencies']:
        for dep in manifest.get(section, {}).values():
            if isinstance(dep, dict) and 'path' in dep:
                closure((source / dep['path']).resolve())
closure(crate)
extras = [root / 'rust/browser-tests' / p for p in ['layout.cjs','authoring.cjs','scalars.cjs','inspector_accessibility.cjs']] if unit == 'F36' else []
changed = git('diff', '--name-only').decode().splitlines()
allowed = {'rust/crates/syson-web/src/lib.rs','rust/crates/syson-web/web/loader.js'} if unit == 'F36' else set()
require(set(changed) <= allowed, 'Unexpected tracked edits: ' + repr(changed))

def freeze():
    result = {str(p.relative_to(root)): sha(p.read_bytes())
            for source in sources for p in sorted(source.rglob('*'))
            if p.is_file() and 'target' not in p.relative_to(source).parts}
    result.update({str(p.relative_to(root)): sha(p.read_bytes()) for p in extras})
    return result

out = root / 'target/batch7-checks' / run
out.mkdir(parents=True, exist_ok=False)
build = Path('/workspace/build/builderbob/syson-batch7-20261025')
build.mkdir(parents=True, exist_ok=True)
lock = (build / 'compiler.lock').open('a')
fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
scratch = build / 'scratch' / run
scratch.mkdir(parents=True, exist_ok=False)
env = os.environ.copy()
env.update(CARGO_BUILD_JOBS='1', RUST_TEST_THREADS='1', CARGO_INCREMENTAL='0',
           CARGO_TARGET_DIR=str(build / 'target'), TMPDIR=str(scratch))
manifest = str(crate / 'Cargo.toml')
commands = [
    ['cargo', 'fmt', '--manifest-path', manifest, '--', '--check'],
    ['cargo', 'test', '--offline', '--locked', '--manifest-path', manifest, '--', '--test-threads=1'],
    ['cargo', 'clippy', '--offline', '--locked', '--manifest-path', manifest, '--all-targets', '--', '-D', 'warnings'],
]
if unit == 'F36':
    commands.append(['cargo','build','--release','--target','wasm32-unknown-unknown','--offline','--locked','--manifest-path',manifest])
receipt = dict(unit=unit, run=run, status='FAIL_SETUP', started_utc=now(), source_before=freeze(),
               source_base=git('rev-parse', 'HEAD').decode().strip(),
               runner_sha256=sha(Path(__file__).read_bytes()), commands=[],
               execution='COORDINATOR_SERIAL_ONE_JOB_OFFLINE_LOCKED', upstream_runtime='NOT_RUN')
try:
    for index, argv in enumerate(commands):
        entry = dict(argv=argv, started_utc=now())
        child = subprocess.Popen(argv, cwd=root, env=env, stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE, start_new_session=True)
        try:
            stdout, stderr = child.communicate(timeout=180)
            code = child.returncode
        except subprocess.TimeoutExpired:
            try:
                os.killpg(child.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                stdout, stderr = child.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                try:
                    stdout, stderr = child.communicate(timeout=3)
                except subprocess.TimeoutExpired as error:
                    stdout, stderr = error.stdout or b'', error.stderr or b''
                    child.stdout.close()
                    child.stderr.close()
                    entry['owned_pipes_remained_open'] = True
            code = 124
            entry['timeout_owned_process_group'] = child.pid
        (out / f'{index:02d}-stdout.txt').write_bytes(stdout)
        (out / f'{index:02d}-stderr.txt').write_bytes(stderr)
        entry.update(completed_utc=now(), exit_code=code,
                     stdout_sha256=sha(stdout), stderr_sha256=sha(stderr))
        receipt['commands'].append(entry)
        if code:
            receipt['status'] = 'FAIL'
            break
    else:
        receipt['status'] = 'PASS'
finally:
    receipt.update(completed_utc=now(), source_after=freeze())
    if unit == 'F36' and receipt.get('status') == 'PASS':
        wasm = (build / 'target/wasm32-unknown-unknown/release/syson_web.wasm').read_bytes()
        receipt['wasm_sha256'] = sha(wasm)
        mirror = root / 'target/batch7-wasm/syson_web.wasm'
        mirror.parent.mkdir(parents=True, exist_ok=True)
        mirror.write_bytes(wasm)
    receipt['source_unchanged'] = receipt['source_before'] == receipt['source_after']
    if not receipt['source_unchanged']:
        receipt['status'] = 'SOURCE_DRIFT'
    (out / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({key: receipt[key] for key in ['unit', 'run', 'status', 'source_unchanged']}))
print(out / 'receipt.json')
sys.exit(0 if receipt['status'] == 'PASS' else 1)
