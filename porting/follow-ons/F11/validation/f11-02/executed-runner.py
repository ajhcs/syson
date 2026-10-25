#!/usr/bin/env python3
"""Frozen serial offline checks for two snapshot-based parity implementation units."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib
from datetime import datetime, timezone

unit, run = sys.argv[1:]
configs = {
    'F11': ('f11-ast-adapter', 'syson-ast-adapter', ['syson-ast', 'syson-reflection', 'syson-ast-values']),
    'F18': ('f18-containers', 'syson-layout', ['syson-node-measurement', 'syson-editor']),
}
folder, name, dependencies = configs[unit]
root = Path('/workspace/worktrees/builderbob/syson') / folder
if not run.startswith(unit.lower() + '-') or '/' in run:
    raise SystemExit('Invalid capture ID')
crate = root / 'rust/crates' / name
sources = [crate] + [crate.parent / dep for dep in dependencies]
path = lambda dep: {'path': '../' + dep}
expected = {
    'syson-ast-adapter': ({dep: path(dep) for dep in ['syson-ast', 'syson-reflection', 'syson-ast-values']}, {}),
    'syson-ast': ({}, {}),
    'syson-reflection': ({}, {}),
    'syson-ast-values': ({}, {}),
    'syson-layout': ({}, {'syson-node-measurement': path('syson-node-measurement')}),
    'syson-node-measurement': ({'syson-editor': path('syson-editor')}, {}),
    'syson-editor': ({}, {}),
}
for source in sources:
    manifest = tomllib.loads((source / 'Cargo.toml').read_text())
    if (source / 'build.rs').exists() or manifest.get('package', {}).get('build'):
        raise SystemExit('Unreviewed build script')
    if any(manifest.get(k) for k in ['build-dependencies', 'target']):
        raise SystemExit('Unreviewed dependency section')
    prod, dev = expected[source.name]
    if manifest.get('dependencies', {}) != prod or manifest.get('dev-dependencies', {}) != dev:
        raise SystemExit('Unreviewed dependency closure: ' + source.name)
extras = []

def now():
    return datetime.now(timezone.utc).isoformat()

def sha(data):
    return hashlib.sha256(data).hexdigest()

def freeze():
    return {str(p.relative_to(root)): sha(p.read_bytes())
        for source in sources + extras for p in sorted(source.rglob('*'))
        if p.is_file() and 'target' not in p.relative_to(source).parts}

out = root / 'target/batch4-checks' / run
out.mkdir(parents=True, exist_ok=False)
scratch = root / 'target/batch4-scratch' / run
scratch.mkdir(parents=True, exist_ok=False)
env = os.environ.copy()
env.update(CARGO_BUILD_JOBS='1', RUST_TEST_THREADS='1',
    CARGO_TARGET_DIR=str(root / 'target/batch4-build'), TMPDIR=str(scratch))
manifest = str(crate / 'Cargo.toml')
commands = [
    ['cargo', 'fmt', '--manifest-path', manifest, '--', '--check'],
    ['cargo', 'test', '--offline', '--locked', '--manifest-path', manifest, '--', '--test-threads=1'],
    ['cargo', 'clippy', '--offline', '--locked', '--manifest-path', manifest, '--all-targets', '--', '-D', 'warnings'],
]
r = dict(unit=unit, run=run, started_utc=now(), source_before=freeze(),
    source_base=subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD'], text=True).strip(),
    runner_sha256=sha(Path(__file__).read_bytes()), commands=[],
    upstream_runtime='NOT_RUN', execution='COORDINATOR_SERIAL_ONE_JOB')
try:
    for index, argv in enumerate(commands):
        entry = dict(argv=argv, started_utc=now())
        try:
            result = subprocess.run(argv, cwd=root, env=env, capture_output=True, timeout=120)
            code, stdout, stderr = result.returncode, result.stdout, result.stderr
        except subprocess.TimeoutExpired as e:
            code, stdout, stderr = 124, e.stdout or b'', e.stderr or b''
        (out / f'{index:02d}-stdout.txt').write_bytes(stdout)
        (out / f'{index:02d}-stderr.txt').write_bytes(stderr)
        entry.update(completed_utc=now(), exit_code=code,
            stdout_sha256=sha(stdout), stderr_sha256=sha(stderr))
        r['commands'].append(entry)
        if code:
            r['status'] = 'FAIL'
            break
    else:
        r['status'] = 'PASS'
finally:
    r.update(completed_utc=now(), source_after=freeze())
    r['source_unchanged'] = r['source_before'] == r['source_after']
    if not r['source_unchanged']:
        r['status'] = 'SOURCE_DRIFT'
    (out / 'receipt.json').write_text(json.dumps(r, indent=2) + '\n')
print(json.dumps({k: r[k] for k in ['unit', 'run', 'status', 'source_unchanged']}))
print(str(out / 'receipt.json'))
sys.exit(0 if r['status'] == 'PASS' else 1)
