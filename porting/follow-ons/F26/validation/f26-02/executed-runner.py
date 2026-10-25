#!/usr/bin/env python3
"""Capture stdlib-only source checks; never execute a migration or database."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / 'porting/persistence/schema-contract'
RUN = sys.argv[1]
if RUN not in {'f26-01', 'f26-02'}:
    raise ValueError('Unexpected capture ID')
OUT = Path(__file__).with_name('validation') / RUN
OUT.mkdir(parents=True, exist_ok=False)
def stamp():
    return datetime.now(timezone.utc).isoformat()
def digest(raw):
    return hashlib.sha256(raw).hexdigest()
def freeze():
    names = ['plan.py', 'test_plan.py', 'source-manifest.json', 'candidate-plan.json', 'synthetic-plan.json']
    return {str((SOURCE / n).relative_to(ROOT)): digest((SOURCE / n).read_bytes()) for n in names}
r = {'started_utc': stamp(), 'source_base': subprocess.check_output(['git', '-C', str(ROOT), 'rev-parse', 'HEAD'], text=True).strip(),
     'source_before': freeze(), 'runner_sha256': digest(Path(__file__).read_bytes()), 'commands': [], 'database_execution': 'NOT_RUN'}
commands = [[sys.executable, '-B', str(SOURCE / 'plan.py')], [sys.executable, '-B', '-m', 'unittest', 'discover', '-s', str(SOURCE), '-v']]
try:
    for i, argv in enumerate(commands):
        c = {'argv': argv, 'started_utc': stamp()}
        child = subprocess.run(argv, cwd=ROOT, capture_output=True, timeout=30)
        (OUT / f'{i:02d}-stdout.txt').write_bytes(child.stdout)
        (OUT / f'{i:02d}-stderr.txt').write_bytes(child.stderr)
        c.update(completed_utc=stamp(), exit_code=child.returncode, stdout_sha256=digest(child.stdout), stderr_sha256=digest(child.stderr))
        r['commands'].append(c)
        if child.returncode:
            r['status'] = 'FAIL'
            break
    else:
        r['status'] = 'PASS'
finally:
    r.update(completed_utc=stamp(), source_after=freeze())
    r['source_unchanged'] = r['source_before'] == r['source_after']
    if not r['source_unchanged']:
        r['status'] = 'SOURCE_DRIFT'
    (OUT / 'receipt.json').write_text(json.dumps(r, indent=2) + '\n')
print(json.dumps({'status': r['status'], 'receipt': str(OUT / 'receipt.json'), 'source_unchanged': r['source_unchanged']}))
sys.exit(0 if r['status'] == 'PASS' else 1)
