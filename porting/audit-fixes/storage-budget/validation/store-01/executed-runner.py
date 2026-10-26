#!/usr/bin/env python3
"""Coordinator-only frozen offline checks for batch-nine selected-model adapters."""
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
configs = {'LINK': ('audit-linker-limit','syson-linker'), 'LINE': ('audit-source-lines','syson-semantic-validation'), 'REC': ('audit-reconcile-index','syson-document-reconcile'), 'UI': ('audit-ui-recovery','syson-web'), 'NAV': ('audit-ui-navigation','syson-web'), 'BRIDGE': ('audit-import-json','syson-json-interchange'), 'STORE': ('audit-storage-budget','syson-storage')}
folder, name = configs[unit]
if '/' in run: raise SystemExit('Invalid run')
root = Path('/workspace/worktrees/builderbob/syson') / folder
crate = root / 'rust/crates' / name
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
    if source != crate or unit != 'BRIDGE':
        require((source / 'Cargo.toml').read_bytes() == git('show', base + ':' + str((source / 'Cargo.toml').relative_to(root))), 'Parent manifest changed')
    for section in ['dependencies', 'dev-dependencies', 'build-dependencies']:
        for dep in manifest.get(section, {}).values():
            if isinstance(dep, dict) and 'path' in dep:
                closure((source / dep['path']).resolve())
closure(crate)
extras = [root/'rust/browser-tests/inspector_accessibility.cjs'] if unit == 'NAV' else []
changed = git('diff', '--name-only').decode().splitlines()
require(all(p.startswith('rust/crates/'+name+'/') or (unit == 'NAV' and p == 'rust/browser-tests/inspector_accessibility.cjs') for p in changed), 'Unexpected tracked dependency edits: '+repr(changed))

def freeze():
    result = {str(p.relative_to(root)): sha(p.read_bytes())
            for source in sources for p in sorted(source.rglob('*'))
            if p.is_file() and 'target' not in p.relative_to(source).parts}
    result.update({str(p.relative_to(root)): sha(p.read_bytes()) for p in extras})
    return result

out = Path('/workspace/build/builderbob/syson-audit-20261026/captures') / run
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
manifest = str(crate / 'Cargo.toml')
manifest = str(crate / 'Cargo.toml')
if unit == 'REC':
    commands = [['cargo','fmt','--manifest-path',manifest,'--','--check'],['cargo','test','--offline','--locked','--manifest-path',manifest,'--','--test-threads=1'],['cargo','clippy','--offline','--locked','--manifest-path',manifest,'--all-targets','--','-D','warnings'],['/usr/bin/time','-v','cargo','run','--release','--offline','--locked','--manifest-path',manifest,'--example','reconcile_bench']]
else:
    commands = [['cargo','fmt','--manifest-path',manifest,'--','--check'],['cargo','test','--offline','--locked','--manifest-path',manifest,'--','--test-threads=1'],['cargo','clippy','--offline','--locked','--manifest-path',manifest,'--all-targets','--','-D','warnings']]
    if unit in ['UI','NAV']: commands.append(['cargo','build','--release','--target','wasm32-unknown-unknown','--offline','--locked','--manifest-path',manifest])
if unit == 'STORE':
    review = json.loads((coordinator/'storage-execution-review.json').read_text())
    require(sha((crate/'src/bridge.py').read_bytes()) == review['bridge_sha256'], 'SQLite bridge drift')
    for tool,digest in review['tool_sha256'].items():
        require(sha(Path(tool).read_bytes()) == digest, 'Storage tool drift')
    commands.append(['cargo','build','--release','--offline','--locked','--manifest-path',manifest,'--example','storage_budget'])
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
    if unit in ['UI','NAV'] and receipt['status']=='PASS':
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
