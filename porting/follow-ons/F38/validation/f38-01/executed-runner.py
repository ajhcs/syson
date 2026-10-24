#!/usr/bin/env python3
"""Serial offline checks with immutable per-attempt source and output receipts."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib
from datetime import datetime, timezone

unit, run = sys.argv[1:]
config = {
    'F38': ('f38-extensions', 'syson-extensions', ['syson-extensions']),
    'F31': ('f31-snapshot-failures', 'syson-workflow', [
        'syson-workflow', 'syson-model', 'syson-domain', 'syson-metamodel',
        'syson-editor', 'syson-persistence', 'syson-initializer-graph', 'syson-initializer']),
}
folder, crate, closure = config[unit]
if not run.startswith(unit.lower() + '-') or '/' in run:
    raise SystemExit('Invalid capture ID')
root = Path('/workspace/worktrees/builderbob/syson') / folder
sources = [root / 'rust/crates' / name for name in closure]
baseline = 'ad814c232ee24109639c1d52239cbeec6f4a3e92'
for source in sources:
    if (source / 'build.rs').exists():
        raise SystemExit('Unreviewed build script')
    manifest = source / 'Cargo.toml'
    data = tomllib.loads(manifest.read_text())
    if data.get('build-dependencies') or data.get('package', {}).get('build'):
        raise SystemExit('Unreviewed build inputs')
    if source.name != 'syson-extensions':
        expected = subprocess.check_output(['git', '-C', str(root), 'show',
            baseline + ':' + str(manifest.relative_to(root))])
        if manifest.read_bytes() != expected:
            raise SystemExit('Existing manifest changed')
    elif any(data.get(k) for k in ('dependencies', 'dev-dependencies', 'target')):
        raise SystemExit('New crate must be std only')

def stamp():
    return datetime.now(timezone.utc).isoformat()

def digest(data):
    return hashlib.sha256(data).hexdigest()

def freeze():
    return {str(p.relative_to(root)): digest(p.read_bytes())
        for source in sources for p in sorted(source.rglob('*'))
        if p.is_file() and 'target' not in p.relative_to(source).parts}

out = root / 'target/batch2-checks' / run
out.mkdir(parents=True, exist_ok=False)
scratch = root / 'target/batch2-scratch' / run
scratch.mkdir(parents=True, exist_ok=False)
env = os.environ.copy()
env.update(CARGO_BUILD_JOBS='1', RUST_TEST_THREADS='1',
    CARGO_TARGET_DIR=str(root / 'target/batch2-build'),
    TMPDIR=str(scratch), SYSON_TEST_DIR=str(scratch))
manifest = str(root / 'rust/crates' / crate / 'Cargo.toml')
commands = [
    ['cargo', 'fmt', '--manifest-path', manifest, '--', '--check'],
    ['cargo', 'test', '--offline', '--locked', '--manifest-path', manifest, '--', '--test-threads=1'],
    ['cargo', 'clippy', '--offline', '--locked', '--manifest-path', manifest, '--all-targets', '--', '-D', 'warnings'],
]
result = dict(unit=unit, run=run, started_utc=stamp(), source_before=freeze(),
    runner_sha256=digest(Path(__file__).read_bytes()), commands=[],
    upstream_runtime='NOT_RUN', execution='COORDINATOR_SERIAL_ONE_JOB')
try:
    for i, argv in enumerate(commands):
        entry = dict(argv=argv, started_utc=stamp())
        try:
            child = subprocess.run(argv, cwd=root, env=env, capture_output=True, timeout=120)
            code, stdout, stderr = child.returncode, child.stdout, child.stderr
        except subprocess.TimeoutExpired as error:
            code, stdout, stderr = 124, error.stdout or b'', error.stderr or b''
        (out / f'{i:02d}-stdout.txt').write_bytes(stdout)
        (out / f'{i:02d}-stderr.txt').write_bytes(stderr)
        entry.update(completed_utc=stamp(), exit_code=code,
            stdout_sha256=digest(stdout), stderr_sha256=digest(stderr))
        result['commands'].append(entry)
        if code:
            result['status'] = 'FAIL'
            break
    else:
        result['status'] = 'PASS'
finally:
    result.update(completed_utc=stamp(), source_after=freeze())
    result['source_unchanged'] = result['source_before'] == result['source_after']
    if not result['source_unchanged']:
        result['status'] = 'SOURCE_DRIFT'
    (out / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({k: result[k] for k in ('unit', 'run', 'status', 'source_unchanged')}))
print(str(out / 'receipt.json'))
sys.exit(0 if result['status'] == 'PASS' else 1)
