#!/usr/bin/env python3
"""Frozen serial offline checks for three stacked parity implementation units."""
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
    'F03': ('f03-public-imports', 'syson-imports', None),
    'F09': ('f09-ast-dispatch', 'syson-ast-dispatch', 'syson-reflection'),
    'F17': ('f17-routing', 'syson-routing', 'syson-layout'),
}
folder, name, allowed = configs[unit]
root = Path('/workspace/worktrees/builderbob/syson') / folder
if not run.startswith(unit.lower() + '-') or '/' in run:
    raise SystemExit('Invalid capture ID')
crate = root / 'rust/crates' / name
data = tomllib.loads((crate / 'Cargo.toml').read_text())
expected = {} if allowed is None else {allowed: {'path': '../' + allowed}}
if data.get('dependencies', {}) not in [{}, expected]:
    raise SystemExit('Unreviewed dependency')
sources = [crate]
if data.get('dependencies'):
    sources.append(crate.parent / allowed)
for source in sources:
    manifest = tomllib.loads((source / 'Cargo.toml').read_text())
    if (source / 'build.rs').exists() or manifest.get('package', {}).get('build'):
        raise SystemExit('Unreviewed build script')
    if any(manifest.get(k) for k in ['dev-dependencies', 'build-dependencies', 'target']):
        raise SystemExit('Unreviewed dependency section')
    if source != crate and manifest.get('dependencies'):
        raise SystemExit('Parent must remain std only')
extras = [root / 'porting/area-inventories/DOMAIN-QUERY/imports'] if unit == 'F03' else []

def now():
    return datetime.now(timezone.utc).isoformat()

def sha(data):
    return hashlib.sha256(data).hexdigest()

def freeze():
    return {str(p.relative_to(root)): sha(p.read_bytes())
        for source in sources + extras for p in sorted(source.rglob('*'))
        if p.is_file() and 'target' not in p.relative_to(source).parts}

out = root / 'target/batch3-checks' / run
out.mkdir(parents=True, exist_ok=False)
scratch = root / 'target/batch3-scratch' / run
scratch.mkdir(parents=True, exist_ok=False)
env = os.environ.copy()
env.update(CARGO_BUILD_JOBS='1', RUST_TEST_THREADS='1',
    CARGO_TARGET_DIR=str(root / 'target/batch3-build'), TMPDIR=str(scratch))
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
