#!/usr/bin/env python3
"""Capture pinned-contract and generated-data correspondence without compiling."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys
ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).with_name('source-check-01')
OUT.mkdir(exist_ok=False)
paths = [ROOT / p for p in [
    'porting/area-inventories/DOMAIN-QUERY/imports/contract.json',
    'porting/area-inventories/DOMAIN-QUERY/imports/fixtures.json',
    'porting/area-inventories/DOMAIN-QUERY/imports/validate.py',
    'porting/follow-ons/F03/generate_fixtures.py',
    'rust/crates/syson-imports/tests/data/f02_cases.rs']]
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
def freeze():
    return {str(p.relative_to(ROOT)): sha(p) for p in paths}
def now():
    return datetime.now(timezone.utc).isoformat()
r = dict(started_utc=now(), source_before=freeze(), runner_sha256=sha(Path(__file__)), commands=[])
commands = [
    [sys.executable, '-B', str(ROOT / 'porting/area-inventories/DOMAIN-QUERY/imports/validate.py')],
    [sys.executable, '-B', str(ROOT / 'porting/follow-ons/F03/generate_fixtures.py'), '--check']]
for i, command in enumerate(commands):
    c = dict(argv=command, started_utc=now())
    child = subprocess.run(command, cwd=ROOT, capture_output=True, timeout=30)
    out, err = OUT / f'{i:02d}-stdout.txt', OUT / f'{i:02d}-stderr.txt'
    out.write_bytes(child.stdout)
    err.write_bytes(child.stderr)
    c.update(exit_code=child.returncode, completed_utc=now(), stdout_sha256=sha(out), stderr_sha256=sha(err))
    r['commands'].append(c)
r.update(source_after=freeze(), completed_utc=now(), upstream_runtime='NOT_RUN')
r['source_unchanged'] = r['source_before'] == r['source_after']
r['status'] = 'PASS' if r['source_unchanged'] and all(c['exit_code'] == 0 for c in r['commands']) else 'FAIL'
(OUT / 'receipt.json').write_text(json.dumps(r, indent=2) + '\n')
print(json.dumps({'status': r['status'], 'receipt': str(OUT / 'receipt.json')}))
raise SystemExit(0 if r['status'] == 'PASS' else 1)
