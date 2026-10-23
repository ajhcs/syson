#!/usr/bin/env python3
"""Capture a bounded integrated Rust checkpoint with real UTC and portable logs."""
import argparse, datetime, hashlib, json, os, re, subprocess
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat().replace('+00:00', 'Z')
def sha(data):
    return hashlib.sha256(data).hexdigest()
p = argparse.ArgumentParser()
p.add_argument('--build-dir', type=Path, required=True)
p.add_argument('--evidence-dir', type=Path, required=True)
p.add_argument('--start-at', default='syson-metamodel', choices=['syson-metamodel', 'syson-model', 'syson-web', 'syson-workflow', 'syson-syntax', 'validation', 'wasm'])
a = p.parse_args()
if not a.build_dir.is_absolute() or not a.build_dir.is_dir():
    p.error('build-dir must be an existing approved absolute directory')
a.evidence_dir.mkdir(exist_ok=False, parents=True)
tests = a.build_dir / 'test-artifacts'
tests.mkdir(exist_ok=True)
env = dict(os.environ, CARGO_BUILD_JOBS='1', CARGO_TARGET_DIR=str(a.build_dir / 'integration'), SYSON_TEST_DIR=str(tests), SYSON_BUILD_DIR=str(a.build_dir), PYTHONDONTWRITEBYTECODE='1')
report = {'source_revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), 'started_at_utc': now(), 'build_jobs': 1, 'commands': [], 'source_sha256': {}}
for base in [ROOT / 'rust/crates', ROOT / 'rust/validation']:
    for path in sorted(base.rglob('*')):
        if path.is_file() and not any(part in ('target', '__pycache__') for part in path.parts):
            report['source_sha256'][str(path.relative_to(ROOT))] = sha(path.read_bytes())
report['runner_sha256'] = sha(Path(__file__).read_bytes())
commands = []
for name in ['syson-metamodel', 'syson-model', 'syson-web', 'syson-workflow', 'syson-syntax']:
    commands.append((name, ['cargo', 'test', '--offline', '--locked', '--jobs', '1', '--manifest-path', f'rust/crates/{name}/Cargo.toml']))
commands += [('validation', ['cargo', 'test', '--offline', '--locked', '--jobs', '1', '--manifest-path', 'rust/validation/Cargo.toml']), ('wasm', ['cargo', 'build', '--offline', '--locked', '--jobs', '1', '--target', 'wasm32-unknown-unknown', '--manifest-path', 'rust/crates/syson-web/Cargo.toml'])]
commands = commands[[name for name, _ in commands].index(a.start_at):]
failed = False
for name, command in commands:
    start = now()
    r = subprocess.run(command, cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    end = now()
    raw = r.stdout
    portable = raw.replace(str(a.build_dir).encode(), b'${SYSON_BUILD_DIR}').replace(str(ROOT).encode(), b'<repository>')
    filename = name + '.log'
    (a.evidence_dir / filename).write_bytes(portable)
    text = portable.decode('utf-8', errors='replace')
    lines = [line for line in text.splitlines() if line.startswith('test result:')]
    count = sum(int(x) for x in re.findall(r'test result: ok\. (\d+) passed;', text))
    report['commands'].append({'name': name, 'argv': command, 'start_utc': start, 'end_utc': end, 'exit_code': r.returncode, 'raw_output_sha256': sha(raw), 'portable_output_sha256': sha(portable), 'output': filename, 'passed_tests': count, 'test_result_lines': lines})
    print(name, r.returncode, count, flush=True)
    if r.returncode:
        print(text, flush=True)
        failed = True
        break
report['end_utc'] = now()
report['status'] = 'FAIL' if failed else 'PASS'
(a.evidence_dir / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
raise SystemExit(1 if failed else 0)
