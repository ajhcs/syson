#!/usr/bin/env python3
"""Coordinator-only frozen offline checks for batch-eight selected-model adapters."""
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
configs = {'F05': ('f05-view-usage', 'syson-view-usage'),
           'F14': ('f14-text-import', 'syson-text-import'),
           'F27': ('f27-document-storage', 'syson-storage')}
folder, name = configs[unit]
if not run.startswith(unit.lower() + '-') or '/' in run:
    raise SystemExit('Invalid capture ID')
root = Path('/workspace/worktrees/builderbob/syson') / folder
crate = root / 'rust/crates' / name
base = {'F05':'46a79b3a265965768220f2976b79a842b05649c1','F14':'aac4d2fd46d6f1c96358fa67bc017a4bcba85c5c','F27':'466517a8f4ba51dec0fa30706a292baeee9eb822'}[unit]
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
extras = []
changed = git('diff', '--name-only').decode().splitlines()
require(not changed, 'Unexpected tracked dependency edits: ' + repr(changed))

def freeze():
    result = {str(p.relative_to(root)): sha(p.read_bytes())
            for source in sources for p in sorted(source.rglob('*'))
            if p.is_file() and 'target' not in p.relative_to(source).parts}
    result.update({str(p.relative_to(root)): sha(p.read_bytes()) for p in extras})
    return result

out = root / 'target/batch8-checks' / run
out.mkdir(parents=True, exist_ok=False)
build = Path('/workspace/build/builderbob/syson-batch8-20261025')
build.mkdir(parents=True, exist_ok=True)
lock = (build / 'compiler.lock').open('a')
fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
scratch = build / 'scratch' / run
scratch.mkdir(parents=True, exist_ok=False)
env = os.environ.copy()
env.update(CARGO_BUILD_JOBS='1', RUST_TEST_THREADS='1', CARGO_INCREMENTAL='0',
           CARGO_TARGET_DIR=str(build / 'target'), TMPDIR=str(scratch))
manifest = str(crate / 'Cargo.toml')
if unit == 'F27':
    review = json.loads((coordinator / 'storage-execution-review.json').read_text())
    require(review['status'] == 'APPROVED_BY_COORDINATOR', 'Storage execution not reviewed')
    require(review['run'] == run, 'Storage run not reviewed')
    for relative, digest in review['source_sha256'].items():
        require(sha((root / relative).read_bytes()) == digest, 'Reviewed storage source changed: ' + relative)
    for path, digest in review['tool_sha256'].items():
        require(sha(Path(path).read_bytes()) == digest, 'Reviewed storage tool changed: ' + path)
commands = [
    ['cargo', 'fmt', '--manifest-path', manifest, '--', '--check'],
    ['cargo', 'test', '--offline', '--locked', '--manifest-path', manifest, '--', '--test-threads=1'],
    ['cargo', 'clippy', '--offline', '--locked', '--manifest-path', manifest, '--all-targets', '--', '-D', 'warnings'],
]
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
    if unit == 'F27':
        receipt['storage_execution_review'] = review
        receipt['storage_tool_sha256_after'] = {path: sha(Path(path).read_bytes()) for path in review['tool_sha256']}
        receipt['retained_database_artifacts'] = {str(p.relative_to(scratch)): dict(bytes=p.stat().st_size, sha256=sha(p.read_bytes())) for p in scratch.rglob('*') if p.is_file()}
        if receipt['storage_tool_sha256_after'] != review['tool_sha256']:
            receipt['status'] = 'TOOL_DRIFT'
    receipt['source_unchanged'] = receipt['source_before'] == receipt['source_after']
    if not receipt['source_unchanged']:
        receipt['status'] = 'SOURCE_DRIFT'
    (out / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({key: receipt[key] for key in ['unit', 'run', 'status', 'source_unchanged']}))
print(out / 'receipt.json')
sys.exit(0 if receipt['status'] == 'PASS' else 1)
