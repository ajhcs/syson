#!/usr/bin/env python3
"""Frozen serial offline checks for postprocessor and rendered canvas implementation units."""
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
    'F12': ('f12-operator-expression', 'syson-ast-adapter', ['syson-ast', 'syson-reflection', 'syson-ast-values', 'syson-ast-dispatch']),
    'F19A': ('f19-layout-controller', 'syson-web', ['syson-model', 'syson-domain', 'syson-metamodel', 'syson-editor', 'syson-workflow', 'syson-initializer-graph', 'syson-initializer', 'syson-node-measurement', 'syson-layout', 'syson-routing']),
    'F19B': ('f19-layout-controller', 'syson-web', ['syson-model', 'syson-domain', 'syson-metamodel', 'syson-editor', 'syson-workflow', 'syson-initializer-graph', 'syson-initializer', 'syson-node-measurement', 'syson-layout', 'syson-routing']),
    'F19C': ('f19-layout-controller', 'syson-web', ['syson-model', 'syson-domain', 'syson-metamodel', 'syson-editor', 'syson-workflow', 'syson-initializer-graph', 'syson-initializer', 'syson-node-measurement', 'syson-layout', 'syson-routing']),
}
folder, name, dependencies = configs[unit]
root = Path('/workspace/worktrees/builderbob/syson') / folder
if not run.startswith(unit.lower() + '-') or '/' in run:
    raise SystemExit('Invalid capture ID')
crate = root / 'rust/crates' / name
sources = [crate] + [crate.parent / dep for dep in dependencies]
base = '1d50b6f988a5ee8847f61a4c76e1d46766bad3a0' if unit == 'F12' else 'a312510b19dd69eb0582043c5edb6829aac7da15'
for source in sources:
    manifest_path = source / 'Cargo.toml'
    original = subprocess.check_output(['git', '-C', str(root), 'show', base + ':' + str(manifest_path.relative_to(root))])
    expected = tomllib.loads(original.decode())
    if source == crate:
        additions = ['syson-ast-dispatch'] if unit == 'F12' else ['syson-node-measurement', 'syson-layout', 'syson-routing']
        expected['dependencies'].update({dep: {'path': '../' + dep} for dep in additions})
    elif manifest_path.read_bytes() != original:
        raise SystemExit('Parent manifest changed')
    manifest = tomllib.loads(manifest_path.read_text())
    if manifest != expected:
        raise SystemExit('Unreviewed manifest edit: ' + source.name)
    if (source / 'build.rs').exists() or manifest.get('package', {}).get('build'):
        raise SystemExit('Unreviewed build script')
lock = tomllib.loads((crate / 'Cargo.lock').read_text())
old_lock = tomllib.loads(subprocess.check_output(['git', '-C', str(root), 'show', base + ':' + str((crate / 'Cargo.lock').relative_to(root))]).decode())
old_registry = {(p['name'], p['version'], p['source']): p for p in old_lock['package'] if p.get('source')}
for package in lock['package']:
    if package.get('source') and old_registry.get((package['name'], package['version'], package['source'])) != package:
        raise SystemExit('Registry lock input changed')
extras = [root / 'rust/browser-tests/layout.cjs'] if unit == 'F19C' else []

def now():
    return datetime.now(timezone.utc).isoformat()

def sha(data):
    return hashlib.sha256(data).hexdigest()

def freeze():
    return {str(p.relative_to(root)): sha(p.read_bytes())
        for source in sources + extras for p in ([source] if source.is_file() else sorted(source.rglob('*')))
        if p.is_file() and 'target' not in p.relative_to(source).parts}

out = root / 'target/batch5-checks' / run
out.mkdir(parents=True, exist_ok=False)
scratch = root / 'target/batch5-scratch' / run
scratch.mkdir(parents=True, exist_ok=False)
env = os.environ.copy()
env.update(CARGO_BUILD_JOBS='1', RUST_TEST_THREADS='1',
    CARGO_TARGET_DIR=str(root / 'target/batch5-build'), TMPDIR=str(scratch))
manifest = str(crate / 'Cargo.toml')
commands = [
    ['cargo', 'fmt', '--manifest-path', manifest, '--', '--check'],
    ['cargo', 'test', '--offline', '--locked', '--manifest-path', manifest, '--', '--test-threads=1'],
    ['cargo', 'clippy', '--offline', '--locked', '--manifest-path', manifest, '--all-targets', '--', '-D', 'warnings'],
]
if unit == 'F19C':
    commands.append(['cargo', 'build', '--release', '--target', 'wasm32-unknown-unknown', '--offline', '--locked', '--manifest-path', manifest])
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
    if unit == 'F19C' and r.get('status') == 'PASS':
        wasm = root / 'target/batch5-build/wasm32-unknown-unknown/release/syson_web.wasm'
        r['wasm_sha256'] = sha(wasm.read_bytes())
    r['source_unchanged'] = r['source_before'] == r['source_after']
    if not r['source_unchanged']:
        r['status'] = 'SOURCE_DRIFT'
    (out / 'receipt.json').write_text(json.dumps(r, indent=2) + '\n')
print(json.dumps({k: r[k] for k in ['unit', 'run', 'status', 'source_unchanged']}))
print(str(out / 'receipt.json'))
sys.exit(0 if r['status'] == 'PASS' else 1)
