#!/usr/bin/env python3
"""Coordinator-only serial offline checks for first Rust parity plan batch."""
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

unit, run = sys.argv[1:3]
stage = sys.argv[3] if len(sys.argv) > 3 else "all"
configs = {'C01c': ('rp-c01c-graphql-contract','rust/tools/native-graphql-contract'), 'C01d': ('rp-c01d-persistence-contract','rust/tools/native-db-contract'), 'C03b': ('rp-c03b-project-preparation','rust/crates/syson-project-preparation'), 'COMPOSE': ('rp-four-migrations-base','rust/crates/syson-migrations')}
folder, relative_crate = configs[unit]
name = Path(relative_crate).name
if '/' in run: raise SystemExit('Invalid run')
root = Path('/workspace/worktrees/builderbob/syson') / folder
crate = root / relative_crate
base = subprocess.check_output(['git','-C',str(root),'rev-parse','HEAD']).decode().strip()
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
extras = sorted((root / 'porting/rust-parity-evidence' / unit).rglob('*')) if (root / 'porting/rust-parity-evidence' / unit).exists() else []
extras = [p for p in extras if p.is_file() and p.suffix in {'.json','.graphql','.sql'}]
if unit == 'COMPOSE':
    extras = sorted(p for family in ['B25a','B25b','B25c','B25d'] for p in (root/'porting/rust-parity-evidence'/family).rglob('*.json'))
changed = git('diff', '--name-only').decode().splitlines()
require(all(p.startswith(relative_crate+'/') or (unit == 'NAV' and p == 'rust/browser-tests/inspector_accessibility.cjs') for p in changed), 'Unexpected tracked dependency edits: '+repr(changed))

def freeze():
    result = {str(p.relative_to(root)): sha(p.read_bytes())
            for source in sources for p in sorted(source.rglob('*'))
            if p.is_file() and 'target' not in p.relative_to(source).parts}
    result.update({str(p.relative_to(root)): sha(p.read_bytes()) for p in extras})
    return result

out = Path('/workspace/build/builderbob/syson-rust-parity-batch4-20261026/captures') / run
out.mkdir(parents=True, exist_ok=False)
(out / 'executed-runner.py').write_bytes(Path(__file__).read_bytes())
for source in sorted(crate.rglob('*')):
    if source.is_file() and 'target' not in source.relative_to(crate).parts:
        retained = out / 'original-inputs' / source.relative_to(root)
        retained.parent.mkdir(parents=True, exist_ok=True)
        retained.write_bytes(source.read_bytes())
for source in extras:
    retained = out / 'original-inputs' / source.relative_to(root)
    retained.parent.mkdir(parents=True, exist_ok=True)
    retained.write_bytes(source.read_bytes())
build = Path('/workspace/build/builderbob/syson-audit-20261026/build')
build.mkdir(parents=True, exist_ok=True)
lock = (build / 'compiler.lock').open('a')
fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
scratch = build / 'scratch' / run
scratch.mkdir(parents=True, exist_ok=False)
env = os.environ.copy()
env.update(CARGO_BUILD_JOBS='1', RUST_TEST_THREADS='1', CARGO_INCREMENTAL='0',
           CARGO_TARGET_DIR=str(build / 'target'), TMPDIR=str(scratch))
if unit == 'C01d':
    fixture_root = scratch/'catalog-fixtures'
    fixture_root.mkdir()
    env['SYSON_DB_CONTRACT_TEST_ROOT'] = str(fixture_root)
if unit == 'C03a':
    fixture_root = scratch/'git-fixtures'
    fixture_root.mkdir()
    env['SYSON_COMPOSITION_TEST_ROOT'] = str(fixture_root)
manifest = str(crate / 'Cargo.toml')
commands = [['cargo','fmt','--manifest-path',manifest,'--','--check'],['cargo','test','--offline','--locked','--manifest-path',manifest,'--','--test-threads=1'],['cargo','clippy','--offline','--locked','--manifest-path',manifest,'--all-targets','--','-D','warnings']]
if stage == 'inspection_lint':
    require(unit == 'C01b', 'Invalid focused stage')
    commands[1] = ['cargo','test','--offline','--locked','--manifest-path',manifest,'--test','inspection','file_symlink_containment_and_stricter_classpath_symlink_rules_are_distinct','--','--exact','--test-threads=1']
if unit == 'B25S' and (crate/'examples/path_memory.rs').exists():
    commands.append(['cargo','build','--release','--offline','--locked','--manifest-path',manifest,'--example','path_memory'])
if stage == 'build':
    commands = commands[-1:]
if unit == 'COMPOSE':
    frozen = freeze()
    require(all(sha(git('show', base+':'+path)) == digest for path,digest in frozen.items()), 'Composition working inputs differ from committed candidate')
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
    if unit in ['UI','NAV','VP'] and receipt['status']=='PASS':
        artifact=build/'target/wasm32-unknown-unknown/release/syson_web.wasm'
        receipt['wasm_sha256']=sha(artifact.read_bytes())
        (out/'syson_web.wasm').write_bytes(artifact.read_bytes())
    receipt['source_unchanged'] = receipt['source_before'] == receipt['source_after']
    if not receipt['source_unchanged']:
        receipt['status'] = 'SOURCE_DRIFT'
    (out / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({key: receipt[key] for key in ['unit', 'run', 'status', 'source_unchanged']}))
print(out / 'receipt.json')
sys.exit(0 if receipt['status'] == 'PASS' else 1)
